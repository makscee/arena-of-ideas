import { eq } from "drizzle-orm";
import { createHash, randomBytes, randomUUID } from "node:crypto";
import type { DB } from "./db.js";
import { sessions, users } from "./schema.js";

/**
 * Sessions service — mint / verify / revoke long-lived bearer tokens.
 * Copied from void-auth (src/services/sessions.ts); arena has no roles, so
 * verify joins users only to confirm the row exists and to carry identity.
 *
 * Security (preserved exactly):
 *   - Tokens are 32 random bytes (~256 bits) base64url-encoded. Raw token
 *     surfaces to the caller exactly once and is never persisted nor logged.
 *   - Only sha256(token) is stored in `sessions.token_hash`.
 *
 * Time: all *At columns and return values are unix **seconds**.
 *
 * Ports: `db` (Drizzle handle; in-memory SQLite for tests) and an optional
 * unix-seconds `clock`.
 */

export interface MintInput {
  userId: string;
  label: string;
  lifetimeDays: number;
}

export interface MintResult {
  /** Raw bearer token. Returned exactly once — never persisted. */
  token: string;
  sessionId: string;
  /** Unix seconds. */
  expiresAt: number;
}

export interface SessionInfo {
  userId: string;
  email: string;
  displayName: string | null;
  sessionId: string;
  /** Unix seconds. */
  expiresAt: number;
}

const SECONDS_PER_DAY = 86400;
const TOKEN_BYTES = 32;

function nowSec(): number {
  return Math.floor(Date.now() / 1000);
}

function hashToken(token: string): string {
  return createHash("sha256").update(token).digest("hex");
}

function generateToken(): string {
  // base64url, no padding — URL/header-safe
  return randomBytes(TOKEN_BYTES).toString("base64url");
}

/**
 * Mint a new session. Returns the raw token + sessionId + unix-second expiry.
 * The raw token is never stored; only sha256(token) lands in the DB.
 */
export function mint(db: DB, input: MintInput, clock: () => number = nowSec): MintResult {
  const token = generateToken();
  const tokenHash = hashToken(token);
  const sessionId = randomUUID();
  const now = clock();
  const expiresAt = now + input.lifetimeDays * SECONDS_PER_DAY;

  db.insert(sessions)
    .values({
      id: sessionId,
      userId: input.userId,
      tokenHash,
      label: input.label,
      createdAt: now,
      lastUsedAt: now,
      expiresAt,
    })
    .run();

  return { token, sessionId, expiresAt };
}

/**
 * Verify a raw bearer token. Returns the bound user + session metadata if the
 * session exists and has not expired; otherwise null. Touches `last_used_at`
 * on success.
 */
export function verify(
  db: DB,
  token: string,
  clock: () => number = nowSec,
): SessionInfo | null {
  const tokenHash = hashToken(token);
  const now = clock();

  const row = db
    .select({
      sessionId: sessions.id,
      userId: sessions.userId,
      expiresAt: sessions.expiresAt,
      email: users.email,
      displayName: users.displayName,
    })
    .from(sessions)
    .innerJoin(users, eq(users.id, sessions.userId))
    .where(eq(sessions.tokenHash, tokenHash))
    .limit(1)
    .all()[0];

  if (!row) return null;
  if (row.expiresAt <= now) return null;

  db.update(sessions)
    .set({ lastUsedAt: now })
    .where(eq(sessions.id, row.sessionId))
    .run();

  return {
    userId: row.userId,
    email: row.email,
    displayName: row.displayName,
    sessionId: row.sessionId,
    expiresAt: row.expiresAt,
  };
}

/** Delete the session row. No-op if the sessionId does not exist. */
export function revoke(db: DB, sessionId: string): void {
  db.delete(sessions).where(eq(sessions.id, sessionId)).run();
}
