# AOI-S22 Pass 5 — implementation evidence

## Boundary and changed paths

Prototype-only work is contained in `design/aoi-s22-pass-5/`:

- `gallery.html` — self-contained interactive comparison gallery;
- `check.mjs` — deterministic Playwright/source/isolation checker;
- `check-output.txt` — final passing output;
- `capture-manifest.txt` — byte sizes and SHA-256 hashes;
- `captures/` — nine real Chromium captures;
- `evidence.md` — this evidence and tradeoff record;
- `verifier-verdict.md` — fresh independent objective verdict after the sole bounded repair.

No production `src/`, `web/`, CSS, mechanics, balance, package, deployment, release, or repository `DESIGN.md` path was edited. Nothing was staged, committed, or pushed. `.pi-subagents/` is pre-existing runtime output outside this ticket and is excluded by the checker rather than claimed as work.

## One fixture and three composition stances

All stances render the same `FIXTURE` object and DOM, and only composition CSS changes. Fixture: Floor 7, 6g, 3 lives, Mira at Floor 8; six offers; selected offer compared with the owned fused front slot; and the same five-Unit team. The count specimen proves the supported 3, 4, 5, and 6-offer states without substituting content.

1. **A · Market Promenade — panorama-first.** Six equal square offers form a single wide scan, then the broad selected decision, then the team dock. Strength: strongest six-offer comparison and clearest accepted vertical reading order. Risk: longest desktop travel. Anti-reference: not a carousel that hides alternatives.
2. **B · Decision Ledger — comparison-first.** A dense 3×2 offer market feeds a tall, vertical Causal Atlas decision ledger; the team still anchors the bottom. Strength: tight offer-to-consequence reading. Risk: denser offer field. Anti-reference: not a generic equal-panel dashboard.
3. **C · Orbit Bench — roster-first.** A narrow two-column constellation rail stays beside a broad selected proof, then resolves into the owned team anchor. Strength: clearest offer-to-roster relationship. Risk: slower two-column scan at six offers. Anti-reference: not a decorative radial layout or palette variant.

At 390px all three intentionally converge on the accepted transformation: compact Run Spine, horizontal offer scroller with partial-card continuation, selected/comparison decision, horizontal five-Unit team scroller, then bounded state proofs. This responsive convergence preserves shared action/order semantics rather than pretending desktop compositions should merely scale down.

## Carried system and state coverage

- Quiet Constellation organic dark surfaces and guided contrast; no external runtime or generic dashboard chrome.
- Persistent desktop Run Spine and compact phone preface expose Floor/Round/Gold/Lives/next opponent before the shop. Fight remains spatially separate from Buy.
- Portrait Ledger v2 identity → behavior cue → fixed mirrored stats, full and compact forms, amber PWR `#ffc76a`, muted disabled PWR `#e2c980`, red HP `#ff7d72`.
- Hard equal A+B split for Frostbiter + Venomancer, ordered provenance, truthful initial fused meter `0/3` at additive 3 PWR / 17 HP, long Necromancer name, and explicit `Awakened · 3/3` Warden. A later Frostbiter or Venomancer parent copy adds +1 PWR / +2 HP and advances the fusion to `1/3`.
- Causal Atlas Trigger → Selector → Effect follows Split Brief’s stable selected/owned comparison header. The phone causal detail is a bottom sheet with close-first focus, focus trap, Escape, and focus return. Inspector content updates with the selected Unit.
- Alternatives, exact Buy consequence, stakes, and the one current commerce action appear in one decision surface. Buy is selection-dependent: Frostbiter/Venomancer offers route to the owned composite and Summoner/Silencer/Necromancer offers route to owned bases, so they remain enabled and state their +1 PWR / +2 HP and meter consequences. The no-match fused offer on the full 5/5 team disables Buy and keeps the production rejection reason adjacent. Reroll, Save team, and Fight each retain separate jobs. Disabled reroll has its adjacent `needs 1g` reason; empty shop preserves Fight guidance; save error preserves the playable team and puts Retry beside the error.
- Production truth seams are read from `candidates/frostbite-striker.json`, `src/content/stress.ts`, `src/tunables.ts`, `src/run.ts`, `src/battle.ts`, and `web/teams.ts`.

## Deterministic validation

Final command:

```text
node design/aoi-s22-pass-5/check.mjs
exit 0
```

Final signal:

