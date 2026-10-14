# Keyboard only (#1064, mission #1043)

Every screen and action works without a mouse on a desktop (1024px and wider),
and each key is shown where it is used. `npm run mvp:keyboard` (in `mvp:check`)
plays it in EN and RU and fails on any pointer click.

## The rules

- **Tab** reaches every control, in reading order. A card (line, bench, offer,
  gift, Codex unit, champion row) is `actionable()` (`mobile/ui/dom.ts`):
  `tabindex=0`, `role=button`, **Enter** clicks it. The focus ring is
  `:focus-visible` in `--action` (all three palettes), 3px, in `style.css`.
- **Space** is the screen's key (Fight in the shop), but on a button reached by
  Tab it presses that button. After a mouse click the focus is not visible, so
  Space is still Fight.
- **Esc** closes the top sheet, then steps back (as before). A sheet takes the
  keyboard when it opens (its primary button), keeps Tab inside, and gives the
  keyboard back to where it was when it closes.
- A redraw keeps the keyboard where it was (`show()`, `keepFocus`,
  `rememberFocus`): the shop redraws after every decision.
- A key never reaches a screen from a text field, and never with Ctrl/Cmd/Alt.
- A key is on its button as a keycap (`withKey`), on desktop only.

## Keys by screen

| Screen | Keys |
|---|---|
| Sign-in, invite | the name field has the keyboard; Enter signs in |
| Home | Enter play · C Codex · F Forge · D Dictionary · S Stats · I ideas · G settings · M sound |
| Shop | Tab to a unit or offer (the inspector reads it) · Enter select · 1–7 buy · R reroll · L lock · Space fight · ← → move · B bench · S sell · F fuse · Esc step back / menu |
| Fuse, the forge (in the inspector) | the primary button has the keyboard: Enter fuses/claims · 1–3 look at sealed design 1–3 · Esc cancels |
| Gift | 1–3 pick that card · Esc sets it aside |
| Codex | 1–6 tabs · / search · Enter on a unit reads it · Esc clears the reader, then leaves |
| Forge / Dictionary | 1–3 tabs (Units, Feed, Dictionary) · ← → the unit · Esc leaves |
| Claim moment | 1–3 pick a sealed card · Enter claim · Esc/Enter back |
| Battle | Space pause · ← → beat · R replay · S skip chain · Esc menu · Tab to Skip/Why/Log |
| End card, run over | Enter goes on · Esc (run over: Home) |
