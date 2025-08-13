use std::collections::{HashMap, HashSet};
use godot::classes::multiplayer_peer::{ConnectionStatus, TransferMode};
use godot::classes::{IMultiplayerPeerExtension, MultiplayerPeerExtension};
use godot::global::Error;
use godot::prelude::*;
use crate::networking::runtime::NetworkingRuntime;
use crate::types::commands::NetworkCommand;
use crate::types::events::NetworkEvent;
use crate::types::packet_data::PacketData;
use crate::types::packet_types::PeerInfo;
use crate::types::relay_state::RelayState;

#[derive(GodotClass)]
#[class(tool, base=MultiplayerPeerExtension)]
pub struct NodeTunnelPeer {
    base: Base<MultiplayerPeerExtension>,

    // Networking
    runtime: Option<NetworkingRuntime>,

    // Connection state
    online_id: Option<String>,
    relay_connection_state: RelayState,

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
            relay_connection_state: RelayState::Disconnected,
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
        if !matches!(self.relay_connection_state, RelayState::Hosting | RelayState::Joined) {
            return Error::ERR_UNCONFIGURED;
        }

        let data = p_buffer.to_vec();

        if let Some(runtime) = &self.runtime {
            if let Err(e) = runtime.send_command(NetworkCommand::SendPacket {
                to_peer: self.target_peer,
                data,
            }) {
                godot_error!("Failed to send packet: {}", e);
                return Error::FAILED;
            }
            Error::OK
        } else {
            Error::ERR_UNCONFIGURED
        }
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
        if let Some(runtime) = &mut self.runtime {
            let events = runtime.poll_events();

            for event in events {
                match event {
                    NetworkEvent::RelayConnected {online_id} => {
                        self.online_id = Some(online_id.clone());
                        self.relay_connection_state = RelayState::Connected;

                        self.signals().relay_connected().emit(&online_id);
                    }
                    NetworkEvent::Hosting { peer_list } => {
                        self.unique_id = 1;
                        self.update_peer_list(peer_list);
                        self.connection_status = ConnectionStatus::CONNECTED;
                        self.signals().hosting().emit();
                    }
                    NetworkEvent::Joined { peer_list } => {
                        if let Some(our_peer) = peer_list.iter().find(|p| Some(&p.online_id) == self.online_id.as_ref()) {
                            self.unique_id = our_peer.numeric_id as i32;
                        }
                        self.update_peer_list(peer_list);
                        self.connection_status = ConnectionStatus::CONNECTED;
                        self.signals().joined().emit();
                    }
                    NetworkEvent::PeerListUpdated { peer_list } => {
                        // Update peer mappings and emit connection/disconnection signals
                        self.update_peer_list(peer_list);
                    }
                    NetworkEvent::PacketReceived { from_online_id, data } => {
                        println!("🎯 Processing packet from '{}': {} bytes", from_online_id, data.len());

                        // Map online_id to numeric_id using our peer list
                        if let Some(&from_peer) = self.connected_peers.iter()
                            .find_map(|(nid, oid)| if oid.as_str() == from_online_id.as_str() { Some(nid) } else { None }) {

                            println!("✅ Mapped '{}' to peer ID {}", from_online_id, from_peer);
                            let packet = PacketData::new(data, from_peer, self.transfer_channel, self.transfer_mode);
                            self.incoming_packets.push(packet);
                            println!("📦 Added packet to queue (total: {})", self.incoming_packets.len());
                        } else {
                            println!("❌ Unknown peer '{}' (known peers: {:?})", from_online_id, self.connected_peers);
                        }
                    }
                    _ => {}
                }
            }
        }
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
    #[signal]
    fn relay_connected(online_id: GString);
    #[signal]
    fn hosting();
    #[signal]
    fn joined();

    #[func]
    pub fn connect_to_relay(&mut self, relay_address: String, port: i32) {
        if self.relay_connection_state != RelayState::Disconnected {
            godot_error!("Already connected or connecting to relay");
            return;
        }

        if self.runtime.is_none() {
            match NetworkingRuntime::new() {
                Ok(runtime) => self.runtime = Some(runtime),
                Err(e) => {
                    godot_error!("Failed to start networking runtime: {}", e);
                    return;
                }
            }
        }

        self.relay_connection_state = RelayState::Connecting;

        if let Some(runtime) = &self.runtime {
            if let Err(e) = runtime.send_command(NetworkCommand::ConnectToRelay {
                host: relay_address,
                port: port as u16,
            }) {
                godot_error!("Failed to send connect command: {}", e);
                self.relay_connection_state = RelayState::Disconnected;
            }
        }
    }

    #[func]
    pub fn host(&mut self) {
        if self.relay_connection_state != RelayState::Connected {
            godot_error!("Not connected to relay");
            return;
        }

        let online_id = match &self.online_id {
            Some(id) => id.clone(),
            None => {
                godot_error!("No online ID available");
                return;
            }
        };

        if let Some(runtime) = &self.runtime {
            if let Err(e) = runtime.send_command(NetworkCommand::Host { online_id }) {
                godot_error!("Failed to send host command: {}", e);
                return;
            }

            self.relay_connection_state = RelayState::Hosting;
        } else {
            godot_error!("Networking runtime not available");
        }
    }

    #[func]
    pub fn join(&mut self, host_online_id: String) {
        // Check if we're connected to relay
        if self.relay_connection_state != RelayState::Connected {
            godot_error!("Must be connected to relay before joining");
            return;
        }

        // Validate host online ID
        if host_online_id.is_empty() {
            godot_error!("Host online ID cannot be empty");
            return;
        }

        // Get our online ID
        let online_id = match &self.online_id {
            Some(id) => id.clone(),
            None => {
                godot_error!("No online ID available");
                return;
            }
        };

        // Send join command
        if let Some(runtime) = &self.runtime {
            if let Err(e) = runtime.send_command(NetworkCommand::Join {
                online_id,
                host_online_id
            }) {
                godot_error!("Failed to send join command: {}", e);
                return;
            }

            self.relay_connection_state = RelayState::Joined;
        } else {
            godot_error!("Networking runtime not available");
        }
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

    fn update_peer_list(&mut self, new_peer_list: Vec<PeerInfo>) {
        let old_peers: HashSet<i32> = self.connected_peers.keys().cloned().collect();
        let new_peers: HashSet<i32> = new_peer_list.iter().map(|p| p.numeric_id as i32).collect();

        for disconnected_peer in old_peers.difference(&new_peers) {
            self.signals().peer_disconnected().emit(*disconnected_peer as i64);
        }

        for connected_peer in new_peers.difference(&old_peers) {
            println!("Connected: {}", connected_peer);
            
            if *connected_peer != self.unique_id {
                self.signals().peer_connected().emit(*connected_peer as i64);
            }
        }

        self.connected_peers.clear();
        for peer in new_peer_list {
            self.connected_peers.insert(peer.numeric_id as i32, peer.online_id);
        }
    }
}
