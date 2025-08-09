use godot::prelude::*;

mod utils;

struct NodeTunnelExtension;

#[gdextension]
unsafe impl ExtensionLibrary for NodeTunnelExtension {}
