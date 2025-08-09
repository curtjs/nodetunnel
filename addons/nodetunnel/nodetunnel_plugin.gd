@tool
extends EditorPlugin

func _enter_tree() -> void:
    print("NodeTunnel (Rust) addon enabled")

func _exit_tree() -> void:
    print("NodeTunnel (Rust) addon disabled")