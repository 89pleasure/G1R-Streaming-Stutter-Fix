# G1R - Streaming Stutter Fix

VRAM-based texture streaming presets to reduce pop-in, streaming pressure and
frametime drops in Gothic 1 Remake.

## Why

The game ships with very small texture streaming pools:

```text
Texture Quality Low:       200 MB
Texture Quality Medium:    300 MB
Texture Quality High-ish:  400-500 MB
Texture Quality Cine:      1000 MB
```

Many players use Texture Quality Cine on modern GPUs, which means the default
pool can still be only `1000 MB`. On GPUs with 8-24 GB VRAM this can cause
unnecessary texture streaming pressure, texture pop-in and frametime drops.

This mod raises the texture streaming pool based on your GPU VRAM and adds
conservative shader/loading tweaks. It does not reduce Lumen, Nanite, shadow
quality, texture quality, view distance or resolution.

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

If you still see streaming stutter and your VRAM usage has several GB of
headroom, try one preset higher.

If you see crashes, new long hitches or VRAM usage near the GPU limit, use one
preset lower.

## Installation

1. Close Gothic 1 Remake completely.
2. Go to:

   ```text
   %LOCALAPPDATA%\G1R\Saved\Config\Windows\
   ```

3. Back up `Engine.ini` and `Scalability.ini` if they exist.
4. Copy `Engine.ini` and `Scalability.ini` from your chosen preset folder.
5. Set `Engine.ini` to read-only.
6. Launch and test.

Linux / Proton users can use the matching Proton prefix path:

```text
steamapps/compatdata/1297900/pfx/drive_c/users/steamuser/AppData/Local/G1R/Saved/Config/Windows/
```

## Notes

This is primarily a stutter/frametime/streaming fix, not a guaranteed average
FPS mod. Average FPS can improve if the game was being limited by streaming
pressure, but the primary goal is smoother gameplay.
