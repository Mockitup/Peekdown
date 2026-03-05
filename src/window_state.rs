use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct WindowState {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

fn config_path() -> PathBuf {
    let mut p = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("mdview");
    p.push("window_state.json");
    p
}

pub fn load_window_state() -> ((i32, i32), (u32, u32)) {
    let path = config_path();
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(state) = serde_json::from_str::<WindowState>(&data) {
            return ((state.x, state.y), (state.width, state.height));
        }
    }
    ((100, 100), (900, 700))
}

pub fn save_window_state(pos: (i32, i32), size: (u32, u32)) {
    let state = WindowState {
        x: pos.0,
        y: pos.1,
        width: size.0,
        height: size.1,
    };
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&path, serde_json::to_string(&state).unwrap_or_default());
}
