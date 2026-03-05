# Peekdown — Markdown Viewer/Editor

## What is this?
A lightweight native Windows markdown viewer/editor built with Rust + WebView2.
Goal: Notepad-fast startup, Obsidian-pretty UI. Single ~800 KB executable.

## Tech Stack
- Rust backend (wry + tao) for window management, file I/O, drag & drop
- HTML/CSS/JS frontend embedded in the binary for UI rendering
- marked.js for markdown → HTML conversion
- highlight.js for syntax highlighting (30+ languages)
- WebView2 (Edge) as the rendering engine (pre-installed on Win10/11)

## Build
```bash
cargo build --release
```
Binary output: `target/release/peekdown.exe`

`build.rs` uses `winresource` to embed `assets/icon.ico` into the .exe.

## Project Layout
- `src/main.rs` — Entry point, window + WebView setup, event loop, HTML assembly
- `src/ipc.rs` — IPC message dispatch between Rust and JS
- `src/file_ops.rs` — File read/write, native open/save dialogs (rfd)
- `src/state.rs` — App state (empty struct — JS owns all tab state)
- `src/window_state.rs` — Window position/size persistence (config dir: `peekdown/`)
- `build.rs` — Embeds app icon via winresource
- `assets/icon.ico` — App icon (multi-resolution, generated from icon.png)
- `src/frontend/` — All HTML/CSS/JS files (embedded at compile time via include_str!)
  - `index.html` — Shell with titlebar, tab bar, find bar, TOC panel, editor, preview
  - `style.css` — Full styling with dark/light theme via CSS custom properties
  - `app.js` — IPC bridge, mode toggle, split view, zoom, find, recent files, TOC, theme, shortcuts
  - `tabs.js` — Tab manager IIFE (JS-owned state: content, path, dirty, mode, scroll, cursor)
  - `editor.js` — Textarea input handling, tab key, dirty tracking, split preview sync
  - `preview.js` — marked.js custom renderer with highlight.js integration
  - `marked.min.js` — Markdown parser (v15)
  - `highlight.min.js` — Syntax highlighting (v11.11.1, common bundle)

## Architecture
- IPC: JS → Rust via `window.ipc.postMessage(JSON)`, Rust → JS via `webview.evaluate_script()`
- Frontend files are concatenated into a single HTML document at compile time using placeholder replacement
- JS-owned tab state: Rust is a stateless file I/O service, JS manages all tab data
- Script load order: highlight.js → marked.js → preview.js → tabs.js → editor.js → app.js
- Toggle mode: single pane switches between edit (textarea) and preview (rendered HTML)
- Split mode: side-by-side editor + live preview with debounced sync
- Theming: CSS custom properties with `[data-theme="light"]` overrides, persisted to localStorage

## Features
- Multi-tab with auto-hiding tab bar (single tab = no bar)
- Dark/Light themes (Catppuccin Mocha / Latte)
- Split view (Ctrl+\) with live preview
- Syntax highlighting for code blocks
- Find bar (Ctrl+F) with match highlighting
- Table of Contents sidebar (Ctrl+Shift+O)
- Zoom (Ctrl+/-, Ctrl+scroll) with toast indicator
- Draggable preview width
- Recent files panel on empty tabs
- Cross-mode selection preservation when toggling edit/preview
- Drag & drop (single and multi-file)
- File associations via CLI arg

## Conventions
- Keep the binary small: use `opt-level = "s"`, `lto = "fat"`, `panic = "abort"`, `strip = "symbols"`
- No external runtime dependencies — everything embedded in the .exe
- Dark theme with Catppuccin-inspired color palette (Mocha dark, Latte light)
- JS uses IIFE pattern for modules (TabManager)
- localStorage keys prefixed with `peekdown-` (theme, recent, preview-width)
