use tokio::sync::mpsc::{Sender, UnboundedReceiver};
use crate::network::network_messages::{NetworkCommand, NetworkEvent};
use crate::tcp::tcp_client::TcpClient;

pub struct NetworkManager {
    tcp_client: Option<TcpClient>
}

impl NetworkManager {
    pub fn new() -> Self {
        NetworkManager {
            tcp_client: None,
        }
    }

    pub async fn run(
        mut rx: UnboundedReceiver<NetworkCommand>,
        tx: Sender<NetworkEvent>
    ) {
        let mut manager = Self::new();
        
        loop {
            tokio::select! {
                command = rx.recv() => {
                    match command {
                        Some(cmd) => {
                            manager.handle_command(cmd, &tx).await;
                        }
                        None => {
                            println!("Command channel closed");
                            break;
                        }
                    }
                }
                
                _ = async {
                    if let Some(client) = &mut manager.tcp_client {
                        // incoming packets from tcp stream
                    } else {
                        tokio::task::yield_now().await;
                    }
                } => {
                    // runs after async completes
                }
            }
        }
    }
    
    async fn handle_command(
        &mut self,
        command: NetworkCommand,
        tx: &Sender<NetworkEvent>
    ) {
        match command {
            NetworkCommand::ConnectToRelay(address, port) => {
                let client = TcpClient::new(address, port);
                
                match client.connect().await { 
                    Ok(_) => {
                        let _ = tx.send(NetworkEvent::ConnectedToRelay("".to_string())).await;
                        self.tcp_client = Some(client);
                    }
                    Err(e) => {
                        let _ = tx.send(NetworkEvent::ConnectionFailed(e.to_string()));
                    }
                }
            }
        }
    }
}
