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

class_name _PacketManager

signal connect_res(oid: String)
signal host_res
signal join_res
signal peer_list_res(nid_to_oid: Dictionary[int, String])
signal leave_room_res

enum PacketType {
	CONNECT,
	HOST,
	JOIN,
	PEERLIST,
	LEAVE_ROOM
}

func handle_msg(data: PackedByteArray) -> void:
	var pkt_type = ByteUtils.unpack_u32(data, 0)
	var payload = data.slice(4)
	
	match pkt_type:
		PacketType.CONNECT:
			var oid = _handle_connect_res(payload)
			connect_res.emit(oid)
		PacketType.HOST:
			host_res.emit()
		PacketType.JOIN:
			join_res.emit()
		PacketType.PEERLIST:
			var p_list = _handle_peer_list(payload)
			peer_list_res.emit(p_list)
		PacketType.LEAVE_ROOM:
			leave_room_res.emit()

## Sends a connect request to the server
func send_conect(tcp: StreamPeerTCP) -> void:
	var msg = PackedByteArray()
	msg.append_array(ByteUtils.pack_u32(PacketType.CONNECT))
	
	tcp.put_data(ByteUtils.pack_u32(msg.size()))
	tcp.put_data(msg)

## Handles the connect response
## Returns OID
func _handle_connect_res(data: PackedByteArray) -> String:
	var oid_len = ByteUtils.unpack_u32(data, 0)
	var oid = data.slice(4, 4 + oid_len).get_string_from_utf8()
	return oid

## Sends a host request to the server
func send_host(tcp: StreamPeerTCP, oid: String) -> void:
	var msg = PackedByteArray()
	msg.append_array(ByteUtils.pack_u32(PacketType.HOST))
	msg.append_array(ByteUtils.pack_u32(oid.length()))
	msg.append_array(oid.to_utf8_buffer())
	
	tcp.put_data(ByteUtils.pack_u32(msg.size()))
	tcp.put_data(msg)

## Sends a join request to the server
func send_join(tcp: StreamPeerTCP, oid: String, host_oid: String) -> void:
	var msg = PackedByteArray()
	msg.append_array(ByteUtils.pack_u32(PacketType.JOIN))
	
	msg.append_array(ByteUtils.pack_u32(oid.length()))
	msg.append_array(oid.to_utf8_buffer())
	
	msg.append_array(ByteUtils.pack_u32(host_oid.length()))
	msg.append_array(host_oid.to_utf8_buffer())
	
	tcp.put_data(ByteUtils.pack_u32(msg.size()))
	tcp.put_data(msg)

## Sends a leave room request to the server
func send_leave_room(tcp: StreamPeerTCP) -> void:
	var msg = PackedByteArray()
	msg.append_array(ByteUtils.pack_u32(PacketType.LEAVE_ROOM))
	
	tcp.put_data(ByteUtils.pack_u32(msg.size()))
	tcp.put_data(msg)

## Requests the peer list
func req_p_list(tcp: StreamPeerTCP) -> void:
	var msg = PackedByteArray()
	msg.append_array(ByteUtils.pack_u32(PacketType.PEERLIST))
	
	tcp.put_data(ByteUtils.pack_u32(msg.size()))
	tcp.put_data(msg)

## Handles receiving the peer list
func _handle_peer_list(data: PackedByteArray) -> Dictionary[int, String]:
	var offset = 0
	var p_count = ByteUtils.unpack_u32(data, offset)
	offset += 4
	
	var nid_to_oid: Dictionary[int, String] = {}
	
	for i in range(p_count):
		var oid_len = ByteUtils.unpack_u32(data, offset)
		offset += 4
		var oid = data.slice(offset, offset + oid_len).get_string_from_utf8()
		offset += oid_len
		
		var nid = ByteUtils.unpack_u32(data, offset)
		offset += 4
		
		nid_to_oid[nid] = oid
	
	return nid_to_oid
