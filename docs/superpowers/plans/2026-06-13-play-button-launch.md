# Play Button Launch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Play button that launches Gothic 1 Remake through detected Steam installation first, with an automatically detected or manually overridden executable fallback.

**Architecture:** Put launch target detection in `optimizer-core` next to existing Steam path detection, expose launch candidates through Tauri app state, and keep process/URI launching in `app/src-tauri`. The static UI stores launch preferences, displays launch target controls in Settings, and adds a Play button beside Optimize and Copy.

**Tech Stack:** Rust workspace (`optimizer-core`, Tauri v2 app), static JavaScript UI, Node-based UI tests, Cargo tests.

---

### Task 1: Core Launch Candidate Detection

**Files:**
- Modify: `optimizer-core/src/paths.rs`
- Modify: `optimizer-core/src/lib.rs`

- [ ] **Step 1: Write failing core tests**

Add tests to `optimizer-core/src/paths.rs`:

```rust
#[test]
fn detects_steam_launch_candidate_from_appmanifest() {
    let root = unique_temp_dir("steam-launch-candidate");
    let steamapps = root.join("SteamLibrary").join("steamapps");
    fs::create_dir_all(&steamapps).expect("create steamapps");
    fs::write(steamapps.join("appmanifest_1297900.acf"), "\"appid\" \"1297900\"")
        .expect("write manifest");

    let candidates = launch_candidates_from_steamapps_roots(&[steamapps], None);

    assert_eq!(candidates[0].kind, LaunchCandidateKind::Steam);
    assert_eq!(candidates[0].label, "Steam: Gothic 1 Remake");
    assert!(candidates[0].exists);
}

#[test]
fn steam_launch_candidate_is_preferred_over_executable() {
    let root = unique_temp_dir("steam-launch-preferred");
    let steamapps = root.join("SteamLibrary").join("steamapps");
    let game_dir = steamapps.join("common").join("Gothic 1 Remake");
    fs::create_dir_all(&game_dir).expect("create game dir");
    fs::write(steamapps.join("appmanifest_1297900.acf"), "\"appid\" \"1297900\"")
        .expect("write manifest");
    fs::write(game_dir.join("G1R.exe"), "").expect("write exe");

    let candidates = launch_candidates_from_steamapps_roots(&[steamapps], None);

    assert_eq!(candidates[0].kind, LaunchCandidateKind::Steam);
    assert!(candidates.iter().any(|candidate| candidate.kind == LaunchCandidateKind::Executable));
}

#[test]
fn includes_valid_manual_executable_candidate() {
    let root = unique_temp_dir("manual-launch-candidate");
    let executable = root.join("G1R.exe");
    fs::write(&executable, "").expect("write executable");

    let candidates = launch_candidates_from_steamapps_roots(&[], Some(&executable));

    assert_eq!(candidates[0].kind, LaunchCandidateKind::Executable);
    assert_eq!(candidates[0].source, "manual");
    assert!(candidates[0].exists);
}

#[test]
fn marks_missing_manual_executable_candidate() {
    let root = unique_temp_dir("missing-manual-launch-candidate");
    let executable = root.join("Missing.exe");

    let candidates = launch_candidates_from_steamapps_roots(&[], Some(&executable));

    assert_eq!(candidates[0].kind, LaunchCandidateKind::Executable);
    assert_eq!(candidates[0].source, "manual");
    assert!(!candidates[0].exists);
}
```

Also add a temp helper inside the existing test module:

```rust
fn unique_temp_dir(name: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!(
        "g1r-optimizer-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}
```

- [ ] **Step 2: Run RED verification**

Run: `cargo test -p optimizer-core paths::tests::detects_steam_launch_candidate_from_appmanifest`

Expected: fail to compile because `LaunchCandidateKind` and `launch_candidates_from_steamapps_roots` do not exist yet.

- [ ] **Step 3: Implement launch detection**

Add public types and functions in `optimizer-core/src/paths.rs`:

