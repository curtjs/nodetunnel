use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, sync::mpsc};

use crate::{types::{events::NetworkEvent, packet_types::PacketType}, utils::byte_utils::ByteUtils};

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

    pub async fn connect(&mut self, host: &str, port: u16) -> Result<(), String> {
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
        if let Some(stream) = &mut self.stream {
            let packet = ByteUtils::pack_u32(PacketType::ConnectToRelay as u32);
            let packet_len = ByteUtils::pack_u32(packet.len() as u32);

            stream.write_all(&packet_len).await
                .map_err(|e| format!("Failed to send length: {}", e))?;
            stream.write_all(&packet).await
                .map_err(|e| format!("Failed to send packet: {}", e))?;

            let response = self.read_packet().await?;
            let online_id = self.parse_connect_response(&response)?;

            Ok(online_id)
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

    fn parse_connect_response(&self, data: &[u8]) -> Result<String, String> {
        if data.len() < 8 {
            return Err("Response too short".to_string());
        }

        let oid_len = ByteUtils::unpack_u32(data, 4)
            .ok_or("Invalid OID length")? as usize;

        if data.len() < 8 + oid_len {
            return Err("Response truncated".to_string());
        }

        let oid_bytes = &data[8..8 + oid_len];
        String::from_utf8(oid_bytes.to_vec())
            .map_err(|_| "Invalid UTF-8 in OID".to_string())
    }
}
