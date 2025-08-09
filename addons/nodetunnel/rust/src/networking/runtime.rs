use tokio::{runtime::Runtime, sync::mpsc, task::JoinHandle};

use crate::types::{commands::NetworkCommand, events::NetworkEvent, relay_state::RelayState};

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
}

impl NetworkCore {
    async fn new(
        in_cmds: mpsc::UnboundedReceiver<NetworkCommand>,
        out_events: mpsc::UnboundedSender<NetworkEvent>,
    ) -> Self {
        Self {
            in_cmds,
            out_events,
            state: RelayState::Disconnected,
            online_id: None,
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

                let _ = self.out_events.send(NetworkEvent::RelayConnected { online_id: "TEST123".to_string() });
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
}