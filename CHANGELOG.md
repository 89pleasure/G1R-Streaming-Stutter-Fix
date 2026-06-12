# Changelog

## Unreleased

- Repositioned the public documentation around `G1R Optimizer` as the primary
  desktop tool workflow.
- Presented the Streaming Stutter Fix and Balanced Overdose Performance as
  equal main features, including the observed 20-30 FPS uplift in tested
  Overdose comparison scenes.
- Documented manual INI preset copying as a fallback instead of the recommended
  installation path.

## 1.1.0

- Added the Tauri/Rust desktop app workflow for preset installation and config
  optimization.
- Added a separate Rust optimizer core for preset discovery, path detection,
  hardware recommendations, install preview, backups, restore and reset.
- Added a static desktop UI for preset selection, optional tweaks, config
  installation and diagnostics.
- Added optional balanced `Scalability.ini` performance tweaks in the desktop
  app.

## 1.0.0

- Added VRAM-based presets from 4 GB to 24 GB.
- Added `Engine.ini` shader/loading/streaming tweaks.
- Added `Scalability.ini` texture profile overrides so `r.Streaming.PoolSize`
  survives G1R scalability profile application.
- Added installation notes and findings documentation.
- Added mod cover artwork.
