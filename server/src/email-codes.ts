import { createHash, randomInt, randomUUID, timingSafeEqual } from "node:crypto";
import { and, desc, eq, gt, isNull } from "drizzle-orm";
import type { DB } from "./db.js";
import { emailCodes } from "./schema.js";

/**
 * email-codes service: issue/verify single-use 6-digit codes for email OTP
 * login. Copied from void-auth (src/services/email-codes.ts) — pinned
 * decision: copy, don't call. Adapted from bun:sqlite to better-sqlite3;
 * logic unchanged.
 *
 * Security properties (preserved exactly):
 * - Raw codes never persisted; sha256 at rest.
 * - Timestamps are unix-seconds (integer), not ms.
 * - 10-minute TTL; 5-attempt cap before the row is dead.
 * - Timing-safe hash comparison.
 *
 * Pure module: takes a Drizzle DB and a clock fn so it can be tested
 * deterministically.
 */

export const OTP_TTL_SECONDS = 10 * 60;
const ATTEMPT_CAP = 5;

export type VerifyCodeResult =
  | { ok: true }
  | { ok: false; reason: "wrong" | "expired" | "exhausted" | "no_code" };

export interface EmailCodesService {
  issue(email: string): { code: string };
  verify(email: string, code: string): VerifyCodeResult;
}

/** Generate a uniformly distributed 6-digit numeric string with leading zeros. */
export function generate(): string {
  const n = randomInt(0, 1_000_000);
  return n.toString().padStart(6, "0");
}

function sha256(s: string): string {
  return createHash("sha256").update(s).digest("hex");
}

export function createEmailCodes(
  db: DB,
  clock: () => number = () => Math.floor(Date.now() / 1000),
): EmailCodesService {
  return {
    issue(email: string): { code: string } {
      const code = generate();
      const now = clock();
      db.insert(emailCodes)
        .values({
          id: randomUUID(),
          email,
          codeHash: sha256(code),
          createdAt: now,
          expiresAt: now + OTP_TTL_SECONDS,
          attempts: 0,
        })
        .run();
      return { code };
    },

    verify(email: string, code: string): VerifyCodeResult {
      const now = clock();

      // Most recent unconsumed unexpired row for this email.
      const row = db
        .select()
        .from(emailCodes)
        .where(
          and(
            eq(emailCodes.email, email),
            isNull(emailCodes.consumedAt),
            gt(emailCodes.expiresAt, now),
          ),
        )
        .orderBy(desc(emailCodes.createdAt))
        .limit(1)
        .all()[0];

      if (!row) {
        // Either nothing was ever issued, or the latest live row was already
        // exhausted/expired/consumed. Distinguish "exhausted" from "no_code"
        // by checking the latest row regardless of state.
        const latest = db
          .select()
          .from(emailCodes)
          .where(eq(emailCodes.email, email))
          .orderBy(desc(emailCodes.createdAt))
          .limit(1)
          .all()[0];
        if (latest && latest.attempts >= ATTEMPT_CAP && latest.consumedAt === null) {
          return { ok: false, reason: "exhausted" };
        }
        if (latest && latest.expiresAt <= now && latest.consumedAt === null) {
          return { ok: false, reason: "expired" };
        }
        return { ok: false, reason: "no_code" };
      }

      if (row.attempts >= ATTEMPT_CAP) {
        // Row is dead — short-circuit even if a correct code is presented.
        return { ok: false, reason: "exhausted" };
      }

      // Timing-safe compare — both inputs are 64-char hex (sha256), equal length.
      const candidate = Buffer.from(sha256(code), "hex");
      const stored = Buffer.from(row.codeHash, "hex");
      if (candidate.length === stored.length && timingSafeEqual(candidate, stored)) {
        db.update(emailCodes)
          .set({ consumedAt: now })
          .where(eq(emailCodes.id, row.id))
          .run();
        return { ok: true };
      }

      // Wrong code — increment attempts. The 5th wrong attempt trips the cap.
      const newAttempts = row.attempts + 1;
      db.update(emailCodes)
        .set({ attempts: newAttempts })
        .where(eq(emailCodes.id, row.id))
        .run();
      if (newAttempts >= ATTEMPT_CAP) {
        return { ok: false, reason: "exhausted" };
      }
      return { ok: false, reason: "wrong" };
    },
  };
}
