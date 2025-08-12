use std::collections::HashMap;
use godot::classes::multiplayer_peer::{ConnectionStatus, TransferMode};
use godot::classes::{IMultiplayerPeerExtension, MultiplayerPeerExtension};
use godot::global::Error;
use godot::prelude::*;
use crate::networking::runtime::NetworkingRuntime;
use crate::types::packet_data::PacketData;
use crate::types::relay_state::RelayState;

#[derive(GodotClass)]
#[class(tool, base=MultiplayerPeerExtension)]
pub struct NodeTunnelPeer {
    base: Base<MultiplayerPeerExtension>,

    // Networking
    runtime: Option<NetworkingRuntime>,

    // Connection state
    online_id: Option<String>,
    connection_state: RelayState,

    // Multiplayer peer state
    unique_id: i32,
    connection_status: ConnectionStatus,
    target_peer: i32,
    transfer_mode: TransferMode,
    transfer_channel: i32,

    // Packet management
    incoming_packets: Vec<PacketData>,

    // Peer tracking
    connected_peers: HashMap<i32, String>,
}

#[godot_api]
impl IMultiplayerPeerExtension for NodeTunnelPeer {
    fn init(base: Base<MultiplayerPeerExtension>) -> Self {
        Self {
            base,
            runtime: None,
            online_id: None,
            connection_state: RelayState::Disconnected,
            unique_id: 0,
            connection_status: ConnectionStatus::CONNECTING, // CONNECTION_DISCONNECTED
            target_peer: 0,
            transfer_mode: TransferMode::RELIABLE,     // TRANSFER_MODE_RELIABLE
            transfer_channel: 0,
            incoming_packets: Vec::new(),
            connected_peers: std::collections::HashMap::new(),
        }
    }

    fn get_available_packet_count(&self) -> i32 {
        self.incoming_packets.len() as i32
    }

    fn get_max_packet_size(&self) -> i32 {
        1400
    }

    fn get_packet_script(&mut self) -> PackedByteArray {
        if let Some(packet) = self.incoming_packets.pop() {
            PackedByteArray::from(packet.data.as_slice())
        } else {
            PackedByteArray::new()
        }
    }

    fn put_packet_script(&mut self, p_buffer: PackedByteArray) -> Error {
        Error::OK
    }

    fn get_packet_channel(&self) -> i32 {
        if let Some(packet) = self.incoming_packets.first() {
            packet.channel
        } else {
            0
        }
    }

    fn get_packet_mode(&self) -> TransferMode {
        if let Some(packet) = self.incoming_packets.first() {
            packet.mode
        } else {
            TransferMode::RELIABLE
        }
    }

    fn set_transfer_channel(&mut self, p_channel: i32) {
        self.transfer_channel = p_channel;
    }

    fn get_transfer_channel(&self) -> i32 {
        self.transfer_channel
    }

    fn set_transfer_mode(&mut self, p_mode: TransferMode) {
        self.transfer_mode = p_mode;
    }

    fn get_transfer_mode(&self) -> TransferMode {
        self.transfer_mode
    }

    fn set_target_peer(&mut self, p_peer: i32) {
        self.target_peer = p_peer;
    }

    fn get_packet_peer(&self) -> i32 {
        if let Some(packet) = self.incoming_packets.first() {
            packet.from_peer
        } else {
            0
        }
    }

    fn is_server(&self) -> bool {
        self.unique_id == 1
    }

    fn poll(&mut self) {
        todo!()
    }

    fn close(&mut self) {
        todo!()
    }

    fn disconnect_peer(&mut self, p_peer: i32, p_force: bool) {
        todo!()
    }

    fn get_unique_id(&self) -> i32 {
        self.unique_id
    }

    fn is_server_relay_supported(&self) -> bool {
        true
    }

    fn get_connection_status(&self) -> ConnectionStatus {
        self.connection_status
    }
}

#[godot_api]
impl NodeTunnelPeer {
    #[func]
    pub fn connect_to_relay(&mut self, relay_address: String, port: i32) {
        todo!("Implement this")
    }

    #[func]
    pub fn host(&mut self) {
        todo!("Implement this")
    }

    #[func]
    pub fn join(&mut self, host_online_id: String) {
        todo!("Implement this")
    }

    #[func]
    pub fn disconnect_from_relay(&mut self) {
        todo!("Implement this")
    }

    #[func]
    pub fn get_online_id(&self) -> String {
        match &self.online_id {
            Some(id) => id.clone(),
            None => {
                godot_error!("get_online_id() called but not connected to relay!");
                String::new()
            }
        }
    }
}
