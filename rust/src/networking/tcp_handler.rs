use std::fmt::format;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, sync::mpsc};

use crate::{types::{events::NetworkEvent, packet_types::PacketType}, utils::byte_utils::ByteUtils};
use crate::types::packet_types::{PacketBuilder, PacketParser, PeerListResponse};

pub struct TcpHandler {
    stream: Option<TcpStream>,
    out_events: mpsc::UnboundedSender<NetworkEvent>
}

impl TcpHandler {
    pub fn new(out_events: mpsc::UnboundedSender<NetworkEvent>) -> Self {
        Self {
            stream: None,
            out_events,
        }
    }

    pub async fn connect_tcp(&mut self, host: &str, port: u16) -> Result<(), String> {
        let addr = format!("{}:{}", host, port);

        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                println!("Connected to TCP: {}", addr);
                self.stream = Some(stream);
                Ok(())
            }
            Err(e) => Err(format!("Failed to connect to {}: {}", addr, e))
        }
    }

    pub async fn send_connect_request(&mut self) -> Result<String, String> {
        let packet = PacketBuilder::build_connect();
        self.send_packet(packet).await?;

        let response = self.read_packet().await?;
        let packet_type = ByteUtils::unpack_u32(&response, 0).ok_or("Missing packet type")?;
        
        let connect_res = match PacketType::from_u32(packet_type) {
            Some(PacketType::ConnectToRelay) => {
                PacketParser::parse_connect(&response)?
            }
            _ => {
                return Err(format!("Unexpected packet type {}", packet_type))
            }
        };

        Ok(connect_res.online_id)
    }

    pub async fn send_host_request(&mut self, online_id: &str) -> Result<PeerListResponse, String> {
        let packet = PacketBuilder::build_host(online_id);
        self.send_packet(packet).await?;

        let response = self.read_packet().await?;
        let packet_type = ByteUtils::unpack_u32(&response, 0).ok_or("Missing packet type")?;

        let plist_res = match PacketType::from_u32(packet_type) {
            Some(PacketType::HostGame) => {
                PacketParser::parse_peers(&response)?
            }
            _ => {
                return Err(format!("Unexpected packet type {}", packet_type))
            }
        };

        Ok(plist_res)
    }
    
    pub async fn send_join_request(&mut self, online_id: &str, host_online_id: &str) -> Result<PeerListResponse, String> {
        let packet = PacketBuilder::build_join(online_id, host_online_id);
        self.send_packet(packet).await?;
        
        let response = self.read_packet().await?;
        let packet_type = ByteUtils::unpack_u32(&response, 0).ok_or("Missing packet type")?;
        
        let plist_res = match PacketType::from_u32(packet_type) {
            Some(PacketType::JoinGame) => {
                PacketParser::parse_peers(&response)?
            }
            _ => {
                return Err(format!("Unexpected packet type {}", packet_type))
            }
        };
        
        Ok(plist_res)
    }
    
    async fn send_packet(&mut self, packet: Vec<u8>) -> Result<(), String> {
        if let Some(stream) = &mut self.stream {
            let packet_len = ByteUtils::pack_u32(packet.len() as u32);

            stream.write_all(&packet_len).await
                .map_err(|e| format!("Failed to send length: {}", e))?;
            stream.write_all(&packet).await
                .map_err(|e| format!("Failed to send packet: {}", e))?;

            Ok(())
        } else {
            Err("Not connected to TCP".to_string())
        }
    }

    async fn read_packet(&mut self) -> Result<Vec<u8>, String> {
        if let Some(stream) = &mut self.stream {
            let mut len_bytes = [0u8; 4];
            stream.read_exact(&mut len_bytes).await
                .map_err(|e| format!("Failed to read length: {}", e))?;

            let len = ByteUtils::unpack_u32(&len_bytes, 0)
                .ok_or("Invalid length header")? as usize;
            
            let mut packet = vec![0u8; len];
            stream.read_exact(&mut packet).await
                .map_err(|e| format!("Failed to read packet: {}", e))?;

            Ok(packet)
        } else {
            Err("Not connected to TCP".to_string())
        }
    }
}
