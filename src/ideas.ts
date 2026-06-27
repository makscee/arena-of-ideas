// The ideas table — free-text suggestions players write, ranked by votes.
// A player submits an idea (any text: content, gameplay, a feature, a removal);
// every player gets one DIRECTIONAL vote per idea — up or down, switchable but
// never removable. The list ranks ideas by a directional metric (up raises,
// down lowers) so the top N can be taken into work. Votes are a PRIORITY QUEUE,
// not an entry gate — they reorder ideas, they never admit or reject one.
//
// This module owns the storage boundary: the IdeaStore interface and its
// in-memory backing (tests, browser). It stays free of node built-ins — index.ts,
// which the browser imports, re-exports it; a file backing (if one is ever
// needed) lives off this index, the way FileLadderStore does for the ladder.
//
// No wall-clock reads: an idea's order in the table is its submission ordinal
// (seq), the same clock-free ordering the ladder uses for its pools. A real
// timestamp, if a client ever wants one, comes in as data.

/** A vote's direction — one player holds at most one per idea. Up raises the
 * idea's rank, down lowers it; both are participation (they accrue currency). */
export type VoteDir = "up" | "down";

/** A player's vote on an idea, keyed by playerId → direction. One vote per
 * player (a map key is unique), directional, and SWITCH-ONLY: a key is added or
 * its direction flipped, never deleted — once you've voted you have voted
 * forever (you may only change up↔down). Keys are kept sorted so the serialized
 * shape is stable across backings. */
export type VoteMap = Record<string, VoteDir>;

/** An idea on the table, as the store holds and returns it. list() returns
 * detached deep copies, so a returned Idea is the caller's to do with freely. */
export interface Idea {
  /** Stable id — "idea-N" off the store's submission counter. */
  id: string;
  /** The player who wrote it; the only player who may remove it. */
  authorId: string;
  /** The free text, trimmed; never empty (submit rejects empty). */
  text: string;
  /** Submission ordinal — createdAt-style ordering without a clock; also the
   * stable tiebreak when two ideas have equal rank (earlier seq ranks higher). */
  seq: number;
  /** The players who have voted on this idea, each mapped to their direction:
   * one vote per player, switchable up↔down, never removed. Keys are sorted so
   * the serialized shape is stable. */
  votes: VoteMap;
}

/** The storage boundary the ideas feature depends on — nothing else.
 * Backings: InMemoryIdeaStore (below), PersistedIdeaStore (web/ideas-store.ts).
 * list() hands back detached deep copies, so a caller (a renderer) may read or
 * even mutate them freely without reaching the store's state — mutation just
 * doesn't write back; submit/castVote/removeOwn are the only ways in. */
export interface IdeaStore {
  /** Append a new idea by `authorId` with `text` (trimmed). Throws on empty /
   * whitespace-only text — an idea with nothing in it is not an idea. Returns
   * the stored idea (with its assigned id, seq, and empty vote set). */
  submit(text: string, authorId: string): Idea;
  /** Cast `playerId`'s `dir` vote on `ideaId`: add it if absent, flip it if the
   * player already voted the other way, no-op if already in this direction. One
   * vote per player per idea, SWITCH-ONLY — there is no operation that returns a
   * player to neutral. Throws on an unknown ideaId. */
  castVote(ideaId: string, playerId: string, dir: VoteDir): void;
  /** Every idea, ranked by directional vote score descending (up raises, down
   * lowers); ties broken by submission order (lower seq first), so the order is
   * total and stable. The returned ideas are detached deep copies — mutating one
   * never touches the store. */
  list(): readonly Idea[];
  /** The currency: the count of ideas `playerId` has voted on (in either
   * direction). Derived from the table, no stored counter. */
  votedCount(playerId: string): number;
  /** The author removes their own idea. Throws on an unknown ideaId; throws if
   * `authorId` is not the idea's author — only the author may remove it. */
  removeOwn(ideaId: string, authorId: string): void;
}

