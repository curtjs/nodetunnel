# NodeTunnel

Easy P2P multiplayer for Godot through relay servers.

## Quick Start

```gdscript
var peer = NodeTunnelPeer.new()
multiplayer.multiplayer_peer = peer

peer.connect_to_relay("nodetunnel.io", 9998)
await peer.relay_connected

# Host or join
peer.host()  # To host
peer.join("HOST_OID")  # To join
