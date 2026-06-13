# NexusMods App Graphics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the approved NexusMods cover and header image set for G1R Optimizer.

**Architecture:** Use the current static app UI as the source of truth for the app-window visual, capture it through Chromium with the app's built-in browser fallback data, then compose final NexusMods assets deterministically. Generated or existing atmospheric imagery is used only as background texture; all final text and UI framing are rendered by the composition step for legibility.

**Tech Stack:** Static HTML/CSS/JavaScript UI, local HTTP server, Chromium screenshot capture, ImageMagick validation.

---

## File Structure

- `tmp/nexus-graphics/`: local scratch directory for screenshots, generated backgrounds, and intermediate renders.
- `ModImages/G1R_Optimizer_NexusMods_Cover_1280x720.png`: final cover asset.
- `ModImages/G1R_Optimizer_NexusMods_Header.png`: final header asset.
- `docs/superpowers/plans/2026-06-12-nexusmods-app-graphics.md`: this execution plan.

## Task 1: Capture Real App UI

- [ ] **Step 1: Start a local static server for `app/ui`**

Run: `python3 -m http.server 8123 --directory app/ui`

Expected: server listens on `http://127.0.0.1:8123/`.

- [ ] **Step 2: Capture the app UI**

Run Chromium headless against `http://127.0.0.1:8123/index.html`, wait for the fallback UI to render, and save a screenshot to:

`tmp/nexus-graphics/app-ui.png`

Expected: screenshot contains the G1R Optimizer sidebar, streaming preset view, and preview panel.

- [ ] **Step 3: Inspect screenshot dimensions**

Run: `magick identify tmp/nexus-graphics/app-ui.png`

Expected: one PNG screenshot large enough to crop into the final graphics.

## Task 2: Build Final Assets

- [ ] **Step 1: Prepare composition script**

Create a temporary Node script in `tmp/nexus-graphics/render-assets.mjs` that loads the UI screenshot and renders two SVG-based compositions:

- Cover: `1280x720`
- Header: wide layout using the same visual language

The script should render text as SVG text and include the staged app screenshot as an embedded image.

- [ ] **Step 2: Render assets**

Run the temporary renderer to write:

- `ModImages/G1R_Optimizer_NexusMods_Cover_1280x720.png`
- `ModImages/G1R_Optimizer_NexusMods_Header.png`

Expected: both files exist and share the approved app-first direction.

- [ ] **Step 3: Validate image metadata**

Run: `magick identify ModImages/G1R_Optimizer_NexusMods_Cover_1280x720.png ModImages/G1R_Optimizer_NexusMods_Header.png`

Expected: cover is `1280x720`; header is a wide PNG.

## Task 3: Visual QA

- [ ] **Step 1: Inspect both final assets**

Open the files with the local image viewer tool.

Expected: text is legible, app window dominates, and the two assets feel like one matched NexusMods set.

- [ ] **Step 2: Check git status**

Run: `git status --short`

Expected: final assets and this plan are the only tracked additions or modifications relevant to this task.
