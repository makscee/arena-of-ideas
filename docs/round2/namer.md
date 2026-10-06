# Fusion names: a bigger model and one-word names

Short answer: switch the namer to **Qwen3-4B-Instruct-2507 (4-bit)** and use a **one-word prompt** (below). Names get clearly better and tied to the two fighters ("Thronewitch" for King + Hag, "Petaldrain" for Rose + Injector, "Shockbite" for Taser + Plague Rat), 99% come out as one word, and it fits on m1 next to everything else. A 7-8B model does not fit on m1 today with 4 GB left free.

## What I ran

- 40 ordered pairs from the real unit list (seeded random; Doctor+Bloodthinner, King+Hag, Rose+Injector, ...), the exact chat messages from `namerMessages()`, at the server's settings (temperature 0.8, max 12 tokens).
- Three prompts:
  - **real**: today's `NAMER_SYSTEM` + `NAMER_EXAMPLES`.
  - **one**: "a single word, at most 14 letters", with one-word examples.
  - **one2**: "one word, made of one or two plain English words joined, easy to read aloud, at most 14 letters", with the same one-word examples. This one is the pick.
- Scored with the real filter (`cleanModelName` imported from the checkout with tsx), plus the clash rule against base unit names. "Odd" is a rough check that I then read by eye. It flags CamelCase kept as one word, another unit's name inside, more than 14 letters, a plain dictionary word, or a word that doesn't split into English words.
- Each model ran once in its own one-off mlx_lm script on m1. A watcher checked the kernel's free-memory level every 2 s and would have killed the run below 25% (about 4 GB). It never fired.
- Latency is cold: the full prompt for every name. The live mlx_lm.server caches the shared system and example prefix, so live asks are faster.

## Results

| Model, prompt | Asks | Valid after filter | One word | Odd | Starts with "S" | Latency (median) | Peak memory |
|---|---|---|---|---|---|---|---|
| Qwen2.5-1.5B (live), real | 80 | 99% | 24% | 16 | 18% | 0.43 s | 1.1 GB |
| Qwen2.5-1.5B, one | 80 | 96% | 99% | 38 | 23% | 0.43 s | 1.1 GB |
| **Qwen3-4B-2507, real** | 80 | 100% | 46% | 10 | 52% | 1.43 s | 2.6 GB |
| **Qwen3-4B-2507, one** | 80 | 99% | 100% | 18 | 50% | 1.42 s | 2.6 GB |
| **Qwen3-4B-2507, one2** | 80 | 100% | 99% | 12 | 48% | 1.63 s | 2.6 GB |
| gemma-3-4b-it, real | 40 | 100% | 10% | 2 | 33% | 1.25 s | 2.8 GB |
| gemma-3-4b-it, one2 | 40 | 100% | 100% | 3 | 35% | 1.25 s | 2.8 GB |

The filter is not the problem: even the 1.5B passes 96-99%. The bigger model buys quality, not validity.

### What the names look like

- **1.5B, real**: mostly two or three words, generic, sometimes clumsy. Examples: Merciless Therapist, Sacrifice Surgeoner, Plague Archimedes, Plagueo-Manipulator, SoulWound Restorer, Rat Lady, Knave of Fruit.
- **1.5B, one-word**: single words, but many are broken. Examples: Aerialidzerzer, Floridirecptive, Siphoniestroider, Blesseddescender, Corruptionril, Kingess, Primero, Plutonium. Some glue a unit name in (Sniperhealer, Powerpathologist, Healdivinity) or come out CamelCase (BalanceWarrior, UnyieldingPhoenix). The filter passes all of these.
- **Qwen3-4B, one2** (the pick): Thronewitch (King+Hag), Petaldrain (Rose+Injector), Shockbite (Taser+Plague Rat), Crownreaper (King+Harvest), Blazefang, Sparkthorn, Rootwarden, Flameguard, Furycleave, Snitchwing (Director+Gnat), Sewermend (Almsgiver+Rot), Stoutspire, Holyflame, Sickbloom.
  - Odd ones (12 of 80): Sanguicure, Sanguiscope, Sangufate, Sanguine (a plain word), Sickwardent, Noctblit, Noctscum, Harguard (×2), Zapshadow, Stanchex, Furncrown. One answer had two words ("Jolt Healer").
- **Qwen3-4B, one**: coins Latin-sounding words. Some are fine (Pyralith, Sanguisire, Pyrethrone), others aren't (Zarptarn, Yieldthron, Balancert, Sicknessedged, Stoiket). The filter refused one: "Smutgiver". The "plain English words joined" line in one2 fixes most of this.
- **Qwen3-4B, real**: half are one word and half are two (Bloomsting, Hushblade, Crownflare; Flame Seraph, Decay Ward). Odd ones: Stillrupt, Nocture, Glimspector, plus "Spike Bloom" and "Spike Healer" (Spike is a unit).
- **gemma-3-4b**: the cleanest spelling, but the same few words again and again.
  - In one2, 13 of 40 names contain "Shadow" (Shadowguard, Shadowmend, Shadowbane, Shadowmaw, Shadowveil, Shadowfall, Shadowforge, ...).
  - In real, 10 of 40 end in "Warden" and 8 contain "Shadow".
  - It gave "Crimsonwise" to two different pairs. With the rule that names are unique, the pool of names would fill with Shadow-names fast. Names are less tied to the fighters (King + Hag gives "Shadowbane").

