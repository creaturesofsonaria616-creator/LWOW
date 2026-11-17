//! Per-tab WebView integration for Lwow (Marco 2).
//!
//! The Slint shell and every tab's WebView2 child window live under the SAME
//! Win32 HWND. Slint draws the chrome; the active tab's WebView2 child sits
//! over the content region. Inactive tabs stay hidden but loaded, so tab
//! switching is instant. Marco 3 adds TrySuspend for idle tabs.
//!
//! Verified API surface (docs.rs wry 0.57.2, 2026-11-17):
//! - `wry::Rect { position: wry::dpi::Position, size: wry::dpi::Size }`
//!   (Rect lives at the wry ROOT, not in wry::dpi)
//! - `wry::raw_window_handle::HasWindowHandle` (wry re-exports the crate)
//! - `WebViewBuilder::new().with_url(..).with_bounds(rect)
//!   .with_visible(bool).build_as_child(&window)`
//! - WebView: `load_url`, `set_bounds`, `set_visible`
//!
//! NOTE: WebViewBuilder is !Send; all WebView lifecycle calls stay on the
//! main/UI thread (Slint callbacks and timers run there).

use crate::tabs::TabManager;
use std::collections::HashMap;
use wry::dpi::{LogicalPosition, LogicalSize, Position, Size};
use wry::raw_window_handle::HasWindowHandle;
use wry::{Rect, WebView, WebViewBuilder};

#[derive(Debug, thiserror::Error)]
pub enum WebviewError {
    #[error("wry error: {0}")]
    Wry(#[from] wry::Error),
}

/// Chrome metrics (logical px) reported by the Slint shell, so the WebView2
/// child is positioned exactly over the content region.
#[derive(Debug, Clone, Copy)]
pub struct ChromeMetrics {
    pub tab_strip_height: f64,
    pub toolbar_height: f64,
    pub window_width: f64,
    pub window_height: f64,
}

impl ChromeMetrics {
    /// Content region rect (logical px, origin at the window top-left).
    pub fn content_rect(&self) -> Rect {
        let top = self.tab_strip_height + self.toolbar_height;
        Rect {
            position: Position::Logical(LogicalPosition::new(0.0, top)),
            size: Size::Logical(LogicalSize::new(
                self.window_width,
                (self.window_height - top).max(0.0),
            )),
        }
    }
}

/// Owns every tab's WebView. One entry per open tab; dropping the entry
/// destroys the WebView2 child window and releases its resources.
pub struct TabWebViews {
    views: HashMap<u64, WebView>,
}

impl TabWebViews {
    pub fn new() -> Self {
        Self { views: HashMap::new() }
    }

    /// Creates the WebView2 child for a tab. Main-thread only.
    pub fn attach(
        &mut self,
        tab_id: u64,
        url: &str,
        window: &slint::Window,
        metrics: &ChromeMetrics,
    ) -> Result<(), WebviewError> {
        if self.views.contains_key(&tab_id) {
            return Ok(()); // already attached
        }

        let rect = metrics.content_rect();
        let webview = WebViewBuilder::new()
            .with_url(url)
            .with_bounds(rect)
            .with_visible(true)
            .build_as_child(&window.window_handle())?;

        self.views.insert(tab_id, webview);
        Ok(())
    }

    /// Shows/hides a single tab's WebView (tab switching).
    pub fn set_visible(&mut self, tab_id: u64, visible: bool) {
        if let Some(view) = self.views.get_mut(&tab_id) {
            let _ = view.set_visible(visible);
        }
    }

    /// Hides the previously active WebView and shows the new active one.
    pub fn set_active(&mut self, previous: Option<u64>, current: Option<u64>) {
        if let Some(prev) = previous {
            if Some(prev) != current {
                self.set_visible(prev, false);
            }
        }
        if let Some(cur) = current {
            self.set_visible(cur, true);
        }
    }

    /// Navigates a tab's WebView to a new URL (omnibar).
    pub fn navigate(&mut self, tab_id: u64, url: &str) -> Result<(), wry::Error> {
        if let Some(view) = self.views.get_mut(&tab_id) {
            view.load_url(url)?;
        }
        Ok(())
    }

    /// Repositions the active WebView after window resize (DPI-aware).
    pub fn layout(&mut self, active: Option<u64>, metrics: &ChromeMetrics) -> Result<(), wry::Error> {
        let rect = metrics.content_rect();
        if let Some(id) = active {
            if let Some(view) = self.views.get_mut(&id) {
                let _ = view.set_bounds(rect);
            }
        }
        Ok(())
    }

    /// Destroys a tab's WebView, releasing its WebView2 child window.
    pub fn detach(&mut self, tab_id: u64) {
        self.views.remove(&tab_id);
    }

    pub fn is_attached(&self, tab_id: u64) -> bool {
        self.views.contains_key(&tab_id)
    }

    pub fn count(&self) -> usize {
        self.views.len()
    }
}

impl Default for TabWebViews {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridges page events back into the TabManager so tab titles, URLs and
/// loading state stay in sync with the Slint UI.
pub struct PageEventBridge;

impl PageEventBridge {
    /// Document title emitted by a tab's WebView.
    pub fn on_title(tabs: &mut TabManager, tab_id: u64, title: &str) {
        tabs.set_title(tab_id, title);
    }

    /// Tab load state changes (tab strip spinner).
    pub fn on_load_state(tabs: &mut TabManager, tab_id: u64, loading: bool) {
        tabs.set_state(
            tab_id,
            if loading {
                crate::tabs::TabState::Loading
            } else {
                crate::tabs::TabState::Active
            },
        );
    }
}
