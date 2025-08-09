use godot::prelude::*;

mod utils;
mod types;
mod networking;
mod test_networking;

struct NodeTunnelExtension;

#[gdextension]
unsafe impl ExtensionLibrary for NodeTunnelExtension {}
