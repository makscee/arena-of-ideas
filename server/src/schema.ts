import { integer, primaryKey, sqliteTable, text, uniqueIndex } from "drizzle-orm/sqlite-core";

/**
 * Arena-owned auth tables. Shapes follow void-auth's schema (the OTP/session
 * services are copied from there — see prds/016), trimmed to what the arena
 * needs: no roles, no phone/telegram identities.
 *
 * Conventions inherited from void-auth:
 * - All *At columns are unix **seconds** (integer), not ms.
 * - Codes and tokens are never stored raw — sha256 hex only.
 */

export const users = sqliteTable("users", {
  id: text("id").primaryKey(),
  email: text("email").notNull().unique(),
  /** Null until the first-login name pick (a later slice wires the UI). */
  displayName: text("display_name"),
  createdAt: integer("created_at").notNull(),
  updatedAt: integer("updated_at").notNull(),
});

export const sessions = sqliteTable("sessions", {
  id: text("id").primaryKey(),
  userId: text("user_id").notNull(),
  /** sha256 hex of the raw bearer token. Raw token surfaces exactly once. */
  tokenHash: text("token_hash").notNull().unique(),
  label: text("label").notNull(),
  createdAt: integer("created_at").notNull(),
  lastUsedAt: integer("last_used_at").notNull(),
  expiresAt: integer("expires_at").notNull(),
});

export const emailCodes = sqliteTable("email_codes", {
  id: text("id").primaryKey(),
  email: text("email").notNull(),
  /** sha256 hex of the 6-digit code. Raw code goes only into the email. */
  codeHash: text("code_hash").notNull(),
  createdAt: integer("created_at").notNull(),
  expiresAt: integer("expires_at").notNull(),
  attempts: integer("attempts").notNull().default(0),
  consumedAt: integer("consumed_at"),
});

/**
 * Shared-ladder tables (slice 2). One ladder per server instance, opened from
 * the kernel's bootstrap at boot. Ghost pools are append-only — (round, seq)
 * is the kernel's insertion ordinal, unique per pool. `userId` is the owner
 * of the run that fielded the team (null for bootstrap ghosts): a user's
 * ghosts span their runs and are excluded from that user's own draws.
 */

export const ladderGhosts = sqliteTable("ladder_ghosts", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  round: integer("round").notNull(),
  seq: integer("seq").notNull(),
  runId: text("run_id").notNull(),
  userId: text("user_id"),
  /** JSON UnitDef[] — the snapshot's team, stored as the kernel serializes it. */
  team: text("team").notNull(),
});

/** Champion history, append-only — the current champion is the latest row.
 * History stays queryable by runId so run re-derivation can replay a champion
 * challenge against the champion that was actually seated at the time. */
export const ladderChampions = sqliteTable("ladder_champions", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  runId: text("run_id").notNull(),
  userId: text("user_id"),
  round: integer("round").notNull(),
  seq: integer("seq").notNull(),
  /** JSON UnitDef[]. */
  team: text("team").notNull(),
});

/** Run-open handshake — the anti-forgery pin. A run must be opened (POST
 * /v1/runs/open) by its owner before it is played; submission requires the
 * matching row, and opens EXPIRE (`RUN_OPEN_TTL_SECONDS` in runs.ts): an open
 * is not a bankable asset whose recorded views appreciate as the ladder grows.
 * `openedAt` is what the TTL is measured from. */
export const runOpens = sqliteTable("run_opens", {
  runId: text("run_id").primaryKey(),
  userId: text("user_id").notNull(),
  /** Highest ladder_ghosts.id at open — provenance of the ladder state the
   * run began against. Replay checks the serve record (run_pool_serves),
   * which is strictly stronger; this stays as a recorded fact. */
  ghostWatermark: integer("ghost_watermark").notNull(),
  openedAt: integer("opened_at").notNull(),
});

