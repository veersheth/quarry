# quarry

- globalized app/shortcut launcher, written in tauri/svelte
- currently developed focused on a hyprland/nixos setup
    - therefore depends on wayland protocols like `wl-copy` and `cliphist`
- currently aiming to fix _my_ gripes with already existing app launchers
- lots of inspiration taken from [raycast](https://www.raycast.com/) 
- this is my first big project, so development might be slow. if you're smart i'd appreciate any feedback 😁

```
$ pnpm install
$ pnpm run tauri dev
```

# current capabilities

- app search 
![](for-readme/ss-apps.png)

- emoji search 
![](for-readme/ss-emojis.png)

- math
![](for-readme/ss-math.png)

- web search
![](for-readme/ss-web.png)

# checklist

FRONTEND
- [x] build out basic list layout
- [x] dumbdown frontend (move ALL functionality to backend)
- [x] emacs based keybindings? (the ones used in vim insert mode)
- [ ] basic markdown renderer layout
- [x] grid layout


BACKEND
- [x] basic skeleton 
- [x] module-based structure
- [x] run in system tray
- [x] cli to toggle visbility of main window
- [ ] prefix settings separate from lib.rs 
- [ ] figure out extendability somehow


"PLUGINS"
- [x] web searches BUILTIN
- [x] math BUILTIN
- [ ] currency conversion
- [ ] color preview/picker
- [x] emoji picker BUILTIN
- [ ] world/clock/timer management  
- [x] lorem generator
- [ ] notes integration  BUILTIN
- [ ] translation 
- [ ] clipboard history BUILTIN
- [ ] basic chatgpt/gemini integration(?) 
- [ ] spotify controls (or general media playback contorls)
