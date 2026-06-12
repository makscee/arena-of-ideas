/**
 * The expired-row sweep: hour-stale email codes (expired or consumed),
 * expired sessions, TTL-lapsed run opens (with their recorded pool serves)
 * and serves of already-submitted runs go; everything inside its TTL —
 * including a code expired less than the 1h debugging grace ago — stays.
 */
import { describe, expect, test } from "vitest";
import { runCleanupOnce } from "./cleanup.js";
import { openDb } from "./db.js";
import { SqliteLadderStore } from "./ladder-store.js";
import { openRun, RUN_OPEN_TTL_SECONDS } from "./runs.js";
import { emailCodes, runOpens, runPoolServes, runSubmissions, sessions } from "./schema.js";

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

    expect(runCleanupOnce(db, NOW)).toEqual({ emailCodes: 2, sessions: 1, runOpens: 0, runPoolServes: 0 });
    expect(db.select().from(emailCodes).all().map((c) => c.id).sort()).toEqual(["in-grace", "live"]);
    expect(db.select().from(sessions).all().map((s) => s.id)).toEqual(["live"]);

    // Idempotent: a second sweep finds nothing.
    expect(runCleanupOnce(db, NOW)).toEqual({ emailCodes: 0, sessions: 0, runOpens: 0, runPoolServes: 0 });
  });

  test("prunes TTL-lapsed opens with their serves, and serves of submitted runs", () => {
    const { db } = openDb(":memory:");
    db.insert(runOpens)
      .values([
        { runId: "live", userId: "u", ghostWatermark: 0, openedAt: NOW - 60 },
        { runId: "lapsed", userId: "u", ghostWatermark: 0, openedAt: NOW - RUN_OPEN_TTL_SECONDS - 1 },
        { runId: "done", userId: "u", ghostWatermark: 0, openedAt: NOW - 60 },
      ])
      .run();
    db.insert(runPoolServes)
      .values([
        { runId: "live", round: 1, servedLen: 2, championRunId: "bootstrap", servedAt: NOW - 60 },
        { runId: "lapsed", round: 1, servedLen: 0, championRunId: "bootstrap", servedAt: NOW - RUN_OPEN_TTL_SECONDS - 1 },
        { runId: "lapsed", round: 2, servedLen: 0, championRunId: "bootstrap", servedAt: NOW - RUN_OPEN_TTL_SECONDS - 1 },
        { runId: "done", round: 1, servedLen: 2, championRunId: "bootstrap", servedAt: NOW - 60 },
      ])
      .run();
    db.insert(runSubmissions)
      .values([{ runId: "done", userId: "u", seed: 1, endedBy: "crown", finalRound: 4, submittedAt: NOW - 30 }])
      .run();

    expect(runCleanupOnce(db, NOW)).toEqual({ emailCodes: 0, sessions: 0, runOpens: 1, runPoolServes: 3 });
    expect(db.select().from(runOpens).all().map((o) => o.runId).sort()).toEqual(["done", "live"]);
    expect(db.select().from(runPoolServes).all().map((s) => s.runId)).toEqual(["live"]);

    // One-shot survives the sweep: a swept-or-not submitted runId never reopens.
    const store = new SqliteLadderStore(db);
    const reopened = openRun({ db, store, clock: () => NOW }, "u", "done");
    expect(reopened).toMatchObject({ opened: false });
    expect((reopened as { reason: string }).reason).toMatch(/already submitted|already open/);
    db.delete(runOpens).run(); // even with every open row gone…
    expect(openRun({ db, store, clock: () => NOW }, "u", "done")).toMatchObject({ opened: false }); // …"done" stays closed

    expect(runCleanupOnce(db, NOW)).toEqual({ emailCodes: 0, sessions: 0, runOpens: 0, runPoolServes: 0 });
  });
});
