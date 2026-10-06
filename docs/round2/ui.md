# Arena MVP v2: UI, what's on screen, desktop, compact cards

Scout of build d8650aa3 (branch mission-574-mvp), run locally (server on :8901, throwaway db, client built into the scratchpad; the live m1 instance was never touched). Server and browser are stopped.

- Screenshots: `v2/ui-shots/phone-NN-*.png` (360x640, 32 shots) and `v2/ui-shots/desk-NN-*.png` (1440x900, 32 shots). Same walk at both sizes: name, home, dev, rules, stats, shop R1, offer sheet, legend, battle + trace, result, late shop (full line), awoken unit sheet, awaken preview, fuse pick, fusion preview, fused, late battle, battle end view, why-I-lost, crown shop, champion pin, crown battle, crown result, run over, stats tabs.
- Capture script: `v2/capture.mjs` (run from the checkout with `node --import tsx/esm`, needs a local server on :8901).
- Mockups (static HTML, inline CSS, plus PNG): `v2/mockups/a-desktop-shop`, `b-phone-compact`, `b-card-anatomy`, `e-navigation` (generator `gen.py`, shooter `shoot.mjs`).

## The short version

1. **Desktop today is the phone page centered at 480px.** At 1440x900, two thirds of the screen is empty and cards are ~86px wide (desk-13, desk-21). A real desktop layout is a small change: a wide board on the left and an **inspector panel** on the right that shows whatever card you hover or select. No overlays on desktop.
2. **The ability text is never on screen in the shop.** You have to tap every card to see what it does, while the win/pick rate is on every card. That's the wrong way round. Cards should show a **trigger icon** (what wakes the unit) and status icons. The full text goes in the sheet or inspector, with keywords lit up and explained.
3. **Rates are everywhere**: on home champion cards, line, offers, offer sheet, unit sheet, battle cards, result cards and stats. Move them to one dim line at the bottom of the sheet/inspector, plus the Stats page and the Codex. Take them off all cards.
4. **Both forms always show** in the sheet (Sleeping and Awoken stacked, offer sheet and unit sheet alike). Show the active one only, with a "See Awoken" button that swaps in the other form and underlines what changes.
5. **Navigation gaps:** no menu during a run (no way to quit to the title screen or abandon), no Codex, and Home doubles as the title screen. Add a ☰ in the run HUD (Esc on desktop) with Resume / Codex / Rules / Title menu / Abandon run, and make Home the title menu with Continue / New run / Codex / Stats / Rules.

## Screen by screen

Legend: **Always** = on screen without a tap. **Key** = the one thing that matters right then. **Behind a tap** = what should move off the main view. **Rates** = where W/P shows today.

### Home (phone-02, desk-02)
- Always: title, your name, champion panel (5 cards with W/P), slayers sentence, a hint, a 5-number records panel (Rating, Runs, Slays, Days👑, Playoff W), a "Dev" fold, Rules / Stats / Play.
- Key: **Play/Continue**, and the champion team you're trying to beat.
- Behind a tap: the records panel (four of the five are 0 for nearly every player; keep Rating next to your name, the rest go to Stats); the Dev fold (dev builds only, or inside Settings); the hint ("Tap a card to read it…"): show once, then drop it.
- Rates: on 3 of 5 champion cards ("—" on the fused ones, which is noise).
- Note: "6 slayers today. At 04:00 Moscow their teams play a round-robin" is useful but long; "6 slayers · crown at 04:00 MSK" says the same.

### Shop (phone-06, phone-13, desk-13)
- Always: HUD (round, hearts, gold), "Next: @opponent", champion pin (name + cut-off emoji), "Your line · front first" with **?** and **Rules**, 5 line cards, hint, "Shop · tap to read and buy", 5 offers, Reroll / Fight.
- Key: **your gold, your line, the offers, and what each unit does.** Today the last one needs a tap per card.
- Behind a tap: Rules and ? (move into ☰ and the Codex Keywords tab, which frees the line header); the champion pin (keep a tiny 👑 button; it shows the full team on tap, as it does today); "tap to read and buy" (obvious after the first time).
- Rates: on every line card and every offer, 10 per screen.
- Space: on the phone, ~260px under the offers sit empty (phone-13). On desktop it's ~480px plus both side margins. So compact cards with **more offers** fit easily (see b).
- Crown shop (phone-24): "Crown vs @bot-Bram the First 🤖" and the pin say the same name twice, side by side. Drop the pin in the Crown.

