extends Node2D

var nt: NetworkTest
var peer: NodeTunnelPeer

func _ready() -> void:
	print("Hello from Godot")
	#nt = NetworkTest.new()
	#nt.start_runtime()
	#nt.test_connect()
	
	peer = NodeTunnelPeer.new()
	peer.connect_to_relay("127.0.0.1", 9998)

func _process(delta: float) -> void:
	if Input.is_action_just_pressed("host"):
		peer.host()
		#nt.test_host()
	if Input.is_action_just_pressed("join"):
		peer.join($HostOID.text)
		#nt.test_join($HostOID.text)
	
	#nt.poll_events()


func _on_join_pressed() -> void:
	pass
	#nt.test_join($HostOID.text)
