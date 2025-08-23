use anyhow::anyhow;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use crate::channels::messages::{NetworkCommand, NetworkEvent};
use crate::networking::packet_type::PacketType;

pub async fn networking_task(
    mut command_receiver: mpsc::UnboundedReceiver<NetworkCommand>,
    event_sender: mpsc::UnboundedSender<NetworkEvent>
) {
    println!("Network task started");

    let mut client = Client::new(event_sender.clone());

    loop {
        tokio::select! {
            Some(cmd) = command_receiver.recv() => {
                if let Err(e) = handle_command(&mut client, cmd).await {
                    let _ = event_sender.send(NetworkEvent::Error(e.to_string()));
                    break;
                }
            }

            result = client.process_network_events() => {
                if let Err(e) = result {
                    let _ = event_sender.send(NetworkEvent::Error(e.to_string()));
                    break;
                }
            }

             else => break,
        }
    }

    // TODO: cleanup
}

async fn handle_command(client: &mut Client, cmd: NetworkCommand) -> anyhow::Result<()> {
    match cmd {
        NetworkCommand::ConnectToRelay(server_addr) => {
            client.connect(server_addr).await?;
        }
    }

    Ok(())
}

pub struct Client {
    event_sender: mpsc::UnboundedSender<NetworkEvent>,
    tcp_stream: Option<TcpStream>
}

impl Client {
    pub fn new(event_sender: mpsc::UnboundedSender<NetworkEvent>) -> Self {
        Self {
            event_sender,
            tcp_stream: None,
        }
    }

    pub async fn connect(&mut self, server_addr: String) -> anyhow::Result<()> {
        println!("Connecting TcpStream to {}...", server_addr);
        let mut stream = TcpStream::connect(&server_addr).await?;
        println!("Connected TcpStream to {}!", server_addr);

        let _ = stream.read_u32().await?; // discard packet length

        if let Some(packet_type) = PacketType::from_u32(stream.read_u32().await?) {
            if !matches!(packet_type, PacketType::OnlineId) {
                return Err(anyhow!("Unexpected packet type!"))
            }
        }

        let string_len = stream.read_u32().await? as usize;

        let mut buffer = vec![0; string_len];
        stream.read_exact(&mut buffer).await?;

        let oid = String::from_utf8(buffer)
            .map_err(|_| anyhow!("Invalid UTF-8"))?;

        self.event_sender.send(NetworkEvent::ConnectedToRelay(oid))?;
        
        Ok(())
    }

    pub async fn process_network_events(&mut self) -> anyhow::Result<()> {
        // incoming packets, peer connections, etc.
        Ok(())
    }
}
