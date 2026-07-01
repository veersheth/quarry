use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub triggers:      TriggerConfig,
    pub theme:         ThemeConfig,
    pub window:        WindowConfig,
    pub web_searches:  Vec<WebSearchConfig>,
    pub default_search: DefaultSearchConfig,
    pub screenshots:   ScreenshotsConfig,
    pub scripts:       ScriptsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            triggers:      TriggerConfig::default(),
            theme:         ThemeConfig::default(),
            window:        WindowConfig::default(),
            web_searches:  default_web_searches(),
            default_search: DefaultSearchConfig::default(),
            screenshots:   ScreenshotsConfig::default(),
            scripts:       ScriptsConfig::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Window
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub width:  u32,
    pub height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self { width: 1200, height: 700 }
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
        Self { path: "~/Pictures/Screenshots".into() }
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
        Self { path: "~/.config/quarry/scripts".into() }
    }
}

// ---------------------------------------------------------------------------
// Web searches
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchConfig {
    pub name:    String,
    pub trigger: String,
    pub url:     String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon:    Option<String>,
}

fn default_web_searches() -> Vec<WebSearchConfig> {
    vec![
        WebSearchConfig {
            name:    "Google".into(),
            trigger: r"^g\s+(.*)$".into(),
            url:     "https://www.google.com/search?q={}".into(),
            icon:    None,
        },
        WebSearchConfig {
            name:    "YouTube".into(),
            trigger: r"^yt\s+(.*)$".into(),
            url:     "https://www.youtube.com/results?search_query={}".into(),
            icon:    None,
        },
        WebSearchConfig {
            name:    "GitHub".into(),
            trigger: r"^gh\s+(.*)$".into(),
            url:     "https://github.com/search?q={}".into(),
            icon:    None,
        },
        WebSearchConfig {
            name:    "Nix Packages".into(),
            trigger: r"^nxp\s+(.*)$".into(),
            url:     "https://search.nixos.org/packages?query={}".into(),
            icon:    None,
        },
    ]
}

// ---------------------------------------------------------------------------
// Default search pinning
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DefaultSearchConfig {
    pub web_searches:    Vec<String>,
    pub max_web_results: usize,
}

impl Default for DefaultSearchConfig {
    fn default() -> Self {
        Self {
            web_searches:    vec!["Google".into(), "YouTube".into()],
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
    pub background_color:      String,
    pub background_opacity:    f32,
    pub font_size:             u32,
    pub font_color:            String,
    pub font_family:           String,
    pub monospace_font_family: String,
    pub border_radius:         u32,
    pub border_color:          String,
    pub border_thickness:      u32,
    pub item_border_radius:    u32,
    pub active_bg_color:       String,
    pub active_border_color:   String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background_color:      "rgba(10, 10, 10, 1)".into(),
            background_opacity:    1.0,
            font_size:             14,
            font_color:            "rgba(255, 255, 255, 1)".into(),
            font_family:           "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif".into(),
            monospace_font_family: "'JetBrains Mono', 'Fira Code', 'SF Mono', Consolas, monospace".into(),
            border_radius:         14,
            border_color:          "rgba(255,255,255,0.35)".into(),
            border_thickness:      1,
            item_border_radius:    12,
            active_bg_color:       "rgba(40, 40, 40, 1)".into(),
            active_border_color:   "rgba(255,255,255,0.1)".into(),
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
    pub shell:        String,
    pub lorem:        String,
    pub math:         String,
    pub dictionary:   String,
    pub system:       String,
    pub color_picker: String,
    pub apps:         String,
    pub url:          String,
    pub currency:     String,
    pub note:         String,
    pub ai:           String,
    pub time:         String,
    pub settings:     String,
    pub timer:        String,
    pub screenshots:  String,
    pub shortcuts:    String,
    pub scripts:      String,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            camera:       r"^cam$".into(),
            bookmarks:    r"^bk\s+(.*)$".into(),
            files:        r"^f\s+(.*)$".into(),
            clipboard:    r"^cp\s+(.*)$".into(),
            emojis:       r"^em\s+(.*)$".into(),
            shell:        r"^!\s+(.*)$".into(),
            lorem:        r"^lorem\s+(.*)$".into(),
            math:         r"^=\s*(.*)$".into(),
            dictionary:   r"^def\s+(.*)$".into(),
            system:       r"^sys\s+(.*)$".into(),
            color_picker: r"^color\s*(.*)$".into(),
            apps:         r"^app\s+(.*)$".into(),
            url:          r"^(https?://\S+|(?:[a-zA-Z0-9](?:[a-zA-Z0-9\-]*[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}(?:[:/]\S*)?)$".into(),
            currency:     r"^fx\s+(.*)$".into(),
            note:         r"^note$".into(),
            ai:           r"^ai\s+(.*)$".into(),
            time:         r"^time\s+(.*)$".into(),
            settings:     r"^set\s*(.*)$".into(),
            timer:        r"^timer\s*(.*)$".into(),
            screenshots:  r"^ss\s*(.*)$".into(),
            shortcuts:    r"^q\s*(.*)$".into(),
            scripts:      r"^sc\s*(.*)$".into(),
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

    pub fn save(config: &Config) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let toml = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
        fs::write(&path, toml).map_err(|e| e.to_string())
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
                .write(true).create(true).truncate(true)
                .mode(0o600)
                .open(&path)
                .map_err(|e| e.to_string())?;
            f.write_all(key.trim().as_bytes()).map_err(|e| e.to_string())?;
        }
        #[cfg(not(unix))]
        fs::write(&path, key.trim()).map_err(|e| e.to_string())?;

        Ok(())
    }
}

