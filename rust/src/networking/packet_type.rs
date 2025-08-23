pub enum PacketType {
    OnlineId = 0,
}

impl PacketType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(PacketType::OnlineId),
            _ => None,
        }
    }
}