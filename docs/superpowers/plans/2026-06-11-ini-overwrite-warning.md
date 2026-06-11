# INI Overwrite Warning Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add detection and UI confirmation before Optimize overwrites untracked or externally modified INI files.

**Architecture:** `optimizer-core` owns manifest persistence and file-state classification because it already owns INI reads and writes. Tauri DTOs expose the classification as a string, and the browser UI uses the existing modal pattern to confirm risky Optimize actions.

**Tech Stack:** Rust workspace crate, Tauri command DTOs, vanilla HTML/CSS/JavaScript UI.

---

### Task 1: Core Manifest And Preview State

**Files:**
- Modify: `optimizer-core/Cargo.toml`
- Modify: `optimizer-core/src/installer.rs`
- Modify: `optimizer-core/src/lib.rs`

- [ ] Add failing core tests for untracked existing files, modified managed files, and clean managed files.
- [ ] Implement manifest structs, stable content checksum, manifest load/save helpers, and preview classification.
- [ ] Update `install_preset` to write manifest entries after successful file writes.
- [ ] Export the new preview state type from `optimizer-core/src/lib.rs`.
- [ ] Run `cargo test -p optimizer-core`.

### Task 2: Tauri DTO

**Files:**
- Modify: `app/src-tauri/src/lib.rs`

- [ ] Add `modification_state` to `FilePreviewDto`.
- [ ] Map the core enum values to stable lowercase strings.
- [ ] Run `cargo check --manifest-path app/src-tauri/Cargo.toml`.

### Task 3: UI Warning Modal

**Files:**
- Modify: `app/ui/index.html`
- Modify: `app/ui/main.js`
- Modify: `app/ui/styles.css`

- [ ] Add a Preview tracking column that displays `Clean`, `New`, `Untracked`, or `Modified`.
- [ ] Reuse the existing confirmation modal machinery with dynamic title, body, and confirm button text.
- [ ] Before `install_preset`, open the overwrite warning modal when preview contains `untracked` or `modified`.
- [ ] Update static preview data to include `modification_state`.

### Task 4: Verification

**Files:**
- Test: `optimizer-core/src/installer.rs`
- Test: `app/src-tauri/src/lib.rs`
- Test: `app/ui/main.js`

- [ ] Run `cargo fmt`.
- [ ] Run `cargo test -p optimizer-core`.
- [ ] Run `cargo check --manifest-path app/src-tauri/Cargo.toml`.
- [ ] Run available UI tests if the repo exposes a local script or direct Node commands.