/** The serialized ideas table — the one shape every persistent backing stores:
 * the ideas in submission (seq) order plus the next-id counter, so a reopened
 * store keeps minting fresh ids after a reload. A web client writes this to
 * localStorage; both backings round-trip byte-equivalent tables. */
export interface IdeasData {
  ideas: Idea[];
  /** The next submission ordinal — the seq the next submit will assign. */
  nextSeq: number;
}

/** A fresh, empty ideas table — what a backing starts from when nothing is stored. */
export function emptyIdeasData(): IdeasData {
  return { ideas: [], nextSeq: 0 };
}

/** Parse stored ideas JSON, loudly: a present-but-unreadable table must never
 * silently become a fresh one — that would drop every idea and vote in it.
 * `label` names the backing in the error (a file path, a storage key). */
export function parseIdeasData(raw: string, label: string): IdeasData {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    throw new Error(`Ideas ${label} is not valid JSON: ${(err as Error).message}`);
  }
  const data = parsed as Partial<IdeasData>;
  if (typeof data !== "object" || data === null || !Array.isArray(data.ideas) || typeof data.nextSeq !== "number") {
    throw new Error(`Ideas ${label} has no ideas array — not an ideas table`);
  }
  return { ideas: data.ideas, nextSeq: data.nextSeq };
}

/** Validate and trim submitted text, or throw — shared by every backing so they
 * can never disagree on what counts as a submittable idea. */
export function assertSubmittableText(text: string): string {
  const trimmed = text.trim();
  if (trimmed === "") {
    throw new Error("an idea's text must not be empty");
  }
  return trimmed;
}

/** A vote map with its keys in sorted order — the canonical, byte-stable shape
 * every backing stores, so a serialized table is identical across them. */
function sortedVotes(votes: VoteMap): VoteMap {
  const out: VoteMap = {};
  for (const id of Object.keys(votes).sort()) out[id] = votes[id]!;
  return out;
}

/** Cast `playerId`'s `dir` vote — the directional, switch-only semantics every
 * backing shares. Absent → add `{playerId: dir}`; present same direction →
 * unchanged (a no-op); present other direction → flip. There is NO branch that
 * removes a player: once you've voted you have voted, you may only switch
 * up↔down. Returns a new sorted map; never mutates the input. */
export function castVote(votes: VoteMap, playerId: string, dir: VoteDir): VoteMap {
  if (votes[playerId] === dir) return sortedVotes(votes);
  return sortedVotes({ ...votes, [playerId]: dir });
}

/** An idea's directional rank weight: +1 per up-vote, −1 per down-vote. Up
 * raises an idea, down lowers it; an idea with equal up/down nets zero, the same
 * weight as an unvoted one but ranked behind whatever has net-positive support. */
export function voteScore(votes: VoteMap): number {
  let score = 0;
  for (const dir of Object.values(votes)) score += dir === "up" ? 1 : -1;
  return score;
}

/** The currency: the count of ideas `playerId` has voted on, in EITHER
 * direction — a participation footprint. Non-farmable by construction: votes are
 * switch-only (you cannot un-vote to re-mint), and a flip leaves the count
 * unchanged. Derived live from the table; no stored counter. */
export function votedCount(data: IdeasData, playerId: string): number {
  return data.ideas.filter((idea) => playerId in idea.votes).length;
}

/** Rank ideas by directional vote score descending (up raises, down lowers),
 * ties broken by submission order (lower seq first) — the total, stable ordering
 * list() returns. Returns DETACHED deep copies (jsonClone, the same convention
 * writes use): a caller that mutates a returned idea's `votes` — slice 3 renders
 * this list — can never reach back into the store's state to corrupt its ranking
 * or persist through the next write. Does not mutate the input. */
export function rankIdeas(ideas: readonly Idea[]): Idea[] {
  return ideas
    .map((idea) => jsonClone(idea))
    .sort((a, b) => voteScore(b.votes) - voteScore(a.votes) || a.seq - b.seq);
}

