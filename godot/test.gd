extends Node2D

var peer: NodeTunnelPeer

func _ready() -> void:
	peer = NodeTunnelPeer.new()
	multiplayer.multiplayer_peer = peer
	peer.start_network()
	peer.connect_to_relay("127.0.0.1:8080")
	var oid = await peer.relay_connected
	print("GD: Relay connected! Online ID: ", oid)
	peer.host()
