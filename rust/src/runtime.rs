use std::sync::OnceLock;
use tokio::runtime::Runtime;

struct RuntimeWrapper(Option<Runtime>);

static RUNTIME: OnceLock<RuntimeWrapper> = OnceLock::new();

pub fn get_runtime() -> Option<&'static Runtime> {
    let wrapper = RUNTIME.get_or_init(|| {
        match Runtime::new() {
            Ok(rt) => {
                println!("Tokio runtime initialized successfully");
                RuntimeWrapper(Some(rt))
            }
            Err(e) => {
                println!("Failed to initialize tokio runtime: {}", e);
                RuntimeWrapper(None)
            }
        }
    });

    wrapper.0.as_ref()
}
