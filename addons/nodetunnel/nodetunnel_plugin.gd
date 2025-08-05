@tool
extends EditorPlugin

func _enter_tree() -> void:
	print("NodeTunnel addon enabled")

func _exit_tree() -> void:
	print("NodeTunnel addon disabled")
