#![windows_subsystem = "windows"]

use std::sync::{Arc, Mutex};
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

mod file_ops;
mod ipc;
mod state;
mod window_state;

const INDEX_HTML: &str = include_str!("frontend/index.html");
const STYLE_CSS: &str = include_str!("frontend/style.css");
const APP_JS: &str = include_str!("frontend/app.js");
const EDITOR_JS: &str = include_str!("frontend/editor.js");
const PREVIEW_JS: &str = include_str!("frontend/preview.js");
const TABS_JS: &str = include_str!("frontend/tabs.js");
const MARKED_JS: &str = include_str!("frontend/marked.min.js");
const HLJS: &str = include_str!("frontend/highlight.min.js");

#[derive(Debug)]
enum UserEvent {
    IpcMessage(String),
}

fn main() {
    let app_state = Arc::new(Mutex::new(state::AppState::new()));

    // Check for CLI arg (file to open)
    let cli_file = std::env::args().nth(1);

    let (pos, size) = window_state::load_window_state();

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy: EventLoopProxy<UserEvent> = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("mdview - Untitled")
        .with_decorations(false)
        .with_inner_size(LogicalSize::new(size.0 as f64, size.1 as f64))
        .with_position(LogicalPosition::new(pos.0 as f64, pos.1 as f64))
        .build(&event_loop)
        .unwrap();

    let full_html = build_html();

    let proxy_ipc = proxy.clone();
    let proxy_drop = proxy.clone();

    let _webview = WebViewBuilder::new()
        .with_html(&full_html)
        .with_ipc_handler(move |request| {
            let body = request.body().to_string();
            let _ = proxy_ipc.send_event(UserEvent::IpcMessage(body));
        })
        .with_new_window_req_handler(|_| false) // block new windows
        .with_drag_drop_handler(move |event| {
            match event {
                wry::DragDropEvent::Enter { .. } => {
                    // We'll show overlay via JS
                }
                wry::DragDropEvent::Drop { paths, .. } => {
                    for path in &paths {
                        let msg = serde_json::json!({
                            "command": "open_file",
                            "path": path.to_string_lossy()
                        })
                        .to_string();
                        let _ = proxy_drop.send_event(UserEvent::IpcMessage(msg));
                    }
                }
                _ => {}
            }
            true
        })
        .with_devtools(true)
        .build(&window)
        .expect("Failed to build WebView");

    // Open CLI file if provided
    if let Some(file_path) = cli_file {
        let msg = serde_json::json!({
            "command": "open_file",
            "path": file_path
        })
        .to_string();
        let _ = proxy.send_event(UserEvent::IpcMessage(msg));
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::IpcMessage(msg)) => {
                ipc::handle_ipc_message(&msg, &_webview, &window, &app_state);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                let inner_size = window.inner_size();
                let outer_pos = window.outer_position().unwrap_or_default();
                window_state::save_window_state(
                    (outer_pos.x, outer_pos.y),
                    (inner_size.width, inner_size.height),
                );
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn escape_for_script_tag(js: &str) -> String {
    // Prevent "</script" in JS from prematurely closing the <script> tag
    js.replace("</script", "<\\/script")
}

fn build_html() -> String {
    // Build script tags with escaped content
    let scripts = format!(
        "<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>",
        escape_for_script_tag(HLJS),
        escape_for_script_tag(MARKED_JS),
        escape_for_script_tag(PREVIEW_JS),
        escape_for_script_tag(TABS_JS),
        escape_for_script_tag(EDITOR_JS),
        escape_for_script_tag(APP_JS),
    );

    INDEX_HTML
        .replace("/* __CSS__ */", STYLE_CSS)
        .replace("<!-- __SCRIPTS__ -->", &scripts)
}
