# Play Button Game Launch Design

## Goal

Add a Play button to G1R Optimizer that launches Gothic 1 Remake without applying optimizer
changes automatically.

## Background

The desktop app currently detects game config folders and installs selected INI settings, but it
does not start the game. Users still launch Gothic 1 Remake separately after pressing Optimize.
The new launch feature should keep that separation: Play starts the game, while Optimize remains
the only action that writes config files.

## Approach

The app will prefer Steam launch when Gothic 1 Remake is installed through Steam. If the Steam
installation cannot be detected, the app will fall back to an executable path. The executable path
is detected automatically from known install locations when possible, and the user can override it
manually in Settings.

This follows the existing app pattern for config folders: automatic candidates first, visible manual
control when detection is wrong or incomplete.

## Backend Design

Launch detection belongs in `optimizer-core`, alongside existing path and platform detection.

The core will expose launch candidates with:

- `kind`: `steam` or `executable`.
- `label`: user-facing description such as `Steam: Gothic 1 Remake` or `Executable`.
- `path`: the source path for diagnostics. Steam candidates point at the detected app manifest or
  library source; executable candidates point at the executable.
- `exists`: whether the candidate is valid now.
- `source`: where the candidate came from, such as `Steam library`, `common path`, or `manual`.

Detection order:

1. Scan known Steam `steamapps` roots, including extra libraries from `libraryfolders.vdf`.
2. Treat `steamapps/appmanifest_1297900.acf` as the signal that Gothic 1 Remake is installed
   through Steam.
3. Add a Steam candidate first when the manifest is found.
4. Add executable candidates found from known install locations, including paths derived from the
   Steam library when a common install folder exists.
5. Validate a manually configured executable path and include it as a manual candidate.

Launch execution remains in the Tauri layer because it starts an external process or URI:

- Steam candidate: open `steam://rungameid/1297900`.
- Executable candidate: start the executable directly.
- Missing candidate: return a clear error message to the UI.

## UI Design

Settings will gain a `Game Launch` section.

It will contain:

- a select box for detected launch targets,
- an executable path field,
- a Browse button for choosing the executable manually,
- a status pill showing `Found`, `Manual`, or `Missing`.

Default selection:

- Steam is selected automatically when detected.
- If Steam is not detected, the best executable candidate is selected.
- If neither exists, Play is disabled until the user selects a valid executable.

The main action row in the Preview panel will add a Play button next to Optimize and Copy. Play
only launches the game and never writes INI files.

## Error Handling

Launch failures are shown in the existing action-result area and added to the activity log.

Expected failure cases:

- Steam candidate selected but the platform cannot open the Steam URI.
- Executable candidate selected but the path is missing or not a file.
- No valid launch target is selected.

The UI should keep the app responsive and re-enable controls after launch attempts, regardless of
success or failure.

## Testing

Core tests will cover:

- parsing Steam library paths,
- finding `appmanifest_1297900.acf`,
- preferring Steam candidates over executable candidates,
- including valid manual executable candidates,
- rejecting missing manual executable paths.

UI tests will cover:

- new i18n keys required by the Game Launch settings,
- persisted launch preferences for selected target and manual executable path,
- Play controls being present in the static HTML.

Manual verification will include:

- `cargo test -p optimizer-core`,
- app UI tests under `app/ui`,
- `cargo check` or Tauri build checks for `app/src-tauri`.

## Out Of Scope

- Automatically pressing Optimize before launching.
- Detecting whether the game process is already running.
- Managing Steam launch options.
- Supporting non-Steam store launch protocols beyond direct executable fallback.
