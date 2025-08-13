use crate::types::packet_types::{PeerInfo};

/// Events sent from networking thread -> main thread
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    RelayConnected { online_id: String },
    Hosting { peer_list: Vec<PeerInfo> },
    Joined { peer_list: Vec<PeerInfo> },
    PacketReceived { from_online_id: String, data: Vec<u8> },
    PeerConnected { peer_id: i32 },
    PeerDisconnected { peer_id: i32 },
    Error { message: String },
    PeerListUpdated { peer_list: Vec<PeerInfo> }
}
