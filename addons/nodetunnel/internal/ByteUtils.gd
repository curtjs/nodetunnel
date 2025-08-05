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

class_name ByteUtils

static func unpack_u32(data: PackedByteArray, offset: int) -> int:
	if offset + 4 > data.size():
		push_warning("ByteUtils.unpack_u32: Not enough data at offset %d (need 4 bytes, have %d)" % [offset, data.size() - offset])
		return 0
	
	# Extract 4 bytes in big-endian order
	var big_endian_bytes = data.slice(offset, offset + 4)
	
	# Convert to little-endian for Godot's decode_u32
	var little_endian_bytes = PackedByteArray([
		big_endian_bytes[3], 
		big_endian_bytes[2], 
		big_endian_bytes[1], 
		big_endian_bytes[0]
	])
	
	return little_endian_bytes.decode_u32(0)

static func pack_u32(value: int) -> PackedByteArray:
	if value < 0 or value > 0xFFFFFFFF:
		push_warning("ByteUtils.pack_u32: Value %d is outside valid range (0 to 4294967295)" % value)
		value = value & 0xFFFFFFFF  # Clamp to 32-bit range
	
	# Encode in little-endian first
	var little_endian_bytes = PackedByteArray()
	little_endian_bytes.resize(4)
	little_endian_bytes.encode_u32(0, value)
	
	# Convert to big-endian for network transmission
	var big_endian_bytes = PackedByteArray([
		little_endian_bytes[3], 
		little_endian_bytes[2], 
		little_endian_bytes[1], 
		little_endian_bytes[0]
	])
	
	return big_endian_bytes
