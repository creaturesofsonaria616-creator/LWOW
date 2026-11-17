//! Tab management: creation, activation, closing, suspension candidates.

use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum TabState {
    Active,
    Loading,
    Suspended,
    Discarded,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub pinned: bool,
    pub state: TabState,
    pub is_playing_audio: bool,
    pub last_active: Option<Instant>,
}

pub struct TabManager {
    tabs: Vec<Tab>,
    next_id: u64,
    active_tab: Option<u64>,
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TabManager {
    pub fn new() -> Self {
        Self { tabs: Vec::new(), next_id: 1, active_tab: None }
    }

    pub fn open_tab(&mut self, url: &str) -> &Tab {
        let tab = Tab {
            id: self.next_id,
            title: String::new(),
            url: url.to_string(),
            pinned: false,
            state: TabState::Loading,
            is_playing_audio: false,
            last_active: Some(Instant::now()),
        };
        self.next_id += 1;
        self.tabs.push(tab);
        self.active_tab = Some(self.tabs.last().unwrap().id);
        self.tabs.last().unwrap()
    }

    pub fn close_tab(&mut self, id: u64) -> Option<Tab> {
        let pos = self.tabs.iter().position(|t| t.id == id)?;
        let closed = self.tabs.remove(pos);
        // Focus the neighbour so the UI never points at a dead tab.
        if self.active_tab == Some(id) {
            self.active_tab = self
                .tabs
                .get(pos.min(self.tabs.len().saturating_sub(1)))
                .map(|t| t.id);
        }
        Some(closed)
    }

    pub fn active(&self) -> Option<&Tab> {
        self.active_tab.and_then(|id| self.tabs.iter().find(|t| t.id == id))
    }

    pub fn set_active(&mut self, id: u64) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.last_active = Some(Instant::now());
            tab.state = TabState::Active;
            self.active_tab = Some(id);
        }
    }

    /// Marks the load state of a tab (spinner in the tab strip).
    pub fn set_state(&mut self, id: u64, state: TabState) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.state = state;
        }
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn count(&self) -> usize {
        self.tabs.len()
    }

    /// Tabs eligible for automatic suspension: not active, not pinned,
    /// no audio playing, idle for at least `min_idle_secs`.
    pub fn suspend_candidates(&self, min_idle_secs: u64) -> Vec<u64> {
        self.tabs
            .iter()
            .filter(|t| {
                !t.pinned
                    && !t.is_playing_audio
                    && Some(t.id) != self.active_tab
                    && t.state != TabState::Suspended
                    && t.state != TabState::Discarded
                    && t.last_active
                        .map(|la| la.elapsed().as_secs() >= min_idle_secs)
                        .unwrap_or(false)
            })
            .map(|t| t.id)
            .collect()
    }

    pub fn mark_audio(&mut self, id: u64, playing: bool) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.is_playing_audio = playing;
        }
    }

    pub fn set_title(&mut self, id: u64, title: &str) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.title = title.to_string();
        }
    }

    pub fn set_url(&mut self, id: u64, url: &str) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.url = url.to_string();
        }
    }
}
