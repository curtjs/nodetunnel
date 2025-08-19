/// Different types of commands to *send*
/// to the network thread.
pub enum NetworkCommand {
    ConnectToRelay(String, u16)
}

/// Different types of events that will be *received*
/// from the network thread.
pub enum NetworkEvent {
    ConnectedToRelay(String),
    ConnectionFailed(String),
}