### Offer sheet / unit sheet (phone-07, phone-15)
- Always (sheet): name, PWR/HP (+ tier), **Sleeping text, Awoken text**, Win/Pick line, then "Joins your line" with a preview card that repeats the same numbers, then Close / Buy.
- Key: **what it does now** and **what buying changes** (Joins / Merges ×2 / Awakens!).
- Behind a tap: the other form (c), and the preview card when it only repeats the header. Keep it for "Awakens!" and merges, where the numbers change.
- Keywords like "Strength", "Shield", "pwr" are plain text with no explanation (Maks's point). Light them up with an icon and make them tappable (hover on desktop).

### Selected unit in the line (phone-14)
- The action row ◀ ▶ INFO SELL +1 (FUSE when 2 awoken) appears under the line, pushing the hint and shop down. That's fine on the phone. On desktop the inspector holds these as buttons + keys.

### Awaken preview / fuse pick / fusion preview (phone-16, 18, 19)
- Fusion preview shows the **name "Wiach" and "by you" before you confirm** (phone-19). Per Maks: for an undiscovered pair show "??? · new fusion" and reveal the name after Fuse. Fusion order matters (When of the 1st, Who of the 2nd), and the only way to flip it is Cancel and tap in the other order. Add a **⇄ Swap order** button in the preview (another card in this batch covers the mechanics).
- The preview lists FUSED text plus both parts' Sleeping and Awoken texts (5 blocks). Show the fused text only, with "Made from: Wire (When) + Coach (Who)" and the parts behind a tap.

### Battle (phone-09, phone-21, desk-21, phone-26)
- Always: "Round 5 · vs @bot · T#" header, "THEM" label, enemy cards (with W/P and status text like "Shield 4 · Vitality 3"), a caption box, your cards (with W/P), "YOU · front first", 3 past captions, ◀ ❚❚ ▶ 1× Skip.
- Key: **who acts on whom, and the numbers changing.**
- Behind a tap: rates on battle cards (pure noise mid-fight); the past-captions list (fine on desktop as a side log, too much on the phone); the "—" turn placeholder.
- Readability notes (for the battle-UX scout): status text wraps to 2 lines under the stats ("Shield 5 · / Vitality 3"), so icons with numbers fit in one line; a fan-out ("Squat → Shield ×2 on" 4 targets) plays as 4 separate steps with identical captions (desk-21), which could be grouped into one step; on desktop the two lines could face each other left/right with the fronts meeting in the middle.
- End view (phone-22): "They win" in the caption box, "No one standing.", DEFEAT, why-I-lost, Continue. There is no **Replay** here, only on the result screen after Continue or Skip. Maks asked for replay when it's done: put Replay next to Continue.

### Result / why I lost (phone-23)
- Always: HUD with "2W 3L", DEFEAT, "vs @bot · 8 turns", "−1 heart", why-I-lost rows, YOU line, THEM line (both with W/P), Replay / Next round.
- Key: **won/lost, hearts left, why.** This screen is decent.
- Behind a tap: both full lines (the battle just showed them; one "Teams" fold is enough), the rates.

### Run over (phone-28)
- Fine: why it ended, record, rating change, last line, Home. Rates on the last line can go.

### Stats (phone-30)
- Records, then Units / Champions / Fusions tabs. The Units tab is a **Win/Pick/Runs table**. That's the right home for rates. In the new navigation, Stats keeps records and history, and the Codex becomes the place to browse units, with rates as its hint.

### Dev (phone-03)
- "Dev" fold with End day now on Home for everyone. Hide it outside dev builds, or move it to ☰ → Dev.

### Where W/P shows today (all of it)
`ui/unit-stats.ts unitStatsLine()` is called from `card()` for every non-fused card and from `unitSheet()`. That puts W/P on: home champion cards, shop line, shop offers, offer-sheet preview card, unit sheet, battle cards (both sides), result cards (both sides), run-over line, champion pin sheet, stats table. The "?" legend spends a row explaining "W59% P9%".

## Proposals

### (a) Desktop layout. Mockup: `mockups/a-desktop-shop.png`
- One client, two layouts by width: `@media (min-width: 1024px)` switches to desktop, below it the phone layout stays as is. No separate app.
- **Top bar** (56px): ☰ menu, round, hearts, gold on the left; next opponent and the 👑 champion chip on the right.
- **Board** (left, fluid): your line, a one-line hint, the shop row, and a keyboard hint line.
- **Inspector** (right, 380px): whatever is hovered, else what's selected. It shows name, stats, form state, the active form's text with lit keywords, "See Awoken", actions (move, fuse, sell / buy) and, at the very bottom, one dim rates line. On desktop it replaces the overlay sheets entirely.
- **Mouse:** hover = read (inspector), click = select, double-click an offer = buy, **drag to reorder** the line (and drag an offer onto the line = buy). **Keyboard:** 1–7 buy offer, R reroll, Space fight, ← → move the selected unit, F fuse, S sell, Esc menu. In battle: Space pause, ← → step, 2 speed, Enter skip/continue.
- **Tooltips:** every lit keyword (Shield, Vitality, Strength, Freeze, triggers) shows its rule on hover on desktop, and on tap in the phone sheet.
- Battle on desktop: same board width, lines facing left/right, the caption + chain as a right-hand log in the inspector's place (details for the battle scout).
- Code: `#app { max-width: 480px }` is the only reason desktop is a phone column. The desktop layout is a CSS grid on top of the same DOM plus an inspector element that `unitSheet()` renders into instead of `overlay()`.

### (b) Compact cards. Mockups: `mockups/b-card-anatomy.png`, `mockups/b-phone-compact.png`
- **Phone 64x84** (today ~61x124 with rates). 5 per row at 360px with 4px gaps (5x64 + 4x4 = 336 = 360 − 2x12). Line (84) + two offer rows (172) + HUD + buttons fit 640px with room to spare, so **7 offers** (5 + 2 in a second row) fit without scrolling (phone mockup, left).
- **Desktop 132x172.** 7 per row in a ~1060px board, line and shop in one view with no scroll (desktop mockup).
- **On the card:** trigger icon top-left (what wakes it), emoji, name (one line, ellipsis), PWR/HP, and **one footer slot**: copies pips ●●○, or AWOKEN, or FUSED, or the price (+ "＋" when owned), or in battle the status icons with stacks (🛡4 ♥+2). Offers get tier dots top-right.
- **Moved to the sheet/inspector:** full ability text (keywords lit), exact copies, what a buy changes, the other form, fusion parts and "discovered by", rates.
- Taps stay ≥44px: a 64x84 card is a tap target; the e2e's 44px rule holds.
- Icons: game-icons.net (CC BY 3.0, needs credit on an About/Credits line). Candidates to pick there: a shield icon for Shield, a heart-plus for Vitality, a flexed-arm/fist for Strength, an ice/snowflake for Freeze, crossed swords or a flag for Battle start, a skull for On death, a burst/impact for When hurt, a cycle arrow for Turn start/end, an up arrow for "gains PWR". The mockups use simple stand-in shapes in the same colors. Pull the SVGs into the repo (`mobile/icons/`), inline, no CDN.

### (c) Active form only. Mockup: `mockups/b-phone-compact.png` (middle and right)
- The sheet shows **only the form the unit has now** (Sleeping, Awoken or Fused). There's no label when it's the only one.
- A sleeping unit gets **"▸ Awoken in 1 copy"** (R2-17: was "See Awoken (1 more copy)", two lines in the 1024px inspector). It swaps the text box to the Awoken form with a gold border and "AWOKEN · after copy 3", underlines what differs, and offers "◂ Back to Sleeping". An awoken unit has no button: it's final until fused.
- The card itself already shows only the active state (pips vs AWOKEN). Keep that.
- Offer sheet: same rule. When the buy would awaken, show "Awakens!" with the Awoken text as the main block (the preview *is* the active form after buying).
- Fused unit: the fused text only, with "Made from Wire + Coach ▸" opening the parts.
- Code: `sheetForms()` in `ui/card.ts` returns both forms. It should return the active one plus a toggle.

### (d) Win/pick rates as a subtle hint
- **Off every card** (shop, line, battle, result, home champion, run over). That frees a row on each card and is half of why cards can be compact.
- **One dim line at the bottom of the sheet/inspector:** "wins 52% · picked 6%" in 11–12px grey, with an ⓘ that explains it (the text from today's legend).
- **Stats → Units table and the Codex** keep them as columns/sort keys. That's where someone who wants the numbers looks.
- The "?" legend loses its W/P row.

### (e) Abandon, title menu, Codex. Mockup: `mockups/e-navigation.png`
- **Title menu = Home**, reshaped: name + rating top right, the champion line (compact cards), one big **Continue run · R8 ♥♥♥** (or **Play** when no run), then New run (only when no run is active, or "Abandon & new" with a confirm), **Codex**, Stats, Rules. Dev and the records panel leave Home.
- **☰ in the run HUD** (Esc on desktop) opens: Resume, Codex, Rules, **Title menu** (the run stays saved server-side as today, and Continue brings you back), **Abandon run…** (red, with one confirm). Abandon needs a server decision (`{kind:"abandon"}` ending the run with `endedBy:"abandoned"`). How abandon counts toward rating is for the rating-formula scout. My suggestion: rate it as a run that ended at that round.
- **Codex**: full-screen page from the title menu and from ☰. Tabs: **Units** (all 81, a compact-card grid, filter by tier and trigger icon, search name/text), **Fusions** (discovered pairs, "?" for the ones nobody has found, credit on each), **Keywords** (every status and trigger with icon and rule: the place Shield/Vitality are explained in full). Tapping a unit opens the same sheet (active form + See Awoken) plus its rates hint. The content pack already has every unit (`GET /content`, 81 units, tiers 1–4), so the Units tab needs no server work.
- Phone: Codex and Stats are pages with a ✕/Back, as Stats is today. Desktop: same pages, cards in a wide grid, the inspector on the right.

## Smaller things seen on the way
- "Next: @bot-Dov 🤖" plus a champion pin cut off at "@bot-Bram the Fi…" both sit in the top row of every shop round. One line is enough: "vs @bot-Dov" in the HUD, 👑 as an icon button.
- Long fused names fit by shrinking the font (fitText) on 61px cards ("Distrpper", "Venomancer" at ~11px). With one-line ellipsis names on compact cards, the full name lives in the sheet.
- "?" opens "? CARDS" in a filled style until first seen. It works, but the Codex Keywords tab would replace it.
- Shop rules text says "5 offers a shop" (`MVP_RULES.offers: 5`). Growing offers per round (Maks's ask) is a rules change for another scout. The compact card sizes above already allow 7 on the phone and 7 in one desktop row.
