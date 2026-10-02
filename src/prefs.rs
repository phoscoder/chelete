//! Small UI preferences persisted next to the database.
use serde::{Deserialize, Serialize};

/// Text size as a percentage of the toolkit's 16px base.
pub const TEXT_SCALES: [u16; 9] = [70, 80, 90, 100, 110, 125, 150, 175, 200];
pub const DEFAULT_TEXT_SCALE: u16 = 110;

/// The next larger (`direction > 0`) or smaller (`< 0`) step, staying put at
/// either end. A value between steps moves to the nearest step in that direction.
pub fn step_text_scale(current: u16, direction: i8) -> u16 {
    let steps = TEXT_SCALES;
    if direction > 0 {
        steps.into_iter().find(|s| *s > current).unwrap_or(steps[steps.len() - 1])
    } else if direction < 0 {
        steps.into_iter().rev().find(|s| *s < current).unwrap_or(steps[0])
    } else {
        current
    }
}

pub fn clamp_text_scale(percent: u16) -> u16 {
    percent.clamp(TEXT_SCALES[0], TEXT_SCALES[TEXT_SCALES.len() - 1])
}

/// Pixels for one rem at the given text size.
pub fn rem_size_px(percent: u16) -> f32 {
    16.0 * f32::from(clamp_text_scale(percent)) / 100.0
}

fn default_text_scale() -> u16 {
    DEFAULT_TEXT_SCALE
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prefs {
    #[serde(default)]
    pub sidebar_collapsed: bool,
    #[serde(default = "default_text_scale")]
    pub text_scale: u16,
}

impl Default for Prefs {
    fn default() -> Self {
        Self { sidebar_collapsed: false, text_scale: DEFAULT_TEXT_SCALE }
    }
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
        assert_eq!(p.text_scale, DEFAULT_TEXT_SCALE);
        assert_eq!(p, Prefs::default());
    }

    #[test]
    fn files_written_before_text_size_existed_still_load() {
        let p: Prefs = serde_json::from_str(r#"{"sidebar_collapsed": true}"#).unwrap();
        assert!(p.sidebar_collapsed);
        assert_eq!(p.text_scale, DEFAULT_TEXT_SCALE);
    }

    #[test]
    fn stepping_moves_one_notch_and_stops_at_the_ends() {
        assert_eq!(step_text_scale(100, 1), 110);
        assert_eq!(step_text_scale(110, 1), 125);
        assert_eq!(step_text_scale(110, -1), 100);
        assert_eq!(step_text_scale(200, 1), 200);
        assert_eq!(step_text_scale(70, -1), 70);
        assert_eq!(step_text_scale(110, 0), 110);
    }

    #[test]
    fn stepping_from_between_notches_goes_to_the_nearest_one_in_that_direction() {
        assert_eq!(step_text_scale(105, 1), 110);
        assert_eq!(step_text_scale(105, -1), 100);
        assert_eq!(step_text_scale(500, -1), 200, "an out-of-range value comes back to the top step");
        assert_eq!(step_text_scale(1, 1), 70);
    }

    #[test]
    fn every_step_maps_to_a_sensible_rem_size() {
        assert_eq!(rem_size_px(100), 16.0);
        assert!((rem_size_px(110) - 17.6).abs() < 1e-4);
        assert_eq!(rem_size_px(200), 32.0);
        assert_eq!(rem_size_px(10), 11.2, "clamped to the smallest step");
        assert_eq!(rem_size_px(900), 32.0, "clamped to the largest step");
        for s in TEXT_SCALES {
            assert_eq!(clamp_text_scale(s), s);
        }
    }

    #[test]
    fn the_default_is_one_of_the_offered_steps() {
        assert!(TEXT_SCALES.contains(&DEFAULT_TEXT_SCALE));
    }
}
