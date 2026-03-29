# Crosshair Overlay

A Rust desktop utility for Windows with three separate tools:

- a click-through crosshair overlay
- window presets that move and resize the current foreground window with global hotkeys
- macro presets with recording and optional AI generation

## Features

- Topmost transparent overlay that does not block mouse clicks
- Crosshair controls for length, thickness, gap, outline, center dot, opacity, color, and center offsets
- Custom crosshair assets from the `custom-crosshairs` folder
- SVG, PNG, JPG, JPEG, BMP, WEBP, and ICO asset support
- Save crosshair profiles and import/export them as shareable codes
- System tray icon with toggle, open settings, and exit actions
- Default crosshair toggle hotkey: `Ctrl + Alt + X`
- Separate `Window Presets` panel with multiple presets
- Per-preset width, height, X, Y, enable toggle, and a custom global hotkey
- Presets are saved in the local config so they persist after restart
- Separate `Macros` panel with multiple macro presets
- Per-macro enable toggle, trigger hotkey, record hotkey, editable key down/up steps, and per-step delays
- Global macro recording that captures key down/up events and timing
- Gemini API integration for generating macro steps from a prompt
- Automatic admin relaunch, high process priority, and single-instance protection to prevent duplicate tray icons

## Run

```powershell
cargo run --release
```

Release binary:

`target\release\crosshair.exe`

## App Data

The app stores its data under LocalAppData and exposes buttons in the UI to open the folders.

- `profiles`: saved crosshair profiles
- `custom-crosshairs`: custom assets you can drop in
- `state.json`: current app state, including window presets
- `state.json`: current app state, including window presets, macro presets, and AI settings

## Notes

- The overlay works best with desktop apps and borderless fullscreen.
- True exclusive fullscreen can still be limited by Windows, GPU drivers, game engines, or anti-cheat systems.
- Window preset hotkeys apply to the current foreground window at the moment you press the preset hotkey.
- Macro preset hotkeys and record hotkeys are ignored while this app window is focused so you can edit presets safely.
- The AI helper uses the Gemini REST API. By default the UI suggests `gemini-2.5-flash-lite` for a fast, lightweight response path.
