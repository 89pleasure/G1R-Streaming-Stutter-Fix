G1R - Streaming Stutter Fix
Version: 1.0.0

WHAT THIS MOD DOES

G1R ships with very small texture streaming pools in its scalability presets.
At Texture Quality Cine, the default pool is only 1000 MB. On GPUs with
8 GB, 12 GB, 16 GB, 20 GB or more VRAM, this can create unnecessary texture
streaming pressure, pop-in and frametime drops.

This mod raises the texture streaming pool based on your GPU VRAM and adds
conservative shader/loading tweaks. It does not lower Lumen, Nanite, shadows,
foliage, resolution or texture quality.


CONFIG LOCATION

Windows / Steam:
%LOCALAPPDATA%\G1R\Saved\Config\Windows\

Linux / Steam Proton example:
<SteamLibrary>/steamapps/compatdata/1297900/pfx/drive_c/users/steamuser/AppData/Local/G1R/Saved/Config/Windows/

Alternate paths for older builds may include:
Gothic\Saved\Config\WindowsNoEditor
G1R_NyrasDemo\Saved\Config\Windows


INSTALLATION

1. Close Gothic 1 Remake completely.
2. Open the config folder listed above.
3. Back up your existing Engine.ini and Scalability.ini if they exist.
4. Pick the preset matching your GPU VRAM.
5. Copy Engine.ini and Scalability.ini from that preset folder into the config folder.
6. Set Engine.ini to read-only. This is important because the game may remove it.
7. Optional: set Scalability.ini to read-only too.
8. Start the game and test.


PRESET GUIDE

4 GB VRAM   -> 1536 MB preset
6 GB VRAM   -> 3072 MB preset
8 GB VRAM   -> 4096 MB preset
10 GB VRAM  -> 5120 MB preset
12 GB VRAM  -> 6144 MB preset
16 GB VRAM  -> 8192 MB preset
20 GB VRAM  -> 10240 MB preset
24 GB VRAM  -> 12288 MB preset

If you still see streaming stutter and your VRAM usage has several GB of
headroom, try one preset higher.

If you see crashes, new long hitches or VRAM usage near the GPU limit, use one
preset lower.


NOTES

This is mainly a stutter/frametime/streaming fix, not a guaranteed average FPS
mod. Average FPS can improve if the game was being limited by streaming
pressure, but the primary goal is smoother gameplay.

r.MotionBlurQuality=0 is included because many players disable motion blur in
the menu, but the Cine post process profile still sets the engine-side motion
blur quality to 4. Remove that line from Engine.ini if you want engine-side
motion blur.

No GameUserSettings.ini is included. Resolution, fullscreen mode, frame limit
and the normal in-game quality sliders should stay under user control.
