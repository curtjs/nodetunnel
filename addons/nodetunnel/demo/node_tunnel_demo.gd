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
	peer.connect_to_relay("127.0.0.1", 9998)
	
	# Wait until we have connected to the relay
	await peer.relay_connected
	
	# Attach peer_connected signal
	peer.peer_connected.connect(_add_player)
	
	# Attach peer_disconnected signal
	peer.peer_disconnected.connect(_remove_player)
	
	# Attach room_left signal
	peer.room_left.connect(_cleanup_room)
	
	# At this point, we can access the online ID that the server generated for us
	%IDLabel.text = "Online ID: " + peer.online_id


func _on_host_pressed() -> void:
	print("Online ID: ", peer.online_id)
	
	# Host a game, must be done *after* relay connection is made
	peer.host()
	
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
	# Join a game, must be done *after* relay connection is made
	# Requires the online ID of the host peer
	peer.join(%HostID.text)
	
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
	peer.leave_room()

func _cleanup_room() -> void:
	%LeaveRoom.hide()
	%ConnectionControls.show()
