# Balanced Performance Optimize Design

## Goal

Add an opt-in `Balanced Performance Tweaks` mode to the desktop app and rename the
main `Install` workflow to `Optimize`.

## Scope

- The current VRAM-based streaming preset remains the default optimization path.
- Balanced performance tweaks are disabled by default and require an explicit user toggle.
- V1 does not write `GameUserSettings.ini`.
- V1 does not disable Lumen, Nanite, virtual shadows, or core lighting features.
- The preview and install commands must receive the opt-in state and show whether
  `Scalability.ini` will include the balanced tweak block.

## UI

The sidebar item and primary action become `Optimize`. The page keeps the current
compact layout:

- `Streaming Preset` panel with hardware recommendation badge.
- `Game Config Folder` panel with read-only toggles.
- New `Balanced Performance Tweaks` panel below the first row.
- Existing `Preview` panel remains below all choices and updates when the toggle changes.

## Core Behavior

When balanced performance is off, the installer writes the preset files as it does
today. When balanced performance is on, the installer appends a conservative
set of Overdose scalability profile caps to the generated `Scalability.ini`
content before writing it. Existing backups, read-only handling, and restore
behavior remain unchanged.

## Settings Policy

The first balanced block only uses post-processing and conservative render-cost
settings with low expected visual impact. It avoids aggressive quality-group
changes and avoids `sg.*` writes.
