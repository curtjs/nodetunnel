use std::fmt::format;
use godot::global::{godot_error, godot_print};
use tokio::{runtime::Runtime, sync::mpsc, task::JoinHandle};

use crate::{networking::tcp_handler::TcpHandler, types::{commands::NetworkCommand, events::NetworkEvent, relay_state::RelayState}};

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
        self.shutdown();
    }
}

struct NetworkCore {
    in_cmds: mpsc::UnboundedReceiver<NetworkCommand>,
    out_events: mpsc::UnboundedSender<NetworkEvent>,
    state: RelayState,
    tcp_handler: TcpHandler,
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
            // online_id: None,
            tcp_handler: TcpHandler::new(out_events),
        }
    }

    async fn run(&mut self) {
        println!("Started NetworkCore");
        
        while let Some(command) = self.in_cmds.recv().await {
            self.handle_command(command).await;
        }
    }

    async fn handle_command(&mut self, command: NetworkCommand) {
        match command {
            NetworkCommand::ConnectToRelay { host, port } => {
                println!("Connecting to relay: {}:{}", host, port);

                self.handle_connect_to_relay(&host, port).await;
            }
            NetworkCommand::Host {online_id} => {
                println!("Starting to host");
                
                match self.tcp_handler.send_host_request(&*online_id).await {
                    Ok(peer_list) => {
                        self.state = RelayState::Hosting;
                        
                        let _ = self.out_events.send(NetworkEvent::Hosting { peer_list: peer_list.peers });
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
                        self.state = RelayState::Joined;
                        
                        let _ = self.out_events.send(NetworkEvent::Joined { peer_list: peer_list.peers });
                    }
                    Err(e) => {
                        println!("Failed to send join request: {}", e)
                    }
                }
            }
            NetworkCommand::SendPacket { to_peer, data } => {
                println!("Sending packet to peer {}: {} bytes", to_peer, data.len());
            }
            NetworkCommand::Disconnect => {
                println!("Disconnecting")
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
}