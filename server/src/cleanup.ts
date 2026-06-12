/**
 * Expired-row sweep — the void-auth cleanup.ts pattern, over the tables the
 * arena accumulates. Codes are deleted an hour AFTER expiry/consumption (the
 * grace keeps a just-failed login debuggable); sessions go at expiry —
 * verify() already refuses them, the sweep just stops the rows piling up.
 * Run opens go once their TTL has lapsed (submission already rejects them —
 * runs.ts), together with their recorded pool serves; serves of SUBMITTED
 * runs go too, since a runId is one-shot and can never replay again.
 * Idempotent, and safe alongside request traffic: SQLite serializes writes.
 */
import { and, inArray, isNotNull, lt, or } from "drizzle-orm";
import type { DB } from "./db.js";
import { RUN_OPEN_TTL_SECONDS } from "./runs.js";
import { emailCodes, runOpens, runPoolServes, runSubmissions, sessions } from "./schema.js";

export interface CleanupResult {
  emailCodes: number;
  sessions: number;
  runOpens: number;
  runPoolServes: number;
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
  // Serves first (they reference opens), then the expired opens themselves.
  const openCutoff = nowSec - RUN_OPEN_TTL_SECONDS;
  const expiredOpenIds = db.select({ runId: runOpens.runId }).from(runOpens).where(lt(runOpens.openedAt, openCutoff));
  const submittedIds = db.select({ runId: runSubmissions.runId }).from(runSubmissions);
  const ps = db
    .delete(runPoolServes)
    .where(or(inArray(runPoolServes.runId, expiredOpenIds), inArray(runPoolServes.runId, submittedIds)))
    .run();
  const ro = db.delete(runOpens).where(lt(runOpens.openedAt, openCutoff)).run();
  return {
    emailCodes: ec.changes ?? 0,
    sessions: s.changes ?? 0,
    runOpens: ro.changes ?? 0,
    runPoolServes: ps.changes ?? 0,
  };
}

/** Start the recurring sweep; returns a stop() handle. The timer is unref'd —
 * it never keeps the process alive on its own. */
export function startCleanupTimer(db: DB, clock: () => number, intervalMs = FIVE_MINUTES_MS): { stop: () => void } {
  const tick = (): void => {
    try {
      const r = runCleanupOnce(db, clock());
      if (r.emailCodes + r.sessions + r.runOpens + r.runPoolServes > 0) {
        console.log(
          `[cleanup] pruned ${r.emailCodes} expired email codes, ${r.sessions} expired sessions, ` +
            `${r.runOpens} expired run opens, ${r.runPoolServes} stale pool serves`,
        );
      }
    } catch (err) {
      console.error(`[cleanup] sweep failed: ${String(err)}`);
    }
  };
  const handle = setInterval(tick, intervalMs);
  handle.unref();
  return { stop: () => clearInterval(handle) };
}
