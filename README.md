# NodeTunnel 🚇

**Easy P2P multiplayer networking for Godot through relay servers**

> ⚠️ **EARLY DEVELOPMENT** - This is experimental software! Expect bugs, breaking changes, and dragons. Not recommended for production use. Please report any issues you run into!

NodeTunnel provides peer-to-peer multiplayer without NAT traversal, port forwarding, or dedicated servers. Simply connect through a relay server and start playing!

## ✨ Features

- **Drop-in replacement** for Godot's built-in multiplayer
- **No port forwarding** or firewall configuration needed
- **Works behind NAT** and restrictive networks
- **Easy host/join** workflow with shareable room codes
- **Compatible** with existing Godot networking code

## 🚀 Quick Start

### YouTube Tutorial
[![Click here to view](http://img.youtube.com/vi/frNKdfPQfxA/0.jpg)](http://www.youtube.com/watch?v=frNKdfPQfxA "I Fixed Godot's Biggest Multiplayer Problem")

### Installation

1. Download from [GitHub Releases](https://github.com/curtjs/nodetunnel/releases)
2. Extract to your project's `addons/` folder
3. Enable "NodeTunnel" in Project Settings > Plugins

### Basic Usage

```gdscript
extends Node2D

func _ready():
    var peer = NodeTunnelPeer.new()
    multiplayer.multiplayer_peer = peer
    
    # Connect to the free public relay
    # Note that this **must** be done before hosting/joining
    peer.connect_to_relay("relay.nodetunnel.io", 9998)
    await peer.relay_connected
    print("Connected! Your ID: ", peer.online_id)

func host():
    # Host a game
    peer.host()
    await peer.hosting
    print("Share this ID: ", peer.online_id)

func join():
    # Join a game
    peer.join(host_id)
    await peer.joined

# Use normal Godot multiplayer from here!
@rpc("any_peer")
func player_moved(position: Vector2):
    print("Player moved to: ", position)
```

## 🎮 How It Works

1. **Connect** to a relay server  
2. **Get an Online ID** (like `ABC12345`)
3. **Host** or **join** using someone's ID
4. **Play** using normal Godot multiplayer

The relay forwards packets between players, so everyone can connect regardless of network setup.

## 🌐 Free Public Relay

I provide a free relay server for testing:

- **Host**: `relay.nodetunnel.io:9998`
- **Uptime**: See [nodetunnel.io](nodetunnel.io)

**Note**: Don't rely on this for anything important! Server source code will be available soon for self-hosting.

## 📚 API Reference

### NodeTunnelPeer

#### Signals
- `relay_connected(online_id: String)` - Connected to relay
- `hosting` - Started hosting  
- `joined` - Joined a session
- `room_left` - Disconnected from the room

#### Methods
- `connect_to_relay(host: String, port: int)` - Connect to relay
- `host()` - Start hosting
- `join(host_oid: String)` - Join using host's online ID
- `leave_room()` - Leaves the current room, will delete the room when called from host
- `disconnect_from_relay()` - Disconnect

#### Properties  
- `online_id: String` - Your unique session ID
- `debug_enabled: bool` - Enable debug logging

## 🔧 Troubleshooting

**Enable debug logging:**
```gdscript
peer.debug_enabled = true
```

**Common issues:**
- Check internet connection
- Verify online IDs are correct
- Make sure both players use the same relay server

## ⚠️ Limitations

- **Early alpha** - expect bugs and breaking changes
- **WebGL not supported** (use WebRTC for web games)  
- **Free relay** has no uptime guarantees
- **API will change** without notice

## 📄 License

MIT License

## 🆘 Support

- 🐛 **Issues**: [GitHub Issues](https://github.com/curtjs/nodetunnel/issues)
- 💬 **Discord**: [Discord Server](https://discord.com/invite/qxjZ3hFVVR)
