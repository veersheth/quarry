use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub triggers:      TriggerConfig,
    pub theme:         ThemeConfig,
    pub web_searches:  Vec<WebSearchConfig>,
    pub default_search: DefaultSearchConfig,
    /// Groq API key for AI chat. Leave empty to be prompted on first use.
    #[serde(default)]
    pub groq_api_key:  String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            triggers:      TriggerConfig::default(),
            theme:         ThemeConfig::default(),
            web_searches:  default_web_searches(),
            default_search: DefaultSearchConfig::default(),
            groq_api_key:  String::new(),
        }
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
    #[serde(default)]
    pub icon:    Option<String>,
}

fn default_web_searches() -> Vec<WebSearchConfig> {
    vec![
        WebSearchConfig {
            name:    "Google".into(),
            trigger: r"^g\s+(.*)$".into(),
            url:     "https://www.google.com/search?q={}".into(),
            icon:    Some("https://upload.wikimedia.org/wikipedia/commons/thumb/c/c1/Google_%22G%22_logo.svg/1200px-Google_%22G%22_logo.svg.png".into()),
        },
        WebSearchConfig {
            name:    "YouTube".into(),
            trigger: r"^yt\s+(.*)$".into(),
            url:     "https://www.youtube.com/results?search_query={}".into(),
            icon:    Some("https://upload.wikimedia.org/wikipedia/commons/thumb/f/fd/YouTube_full-color_icon_%282024%29.svg/1920px-YouTube_full-color_icon_%282024%29.svg.png".into()),
        },
        WebSearchConfig {
            name:    "GitHub".into(),
            trigger: r"^gh\s+(.*)$".into(),
            url:     "https://github.com/search?q={}".into(),
            icon:    Some("https://upload.wikimedia.org/wikipedia/commons/thumb/a/ae/Github-desktop-logo-symbol.svg/2048px-Github-desktop-logo-symbol.svg.png".into()),
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
    pub background_color:    String,
    pub background_opacity:  f32,
    pub font_size:           u32,
    pub font_color:          String,
    pub border_radius:       u32,
    pub border_color:        String,
    pub border_thickness:    u32,
    pub item_border_radius:  u32,
    pub active_bg_color:     String,
    pub active_border_color: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background_color:    "rgba(10, 10, 10, 1)".into(),
            background_opacity:  1.0,
            font_size:           14,
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
            color_picker: r"^color$".into(),
            apps:         r"^app\s+(.*)$".into(),
            url:          r"^(https?://\S+|(?:[a-zA-Z0-9](?:[a-zA-Z0-9\-]*[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}(?:[:/]\S*)?)$".into(),
            currency:     r"^fx\s+(.*)$".into(),
            note:         r"^note$".into(),
            ai:           r"^ai\s+(.*)$".into(),
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

    /// Write just the groq_api_key into the config file, preserving all other content.
    pub fn save_groq_api_key(key: &str) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let existing = fs::read_to_string(&path).unwrap_or_default();

        // If key already exists in file, replace it; otherwise append it.
        let new_content = if existing.contains("groq_api_key") {
            let mut lines: Vec<String> = existing.lines().map(String::from).collect();
            for line in &mut lines {
                if line.trim_start().starts_with("groq_api_key") {
                    *line = format!("groq_api_key = \"{}\"", key);
                }
            }
            lines.join("\n")
        } else {
            // Append at top level (not inside any section)
            // Insert before the first [section] header so it's a top-level key
            let insert_line = format!("groq_api_key = \"{}\"\n", key);
            if let Some(pos) = existing.find("\n[") {
                let (before, after) = existing.split_at(pos + 1);
                format!("{}{}{}", before, insert_line, after)
            } else {
                format!("{}\n{}", existing.trim_end(), insert_line)
            }
        };

        fs::write(&path, new_content).map_err(|e| e.to_string())
    }
}

const DEFAULT_CONFIG_TOML: &str = r#"# Quarry configuration — ~/.config/quarry/config.toml
# Every key is optional. Remove or comment out any line to keep the default.

# groq_api_key = ""   # Add your Groq API key here to enable AI chat (ai <query>)

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

[default_search]
web_searches    = ["Google", "YouTube", "Nix Packages", "GitHub"]
max_web_results = 1

[[web_searches]]
name    = "Google"
trigger = '^g\s+(.*)$'
url     = "https://www.google.com/search?q={}"
icon    = "https://upload.wikimedia.org/wikipedia/commons/thumb/c/c1/Google_%22G%22_logo.svg/1200px-Google_%22G%22_logo.svg.png"

[[web_searches]]
name    = "YouTube"
trigger = '^yt\s+(.*)$'
url     = "https://www.youtube.com/results?search_query={}"
icon    = "https://upload.wikimedia.org/wikipedia/commons/thumb/f/fd/YouTube_full-color_icon_%282024%29.svg/1920px-YouTube_full-color_icon_%282024%29.svg.png"

[[web_searches]]
name    = "GitHub"
trigger = '^gh\s+(.*)$'
url     = "https://github.com/search?q={}"
icon    = "https://upload.wikimedia.org/wikipedia/commons/thumb/a/ae/Github-desktop-logo-symbol.svg/2048px-Github-desktop-logo-symbol.svg.png"

[[web_searches]]
name    = "Nix Packages"
trigger = '^nxp\s+(.*)$'
url     = "https://search.nixos.org/packages?query={}"
"#;
