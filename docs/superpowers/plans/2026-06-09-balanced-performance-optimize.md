# Balanced Performance Optimize Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add opt-in balanced scalability performance tweaks and rename the main app workflow to Optimize.

**Architecture:** `optimizer-core` owns file generation and preview semantics. The Tauri layer forwards the new opt-in flag. The static HTML/JS UI adds the toggle, renames the workflow, and displays the balanced state in Preview.

**Tech Stack:** Rust 2024 workspace, Tauri commands, plain HTML/CSS/JavaScript frontend.

---

### Task 1: Core Opt-in Scalability Content

**Files:**
- Modify: `optimizer-core/src/installer.rs`

- [x] Add `apply_balanced_performance_tweaks` to `InstallOptions`.
- [x] Add a `will_apply_balanced_performance_tweaks` field to `FilePreview`.
- [x] Generate `Scalability.ini` content from the preset file and append the balanced block only when the opt-in flag is true.
- [x] Update installer tests to assert opt-in preview and installed content.
- [x] Run `cargo test -p optimizer-core`.

### Task 2: Tauri Command Wiring

**Files:**
- Modify: `app/src-tauri/src/lib.rs`

- [x] Add `balanced_performance` to `preview_install` and `install_preset` command arguments.
- [x] Pass it into `InstallOptions`.
- [x] Expose `will_apply_balanced_performance_tweaks` on `FilePreviewDto`.
- [x] Run `cargo check --manifest-path app/src-tauri/Cargo.toml`.

### Task 3: Optimize UI

**Files:**
- Modify: `app/ui/index.html`
- Modify: `app/ui/main.js`
- Modify: `app/ui/styles.css`

- [x] Rename the main nav item and primary button from Install to Optimize.
- [x] Add the `Balanced Performance Tweaks` opt-in panel, off by default.
- [x] Pass `balancedPerformance` to preview and install commands.
- [x] Add a Preview column showing `Balanced` for `Scalability.ini` when enabled.
- [x] Run `node --check app/ui/main.js`.

### Task 4: Documentation and Visual Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/nexus-description.md`

- [x] Document that balanced performance is desktop-app opt-in.
- [x] Verify the static UI in Chromium on desktop and mobile.
- [x] Run `git diff --check`.
