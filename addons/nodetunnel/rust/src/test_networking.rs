use godot::prelude::*;

use crate::{networking::runtime::NetworkingRuntime, types::commands::NetworkCommand};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct NetworkTest {
    base: Base<Node>,
    runtime: Option<NetworkingRuntime>,
}

#[godot_api]
impl INode for NetworkTest {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            runtime: None
        }
    }

    fn ready(&mut self) {
        self.runtime = Some(NetworkingRuntime::new());
        godot_print!("NetworkTest ready!");
    }
}

#[godot_api]
impl NetworkTest {
    #[func]
    fn test_connect(&mut self) {
        if let Some(runtime) = &self.runtime {
            let _ = runtime.send_command(NetworkCommand::ConnectToRelay {
                host: "test.example.com".to_string(),
                port: 9998,
            });
            godot_print!("Sent connect command");
        }
    }

    #[func]
    fn poll_events(&mut self) {
        if let Some(runtime) = &mut self.runtime {
            let events = runtime.poll_events();
            for event in events {
                godot_print!("Received event: {:?}", event);
            }
        }
    }
}