mod node_tunnel_peer;
mod tcp;
mod network;

use godot::prelude::*;

struct NodeTunnelExtension;

#[gdextension]
unsafe impl ExtensionLibrary for NodeTunnelExtension {}
