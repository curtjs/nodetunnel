extends Node2D

const PLAYER_SCENE = preload("res://addons/nodetunnel/demo/player/node_tunnel_demo_player.tscn")

var peer: NodeTunnelPeer


func _ready() -> void:
	# Create the NodeTunnelPeer
	peer = NodeTunnelPeer.new()
	#peer.debug_enabled = true # Enable debugging if needed
	
	# Always set the global peer *before* attempting to connect
	multiplayer.multiplayer_peer = peer
	
	# Connect to the public relay
	peer.connect_to_relay("localhost", 9998)
	
	# Wait until we have connected to the relay
	await peer.relay_connected
	
	_update_room_list()
	
	# Attach peer_connected signal
	peer.peer_connected.connect(_add_player)
	
	# Attach peer_disconnected signal
	peer.peer_disconnected.connect(_remove_player)
	
	# Attach room_left signal
	peer.room_left.connect(_cleanup_room)
	
	# At this point, we can access the online ID that the server generated for us
	%IDLabel.text = "Online ID: " + peer.online_id

func _update_room_list() -> void:
	var rooms = await peer.room_list()
	
	for button in %PublicRooms.get_children():
		button.queue_free()
	
	for room in rooms:
		var button = Button.new()
		button.text = room
		button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		button.pressed.connect(_join_room.bind(room))
		%PublicRooms.add_child(button)

func _on_host_pressed() -> void:
	print("Online ID: ", peer.online_id)
	
	var flags = RoomFlags.RoomFlags.NONE
	
	if not %Public.button_pressed:
		flags |= RoomFlags.RoomFlags.UNLISTED
	
	# Host a game, must be done *after* relay connection is made
	peer.host(flags)
	
	# Copy online id to clipboard
	DisplayServer.clipboard_set(peer.online_id)
	
	# Wait until peer has started hosting
	await peer.hosting
	
	# Spawn the host player
	_add_player()
	
	# Hide the UI
	%ConnectionControls.hide()
	
	# Show leave room button
	%LeaveRoom.show()

func _on_join_pressed() -> void:
	_join_room(%HostID.text)

func _join_room(host_id: String) -> void:
	# Join a game, must be done *after* relay connection is made
	# Requires the online ID of the host peer
	peer.join(host_id)
	
	# Wait until peer has finished joining
	await peer.joined
	
	# Hide the UI
	%ConnectionControls.hide()
	
	# Show leave room button
	%LeaveRoom.show()

# Same as any other Godot game
# Uses the MultiplayerSpawner node's auto-spawn list to spawn players
func _add_player(peer_id: int = 1) -> void:
	if !multiplayer.is_server(): return
	
	print("Player Joined: ", peer_id)
	var player = PLAYER_SCENE.instantiate()
	player.name = str(peer_id)
	add_child(player)


func _remove_player(peer_id: int) -> void:
	if !multiplayer.is_server(): return
	
	var player = get_node(str(peer_id))
	player.queue_free()


func _on_leave_room_pressed() -> void:
	# Tells NodeTunnel to remove this peer from the room
	# Will eventually result in `peer.room_left` being emitted
	peer.leave_room()


# This function runs whenever this peer gets removed from a room,
# whether it's intentional or due to the host leaving.
# See peer.room_left.connect(_cleanup_room) in the _ready() function
func _cleanup_room() -> void:
	# Hide the leave room button
	%LeaveRoom.hide()
	
	# Show the main menu again
	%ConnectionControls.show()


func _on_refresh_pressed() -> void:
	_update_room_list()
