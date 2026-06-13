# G1R Optimizer

Desktop tool for Gothic 1 Remake that applies two main tuning paths without
requiring manual INI file copying: the G1R Streaming Stutter Fix for texture
streaming pressure, and Balanced Overdose Performance for substantially higher
FPS on the Overdose profile.

The project started as a set of VRAM-based `Engine.ini` and `Scalability.ini`
presets. The primary workflow is now the Tauri desktop app in `app/`. The
`Presets/` directory is still bundled with the app and remains available as a
manual fallback.

![G1R - Streaming Stutter Fix cover](ModImages/G1R_StreamingStutterFix_cover_1280x720.jpg)

## Main Features

G1R Optimizer is built around two equal feature groups:

- **Streaming Stutter Fix:** raises the texture streaming pool based on GPU VRAM
  to reduce pop-in, streaming pressure and frametime drops.
- **Balanced Overdose Performance:** optional Overdose-only performance caps
  that were observed to improve tested scenes by roughly 20-30 FPS with small
  visual trade-offs.

Additional app features include Game.ini tweaks, backups, restore, Reset to
Vanilla, config-folder detection, overwrite warnings and manual copy support.

## Streaming Stutter Fix

Gothic 1 Remake ships with very small texture streaming pools in its default
scalability profiles:

```text
TextureQuality@0     200 MB
TextureQuality@1     300 MB
TextureQuality@2     400 MB
TextureQuality@3     500 MB
TextureQuality@Cine  1000 MB (Overdose in the game menu)
```

Many users play with Texture Quality Overdose. Internally, that maps to
`sg.TextureQuality=4` and `TextureQuality@Cine`.
That means the default texture streaming pool can be only `1000 MB`, even on
modern GPUs with much more VRAM.

The base streaming fix raises that pool in a controlled, VRAM-based way and can
add conservative shader/loading tweaks. It is intended to reduce texture pop-in,
streaming pressure and frametime drops without lowering Lumen, Nanite, shadow
quality, view distance, resolution or texture quality.

## Balanced Overdose Performance

The desktop app also includes optional `Balanced Performance Tweaks` for players
who want more FPS while staying close to the Overdose look.

This mode applies conservative `Scalability.ini` caps only to the Overdose
profile. In tested comparison scenes, it improved performance by about 20-30 FPS
while keeping Lumen, Nanite and virtual shadows enabled. The trade-offs are
focused on shadow, reflection, volumetric and post-processing cost.

Balanced performance is off by default, shown separately in the app, and
previewed before writing files. The app labels it as `Overdose only` because the
current tweak set targets the Overdose profile, not the lower scalability
profiles.

Depending on the selected options, the app writes managed settings to
`Engine.ini`, `Scalability.ini` and `Game.ini`.

## Desktop Tool

G1R Optimizer wraps the INI presets with a safer workflow:

- detects common Windows and Linux/Proton config folders
- detects hardware and recommends a matching VRAM preset when possible
- previews the selected preset before writing files
- backs up existing `Engine.ini`, `Scalability.ini` and `Game.ini`
- warns when existing INI files contain custom or third-party settings
- can merge app-managed settings into existing INI files
- installs the selected streaming preset
- can opt into balanced `Scalability.ini` performance tweaks
- can opt into `Engine.ini` stutter experiments such as PSO cache and GC tuning
- can opt into `Game.ini` tweaks such as skipping intro videos
- can reset managed config files back to vanilla
- restores backups created by the app
- can copy generated INI settings for manual use
- can launch Gothic 1 Remake through Steam when detected, or through a selected executable fallback

The optimizer core lives in `optimizer-core/` and is intentionally separate from
the Tauri UI so it can also become a CLI later.

## Installation

1. Close Gothic 1 Remake completely.
2. Start the `G1R Optimizer` desktop app.
3. Use the detected config folder, or browse to the game config folder manually:

   ```text
   %LOCALAPPDATA%\G1R\Saved\Config\Windows\
   ```

   Linux / Proton example:

   ```text
   <SteamLibrary>/steamapps/compatdata/1297900/pfx/drive_c/users/steamuser/AppData/Local/G1R/Saved/Config/Windows/
   ```

