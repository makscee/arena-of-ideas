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

export type User = typeof users.$inferSelect;
export type Session = typeof sessions.$inferSelect;
export type EmailCode = typeof emailCodes.$inferSelect;
