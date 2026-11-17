//! Theme engine for Lwow Themes: palette model, presets, persistence.
//!
//! A theme is a flat palette the Slint shell reads at render time. All colors
//! are stored as #RRGGBB strings so themes round-trip losslessly to JSON for
//! import/export in the visual theme editor.

use crate::store::Store;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub accent: String,
    pub background: String,
    pub surface: String,
    pub text: String,
    pub text_muted: String,
    pub border: String,
    pub radius: f32,
}

impl Theme {
    /// Loads the active theme name from the store; falls back to Graphite.
    pub fn load_active(store: &Store) -> Self {
        match store.get_string("theme.active").as_deref() {
            Some("snow") => Self::snow(),
            Some("aurora") => Self::aurora(),
            Some("amber") => Self::amber(),
            _ => Self::graphite(),
        }
    }

    pub fn save_active(&self, store: &Store) {
        let key = match self.name.as_str() {
            "Lwow Snow" => "snow",
            "Lwow Aurora" => "aurora",
            "Lwow Amber" => "amber",
            _ => "graphite",
        };
        store.set_string("theme.active", key);
    }

    pub fn graphite() -> Self {
        Self {
            name: "Lwow Graphite".into(),
            accent: "#7C5CFF".into(),
            background: "#0F1419".into(),
            surface: "#1A2129".into(),
            text: "#F4F6F8".into(),
            text_muted: "#8B95A1".into(),
            border: "#2A323C".into(),
            radius: 10.0,
        }
    }

    pub fn snow() -> Self {
        Self {
            name: "Lwow Snow".into(),
            accent: "#7C5CFF".into(),
            background: "#F4F6F8".into(),
            surface: "#FFFFFF".into(),
            text: "#16181D".into(),
            text_muted: "#5B6470".into(),
            border: "#E1E5EA".into(),
            radius: 10.0,
        }
    }

    pub fn aurora() -> Self {
        Self {
            name: "Lwow Aurora".into(),
            accent: "#22D3EE".into(),
            background: "#0B1020".into(),
            surface: "#151C33".into(),
            text: "#EDF2FF".into(),
            text_muted: "#8B95A1".into(),
            border: "#243052".into(),
            radius: 12.0,
        }
    }

    pub fn amber() -> Self {
        Self {
            name: "Lwow Amber".into(),
            accent: "#F59E0B".into(),
            background: "#141210".into(),
            surface: "#201C18".into(),
            text: "#F5F2ED".into(),
            text_muted: "#97928A".into(),
            border: "#332E28".into(),
            radius: 8.0,
        }
    }

    pub fn presets() -> Vec<Theme> {
        vec![Self::graphite(), Self::snow(), Self::aurora(), Self::amber()]
    }
}