const DEFAULT_CONFIG_TOML: &str = r#"# Quarry configuration — ~/.config/quarry/config.toml
# Every key is optional. Remove or comment out any line to keep the default.

# Groq API key is stored separately in ~/.config/quarry/.groq_key
# Set it via Settings → General, or write the key to that file directly.

# [window]
# width  = 780   # launcher window width in pixels
# height = 520   # launcher window height in pixels

[theme]
background_color      = "rgba(10, 10, 10, 1)"
background_opacity    = 1.0
font_size             = 14
font_color            = "rgba(255, 255, 255, 1)"
font_family           = "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
monospace_font_family = "'JetBrains Mono', 'Fira Code', 'SF Mono', Consolas, monospace"
border_radius         = 14
border_color          = "rgba(255,255,255,0.35)"
border_thickness      = 1
item_border_radius    = 12
active_bg_color       = "rgba(40, 40, 40, 1)"
active_border_color   = "rgba(255,255,255,0.1)"

[triggers]
camera       = '^cam$'
bookmarks    = '^bk\s+(.*)$'
files        = '^f\s+(.*)$'
clipboard    = '^cp\s+(.*)$'
emojis       = '^em\s+(.*)$'
shell        = '^!\s+(.*)$'
lorem        = '^lorem\s+(.*)$'
math         = '^=\s*(.*)$'
dictionary   = '^def\s+(.*)$'
system       = '^sys\s+(.*)$'
color_picker = '^color$'
apps         = '^app\s+(.*)$'
url          = '^(https?://\S+|(?:[a-zA-Z0-9](?:[a-zA-Z0-9\-]*[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}(?:[:/]\S*)?)$'
ai           = '^ai\s+(.*)$'
time         = '^time in\s(.*)$'
shortcuts    = '^q\s*(.*)$'
scripts      = '^sc\s*(.*)$'

[screenshots]
# path = "~/Pictures/Screenshots"   # uncomment to override the default

[scripts]
# path = "~/.config/quarry/scripts"   # uncomment to override the default scripts folder

[default_search]
web_searches    = ["Google", "YouTube", "Nix Packages", "GitHub"]
max_web_results = 1

[[web_searches]]
name    = "Google"
trigger = '^g\s+(.*)$'
url     = "https://www.google.com/search?q={}"

[[web_searches]]
name    = "YouTube"
trigger = '^yt\s+(.*)$'
url     = "https://www.youtube.com/results?search_query={}"

[[web_searches]]
name    = "GitHub"
trigger = '^gh\s+(.*)$'
url     = "https://github.com/search?q={}"

[[web_searches]]
name    = "Nix Packages"
trigger = '^nxp\s+(.*)$'
url     = "https://search.nixos.org/packages?query={}"
"#;