/** Pool views the server HANDED OUT for an open run — the replay's ground
 * truth. Every play read (GET /v1/runs/:runId/pool/:round) records the
 * user-filtered prefix length it served and the champion seated at that
 * moment; submission replay accepts a claimed Snapshotted.seq only if it
 * equals a served length for that (runId, round), and a champion challenge
 * only against the champion CO-SERVED with that view. Nothing is trusted
 * that the server did not itself serve, at the time it served it. Re-reads
 * append rows (identical views dedupe via the unique index), so refreshing
 * a round's pool never bricks a submission. */
export const runPoolServes = sqliteTable(
  "run_pool_serves",
  {
    id: integer("id").primaryKey({ autoIncrement: true }),
    runId: text("run_id").notNull(),
    round: integer("round").notNull(),
    /** Length of the user-filtered pool prefix served — what Snapshotted.seq
     * must equal for this round. */
    servedLen: integer("served_len").notNull(),
    /** runId of the champion seated when this view was served. */
    championRunId: text("champion_run_id").notNull(),
    servedAt: integer("served_at").notNull(),
  },
  (t) => [uniqueIndex("run_pool_serves_view_idx").on(t.runId, t.round, t.servedLen, t.championRunId)],
);

/** Accepted run submissions — one row per re-derived run. The primary key
 * makes runIds globally unique: a resubmission (or a cross-user runId
 * collision, which would corrupt the kernel's own-ghost runId filter) is
 * rejected at the door. */
export const runSubmissions = sqliteTable("run_submissions", {
  runId: text("run_id").primaryKey(),
  userId: text("user_id").notNull(),
  seed: integer("seed").notNull(),
  endedBy: text("ended_by").notNull(),
  finalRound: integer("final_round").notNull(),
  submittedAt: integer("submitted_at").notNull(),
});

/**
 * Ideas table (#076 slice 2) — the server backing for the free-text idea queue
 * (kernel shape: src/ideas.ts). One row per submitted idea; `seq` is the
 * submission ordinal that the kernel's `Idea` carries (id = `idea-<seq>`, and
 * the rank tiebreak), assigned server-side as the next-highest seq. Votes live
 * in their own table so the one-vote-per-player rule is a DB constraint, not an
 * app convention. `createdAt` is unix seconds — a real timestamp the kernel
 * shape leaves to data (it orders by seq, clock-free).
 */
export const ideas = sqliteTable("ideas", {
  /** The kernel id — "idea-<seq>". Stable, globally unique per server. */
  id: text("id").primaryKey(),
  /** Submission ordinal: the kernel's seq, also the rank tiebreak. Unique so
   * two submits can never collide on an ordinal. */
  seq: integer("seq").notNull().unique(),
  authorId: text("author_id").notNull(),
  text: text("text").notNull(),
  createdAt: integer("created_at").notNull(),
});

/** One vote per (idea, player) — the composite PK is the one-vote-per-player
 * guarantee at the DB layer: a player voting twice on one idea hits the primary
 * key and is a no-op (toggleIdeaVote deletes to un-vote), never a double count.
 * `votedAt` keeps the order votes arrived, should a later slice want it. */
export const ideaVotes = sqliteTable(
  "idea_votes",
  {
    ideaId: text("idea_id").notNull(),
    userId: text("user_id").notNull(),
    votedAt: integer("voted_at").notNull(),
  },
  (t) => [primaryKey({ columns: [t.ideaId, t.userId] })],
);

export type Idea = typeof ideas.$inferSelect;
export type IdeaVote = typeof ideaVotes.$inferSelect;

export type User = typeof users.$inferSelect;
export type Session = typeof sessions.$inferSelect;
export type EmailCode = typeof emailCodes.$inferSelect;
export type LadderGhost = typeof ladderGhosts.$inferSelect;
export type RunOpen = typeof runOpens.$inferSelect;
export type RunPoolServe = typeof runPoolServes.$inferSelect;
export type LadderChampion = typeof ladderChampions.$inferSelect;
export type RunSubmission = typeof runSubmissions.$inferSelect;
