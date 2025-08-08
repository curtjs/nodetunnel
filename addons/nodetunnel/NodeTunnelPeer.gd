class_name NodeTunnelPeer extends MultiplayerPeerExtension
## A relay-based multiplayer peer that connects through NodeTunnel servers
##
## Provides P2P-style multiplayer networking through a relay server without
## requiring direct connections between clients. Integrates with Godot's
## MultiplayerAPI system.

## Connection states for tracking relay connection progress
enum ConnectionState {
	DISCONNECTED,
	CONNECTING,
	CONNECTED,
	HOSTING,
	JOINED
}

# Public signals

## Fires when [member connect_to_relay] succeeds.
## [br]
## [br][param online_id] The online ID from NodeTunnel
signal relay_connected(online_id: String)
## Fires when [member host] succeeds.
signal hosting
## Fires when [member join] succeeds.
signal joined
## Fires when this peer leaves a room
## Also fires when the room host leaves and kicks this peer from the room
signal room_left

# Connection configuration
var relay_host: String
var relay_port: int
var online_id: String = ""
var connection_state: ConnectionState = ConnectionState.DISCONNECTED

# Multiplayer peer state
var unique_id: int = 0
var connected_peers: Dictionary[int, bool] = {}
var connection_status: MultiplayerPeer.ConnectionStatus = MultiplayerPeer.CONNECTION_CONNECTING
var target_peer: int = 0
var current_transfer_mode: MultiplayerPeer.TransferMode = MultiplayerPeer.TRANSFER_MODE_RELIABLE
var current_transfer_channel: int = 0

# Internal networking components
var _packet_manager: _PacketManager
var _tcp_handler: _NodeTunnelTCP
var _udp_handler: _NodeTunnelUDP

# Peer management
var _numeric_to_online_id: Dictionary[int, String] = {}
var _peer_list_ready: bool = false
var _peer_leaving_room: bool = false

# Packet management
var _incoming_packets: Array = []
var _udp_packet_buffer: Array = []

# Debug configuration
static var debug_enabled: bool = false

## Packet data container for multiplayer system
class PacketData:
	var data: PackedByteArray
	var from_peer: int
	var channel: int
	var mode: MultiplayerPeer.TransferMode
	
	func _init(p_data: PackedByteArray, p_from: int, p_channel: int = 0, p_mode: MultiplayerPeer.TransferMode = MultiplayerPeer.TRANSFER_MODE_RELIABLE):
		data = p_data
		from_peer = p_from
		channel = p_channel
		mode = p_mode

func _init():
	_packet_manager = _PacketManager.new()
	_tcp_handler = _NodeTunnelTCP.new()
	_udp_handler = _NodeTunnelUDP.new()
	_packet_manager.peer_list_res.connect(_handle_peer_list)
	_packet_manager.leave_room_res.connect(_handle_leave_room)

# ============================================================================
# PUBLIC API
# ============================================================================

## Connect to a NodeTunnel relay server
## [br]
## [br][param host]: The relay server hostname or IP address
## [br][param port]: The TCP port for the relay server (typically 9998)
func connect_to_relay(node_tunnel_address: String, node_tunnel_port: int) -> void:
	if connection_state != ConnectionState.DISCONNECTED:
		_log_warning("Already connected or connecting to relay")
		return
	
	connection_state = ConnectionState.CONNECTING
	connection_status = MultiplayerPeer.CONNECTION_CONNECTING
	relay_host = node_tunnel_address
	relay_port = node_tunnel_port
	
	_tcp_handler.tcp.connect_to_host(relay_host, relay_port)
	await _tcp_handler.tcp_connected
	
	_packet_manager.send_conect(_tcp_handler.tcp)
	online_id = await _packet_manager.connect_res
	
	_udp_handler.connect_to_relay(relay_host, 9999, online_id)
	_udp_handler.packet_received.connect(_handle_udp_packet)
	
	connection_state = ConnectionState.CONNECTED
	connection_status = MultiplayerPeer.CONNECTION_CONNECTED
	_log("Connected to relay with OID: " + online_id)
	relay_connected.emit(online_id)

