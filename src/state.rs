pub struct AppState {
    pub pending_file: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            pending_file: None,
        }
    }
}
