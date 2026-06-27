/**
 * Server backing for the ideas table (#076 slice 2). The kernel (src/ideas.ts)
 * defined the IdeaStore surface and the serialized `Idea` shape; the in-memory
 * and localStorage backings live there. This is the THIRD backing: SQLite,
 * server-side, one vote per player keyed on the session's userId.
 *
 * The shape every backing agrees on is the kernel's: an `Idea` is
 * { id, authorId, text, seq, votes[] }, ids are "idea-<seq>", seq is the
 * submission ordinal AND the rank tiebreak, and list() returns ideas ranked by
 * vote count descending, ties by seq. These pure functions re-derive exactly
 * that shape from two tables (ideas + idea_votes), so a row round-trips
 * byte-equivalent to what InMemoryIdeaStore would hold — slice 3's UI talks to
 * one interface whether the backing is local or this remote one.
 *
 * The one-vote-per-player rule is a DB constraint, not app logic: idea_votes
 * has a composite primary key on (ideaId, userId). A second vote for the same
 * (idea, player) collides with the PK — onConflictDoUpdate switches the existing
 * row's direction in place, so a re-vote can never become a second row even
 * under a race. Votes are SWITCH-ONLY: the insert-or-switch has no delete branch,
 * so once a row exists it stays (you may only flip up↔down). Remove the PK and a
 * concurrent double-insert would make two rows (the ideas.test.ts must-fail-first
 * proves the constraint, not app logic, is what holds).
 *
 * Pure functions over a `deps` object (db + clock), the runs.ts pattern: a
 * route reads the request, calls one of these, and json's the outcome. No
 * wall-clock reads in the fns — `clock` is injected, unix seconds.
 */
import { and, eq, gte, lt, max, sql } from "drizzle-orm";
import {
  assertSubmittableText,
  rankIdeas,
  type Idea,
  type VoteDir,
  type VoteMap,
} from "../../src/index.js";
import type { DB } from "./db.js";
import { ideas as ideasTable, ideaVotes } from "./schema.js";

export interface IdeaDeps {
  db: DB;
  /** Unix seconds. */
  clock: () => number;
}

export type SubmitIdeaOutcome =
  | { submitted: true; idea: Idea }
  | { submitted: false; reason: string };

export type CastVoteOutcome =
  | { cast: true; direction: VoteDir; idea: Idea }
  | { cast: false; reason: string };

/** Seconds in a day — the one-per-day window, measured against UTC midnight
 * (unix time is anchored to UTC midnight, so flooring to this aligns days). */
const DAY_SECONDS = 86_400;

/** Submit an idea by `authorId`. Text is trimmed and validated by the kernel's
 * shared rule (empty/whitespace-only is rejected) so this backing can never
 * disagree with the in-memory one on what counts as submittable. One idea per
 * player per UTC day: a second submit inside the same day is refused (the limit
 * is a SERVER rule — the client can't be trusted — read against the injected
 * `clock` and the already-present `ideas.created_at`, no new state). The new
 * idea gets the next submission ordinal and an empty vote set. */
export function submitIdea(deps: IdeaDeps, authorId: string, text: string): SubmitIdeaOutcome {
  let trimmed: string;
  try {
    trimmed = assertSubmittableText(text);
  } catch (err) {
    return { submitted: false, reason: (err as Error).message };
  }
  // One-per-day: count this author's ideas created within the current UTC day.
  const dayStart = Math.floor(deps.clock() / DAY_SECONDS) * DAY_SECONDS;
  const todays = deps.db
    .select({ c: sql<number>`count(*)` })
    .from(ideasTable)
    .where(and(eq(ideasTable.authorId, authorId), gte(ideasTable.createdAt, dayStart), lt(ideasTable.createdAt, dayStart + DAY_SECONDS)))
    .all()[0]?.c ?? 0;
  if (todays >= 1) {
    return {
      submitted: false,
      reason: "You've already shared an idea today — one idea per day. The limit resets at midnight UTC.",
    };
  }
  const seq = nextSeq(deps.db);
  const id = `idea-${seq}`;
  deps.db
    .insert(ideasTable)
    .values({ id, seq, authorId, text: trimmed, createdAt: deps.clock() })
    .run();
  return { submitted: true, idea: { id, authorId, text: trimmed, seq, votes: {} } };
}

