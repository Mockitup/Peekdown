//! Window placement persistence, multi-monitor and per-monitor-DPI aware.
//!
//! Everything here works in *physical* pixels and virtual-screen coordinates
//! (the coordinate space Win32 uses for a per-monitor-DPI-aware process), and
//! every rect is the window's *outer* rect as reported by `GetWindowRect`.
//! Storing the outer rect means we never have to re-derive the invisible resize
//! borders tao adds to undecorated windows: we put back exactly what we read.
//!
//! What we persist is the window's *restored* rect (where it goes when it is not
//! maximized/minimized), whether it was maximized, and the DPI of the monitor it
//! was on, so the size can be rescaled if that monitor's scaling changes.
//!
//! On startup the saved rect is validated against the current monitor layout:
//! if its monitor is gone we fall back to the monitor under the mouse, and the
//! window is always fitted fully inside a single monitor's work area.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tao::platform::windows::WindowExtWindows;
use tao::window::Window;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, POINT, RECT, TRUE};
use windows::Win32::Graphics::Gdi::{
    ClientToScreen, GetMonitorInfoW, MonitorFromPoint, MonitorFromRect, HMONITOR, MONITORINFO,
    MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTONULL, MONITOR_DEFAULTTOPRIMARY,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetClientRect, GetCursorPos, GetWindowPlacement, GetWindowRect,
    IsIconic, IsWindowVisible, IsZoomed, SetWindowPos, ShowWindow, SWP_NOACTIVATE, SWP_NOZORDER,
    SW_SHOWMAXIMIZED, SW_SHOWNORMAL, WINDOWPLACEMENT, WPF_RESTORETOMAXIMIZED,
};

/// Window class name for Peekdown windows, so instances can find each other.
pub const WINDOW_CLASS: &str = "PeekdownWindow";

/// Sizes below are in logical pixels (DIPs) and get scaled by the monitor's DPI.
const DEFAULT_SIZE: (i32, i32) = (900, 700);
const MIN_SIZE: (i32, i32) = (320, 240);
const CASCADE_STEP: i32 = 32;
const BASE_DPI: u32 = 96;

#[derive(Serialize, Deserialize)]
struct Saved {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    dpi: u32,
    maximized: bool,
}

/// Where the window should go, resolved against the current monitor layout.
pub struct Placement {
    /// Work area (screen minus taskbar) of the target monitor.
    work: RECT,
    dpi: u32,
    /// Desired outer rect, already rescaled to `dpi` but not yet fitted to `work`.
    rect: RECT,
    maximized: bool,
}

impl Placement {
    /// Position and size to create the window with. This lies inside the target
    /// monitor, so Windows gives the new window that monitor's DPI from the start
    /// (moving it there afterwards would trigger a DPI change and resize it).
    pub fn initial_bounds(&self) -> ((i32, i32), (u32, u32)) {
        let r = fit(self.rect, self.work, (0, 0));
        ((r.left, r.top), (width(&r) as u32, height(&r) as u32))
    }
}

fn config_path() -> PathBuf {
    let mut p = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("peekdown");
    p.push("window_state.json");
    p
}

/// Work out where the window should open: the saved placement if it is still
/// on a connected monitor, otherwise a default size on the monitor under the mouse.
pub fn plan() -> Placement {
    let saved = fs::read_to_string(config_path())
        .ok()
        .and_then(|data| serde_json::from_str::<Saved>(&data).ok());
    saved
        .and_then(placement_from_saved)
        .unwrap_or_else(default_placement)
}

fn placement_from_saved(s: Saved) -> Option<Placement> {
    let rect = RECT { left: s.left, top: s.top, right: s.right, bottom: s.bottom };
    if width(&rect) <= 0 || height(&rect) <= 0 {
        return None;
    }
    // The monitor that overlaps the rect the most; null if it overlaps none
    // (e.g. the monitor it was on has been disconnected).
    let monitor = unsafe { MonitorFromRect(&rect, MONITOR_DEFAULTTONULL) };
    if monitor.is_invalid() {
        return None;
    }
    let (work, dpi) = monitor_work_area(monitor)?;

    // If that monitor's scale factor changed since we saved, keep the same
    // logical size rather than the same pixel size.
    let rect = if s.dpi != 0 && s.dpi != dpi {
        let w = scale_by(width(&rect), dpi, s.dpi);
        let h = scale_by(height(&rect), dpi, s.dpi);
        RECT { left: rect.left, top: rect.top, right: rect.left + w, bottom: rect.top + h }
    } else {
        rect
    };

    Some(Placement { work, dpi, rect, maximized: s.maximized })
}

