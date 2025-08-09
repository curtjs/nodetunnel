/// Commands sent from the main thread -> networking thread
#[derive(Debug, Clone)]
pub enum NetworkCommand {
    ConnectToRelay { host: String, port: u16 },
    Host,
    Join { host_oid: String },
    SendPacket { to_peer: i32, data: Vec<u8> },
    Disconnect,
}
