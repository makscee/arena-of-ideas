/**
 * Expired-row sweep — the void-auth cleanup.ts pattern, trimmed to the two
 * tables the arena accumulates: email_codes and sessions. Codes are deleted
 * an hour AFTER expiry/consumption (the grace keeps a just-failed login
 * debuggable); sessions go at expiry — verify() already refuses them, the
 * sweep just stops the rows piling up. Idempotent, and safe alongside
 * request traffic: SQLite serializes writes.
 */
import { and, isNotNull, lt, or } from "drizzle-orm";
import type { DB } from "./db.js";
import { emailCodes, sessions } from "./schema.js";

export interface CleanupResult {
  emailCodes: number;
  sessions: number;
}

const ONE_HOUR_SECONDS = 60 * 60;
const FIVE_MINUTES_MS = 5 * 60 * 1000;

/** One sweep. `nowSec` is unix seconds (injectable, the clock convention). */
export function runCleanupOnce(db: DB, nowSec: number): CleanupResult {
  const codeBoundary = nowSec - ONE_HOUR_SECONDS;
  const ec = db
    .delete(emailCodes)
    .where(
      or(
        lt(emailCodes.expiresAt, codeBoundary),
        and(isNotNull(emailCodes.consumedAt), lt(emailCodes.consumedAt, codeBoundary)),
      ),
    )
    .run();
  const s = db.delete(sessions).where(lt(sessions.expiresAt, nowSec)).run();
  return { emailCodes: ec.changes ?? 0, sessions: s.changes ?? 0 };
}

/** Start the recurring sweep; returns a stop() handle. The timer is unref'd —
 * it never keeps the process alive on its own. */
export function startCleanupTimer(db: DB, clock: () => number, intervalMs = FIVE_MINUTES_MS): { stop: () => void } {
  const tick = (): void => {
    try {
      const r = runCleanupOnce(db, clock());
      if (r.emailCodes + r.sessions > 0) {
        console.log(`[cleanup] pruned ${r.emailCodes} expired email codes, ${r.sessions} expired sessions`);
      }
    } catch (err) {
      console.error(`[cleanup] sweep failed: ${String(err)}`);
    }
  };
  const handle = setInterval(tick, intervalMs);
  handle.unref();
  return { stop: () => clearInterval(handle) };
}
