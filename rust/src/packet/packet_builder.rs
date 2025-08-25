use crate::packet::packet_type::PacketType;
use crate::utils::byte_utils::ByteUtils;

pub struct PacketBuilder;

impl PacketBuilder {
    pub fn build_connect() -> Vec<u8> {
        ByteUtils::pack_u32(PacketType::Connect as u32)
    }
    
    pub fn build_host() -> Vec<u8> {
        ByteUtils::pack_u32(PacketType::Host as u32)
    }

    pub fn build_join(host_online_id: String) -> Vec<u8> {
        let mut packet = ByteUtils::pack_u32(PacketType::Join as u32);
        packet.extend(ByteUtils::pack_str(host_online_id.as_str()));
        packet
    }
}
