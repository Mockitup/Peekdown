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
    #[serde(default)]
    title: Option<String>,
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
            if let Some(ref content) = parsed.content {
                if let Some(ref path) = parsed.path {
                    match file_ops::write_file(path, content) {
                        Ok(_) => {
                            send_to_js(webview, "file_saved", &serde_json::json!({
                                "path": path
                            }));
                        }
                        Err(e) => send_to_js(webview, "error", &serde_json::json!({
                            "message": format!("Failed to save: {e}")
                        })),
                    }
                } else {
                    handle_save_as(webview, parsed.content);
                }
            }
        }
        "save_as" => {
            handle_save_as(webview, parsed.content);
        }
        "set_title" => {
            if let Some(title) = parsed.title {
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
            let inner_size = window.inner_size();
            let outer_pos = window.outer_position().unwrap_or_default();
            crate::window_state::save_window_state(
                (outer_pos.x, outer_pos.y),
                (inner_size.width, inner_size.height),
            );
            std::process::exit(0);
        }
        "read_image" => {
            if let Some(ref path) = parsed.path {
                match std::fs::read(path) {
                    Err(e) => eprintln!("read_image failed for {}: {}", path, e),
                    Ok(data) => {
                    let ext = std::path::Path::new(path)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let mime = match ext.as_str() {
                        "png" => "image/png",
                        "jpg" | "jpeg" => "image/jpeg",
                        "gif" => "image/gif",
                        "svg" => "image/svg+xml",
                        "webp" => "image/webp",
                        "bmp" => "image/bmp",
                        _ => "application/octet-stream",
                    };
                    let b64 = base64_encode(&data);
                    let data_uri = format!("data:{};base64,{}", mime, b64);
                    let script = format!(
                        "window.__setImage({}, {})",
                        serde_json::to_string(path).unwrap(),
                        serde_json::to_string(&data_uri).unwrap(),
                    );
                    let _ = webview.evaluate_script(&script);
                    }
                }
            }
        }
        "ready" => {
            let (pending_file, pending_content, pending_title) = {
                let mut st = state.lock().unwrap();
                (st.pending_file.take(), st.pending_content.take(), st.pending_title.take())
            };
            if let Some(p) = pending_file {
                match file_ops::read_file(&p) {
                    Ok(contents) => {
                        send_to_js(webview, "file_opened", &serde_json::json!({
                            "content": contents,
                            "path": p
                        }));
                    }
                    Err(e) => send_to_js(webview, "error", &serde_json::json!({
                        "message": format!("Failed to open file: {e}")
                    })),
                }
            } else if let Some(content) = pending_content {
                let title = pending_title.unwrap_or_else(|| "stdin".to_string());
                send_to_js(webview, "stdin_opened", &serde_json::json!({
                    "content": content,
                    "title": title
                }));
            }
        }
        _ => eprintln!("Unknown IPC command: {}", parsed.command),
    }
}

fn handle_save_as(
    webview: &WebView,
    content: Option<String>,
) {
    if let Some(content) = content {
        if let Some(path) = file_ops::pick_save_file() {
            match file_ops::write_file(&path, &content) {
                Ok(_) => {
                    send_to_js(webview, "file_saved", &serde_json::json!({
                        "path": path
                    }));
                }
                Err(e) => send_to_js(webview, "error", &serde_json::json!({
                    "message": format!("Failed to save: {e}")
                })),
            }
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn send_to_js(webview: &WebView, event: &str, data: &serde_json::Value) {
    let script = format!(
        "window.__fromRust({}, {})",
        serde_json::to_string(event).unwrap(),
        serde_json::to_string(data).unwrap(),
    );
    let _ = webview.evaluate_script(&script);
}
