```toml
# Quarry config: ~/.config/quarry/config.toml
# Uncomment only what you change; omitted values use these defaults.
# Run "Reload Quarry" after editing. Window size changes need a restart.
# Clipboard tracking, file indexing, and sidebar widths are stored locally.
# Groq API key: ~/.local/share/quarry/.groq_key

# Wizard completion; normally managed by the wizard.
# setup_completed = false
# Timer alert: "synth" or "beep".
# timer_sound = "synth"

[window]
# Launcher size in pixels.
# width = 1200
# height = 700

[screenshots]
# Folder searched for screenshots; use an absolute path or ~.
# path = "~/Pictures/Screenshots"

[scripts]
# Folder containing user scripts; use an absolute path or ~.
# path = "~/.config/quarry/scripts"

[default_search]
# Web engines offered when no local result matches.
# web_searches = ["Google", "YouTube"]
# Results per web engine.
# max_web_results = 1

[triggers]
# Regexes match the full query. Capture (.*) to pass search text.
# Single-quoted TOML strings keep regex backslashes literal.
# camera = '^cam$'
# bookmarks = '^bk\s+(.*)$'
# files = '^f\s+(.*)$'
# fonts = '^font(?:\s+(.*))?$'
# clipboard = '^cp\s+(.*)$'
# emojis = '^em\s+(.*)$'
# shell = '^!\s+(.*)$'
# lorem = '^lorem\s+(.*)$'
# math = '^=\s*(.*)$'
# dictionary = '^def\s+(.*)$'
# system = '^sys\s+(.*)$'
# color_picker = '^color\s*(.*)$'
# apps = '^app\s+(.*)$'
# currency = '^fx\s+(.*)$'
# ai = '^ (.*)$'
# time = '^time\s+(.*)$'
# timer = '^timer\s*(.*)$'
# screenshots = '^ss\s*(.*)$'
# shortcuts = '^q\s*(.*)$'
# scripts = '^sc\s*(.*)$'
# qr_code = '^qr\s+(.*)$'

[theme]
# Window background and opacity.
# background_color = "rgba(10, 10, 10, 1)"
# background_opacity = 1.0
# Main text size and color.
# font_size = 14
# font_color = "rgba(255, 255, 255, 1)"
# Main and monospace font stacks.
# font_family = "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
# monospace_font_family = "'JetBrains Mono', 'Fira Code', 'SF Mono', Consolas, monospace"
# Window corners and border.
# border_radius = 14
# border_color = "rgba(255,255,255,0.35)"
# border_thickness = 1
# Result item corners.
# item_border_radius = 12
# Selected item fill and border.
# active_bg_color = "rgba(40, 40, 40, 1)"
# active_border_color = "rgba(255,255,255,0.1)"
# Base, muted, hover, and modal surfaces.
# surface = "rgba(5, 5, 5, 0.0)"
# surface_subtle = "rgba(255, 255, 255, 0.08)"
# surface_hover = "rgba(255, 255, 255, 0.2)"
# overlay = "rgba(30, 30, 30, 0.8)"
# Border shades.
# border_subtle = "rgba(60, 60, 60, 0.3)"
# border_medium = "rgba(255, 255, 255, 0.15)"
# border_strong = "rgba(255, 255, 255, 0.26)"
# border_dark = "rgb(51, 51, 51)"
# Divider shades.
# divider = "rgba(80, 80, 80, 0.7)"
# divider_dark = "rgb(46, 46, 46)"
# Code, thumbnail, and glow colors.
# code_bg = "rgb(24, 24, 24)"
# thumb_bg = "rgb(17, 17, 17)"
# glow = "rgba(50, 50, 50, 1)"

# Web searches replace the entire built-in list when any entries are set.
# Each needs a name, trigger regex, and URL with {} for the encoded query.
# icon is optional. Built-ins: Google, YouTube, GitHub, Nix Packages.
# [[web_searches]]
# name = "Google"
# trigger = '^g\s+(.*)$'
# url = "https://www.google.com/search?q={}"
# icon = "https://example.com/icon.png"
#
# [[web_searches]]
# name = "YouTube"
# trigger = '^yt\s+(.*)$'
# url = "https://www.youtube.com/results?search_query={}"
#
# [[web_searches]]
# name = "GitHub"
# trigger = '^gh\s+(.*)$'
# url = "https://github.com/search?q={}"
#
# [[web_searches]]
# name = "Nix Packages"
# trigger = '^nxp\s+(.*)$'
# url = "https://search.nixos.org/packages?query={}"
```
