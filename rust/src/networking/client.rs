use std::mem;
use anyhow::{anyhow, Error};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use crate::channels::messages::{NetworkCommand, NetworkEvent};
use crate::packet::packet_builder::PacketBuilder;
use crate::packet::packet_type::PacketType;
use crate::utils::byte_utils::ByteUtils;

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
            client.send_connect_req(server_addr).await?;
        }
        NetworkCommand::Host => {
            client.send_host_req().await?;
        }
    }

    Ok(())
}

enum ClientState {
    Disconnected,
    Connecting { stream: TcpStream },
    Connected { stream: TcpStream, online_id: String }
}

pub struct Client {
    event_sender: mpsc::UnboundedSender<NetworkEvent>,
    state: ClientState,
}

impl Client {
    pub fn new(event_sender: mpsc::UnboundedSender<NetworkEvent>) -> Self {
        Self {
            event_sender,
            state: ClientState::Disconnected
        }
    }

    pub async fn send_connect_req(&mut self, server_addr: String) -> anyhow::Result<()> {
        println!("Connecting TcpStream to {}...", server_addr);
        let stream = TcpStream::connect(&server_addr).await?;
        println!("Connected TcpStream to {}!", server_addr);

        self.state = ClientState::Connecting { stream };
        
        let packet = PacketBuilder::build_connect();
        self.send_packet(packet).await?;

        Ok(())
    }

    pub async fn handle_connect_res(&mut self, packet: &[u8]) -> anyhow::Result<()> {
        let (online_id, _) = ByteUtils::unpack_str(&packet, 0)
            .ok_or(anyhow!("Invalid UTF-8"))?;

        if let ClientState::Connecting { stream } = mem::replace(&mut self.state, ClientState::Disconnected) {
            self.state = ClientState::Connected { stream, online_id: online_id.clone() }
        }

        self.event_sender.send(NetworkEvent::ConnectedToRelay(online_id))
            .map_err(|e| anyhow!("Failed to send event to main thread: {}", e))?;

        Ok(())
    }

    pub async fn send_host_req(&mut self) -> anyhow::Result<()> {
        let packet = PacketBuilder::build_host();
        self.send_packet(packet).await?;
        
        Ok(())
    }

    pub async fn process_network_events(&mut self) -> anyhow::Result<()> {
        match &mut self.state {
            ClientState::Connecting { .. } => {
                let packet = self.read_packet().await?;
                let mut offset = 0;

                let packet_type_u32 = ByteUtils::unpack_u32(&packet, offset)
                    .ok_or(anyhow!("Failed to read packet type"))?;

                offset += 4;

                let packet_type = PacketType::from_u32(packet_type_u32)
                    .ok_or(anyhow!("Received invalid packet type"))?;
                
                match packet_type {
                    PacketType::Connect => {
                        self.handle_connect_res(&packet[offset..]).await?;
                    },
                    _ => return Err(anyhow!("Unexpected packet type in connecting state")),
                }
            }
            ClientState::Connected { .. } => {
                
            }
            _ => {}
        }

        Ok(())
    }

    /// Reads a complete packet from the stream
    /// Does not include the packet length
    pub async fn read_packet(&mut self) -> Result<Vec<u8>, Error> {
        let stream = self.get_stream()?;

        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await
            .map_err(|e| anyhow!("Failed to read length: {}", e))?;

        let len = ByteUtils::unpack_u32(&len_bytes, 0)
            .ok_or(anyhow!("Invalid length header"))? as usize;

        let mut packet = vec![0u8; len];
        stream.read_exact(&mut packet).await
            .map_err(|e| anyhow!("Failed to read packet: {}", e))?;

        Ok(packet)
    }

    async fn send_packet(&mut self, packet: Vec<u8>) -> Result<(), Error> {
        let stream = self.get_stream()?;
        let packet_len = ByteUtils::pack_u32(packet.len() as u32);

        stream.write_all(&packet_len).await
            .map_err(|e| anyhow!("Failed to send length: {}", e))?;
        stream.write_all(&packet).await
            .map_err(|e| anyhow!("Failed to send packet: {}", e))?;

        Ok(())
    }

    fn get_stream(&mut self) -> Result<&mut TcpStream, Error> {
        match &mut self.state {
            ClientState::Connecting { stream } |
            ClientState::Connected { stream, .. } => Ok(stream),
            ClientState::Disconnected => Err(anyhow!("Not connected to relay")),
        }
    }
}
