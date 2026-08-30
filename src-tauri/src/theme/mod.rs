use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OmarchyTheme {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub colors: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct ColorsToml {
    mode: Option<String>,
    accent: Option<String>,
    selection: Option<String>,
    muted: Option<String>,
    background: Option<String>,
    dark_background: Option<String>,
    darker_background: Option<String>,
    lighter_background: Option<String>,
    foreground: Option<String>,
    dark_foreground: Option<String>,
    light_foreground: Option<String>,
    bright_foreground: Option<String>,
    red: Option<String>,
    yellow: Option<String>,
    orange: Option<String>,
    green: Option<String>,
    cyan: Option<String>,
    blue: Option<String>,
    magenta: Option<String>,
    brown: Option<String>,
    bright_red: Option<String>,
    bright_yellow: Option<String>,
    bright_green: Option<String>,
    bright_cyan: Option<String>,
    bright_blue: Option<String>,
    bright_magenta: Option<String>,
}

pub fn detect_theme() -> Result<OmarchyTheme, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;

    // Check the standard Omarchy theme location
    let theme_dir = home.join(".local/state/omarchy/current/theme");
    let theme_name_file = home.join(".local/state/omarchy/current/theme.name");
    let colors_file = theme_dir.join("colors.toml");

    if colors_file.exists() {
        // Read theme name
        let theme_name = if theme_name_file.exists() {
            fs::read_to_string(&theme_name_file)
                .unwrap_or_else(|_| "Omarchy".to_string())
                .trim()
                .to_string()
        } else {
            "Omarchy".to_string()
        };

        return parse_omarchy_theme(&colors_file, &theme_name);
    }

    // Fallback: generate a default dark theme
    Ok(default_theme())
}

fn parse_omarchy_theme(path: &PathBuf, theme_name: &str) -> Result<OmarchyTheme, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read theme file: {}", e))?;

    let parsed: ColorsToml =
        toml::from_str(&content).map_err(|e| format!("Failed to parse theme file: {}", e))?;

    let mut color_map = HashMap::new();

    // Insert all colors from the TOML file
    if let Some(v) = &parsed.background {
        color_map.insert("background".to_string(), v.clone());
    }
    if let Some(v) = &parsed.foreground {
        color_map.insert("foreground".to_string(), v.clone());
    }
    if let Some(v) = &parsed.accent {
        color_map.insert("accent".to_string(), v.clone());
    }
    if let Some(v) = &parsed.selection {
        color_map.insert("selection".to_string(), v.clone());
    }
    if let Some(v) = &parsed.muted {
        color_map.insert("muted".to_string(), v.clone());
    }
    if let Some(v) = &parsed.dark_background {
        color_map.insert("dark_background".to_string(), v.clone());
    }
    if let Some(v) = &parsed.darker_background {
        color_map.insert("darker_background".to_string(), v.clone());
    }
    if let Some(v) = &parsed.lighter_background {
        color_map.insert("lighter_background".to_string(), v.clone());
    }
    if let Some(v) = &parsed.dark_foreground {
        color_map.insert("dark_foreground".to_string(), v.clone());
    }
    if let Some(v) = &parsed.light_foreground {
        color_map.insert("light_foreground".to_string(), v.clone());
    }
    if let Some(v) = &parsed.bright_foreground {
        color_map.insert("bright_foreground".to_string(), v.clone());
    }
    if let Some(v) = &parsed.red {
        color_map.insert("red".to_string(), v.clone());
        color_map.insert("danger".to_string(), v.clone());
    }
    if let Some(v) = &parsed.yellow {
        color_map.insert("yellow".to_string(), v.clone());
        color_map.insert("warning".to_string(), v.clone());
    }
    if let Some(v) = &parsed.orange {
        color_map.insert("orange".to_string(), v.clone());
    }
    if let Some(v) = &parsed.green {
        color_map.insert("green".to_string(), v.clone());
        color_map.insert("success".to_string(), v.clone());
    }
    if let Some(v) = &parsed.cyan {
        color_map.insert("cyan".to_string(), v.clone());
        color_map.insert("info".to_string(), v.clone());
    }
    if let Some(v) = &parsed.blue {
        color_map.insert("blue".to_string(), v.clone());
    }
    if let Some(v) = &parsed.magenta {
        color_map.insert("magenta".to_string(), v.clone());
    }
    if let Some(v) = &parsed.brown {
        color_map.insert("brown".to_string(), v.clone());
    }
    if let Some(v) = &parsed.bright_red {
        color_map.insert("bright_red".to_string(), v.clone());
    }
    if let Some(v) = &parsed.bright_yellow {
        color_map.insert("bright_yellow".to_string(), v.clone());
    }
    if let Some(v) = &parsed.bright_green {
        color_map.insert("bright_green".to_string(), v.clone());
    }
    if let Some(v) = &parsed.bright_cyan {
        color_map.insert("bright_cyan".to_string(), v.clone());
    }
    if let Some(v) = &parsed.bright_blue {
        color_map.insert("bright_blue".to_string(), v.clone());
    }
    if let Some(v) = &parsed.bright_magenta {
        color_map.insert("bright_magenta".to_string(), v.clone());
    }

    // Derive colors from background
    let background = parsed.background.unwrap_or_else(|| "#121212".to_string());
    let foreground = parsed.foreground.unwrap_or_else(|| "#bebebe".to_string());
    let accent = parsed.accent.unwrap_or_else(|| "#e68e0d".to_string());

    let bg_rgb = hex_to_rgb(&background);
    let surface = rgb_to_hex(
        bg_rgb.0.saturating_sub(10),
        bg_rgb.1.saturating_sub(10),
        bg_rgb.2.saturating_sub(10),
    );
    let surface_hover = rgb_to_hex(
        bg_rgb.0.saturating_sub(20),
        bg_rgb.1.saturating_sub(20),
        bg_rgb.2.saturating_sub(20),
    );
    color_map.insert("surface".to_string(), surface);
    color_map.insert("surface_hover".to_string(), surface_hover);

    let border = rgb_to_hex(
        bg_rgb.0.saturating_sub(20),
        bg_rgb.1.saturating_sub(20),
        bg_rgb.2.saturating_sub(20),
    );
    color_map.insert("border".to_string(), border);

    let fg_rgb = hex_to_rgb(&foreground);
    let muted = rgb_to_hex(
        (fg_rgb.0 as u32 * 60 / 100) as u8,
        (fg_rgb.1 as u32 * 60 / 100) as u8,
        (fg_rgb.2 as u32 * 60 / 100) as u8,
    );
    color_map.insert("foreground_muted".to_string(), muted);

    let subtle = rgb_to_hex(
        (fg_rgb.0 as u32 * 40 / 100) as u8,
        (fg_rgb.1 as u32 * 40 / 100) as u8,
        (fg_rgb.2 as u32 * 40 / 100) as u8,
    );
    color_map.insert("foreground_subtle".to_string(), subtle);

    Ok(OmarchyTheme {
        name: theme_name.to_string(),
        background,
        foreground,
        accent,
        colors: color_map,
    })
}

