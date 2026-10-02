//! Small UI preferences persisted next to the database.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Prefs {
    #[serde(default)]
    pub sidebar_collapsed: bool,
}

fn path() -> std::path::PathBuf {
    crate::database::data_dir().join("ui.json")
}

impl Prefs {
    pub fn load() -> Self {
        std::fs::read_to_string(path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::create_dir_all(crate::database::data_dir());
            let _ = std::fs::write(path(), json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let p: Prefs = serde_json::from_str("{}").unwrap();
        assert!(!p.sidebar_collapsed);
    }
}
