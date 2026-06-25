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
 * has a composite primary key on (ideaId, userId). A second vote in the same
 * direction collides with the PK — onConflictDoNothing makes it the no-op the
 * toggle semantics require, so a re-vote can never double-count even under a
 * race. toggleIdeaVote reads presence to decide add-vs-remove, but the floor is
 * the constraint: remove the PK and a concurrent double-insert would count
 * twice (the ideas.test.ts must-fail-first proves it).
 *
 * Pure functions over a `deps` object (db + clock), the runs.ts pattern: a
 * route reads the request, calls one of these, and json's the outcome. No
 * wall-clock reads in the fns — `clock` is injected, unix seconds.
 */
import { and, eq, max } from "drizzle-orm";
import {
  assertSubmittableText,
  rankIdeas,
  type Idea,
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

export type ToggleVoteOutcome =
  | { toggled: true; voted: boolean; idea: Idea }
  | { toggled: false; reason: string };

/** Submit an idea by `authorId`. Text is trimmed and validated by the kernel's
 * shared rule (empty/whitespace-only is rejected) so this backing can never
 * disagree with the in-memory one on what counts as submittable. The new idea
 * gets the next submission ordinal and an empty vote set. */
export function submitIdea(deps: IdeaDeps, authorId: string, text: string): SubmitIdeaOutcome {
  let trimmed: string;
  try {
    trimmed = assertSubmittableText(text);
  } catch (err) {
    return { submitted: false, reason: (err as Error).message };
  }
  const seq = nextSeq(deps.db);
  const id = `idea-${seq}`;
  deps.db
    .insert(ideasTable)
    .values({ id, seq, authorId, text: trimmed, createdAt: deps.clock() })
    .run();
  return { submitted: true, idea: { id, authorId, text: trimmed, seq, votes: [] } };
}

/** Toggle `userId`'s vote on `ideaId`: add it if absent, remove it if present —
 * the kernel's toggle semantics, here over the idea_votes table. The composite
 * PK is the one-vote-per-player floor: the insert is onConflictDoNothing, so a
 * re-vote in the same direction is a DB-level no-op, never a double count.
 * Returns the idea in its post-toggle state and whether the caller now holds a
 * vote. Unknown ideaId is a rejected outcome (no such idea to vote on). */
export function toggleIdeaVote(deps: IdeaDeps, userId: string, ideaId: string): ToggleVoteOutcome {
  const { db } = deps;
  const row = db.select().from(ideasTable).where(eq(ideasTable.id, ideaId)).all()[0];
  if (row === undefined) {
    return { toggled: false, reason: `no idea with id ${ideaId}` };
  }
  const existing = db
    .select()
    .from(ideaVotes)
    .where(and(eq(ideaVotes.ideaId, ideaId), eq(ideaVotes.userId, userId)))
    .all();
  let voted: boolean;
  if (existing.length > 0) {
    db.delete(ideaVotes).where(and(eq(ideaVotes.ideaId, ideaId), eq(ideaVotes.userId, userId))).run();
    voted = false;
  } else {
    // onConflictDoNothing: the PK is the guarantee — a second add for the same
    // (idea, user) collides and is dropped, so it can never double-count.
    db.insert(ideaVotes)
      .values({ ideaId, userId, votedAt: deps.clock() })
      .onConflictDoNothing()
      .run();
    voted = true;
  }
  return { toggled: true, voted, idea: readIdea(db, row) };
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
    votes: votesByIdea.get(r.id) ?? [],
  }));
  return rankIdeas(all);
}

/** The next submission ordinal — one past the highest stored seq, 0 on an empty
 * table. The seq column is UNIQUE, so two submits can never collide on it. */
function nextSeq(db: DB): number {
  const top = db.select({ m: max(ideasTable.seq) }).from(ideasTable).all()[0]?.m;
  return top === null || top === undefined ? 0 : top + 1;
}

/** Re-derive one idea's kernel shape (with its current sorted vote set) from a
 * stored row. */
function readIdea(db: DB, row: typeof ideasTable.$inferSelect): Idea {
  const votes = db
    .select({ userId: ideaVotes.userId })
    .from(ideaVotes)
    .where(eq(ideaVotes.ideaId, row.id))
    .all()
    .map((v) => v.userId)
    .sort();
  return { id: row.id, authorId: row.authorId, text: row.text, seq: row.seq, votes };
}

/** All votes, grouped by ideaId into sorted distinct vote sets — the kernel's
 * vote shape. Sorted so the serialized shape is stable, the same as the
 * in-memory backing. */
function votesByIdeaId(db: DB): Map<string, string[]> {
  const grouped = new Map<string, string[]>();
  for (const v of db.select().from(ideaVotes).all()) {
    const list = grouped.get(v.ideaId) ?? [];
    list.push(v.userId);
    grouped.set(v.ideaId, list);
  }
  for (const [id, list] of grouped) grouped.set(id, [...list].sort());
  return grouped;
}