fn default_placement() -> Placement {
    // Open where the user is working: the monitor under the mouse.
    let mut cursor = POINT::default();
    let _ = unsafe { GetCursorPos(&mut cursor) };
    let monitor = unsafe { MonitorFromPoint(cursor, MONITOR_DEFAULTTOPRIMARY) };
    let (work, dpi) = monitor_work_area(monitor)
        .unwrap_or((RECT { left: 0, top: 0, right: 1024, bottom: 768 }, BASE_DPI));

    let w = scale_by(DEFAULT_SIZE.0, dpi, BASE_DPI).min(width(&work) * 9 / 10);
    let h = scale_by(DEFAULT_SIZE.1, dpi, BASE_DPI).min(height(&work) * 9 / 10);
    let left = work.left + (width(&work) - w) / 2;
    let top = work.top + (height(&work) - h) / 2;
    let rect = RECT { left, top, right: left + w, bottom: top + h };

    Placement { work, dpi, rect, maximized: false }
}

/// Move the (still hidden) window to its final rect and show it.
pub fn apply(window: &Window, p: &Placement) {
    let hwnd = hwnd_of(window);
    let insets = frame_insets(hwnd);

    // Fit the *visible* part of the window into the work area; the invisible
    // resize borders may hang off the edge, just like with Aero Snap.
    let min = (scale_by(MIN_SIZE.0, p.dpi, BASE_DPI), scale_by(MIN_SIZE.1, p.dpi, BASE_DPI));
    let visible = fit(deflate(p.rect, &insets), p.work, min);
    let visible = cascade(visible, &insets, p);
    let outer = inflate(visible, &insets);

    unsafe {
        let _ = SetWindowPos(
            hwnd,
            None,
            outer.left,
            outer.top,
            width(&outer),
            height(&outer),
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
        // Showing maximized directly (instead of show-then-maximize) avoids a
        // flash of the restored window. The rect set above becomes the window's
        // restore position.
        let _ = ShowWindow(hwnd, if p.maximized { SW_SHOWMAXIMIZED } else { SW_SHOWNORMAL });
    }
}

/// Persist the window's current placement.
pub fn save(window: &Window) {
    let Some(state) = capture(hwnd_of(window)) else { return };
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(&state) {
        let _ = fs::write(&path, json);
    }
}

fn capture(hwnd: HWND) -> Option<Saved> {
    let mut wp = WINDOWPLACEMENT {
        length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
        ..Default::default()
    };
    unsafe { GetWindowPlacement(hwnd, &mut wp) }.ok()?;

    let minimized = unsafe { IsIconic(hwnd) }.as_bool();
    let maximized = if minimized {
        wp.flags.0 & WPF_RESTORETOMAXIMIZED.0 != 0
    } else {
        unsafe { IsZoomed(hwnd) }.as_bool()
    };

    let rect = if minimized || maximized {
        // The current rect is the maximized/minimized one; we want the restore
        // rect. rcNormalPosition is in *workspace* coordinates (relative to the
        // monitor's work area), so shift it back to screen coordinates.
        let mut r = wp.rcNormalPosition;
        let monitor = unsafe { MonitorFromRect(&r, MONITOR_DEFAULTTONEAREST) };
        if let Some(mi) = monitor_info(monitor) {
            let dx = mi.rcWork.left - mi.rcMonitor.left;
            let dy = mi.rcWork.top - mi.rcMonitor.top;
            r = RECT { left: r.left + dx, top: r.top + dy, right: r.right + dx, bottom: r.bottom + dy };
        }
        r
    } else {
        // In the normal state use the live rect: unlike rcNormalPosition it
        // reflects Aero Snap (half-screen etc.) positions.
        let mut r = RECT::default();
        unsafe { GetWindowRect(hwnd, &mut r) }.ok()?;
        r
    };

    if width(&rect) <= 0 || height(&rect) <= 0 {
        return None;
    }
    let monitor = unsafe { MonitorFromRect(&rect, MONITOR_DEFAULTTONEAREST) };
    let dpi = monitor_work_area(monitor).map_or(BASE_DPI, |(_, dpi)| dpi);

    Some(Saved {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
        dpi,
        maximized,
    })
}

/// If another visible Peekdown window already sits at this spot (typical when
/// opening several files at once), step diagonally so the new window doesn't
/// hide it exactly, wrapping back to the work area's corner when out of room.
/// Peers' outer rects are compared against our outer rect (`visible` inflated
/// by `insets`), since all Peekdown windows have the same insets at a given DPI.
fn cascade(visible: RECT, insets: &RECT, p: &Placement) -> RECT {
    if p.maximized {
        return visible;
    }
    let mut peers: Vec<RECT> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect_peers), LPARAM(&mut peers as *mut Vec<RECT> as isize));
    }
    let taken = |v: &RECT| {
        let o = inflate(*v, insets);
        peers.iter().any(|x| (x.left - o.left).abs() <= 2 && (x.top - o.top).abs() <= 2)
    };

    let step = scale_by(CASCADE_STEP, p.dpi, BASE_DPI);
    let mut r = visible;
    for _ in 0..=peers.len() {
        if !taken(&r) {
            break;
        }
        r = offset(r, step, step);
        if r.right > p.work.right || r.bottom > p.work.bottom {
            r = offset(r, p.work.left - r.left, p.work.top - r.top);
        }
    }
    r
}

