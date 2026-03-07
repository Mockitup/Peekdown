pub struct AppState {
    pub pending_file: Option<String>,
    pub pending_content: Option<String>,
    pub pending_title: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            pending_file: None,
            pending_content: None,
            pending_title: None,
        }
    }
}
