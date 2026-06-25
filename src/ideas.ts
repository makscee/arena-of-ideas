// The ideas table — free-text suggestions players write, ranked by votes.
// A player submits an idea (any text: content, gameplay, a feature, a removal);
// every player gets one toggleable vote per idea; the list ranks ideas by vote
// count so the top N can be taken into work. Votes are a PRIORITY QUEUE, not an
// entry gate — they reorder ideas, they never admit or reject one.
//
// This module owns the storage boundary: the IdeaStore interface and its
// in-memory backing (tests, browser). It stays free of node built-ins — index.ts,
// which the browser imports, re-exports it; a file backing (if one is ever
// needed) lives off this index, the way FileLadderStore does for the ladder.
//
// No wall-clock reads: an idea's order in the table is its submission ordinal
// (seq), the same clock-free ordering the ladder uses for its pools. A real
// timestamp, if a client ever wants one, comes in as data.

/** An idea on the table, as the store holds and returns it.
 * Returned ideas are owned by the store: treat them as immutable. */
export interface Idea {
  /** Stable id — "idea-N" off the store's submission counter. */
  id: string;
  /** The player who wrote it; the only player who may remove it. */
  authorId: string;
  /** The free text, trimmed; never empty (submit rejects empty). */
  text: string;
  /** Submission ordinal — createdAt-style ordering without a clock; also the
   * stable tiebreak when two ideas have equal votes (earlier seq ranks higher). */
  seq: number;
  /** The set of players who have voted for this idea, modelled as a sorted
   * array of distinct playerIds: one vote per player, so a re-vote in the same
   * direction never double-counts. Sorted so the serialized shape is stable. */
  votes: string[];
}

/** The storage boundary the ideas feature depends on — nothing else.
 * Backings: InMemoryIdeaStore (below), PersistedIdeaStore (web/ideas-store.ts).
 * Returned ideas are owned by the store: treat them as immutable. */
export interface IdeaStore {
  /** Append a new idea by `authorId` with `text` (trimmed). Throws on empty /
   * whitespace-only text — an idea with nothing in it is not an idea. Returns
   * the stored idea (with its assigned id, seq, and empty vote set). */
  submit(text: string, authorId: string): Idea;
  /** Toggle `playerId`'s vote on `ideaId`: add it if absent, remove it if
   * present. One vote per player per idea (votes are a set), so voting the same
   * direction twice never double-counts. Throws on an unknown ideaId. */
  toggleVote(ideaId: string, playerId: string): void;
  /** Every idea, ranked by vote count descending; ties broken by submission
   * order (lower seq first), so the order is total and stable. */
  list(): readonly Idea[];
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

/** Add `playerId` to a sorted, distinct vote set if absent, remove it if
 * present — the toggle semantics, shared by every backing. Returns a new sorted
 * array; never mutates the input. One vote per player: a re-add is a no-op on
 * the set, a re-remove leaves it absent. */
export function toggledVotes(votes: readonly string[], playerId: string): string[] {
  const next = votes.includes(playerId)
    ? votes.filter((id) => id !== playerId)
    : [...votes, playerId];
  return [...next].sort();
}

/** Rank ideas by vote count descending, ties broken by submission order (lower
 * seq first) — the total, stable ordering list() returns. Does not mutate the
 * input; returns a new array of the same (store-owned) ideas. */
export function rankIdeas(ideas: readonly Idea[]): Idea[] {
  return [...ideas].sort((a, b) => b.votes.length - a.votes.length || a.seq - b.seq);
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

  toggleVote(ideaId: string, playerId: string): void {
    toggleVoteIn(this.data, ideaId, playerId);
  }

  list(): readonly Idea[] {
    return rankIdeas(this.data.ideas);
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

  toggleVote(ideaId: string, playerId: string): void {
    toggleVoteIn(this.data, ideaId, playerId);
    this.persist(this.data);
  }

  list(): readonly Idea[] {
    return rankIdeas(this.data.ideas);
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
  const idea: Idea = { id: `idea-${seq}`, authorId, text: trimmed, seq, votes: [] };
  data.ideas.push(jsonClone(idea));
  data.nextSeq = seq + 1;
  return idea;
}

function toggleVoteIn(data: IdeasData, ideaId: string, playerId: string): void {
  const idea = requireIdea(data, ideaId);
  idea.votes = toggledVotes(idea.votes, playerId);
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
