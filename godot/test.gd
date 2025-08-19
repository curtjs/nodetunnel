extends Node2D

var peer: NodeTunnelPeer

func _ready() -> void:
	peer = NodeTunnelPeer.new()
	peer.connect_to_relay("127.0.0.1", 9998)
