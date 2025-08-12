/// Commands sent from the main thread -> networking thread
#[derive(Debug, Clone)]
pub enum NetworkCommand {
    ConnectToRelay { host: String, port: u16 },
    Host { online_id: String },
    Join { online_id: String, host_online_id: String },
    SendPacket { to_peer: i32, data: Vec<u8> },
    Disconnect,
}
