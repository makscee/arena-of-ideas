# AOI-S22 Pass 4 — inspector/reference evidence

## Product boundary

Prototype-only gallery; no production `src/`, `web/`, mechanics, balance, registry, package, deployment, or release files changed. Repository `DESIGN.md` was absent at dispatch (`fde1713d`), so the accepted contract was recovered from AOI-78 evidence and its cited AOI-74–77 sources. The Void Vault writer service is inactive on Tower, so this bounded repo evidence is used rather than leaving an unsynchronized canonical-vault edit.

## Three compositions

- **A · Field Lens — browse-first:** a card-anchored popover preserves the alternatives field. Fastest scan; least room for deep ancestry.
- **B · Split Brief — decision-first:** a persistent right-side brief stabilizes comparison while alternatives remain visible. Clearest repeat comparisons; spends desktop width.
- **C · Causal Atlas — explanation-first:** vertical alternatives feed a broad causal/reference canvas. Best deep comprehension; slower quick stat scan.

**Recommendation:** Causal Atlas hierarchy with Split Brief’s stable comparison header. Maks remains the selection/hybridization/rejection gate.

All three preserve Run Spine, Quiet Constellation, Portrait Ledger v2, equal Frostbiter + Venomancer provenance, PWR `#ffc76a` / disabled `#e2c980`, and HP `#ff7d72`. Each contains selected detail and Warden comparison; Trigger → Selector → Effect; Frostbite Ability, Shield Status, and Imp Summon references; Base/Awakened/Fused A+B states; long, empty, disabled, and error states. At 390px each becomes the same bounded bottom-sheet pattern.

## Rendered inspection

Nine captures in `captures/`: desktop, 390px sheet-top, and 390px scrolled-reference state for every stance. `capture-manifest.txt` records byte sizes and SHA-256 identities.

Manual inspection found and repaired: (1) mobile sheet initially obscured every alternative, fixed by compacting Run Spine and bounding sheet height; (2) capture focus traversal left pages scrolled and hid composition tops, fixed by deterministic scroll reset plus separate sheet states; (3) Field Lens detail overlapped the tradeoff footer, fixed by stage height; (4) Causal Atlas stats escaped their narrow rail due a stale selector, fixed with an explicit vertical stat rail and a new containment assertion. Fresh verification then found one S1 truthful-copy defect: Warden incorrectly gained Strength as an Effect and Shield consumed one stack per hit. The single allowed repair now uses Warden’s inert Strike recipe, treats Strength ×2 as a starting passive status, uses Shield’s exact absorb/consume semantics, asserts those strings, and leaves ≥80px of an alternative visible above the phone sheet. Final inspection of all nine regenerated PNGs found no visible clipping, overlap, unintended truncation, or concealed alternative.

## Deterministic check

```text
node design/aoi-s22-pass-4/check.mjs
exit 0
```

The check verifies source truth against `candidates/frostbite-striker.json`, `src/content/stress.ts`, and additive fusion in `src/run.ts`; exactly three stances; self-containment; all required states/references; 1440px and 390px rendered geometry; visible alternatives/consequences; no page or unapproved text overflow; stat containment; ≥40px targets; ≥10px text; computed PWR/HP colors; 3px focus; reduced motion; bottom-sheet geometry; keyboard selection with matching detail/consequence; close-first and trapped focus order; Escape/focus return; comparison and reference tabs; production isolation; and nine captures. Machine checks explicitly do not judge taste.