/** Cast `userId`'s `direction` vote on `ideaId` — the kernel's directional,
 * switch-only semantics, here over the idea_votes table. Insert-or-switch: the
 * composite PK is the one-vote-per-player floor, and onConflictDoUpdate switches
 * an existing row's direction in place (same direction is an idempotent write,
 * the other direction flips it). There is NO delete branch — a vote is never
 * removed, only switched, so a row's presence is the player's permanent
 * participation. Returns the idea in its post-cast state. Unknown ideaId is a
 * rejected outcome (no such idea to vote on). */
export function castIdeaVote(deps: IdeaDeps, userId: string, ideaId: string, direction: VoteDir): CastVoteOutcome {
  const { db } = deps;
  const row = db.select().from(ideasTable).where(eq(ideasTable.id, ideaId)).all()[0];
  if (row === undefined) {
    return { cast: false, reason: `no idea with id ${ideaId}` };
  }
  // Insert-or-switch: the PK blocks a second row; on collision we update the
  // direction (idempotent if unchanged, a flip if opposite). Never a delete.
  db.insert(ideaVotes)
    .values({ ideaId, userId, direction, votedAt: deps.clock() })
    .onConflictDoUpdate({ target: [ideaVotes.ideaId, ideaVotes.userId], set: { direction } })
    .run();
  return { cast: true, direction, idea: readIdea(db, row) };
}

/** The currency: how many distinct ideas `userId` has voted on, in either
 * direction — `count(*)` over their idea_votes rows. Derived, no stored counter;
 * non-farmable because votes are switch-only (no un-vote to re-mint) and a flip
 * does not change the row count. */
export function votedIdeaCount(deps: IdeaDeps, userId: string): number {
  const row = deps.db
    .select({ c: sql<number>`count(*)` })
    .from(ideaVotes)
    .where(eq(ideaVotes.userId, userId))
    .all()[0];
  return row?.c ?? 0;
}

/** Every idea, ranked by vote count descending, ties by submission order — the
 * kernel's total, stable ordering. Detached deep copies (rankIdeas clones), so
 * the same serialized shape the in-memory backing returns. */
export function listIdeas(deps: IdeaDeps): Idea[] {
  const rows = deps.db.select().from(ideasTable).all();
  const votesByIdea = votesByIdeaId(deps.db);
  const all: Idea[] = rows.map((r) => ({
    id: r.id,
    authorId: r.authorId,
    text: r.text,
    seq: r.seq,
    votes: votesByIdea.get(r.id) ?? {},
  }));
  return rankIdeas(all);
}

/** The next submission ordinal — one past the highest stored seq, 0 on an empty
 * table. The seq column is UNIQUE, so two submits can never collide on it. */
function nextSeq(db: DB): number {
  const top = db.select({ m: max(ideasTable.seq) }).from(ideasTable).all()[0]?.m;
  return top === null || top === undefined ? 0 : top + 1;
}

/** Re-derive one idea's kernel shape (with its current sorted, directional vote
 * map) from a stored row. */
function readIdea(db: DB, row: typeof ideasTable.$inferSelect): Idea {
  const rows = db
    .select({ userId: ideaVotes.userId, direction: ideaVotes.direction })
    .from(ideaVotes)
    .where(eq(ideaVotes.ideaId, row.id))
    .all();
  return { id: row.id, authorId: row.authorId, text: row.text, seq: row.seq, votes: voteMapOf(rows) };
}

/** All votes, grouped by ideaId into directional vote maps — the kernel's vote
 * shape. Keys sorted so the serialized shape is stable, the same as the
 * in-memory backing. */
function votesByIdeaId(db: DB): Map<string, VoteMap> {
  const byIdea = new Map<string, Array<{ userId: string; direction: string }>>();
  for (const v of db.select().from(ideaVotes).all()) {
    const list = byIdea.get(v.ideaId) ?? [];
    list.push({ userId: v.userId, direction: v.direction });
    byIdea.set(v.ideaId, list);
  }
  const out = new Map<string, VoteMap>();
  for (const [id, list] of byIdea) out.set(id, voteMapOf(list));
  return out;
}

/** Build a sorted-key directional vote map from raw (userId, direction) rows —
 * the canonical, byte-stable shape the kernel stores. */
function voteMapOf(rows: Array<{ userId: string; direction: string }>): VoteMap {
  const map: VoteMap = {};
  for (const { userId } of [...rows].sort((a, b) => (a.userId < b.userId ? -1 : a.userId > b.userId ? 1 : 0))) {
    map[userId] = rows.find((r) => r.userId === userId)!.direction as VoteDir;
  }
  return map;
}
