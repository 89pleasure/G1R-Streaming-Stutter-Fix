# Desktop App Architecture

G1R Optimizer is split into a Rust optimizer core and a Tauri desktop shell. The
desktop app is now the primary distribution path; the raw INI presets remain as
bundled app resources and as a manual fallback.

## Layout

```text
optimizer-core/       Rust library with install logic and tests
app/src-tauri/        Tauri v2 wrapper and desktop commands
app/ui/               Static HTML/CSS/JavaScript UI
Presets/              Bundled INI presets used by the app
```

## Core Responsibilities

`optimizer-core` owns file-system behavior:

- scan `Presets/` folders
- detect common Windows and Linux/Proton Gothic 1 Remake config paths
- detect hardware and recommend a matching preset
- detect Steam or executable launch targets for Gothic 1 Remake
- preview target changes
- generate selected `Engine.ini`, `Scalability.ini` and `Game.ini` content
- back up existing managed INI files
- classify existing files as clean, modified, untracked or unknown
- merge app-managed settings into existing custom INI files when requested
- install selected optimizer settings
- reset managed files back to vanilla by removing them after backup
- restore app-created backups

The core currently avoids third-party dependencies so it can be tested quickly
and reused by a future CLI.

## Tauri Responsibilities

`app/src-tauri` resolves the bundled `Presets/` resource and exposes core
functions as Tauri commands:

- `get_app_state`
- `preview_install`
- `ini_file_contents`
- `install_preset`
- `list_backups`
- `restore_backup`
- `reset_to_vanilla`
- `launch_game`

The app sets `withGlobalTauri` to `true` so the static UI can call
`window.__TAURI__.core.invoke(...)` without a frontend bundler.

## UI Responsibilities

`app/ui` owns the static desktop interface:

- streaming preset selection, including custom pool size
- optional performance tweaks
- optional game tweaks
- config folder selection and detected path display
- game launch target selection and Play action
- preview, overwrite warning, merge/replace choice and Optimize action
- backups, restore, reset and diagnostics
- localized text through `app/ui/locales`

## Packaging

The Tauri config bundles `../../Presets` into the app resource directory as
`Presets`.

Platform-specific targets are:

- Linux: AppImage and deb
- Windows: NSIS installer

Build from `app/`:

```bash
npm install
npm run build
```
