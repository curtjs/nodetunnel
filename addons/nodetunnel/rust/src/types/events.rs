/// Events sent from networking thread -> main thread
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    RelayConnected { online_id: String },
    Hosting,
    Joined,
    PacketReceived { from_peer: i32, data: Vec<u8> },
    PeerConnected { peer_id: i32 },
    PeerDisconnected { peer_id: i32 },
    Error { message: String },
}
