# G1R - Streaming Stutter Fix

VRAM-based texture streaming presets for Gothic 1 Remake.

This mod raises the Unreal Engine texture streaming pool based on your GPU VRAM
and adds conservative shader/loading tweaks. It is intended to reduce texture
pop-in, streaming pressure and frametime drops without lowering Lumen, Nanite,
shadow quality, view distance, resolution or texture quality.

![G1R - Streaming Stutter Fix cover](ModImages/G1R_StreamingStutterFix_cover_1280x720.jpg)

## What It Changes

Gothic 1 Remake ships with very small texture streaming pools in its default
scalability profiles:

```text
TextureQuality@0     200 MB
TextureQuality@1     300 MB
TextureQuality@2     400 MB
TextureQuality@3     500 MB
TextureQuality@Cine  1000 MB
```

Many users play with `sg.TextureQuality=4`, which maps to `TextureQuality@Cine`.
That means the default texture streaming pool can be only `1000 MB`, even on
modern GPUs with much more VRAM.

This project provides `Engine.ini` and `Scalability.ini` presets that raise that
pool in a controlled, VRAM-based way.

## Presets

```text
4 GB VRAM   -> 1536 MB
6 GB VRAM   -> 3072 MB
8 GB VRAM   -> 4096 MB
10 GB VRAM  -> 5120 MB
12 GB VRAM  -> 6144 MB
16 GB VRAM  -> 8192 MB
20 GB VRAM  -> 10240 MB
24 GB VRAM  -> 12288 MB
```

If streaming stutter remains and your VRAM usage has several GB of headroom, try
one preset higher. If you see crashes, new long hitches or VRAM usage close to
the GPU limit, use one preset lower.

## Installation

1. Close Gothic 1 Remake completely.
2. Open the config folder:

   ```text
   %LOCALAPPDATA%\G1R\Saved\Config\Windows\
   ```

   Linux / Proton example:

   ```text
   <SteamLibrary>/steamapps/compatdata/1297900/pfx/drive_c/users/steamuser/AppData/Local/G1R/Saved/Config/Windows/
   ```

3. Back up existing `Engine.ini` and `Scalability.ini` if they exist.
4. Pick the preset matching your GPU VRAM.
5. Copy `Engine.ini` and `Scalability.ini` from that preset folder into the config folder.
6. Set `Engine.ini` to read-only. Optionally set `Scalability.ini` to read-only too.
7. Launch and test.

## Notes

This is mainly a stutter/frametime/streaming fix, not a guaranteed average FPS
mod. Average FPS can improve if the game was limited by streaming pressure, but
the main goal is smoother gameplay.

`r.MotionBlurQuality=0` is included because many users disable motion blur in
the menu, while the Cine post-process profile can still set engine-side motion
blur quality. Remove that line from `Engine.ini` if you want engine-side motion
blur.

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
