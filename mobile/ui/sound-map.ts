// Which sound a game event makes (round 3, note 16; docs/round3/sounds.md).
// Pure: no audio here, so node tests cover it. ui/sound.ts plays the cues.
import type { BattleEvent } from "../../src/types";
import type { Decision, FightKind, Outcome, RunView } from "../../src/mvp/contract";
import type { PlayBeat, Step } from "../../src/mvp/trace";

/** The 26 files in mobile/public/sfx/, named by what they do. */
export const SOUND_KEYS = [
  "block", "buff", "click", "coin", "debuff", "discover", "draw", "freeze", "fuse", "hit", "level-up", "lose", "merge",
  "move-left", "move-right", "reroll", "sell", "shield", "spawn", "start", "status", "status-off", "unfreeze", "win", "wrong", "zap",
] as const;
export type SoundKey = (typeof SOUND_KEYS)[number];

/** Linear gain per file, picked to even out loudness (UI ticks ≈ −24 dB,
 * battle ≈ −19, jingles ≈ −16). A starting point to tune by ear. */
export const GAIN: Record<SoundKey, number> = {
  block: 0.86, buff: 1.16, click: 1.06, coin: 1.24, debuff: 0.55, discover: 1.01, draw: 1.27, freeze: 0.37, fuse: 0.84,
  hit: 0.79, "level-up": 1.17, lose: 0.83, merge: 0.7, "move-left": 0.27, "move-right": 0.26, reroll: 0.21, sell: 1.08,
  shield: 0.72, spawn: 0.93, start: 0.71, status: 0.18, "status-off": 0.18, unfreeze: 0.39, win: 1.02, wrong: 0.27, zap: 1.1,
};

/** One sound to play: the file, its playback rate, and its gain (the key's
 * own gain with any extra folded in). `delay`: ms after the cue's moment. */
export interface Cue {
  key: SoundKey;
  rate: number;
  gain: number;
  delay?: number;
}

export const cue = (key: SoundKey, rate = 1, extra = 1, delay?: number): Cue => ({ key, rate, gain: GAIN[key] * extra, ...(delay ? { delay } : {}) });

// ---------- shop ----------

/** The sound of a shop decision, read off the run before and after it (the
 * reply carries no events): a buy that adds a unit clinks, one that merges a
 * copy in merges (line or bench), and the copy that awakens it levels up. A
 * lock freezes and an unlock unfreezes. Fuse, the discovery reveal, refusals
 * and selection have their own calls. */
export function shopSound(d: Decision, before: RunView, after: RunView): Cue | null {
  switch (d.kind) {
    case "buy": {
      // The bench too: a copy merges, and can awaken, there (R3-14).
      const was = new Map([...before.line, ...(before.bench ?? [])].map((u) => [u.uid, u]));
      for (const u of [...after.line, ...(after.bench ?? [])]) {
        const old = was.get(u.uid);
        if (!old) continue;
        if (u.copies > old.copies) return cue(old.form !== "awoken" && u.form === "awoken" ? "level-up" : "merge");
      }
      return cue("coin");
    }
    case "sell":
      return cue("sell");
    case "reroll":
      return cue("reroll");
    case "reorder":
      return d.to === d.from ? null : cue(d.to < d.from ? "move-left" : "move-right");
    case "fuse":
      return cue("fuse");
    case "lock":
      return cue(after.offers[d.slot]?.locked ? "freeze" : "unfreeze");
    default:
      return null;
  }
}

// ---------- battle ----------

/** At most this many sounds a beat; deaths and summons always play. */
export const BEAT_CAP = 5;

/** The chain sound: wave i of a beat plays a little higher, up to 1.3×. */
export const waveRate = (i: number): number => Math.round((1 + 0.05 * Math.min(i, 6)) * 100) / 100;

/** A death: the −stat thud pitched down, so the two differ. */
const DEATH: Cue = cue("debuff", 0.7, 0.8 / GAIN.debuff);
/** Deaths and summons always play, outside a beat's cap. */
const uncapped = (c: Cue) => c === DEATH || c.key === "spawn";

/** The one sound a wave makes, by priority: death > summon > damage (block
 * when every hit was absorbed) > heal > +stat > −stat or silence > Shield
 * applied > status applied > status removed (alone) > ChainCapped. */
export function waveSound(step: Step, log: BattleEvent[], waveIndex: number): Cue | null {
  const rate = waveRate(waveIndex);
  const evs = step.eventIds.map((id) => log[id]).filter((e): e is BattleEvent => e !== undefined);
  const has = (t: BattleEvent["type"]) => evs.some((e) => e.type === t);
  if (has("Death")) return DEATH;
  const summon = evs.find((e) => e.type === "Summon");
  if (summon) return cue("spawn", summon.type === "Summon" && summon.resurrected ? 1.15 : 1);
  const hurts = evs.filter((e): e is Extract<BattleEvent, { type: "Hurt" }> => e.type === "Hurt");
  const landed = hurts.filter((e) => e.amount > 0);
  if (landed.length > 0) {
    const struck = landed.some((e) => e.causedBy !== null && log[e.causedBy]?.type === "Strike");
    return cue(struck ? "hit" : "zap", rate);
  }
  if (hurts.some((e) => (e.absorbed ?? 0) > 0)) return cue("block", rate);
  if (has("Heal")) return cue("buff", 1.2 * rate);
  const stats = evs.filter((e): e is Extract<BattleEvent, { type: "StatChanged" }> => e.type === "StatChanged");
  if (stats.some((e) => e.delta > 0)) return cue("buff", rate);
  if (stats.some((e) => e.delta < 0) || has("Silenced")) return cue("debuff", rate);
  const applied = evs.filter((e): e is Extract<BattleEvent, { type: "StatusApplied" }> => e.type === "StatusApplied");
  if (applied.some((e) => e.status === "Shield")) return cue("shield", rate);
  if (applied.length > 0) return cue("status", rate);
  if (has("StatusRemoved")) return cue("status-off", rate);
  if (has("ChainCapped")) return cue("wrong", 1, 0.4 / GAIN.wrong);
  return null;
}

/** A beat's cues, one per wave (null for a silent wave): at most BEAT_CAP
 * sounds, deaths and summons not counted and never dropped. */
export function beatCues(beat: PlayBeat, log: BattleEvent[]): (Cue | null)[] {
  let counted = 0;
  return beat.waves.map((w, i) => {
    const c = waveSound(w, log, i);
    if (!c) return null;
    if (uncapped(c)) return c;
    if (counted >= BEAT_CAP) return null;
    counted++;
    return c;
  });
}

/** The end card's sound. `you` is false for a battle watched without a side
 * (a playoff game, the champion's): it plays win, or draw. A Crown win (a
 * slay) adds the discovery chime 0.5 s later. */
export function endSound(outcome: Outcome, kind: FightKind, you = true): Cue[] {
  if (outcome === "draw") return [cue("draw")];
  if (!you) return [cue("win")];
  if (outcome === "loss") return [cue("lose")];
  return kind === "crown" ? [cue("win"), cue("discover", 1, 1, 500)] : [cue("win")];
}
