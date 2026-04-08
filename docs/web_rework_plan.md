# Web-First Client Rework

## Context

Replacing the Bevy/egui desktop client with a **Dioxus 0.6** (Rust web framework, React-like API) browser client. Primary motivation: **AI-assisted development velocity** — HTML/CSS/component trees are the most LLM-understood UI paradigm.

**Architecture**: Dioxus CSR + CSS for layout/controls/cards, `<canvas>` for unit visuals, Rhai scripts compiled to WASM for painter execution. Served alongside SpacetimeDB server. Full replacement of Bevy client.

---

## Completed (Phase 1: Battle Viewer MVP)

- [x] `/web` crate added to workspace with Dioxus 0.6 CSR
- [x] Shared types reused directly (`BattleResult`, `BattleAction`, `Unit`, etc.)
- [x] Test battle data (3v3, ~30 actions across 4 turns)
- [x] `BattlePlaybackState` with step forward/back, auto-play, speed control
- [x] Battle page layout: enemy team row, duel area (VS), ally team row, action log, controls
- [x] Unit cards with name, HP/PWR stats, HP bar, alive/dead/active states
- [x] Action log with color-coded cards (damage/heal/death/ability/stat/fatigue/spawn)
- [x] Playback controls: Reset, Step Back, Step, Play/Pause, End, speed toggle
- [x] Responsive CSS with mobile breakpoint
- [x] Builds to WASM via `dx serve`, renders in browser

---

## Phase 2: Canvas Unit Visuals

- [ ] Port `PainterAction` enum (Circle, Rectangle, Hollow, Solid, Translate, Color, Alpha, Paint)
- [ ] Canvas 2D renderer: execute PainterAction list on `<canvas>` via web-sys
- [ ] Integrate Rhai engine (compiled to WASM) to execute `painter_script` per unit
- [ ] `requestAnimationFrame` loop passing `t` (time) to Rhai scripts for animation
- [ ] Default painter for units without scripts (colored circle + name initial)
- [ ] Duel area gets larger canvases for active fighters

## Phase 3: Polish & Real Data

- [ ] SpacetimeDB SDK integration (JS SDK via wasm-bindgen, or native Rust SDK)
- [ ] Load real battles from server instead of test data
- [ ] Router setup (Dioxus Router) — `/battle/:id` route
- [ ] Loading states, error handling for missing battles
- [ ] Ability tooltip on hover (name, target type, description)
- [ ] Turn markers in action log (group actions by turn number)
- [ ] Auto-scroll action log to latest action
- [ ] Winner celebration animation

## Phase 4: Shop & Collection

- [ ] Shop page: browse/buy units, reroll
- [ ] Collection page: view owned units and abilities
- [ ] Team builder: drag-drop unit slots
- [ ] Fusion UI: select 3 copies, choose fusion partner
- [ ] Feeding UI: donate abilities to fused unit

## Phase 5: Game Flow

- [ ] Login/auth (SpacetimeDB identity)
- [ ] Match state machine: Shop -> Battle -> floor advancement
- [ ] Floor progression UI
- [ ] Boss battle indicator
- [ ] Game over / victory screen

## Phase 6: Create & AI Integration

- [ ] Ability breeding UI (pick 2 parents + prompt -> AI generates)
- [ ] Unit creation UI (trigger/abilities/tier -> AI generates name + painter)
- [ ] Rate limiting display (breeds/generations remaining today)
- [ ] Evolution tree viewer for abilities

## Phase 7: Bevy Client Removal

- [ ] Feature parity verified
- [ ] Remove `/client` crate from workspace
- [ ] Update CI/CD to build web only
- [ ] Deploy WASM bundle alongside SpacetimeDB

---

## Key Files

```
web/
  Cargo.toml              # Dioxus 0.6 + shared dep
  Dioxus.toml             # dx CLI config
  style/main.css          # All styles
  src/
    main.rs               # App mount
    test_data.rs           # Hardcoded 3v3 battle
    state/mod.rs           # BattlePlaybackState, UnitSnapshot
    components/
      mod.rs
      battle_page.rs       # Main layout (grid)
      unit_card.rs         # Unit card with stats + canvas
      action_log.rs        # Scrollable action cards
      controls.rs          # Playback controls + auto-play coroutine
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| Web framework | Dioxus 0.6 (CSR, React-like API) |
| Build tool | dx CLI (Dioxus CLI) |
| Styling | Plain CSS (flexbox/grid) |
| Unit visuals | canvas 2D context via web-sys |
| Painter scripts | Rhai compiled to WASM |
| Battle logic | /shared crate (direct dependency) |
| Backend | SpacetimeDB |
| Testing | Playwright (real DOM) |