unsafe extern "system" fn collect_peers(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let out = &mut *(lparam.0 as *mut Vec<RECT>);
    let mut buf = [0u16; 64];
    let len = GetClassNameW(hwnd, &mut buf);
    if len > 0
        && String::from_utf16_lossy(&buf[..len as usize]) == WINDOW_CLASS
        && IsWindowVisible(hwnd).as_bool()
        && !IsIconic(hwnd).as_bool()
        && !IsZoomed(hwnd).as_bool()
    {
        let mut r = RECT::default();
        if GetWindowRect(hwnd, &mut r).is_ok() {
            out.push(r);
        }
    }
    TRUE
}

/// Thickness of the invisible frame on each side of the window (outer rect
/// minus client rect). Must be measured while the window is not maximized.
fn frame_insets(hwnd: HWND) -> RECT {
    let mut outer = RECT::default();
    let mut client = RECT::default();
    let mut origin = POINT::default();
    unsafe {
        if GetWindowRect(hwnd, &mut outer).is_err()
            || GetClientRect(hwnd, &mut client).is_err()
            || !ClientToScreen(hwnd, &mut origin).as_bool()
        {
            return RECT::default();
        }
    }
    RECT {
        left: (origin.x - outer.left).max(0),
        top: (origin.y - outer.top).max(0),
        right: (outer.right - (origin.x + client.right)).max(0),
        bottom: (outer.bottom - (origin.y + client.bottom)).max(0),
    }
}

fn monitor_info(monitor: HMONITOR) -> Option<MONITORINFO> {
    let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
    unsafe { GetMonitorInfoW(monitor, &mut mi) }.as_bool().then_some(mi)
}

fn monitor_work_area(monitor: HMONITOR) -> Option<(RECT, u32)> {
    let mi = monitor_info(monitor)?;
    let (mut dpi_x, mut dpi_y) = (BASE_DPI, BASE_DPI);
    let _ = unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };
    Some((mi.rcWork, dpi_x.max(1)))
}

/// Clamp `r` to fit fully inside `area`, shrinking it if it is too big and
/// enforcing a minimum size (itself capped by the area).
fn fit(r: RECT, area: RECT, min: (i32, i32)) -> RECT {
    let (aw, ah) = (width(&area), height(&area));
    let w = width(&r).clamp(min.0.min(aw), aw);
    let h = height(&r).clamp(min.1.min(ah), ah);
    let left = r.left.clamp(area.left, area.right - w);
    let top = r.top.clamp(area.top, area.bottom - h);
    RECT { left, top, right: left + w, bottom: top + h }
}

fn scale_by(v: i32, num: u32, den: u32) -> i32 {
    (v as f64 * num as f64 / den as f64).round() as i32
}

fn width(r: &RECT) -> i32 {
    r.right - r.left
}

fn height(r: &RECT) -> i32 {
    r.bottom - r.top
}

fn offset(r: RECT, dx: i32, dy: i32) -> RECT {
    RECT { left: r.left + dx, top: r.top + dy, right: r.right + dx, bottom: r.bottom + dy }
}

fn deflate(r: RECT, by: &RECT) -> RECT {
    RECT { left: r.left + by.left, top: r.top + by.top, right: r.right - by.right, bottom: r.bottom - by.bottom }
}

fn inflate(r: RECT, by: &RECT) -> RECT {
    RECT { left: r.left - by.left, top: r.top - by.top, right: r.right + by.right, bottom: r.bottom + by.bottom }
}

fn hwnd_of(window: &Window) -> HWND {
    HWND(window.hwnd() as *mut _)
}
