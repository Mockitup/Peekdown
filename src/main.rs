#![windows_subsystem = "windows"]

use std::borrow::Cow;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tao::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    platform::windows::WindowBuilderExtWindows,
    window::WindowBuilder,
};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOZORDER};
use wry::{WebViewBuilder, WebViewBuilderExtWindows, WebViewExtWindows};

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

    // Parse CLI args
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut cli_file: Option<String> = None;
    let mut stdin_flag = false;
    let mut title_arg: Option<String> = None;
    {
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--stdin" => stdin_flag = true,
                "--title" => {
                    if i + 1 < args.len() {
                        i += 1;
                        title_arg = Some(args[i].clone());
                    }
                }
                _ => {
                    if cli_file.is_none() {
                        cli_file = Some(args[i].clone());
                    }
                }
            }
            i += 1;
        }
    }

    // Read stdin if --stdin flag and stdin is a pipe/file (not a console)
    if stdin_flag {
        extern "system" {
            fn GetStdHandle(nStdHandle: u32) -> isize;
            fn GetFileType(hFile: isize) -> u32;
        }
        const STD_INPUT_HANDLE: u32 = 0xFFFF_FFF6; // (DWORD)-10
        const FILE_TYPE_DISK: u32 = 0x0001;
        const FILE_TYPE_PIPE: u32 = 0x0003;

        let is_piped = unsafe {
            let handle = GetStdHandle(STD_INPUT_HANDLE);
            let ft = GetFileType(handle);
            ft == FILE_TYPE_PIPE || ft == FILE_TYPE_DISK
        };
        if is_piped {
            let mut buf = String::new();
            if std::io::stdin().read_to_string(&mut buf).is_ok() && !buf.is_empty() {
                let mut st = app_state.lock().unwrap();
                st.pending_content = Some(buf);
                st.pending_title = title_arg;
            }
        }
    }

    // The event loop must exist first: creating it makes the process
    // per-monitor DPI aware, which the placement math depends on.
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy: EventLoopProxy<UserEvent> = event_loop.create_proxy();

    // Create hidden on the target monitor, then move to the exact saved rect
    // and show (maximized if it was) so there is no visible jump or flash.
    let placement = window_state::plan();
    let (pos, size) = placement.initial_bounds();
    let window = WindowBuilder::new()
        .with_title("Peekdown - Untitled")
        .with_decorations(false)
        .with_visible(false)
        .with_window_classname(window_state::WINDOW_CLASS)
        .with_inner_size(PhysicalSize::new(size.0, size.1))
        .with_position(PhysicalPosition::new(pos.0, pos.1))
        .build(&event_loop)
        .unwrap();
    window_state::apply(&window, &placement);
    // Already shown natively; this keeps tao's own visibility flag in sync.
    window.set_visible(true);

    let full_html = build_html();
    {
        app_state.lock().unwrap().html = full_html;
    }

    let proxy_ipc = proxy.clone();
    let proxy_drop = proxy.clone();

    let state_proto = Arc::clone(&app_state);
    let _webview = WebViewBuilder::new()
        .with_custom_protocol("peekdown".to_string(), move |_id, request| {
            let uri = request.uri().path();
            if uri == "/" || uri == "/index.html" {
                let st = state_proto.lock().unwrap();
                wry::http::Response::builder()
                    .header("Content-Type", "text/html")
                    .body(Cow::Owned(st.html.as_bytes().to_vec()))
                    .unwrap()
            } else if uri.starts_with("/local-image") {
                let query = request.uri().query().unwrap_or("");
                let file_path = percent_encoding::percent_decode_str(query)
                    .decode_utf8_lossy()
                    .to_string();
                match std::fs::read(&file_path) {
                    Ok(data) => {
                        let ext = std::path::Path::new(&file_path)
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
                            "ico" => "image/x-icon",
                            "tiff" | "tif" => "image/tiff",
                            _ => "application/octet-stream",
                        };
                        wry::http::Response::builder()
                            .header("Content-Type", mime)
                            .body(Cow::Owned(data))
                            .unwrap()
                    }
                    Err(_) => wry::http::Response::builder()
                        .status(404)
                        .body(Cow::Borrowed(b"Image not found" as &[u8]))
                        .unwrap(),
                }
            } else {
                wry::http::Response::builder()
                    .status(404)
                    .body(Cow::Borrowed(b"Not found" as &[u8]))
                    .unwrap()
            }
        })
        .with_url("http://peekdown.localhost/")
        .with_ipc_handler(move |request| {
            let body = request.body().to_string();
            let _ = proxy_ipc.send_event(UserEvent::IpcMessage(body));
        })
        .with_new_window_req_handler(|_| false)
        .with_drag_drop_handler(move |event| {
            match event {
                wry::DragDropEvent::Enter { .. } => {
                    let msg = serde_json::json!({"command": "drag_enter"}).to_string();
                    let _ = proxy_drop.send_event(UserEvent::IpcMessage(msg));
                }
                wry::DragDropEvent::Drop { paths, .. } => {
                    let leave = serde_json::json!({"command": "drag_leave"}).to_string();
                    let _ = proxy_drop.send_event(UserEvent::IpcMessage(leave));
                    for path in &paths {
                        let ext = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        if ext == "md" || ext == "markdown" || ext == "txt" {
                            let msg = serde_json::json!({
                                "command": "open_file",
                                "path": path.to_string_lossy()
                            })
                            .to_string();
                            let _ = proxy_drop.send_event(UserEvent::IpcMessage(msg));
                        }
                    }
                }
                wry::DragDropEvent::Leave => {
                    let msg = serde_json::json!({"command": "drag_leave"}).to_string();
                    let _ = proxy_drop.send_event(UserEvent::IpcMessage(msg));
                }
                _ => {}
            }
            true
        })
        .with_browser_accelerator_keys(false)
        .with_devtools(true)
        .build(&window)
        .expect("Failed to build WebView");

    // Store CLI file path to open once JS is ready
    if let Some(file_path) = cli_file {
        app_state.lock().unwrap().pending_file = Some(file_path);
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::IpcMessage(msg)) => {
                ipc::handle_ipc_message(&msg, &_webview, &window, &app_state);
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                let w = new_size.width as i32;
                let h = new_size.height as i32;
                unsafe {
                    let controller = _webview.controller();
                    let _ = controller.SetBounds(RECT { left: 0, top: 0, right: w, bottom: h });
                    let mut host = HWND::default();
                    if controller.ParentWindow(&mut host).is_ok() {
                        let _ = SetWindowPos(host, None, 0, 0, w, h,
                            SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOZORDER);
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                window_state::save(&window);
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
