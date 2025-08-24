pub enum NetworkCommand {
    ConnectToRelay(String),
    Host,
}

pub enum NetworkEvent {
    ConnectedToRelay(String),
    Error(String),
}