```rust
pub const GOTHIC_1_REMAKE_STEAM_APP_ID: &str = "1297900";
pub const GOTHIC_1_REMAKE_STEAM_URI: &str = "steam://rungameid/1297900";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchCandidateKind {
    Steam,
    Executable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchCandidate {
    pub id: String,
    pub kind: LaunchCandidateKind,
    pub label: String,
    pub path: PathBuf,
    pub exists: bool,
    pub source: String,
}

pub fn detect_launch_candidates(manual_executable_path: Option<&Path>) -> Vec<LaunchCandidate> {
    launch_candidates_from_steamapps_roots(&steamapps_roots(), manual_executable_path)
}

pub fn launch_candidates_from_steamapps_roots(
    steamapps_roots: &[PathBuf],
    manual_executable_path: Option<&Path>,
) -> Vec<LaunchCandidate> {
    let mut candidates = Vec::new();
    for steamapps in steamapps_roots {
        push_steam_launch_candidate(&mut candidates, steamapps);
        push_steam_executable_candidates(&mut candidates, steamapps);
    }
    push_manual_launch_candidate(&mut candidates, manual_executable_path);

    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter(|candidate| seen.insert(candidate.id.clone()))
        .collect()
}
```

Implement helpers for Steam manifest path, common executable names, candidate IDs, and manual path
validation. Export the new types/functions from `optimizer-core/src/lib.rs`.

- [ ] **Step 4: Run GREEN verification**

Run: `cargo test -p optimizer-core paths::tests::`

Expected: all path tests pass.

- [ ] **Step 5: Commit**

Run:

```bash
git add optimizer-core/src/paths.rs optimizer-core/src/lib.rs
git commit -m "feat: detect game launch targets"
```

### Task 2: Tauri Launch State and Command

**Files:**
- Modify: `app/src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing Rust tests for launch argument resolution**

Add unit tests in `app/src-tauri/src/lib.rs` under a new `#[cfg(test)] mod tests`:

```rust
#[test]
fn steam_launch_command_uses_steam_uri() {
    let request = LaunchGameRequest {
        kind: "steam".to_string(),
        path: "/tmp/appmanifest_1297900.acf".to_string(),
    };

    let command = launch_command_for_request(&request).expect("command");

    assert_eq!(command.program, platform_open_program());
    assert!(command.args.iter().any(|arg| arg == GOTHIC_1_REMAKE_STEAM_URI));
}

#[test]
fn executable_launch_command_uses_selected_path() {
    let request = LaunchGameRequest {
        kind: "executable".to_string(),
        path: "/tmp/G1R.exe".to_string(),
    };

    let command = launch_command_for_request(&request).expect("command");

    assert_eq!(command.program, "/tmp/G1R.exe");
    assert!(command.args.is_empty());
}

#[test]
fn invalid_launch_kind_returns_error() {
    let request = LaunchGameRequest {
        kind: "unsupported".to_string(),
        path: "/tmp/G1R.exe".to_string(),
    };

    assert!(launch_command_for_request(&request).is_err());
}
```

- [ ] **Step 2: Run RED verification**

Run: `cargo test -p g1r-optimizer-app launch_command`

Expected: fail to compile because launch request and command helpers do not exist.

- [ ] **Step 3: Implement Tauri DTOs and command**

Add:

- `LaunchCandidateDto`
- `LaunchGameRequest`
- `LaunchCommand`
- `launch_game(request: LaunchGameRequest) -> Result<LaunchReportDto, String>`
- `launch_command_for_request`
- `platform_open_program`

Extend `AppStateDto` with `launch_candidates: Vec<LaunchCandidateDto>` and call
`detect_launch_candidates(None)` in `get_app_state`.

Register `launch_game` in `tauri::generate_handler!`.

- [ ] **Step 4: Run GREEN verification**

Run: `cargo test -p g1r-optimizer-app launch_command`

Expected: launch command tests pass.

- [ ] **Step 5: Commit**

Run:

```bash
git add app/src-tauri/src/lib.rs
git commit -m "feat: expose game launch command"
```

### Task 3: UI Preferences, Settings, and Play Button

**Files:**
- Modify: `app/ui/preferences.js`
- Modify: `app/ui/preferences.test.mjs`
- Modify: `app/ui/index.html`
- Modify: `app/ui/main.js`
- Modify: `app/ui/styles.css`
- Modify: `app/ui/locales/en.js`
- Modify: `app/ui/locales/de.js`
- Modify: `app/ui/locales/pl.js`
- Modify: `app/ui/locales/es.js`
- Modify: `app/ui/locales/fr.js`
- Modify: `app/ui/locales/it.js`
- Modify: `app/ui/locales/ru.js`
- Modify: `app/ui/locales/ja.js`
- Modify: `app/ui/locales/zh.js`
- Modify: `app/ui/locales/pt.js`
- Modify: `app/ui/localization-layout.test.mjs`

