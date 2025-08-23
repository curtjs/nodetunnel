pub enum NetworkCommand {
    ConnectToRelay(String)
}

pub enum NetworkEvent {
    ConnectedToRelay(String),
    Error(String),
}
