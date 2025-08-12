use crate::utils::byte_utils::ByteUtils;

/// Packet types matching the relay server
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum PacketType {
    ConnectToRelay = 0,
    HostGame = 1,
    JoinGame = 2,
    PeerList = 3
}

impl PacketType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(PacketType::ConnectToRelay),
            1 => Some(PacketType::HostGame),
            2 => Some(PacketType::JoinGame),
            3 => Some(PacketType::ConnectToRelay),
            _ => None,
        }
    }
}

// Parsed packet data structures

#[derive(Debug, Clone)]
pub struct ConnectResponse {
    pub online_id: String,
}

#[derive(Debug, Clone)]
pub struct HostRequest {
    pub online_id: String,
}

#[derive(Debug, Clone)]
pub struct JoinRequest {
    pub joiner_id: String,
    pub host_id: String,
}

#[derive(Debug, Clone)]
pub struct PeerListResponse {
    pub peers: Vec<PeerInfo>,
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub online_id: String,
    pub numeric_id: u32,
}

// Packet parser utilities

pub struct PacketBuilder;

impl PacketBuilder {
    pub fn build_connect() -> Vec<u8> {
        ByteUtils::pack_u32(PacketType::ConnectToRelay as u32)
    }

    pub fn build_host(online_id: &str) -> Vec<u8> {
        let mut packet = ByteUtils::pack_u32(PacketType::HostGame as u32);
        packet.extend(ByteUtils::pack_str(online_id));
        packet
    }
    
    pub fn build_join(online_id: &str, host_online_id: &str) -> Vec<u8> {
        let mut packet = ByteUtils::pack_u32(PacketType::JoinGame as u32);
        packet.extend(ByteUtils::pack_str(online_id));
        packet.extend(ByteUtils::pack_str(host_online_id));
        packet
    }
}

pub struct PacketParser;

impl PacketParser {
    pub fn parse_connect(data: &[u8]) -> Result<ConnectResponse, String> {
        if data.len() < 8 {
            return Err("Connect response too short".to_string());
        }

        let (online_id, _) = ByteUtils::unpack_str(data, 4)
            .ok_or("Failed to parse online ID")?;

        Ok(ConnectResponse { online_id })
    }
    
    pub fn parse_peers(data: &[u8]) -> Result<PeerListResponse, String> {
        let mut offset = 4;
        
        let peer_count = ByteUtils::unpack_u32(data, offset)
            .ok_or("Peer list missing peer count")?;
        
        let mut peers = Vec::with_capacity(peer_count as usize);
        
        offset += 4;
        
        for _ in 0..peer_count {
            let (online_id, new_offset) = ByteUtils::unpack_str(data, offset)
                .ok_or("Failed to read peer online ID")?;
            offset = new_offset;
            
            let numeric_id = ByteUtils::unpack_u32(data, offset)
                .ok_or("Failed to read peer numeric ID")?;
            offset += 4;
            
            peers.push(PeerInfo {
                online_id,
                numeric_id
            });
        }
        
        Ok(PeerListResponse { peers })
    }
}
