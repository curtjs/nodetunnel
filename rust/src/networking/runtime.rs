use std::collections::HashMap;
use std::fmt::format;
use godot::global::{godot_error, godot_print};
use tokio::{runtime::Runtime, sync::mpsc, task::JoinHandle};

use crate::{networking::tcp_handler::TcpHandler, types::{commands::NetworkCommand, events::NetworkEvent, relay_state::RelayState}};
use crate::networking::udp_handler::UdpHandler;
use crate::types::packet_types::{PacketParser, PacketType, PeerInfo};
use crate::utils::byte_utils::ByteUtils;

pub struct NetworkingRuntime {
    out_cmds: mpsc::UnboundedSender<NetworkCommand>,
    in_events: mpsc::UnboundedReceiver<NetworkEvent>,
    runtime_handle: Option<std::thread::JoinHandle<()>>,
}

impl NetworkingRuntime {
    pub fn new() -> Result<Self, String> {
        let (out_cmds, in_cmds) = mpsc::unbounded_channel();
        let (out_events, in_events) = mpsc::unbounded_channel();

        let runtime_handle = std::thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create Tokio runtime in thread");
            rt.block_on(async {
                let mut network_core = NetworkCore::new(in_cmds, out_events).await;
                network_core.run().await;
            });
        });

        Ok(Self {
            out_cmds,
            in_events,
            runtime_handle: Some(runtime_handle)
        })
    }

    pub fn send_command(&self, command: NetworkCommand) -> Result<(), String> {
        if let Some(handle) = &self.runtime_handle {
            if handle.is_finished() {
                return Err("Networking task has terminated".to_string())
            }
        }
        
        self.out_cmds.send(command)
            .map_err(|_| "Failed to send command to networking thread".to_string())
    }

    pub fn poll_events(&mut self) -> Vec<NetworkEvent> {
        let mut events = Vec::new();

        while let Ok(event) = self.in_events.try_recv() {
            events.push(event);
        }

        events
    }

    pub fn shutdown(&mut self) {
        if let Some(handle) = self.runtime_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for NetworkingRuntime {
    fn drop(&mut self) {
        let _ = self.out_cmds.send(NetworkCommand::Disconnect);
        self.shutdown();
    }
}

struct NetworkCore {
    in_cmds: mpsc::UnboundedReceiver<NetworkCommand>,
    out_events: mpsc::UnboundedSender<NetworkEvent>,
    state: RelayState,
    tcp_handler: TcpHandler,
    udp_handler: Option<UdpHandler>,
    relay_host: String,
    relay_port: u16,
    online_id_to_numeric: HashMap<String, i32>,
    numeric_to_online_id: HashMap<i32, String>,
}

impl NetworkCore {
    async fn new(
        in_cmds: mpsc::UnboundedReceiver<NetworkCommand>,
        out_events: mpsc::UnboundedSender<NetworkEvent>,
    ) -> Self {
        Self {
            in_cmds,
            out_events: out_events.clone(),
            state: RelayState::Disconnected,
            tcp_handler: TcpHandler::new(out_events),
            udp_handler: None,
            relay_host: String::new(),
            relay_port: 0,
            online_id_to_numeric: HashMap::new(),
            numeric_to_online_id: HashMap::new(),
        }
    }

    async fn run(&mut self) {
        println!("Started NetworkCore");

        loop {
            tokio::select! {
                // Handle commands
                command = self.in_cmds.recv() => {
                    if let Some(command) = command {
                        let should_shutdown = matches!(command, NetworkCommand::Disconnect);
                        self.handle_command(command).await;
                        if should_shutdown { break; }
                    } else {
                        break;
                    }
                }
                
                // Poll TCP for peer list updates (only when hosting/joined)
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)), if matches!(self.state, RelayState::Hosting | RelayState::Joined) => {
                    self.poll_tcp_messages().await;
                }
            }
        }

        println!("NetworkCore shutting down");
    }

    async fn poll_tcp_messages(&mut self) {
        let messages = self.tcp_handler.poll_messages().await;

        for message in messages {
            if let Err(e) = self.handle_tcp_message(message).await {
                println!("Error handling TCP message: {}", e);
            }
        }
    }

    async fn handle_tcp_message(&mut self, data: Vec<u8>) -> Result<(), String> {
        let packet_type = ByteUtils::unpack_u32(&data, 0).ok_or("Missing packet type")?;

        match PacketType::from_u32(packet_type) {
            Some(PacketType::PeerList) => {
                let peer_list = PacketParser::parse_peers(&data)?;
                self.update_peer_mapping(&peer_list.peers);
                let _ = self.out_events.send(NetworkEvent::PeerListUpdated { peer_list: peer_list.peers });
            }
            _ => {
                println!("Received unexpected TCP packet type: {}", packet_type);
            }
        }
        Ok(())
    }

    async fn handle_command(&mut self, command: NetworkCommand) {
        match command {
            NetworkCommand::ConnectToRelay { host, port } => {
                println!("Connecting to relay: {}:{}", host, port);
                self.relay_host = host.clone();
                self.relay_port = port;
                self.handle_connect_to_relay(&host, port).await;
            }
            NetworkCommand::Host {online_id} => {
                println!("Starting to host");
                
                match self.tcp_handler.send_host_request(&*online_id).await {
                    Ok(peer_list) => {
                        if let Err(e) = self.init_udp(&online_id).await {
                            println!("Failed to initialize UDP: {}", e);
                        } else {
                            self.state = RelayState::Hosting;
                            self.update_peer_mapping(&peer_list.peers);
                            let _ = self.out_events.send(NetworkEvent::Hosting { peer_list: peer_list.peers });
                        }
                    }
                    Err(e) => {
                        println!("Failed to send host request: {}", e);
                    }
                }
            }
            NetworkCommand::Join { online_id, host_online_id } => {
                println!("Joining host: {}", host_online_id);
                
                match self.tcp_handler.send_join_request(&*online_id, &*host_online_id).await {
                    Ok(peer_list) => {
                        if let Err(e) = self.init_udp(&online_id).await {
                            println!("Failed to initialize UDP: {}", e);
                        } else {
                            self.state = RelayState::Joined;
                            self.update_peer_mapping(&peer_list.peers);
                            let _ = self.out_events.send(NetworkEvent::Joined { peer_list: peer_list.peers });
                        }
                    }
                    Err(e) => {
                        println!("Failed to send join request: {}", e)
                    }
                }
            }
            NetworkCommand::SendPacket { to_peer, data } => {
                if let Some(udp_handler) = &mut self.udp_handler {
                    if let Err(e) = udp_handler.send_packet(to_peer, data, &self.numeric_to_online_id).await {
                        println!("Failed to send packet: {}", e);
                    }
                } else {
                    println!("UDP not initialized, cannot send packet");
                }
            }
            NetworkCommand::Disconnect => {
                println!("Shutting down networking core");
                return;
            }
        }
    }

    async fn handle_connect_to_relay(&mut self, host: &str, port: u16) {
        self.state = RelayState::Connecting;

        // Connect to TCP
        match self.tcp_handler.connect_tcp(host, port).await {
            Ok(()) => {
                println!("TCP connected, sending connect request...");

                match self.tcp_handler.send_connect_request().await {
                    Ok(online_id) => {
                        println!("Received online ID: {}", online_id);
                        self.state = RelayState::Connected;

                        let _ = self.out_events.send(NetworkEvent::RelayConnected { online_id });
                    }
                    Err(e) => {
                        println!("Failed to get online ID: {}", e);
                        self.state = RelayState::Disconnected;
                        let _ = self.out_events.send(NetworkEvent::Error { message: format!("Relay connect failed: {}", e) });
                    }
                }
            }
            Err(e) => {
                godot_error!("TCP connection failed: {}", e);
                self.state = RelayState::Disconnected;
                let _ = self.out_events.send(NetworkEvent::Error { message: format!("Connection failed: {}", e) });
            }
        }
    }
    
    async fn init_udp(&mut self, online_id: &str) -> Result<(), String> {
        let mut udp_handler = UdpHandler::new(self.out_events.clone());
        udp_handler.connect_udp(&self.relay_host, self.relay_port + 1, online_id.to_string()).await?;
        self.udp_handler = Some(udp_handler);
        Ok(())
    }

    fn update_peer_mapping(&mut self, peer_list: &[PeerInfo]) {
        self.numeric_to_online_id.clear();
        for peer in peer_list {
            self.numeric_to_online_id.insert(peer.numeric_id as i32, peer.online_id.clone());
        }
    }
}