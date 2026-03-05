# mdview — Markdown Viewer/Editor

## What is this?
A lightweight native Windows markdown reader/editor built with Rust + WebView2.
Goal: Notepad-fast startup, Obsidian-pretty UI.

## Tech Stack
- Rust backend (wry + tao) for window management, file I/O, drag & drop
- HTML/CSS/JS frontend embedded in the binary for UI rendering
- marked.js for markdown → HTML conversion
- WebView2 (Edge) as the rendering engine (pre-installed on Win10/11)

## Build
```bash
cargo build --release
```
Binary output: `target/release/mdview.exe`

## Project Layout
- `src/main.rs` — Entry point, window + WebView setup, event loop
- `src/ipc.rs` — IPC message dispatch between Rust and JS
- `src/file_ops.rs` — File read/write, native open/save dialogs (rfd)
- `src/state.rs` — App state (current file path, dirty flag)
- `src/window_state.rs` — Window position/size persistence
- `src/frontend/` — All HTML/CSS/JS files (embedded at compile time via include_str!)
- `docs/PLAN.md` — Full implementation plan

## Architecture
- IPC: JS → Rust via `window.ipc.postMessage(JSON)`, Rust → JS via `webview.evaluate_script()`
- Frontend files are concatenated into a single HTML document at compile time using placeholder replacement
- Toggle mode: single pane switches between edit (textarea) and preview (rendered HTML)

## Conventions
- Keep the binary small: use `opt-level = "s"`, `lto = "fat"`, `panic = "abort"`, `strip = "symbols"`
- No external runtime dependencies — everything embedded in the .exe
- Dark theme with Catppuccin-inspired color palette
