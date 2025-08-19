use std::thread::JoinHandle;
use godot::classes::multiplayer_peer::{ConnectionStatus, TransferMode};
use godot::classes::{IMultiplayerPeerExtension, MultiplayerPeerExtension};
use godot::global::Error;
use godot::prelude::*;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, UnboundedSender};
use crate::network::network_messages::{NetworkCommand, NetworkEvent};
use crate::tcp::tcp_client::TcpClient;

#[derive(GodotClass)]
#[class(tool, base=MultiplayerPeerExtension)]
pub struct NodeTunnelPeer {
    base: Base<MultiplayerPeerExtension>,

    // Networking
    runtime: Runtime,
    tcp_client: Option<TcpClient>,
    tx: Option<UnboundedSender<NetworkCommand>>,
    rx: Option<Receiver<NetworkEvent>>,

    network_thread: Option<JoinHandle<()>>,

    // Multiplayer peer state
    unique_id: i32,
    connection_status: ConnectionStatus,
    target_peer: i32,
    transfer_mode: TransferMode,
    transfer_channel: i32,

    // Packet management
    incoming_packets: Vec<u8>,
}

#[godot_api]
impl IMultiplayerPeerExtension for NodeTunnelPeer {
    fn init(base: Base<MultiplayerPeerExtension>) -> Self {
        Self {
            base,
            runtime: Runtime::new().unwrap(),
            tcp_client: None,
            tx: None,
            rx: None,
            network_thread: None,
            unique_id: 0,
            connection_status: ConnectionStatus::CONNECTING, // CONNECTION_DISCONNECTED
            target_peer: 0,
            transfer_mode: TransferMode::RELIABLE,     // TRANSFER_MODE_RELIABLE
            transfer_channel: 0,
            incoming_packets: Vec::new(),
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

#[godot_api]
impl NodeTunnelPeer {
    #[signal]
    fn relay_connected(online_id: GString);
    #[signal]
    fn hosting();
    #[signal]
    fn joined();

    #[func]
    pub fn connect_to_relay(&mut self, relay_address: String, relay_port: u16) {
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel();
        
        let tcp_client = TcpClient::new(relay_address, relay_port);

        self.runtime.spawn(async move {
            match tcp_client.connect().await {
                Ok(_) => {
                    println!("Connected to TCP!");
                },
                Err(e) => {
                    println!("Failed to connect to TCP: {}", e);
                }
            }
        });
    }
}