# NexusMods App Graphics Design

## Context

G1R Optimizer is now the primary distribution path for the Gothic 1 Remake
streaming and performance presets. The current NexusMods graphics still read
mostly like a classic INI mod: atmospheric Gothic background, large "Streaming
Stutter Fix" title, and configuration-file emphasis.

The new graphics should present the project as a safer desktop optimizer app:
detect config folders, preview changes, apply VRAM-based presets, back up files,
merge with existing INI edits, restore backups, and optionally enable Balanced
Overdose Performance.

## Assets

Create two related NexusMods-facing bitmap assets in `ModImages/`:

- Cover image: 16:9 presentation image, exported at `1280x720`.
- Header image: wider and calmer page-header image, exported from a wide layout
  suitable for NexusMods page placement.

Both assets should share one visual system so the mod page feels intentional.
The cover can carry more explanatory text; the header should be quieter and
more spacious.

## Visual Direction

Use the approved **App-first hybrid** direction.

The real G1R Optimizer desktop UI should be the main credibility signal. Present
it as an inscribed app window rather than a raw screenshot pasted on a
background. Surround it with restrained performance and streaming context:
cyan streaming/data lines, small preset or backup cues, and warm Gothic-style
lighting accents.

The Gothic mood stays present but secondary. Use a dark, game-adjacent
background treatment inspired by swamp/camp night lighting, but do not rely on
official game logos, official key art, or recognisable copyrighted screenshots.

## Cover Composition

The cover should communicate the product quickly in a small NexusMods card.

- Primary title: `G1R Optimizer`
- Secondary copy: `Streaming Stutter Fix + Balanced Overdose Performance`
- Support cues: `VRAM Presets`, `Safe Merge`, `Backups`
- Main image: staged G1R Optimizer app window, large enough that it reads as a
  desktop tool.
- Background: dark Gothic atmosphere with subtle cyan data/streaming movement
  and warm torch-like highlights.

The cover should feel like an app/tool announcement, not a landscape poster.

## Header Composition

The header should use the same art direction but reduce text density.

- Primary title: `G1R Optimizer`
- Support copy: `Detect • Preview • Backup • Restore`
- Small contextual line: `VRAM-based presets for Gothic 1 Remake`
- Main image: app window placed left or center-left, with enough negative space
  for wide-page cropping.

The header should remain readable if NexusMods crops or scales it.

## Style Constraints

- Keep text crisp and large enough to survive NexusMods scaling.
- Use cream/off-white title text, cyan accent lines, and restrained warm orange
  highlights.
- Avoid one-note blue/purple styling; the palette should mix cyan app accents
  with warm Gothic lighting.
- Avoid fake NexusMods branding, official Gothic logos, watermarks, and exact
  game key-art reproduction.
- Avoid over-promising performance. If a numerical FPS claim appears, it must
  be contextual and derived from existing project copy. The preferred graphics
  avoid a large numerical claim and emphasize workflow instead.
- Keep the UI screenshot or mockup consistent with the current app: presets,
  preview, backup/restore, merge safety, and optimizer workflow.

## Acceptance Criteria

- Both final files are saved in `ModImages/`.
- Cover and header look like a matched set.
- The desktop app is the dominant signal in both images.
- Text is legible at NexusMods thumbnail and page scales.
- The cover works as a standalone card, and the header works as a wider,
  lower-density page asset.
- No temporary visual-companion files are committed.
