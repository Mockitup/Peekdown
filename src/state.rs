pub struct AppState {
    pub html: String,
    pub pending_file: Option<String>,
    pub pending_content: Option<String>,
    pub pending_title: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            html: String::new(),
            pending_file: None,
            pending_content: None,
            pending_title: None,
        }
    }
}
