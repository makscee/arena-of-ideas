// Earning ideas (mission 2, M2-3, makscee/void-board#787): 1 idea per
// MvpRules.ideaEveryRuns finished runs, held up to ideaHold. Counted from what
// the store already knows, the finished runs on the rating row (Rating.runs:
// every ending a rating counts, given up included), plus the player's
// IdeaCounts: nothing to backfill for players who played before ideas.
//
//   earned = floor(runs / every) + granted
//   held   = earned − spent − forfeited, at most `hold`
//
// Ideas earned past the cap are forfeited, written down the next time the
// counts are read (settle), so a spent idea doesn't bring back the runs
// played while full. Earning only grows between reads and every write
// settles first, so settling late gives what settling at each run would.
//
// Writing one (M2-4, makscee/void-board#790) spends a held idea and stores its
// text, private to its author, as a `written` Idea for M2-5's reader. Its
// author may take a `written` one back, which refunds it. M2-5's reader
// (./idea-reading.ts) refunds one it couldn't make.
import { randomUUID } from "node:crypto";
import { IDEA_TEXT_MAX, IDEA_TEXT_MIN, type Idea, type IdeasView, type MvpRules, type MyIdeasView } from "../../../src/mvp/contract.js";
import type { RunDeps } from "./runs.js";
import type { IdeaCounts } from "./store.js";

type IdeaDeps = Pick<RunDeps, "store" | "rules">;
type WriteDeps = Pick<RunDeps, "store" | "rules" | "now">;

/** The idea tunables, defaults filled in. */
export function ideaRules(rules: Pick<MvpRules, "ideaEveryRuns" | "ideaHold">): { every: number; hold: number } {
  return { every: Math.max(1, rules.ideaEveryRuns ?? 3), hold: Math.max(0, rules.ideaHold ?? 3) };
}

/** Reads the player's counts, forfeits ideas earned past the cap (writing
 * only when that changed something), and returns them with the ideas held. */
function settle(deps: IdeaDeps, playerId: string): { counts: IdeaCounts; runs: number; held: number } {
  const { every, hold } = ideaRules(deps.rules);
  const runs = deps.store.rating(playerId)?.runs ?? 0;
  const counts = deps.store.ideaCounts(playerId);
  const raw = Math.floor(runs / every) + counts.granted - counts.spent - counts.forfeited;
  if (raw > hold) {
    counts.forfeited += raw - hold;
    deps.store.putIdeaCounts(playerId, counts);
  }
  // A rate raised later can earn less than was spent: none held, not negative.
  return { counts, runs, held: Math.max(0, Math.min(raw, hold)) };
}

/** M2-6: the states where an idea waits for its author's pick. */
const READY_STATES = new Set(["pick-archetype", "pick-reading"]);

function view(deps: IdeaDeps, playerId: string, s: { runs: number; held: number }): IdeasView {
  const { every, hold } = ideaRules(deps.rules);
  const ready = deps.store.ideas({ playerId }).filter((i) => READY_STATES.has(i.state)).length;
  return { held: s.held, nextIn: s.held >= hold ? null : every - (s.runs % every), ready };
}

/** The player's ideas, for Home. */
export function ideasOf(deps: IdeaDeps, playerId: string): IdeasView {
  return view(deps, playerId, settle(deps, playerId));
}

/** Dev "+1 idea": one more, unless the player already holds the cap. */
export function grantIdea(deps: IdeaDeps, playerId: string): IdeasView {
  const s = settle(deps, playerId);
  if (s.held < ideaRules(deps.rules).hold) {
    s.counts.granted++;
    deps.store.putIdeaCounts(playerId, s.counts);
    s.held++;
  }
  return view(deps, playerId, s);
}

/** Spends one idea (M2-4 calls it when an idea is written): false when the
 * player holds none. */
export function spendIdea(deps: IdeaDeps, playerId: string): boolean {
  const s = settle(deps, playerId);
  if (s.held < 1) return false;
  s.counts.spent++;
  deps.store.putIdeaCounts(playerId, s.counts);
  return true;
}

/** Gives back a spent idea (M2-5: an idea the reader couldn't make). Like
 * any idea earned, one past the cap is forfeited. */
export function refundIdea(deps: IdeaDeps, playerId: string): void {
  const counts = deps.store.ideaCounts(playerId);
  counts.spent = Math.max(0, counts.spent - 1);
  deps.store.putIdeaCounts(playerId, counts);
  settle(deps, playerId);
}

/** A write or cancel the rules refuse, with the HTTP status the API answers. */
export class IdeaRefused extends Error {
  constructor(readonly status: 400 | 404 | 409, message: string) {
    super(message);
  }
}

/** The player's My ideas screen: the ideas held, and their own sent ideas,
 * newest first. Never anyone else's text. */
export function myIdeas(deps: IdeaDeps, playerId: string): MyIdeasView {
  const sent = deps.store
    .ideas({ playerId })
    .reverse()
    .map(({ playerId: _, ...mine }) => mine);
  return { ideas: ideasOf(deps, playerId), sent };
}

/** Writes an idea: `text` trimmed, IDEA_TEXT_MIN–MAX characters, spending one
 * held idea. Throws IdeaRefused (400 the length, 409 none held). */
export function writeIdea(deps: WriteDeps, playerId: string, text: string): Idea {
  const t = text.trim();
  const chars = [...t].length;
  if (chars < IDEA_TEXT_MIN || chars > IDEA_TEXT_MAX) throw new IdeaRefused(400, `An idea is ${IDEA_TEXT_MIN}–${IDEA_TEXT_MAX} characters.`);
  if (!spendIdea(deps, playerId)) throw new IdeaRefused(409, "You hold no idea to send. Finish runs to earn one.");
  const idea: Idea = { ideaId: randomUUID(), playerId, text: t, state: "written", createdAt: deps.now().toISOString(), data: {} };
  deps.store.putIdea(idea);
  return idea;
}

/** Takes back the player's own `written` idea and refunds it. Throws
 * IdeaRefused: 404 not theirs (or no such idea), 409 already being read, or
 * the player holds the most ideas already (the refund would be lost). */
export function cancelIdea(deps: IdeaDeps, playerId: string, ideaId: string): void {
  const idea = deps.store.idea(ideaId);
  if (!idea || idea.playerId !== playerId) throw new IdeaRefused(404, "no such idea");
  if (idea.state !== "written") throw new IdeaRefused(409, "This idea is being read already.");
  const s = settle(deps, playerId);
  if (s.held >= ideaRules(deps.rules).hold) throw new IdeaRefused(409, `You hold ${s.held} ideas, the most you can. Send one first.`);
  deps.store.deleteIdea(ideaId);
  s.counts.spent--;
  deps.store.putIdeaCounts(playerId, s.counts);
}
