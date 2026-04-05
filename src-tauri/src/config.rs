use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub triggers: TriggerConfig,
    pub theme:    ThemeConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            triggers: TriggerConfig::default(),
            theme:    ThemeConfig::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub background_color:    String,
    pub background_opacity:   f32,
    pub font_size:   u32,
    pub font_color:           String,
    pub border_radius:        u32,
    pub border_color:         String,
    pub border_thickness:     u32,
    pub item_border_radius:   u32,
    pub active_bg_color:      String,
    pub active_border_color:  String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background_color:    "rgba(10, 10, 10, 1)".into(),
            background_opacity:  1.0,
            font_size:  14,
            font_color:          "rgba(255, 255, 255, 1)".into(),
            border_radius:       14,
            border_color:        "rgba(255,255,255,0.35)".into(),
            border_thickness:    1,
            item_border_radius:  12,
            active_bg_color:     "rgba(40, 40, 40, 1)".into(),
            active_border_color: "rgba(255,255,255,0.1)".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Triggers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TriggerConfig {
    pub camera:       String,
    pub bookmarks:    String,
    pub files:        String,
    pub clipboard:    String,
    pub emojis:       String,
    pub google:       String,
    pub youtube:      String,
    pub nix:          String,
    pub github:       String,
    pub shell:        String,
    pub lorem:        String,
    pub math:         String,
    pub dictionary:   String,
    pub system:       String,
    pub color_picker: String,
    pub apps:         String,
    pub url:          String,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            camera:       r"^cam$".into(),
            bookmarks:    r"^bk\s+(.*)$".into(),
            files:        r"^f\s+(.*)$".into(),
            clipboard:    r"^cp\s+(.*)$".into(),
            emojis:       r"^em\s+(.*)$".into(),
            google:       r"^g\s+(.*)$".into(),
            youtube:      r"^yt\s+(.*)$".into(),
            nix:          r"^nxp\s+(.*)$".into(),
            github:       r"^gh\s+(.*)$".into(),
            shell:        r"^!\s+(.*)$".into(),
            lorem:        r"^lorem\s+(.*)$".into(),
            math:         r"^=\s*(.*)$".into(),
            dictionary:   r"^def\s+(.*)$".into(),
            system:       r"^sys\s+(.*)$".into(),
            color_picker: r"^color$".into(),
            apps:         r"^app\s+(.*)$".into(),
            url:          r"^(https?://\S+|(?:[a-zA-Z0-9](?:[a-zA-Z0-9\-]*[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}(?:[:/]\S*)?)$".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Loading / writing
// ---------------------------------------------------------------------------

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config/quarry/config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            return Self::default();
        }
        let contents = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("quarry: could not read config ({}), using defaults", e);
                return Self::default();
            }
        };
        match toml::from_str::<Config>(&contents) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("quarry: config parse error — {} — using defaults", e);
                Self::default()
            }
        }
    }

    pub fn write_default_if_missing() {
        let path = Self::config_path();
        if path.exists() {
            return;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&path, DEFAULT_CONFIG_TOML).ok();
    }
}

const DEFAULT_CONFIG_TOML: &str = r#"# Quarry configuration — ~/.config/quarry/config.toml
# Every key is optional. Remove or comment out any line to keep the default.

[theme]
background_color    = "rgba(10, 10, 10, 1)"
background_opacity  = 1.0
font_size           = 14
font_color          = "rgba(255, 255, 255, 1)"
border_radius       = 14
border_color        = "rgba(255,255,255,0.35)"
border_thickness    = 1
item_border_radius  = 12
active_bg_color     = "rgba(40, 40, 40, 1)"
active_border_color = "rgba(255,255,255,0.1)"

[triggers]
# Each value is a full regex. The first capture group is passed as the
# query to the searcher. No capture group = empty string passed.
# Invalid regex: that trigger is skipped with a warning at startup.
camera       = '^cam$'
bookmarks    = '^bk\s+(.*)$'
files        = '^f\s+(.*)$'
clipboard    = '^cp\s+(.*)$'
emojis       = '^em\s+(.*)$'
google       = '^g\s+(.*)$'
youtube      = '^yt\s+(.*)$'
nix          = '^nxp\s+(.*)$'
github       = '^gh\s+(.*)$'
shell        = '^!\s+(.*)$'
lorem        = '^lorem\s+(.*)$'
math         = '^=\s*(.*)$'
dictionary   = '^def\s+(.*)$'
system       = '^sys\s+(.*)$'
color_picker = '^color$'
apps         = '^app\s+(.*)$'
url          = '^(https?://\S+|(?:[a-zA-Z0-9](?:[a-zA-Z0-9\-]*[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}(?:[:/]\S*)?)$'
"#;
