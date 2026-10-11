// Which sound a game event makes (round 3, note 16; docs/round3/sounds.md).
// Pure: no audio here, so node tests cover it. ui/sound.ts plays the cues.
import type { BattleEvent } from "../../src/types";
import type { Decision, FightKind, Outcome, RunView } from "../../src/mvp/contract";
import type { PlayBeat, Step } from "../../src/mvp/trace";
import SOUNDS from "./sounds.json";

/** The cues the game asks for, named by what happens. Each plays one of
 * SOUND_FILES at its own rate (void-board#922: one family, Duamo's, so a cue
 * from the 2022 set now plays a Duamo file instead). */
export const SOUND_KEYS = [
  "block", "buff", "click", "coin", "death", "debuff", "discover", "draw", "freeze", "fuse", "hit", "level-up", "lose", "merge",
  "move-left", "move-right", "reroll", "sell", "shield", "spawn", "start", "status", "status-off", "unfreeze", "win", "wrong", "zap",
] as const;
export type SoundKey = (typeof SOUND_KEYS)[number];

/** The files in mobile/public/sfx/, all from Duamo (see CREDITS.txt there). */
export const SOUND_FILES: readonly string[] = SOUNDS.files;

interface CueSpec {
  file: string;
  rate: number;
  /** Linear gain that lands the cue at its tier's loudness (docs/mission5/sound/level.py writes it). */
  gain: number;
  /** Seconds after which the voice fades out, so a long file makes a short cue. */
  cut?: number;
  /** Which of two cues at once wins: higher plays (see admit). */
  prio: number;
}
const SPEC = SOUNDS.cues as Record<SoundKey, CueSpec>;

/** Linear gain per cue: the measured level, so cues of a tier sound equally
 * loud (UI ticks −24 LUFS, shop −20, battle −19, jingles −16). */
export const GAIN = Object.fromEntries(SOUND_KEYS.map((k) => [k, SPEC[k].gain])) as Record<SoundKey, number>;

/** One sound to play: the file, its playback rate, and its gain (the key's
 * own gain with any extra folded in). `delay`: ms after the cue's moment. */
export interface Cue {
  key: SoundKey;
  file: string;
  rate: number;
  gain: number;
  prio: number;
  cut?: number;
  delay?: number;
}

export const cue = (key: SoundKey, rate = 1, extra = 1, delay?: number): Cue => {
  const s = SPEC[key];
  return { key, file: s.file, rate: Math.round(s.rate * rate * 1000) / 1000, gain: s.gain * extra, prio: s.prio, ...(s.cut ? { cut: s.cut } : {}), ...(delay ? { delay } : {}) };
};

// ---------- the mix ----------

/** Two cues closer than this are one moment: only the stronger plays. */
export const GAP_MS = 90;
/** The same cue never starts twice within this. */
export const KEY_COOLDOWN_MS = 120;
/** At most this many sounds at once; a new one past it cuts the weakest, oldest. */
export const MAX_VOICES = 3;

/** A sound that is playing. */
export interface Voice {
  id: number;
  key: SoundKey;
  prio: number;
  /** ms, on the clock `admit` gets. */
  at: number;
}

/** Whether a cue plays now, and which playing voices it cuts short. Pure:
 * the engine keeps the voices that are still sounding and passes them in.
 *  1. the same cue within KEY_COOLDOWN_MS: dropped;
 *  2. any cue within GAP_MS: the stronger wins (a tie keeps the first);
 *  3. MAX_VOICES at once: the weakest, then oldest, is cut. */
export function admit(voices: readonly Voice[], c: Pick<Cue, "key" | "prio">, now: number): { play: boolean; cut: number[] } {
  const no = { play: false, cut: [] };
  if (voices.some((v) => v.key === c.key && now - v.at < KEY_COOLDOWN_MS)) return no;
  const close = voices.filter((v) => now - v.at < GAP_MS);
  if (close.some((v) => v.prio >= c.prio)) return no;
  const cut = close.map((v) => v.id);
  const left = voices.filter((v) => !cut.includes(v.id)).sort((a, b) => a.prio - b.prio || a.at - b.at);
  while (left.length >= MAX_VOICES) cut.push(left.shift()!.id);
  return { play: true, cut };
}

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
    // The awakening gift (R3-16): a pick chimes, a skip clicks.
    case "gift":
      return cue(d.pick === null ? "click" : "discover");
    default:
      return null;
  }
}

// ---------- battle ----------

/** At most this many sounds a beat; deaths and summons always play. The
 * engine's mix (admit) then thins waves that land closer than GAP_MS. */
export const BEAT_CAP = 3;

/** The chain sound: wave i of a beat plays a little higher, up to 1.3×. */
export const waveRate = (i: number): number => Math.round((1 + 0.05 * Math.min(i, 6)) * 100) / 100;

/** Deaths and summons always play, outside a beat's cap. */
const uncapped = (c: Cue) => c.key === "death" || c.key === "spawn";

/** The one sound a wave makes, by priority: death > summon > damage (block
 * when every hit was absorbed) > heal > +stat > −stat or silence > Shield
 * applied > status applied > status removed (alone) > ChainCapped. */
export function waveSound(step: Step, log: BattleEvent[], waveIndex: number): Cue | null {
  const rate = waveRate(waveIndex);
  const evs = step.eventIds.map((id) => log[id]).filter((e): e is BattleEvent => e !== undefined);
  const has = (t: BattleEvent["type"]) => evs.some((e) => e.type === t);
  if (has("Death")) return cue("death");
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
  if (has("ChainCapped")) return cue("wrong", 1, 0.5);
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
 * slay) plays the win a step higher (win and discover share a file, so the
 * old chime on top would sound twice). */
export function endSound(outcome: Outcome, kind: FightKind, you = true): Cue[] {
  if (outcome === "draw") return [cue("draw")];
  if (!you) return [cue("win")];
  if (outcome === "loss") return [cue("lose")];
  return [cue("win", kind === "crown" ? 1.12 : 1)];
}
