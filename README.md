# Quarry

<img width="1148" height="812" alt="image" src="https://github.com/user-attachments/assets/aff58812-e2a2-467f-8f2a-8207de41d5ad" />

- Globalized app/shortcut launcher
- Currently developed focused on a Hyprland / NixOS setup, but it _should_ work on most distributions (not tested)
- Currently aiming to fix _my_ gripes with already existing app launchers
- Lots of inspiration taken from [Raycast](https://www.raycast.com/) 
- This is my first big project, so development might be slow. If you're smart I'd appreciate any feedback 😁

> Rust/Svelte

## Setup

Clone the repo and `cd` to it

```
$ nix-shell 
$ pnpm install
$ pnpm run tauri dev
```

- This should start a process and put an icon in your system tray
- To toggle visibility
    - Either use the tray icon menu
    - Or run the `quarry-toggle` binary in `src-tauri/target/debug/quarry-toggle`, I have this mapped to `Alt-Space`

# Capabilities
- Search apps, emojis, files 
- Colorpicker
- Clipboard manager (text + images)
- Calculator
- Camera preview
- Custom bookmarks implementation
- Web search
- Dictionary

# Configuration

- Edit the `.config/quarry/config.toml` file 

```toml
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

[default_search]
web_searches    = ["Google", "YouTube"]
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
```

