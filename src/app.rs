//! Application orchestration: state, settings, themes, and the event loop.

use crate::store::Store;
use crate::themes::Theme;
use std::sync::Arc;

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

/// Enters the Slint event loop until the last Lwow window is closed.
pub fn run(state: AppState) {
    let _ = state; // wired to the Slint shell in the next milestone.
    slint::run_event_loop_until_quit();
}
