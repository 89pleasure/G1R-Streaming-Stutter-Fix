# Desktop App Architecture

The desktop app is split into a Rust optimizer core and a Tauri shell.

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
- preview target changes
- back up existing `Engine.ini` and `Scalability.ini`
- install selected preset files
- restore app-created backups

The core currently avoids third-party dependencies so it can be tested quickly
and reused by a future CLI.

## Tauri Responsibilities

`app/src-tauri` resolves the bundled `Presets/` resource and exposes core
functions as Tauri commands:

- `get_app_state`
- `preview_install`
- `install_preset`
- `list_backups`
- `restore_backup`

The app sets `withGlobalTauri` to `true` so the static UI can call
`window.__TAURI__.core.invoke(...)` without a frontend bundler.

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
