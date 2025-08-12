use godot::classes::multiplayer_peer::TransferMode;

#[derive(Debug, Clone)]
pub struct PacketData {
    pub data: Vec<u8>,
    pub from_peer: i32,
    pub channel: i32,
    pub mode: TransferMode,
}

impl PacketData {
    pub fn new(data: Vec<u8>, from_peer: i32, channel: i32, mode: TransferMode) -> Self {
        Self {
            data,
            from_peer,
            channel,
            mode,
        }
    }
}