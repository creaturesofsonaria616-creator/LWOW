//! Suspension policy: when idle tabs release their WebView2 memory.
//!
//! On Windows, wry/webview2-com exposes `ICoreWebView2_3::TrySuspend`, which
//! suspends the underlying renderer process of a WebView2 instance. This is
//! the mechanism that makes suspended tabs cheap in RAM; the policy below
//! decides *when* it fires, per performance mode.

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceMode {
    Economia,
    Equilibrado,
    Desempenho,
}

impl PerformanceMode {
    /// Idle time before an eligible tab is suspended via TrySuspend.
    pub fn suspend_after(&self) -> Duration {
        match self {
            PerformanceMode::Economia => Duration::from_secs(60),
            PerformanceMode::Equilibrado => Duration::from_secs(300),
            PerformanceMode::Desempenho => Duration::from_secs(1800),
        }
    }

    /// Whether low-priority background work (preconnect, cache warming) runs.
    pub fn allows_background_hints(&self) -> bool {
        matches!(self, PerformanceMode::Equilibrado | PerformanceMode::Desempenho)
    }

    /// Seconds between periodic session autosaves to SQLite.
    pub fn autosave_interval(&self) -> Duration {
        match self {
            PerformanceMode::Economia => Duration::from_secs(120),
            PerformanceMode::Equilibrado => Duration::from_secs(30),
            PerformanceMode::Desempenho => Duration::from_secs(30),
        }
    }

    pub fn from_setting(value: &str) -> Self {
        match value {
            "economia" => PerformanceMode::Economia,
            "desempenho" => PerformanceMode::Desempenho,
            _ => PerformanceMode::Equilibrado,
        }
    }

    pub fn as_setting(&self) -> &'static str {
        match self {
            PerformanceMode::Economia => "economia",
            PerformanceMode::Equilibrado => "equilibrado",
            PerformanceMode::Desempenho => "desempenho",
        }
    }
}
