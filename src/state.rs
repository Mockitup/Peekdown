pub struct AppState {
    pub current_file: Option<String>,
    pub dirty: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_file: None,
            dirty: false,
        }
    }
}