```text
source/truth: PASS exactly three compositions; self-contained; tokens and Frostbiter/stress/tunable/fusion meter 0/buy routing/team/save seams match production
promenade/desktop: PASS geometry, no page/text overflow, order, 3–6 offers, fixture, targets, text, focus, reduced motion
promenade/phone: PASS geometry, no page/text overflow, order, 3–6 offers, fixture, targets, text, focus, reduced motion
ledger/desktop: PASS geometry, no page/text overflow, order, 3–6 offers, fixture, targets, text, focus, reduced motion
ledger/phone: PASS geometry, no page/text overflow, order, 3–6 offers, fixture, targets, text, focus, reduced motion
orbit/desktop: PASS geometry, no page/text overflow, order, 3–6 offers, fixture, targets, text, focus, reduced motion
orbit/phone: PASS geometry, no page/text overflow, order, 3–6 offers, fixture, targets, text, focus, reduced motion
interaction: PASS full-team no-match Buy disabled with adjacent rejection reason; owned base/composite routes enabled with +1/+2 and meter consequences; phone Causal Atlas sheet focus behavior
production isolation: PASS only design/aoi-s22-pass-5/ changed (runtime .pi-subagents ignored); no staged files
captures: PASS 9 real Chromium PNGs (3 desktop + 3 phone + 3 phone inspector), SHA-256 manifest written
```

The checker proves: exact named stance count; fixture fingerprint equality across all stances/viewports; 3–6 offer controls; five owned Units; required exceptional states; production source seams; additive fusion initialized at meter 0; full-team rejection for unmatched offers; enabled owned-base and composite-parent routes with +1 PWR / +2 HP and meter consequences; self-containment; 1440×1000 and 390×844 geometry; no page or unapproved text overflow; shop → selected decision → team order; >=40px enabled targets; >=9px specimen text; 3px keyboard focus; PWR/HP computed colors; reduced-motion media; selection-driven consequence updates; phone sheet focus behavior; capture count/hashes; production isolation; and no staged files.

This sole bounded repair cycle corrected the two fresh S1 objective truth failures only: selection-dependent Buy availability/reasoning on a full team, and the fused 3 PWR / 17 HP meter from `1/3` to `0/3` with later-parent +1/+2 advancement. The final run above passed and regenerated all nine captures plus hashes. The checker itself makes no taste verdict.

## Capture inventory and visual spot inspection

`capture-manifest.txt` is the authoritative hash list. Inventory:

- Desktop: `promenade-desktop.png`, `ledger-desktop.png`, `orbit-desktop.png`.
- 390px phone full-page: `promenade-phone.png`, `ledger-phone.png`, `orbit-phone.png`.
- 390px phone inspector viewport: `promenade-phone-inspector.png`, `ledger-phone-inspector.png`, `orbit-phone-inspector.png`.

Representative spot inspection of the regenerated `promenade-desktop.png`, `ledger-phone.png`, and `orbit-phone-inspector.png` found no overlap or concealed commit consequence; each visibly shows the corrected fused `0/3` label. The selected owned Necromancer base route remains enabled and states +1 PWR / +2 HP with `1/3` → `2/3`; the phone sheet retains comparison context and matching detail. Geometry and interaction checks cover all nine captures. Duplicate phone-inspector hashes for A/B remain expected because responsive compositions intentionally converge.

## Repair validation commands

```text
node design/aoi-s22-pass-5/check.mjs
exit 0

[ -z "$(git diff --cached --name-only)" ]; reject status paths outside design/aoi-s22-pass-5/ and .pi-subagents/; assert DESIGN.md/src/web/server/package diffs empty
exit 0 — isolation: PASS only design/aoi-s22-pass-5/ plus runtime .pi-subagents; production/design contract diff empty; no staged files
```

`check.mjs` is the updated deterministic test. It generated `check-output.txt`, all nine PNGs, and `capture-manifest.txt` in the passing run.

## Residual risks and review envelope

- Portrait motifs remain intentionally schematic; final motif/icon/density polish is deferred by the accepted contract.
- `Save team` follows the ticket’s explicit save requirement and the production local saved-team seam; this prototype does not authorize or implement production persistence behavior.
- Phone composition convergence is a deliberate contract-preserving choice, not evidence that desktop stances are palette variants.
- The checker’s Playwright fallback uses `/root/work/arena-of-ideas/node_modules/playwright` only when this worktree lacks its declared package installation; `ARENA_PLAYWRIGHT` can override it.
- Fresh independent verifier Cass returned PASS with no objective S0/S1 blockers after the sole bounded repair; see `verifier-verdict.md`. Maks’s taste gate remains outstanding.

## Acceptance trace

- **criterion-1 — satisfied:** only `design/aoi-s22-pass-5/` changed; the sole repair corrects dynamic Buy truth and fused meter/consequences without production or contract edits. Final checker proves production isolation and no staged files.
- **criterion-2 — satisfied:** deterministic checker/output, nine regenerated browser captures, regenerated SHA-256 manifest, exact paths/command, and residual risks are preserved here for independent review.
