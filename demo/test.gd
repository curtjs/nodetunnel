extends Node2D

var network_test: NetworkTest

func _ready() -> void:
	network_test = NetworkTest.new()
	add_child(network_test)
	network_test.test_connect()

func _process(delta: float) -> void:
	if network_test:
		network_test.poll_events()
