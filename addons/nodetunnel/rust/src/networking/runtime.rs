use godot::global::godot_print;
use tokio::{runtime::Runtime, sync::mpsc, task::JoinHandle};

use crate::{networking::tcp_handler::TcpHandler, types::{commands::NetworkCommand, events::NetworkEvent, relay_state::RelayState}};

pub struct NetworkingRuntime {
    runtime: Runtime,
    out_cmds: mpsc::UnboundedSender<NetworkCommand>,
    in_events: mpsc::UnboundedReceiver<NetworkEvent>,
    runtime_handle: Option<JoinHandle<()>>,
}

impl NetworkingRuntime {
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create tokio runtime");

        let (out_cmds, in_cmds) = mpsc::unbounded_channel();
        let (out_events, in_events) = mpsc::unbounded_channel();

        let runtime_handle = runtime.spawn(async move {
            let mut network_core = NetworkCore::new(in_cmds, out_events).await;
            network_core.run().await;
        });

        Self {
            runtime,
            out_cmds,
            in_events,
            runtime_handle: Some(runtime_handle)
        }
    }

    pub fn send_command(&self, command: NetworkCommand) -> Result<(), String> {
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
            handle.abort();
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
    online_id: Option<String>,
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
            online_id: None,
            tcp_handler: TcpHandler::new(out_events),
        }
    }

    async fn run(&mut self) {
        println!("NetworkCore Started");

        while let Some(command) = self.in_cmds.recv().await {
            self.handle_command(command).await;
        }

        println!("NetworkCore Stopped")
    }

    async fn handle_command(&mut self, command: NetworkCommand) {
        match command {
            NetworkCommand::ConnectToRelay { host, port } => {
                println!("Connecting to relay: {}:{}", host, port);

                self.handle_connect_to_relay(&host, port).await;
            }
            NetworkCommand::Host => {
                println!("Starting to host");

                let _ = self.out_events.send(NetworkEvent::Hosting);
            }
            NetworkCommand::Join { host_oid } => {
                println!("Joining host: {}", host_oid);
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
        match self.tcp_handler.connect(host, port).await {
            Ok(()) => {
                println!("TCP connected, sending connect request...");

                match self.tcp_handler.send_connect_request().await {
                    Ok(online_id) => {
                        println!("Received online ID: {}", online_id);
                        self.state = RelayState::Connected;
                        self.online_id = Some(online_id.clone());

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
                println!("TCP connection failed: {}", e);
                self.state = RelayState::Disconnected;
                let _ = self.out_events.send(NetworkEvent::Error { message: format!("Connection failed: {}", e) });
            }
        }
    }
}