### Two things to fix along with the switch

1. **The "S" habit.** Qwen3-4B starts about half its names with S (Stout..., Sick..., Steel..., Shield..., Sangu...). Over hundreds of fusions that looks samey. I didn't test a fix. In the build attempt, try temperature 1.0 first, or have the server add "Start with the letter X" with a random X. Measure the same way.
2. **Filter gaps the bench showed** (all small):
   - "Noctscum" passed: "scum" is on the ORDINARY list, so the "cum" stem is skipped.
   - CamelCase single words pass as one word ("BalanceWarrior"). Split them or refuse them.
   - Names that contain another unit's name pass ("Blood Guardian", "Harvestguard", "Joltmedic"). That reads confusingly in game. Refusing a whole unit name of 5+ letters inside the answer would catch it.

## Memory and load on m1

- **At the start:**
  - 16 GB machine.
  - Docker's VM shows about 7.3 GB, the live namer 0.9 GB, Chrome and Zen about 2 GB.
  - Kernel free level 46% (about 7.4 GB), swap 1.1 GB used.
  - mediaanalysisd was busy, load about 3.4.
- **During the runs** (the live 1.5B stayed loaded the whole time, so these numbers include both models):
  - 1.5B copy: lowest level 29%, median 35%.
  - Qwen3-4B: lowest 25%, median 35%. The 25% was only while an hf download held 1.4 GB at the same time. Load peaked at 8.
  - Qwen3-4B, one2 run: lowest 35%. gemma: lowest 36%. Load stayed under 6.
- **If the 4B replaces the 1.5B**, the namer costs about 1.5 GB more than today (2.6 GB peak vs 1.1 GB). That leaves m1 at about 40% free when idle, well above 4 GB.
- **7-8B models (Qwen2.5-7B, Qwen3-8B, Llama-3.1-8B):** not run. At 4-bit they need about 4.5-5 GB, which would take m1 to about 20-25% free, under the 4 GB line. To make room, Docker Desktop's VM on m1 would need less memory, or the namer would need another host. I don't recommend either just for names: the 4B is already a big step up.
- **Side effect:** swap on m1 grew from 1.1 GB to 2.9 GB during the runs and hasn't come back down yet. At the end, the free level was back to 55% and load was 3.8.

## Recommendation and what the server needs

1. **Model (live change, needs Maks's go):** in `~/Library/LaunchAgents/ru.makscee.arena-namer.plist` on m1, change `--model` to `mlx-community/Qwen3-4B-Instruct-2507-4bit`, then reload the agent.
   - It is already in m1's HF cache (2.3 GB), so it starts in about 2 s.
   - The game server already waits through the namer's 503s while it loads.
   - It is a non-thinking model, so nothing else changes.
2. **Prompt** (`server/src/mvp/fusions.ts`):

   ```
   NAMER_SYSTEM = "You name creatures in a fantasy auto-battler. Two fighters merge into one new creature. Invent a fresh, evocative name for it: one word, made of one or two plain English words joined, easy to read aloud, at most 14 letters. Do not just join or repeat the two fighters' names, use no emoji, no quotes, no explanation, and never a name from an existing game, film, book or comic."
   NAMER_EXAMPLES = Knight+Wolf → Fangwarden, Spark+Healer → Stormmender, Golem+Raven → Gravewing, Thief+Monk → Almscutter
   ```

   - None of these example words is a unit name. The bench script checks it, the same way the existing test does.
   - `fusions.test.ts` checks the exact strings ("Fang Paladin"), so update it.
3. **Keep:** `max_tokens` 12, the 15 s timeout, 3 tries, the 1-3 word filter (two-word answers stay possible but are rare).
   - Optional: in `drainFusionNames`, prefer a one-word answer within the 3 tries and keep a two-word answer only as the last resort.
4. **Timing:**
   - About 1.4-1.6 s per name cold (vs 0.4 s), less with the server's prefix cache.
   - Humans never wait. A bot waiting on one pair waits about 1.5 s instead of 0.4 s.
   - The queue drains at about 40 names a minute instead of about 140. That's fine at today's player count.
5. **Also do:** the "S" habit and the filter gaps above, measured with the same 40 pairs.

## Cleanup

- Deleted gemma-3-4b-it-4bit (3.4 GB) and its partial downloads. I never downloaded Llama-3.2-3B because I stopped that download early to save memory. Qwen3-4B-Instruct-2507-4bit (2.3 GB) stays in m1's HF cache for the switch.
- Every process I started (bench scripts, watchers, hf downloads) is gone, and `~/namer-bench` on m1 is removed.
- The live namer was never touched: same PID 95188, still on Qwen2.5-1.5B, `/v1/models` gives 200, and a chat ask answered at the end. arena-mvp is still running.

## Files (scratchpad)

`v2/namer/`:
- `pairs.mts`: builds the pairs and the one-word prompt from the checkout.
- `pairs.json`: the 40 pairs with all three prompts.
- `bench.py`: the m1 run.
- `watch.sh`: the memory guard.
- `score.mts`: the real filter plus the odd-name check.
- `out-*.jsonl`: every raw answer.
- `sum-*.json`, `watch-*.log`: latency, memory and load.
