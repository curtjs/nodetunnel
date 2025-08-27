pub enum NetworkCommand {
    ConnectToRelay(String),
    Host,
    Join(String),
}

pub enum NetworkEvent {
    ConnectedToRelay(String),
    Error(String),
    ConnectedToRoom(u32),
    PeerList(Vec<u32>)
}
