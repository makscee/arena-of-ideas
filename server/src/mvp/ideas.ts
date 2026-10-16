// Ideas a day (Toybox R2, makscee/void-board#915; before it, M2-3 earned 1
// per 3 finished runs): each arena day (./day.ts today(), the champion's
// day) a player gets their ideas back up to MvpRules.ideaHold (3). Unspent
// ones don't stack past it; runs earn none. IdeaCounts keeps the ideas held
// and the day they were last refilled: the first read on a later day (or
// ever, a fresh player or one from before this rule) refills, so nobody
// loses an idea they held.
//
// Writing one (M2-4, makscee/void-board#790) spends a held idea and stores its
// text, private to its author, as a `written` Idea for M2-5's reader. Its
// author may take a `written` one back, which refunds it. M2-5's reader
// (./idea-reading.ts) refunds one it couldn't make. M3-4: a proposal for a new
// version of a Library unit is written, spent and refunded the same way.
import { isCrudeText } from "./crude.js";
import { randomUUID } from "node:crypto";
import { IDEA_TEXT_MAX, IDEA_TEXT_MIN, type Idea, type IdeasView, type MvpRules, type MyIdeasView, type UnitId } from "../../../src/mvp/contract.js";
import type { RunDeps } from "./runs.js";
import type { IdeaCounts } from "./store.js";

type IdeaDeps = Pick<RunDeps, "store" | "rules" | "today">;
type WriteDeps = Pick<RunDeps, "store" | "rules" | "today" | "now">;

/** The ideas a player gets back each day and holds at most; absent, 3. */
export function ideaHold(rules: Pick<MvpRules, "ideaHold">): number {
  return Math.max(0, rules.ideaHold ?? 3);
}

/** Reads the player's counts and refills them on a new day (writing only
 * when that changed something); returns them with the ideas held and the
 * day's end, when the next refill comes. */
function settle(deps: IdeaDeps, playerId: string): { counts: IdeaCounts; held: number; refillsAt: string } {
  const hold = ideaHold(deps.rules);
  const day = deps.today();
  const counts = deps.store.ideaCounts(playerId);
  if (counts.day !== day.seq) {
    counts.held = Math.max(counts.held ?? 0, hold);
    counts.day = day.seq;
    deps.store.putIdeaCounts(playerId, counts);
  }
  return { counts, held: counts.held ?? 0, refillsAt: day.endsAt };
}

/** Stores a new number of ideas held. */
function hold(deps: IdeaDeps, playerId: string, counts: IdeaCounts, held: number): void {
  counts.held = held;
  deps.store.putIdeaCounts(playerId, counts);
}

/** M2-6: the states where an idea waits for its author's pick. */
const READY_STATES = new Set(["pick-archetype", "pick-reading"]);

function view(deps: IdeaDeps, playerId: string, s: { held: number; refillsAt: string }): IdeasView {
  const ready = deps.store.ideas({ playerId }).filter((i) => READY_STATES.has(i.state)).length;
  return { held: s.held, perDay: ideaHold(deps.rules), refillsAt: s.refillsAt, ready };
}

/** The player's ideas, for Home. */
export function ideasOf(deps: IdeaDeps, playerId: string): IdeasView {
  return view(deps, playerId, settle(deps, playerId));
}

/** Dev "+1 idea": one more, unless the player already holds the cap. */
export function grantIdea(deps: IdeaDeps, playerId: string): IdeasView {
  const s = settle(deps, playerId);
  if (s.held < ideaHold(deps.rules)) {
    s.counts.granted++;
    hold(deps, playerId, s.counts, ++s.held);
  }
  return view(deps, playerId, s);
}

/** Spends one idea (M2-4 calls it when an idea is written): false when the
 * player holds none. */
export function spendIdea(deps: IdeaDeps, playerId: string): boolean {
  const s = settle(deps, playerId);
  if (s.held < 1) return false;
  s.counts.spent++;
  hold(deps, playerId, s.counts, s.held - 1);
  return true;
}

/** Gives back a spent idea (M2-5: an idea the reader couldn't make). One
 * past the cap is forfeited. */
export function refundIdea(deps: IdeaDeps, playerId: string): void {
  const s = settle(deps, playerId);
  s.counts.spent = Math.max(0, s.counts.spent - 1);
  if (s.held >= ideaHold(deps.rules)) s.counts.forfeited++;
  hold(deps, playerId, s.counts, Math.min(ideaHold(deps.rules), s.held + 1));
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
 * held idea. M3-4: an `evolve` one proposes a new version of `target`, a unit
 * in the Library. Throws IdeaRefused (404 no such unit, 409 not in the
 * Library, 400 the length, 409 none held). */
export function writeIdea(deps: WriteDeps, playerId: string, text: string, evolve?: { target: UnitId }): Idea {
  if (evolve) {
    const unit = deps.store.unit(evolve.target);
    if (!unit) throw new IdeaRefused(404, "no such unit");
    if (unit.status !== "library") throw new IdeaRefused(409, "Only a unit in the Library can get a new version.");
  }
  const t = text.trim();
  const chars = [...t].length;
  if (chars < IDEA_TEXT_MIN || chars > IDEA_TEXT_MAX) throw new IdeaRefused(400, `An idea is ${IDEA_TEXT_MIN}–${IDEA_TEXT_MAX} characters.`);
  if (isCrudeText(t)) throw new IdeaRefused(400, "Try other words.");
  if (!spendIdea(deps, playerId)) throw new IdeaRefused(409, "You have no ideas left today. They come back at the day's end.");
  const idea: Idea = { ideaId: randomUUID(), playerId, text: t, state: "written", createdAt: deps.now().toISOString(), data: evolve ? { kind: "evolve", target: evolve.target } : {} };
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
  if (s.held >= ideaHold(deps.rules)) throw new IdeaRefused(409, `You hold ${s.held} ideas, the most you can. Send one first.`);
  deps.store.deleteIdea(ideaId);
  s.counts.spent--;
  hold(deps, playerId, s.counts, s.held + 1);
}