## Start hosting a multiplayer session
func host() -> void:
	if connection_state != ConnectionState.CONNECTED:
		_log_error("Must be connected to relay before hosting")
		return
	
	if !_udp_handler.connected:
		_log("Sending UDP Connect Request")
		_udp_handler.send_connect()
		await _udp_handler.udp_connected
	_log("UDP Connected")
	
	_log("Sending TCP Host Request")
	_packet_manager.send_host(_tcp_handler.tcp, online_id)
	await _packet_manager.peer_list_res
	_log("Peer List Received")
	
	connection_state = ConnectionState.HOSTING
	connection_status = MultiplayerPeer.CONNECTION_CONNECTED
	_log("Started hosting session")
	hosting.emit()

## Join a multiplayer session using the host's online ID
## [br]
## [br][param host_oid]: The online ID of the hosting peer
func join(host_oid: String) -> void:
	if connection_state != ConnectionState.CONNECTED:
		_log_error("Must be connected to relay before joining")
		return
	
	if host_oid.is_empty():
		_log_error("Host OID cannot be empty")
		return
	
	_udp_handler.send_connect()
	await _udp_handler.udp_connected
	
	_packet_manager.send_join(_tcp_handler.tcp, online_id, host_oid)
	await _packet_manager.peer_list_res
	
	connection_state = ConnectionState.JOINED
	connection_status = MultiplayerPeer.CONNECTION_CONNECTED
	_log("Joined session with host: " + host_oid)
	joined.emit()

func leave_room() -> void:
	if connection_state != ConnectionState.JOINED && connection_state != ConnectionState.HOSTING:
		_log_error("Must be in a room before attempting to leave!")
		return
	
	_peer_leaving_room = true
	
	_packet_manager.send_leave_room(_tcp_handler.tcp)

## Disconnect from the relay server and clean up all connections
func disconnect_from_relay() -> void:
	_reset_connection()
	online_id = ""
	_log("Disconnected from relay")

# ============================================================================
# PRIVATE NETWORKING METHODS
# ============================================================================

## WARNING: Internal NodeTunnel Code
## [b]Do not call this method directly![/b]
func _send_relay_data(to_peer_numeric: int, data: PackedByteArray) -> void:
	var to_oid = "0"  # Broadcast by default
	
	if to_peer_numeric != 0 && _numeric_to_online_id.has(to_peer_numeric):
		to_oid = _numeric_to_online_id[to_peer_numeric]
	
	_udp_handler.send_packet(to_oid, data)

## WARNING: Internal NodeTunnel Code
## [b]Do not call this method directly![/b]
func _handle_udp_packet(from_oid: String, data: PackedByteArray) -> void:
	if !_peer_list_ready || _numeric_to_online_id.is_empty():
		_udp_packet_buffer.append({"from_oid": from_oid, "data": data})
		return
	
	_process_udp_packet(from_oid, data)

## WARNING: Internal NodeTunnel Code
## [b]Do not call this method directly![/b]
func _process_udp_packet(from_oid: String, data: PackedByteArray) -> void:
	var from_nid = _get_numeric_id_for_online_id(from_oid)
	
	if from_nid == 0:
		return
	if from_nid == unique_id:
		return
	
	var packet_data = PacketData.new(data, from_nid, current_transfer_channel, current_transfer_mode)
	_incoming_packets.append(packet_data)

## WARNING: Internal NodeTunnel Code
## [b]Do not call this method directly![/b]
func _handle_peer_list(numeric_to_online_id: Dictionary[int, String]) -> void:
	# Store old state before clearing
	var old_connected_peers = connected_peers.duplicate()
	
	# Emit disconnection signals for peers that are no longer in the list
	for old_nid in old_connected_peers.keys():
		if !numeric_to_online_id.has(old_nid):
			peer_disconnected.emit(old_nid)
	
	# Update peer mappings
	connected_peers.clear()
	_numeric_to_online_id = numeric_to_online_id
	
	# Process new peer connections
	for nid in numeric_to_online_id.keys():
		var was_connected = old_connected_peers.has(nid)
		connected_peers[nid] = true
		
		if numeric_to_online_id[nid] == online_id:
			unique_id = nid
		elif !was_connected:
			peer_connected.emit(nid)
	
	_peer_list_ready = true
	
	# Process any buffered UDP packets
	for packet in _udp_packet_buffer:
		_process_udp_packet(packet.from_oid, packet.data)
	_udp_packet_buffer.clear()
	
	_log("Updated peer list: " + str(connected_peers))

