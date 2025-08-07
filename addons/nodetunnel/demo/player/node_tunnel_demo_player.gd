extends CharacterBody2D

# Same as any other movement script
# Uses MultiplayerSynchronizer to sync position

const SPEED = 250.0


func _enter_tree() -> void:
	set_multiplayer_authority(name.to_int())


func _physics_process(delta: float) -> void:
	if !is_multiplayer_authority():
		return
	
	var input_dir = Input.get_vector("ui_left", "ui_right", "ui_up", "ui_down")
	velocity = input_dir * SPEED
	
	move_and_slide()
