use godot::prelude::*;

use crate::{networking::runtime::NetworkingRuntime, types::commands::NetworkCommand};
use crate::types::events::NetworkEvent;
use crate::types::packet_types::PeerInfo;

#[derive(GodotClass)]
#[class(base=Object)]
pub struct NetworkTest {
    base: Base<Object>,
    runtime: Option<NetworkingRuntime>,
    online_id: Option<String>,
    peer_list: Option<Vec<PeerInfo>>
}

#[godot_api]
impl IObject for NetworkTest {
    fn init(base: Base<Object>) -> Self {
        Self {
            base,
            runtime: None,
            online_id: None,
            peer_list: None
        }
    }
}

#[godot_api]
impl NetworkTest {
    #[func]
    fn start_runtime(&mut self) {
        match NetworkingRuntime::new() {
            Ok(runtime) => {
                self.runtime = Some(runtime);
            }
            Err(e) => {
                godot_error!("Failed to start runtime: {}", e);
            }
        }
    }
    
    #[func]
    fn test_connect(&mut self) {
        if let Some(runtime) = &self.runtime {
            runtime.send_command(NetworkCommand::ConnectToRelay {
                host: "localhost".to_string(),
                port: 9998,
            }).unwrap_or_else(|e| {
                println!("Error sending command: {}", e)
            });
            println!("Sent connect command");
        }
    }

    #[func]
    fn test_host(&mut self) {
        let runtime = self.runtime.as_ref().expect("Networking runtime not started");
        let online_id = self.online_id.as_ref().expect("Online ID not set; did you connect first?");
        
        runtime.send_command(NetworkCommand::Host {
            online_id: online_id.clone(),
        }).unwrap_or_else(|e| {
            println!("Failed to send command: {}", e)
        });
    }
    
    #[func]
    fn test_join(&mut self, host_online_id: String) {
        let runtime = self.runtime.as_ref().expect("Networking runtime not started");
        let online_id = self.online_id.as_ref().expect("Online ID not set; did you connect first?");

        runtime.send_command(NetworkCommand::Join {
            online_id: online_id.clone(),
            host_online_id
        }).unwrap_or_else(|e| {
            println!("Failed to send command: {}", e)
        });
    }

    #[func]
    fn poll_events(&mut self) {
        if let Some(runtime) = &mut self.runtime {
            let events = runtime.poll_events();
            for event in events {
                match &event {
                    NetworkEvent::RelayConnected { online_id } => {
                        self.online_id = Some(online_id.clone());
                    }
                    NetworkEvent::Hosting { peer_list } => {
                        println!("Hosting");
                        self.peer_list = Some(peer_list.clone())
                    }
                    NetworkEvent::Joined { peer_list } => {
                        println!("Joined!");
                        self.peer_list = Some(peer_list.clone())
                    }
                    _ => {
                        println!("Received event: {:?}", event);
                    }
                }
            }
        } else {
            println!("Not connected")
        }
    }
    
    #[func]
    fn get_online_id(&self) -> String {
        self.online_id.clone().unwrap_or("Not connected".to_string())
    }
}
