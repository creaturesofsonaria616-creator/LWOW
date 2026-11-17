//! Application orchestration: state, settings, and the Slint event loop
//! wired to the tab manager and the per-tab WebView2 layer (Marco 2).

use crate::store::Store;
use crate::themes::Theme;
use crate::webviews::{ChromeMetrics, TabWebViews};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

slint::include_modules!();

#[derive(Debug, Clone)]
pub struct Settings {
    pub restore_session_on_startup: bool,
    pub startup_page: StartupPage,
    pub window_width: f32,
    pub window_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StartupPage {
    StartPage,
    HomePage,
    LastSession,
}

pub struct AppState {
    pub store: Arc<Store>,
    pub theme: Theme,
    pub settings: Settings,
    pub tabs: crate::tabs::TabManager,
}

impl AppState {
    pub fn new(store: Store, theme: Theme, settings: Settings) -> Self {
        Self {
            store: Arc::new(store),
            theme,
            settings,
            tabs: crate::tabs::TabManager::new(),
        }
    }
}

impl Settings {
    pub fn load(store: &Store) -> Self {
        Self {
            restore_session_on_startup: store
                .get_bool("settings.restore_session_on_startup")
                .unwrap_or(true),
            startup_page: match store.get_string("settings.startup_page").as_deref() {
                Some("home") => StartupPage::HomePage,
                Some("last") => StartupPage::LastSession,
                _ => StartupPage::StartPage,
            },
            window_width: store.get_f64("settings.window_width").unwrap_or(1280.0) as f32,
            window_height: store.get_f64("settings.window_height").unwrap_or(800.0) as f32,
        }
    }

    pub fn save(&self, store: &Store) {
        store.set_string(
            "settings.startup_page",
            match self.startup_page {
                StartupPage::StartPage => "start",
                StartupPage::HomePage => "home",
                StartupPage::LastSession => "last",
            },
        );
        store.set_f64("settings.window_width", self.window_width as f64);
        store.set_f64("settings.window_height", self.window_height as f64);
    }
}

/// Normalizes omnibar input into a loadable URL.
fn normalize_omnibar(input: &str) -> String {
    let input = input.trim();
    if input.is_empty() {
        return String::new();
    }
    let looks_like_url =
        input.contains("://") || input.starts_with("localhost") || (input.contains('.') && !input.contains(' '));
    if looks_like_url {
        if input.contains("://") {
            input.to_string()
        } else {
            format!("https://{input}")
        }
    } else {
        format!("https://duckduckgo.com/?q={}", url_encode(input))
    }
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

struct SharedRuntime {
    tabs: crate::tabs::TabManager,
    views: TabWebViews,
}

fn sync_tabs(ui: &slint::Weak<BrowserWindow>, rt: &SharedRuntime) {
    if let Some(ui) = ui.upgrade() {
        let rows: Vec<TabData> = rt
            .tabs
            .tabs()
            .iter()
            .map(|t| TabData {
                id: t.id as i32,
                title: t.title.clone().into(),
                url: t.url.clone().into(),
                pinned: t.pinned,
                loading: matches!(t.state, crate::tabs::TabState::Loading),
                playing_audio: t.is_playing_audio,
            })
            .collect();
        ui.set_tabs(rows.into());
        if let Some(active) = rt.tabs.active() {
            let idx = rt
                .tabs
                .tabs()
                .iter()
                .position(|t| t.id == active.id)
                .unwrap_or(0);
            ui.set_active_index(idx as i32);
        }
    }
}

pub fn run(state: AppState) {
    let AppState { store, theme: _theme, settings, mut tabs } = state;

    let ui = BrowserWindow::new().expect("failed to instantiate Slint shell");
    ui.window().on_close_requested(|| {
        slint::CloseRequestResponse::hide_and_stop_event_loop()
    });

    let shared = Rc::new(RefCell::new(SharedRuntime { tabs, views: TabWebViews::new() }));

    // Tab strip callbacks
    let weak = ui.as_weak();
    let s = shared.clone();
    ui.on_open_new_tab(move || {
        let mut rt = s.borrow_mut();
        rt.tabs.open_tab("about:blank");
        sync_tabs(&weak, &rt);
    });

    let weak = ui.as_weak();
    let s = shared.clone();
    ui.on_close_tab(move |tab_id| {
        let mut rt = s.borrow_mut();
        if let Some(closed) = rt.tabs.close_tab(tab_id as u64) {
            rt.views.detach(closed.id);
        }
        sync_tabs(&weak, &rt);
    });

    let weak = ui.as_weak();
    let s = shared.clone();
    ui.on_select_tab(move |tab_id| {
        let mut rt = s.borrow_mut();
        let previous = rt.tabs.active().map(|t| t.id);
        rt.tabs.set_active(tab_id as u64);
        rt.views.set_active(previous, rt.tabs.active().map(|t| t.id));
        sync_tabs(&weak, &rt);
    });

    // Omnibar
    let s = shared.clone();
    let weak = ui.as_weak();
    ui.on_navigate(move |input| {
        let url = normalize_omnibar(&input);
        if url.is_empty() {
            return;
        }
        let mut rt = s.borrow_mut();
        if let Some(id) = rt.tabs.active().map(|t| t.id) {
            rt.tabs.set_url(id, &url);
            let _ = rt.views.navigate(id, &url);
        }
        if let Some(ui) = weak.upgrade() {
            ui.set_address(url.into());
        }
    });

    // Back/Forward/Reload: native WebView2 commands land in Marco 3.
    // No fake behavior until then.
    ui.on_go_back(|| {});
    ui.on_go_forward(|| {});
    ui.on_reload(|| {});

    // Startup tab
    let initial_url = match settings.startup_page {
        StartupPage::HomePage => "https://duckduckgo.com".to_string(),
        _ => "about:blank".to_string(),
    };
    {
        let mut rt = shared.borrow_mut();
        rt.tabs.open_tab(&initial_url);
        sync_tabs(&ui.as_weak(), &rt);
    }

    ui.show();

    // Attach + layout timer: the Slint HWND only becomes available after
    // show(), so WebView2 attachment is retried here until it succeeds.
    let weak = ui.as_weak();
    let s = shared.clone();
    let timer = slint::Timer::default();
    {
        let mut timer = timer;
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(100),
            move || {
                let Ok(ui) = weak.upgrade() else { return };
                let size = ui.window().size();
                let metrics = ChromeMetrics {
                    tab_strip_height: 40.0,
                    toolbar_height: 44.0,
                    window_width: size.width as f64,
                    window_height: size.height as f64,
                };
                let mut rt = s.borrow_mut();
                for tab in rt.tabs.tabs().to_vec() {
                    if !rt.views.is_attached(tab.id) {
                        let _ = rt.views.attach(tab.id, &tab.url, &ui.window(), &metrics);
                    }
                }
                let active = rt.tabs.active().map(|t| t.id);
                let _ = rt.views.layout(active, &metrics);
            },
        );
    }

    // Persist window geometry on close.
    let store_close = store.clone();
    {
        let weak = ui.as_weak();
        ui.window().on_close_requested(move || {
            if let Some(ui) = weak.upgrade() {
                let size = ui.window().size();
                store_close.set_f64("settings.window_width", size.width as f64);
                store_close.set_f64("settings.window_height", size.height as f64);
            }
            slint::CloseRequestResponse::hide_and_stop_event_loop()
        });
    }

    slint::run_event_loop_until_quit().expect("Slint event loop failed");
}
