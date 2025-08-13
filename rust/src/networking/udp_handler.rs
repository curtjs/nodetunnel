use std::collections::HashMap;
use std::sync::Arc;
use tokio::{net::UdpSocket, sync::mpsc};
use crate::{types::events::NetworkEvent, utils::byte_utils::ByteUtils};

pub struct UdpHandler {
    socket: Option<Arc<UdpSocket>>,
    relay_host: String,
    relay_port: u16,
    online_id: String,
    out_events: mpsc::UnboundedSender<NetworkEvent>,
}

impl UdpHandler {
    pub fn new(out_events: mpsc::UnboundedSender<NetworkEvent>) -> Self {
        Self {
            socket: None,
            relay_host: String::new(),
            relay_port: 0,
            online_id: String::new(),
            out_events,
        }
    }

    pub async fn connect_udp(&mut self, host: &str, port: u16, online_id: String) -> Result<(), String> {
        self.relay_host = host.to_string();
        self.relay_port = port;
        self.online_id = online_id;

        let socket = UdpSocket::bind("0.0.0.0:0").await
            .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;

        socket.connect(format!("{}:{}", host, port)).await
            .map_err(|e| format!("Failed to connect UDP: {}", e))?;

        let arc_socket = Arc::new(socket);
        self.socket = Some(arc_socket.clone());

        // Send UDP_CONNECT message
        self.send_udp_connect().await?;

        // Spawn receiving task
        let receive_socket = arc_socket.clone();
        let events_sender = self.out_events.clone();
        let online_id_clone = self.online_id.clone();

        tokio::spawn(async move {
            Self::receive_loop(receive_socket, events_sender, online_id_clone).await;
        });

        Ok(())
    }

    async fn receive_loop(
        socket: Arc<UdpSocket>,
        out_events: mpsc::UnboundedSender<NetworkEvent>,
        online_id: String
    ) {
        let mut buffer = [0u8; 1500];

        loop {
            match socket.recv(&mut buffer).await {
                Ok(len) => {
                    if let Err(e) = Self::handle_packet_static(&buffer[..len], &out_events, &online_id).await {
                        println!("Error handling UDP packet: {}", e);
                    }
                }
                Err(e) => {
                    println!("UDP receive error: {}", e);
                    break;
                }
            }
        }
    }

    async fn send_udp_connect(&mut self) -> Result<(), String> {
        self.send_raw_packet("SERVER", b"UDP_CONNECT").await
    }

    pub async fn send_packet(&mut self, to_peer: i32, data: Vec<u8>, peer_mapping: &HashMap<i32, String>) -> Result<(), String> {
        let target_oid = if to_peer == 0 {
            "0".to_string()
        } else {
            peer_mapping.get(&to_peer)
                .cloned()
                .unwrap_or_else(|| {
                    println!("Warning: Unknown peer ID {}, using fallback", to_peer);
                    format!("{}", to_peer)
                })
        };

        self.send_raw_packet(&target_oid, &data).await
    }

    async fn send_raw_packet(&mut self, to_oid: &str, data: &[u8]) -> Result<(), String> {
        if let Some(socket) = &self.socket {
            let mut packet = Vec::new();

            // Sender OID length and data
            packet.extend(ByteUtils::pack_u32(self.online_id.len() as u32));
            packet.extend(self.online_id.as_bytes());

            // Target OID length and data
            packet.extend(ByteUtils::pack_u32(to_oid.len() as u32));
            packet.extend(to_oid.as_bytes());

            // Game data
            packet.extend(data);

            println!("📤 Sending UDP packet to '{}': {} bytes", to_oid, data.len());
            socket.send(&packet).await
                .map_err(|e| format!("Failed to send UDP packet: {}", e))?;
            println!("✅ UDP packet sent successfully");

            Ok(())
        } else {
            Err("UDP socket not connected".to_string())
        }
    }

    async fn handle_packet_static(
        data: &[u8],
        out_events: &mpsc::UnboundedSender<NetworkEvent>,
        _online_id: &str
    ) -> Result<(), String> {
        if data.len() < 8 { return Ok(()); }

        let mut offset = 0;

        // Parse sender OID
        let sender_len = ByteUtils::unpack_u32(data, offset).ok_or("Invalid sender length")?;
        offset += 4;

        if offset + sender_len as usize > data.len() { return Ok(()); }
        let sender_oid = String::from_utf8(data[offset..offset + sender_len as usize].to_vec())
            .map_err(|_| "Invalid sender OID")?;
        println!("📥 UDP packet from: '{}'", sender_oid);
        offset += sender_len as usize;

        // Parse target OID (skip it, we know it's for us)
        let target_len = ByteUtils::unpack_u32(data, offset).ok_or("Invalid target length")?;
        offset += 4 + target_len as usize;

        if offset > data.len() { return Ok(()); }

        // Game data
        let game_data = data[offset..].to_vec();

        // Handle server responses or forward to game
        if sender_oid == "SERVER" {
            if game_data == b"UDP_CONNECT_RES" {
                println!("UDP connected successfully");
            }
        } else {
            // This is game data, send to main thread
            println!("📨 Forwarding game data to main thread: {} bytes from {}", game_data.len(), sender_oid);
            let _ = out_events.send(NetworkEvent::PacketReceived {
                from_online_id: sender_oid,
                data: game_data,
            });
        }

        Ok(())
    }
}