- [ ] **Step 1: Write failing UI tests**

Extend `app/ui/preferences.test.mjs` default and save/load assertions:

```js
assert.equal(preferences.selectedLaunchTargetId, "");
assert.equal(preferences.manualExecutablePath, "");
```

and:

```js
selectedLaunchTargetId: "steam:/tmp/appmanifest_1297900.acf",
manualExecutablePath: "/games/G1R/G1R.exe",
```

Extend `app/ui/localization-layout.test.mjs`:

```js
assert.match(settingsView, /id="launchTargetSelect"/);
assert.match(settingsView, /id="manualExecutableInput"/);
assert.match(settingsView, /id="browseExecutableButton"/);
assert.match(html, /id="playButton"/);
assert.match(mainJs, /elements\.playButton\.addEventListener\("click", launchGame\)/);
```

- [ ] **Step 2: Run RED verification**

Run:

```bash
node app/ui/preferences.test.mjs
node app/ui/localization-layout.test.mjs
```

Expected: fail because launch preferences and UI elements do not exist.

- [ ] **Step 3: Implement UI state and controls**

Add launch state fields to `state` in `main.js`, bind the new elements, persist launch preferences,
load `appState.launch_candidates`, render launch target options, and implement `launchGame()` calling
`invokeCommand("launch_game", { request: selectedLaunchRequest() })`.

Add Settings controls:

```html
<section class="panel target-panel" aria-labelledby="launchHeading">
  <div class="panel-header">
    <h2 id="launchHeading" data-i18n="settings.launchHeading">Game Launch</h2>
    <span class="pill" id="launchStatus" data-i18n="pathStatus.unchecked">Unchecked</span>
  </div>
  <label class="field-label" for="launchTargetSelect" data-i18n="settings.launchTargets">
    Launch target
  </label>
  <select id="launchTargetSelect"></select>
  <label class="field-label" for="manualExecutableInput" data-i18n="settings.executablePath">
    Executable path
  </label>
  <div class="path-input-row">
    <input id="manualExecutableInput" type="text" spellcheck="false" />
    <button class="secondary-button path-picker-button" id="browseExecutableButton" type="button">
      ...
    </button>
  </div>
</section>
```

Add Play button markup in the action row:

```html
<button class="secondary-button play-button" id="playButton" type="button" data-i18n="actions.play">
  Play
</button>
```

- [ ] **Step 4: Run GREEN verification**

Run:

```bash
node app/ui/preferences.test.mjs
node app/ui/localization-layout.test.mjs
```

Expected: both tests pass.

- [ ] **Step 5: Commit**

Run:

```bash
git add app/ui
git commit -m "feat: add play button controls"
```

### Task 4: Full Verification and Documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/desktop-app.md`

- [ ] **Step 1: Update docs**

Add the Play button to feature lists and architecture notes. State explicitly that Play launches the
game and does not run Optimize.

- [ ] **Step 2: Run full verification**

Run:

```bash
cargo test -p optimizer-core
cargo test -p g1r-optimizer-app
node app/ui/preferences.test.mjs
node app/ui/localization-layout.test.mjs
node app/ui/i18n.test.mjs
cargo check -p g1r-optimizer-app
```

Expected: every command exits with status 0.

- [ ] **Step 3: Commit docs and verification fixes**

Run:

```bash
git add README.md docs/desktop-app.md
git commit -m "docs: document play button launch"
```

### Task 5: Publish PR

**Files:**
- No source file changes expected.

- [ ] **Step 1: Inspect final diff**

Run:

```bash
git status --short
git diff --stat main...HEAD
```

Expected: only intended Play button launch files are changed, plus the existing user-owned
`docs/nexus-description.md` remains unstaged if still modified.

- [ ] **Step 2: Push branch**

Run: `git push -u origin codex/play-button-launch`

Expected: branch pushes successfully.

- [ ] **Step 3: Create draft PR**

Use the GitHub connector or `gh pr create` fallback with:

```text
Title: Add Play button game launch
Body:
- detects Steam install targets for Gothic 1 Remake
- adds executable fallback with manual override
- adds Play button that launches without applying optimizer changes

Tests:
- cargo test -p optimizer-core
- cargo test -p g1r-optimizer-app
- node app/ui/preferences.test.mjs
- node app/ui/localization-layout.test.mjs
- node app/ui/i18n.test.mjs
- cargo check -p g1r-optimizer-app
```
