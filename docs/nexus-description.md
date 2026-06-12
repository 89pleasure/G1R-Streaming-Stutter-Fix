# G1R Optimizer - Streaming Stutter Fix

Desktop tool for Gothic 1 Remake that applies two main tuning paths without
requiring manual INI file copying: VRAM-based texture streaming fixes for
stutter/pop-in, and Balanced Overdose Performance for substantially higher FPS
on the Overdose profile.

## Why

The game ships with very small texture streaming pools:

```text
Texture Quality Low:       200 MB
Texture Quality Medium:    300 MB
Texture Quality High-ish:  400-500 MB
Texture Quality Overdose:  1000 MB
```

Many players use Texture Quality Overdose on modern GPUs, which means the
default pool can still be only `1000 MB`. On GPUs with 8-24 GB VRAM this can
cause unnecessary texture streaming pressure, texture pop-in and frametime drops.

The base streaming fix raises the texture streaming pool based on your GPU VRAM
and can add conservative shader/loading tweaks. It does not reduce Lumen, Nanite,
shadow quality, texture quality, view distance or resolution.

Balanced Overdose Performance is the second main feature. It is optional, off by
default, and applies conservative `Scalability.ini` caps only to the Overdose
profile. In tested comparison scenes, it improved performance by about 20-30 FPS
while keeping Lumen, Nanite and virtual shadows enabled. The trade-offs are
focused on shadow, reflection, volumetric and post-processing cost.

The desktop tool:

- detects common Windows and Linux/Proton config folders
- recommends a VRAM preset when hardware detection is available
- previews changes before writing files
- backs up existing `Engine.ini`, `Scalability.ini` and `Game.ini`
- can merge app settings into existing custom INI files
- can restore backups or reset managed files back to vanilla

Optional `Balanced Performance Tweaks` do not write `GameUserSettings.ini`. This
mode is labeled `Overdose only` because the current tweak set targets the
Overdose profile, not the lower profiles.

Optional `Game Tweaks` include `Skip Intro Videos`. This writes `Game.ini` to
skip startup logo/legal movies while keeping the engine loading screen. Unlike
file-replacement intro skip mods, it does not delete, overwrite, or rename the
game's original video files.

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

If you still see streaming stutter and your VRAM usage has several GB of
headroom, try one preset higher.

If you see crashes, new long hitches or VRAM usage near the GPU limit, use one
preset lower.

## Installation

1. Close Gothic 1 Remake completely.
2. Start G1R Optimizer.
3. Use the detected config folder, or browse to:

   ```text
   %LOCALAPPDATA%\G1R\Saved\Config\Windows\
   ```

4. Select the recommended preset, choose another VRAM preset, or enter a custom
   texture pool size.
5. Enable optional performance or game tweaks if wanted.
6. Review the preview. If existing INI files contain custom settings, choose
   `Merge` to keep them or `Use App Settings Only` to replace them.
7. Press `Optimize`.
8. Launch and test.

Linux / Proton users can use the matching Proton prefix path:

```text
steamapps/compatdata/1297900/pfx/drive_c/users/steamuser/AppData/Local/G1R/Saved/Config/Windows/
```

Manual INI installation from the bundled `Presets` folders is still possible,
but the desktop tool is the recommended path. For manual editing, prefer the
app's `Copy` function: select the preset and optional tweaks in G1R Optimizer,
press `Copy`, then paste the generated app-managed settings into your own INI
files.

## Notes

The streaming fix and Balanced Overdose Performance target different problems:
the streaming fix targets pop-in, streaming pressure and frametime instability;
Balanced Overdose Performance targets average FPS on the Overdose profile.