/** The in-memory backing — tests now, a parity reference for the web client.
 * Ideas are JSON-cloned on write so a stored idea is exactly what a persistent
 * backing would round-trip: isolated from later caller mutation, and
 * byte-equivalent across backings. */
export class InMemoryIdeaStore implements IdeaStore {
  private data: IdeasData = emptyIdeasData();

  submit(text: string, authorId: string): Idea {
    return submitInto(this.data, text, authorId);
  }

  castVote(ideaId: string, playerId: string, dir: VoteDir): void {
    castVoteIn(this.data, ideaId, playerId, dir);
  }

  list(): readonly Idea[] {
    return rankIdeas(this.data.ideas);
  }

  votedCount(playerId: string): number {
    return votedCount(this.data, playerId);
  }

  removeOwn(ideaId: string, authorId: string): void {
    removeOwnFrom(this.data, ideaId, authorId);
  }
}

/** An IdeaStore over an IdeasData record with a write-through persist hook —
 * the shared engine of every persistent backing (localStorage, file): same
 * clone-on-write isolation as InMemoryIdeaStore, plus `persist(data)` after
 * every mutation. The hook owns serialization and the medium; this class owns
 * the IdeaStore semantics, so backings can never disagree on them. */
export class PersistedIdeaStore implements IdeaStore {
  private readonly data: IdeasData;
  private readonly persist: (data: IdeasData) => void;

  constructor(data: IdeasData, persist: (data: IdeasData) => void) {
    this.data = data;
    this.persist = persist;
  }

  submit(text: string, authorId: string): Idea {
    const idea = submitInto(this.data, text, authorId);
    this.persist(this.data);
    return idea;
  }

  castVote(ideaId: string, playerId: string, dir: VoteDir): void {
    castVoteIn(this.data, ideaId, playerId, dir);
    this.persist(this.data);
  }

  list(): readonly Idea[] {
    return rankIdeas(this.data.ideas);
  }

  votedCount(playerId: string): number {
    return votedCount(this.data, playerId);
  }

  removeOwn(ideaId: string, authorId: string): void {
    removeOwnFrom(this.data, ideaId, authorId);
    this.persist(this.data);
  }
}

// The mutation core, shared by both backings so they can never disagree on
// semantics — each backing wraps these and adds its own persistence (none for
// InMemory, write-through for Persisted).

function submitInto(data: IdeasData, text: string, authorId: string): Idea {
  const trimmed = assertSubmittableText(text);
  const seq = data.nextSeq;
  const idea: Idea = { id: `idea-${seq}`, authorId, text: trimmed, seq, votes: {} };
  data.ideas.push(jsonClone(idea));
  data.nextSeq = seq + 1;
  return idea;
}

function castVoteIn(data: IdeasData, ideaId: string, playerId: string, dir: VoteDir): void {
  const idea = requireIdea(data, ideaId);
  idea.votes = castVote(idea.votes, playerId, dir);
}

function removeOwnFrom(data: IdeasData, ideaId: string, authorId: string): void {
  const idea = requireIdea(data, ideaId);
  if (idea.authorId !== authorId) {
    throw new Error(`idea ${ideaId} is not authored by ${authorId} — only its author may remove it`);
  }
  data.ideas = data.ideas.filter((i) => i.id !== ideaId);
}

/** The idea with `ideaId`, or throw — an operation on an idea that does not
 * exist is a caller bug, surfaced loudly rather than silently dropped. */
function requireIdea(data: IdeasData, ideaId: string): Idea {
  const idea = data.ideas.find((i) => i.id === ideaId);
  if (idea === undefined) {
    throw new Error(`no idea with id ${ideaId}`);
  }
  return idea;
}

/** The clone every write path shares — JSON-safe data in, isolated copy out,
 * exactly the kernel ladder's rule: a stored idea is byte-equivalent across
 * backings and isolated from later caller mutation. */
export function jsonClone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v)) as T;
}
