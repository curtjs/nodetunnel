pub enum PacketType {
    Connect = 0,
    Host = 1,
    Join = 2,
    RoomConnect = 3,
}

impl PacketType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(PacketType::Connect),
            1 => Some(PacketType::Host),
            2 => Some(PacketType::Join),
            3 => Some(PacketType::RoomConnect),
            _ => None,
        }
    }
}
