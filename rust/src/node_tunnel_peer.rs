use godot::classes::multiplayer_peer::{ConnectionStatus, TransferMode};
use godot::classes::{IMultiplayerPeerExtension, MultiplayerPeerExtension};
use godot::global::Error;
use godot::prelude::*;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use crate::channels::messages::{NetworkCommand, NetworkEvent};
use crate::networking::client;
use crate::runtime;

#[derive(GodotClass)]
#[class(tool, base=MultiplayerPeerExtension)]
pub struct NodeTunnelPeer {
    base: Base<MultiplayerPeerExtension>,

    // Networking
    command_sender: Option<UnboundedSender<NetworkCommand>>,
    event_receiver: Option<UnboundedReceiver<NetworkEvent>>,
    networking_task: Option<JoinHandle<()>>,
    #[var]
    online_id: GString,

    // Multiplayer peer state
    unique_id: i32,
    connection_status: ConnectionStatus,
    target_peer: i32,
    transfer_mode: TransferMode,
    transfer_channel: i32,

    // Packet management
    incoming_packets: Vec<u8>,

    // Peer management
    connected_peers: Vec<u32>,
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
    fn start_network(&mut self) {
        if let Some(rt) = runtime::get_runtime() {
            let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<NetworkCommand>();
            let (event_tx, event_rx) = mpsc::unbounded_channel::<NetworkEvent>();

            let handle = rt.spawn(client::networking_task(cmd_rx, event_tx));

            self.command_sender = Some(cmd_tx);
            self.event_receiver = Some(event_rx);
            self.networking_task = Some(handle);
        }
    }

    #[func]
    fn connect_to_relay(&mut self, server_addr: String) {
        self.send_command(NetworkCommand::ConnectToRelay(server_addr));
    }
    
    #[func]
    fn host(&mut self) {
        self.send_command(NetworkCommand::Host);
    }

    #[func]
    fn join(&mut self, host_online_id: GString) {
        self.send_command(NetworkCommand::Join(host_online_id.to_string()))
    }

    fn send_command(&mut self, network_cmd: NetworkCommand) {
        if let Some(cmd) = &self.command_sender {
            match cmd.send(network_cmd) {
                Ok(_) => println!("Sent command!"),
                Err(e) => println!("Failed to send command: {}", e)
            }
        }
    }

    fn handle_event(&mut self, event: NetworkEvent) {
        match event {
            NetworkEvent::ConnectedToRelay(online_id) => {
                self.signals().relay_connected().emit(&online_id);
                self.online_id = online_id.to_godot();
            },
            NetworkEvent::Error(e) => println!("Network thread error: {}", e),
            NetworkEvent::ConnectedToRoom(numeric_id) => {
                self.unique_id = numeric_id as i32;
                println!("Connected to room with NID of {}", self.unique_id);
            },
            NetworkEvent::PeerList(new_peer_list) => {
                self.handle_peer_list_update(new_peer_list);
            }
        }
    }

    fn handle_peer_list_update(&mut self, new_peer_ids: Vec<u32>) {
        let old_peers = &self.connected_peers;

        let mut disconnected_peers = Vec::new();
        let mut connected_peers = Vec::new();

        // Find disconnected peers
        for &old_peer_id in old_peers {
            if !new_peer_ids.contains(&old_peer_id) && old_peer_id != self.unique_id as u32 {
                println!("Peer {} disconnected", old_peer_id);
                disconnected_peers.push(old_peer_id);
            }
        }

        // Find connected peers
        for &new_peer_id in &new_peer_ids {
            if !old_peers.contains(&new_peer_id) && new_peer_id != self.unique_id as u32 {
                println!("Peer {} connected", new_peer_id);
                connected_peers.push(new_peer_id);
            }
        }

        // Update peer list first
        self.connected_peers = new_peer_ids;

        if self.connection_status == ConnectionStatus::CONNECTING {
            self.connection_status = ConnectionStatus::CONNECTED;
        }

        // Now emit all signals with a single base_mut() call
        let mut base = self.base_mut();
        for peer_id in disconnected_peers {
            base.emit_signal("peer_disconnected", &[peer_id.to_variant()]);
        }
        for peer_id in connected_peers {
            base.emit_signal("peer_connected", &[peer_id.to_variant()]);
        }

        println!("Peer list updated");
    }
}

#[godot_api]
impl IMultiplayerPeerExtension for NodeTunnelPeer {
    fn init(base: Base<MultiplayerPeerExtension>) -> Self {
        Self {
            base,

            command_sender: None,
            event_receiver: None,
            networking_task: None,
            online_id: "".to_godot(),

            unique_id: 0,
            connection_status: ConnectionStatus::CONNECTING, // CONNECTION_DISCONNECTED
            target_peer: 0,
            transfer_mode: TransferMode::RELIABLE,     // TRANSFER_MODE_RELIABLE
            transfer_channel: 0,
            incoming_packets: Vec::new(),

            connected_peers: Vec::new(),
        }
    }

    fn get_available_packet_count(&self) -> i32 {
        self.incoming_packets.len() as i32
    }

    fn get_max_packet_size(&self) -> i32 {
        1400
    }

    fn get_packet_script(&mut self) -> PackedByteArray {
        PackedByteArray::new()
    }

    fn put_packet_script(&mut self, _p_buffer: PackedByteArray) -> Error {
        Error::OK
    }

    fn get_packet_channel(&self) -> i32 {
        0
    }

    fn get_packet_mode(&self) -> TransferMode {
        TransferMode::RELIABLE
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
        0
    }

    fn is_server(&self) -> bool {
        self.unique_id == 1
    }

    fn poll(&mut self) {
        let mut events_to_handle = Vec::new();

        if let Some(events) = &mut self.event_receiver {
            while let Ok(event) = events.try_recv() {
                events_to_handle.push(event);
            }
        }

        for event in events_to_handle {
            self.handle_event(event);
        }
    }

    fn close(&mut self) {
        todo!()
    }

    fn disconnect_peer(&mut self, _p_peer: i32, _p_force: bool) {
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