fn default_theme() -> OmarchyTheme {
    let mut colors = HashMap::new();
    colors.insert("background".to_string(), "#1a1b26".to_string());
    colors.insert("foreground".to_string(), "#c0caf5".to_string());
    colors.insert("surface".to_string(), "#24283b".to_string());
    colors.insert("surface_hover".to_string(), "#2f3349".to_string());
    colors.insert("border".to_string(), "#3b4261".to_string());
    colors.insert("accent".to_string(), "#7aa2f7".to_string());
    colors.insert("success".to_string(), "#9ece6a".to_string());
    colors.insert("danger".to_string(), "#f7768e".to_string());
    colors.insert("warning".to_string(), "#e0af68".to_string());
    colors.insert("info".to_string(), "#7dcfff".to_string());
    colors.insert("foreground_muted".to_string(), "#737aa2".to_string());
    colors.insert("foreground_subtle".to_string(), "#565f89".to_string());
    colors.insert("black".to_string(), "#15161e".to_string());
    colors.insert("red".to_string(), "#f7768e".to_string());
    colors.insert("green".to_string(), "#9ece6a".to_string());
    colors.insert("yellow".to_string(), "#e0af68".to_string());
    colors.insert("blue".to_string(), "#7aa2f7".to_string());
    colors.insert("magenta".to_string(), "#bb9af7".to_string());
    colors.insert("cyan".to_string(), "#7dcfff".to_string());
    colors.insert("white".to_string(), "#a9b1d6".to_string());

    OmarchyTheme {
        name: "Tokyo Night".to_string(),
        background: "#1a1b26".to_string(),
        foreground: "#c0caf5".to_string(),
        accent: "#7aa2f7".to_string(),
        colors,
    }
}

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    (r, g, b)
}

fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}
