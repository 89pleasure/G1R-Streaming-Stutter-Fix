# INI Overwrite Warning Design

## Goal

Warn users before Optimize overwrites existing INI files that may contain mod or manual edits.

## Background

The app currently previews and writes planned `Engine.ini`, `Game.ini`, and `Scalability.ini`
files through `optimizer-core`. Existing files are backed up before replacement, but the UI does
not distinguish files last written by the app from files created or changed outside the app.

## Design

The core installer will maintain a small manifest under
`.g1r-streaming-stutter-fix/manifest.json` in the selected config folder. Each manifest entry
records the managed file name, a stable content checksum for the last app-written contents, the
byte count, and the install timestamp.

Preview will classify each planned file:

- `missing`: no target file exists.
- `unchanged`: target file exists and matches the manifest checksum.
- `untracked`: target file exists but has no manifest entry.
- `modified`: target file exists and differs from the manifest checksum.

`untracked` covers first-run installs where users already have INI files from mods or manual edits.
`modified` covers files changed after a previous app install. Both are overwrite-risk states.

## UI Flow

The Preview table will expose the tracking state for each file. On Optimize, the UI checks the
current preview. If any planned file is `untracked` or `modified`, a confirmation modal explains
that Optimize will overwrite those settings and that a backup will be created first. The user must
choose between cancelling and overwriting.

## Out Of Scope

Merging app settings into existing modded INI files is intentionally deferred.
