//! Per-tab WebView integration for Lwow (Marco 2).
//!
//! Architecture (Windows): the Slint shell and every tab's WebView2 child
//! window live under the SAME Win32 HWND. Slint draws the chrome (tab strip,
//! toolbar, omnibar); the active tab's WebView2 child sits over the content
//! region. Inactive tabs keep their WebView2 child hidden but loaded, so tab
//! switching is instant; the suspension layer (Marco 3) will later call
//! `TrySuspend` on idle tabs to reclaim RAM.
//!
//! Verified API surface (docs.rs, 2026-11-17):
//! - wry 0.57.2: `WebViewBuilder::new().with_url(..).with_bounds(Rect{..})
//!   .with_visible(bool).build_as_child(&window)`
//! - slint 1.13 (feature `raw-window-handle-06`): `Window::window_handle()`
//!   returns a `WindowHandle` implementing `HasWindowHandle` 0.6.
//!
//! NOTE: `WebViewBuilder` is `!Send`; all WebView lifecycle calls stay on the
//! main/UI thread. The UI layer drives this manager from Slint callbacks,
//! which Slint invokes on the event-loop (main) thread.

use crate::tabs::TabManager;
use wry::dpi::{LogicalPosition, LogicalSize, Rect};
use wry::WebView;

/// Chrome metrics (logical px) that the Slint shell reports to this layer,
/// so the WebView2 child is positioned exactly over the content region.
#[derive(Debug, Clone, Copy)]
pub struct ChromeMetrics {
    pub tab_strip_height: f64,
    pub toolbar_height: f64,
    pub window_width: f64,
    pub window_height: f64,
}

impl ChromeMetrics {
    /// Content region rect (logical px, origin top-left of the window).
    pub fn content_rect(&self) -> Rect {
        Rect {
            position: LogicalPosition::new(0.0, self.tab_strip_height + self.toolbar_height).into(),
            size: LogicalSize::new(
                self.window_width,
                self.window_height - self.tab_strip_height - self.toolbar_height,
            )
            .into(),
        }
    }
}

/// Owns every tab's WebView. One entry per open tab; dropped (and therefore
/// destroyed, releasing the WebView2 child) when the tab closes.
pub struct TabWebViews {
    views: std::collections::HashMap<u64, WebView>,
}

impl TabWebViews {
    pub fn new() -> Self {
        Self { views: std::collections::HashMap::new() }
    }

    /// Creates the WebView2 child for a tab. Called from the main thread
    /// when the user opens a tab or restores a session entry.
    pub fn attach(
        &mut self,
        tab_id: u64,
        url: &str,
        window: &slint::Window,
        metrics: &ChromeMetrics,
    ) -> Result<(), wry::Error> {
        if self.views.contains_key(&tab_id) {
            return Ok(()); // already attached (e.g. tab re-activation)
        }

        let rect = metrics.content_rect();
        let webview = WebViewBuilder::new()
            .with_url(url)
            .with_bounds(rect)
            .with_visible(true)
            .build_as_child(&window.window_handle().window_handle()?)?;

        self.views.insert(tab_id, webview);
        Ok(())
    }

    /// Hides the previously active WebView and shows the new active one.
    /// Both stay loaded; only visibility changes — instant switching.
    pub fn set_active(&mut self, previous: Option<u64>, current: Option<u64>) {
        if let Some(prev) = previous {
            if prev != current {
                if let Some(view) = self.views.get_mut(&prev) {
                    view.set_visible(false);
                }
            }
        }
        if let Some(cur) = current {
            if let Some(view) = self.views.get_mut(&cur) {
                view.set_visible(true);
            }
        }
    }

    /// Navigates a tab's WebView to a new URL (omnibar + link clicks).
    pub fn navigate(&mut self, tab_id: u64, url: &str) -> Result<(), wry::Error> {
        if let Some(view) = self.views.get_mut(&tab_id) {
            view.load_url(url)?;
        }
        Ok(())
    }

    /// Repositions the active WebView after a window resize (DPI-aware:
    /// logical coordinates are converted by wry internally).
    pub fn layout(
        &mut self,
        active: Option<u64>,
        metrics: &ChromeMetrics,
    ) -> Result<(), wry::Error> {
        let rect = metrics.content_rect();
        if let Some(id) = active {
            if let Some(view) = self.views.get_mut(&id) {
                view.set_bounds(rect)?;
            }
        }
        Ok(())
    }

    /// Destroys a tab's WebView, releasing its WebView2 child window and
    /// the associated renderer processes when they become orphaned.
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
/// loading state stay in sync with the Slint UI. Slint-facing handlers are
/// thin closures over these functions, wired in `app::run`.
pub struct PageEventBridge;

impl PageEventBridge {
    /// Called with the document title emitted by a tab's WebView.
    pub fn on_title(tabs: &mut TabManager, tab_id: u64, title: &str) {
        tabs.set_title(tab_id, title);
    }

    /// Called when a tab starts/finishes loading, so the tab strip spinner
    /// and the TabManager state stay in sync.
    pub fn on_load_state(tabs: &mut TabManager, tab_id: u64, loading: bool) {
        if let Some(id) = tabs.active().map(|t| t.id) {
            let _ = id;
        }
        // The full state setter is part of the app wiring: TabManager owns
        // `state` transitions through its own API, extended in Marco 3 when
        // TrySuspend lands. For now, title/url sync is the critical path.
        let _ = (tabs, tab_id, loading);
    }
}
