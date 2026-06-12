/**
 * The expired-row sweep: hour-stale email codes (expired or consumed) and
 * expired sessions go; everything inside its TTL — including a code expired
 * less than the 1h debugging grace ago — stays.
 */
import { describe, expect, test } from "vitest";
import { runCleanupOnce } from "./cleanup.js";
import { openDb } from "./db.js";
import { emailCodes, sessions } from "./schema.js";

const NOW = 1_750_000_000;
const HOUR = 60 * 60;

describe("expired-row cleanup", () => {
  test("prunes hour-stale codes and expired sessions, keeps the live ones", () => {
    const { db } = openDb(":memory:");
    db.insert(emailCodes)
      .values([
        { id: "live", email: "a@x.com", codeHash: "h", createdAt: NOW - 60, expiresAt: NOW + 540 },
        { id: "stale", email: "a@x.com", codeHash: "h", createdAt: NOW - 2 * HOUR, expiresAt: NOW - HOUR - 1 },
        { id: "spent", email: "a@x.com", codeHash: "h", createdAt: NOW - 3 * HOUR, expiresAt: NOW + 540, attempts: 1, consumedAt: NOW - HOUR - 1 },
        { id: "in-grace", email: "a@x.com", codeHash: "h", createdAt: NOW - 700, expiresAt: NOW - 60 }, // expired, but inside the 1h grace
      ])
      .run();
    db.insert(sessions)
      .values([
        { id: "live", userId: "u", tokenHash: "t1", label: "x", createdAt: NOW - 100, lastUsedAt: NOW, expiresAt: NOW + 100 },
        { id: "dead", userId: "u", tokenHash: "t2", label: "x", createdAt: NOW - 200, lastUsedAt: NOW, expiresAt: NOW - 1 },
      ])
      .run();

    expect(runCleanupOnce(db, NOW)).toEqual({ emailCodes: 2, sessions: 1 });
    expect(db.select().from(emailCodes).all().map((c) => c.id).sort()).toEqual(["in-grace", "live"]);
    expect(db.select().from(sessions).all().map((s) => s.id)).toEqual(["live"]);

    // Idempotent: a second sweep finds nothing.
    expect(runCleanupOnce(db, NOW)).toEqual({ emailCodes: 0, sessions: 0 });
  });
});