4. Select the recommended preset, choose a different VRAM preset, or enter a
   custom texture pool size.
5. Enable any optional performance or game tweaks you want.
6. Review the preview. If existing INI files contain custom settings, choose
   `Merge` to keep those settings or `Use App Settings Only` to replace them.
7. Press `Optimize`, then launch and test the game.

The app also includes a `Play` button. It only starts Gothic 1 Remake and does
not run `Optimize` automatically. When Steam ownership is detected, Play starts
the game through Steam; otherwise the Settings page can use a detected or
manually selected game executable.

The app creates backups before replacing managed config files. Backup restore and
`Reset to Vanilla` are available from the desktop UI.

## Manual Presets

Manual INI installation is still possible, but it is no longer the recommended
path.

The better manual path is the app's `Copy` function: choose the same preset and
optional tweaks in G1R Optimizer, click `Copy`, and paste the generated
app-managed settings into your own INI files. This keeps manual edits aligned
with the current desktop tool output.

The bundled preset folders are a lower-level fallback:

1. Close Gothic 1 Remake completely.
2. Open the config folder listed above.
3. Back up existing `Engine.ini` and `Scalability.ini` if they exist.
4. Pick the preset matching your GPU VRAM.
5. Copy `Engine.ini` and `Scalability.ini` from that preset folder into the
   config folder.
6. Set `Engine.ini` and `Scalability.ini` to read-only. This prevents the game
   from removing or rewriting them on launch.
7. Launch and test.

## Development

Run tests and the app from the repository:

```bash
cargo test -p optimizer-core
cd app
npm install
npm run dev
```

Build commands:

```bash
cd app
npm run build
```

The Tauri bundle includes the repository `Presets/` directory as an app resource.
See `docs/desktop-app.md` for architecture notes.

## Presets

```text
4 GB VRAM   -> 1536 MB
6 GB VRAM   -> 3072 MB
8 GB VRAM   -> 4096 MB
10 GB VRAM  -> 5120 MB
12 GB VRAM  -> 6144 MB
16 GB VRAM  -> 8192 MB
20 GB VRAM  -> 10240 MB
24+ GB VRAM -> 12288 MB
```

If streaming stutter remains and your VRAM usage has several GB of headroom, try
one preset higher. If you see crashes, new long hitches or VRAM usage close to
the GPU limit, use one preset lower.

## Notes

The streaming fix and Balanced Overdose Performance target different problems.
The streaming fix mainly targets stutter, texture pop-in and frametime
instability. Balanced Overdose Performance targets average FPS on the Overdose
profile and may trade some visual quality for speed.

The `Game Tweaks` page includes an optional `Skip Intro Videos` switch. It
writes `Game.ini` to remove startup logo/legal movies from the startup loading
screen list while keeping the engine loading screen, so the game reaches the
menu without an extra click. It does not delete, overwrite, or rename original
video files.

`r.MotionBlurQuality=0` is included because many users disable motion blur in
the menu, while the engine profile behind Overdose can still set engine-side
motion blur quality. Remove that line from `Engine.ini` if you want engine-side
motion blur.

No `GameUserSettings.ini` is included. Resolution, fullscreen mode, frame limit
and normal in-game quality sliders remain under user control.

## Tested Setup

The initial tuning was tested on:

```text
CPU: AMD Ryzen 7 9800X3D
GPU: Radeon RX 7900 XT 20 GB
Resolution: 2560x1440
OS/runtime: Linux + Steam Proton
```

On that setup, the `20GB_VRAM_10240MB` preset noticeably reduced drops and made
the Swamp Camp area feel smoother.

Balanced Overdose Performance was evaluated with before/after comparison scenes
that keep the FPS overlay visible. Those tested scenes showed about 20-30 FPS
higher performance versus standard Overdose.
