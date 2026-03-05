use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tao::window::Window;
use wry::WebView;

use crate::file_ops;
use crate::state::AppState;

#[derive(Deserialize)]
struct IpcMessage {
    command: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

pub fn handle_ipc_message(
    msg: &str,
    webview: &WebView,
    window: &Window,
    state: &Arc<Mutex<AppState>>,
) {
    let parsed: IpcMessage = match serde_json::from_str(msg) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("IPC parse error: {e}");
            return;
        }
    };

    match parsed.command.as_str() {
        "open_file" => {
            let path = parsed.path.or_else(file_ops::pick_open_file);
            if let Some(p) = path {
                match file_ops::read_file(&p) {
                    Ok(contents) => {
                        let mut s = state.lock().unwrap();
                        s.current_file = Some(p.clone());
                        s.dirty = false;
                        window.set_title(&format!("mdview - {}", file_ops::filename(&p)));
                        send_to_js(webview, "file_opened", &serde_json::json!({
                            "content": contents,
                            "path": p
                        }));
                    }
                    Err(e) => send_to_js(webview, "error", &serde_json::json!({
                        "message": format!("Failed to open file: {e}")
                    })),
                }
            }
        }
        "save_file" => {
            let s = state.lock().unwrap();
            let current_file = s.current_file.clone();
            drop(s);

            if let (Some(path), Some(content)) = (current_file, &parsed.content) {
                match file_ops::write_file(&path, content) {
                    Ok(_) => {
                        state.lock().unwrap().dirty = false;
                        window.set_title(&format!("mdview - {}", file_ops::filename(&path)));
                        send_to_js(webview, "file_saved", &serde_json::json!({}));
                    }
                    Err(e) => send_to_js(webview, "error", &serde_json::json!({
                        "message": format!("Failed to save: {e}")
                    })),
                }
            } else {
                handle_save_as(webview, window, state, parsed.content);
            }
        }
        "save_as" => {
            handle_save_as(webview, window, state, parsed.content);
        }
        "new_file" => {
            let mut s = state.lock().unwrap();
            s.current_file = None;
            s.dirty = false;
            window.set_title("mdview - Untitled");
            send_to_js(webview, "file_opened", &serde_json::json!({
                "content": "",
                "path": ""
            }));
        }
        "content_changed" => {
            let mut s = state.lock().unwrap();
            if !s.dirty {
                s.dirty = true;
                let title = if let Some(ref p) = s.current_file {
                    format!("mdview - {} *", file_ops::filename(p))
                } else {
                    "mdview - Untitled *".to_string()
                };
                window.set_title(&title);
            }
        }
        "window_minimize" => {
            window.set_minimized(true);
        }
        "window_maximize" => {
            window.set_maximized(!window.is_maximized());
        }
        "window_close" => {
            // Save window state before closing
            let inner_size = window.inner_size();
            let outer_pos = window.outer_position().unwrap_or_default();
            crate::window_state::save_window_state(
                (outer_pos.x, outer_pos.y),
                (inner_size.width, inner_size.height),
            );
            std::process::exit(0);
        }
        "ready" => {
            // Frontend loaded; open file from CLI args if provided
        }
        _ => eprintln!("Unknown IPC command: {}", parsed.command),
    }
}

fn handle_save_as(
    webview: &WebView,
    window: &Window,
    state: &Arc<Mutex<AppState>>,
    content: Option<String>,
) {
    if let Some(content) = content {
        if let Some(path) = file_ops::pick_save_file() {
            match file_ops::write_file(&path, &content) {
                Ok(_) => {
                    let mut s = state.lock().unwrap();
                    s.current_file = Some(path.clone());
                    s.dirty = false;
                    window.set_title(&format!("mdview - {}", file_ops::filename(&path)));
                    send_to_js(webview, "file_saved", &serde_json::json!({}));
                }
                Err(e) => send_to_js(webview, "error", &serde_json::json!({
                    "message": format!("Failed to save: {e}")
                })),
            }
        }
    }
}

fn send_to_js(webview: &WebView, event: &str, data: &serde_json::Value) {
    let script = format!(
        "window.__fromRust({}, {})",
        serde_json::to_string(event).unwrap(),
        serde_json::to_string(data).unwrap(),
    );
    let _ = webview.evaluate_script(&script);
}
