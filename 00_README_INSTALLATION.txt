G1R Optimizer - Streaming Stutter Fix
Version: 1.1.0

WHAT THIS TOOL DOES

G1R Optimizer is a desktop tool for Gothic 1 Remake. It applies two main tuning
paths without requiring manual INI file copying: the G1R Streaming Stutter Fix
for texture streaming pressure, and Balanced Overdose Performance for
substantially higher FPS on the Overdose profile.

G1R ships with very small texture streaming pools in its scalability presets. At
Texture Quality Overdose, the default pool is only 1000 MB. On GPUs with 8 GB,
12 GB, 16 GB, 20 GB or more VRAM, this can create unnecessary texture streaming
pressure, pop-in and frametime drops.

The base streaming fix raises the texture streaming pool based on your GPU VRAM
and can add conservative shader/loading tweaks. It does not lower Lumen, Nanite,
shadows, foliage, resolution or texture quality. Optional performance toggles
are separate, previewed changes and may trade some visual quality for
performance.

Balanced Performance Tweaks are optional and off by default. In tested
comparison scenes, the Overdose-only Balanced profile improved performance by
about 20-30 FPS while keeping Lumen, Nanite and virtual shadows enabled.

Additional optional features include shader/PSO stutter tests, Game.ini intro
video skipping, backups, restore and Reset to Vanilla.


DESKTOP TOOL INSTALLATION

1. Close Gothic 1 Remake completely.
2. Start G1R Optimizer.
3. Use the detected config folder, or browse to the config folder listed below.
4. Select the recommended preset, choose another VRAM preset, or enter a custom
   texture pool size.
5. Enable any optional performance or game tweaks you want.
6. Review the preview. If existing INI files contain custom settings, choose
   Merge to keep them or Use App Settings Only to replace them.
7. Press Optimize.
8. Start the game and test.

The app creates backups before replacing managed config files. Backup restore
and Reset to Vanilla are available inside the desktop UI.


CONFIG LOCATIONS

Windows / Steam:
%LOCALAPPDATA%\G1R\Saved\Config\Windows\

Linux / Steam Proton example:
<SteamLibrary>/steamapps/compatdata/1297900/pfx/drive_c/users/steamuser/AppData/Local/G1R/Saved/Config/Windows/

Alternate paths for older builds may include:
Gothic\Saved\Config\WindowsNoEditor
G1R_NyrasDemo\Saved\Config\Windows


MANUAL PRESET FALLBACK

Manual INI installation is still possible, but the desktop tool is the
recommended path.

For manual editing, prefer the app's Copy function. Select the preset and
optional tweaks in G1R Optimizer, press Copy, then paste the generated
app-managed settings into your own INI files. This keeps manual edits aligned
with the current desktop tool output.

The bundled preset folders are a lower-level fallback:

1. Close Gothic 1 Remake completely.
2. Open the config folder listed above.
3. Back up your existing Engine.ini and Scalability.ini if they exist.
4. Pick the preset matching your GPU VRAM.
5. Copy Engine.ini and Scalability.ini from that preset folder into the config folder.
6. Set Engine.ini and Scalability.ini to read-only. This is important because the
   game may remove or rewrite them on launch.
7. Start the game and test.


PRESET GUIDE

4 GB VRAM   -> 1536 MB preset
6 GB VRAM   -> 3072 MB preset
8 GB VRAM   -> 4096 MB preset
10 GB VRAM  -> 5120 MB preset
12 GB VRAM  -> 6144 MB preset
16 GB VRAM  -> 8192 MB preset
20 GB VRAM  -> 10240 MB preset
24+ GB VRAM -> 12288 MB preset

If you still see streaming stutter and your VRAM usage has several GB of
headroom, try one preset higher.

If you see crashes, new long hitches or VRAM usage near the GPU limit, use one
preset lower.


NOTES

The streaming fix and Balanced Overdose Performance target different problems.
The streaming fix targets texture pop-in, streaming pressure and frametime
instability. Balanced Overdose Performance targets average FPS on the Overdose
profile and may trade some visual quality for speed.

r.MotionBlurQuality=0 is included because many players disable motion blur in
the menu, but the engine profile behind Overdose still sets the engine-side
motion blur quality to 4. Remove that line from Engine.ini if you want
engine-side motion blur.

No GameUserSettings.ini is included. Resolution, fullscreen mode, frame limit
and the normal in-game quality sliders should stay under user control.
