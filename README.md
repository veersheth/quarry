# Quarry

<img width="2256" height="1504" alt="image" src="https://github.com/user-attachments/assets/2b1e341d-2044-404b-87fd-cc6ea323f7ae" />


- Globalized app/shortcut launcher
- Currently developed focused on a Hyprland / NixOS setup, but it _should_ work on most distributions (not tested)
- Currently aiming to fix _my_ gripes with already existing app launchers
- Lots of inspiration taken from [Raycast](https://www.raycast.com/) 
- This is my first big project, so development might be slow. If you're smart I'd appreciate any feedback 😁

## Setup

```
nix-shell
pnpm install
pnpm run tauri dev
```

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

Edit `~/.config/quarry/config.toml`. Quarry creates a short file on first run. Add only the settings you want to change, then use **Reload Quarry**. See the [complete configuration reference](CONFIGURATION.md) for every supported key and default.

```toml
[theme]
font_size = 16

[triggers]
files = '^find\s+(.*)$'
```

Clipboard tracking, file indexing, and sidebar widths are stored separately in local app data; none is controlled by TOML. The Groq API key is stored separately at `~/.local/share/quarry/.groq_key`.

