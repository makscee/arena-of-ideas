# AI-Driven Visual UI Development & Testing Research

Research date: 2026-03-27 | Stack: Bevy 0.18 + egui + Rust

---

## 1. Claude Computer Use (Already Available in This Environment)

**Key finding:** The `mcp__computer-use-mcp__computer` tool is already configured in this Claude Code session. It supports:

- `get_screenshot` - capture the current screen
- `left_click` / `right_click` / `double_click` at `[x, y]` coordinates
- `type` - type text, `key` - press key combos (e.g., "ctrl+s")
- `mouse_move`, `scroll`, `left_click_drag`

**How it works:** Claude takes a screenshot, analyzes it visually, decides what to click/type, performs the action, takes another screenshot, and repeats. A red crosshair shows cursor position for calibration.

**API-level computer use** (for custom integrations) requires beta header `computer-use-2025-11-24` and supports the same actions plus enhanced ones like `hold_key`, `triple_click`, fine-grained mouse control.

**Cost:** ~$3/MTok input, $15/MTok output (screenshot-heavy workflows are expensive due to image tokens).

**Sources:**
- [Computer use tool - Claude API Docs](https://platform.claude.com/docs/en/agents-and-tools/tool-use/computer-use-tool)
- [Claude Code Desktop Docs](https://code.claude.com/docs/en/desktop)

---

## 2. Bevy 0.18 Screenshot Capture API

Bevy 0.18 has a mature screenshot system at `bevy::render::view::window::screenshot`:

### Key Types
| Type | Purpose |
|------|---------|
| `Screenshot` | Component that signals renderer to capture a frame |
| `Captured` | Marker indicating screenshot is ready |
| `ScreenshotCaptured` | Event with the image data |
| `save_to_disk` | Observer/function to save captured screenshot to disk |
| `ScreenshotPlugin` | Core plugin (included by default) |

### Programmatic Capture
```rust
// Spawn a Screenshot entity targeting the primary window
commands.spawn(Screenshot::primary_window());

// Observe when it's captured and save to disk
app.add_observer(save_to_disk("screenshot.png"));
```

### New in 0.18
- `EasyScreenshotPlugin` - one-liner for PrintScreen key capture
- `EasyScreenRecordPlugin` - video recording with programmatic start/stop
- Can capture from any render target, not just windows (via `Screenshot::render_target(image_handle)`)

**Sources:**
- [Bevy screenshot module docs](https://docs.rs/bevy/latest/bevy/render/view/window/screenshot/index.html)
- [Bevy 0.18 release notes](https://bevy.org/news/bevy-0-18/)

---

## 3. Visual Regression Testing in Rust

### insta (Snapshot Testing)
- **Text-only snapshots:** insta supports JSON, YAML, TOML, RON, CSV via Serde -- **not** image comparison
- Great for snapshotting serialized UI state (widget trees, layout data) but not pixels
- `cargo insta review` provides interactive diff review

### Image Comparison Options
| Crate | Approach |
|-------|----------|
| `image` crate | Load PNGs, pixel-by-pixel comparison |
| `dify` | Image diff library (used by egui_kittest internally) |
| `xray` | Screenshot testing wrapper with orchestration |
| `egui_kittest` (snapshot feature) | Full pipeline: render egui -> save PNG -> compare with `dify` |

### Recommended Approach for This Project
1. Use `egui_kittest` with `snapshot` + `wgpu` features for egui panel testing
2. Use Bevy's `Screenshot` API + `image` crate for full-frame game rendering comparison
3. Store reference images in `tests/snapshots/` and compare with threshold-based pixel diff

**Sources:**
- [insta crate](https://insta.rs/)
- [Screenshot testing with Rust - Tony Finn](https://tonyfinn.com/blog/rust-screenshot-testing/)

---

## 4. egui_kittest -- The Best Fit for egui UI Testing

**This is the most directly applicable tool for this project.** Built by Rerun (egui maintainers), it provides:

### Programmatic UI Interaction (via AccessKit)
```rust
use egui_kittest::{Harness, kittest::{Queryable, NodeT}};

let mut harness = Harness::new_ui(|ui: &mut egui::Ui| {
    ui.checkbox(&mut checked, "Check me!");
});

// Query by accessibility label
let checkbox = harness.get_by_label("Check me!");
assert_eq!(checkbox.accesskit_node().toggled(), Some(Toggled::False));

// Simulate click
checkbox.click();
harness.run();

// Verify state changed
let checkbox = harness.get_by_label("Check me!");
assert_eq!(checkbox.accesskit_node().toggled(), Some(Toggled::True));
```

### Visual Snapshot Testing
```rust
harness.fit_contents();
#[cfg(all(feature = "wgpu", feature = "snapshot"))]
harness.snapshot("my_test_name");
// Saves to tests/snapshots/my_test_name.png
// On re-run, compares pixel-by-pixel using dify
```

### Current Version: 0.34.0
- Uses `kittest` 0.4.0 (AccessKit-based, framework-agnostic)
- Querying: `get_by_label`, `get_by_role`, etc. (Testing Library-style API)
- Requires `wgpu` feature for rendering/snapshots

### Limitation for Bevy Integration
egui_kittest tests **standalone egui UIs**, not egui-within-Bevy. To test your game's egui panels:
- Extract UI functions as `fn my_panel(ui: &mut egui::Ui, state: &mut MyState)`
- Test them directly with `Harness::new_ui(...)` outside Bevy
- For full Bevy integration testing, use Bevy's screenshot API + computer use

**Sources:**
- [egui_kittest docs](https://docs.rs/egui_kittest/latest/egui_kittest/)
- [kittest GitHub](https://github.com/rerun-io/kittest)
- [egui-screenshot-testing](https://github.com/thomaskrause/egui-screenshot-testing)

---

## 5. macOS UI Automation (Accessibility-Based)

### MCP Servers Available
| Server | Approach |
|--------|----------|
| `macos-ui-automation` (mb-dev) | Playwright-like API for native macOS apps via accessibility |
| `macos-accessibility-mcp` (adamrdrew) | Read UI trees, find elements, click buttons via Accessibility API |
| `mcp-remote-macos-use` | Full remote macOS control from Claude |

### Setup for macOS Accessibility
1. System Settings -> Privacy & Security -> Accessibility -> Add Claude Code / Terminal
2. Install MCP server: add to `.claude/settings.json` or project MCP config
3. Claude can then query the accessibility tree of any running app

### Limitation
Bevy/egui accessibility support depends on AccessKit integration. Bevy 0.18 does have AccessKit support (`accesskit 0.21` is a dev dependency), but the accessibility tree may not expose all game-specific UI elements -- it works best for standard egui widgets.

**Sources:**
- [macOS UI Automation MCP](https://playbooks.com/mcp/mb-dev-macos-ui-automation)
- [macOS Accessibility MCP](https://lobehub.com/mcp/adamrdrew-macos-accessibility-mcp)

---

## 6. Recommended Full AI-in-the-Loop Visual Development Workflow

### Tier 1: Immediate (No Setup Required)
Use the `mcp__computer-use-mcp__computer` tool already in this session:
```
1. Claude edits Rust code
2. Claude runs `cargo run` via Bash (background)
3. Claude calls get_screenshot to see the game window
4. Claude identifies visual issues from the screenshot
5. Claude edits code to fix issues
6. Claude kills and re-launches the game
7. Claude calls get_screenshot to verify the fix
```
**Pros:** Works right now, zero setup, handles any visual element
**Cons:** Slow iteration (full compile), expensive (image tokens), fragile coordinate-based clicking

### Tier 2: Unit-Level egui Testing (Moderate Setup)
Add `egui_kittest` to the project:
```toml
[dev-dependencies]
egui_kittest = { version = "0.34", features = ["wgpu", "snapshot"] }
```
```
1. Extract egui panel functions to be testable standalone
2. Write Harness-based tests that render panels and snapshot them
3. Claude runs `cargo test` to verify visual output
4. Claude reads snapshot diffs to identify regressions
5. No need to launch the full game for UI layout work
```
**Pros:** Fast (no game launch), deterministic, CI-compatible, pixel-perfect diffs
**Cons:** Only tests egui panels in isolation, not full game rendering

### Tier 3: Full Integration (More Setup)
Combine Bevy screenshot capture + headless testing:
```
1. Add a --screenshot CLI flag to the game that:
   a. Launches the game
   b. Waits N frames for rendering to stabilize
   c. Captures screenshot via Bevy's Screenshot API
   d. Saves to disk and exits
2. Claude runs: cargo run -- --screenshot /tmp/test.png
3. Claude reads the image file to inspect it visually
4. Compare against reference images programmatically
```
**Pros:** Tests full rendering pipeline, automatable in CI
**Cons:** Requires headless GPU or display server, slower than unit tests

### Recommended Implementation Order
1. **Now:** Use computer use tool (`get_screenshot`) for ad-hoc visual verification
2. **Soon:** Refactor egui panels to be testable, add `egui_kittest` snapshot tests
3. **Later:** Add `--screenshot` mode to game binary for full integration testing
4. **CI:** Run egui_kittest in CI, optionally run integration screenshots with Xvfb/headless

---

## Key Crate Versions (Compatible with Bevy 0.18)
| Crate | Version | Purpose |
|-------|---------|---------|
| `egui_kittest` | 0.34.0 | egui unit testing + snapshots |
| `kittest` | 0.4.0 | AccessKit-based UI querying |
| `dify` | 0.8.x | Image diff (used by egui_kittest) |
| `image` | 0.25.x | Image loading/comparison |
| `insta` | latest | Text snapshot testing (non-visual) |
