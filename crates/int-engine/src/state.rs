use std::sync::Mutex;
use int_core::Manifest;

pub struct AppState {
    pub current_manifest: Mutex<Option<Manifest>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_manifest: Mutex::new(None),
        }
    }
}
