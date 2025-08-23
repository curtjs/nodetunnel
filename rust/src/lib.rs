mod node_tunnel_peer;
mod networking;
mod channels;
pub mod runtime;

use godot::prelude::*;

struct NodeTunnelExtension;

#[gdextension]
unsafe impl ExtensionLibrary for NodeTunnelExtension {}
