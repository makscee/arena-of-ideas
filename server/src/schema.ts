import { integer, sqliteTable, text } from "drizzle-orm/sqlite-core";

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

export type User = typeof users.$inferSelect;
export type Session = typeof sessions.$inferSelect;
export type EmailCode = typeof emailCodes.$inferSelect;
export type LadderGhost = typeof ladderGhosts.$inferSelect;
export type LadderChampion = typeof ladderChampions.$inferSelect;
export type RunSubmission = typeof runSubmissions.$inferSelect;
