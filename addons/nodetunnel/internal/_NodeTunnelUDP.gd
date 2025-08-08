# ============================================================================
# ⚠️  WARNING: INTERNAL NODETUNNEL CODE - DO NOT MODIFY ⚠️
# ============================================================================
# This class is part of NodeTunnel's internal networking implementation.
# 
# 🚫 DO NOT:
#   - Call these methods directly
#   - Modify this code
#   - Inherit from this class
#   - Copy this code
#
# ✅ USE INSTEAD: NodeTunnelPeer class for all networking operations
#
# Modifying this code WILL break your networking and void support!
# ============================================================================

class_name _NodeTunnelUDP

var socket: PacketPeerUDP
var relay_host: String
var relay_port: int
var online_id: String
var connected: bool = false

signal udp_connected
signal packet_received(from_oid: String, data: PackedByteArray)

func _init() -> void:
	socket = PacketPeerUDP.new()

func connect_to_relay(host: String, port: int, oid: String):
	relay_port = port
	online_id = oid
	
	if not host.is_valid_ip_address():
		var ip = IP.resolve_hostname(host)
		if ip == "":
			print("Failed to resolve hostname: ", host)
			return
		relay_host = ip
		print("Resolved ", host, " to ", ip)
	else:
		relay_host = host
	
	var res = socket.connect_to_host(relay_host, port)
	if res != OK:
		NodeTunnelPeer._log_error("Failed to connect to UDP socket: " + str(res))

func send_connect():
	send_packet("SERVER", "UDP_CONNECT".to_utf8_buffer())

func handle_connect_res():
	udp_connected.emit()
	connected = true

func send_packet(to_oid: String, data: PackedByteArray):
	var packet = PackedByteArray()
	
	packet.append_array(ByteUtils.pack_u32(online_id.length()))
	packet.append_array(online_id.to_utf8_buffer())
	
	packet.append_array(ByteUtils.pack_u32(to_oid.length()))
	packet.append_array(to_oid.to_utf8_buffer())
	
	packet.append_array(data)
	
	socket.put_packet(packet)

func poll():
	while socket.get_available_packet_count() > 0:
		var packet = socket.get_packet()
		
		if packet.size() < 8:
			continue
		
		var offset = 0
		
		var sender_oid_len = ByteUtils.unpack_u32(packet, offset)
		offset += 4
		
		if packet.size() < offset + sender_oid_len + 4:
			continue
		
		var sender_oid = packet.slice(offset, offset + sender_oid_len).get_string_from_utf8()
		offset += sender_oid_len
		
		var target_oid_len = ByteUtils.unpack_u32(packet, offset)
		offset += 4
		
		if packet.size() < offset + target_oid_len:
			continue
		
		var _target_oid = packet.slice(offset, offset + target_oid_len).get_string_from_utf8()
		offset += target_oid_len
		
		var game_data = packet.slice(offset)
		
		if sender_oid == "SERVER":
			var message = game_data.get_string_from_utf8()
			if message == "UDP_CONNECT_RES":
				handle_connect_res()
		else:
			packet_received.emit(sender_oid, game_data)
