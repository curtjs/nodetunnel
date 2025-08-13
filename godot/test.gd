extends Node2D

var nt: NetworkTest
var peer: NodeTunnelPeer

const PLAYER = preload("res://player.tscn")

func _ready() -> void:
	print("Hello from Godot")
	
	peer = NodeTunnelPeer.new()
	multiplayer.multiplayer_peer = peer
	peer.connect_to_relay("127.0.0.1", 9998)
	await peer.relay_connected
	print("Connected to relay!!!")

func _process(delta: float) -> void:
	if Input.is_action_just_pressed("host"):
		peer.host()
		
		DisplayServer.clipboard_set(peer.get_online_id())
		
		add_child(PLAYER.instantiate())
		
		peer.peer_connected.connect(
			func(pid):
				print("Hello, ", pid)
				add_child(PLAYER.instantiate(), true)
		)
	#if Input.is_action_just_pressed("join"):
		#peer.join($HostOID.text)

func _on_join_pressed() -> void:
	peer.join($HostOID.text)