func _handle_leave_room() -> void:
	for p in connected_peers:
		peer_disconnected.emit(p)
	
	connection_state = ConnectionState.CONNECTED
	connection_status = MultiplayerPeer.CONNECTION_CONNECTING
	connected_peers.clear()
	_incoming_packets.clear()
	_udp_packet_buffer.clear()
	_numeric_to_online_id.clear()
	unique_id = 0
	_peer_list_ready = false
	
	room_left.emit()

# ============================================================================
# UTILITY METHODS
# ============================================================================

## WARNING: Internal NodeTunnel Code
## [b]Do not call this method directly![/b]
func _get_numeric_id_for_online_id(oid: String) -> int:
	for nid in _numeric_to_online_id.keys():
		if _numeric_to_online_id[nid] == oid:
			return nid
	return 0

## WARNING: Internal NodeTunnel Code
## [b]Do not call this method directly![/b]
func _reset_connection() -> void:
	connection_state = ConnectionState.DISCONNECTED
	connection_status = MultiplayerPeer.CONNECTION_DISCONNECTED
	connected_peers.clear()
	_incoming_packets.clear()
	_udp_packet_buffer.clear()
	_numeric_to_online_id.clear()
	unique_id = 0
	_peer_list_ready = false

## WARNING: Internal NodeTunnel Code
## [b]Do not call this method directly![/b]
static func _log(message: String) -> void:
	if debug_enabled:
		print("[NodeTunnel] " + message)

## WARNING: Internal NodeTunnel Code
## [b]Do not call this method directly![/b]
static func _log_warning(message: String) -> void:
	if debug_enabled:
		push_warning("[NodeTunnel] " + message)

## WARNING: Internal NodeTunnel Code
## [b]Do not call this method directly![/b]
static func _log_error(message: String) -> void:
	push_error("[NodeTunnel] " + message)

# ============================================================================
# MULTIPLAYER PEER EXTENSION IMPLEMENTATION
# ============================================================================

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _get_connection_status() -> MultiplayerPeer.ConnectionStatus:
	return connection_status

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _get_unique_id() -> int:
	return unique_id

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _is_server() -> bool:
	return unique_id == 1

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _is_server_relay_supported() -> bool:
	return true

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _get_packet_script() -> PackedByteArray:
	if _incoming_packets.is_empty():
		return PackedByteArray()
	
	var packet_data = _incoming_packets.pop_front()
	return packet_data.data

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _put_packet_script(p_buffer: PackedByteArray) -> Error:
	if connection_state != ConnectionState.HOSTING and connection_state != ConnectionState.JOINED:
		return ERR_UNCONFIGURED
	
	_send_relay_data(target_peer, p_buffer)
	return OK

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _get_available_packet_count() -> int:
	return _incoming_packets.size()

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _get_max_packet_size() -> int:
	return 1400  # Safe UDP packet size

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _get_packet_peer() -> int:
	if _incoming_packets.is_empty():
		return 0
	return _incoming_packets[0].from_peer

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _get_packet_channel() -> int:
	if _incoming_packets.is_empty():
		return 0
	return _incoming_packets[0].channel

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _get_packet_mode() -> MultiplayerPeer.TransferMode:
	if _incoming_packets.is_empty():
		return MultiplayerPeer.TRANSFER_MODE_RELIABLE
	return _incoming_packets[0].mode

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _set_target_peer(p_peer: int) -> void:
	target_peer = p_peer

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _set_transfer_channel(p_channel: int) -> void:
	current_transfer_channel = p_channel

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _get_transfer_channel() -> int:
	return current_transfer_channel

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _set_transfer_mode(p_mode: MultiplayerPeer.TransferMode) -> void:
	current_transfer_mode = p_mode

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _get_transfer_mode() -> MultiplayerPeer.TransferMode:
	return current_transfer_mode

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _close() -> void:
	disconnect_from_relay()

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _disconnect_peer(_p_peer: int, _p_force: bool = false) -> void:
	# Cannot directly disconnect peers in relay mode
	pass

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _is_refusing_new_connections() -> bool:
	return refuse_new_connections

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _set_refuse_new_connections(p_enable: bool) -> void:
	refuse_new_connections = p_enable

## WARNING: Called automatically by MultiplayerAPI
## [b]Do not call this method directly![/b]
func _poll() -> void:
	_tcp_handler.poll(_packet_manager)
	_udp_handler.poll()
