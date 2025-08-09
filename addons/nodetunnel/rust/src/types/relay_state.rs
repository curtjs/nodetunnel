/// An enum to handle what state this client is in.
/// Used for relay logic only.
/// 
/// **Not related to `MultiplayerPeer.ConnectionStatus`**
#[derive(Debug, Clone, PartialEq)]
pub enum RelayState {
    Disconnected,
    Connecting,
    Connected,
    Hosting,
    Joined
}
