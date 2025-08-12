extends Node2D

var nt: NetworkTest

func _ready() -> void:
	print("Hello from Godot")
	nt = NetworkTest.new()
	nt.start_runtime()
	nt.test_connect()

func _process(delta: float) -> void:
	if Input.is_action_just_pressed("host"):
		nt.test_host()
	#if Input.is_action_just_pressed("join"):
		#nt.test_join($HostOID.text)
	
	nt.poll_events()


func _on_join_pressed() -> void:
	nt.test_join($HostOID.text)
