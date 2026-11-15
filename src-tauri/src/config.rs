use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub triggers: TriggerConfig,
    pub theme: ThemeConfig,
    pub window: WindowConfig,
    pub web_searches: Vec<WebSearchConfig>,
    pub default_search: DefaultSearchConfig,
    pub screenshots: ScreenshotsConfig,
    pub scripts: ScriptsConfig,
    /// Timer alert sound. Options: "synth" (default), "beep"
    #[serde(default = "default_timer_sound")]
    pub timer_sound: String,
    /// Whether the first-run setup wizard has been completed.
    #[serde(default)]
    pub setup_completed: bool,
}

fn default_timer_sound() -> String {
    "synth".into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            triggers: TriggerConfig::default(),
            theme: ThemeConfig::default(),
            window: WindowConfig::default(),
            web_searches: default_web_searches(),
            default_search: DefaultSearchConfig::default(),
            screenshots: ScreenshotsConfig::default(),
            scripts: ScriptsConfig::default(),
            timer_sound: default_timer_sound(),
            setup_completed: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Window
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 700,
        }
    }
}

// ---------------------------------------------------------------------------
// Screenshots
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScreenshotsConfig {
    /// Directory to scan for screenshots. Defaults to ~/Pictures/Screenshots.
    pub path: String,
}

