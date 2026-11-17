//! WebView2 bridge: one child webview per tab, attached to the Slint window.
//!
//! Slint exposes the native HWND through raw-window-handle 0.6 (the
//! `raw-window-handle-06` feature). wry consumes any `HasWindowHandle`, so
//! WebView2 child windows are created against the same Win32 window that
//! renders the Slint chrome: chrome on top, webview below the toolbar,
//! repositioned by the layout engine on resize.
//!
//! Reference (verified 2026-11-17): wry 0.57.2 `WebViewBuilder::build_as_child`
//! supports child webviews on Windows/macOS/Linux-X11; the Windows engine is
//! WebView2 (Chromium), served Evergreen by Microsoft.

use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// Extracts the Win32 HWND from a Slint window handle.
pub fn hwnd_of(window: &slint::Window) -> Option<*mut core::ffi::c_void> {
    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(win32) => Some(win32.hwnd.get() as *mut core::ffi::c_void),
        _ => None,
    }
}

/// Reserves the child webview geometry for a tab. The webview layer calls
/// this on layout changes so the WebView2 child stays in sync with the
/// Slint-drawn chrome (tab strip height, toolbar height, DPI scale).
#[derive(Debug, Clone, Copy)]
pub struct WebviewGeometry {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
