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

class_name _NodeTunnelTCP

# Connection state
signal tcp_connected
@warning_ignore("unused_signal")
signal relay_connected(oid: String)

# Connection state
var tcp: StreamPeerTCP
var connected = false
var oid = ""

# Message buffering
var _buffer = PackedByteArray()

func _init() -> void:
	tcp = StreamPeerTCP.new()
	tcp.big_endian = true

func poll(packet_manager: _PacketManager) -> void:
	tcp.poll()
	
	if tcp.get_status() != StreamPeerTCP.STATUS_CONNECTED:
		return
	
	if connected == false:
		tcp_connected.emit()
		connected = true
	
	var available = tcp.get_available_bytes()
	if available > 0:
		var new_data = tcp.get_data(available)[1]
		_buffer.append_array(new_data)
	
	while _buffer.size() >= 4:
		var msg_len = ByteUtils.unpack_u32(_buffer, 0)
		
		if _buffer.size() >= 4 + msg_len:
			var msg_data = _buffer.slice(4, 4 + msg_len)
			_buffer = _buffer.slice(4 + msg_len)
			
			packet_manager.handle_msg(msg_data)
		else:
			break 