impl Default for ScreenshotsConfig {
    fn default() -> Self {
        Self {
            path: "~/Pictures/Screenshots".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Scripts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScriptsConfig {
    /// Directory that holds user executable scripts. Defaults to ~/.config/quarry/scripts.
    pub path: String,
}

impl Default for ScriptsConfig {
    fn default() -> Self {
        Self {
            path: "~/.config/quarry/scripts".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Web searches
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchConfig {
    pub name: String,
    pub trigger: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

fn default_web_searches() -> Vec<WebSearchConfig> {
    vec![
        WebSearchConfig {
            name: "Google".into(),
            trigger: r"^g\s+(.*)$".into(),
            url: "https://www.google.com/search?q={}".into(),
            icon: None,
        },
        WebSearchConfig {
            name: "YouTube".into(),
            trigger: r"^yt\s+(.*)$".into(),
            url: "https://www.youtube.com/results?search_query={}".into(),
            icon: None,
        },
        WebSearchConfig {
            name: "GitHub".into(),
            trigger: r"^gh\s+(.*)$".into(),
            url: "https://github.com/search?q={}".into(),
            icon: None,
        },
        WebSearchConfig {
            name: "Nix Packages".into(),
            trigger: r"^nxp\s+(.*)$".into(),
            url: "https://search.nixos.org/packages?query={}".into(),
            icon: None,
        },
    ]
}

// ---------------------------------------------------------------------------
// Default search pinning
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DefaultSearchConfig {
    pub web_searches: Vec<String>,
    pub max_web_results: usize,
}

impl Default for DefaultSearchConfig {
    fn default() -> Self {
        Self {
            web_searches: vec!["Google".into(), "YouTube".into()],
            max_web_results: 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub background_color: String,
    pub background_opacity: f32,
    pub font_size: u32,
    pub font_color: String,
    pub font_family: String,
    pub monospace_font_family: String,
    pub border_radius: u32,
    pub border_color: String,
    pub border_thickness: u32,
    pub item_border_radius: u32,
    pub active_bg_color: String,
    pub active_border_color: String,

    // Surface / overlay
    pub surface: String,
    pub surface_subtle: String,
    pub surface_hover: String,
    pub overlay: String,

    // Borders
    pub border_subtle: String,
    pub border_medium: String,
    pub border_strong: String,
    pub border_dark: String,

    // Misc
    pub divider: String,
    pub divider_dark: String,
    pub code_bg: String,
    pub thumb_bg: String,
    pub glow: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background_color: "rgba(10, 10, 10, 1)".into(),
            background_opacity: 1.0,
            font_size: 14,
            font_color: "rgba(255, 255, 255, 1)".into(),
            font_family:
                "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
                    .into(),
            monospace_font_family: "'JetBrains Mono', 'Fira Code', 'SF Mono', Consolas, monospace"
                .into(),
            border_radius: 14,
            border_color: "rgba(255,255,255,0.35)".into(),
            border_thickness: 1,
            item_border_radius: 12,
            active_bg_color: "rgba(40, 40, 40, 1)".into(),
            active_border_color: "rgba(255,255,255,0.1)".into(),

            surface: "rgba(5, 5, 5, 0.0)".into(),
            surface_subtle: "rgba(255, 255, 255, 0.08)".into(),
            surface_hover: "rgba(255, 255, 255, 0.2)".into(),
            overlay: "rgba(30, 30, 30, 0.8)".into(),

            border_subtle: "rgba(60, 60, 60, 0.3)".into(),
            border_medium: "rgba(255, 255, 255, 0.15)".into(),
            border_strong: "rgba(255, 255, 255, 0.26)".into(),
            border_dark: "rgb(51, 51, 51)".into(),

            divider: "rgba(80, 80, 80, 0.7)".into(),
            divider_dark: "rgb(46, 46, 46)".into(),
            code_bg: "rgb(24, 24, 24)".into(),
            thumb_bg: "rgb(17, 17, 17)".into(),
            glow: "rgba(50, 50, 50, 1)".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Triggers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TriggerConfig {
    pub camera: String,
    pub bookmarks: String,
    pub files: String,
    pub fonts: String,
    pub clipboard: String,
    pub emojis: String,
    pub shell: String,
    pub lorem: String,
    pub math: String,
    pub dictionary: String,
    pub system: String,
    pub color_picker: String,
    pub apps: String,
    pub currency: String,
    pub ai: String,
    pub time: String,
    pub timer: String,
    pub screenshots: String,
    pub shortcuts: String,
    pub scripts: String,
    pub qr_code: String,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            camera: r"^cam$".into(),
            bookmarks: r"^bk\s+(.*)$".into(),
            files: r"^f\s+(.*)$".into(),
            fonts: r"^font(?:\s+(.*))?$".into(),
            clipboard: r"^cp\s+(.*)$".into(),
            emojis: r"^em\s+(.*)$".into(),
            shell: r"^!\s+(.*)$".into(),
            lorem: r"^lorem\s+(.*)$".into(),
            math: r"^=\s*(.*)$".into(),
            dictionary: r"^def\s+(.*)$".into(),
            system: r"^sys\s+(.*)$".into(),
            color_picker: r"^color\s*(.*)$".into(),
            apps: r"^app\s+(.*)$".into(),
            currency: r"^fx\s+(.*)$".into(),
            ai: r"^ (.*)$".into(),
            time: r"^time\s+(.*)$".into(),
            timer: r"^timer\s*(.*)$".into(),
            screenshots: r"^ss\s*(.*)$".into(),
            shortcuts: r"^q\s*(.*)$".into(),
            scripts: r"^sc\s*(.*)$".into(),
            qr_code: r"^qr\s+(.*)$".into(),
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
                eprintln!("quarry: config parse error - {} - using defaults", e);
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

    pub fn save(config: &Config) -> Result<(), String> {
        let path = Self::config_path();
        // A malformed hand-edited file must never be replaced by an in-app save.
        if path.exists() {
            let contents = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            toml::from_str::<Config>(&contents)
                .map_err(|e| format!("Fix {} before saving: {e}", path.display()))?;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let contents = config.to_minimal_toml()?;
        let temporary = path.with_extension(format!("toml.{}.tmp", std::process::id()));
        fs::write(&temporary, contents).map_err(|e| e.to_string())?;
        if let Err(error) = fs::rename(&temporary, &path) {
            let _ = fs::remove_file(&temporary);
            return Err(error.to_string());
        }
        Ok(())
    }

    fn to_minimal_toml(&self) -> Result<String, String> {
        let mut current = toml::Value::try_from(self).map_err(|e| e.to_string())?;
        let defaults = toml::Value::try_from(Self::default()).map_err(|e| e.to_string())?;
        remove_defaults(&mut current, &defaults);
        // Keep wizard completion explicit in the generated file.
        let table = current.as_table_mut().ok_or("Invalid config structure")?;
        table.insert(
            "setup_completed".into(),
            toml::Value::Boolean(self.setup_completed),
        );
        toml::to_string_pretty(&current).map_err(|e| e.to_string())
    }

    fn key_file_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from(".local/share"))
            .join("quarry/.groq_key")
    }

    pub fn load_groq_api_key() -> String {
        fs::read_to_string(Self::key_file_path())
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    pub fn save_groq_api_key(key: &str) -> Result<(), String> {
        let path = Self::key_file_path();
        let dir = path.parent().unwrap();
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;

        // Write key with owner-only permissions.
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut f = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)
                .map_err(|e| e.to_string())?;
            f.write_all(key.trim().as_bytes())
                .map_err(|e| e.to_string())?;
        }
        #[cfg(not(unix))]
        fs::write(&path, key.trim()).map_err(|e| e.to_string())?;

        Ok(())
    }
}

fn remove_defaults(current: &mut toml::Value, defaults: &toml::Value) {
    if let (Some(current), Some(defaults)) = (current.as_table_mut(), defaults.as_table()) {
        current.retain(|key, value| {
            if let Some(default) = defaults.get(key) {
                remove_defaults(value, default);
                value != default && value.as_table().is_none_or(|table| !table.is_empty())
            } else {
                true
            }
        });
    }
}

const DEFAULT_CONFIG_TOML: &str =
    "# Quarry settings. Add only the values you want to change.\nsetup_completed = false\n";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_run_config_matches_defaults() {
        let loaded: Config = toml::from_str(DEFAULT_CONFIG_TOML).unwrap();
        let defaults = Config::default();
        assert_eq!(loaded.setup_completed, defaults.setup_completed);
        assert_eq!(loaded.triggers.time, defaults.triggers.time);
        assert_eq!(loaded.web_searches.len(), defaults.web_searches.len());
    }

    #[test]
    fn minimal_config_round_trips_custom_settings() {
        let mut original = Config::default();
        original.setup_completed = true;
        original.theme.font_size = 18;
        original.triggers.files = r"^find\s+(.*)$".into();
        original.web_searches.push(WebSearchConfig {
            name: "Custom".into(),
            trigger: r"^custom\s+(.*)$".into(),
            url: "https://example.com/search?q={}".into(),
            icon: None,
        });

        let serialized = original.to_minimal_toml().unwrap();
        assert!(!serialized.contains("background_color"));
        assert!(!serialized.contains("[window]"));
        let loaded: Config = toml::from_str(&serialized).unwrap();
        assert!(loaded.setup_completed);
        assert_eq!(loaded.theme.font_size, 18);
        assert_eq!(loaded.triggers.files, original.triggers.files);
        assert_eq!(loaded.web_searches.len(), original.web_searches.len());
    }

    #[test]
    fn tracking_cannot_be_enabled_from_toml() {
        let config: Config =
            toml::from_str("clipboard_tracking_enabled = true\nfile_indexing_enabled = true")
                .unwrap();
        let saved = config.to_minimal_toml().unwrap();
        assert!(!saved.contains("clipboard_tracking_enabled"));
        assert!(!saved.contains("file_indexing_enabled"));
    }
}
