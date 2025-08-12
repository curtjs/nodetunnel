use godot::prelude::*;

mod utils;
mod types;
mod networking;
mod test_networking;
mod node_tunnel_peer;

struct NodeTunnelExtension;

#[gdextension]
unsafe impl ExtensionLibrary for NodeTunnelExtension {}
