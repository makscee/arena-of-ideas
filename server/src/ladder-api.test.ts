/**
 * Shared-ladder API contract: public leaderboard reads, authenticated run
 * submission with kernel re-derivation, tamper rejection, own-ghost exclusion
 * across users and across one user's runs, and the crown race.
 *
 * The test client plays the way the slice-3 remote backing will: kernel
 * transitions against a per-fight view fetched over HTTP (`?exclude=me` pool
 * + current champion), then submits the finished run in serializeRun form.
 * Content is injected as a one-unit Titan pool — the kernel ladder tests'
 * deterministic climber — so outcomes are pinned by seed, not luck.
 */
import { describe, expect, test } from "vitest";
import type { Hono } from "hono";
import {
  BOOTSTRAP_RUN_ID,
  BOOTSTRAP_TEAMS,
  buy,
  fight,
  InMemoryLadderStore,
  initRun,
  ladderFight,
  openLadder,
  serializeRun,
  STARTING_LIVES,
  stressRegistry,
  type LadderStore,
  type RunEvent,
  type RunState,
  type TeamSnapshot,
  type UnitDef,
} from "../../src/index.js";
import { createApp } from "./app.js";
import { MAX_RUN_BYTES, MAX_RUN_LOG_EVENTS } from "./runs.js";
import type { AuthEnv } from "./auth.js";
import { openDb } from "./db.js";
import { createMockMailClient, type MockMailClient } from "./mail.js";
import { createRateLimiter } from "./rate-limit.js";

const TITAN: UnitDef = { name: "Titan", base: { hp: 100, pwr: 50 } };
const GOLIATH: UnitDef = { name: "Goliath", base: { hp: 200, pwr: 80 } };
const BASE = BOOTSTRAP_TEAMS[0]!.length; // round-1 bootstrap ghost count

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

interface Ctx {
  app: Hono<AuthEnv>;
  mailer: MockMailClient;
}

function makeCtx(): Ctx {
  const { db } = openDb(":memory:");
  const mailer = createMockMailClient();
  const clock = () => 1_750_000_000;
  const app = createApp({
    db,
    clock,
    mailClient: mailer,
    rateLimiters: {
      ipStart: createRateLimiter({ limit: 100, windowMs: 60_000, clock: () => clock() * 1000 }),
      emailStart: createRateLimiter({ limit: 100, windowMs: 60_000, clock: () => clock() * 1000 }),
    },
    content: { pool: [TITAN], statuses: stressRegistry },
  });
  return { app, mailer };
}

async function login(ctx: Ctx, email: string): Promise<string> {
  const start = await ctx.app.request("/v1/auth/login/email/start", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ email }),
  });
  expect(start.status).toBe(200);
  const code = ctx.mailer.sent[ctx.mailer.sent.length - 1]!.text.match(/\b(\d{6})\b/)![1]!;
  const verify = await ctx.app.request("/v1/auth/login/email/verify", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ email, code }),
  });
  expect(verify.status).toBe(200);
  return ((await verify.json()) as { token: string }).token;
}

async function fetchMyPool(ctx: Ctx, token: string, round: number): Promise<TeamSnapshot[]> {
  const res = await ctx.app.request(`/v1/ladder/pool/${round}?exclude=me`, {
    headers: { authorization: `Bearer ${token}` },
  });
  expect(res.status).toBe(200);
  return ((await res.json()) as { pool: TeamSnapshot[] }).pool;
}

async function fetchPublicPool(ctx: Ctx, round: number): Promise<TeamSnapshot[]> {
  const res = await ctx.app.request(`/v1/ladder/pool/${round}`);
  expect(res.status).toBe(200);
  return ((await res.json()) as { pool: TeamSnapshot[] }).pool;
}

async function fetchChampion(ctx: Ctx): Promise<{ champion: TeamSnapshot | null; holder: string | null }> {
  const res = await ctx.app.request("/v1/ladder/champion");
  expect(res.status).toBe(200);
  return (await res.json()) as { champion: TeamSnapshot | null; holder: string | null };
}

/** Open a run — the handshake every legit client performs before playing. */
async function openRun(ctx: Ctx, token: string, runId: string): Promise<{ status: number; body: Record<string, unknown> }> {
  const res = await ctx.app.request("/v1/runs/open", {
    method: "POST",
    headers: { "content-type": "application/json", authorization: `Bearer ${token}` },
    body: JSON.stringify({ runId }),
  });
  return { status: res.status, body: (await res.json()) as Record<string, unknown> };
}

/** Play a whole run the remote-backing way: open the runId, then each fight
 * against a view freshly fetched over HTTP (own ghosts excluded), writes
 * staying local. */
async function playSharedRun(ctx: Ctx, token: string, seed: number, runId: string): Promise<RunState> {
  expect((await openRun(ctx, token, runId)).status).toBe(200);
  return playUnopenedRun(ctx, token, seed, runId);
}

/** The play loop without the open handshake — what a client that skips the
 * handshake produces (its submission must be rejected). */
async function playUnopenedRun(ctx: Ctx, token: string, seed: number, runId: string): Promise<RunState> {
  let s = buy(initRun({ seed, runId, pool: [TITAN], statuses: stressRegistry }), 0);
  for (let guard = 0; s.status === "active"; guard++) {
    if (guard > 200) throw new Error(`run ${runId} did not terminate`);
    const pool = await fetchMyPool(ctx, token, s.round);
    const champ = (await fetchChampion(ctx)).champion;
    const round = s.round;
    const view: LadderStore = {
      poolAt: (r) => (r === round ? pool : []),
      addSnapshot: () => {},
      champion: () => champ,
      setChampion: () => {},
    };
    s = ladderFight(s, view);
  }
  return s;
}

async function submit(ctx: Ctx, token: string, raw: string): Promise<{ status: number; body: Record<string, unknown> }> {
  const res = await ctx.app.request("/v1/runs", {
    method: "POST",
    headers: { "content-type": "application/json", authorization: `Bearer ${token}` },
    body: JSON.stringify({ run: raw }),
  });
  return { status: res.status, body: (await res.json()) as Record<string, unknown> };
}

const ofType = <T extends RunEvent["type"]>(log: readonly RunEvent[], t: T) =>
  log.filter((e): e is Extract<RunEvent, { type: T }> => e.type === t);

// ---------------------------------------------------------------------------
// Leaderboard reads — public, no login
// ---------------------------------------------------------------------------

describe("leaderboard reads work logged-out", () => {
  test("champion and pools are readable with no bearer; fresh ladder shows the bootstrap", async () => {
    const ctx = makeCtx();
    const { champion, holder } = await fetchChampion(ctx);
    expect(champion!.runId).toBe(BOOTSTRAP_RUN_ID);
    expect(holder).toBeNull();
    const pool = await fetchPublicPool(ctx, 1);
    expect(pool.length).toBe(BASE);
    expect(pool.every((g) => g.runId === BOOTSTRAP_RUN_ID)).toBe(true);
  });

  test("a bad round is 400; exclude=me without a session is 401; submission needs auth", async () => {
    const ctx = makeCtx();
    expect((await ctx.app.request("/v1/ladder/pool/zero")).status).toBe(400);
    expect((await ctx.app.request("/v1/ladder/pool/10001")).status).toBe(400); // bounded above too
    expect((await ctx.app.request("/v1/ladder/pool/1?exclude=me")).status).toBe(401);
    const res = await ctx.app.request("/v1/runs", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ run: "{}" }),
    });
    expect(res.status).toBe(401);
  });
});

// ---------------------------------------------------------------------------
// Happy path: legal replay → ghosts in the pool, crown on the seat
// ---------------------------------------------------------------------------

describe("authenticated submission happy path", () => {
  test("a legally-replayed run lands its ghosts and its crown", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");

    const run = await playSharedRun(ctx, ada, 1, "titan-a");
    expect(run).toMatchObject({ status: "over", endedBy: "crown" }); // seed 1 dethrones the bootstrap champion

    const { status, body } = await submit(ctx, ada, serializeRun(run));
    expect(status).toBe(200);
    expect(body).toMatchObject({ accepted: true, runId: "titan-a", endedBy: "crown", crowned: true });

    // Ghosts appear in subsequent pool reads — one per round fought, public.
    for (let round = 1; round <= run.round; round++) {
      const pool = await fetchPublicPool(ctx, round);
      const ghost = pool.find((g) => g.runId === "titan-a")!;
      expect(ghost.team.map((u) => u.name)).toEqual(["Titan"]);
      expect(ghost.seq).toBe(pool.length - 1); // re-sequenced onto the pool's end
    }
    // The crown is on the seat, visible logged-out.
    expect((await fetchChampion(ctx)).champion!.runId).toBe("titan-a");
  });

  test("two users share one ladder; each user's draws exclude their own ghosts across runs", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");

    const runA = await playSharedRun(ctx, ada, 1, "titan-a");
    expect((await submit(ctx, ada, serializeRun(runA))).body).toMatchObject({ accepted: true });

    // Bob sees Ada's ghost; Ada does not see her own.
    expect((await fetchMyPool(ctx, bob, 1)).map((g) => g.runId)).toContain("titan-a");
    expect((await fetchMyPool(ctx, ada, 1)).map((g) => g.runId)).not.toContain("titan-a");

    // Bob's run draws from pools holding Ada's ghosts — and replays cleanly.
    const runB = await playSharedRun(ctx, bob, 2, "titan-b");
    expect((await submit(ctx, bob, serializeRun(runB))).body).toMatchObject({ accepted: true });

    // Ada's SECOND run: her first run's ghosts are out of her draws (user-level
    // exclusion, not just runId-level), while the public pool still holds all.
    const runA2 = await playSharedRun(ctx, ada, 3, "titan-a2");
    const firstDraw = ofType(runA2.log, "OpponentDrawn")[0]!;
    expect(firstDraw.candidates).toBe(BASE + 1); // bootstrap + bob only — titan-a excluded
    expect((await fetchPublicPool(ctx, 1)).length).toBe(BASE + 2); // …though it is right there
    const { body } = await submit(ctx, ada, serializeRun(runA2));
    expect(body).toMatchObject({ accepted: true }); // the replay used the same filtered view
  });
});

// ---------------------------------------------------------------------------
// Tampered submissions
// ---------------------------------------------------------------------------

type Loose = { [k: string]: any };

describe("tampered submissions are rejected", () => {
  async function legitRun(ctx: Ctx, token: string): Promise<RunState> {
    return playSharedRun(ctx, token, 1, "titan-a");
  }

  test("mutated final stats diverge from re-derivation", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const claimed = JSON.parse(serializeRun(await legitRun(ctx, ada))) as Loose;
    claimed["team"][0].base.hp += 100; // the inflated stat line
    const { status, body } = await submit(ctx, ada, JSON.stringify(claimed));
    expect(status).toBe(422);
    expect(body).toMatchObject({ accepted: false });
    expect(body["reason"]).toMatch(/diverges/);
    // Nothing entered the pool.
    expect((await fetchPublicPool(ctx, 1)).length).toBe(BASE);
  });

  test("an illegal decision sequence is rejected by the kernel itself", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const claimed = JSON.parse(serializeRun(await legitRun(ctx, ada))) as Loose;
    const log = claimed["log"] as Loose[];
    const i = log.findIndex((e) => e["type"] === "Bought");
    log.splice(i + 1, 0, ...Array.from({ length: 20 }, () => ({ ...log[i] }))); // buys the gold can't pay for
    const { status, body } = await submit(ctx, ada, JSON.stringify(claimed));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/invalid decision/);
  });

  test("a wrong-seed replay cannot reproduce the claimed run", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const claimed = JSON.parse(serializeRun(await legitRun(ctx, ada))) as Loose;
    claimed["seed"] += 1;
    const { status, body } = await submit(ctx, ada, JSON.stringify(claimed));
    expect(status).toBe(422);
    expect(body).toMatchObject({ accepted: false });
  });

  test("a champion this ladder never seated is rejected by name", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const claimed = JSON.parse(serializeRun(await legitRun(ctx, ada))) as Loose;
    for (const e of claimed["log"] as Loose[]) {
      if (e["type"] === "ChampionChallenged") e["champion"] = "phantom";
    }
    const { status, body } = await submit(ctx, ada, JSON.stringify(claimed));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/never seated/);
  });

  test("a run played with foreign content is rejected", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    // Legally played and serialized — but against a pool the arena never shipped.
    expect((await openRun(ctx, ada, "goliath")).status).toBe(200);
    const local = openLadder(new InMemoryLadderStore(), stressRegistry);
    let s = buy(initRun({ seed: 1, runId: "goliath", pool: [GOLIATH], statuses: stressRegistry }), 0);
    while (s.status === "active") s = ladderFight(s, local);
    const { status, body } = await submit(ctx, ada, serializeRun(s));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/content/);
  });

  test("explicit-opponent fights don't belong on the shared ladder", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    expect((await openRun(ctx, ada, "offline")).status).toBe(200);
    let s = buy(initRun({ seed: 1, runId: "offline", pool: [TITAN], statuses: stressRegistry }), 0);
    for (let i = 0; i < STARTING_LIVES; i++) s = fight(s, [{ name: "Wall", base: { hp: 9999, pwr: 9999 } }]);
    expect(s.status).toBe("over");
    const { status, body } = await submit(ctx, ada, serializeRun(s));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/fights only on the ladder/);
  });

  test("resubmission, reserved runId, unfinished runs, garbage", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const run = await legitRun(ctx, ada);

    expect((await submit(ctx, ada, serializeRun(run))).status).toBe(200);
    const again = await submit(ctx, ada, serializeRun(run));
    expect(again.status).toBe(422);
    expect(again.body["reason"]).toMatch(/already submitted/);

    const reserved = JSON.parse(serializeRun(run)) as Loose;
    reserved["runId"] = BOOTSTRAP_RUN_ID;
    expect((await submit(ctx, ada, JSON.stringify(reserved))).body["reason"]).toMatch(/reserved/);

    const active = buy(initRun({ seed: 9, runId: "wip", pool: [TITAN], statuses: stressRegistry }), 0);
    expect((await submit(ctx, ada, serializeRun(active))).body["reason"]).toMatch(/still active/);

    expect((await submit(ctx, ada, "not json")).status).toBe(422);
    const bad = await ctx.app.request("/v1/runs", {
      method: "POST",
      headers: { "content-type": "application/json", authorization: `Bearer ${ada}` },
      body: JSON.stringify({ run: 42 }),
    });
    expect(bad.status).toBe(400);
  });
});

// ---------------------------------------------------------------------------
// Prefix forgeries — the claimed pool prefix must be one the player could
// actually have observed (Cass's refutation of the first slice-2 build)
// ---------------------------------------------------------------------------

describe("prefix forgeries are rejected", () => {
  test("claiming an empty round-1 pool to grab a free champion challenge is rejected", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    // The handshake is performed honestly — the forgery is in the claimed prefix.
    expect((await openRun(ctx, ada, "forged")).status).toBe(200);

    // Forge: play locally against an EMPTY pool view — the log then claims
    // round-1's pool held 0 ghosts, so the kernel reads the empty draw as
    // "outran every ghost" and challenges the champion at round 1. On a
    // bootstrap-seeded ladder the round-1 pool is never smaller than BASE,
    // so no player can ever have observed this.
    const champ = (await fetchChampion(ctx)).champion;
    const emptyView: LadderStore = {
      poolAt: () => [],
      addSnapshot: () => {},
      champion: () => champ,
      setChampion: () => {},
    };
    let s = buy(initRun({ seed: 1, runId: "forged", pool: [TITAN], statuses: stressRegistry }), 0);
    while (s.status === "active") s = ladderFight(s, emptyView);
    expect(s.endedBy).toBe("crown"); // seed 1: a lone round-1 Titan beats the bootstrap champion
    expect(ofType(s.log, "Snapshotted")[0]!.seq).toBe(0); // the impossible claim

    const { status, body } = await submit(ctx, ada, serializeRun(s));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/impossibly short/);
    // Nothing landed: no ghost, and the bootstrap champion still holds the seat.
    expect((await fetchPublicPool(ctx, 1)).length).toBe(BASE);
    expect((await fetchChampion(ctx)).champion!.runId).toBe(BOOTSTRAP_RUN_ID);
  });

  test("claiming a shorter-than-observed prefix to cherry-pick the opponent draw is rejected", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");

    // Ada's accepted run lands a ghost in every round she fought: Bob's
    // visible round-1 pool is now BASE+1 long.
    const runA = await playSharedRun(ctx, ada, 1, "titan-a");
    expect((await submit(ctx, ada, serializeRun(runA))).body).toMatchObject({ accepted: true });

    // Forge: Bob opens AFTER Ada's ghosts landed, then plays against views
    // truncated to the bootstrap-only prefix — claiming the pool as it stood
    // BEFORE Ada, cherry-picking the weaker deterministic draw. His open
    // pinned the longer pool: he cannot claim not to have seen it.
    expect((await openRun(ctx, bob, "cherry")).status).toBe(200);
    expect((await fetchMyPool(ctx, bob, 1)).length).toBe(BASE + 1);
    let s = buy(initRun({ seed: 2, runId: "cherry", pool: [TITAN], statuses: stressRegistry }), 0);
    for (let guard = 0; s.status === "active"; guard++) {
      if (guard > 200) throw new Error("forged run did not terminate");
      const round = s.round;
      const visible = await fetchMyPool(ctx, bob, round);
      const truncated = visible.filter((g) => g.runId === BOOTSTRAP_RUN_ID); // bootstrap rows are the pool's prefix
      const champ = (await fetchChampion(ctx)).champion;
      const view: LadderStore = {
        poolAt: (r) => (r === round ? truncated : []),
        addSnapshot: () => {},
        champion: () => champ,
        setChampion: () => {},
      };
      s = ladderFight(s, view);
    }

    const { status, body } = await submit(ctx, bob, serializeRun(s));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/impossibly short/);
    expect((await fetchPublicPool(ctx, 1)).map((g) => g.runId)).not.toContain("cherry");
  });
});

// ---------------------------------------------------------------------------
// The open handshake contract
// ---------------------------------------------------------------------------

describe("the open handshake", () => {
  test("a finished run whose runId was never opened is rejected", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const run = await playUnopenedRun(ctx, ada, 1, "no-handshake");
    const { status, body } = await submit(ctx, ada, serializeRun(run));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/never opened/);
  });

  test("runIds open once, are length-bounded, and the bootstrap's is reserved", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    expect((await openRun(ctx, ada, "once")).status).toBe(200);
    const again = await openRun(ctx, ada, "once");
    expect(again.status).toBe(422);
    expect(again.body["reason"]).toMatch(/already open/);
    expect((await openRun(ctx, ada, BOOTSTRAP_RUN_ID)).body["reason"]).toMatch(/reserved/);
    expect((await openRun(ctx, ada, "")).status).toBe(422);
    expect((await openRun(ctx, ada, "x".repeat(129))).status).toBe(422);
    const badBody = await ctx.app.request("/v1/runs/open", {
      method: "POST",
      headers: { "content-type": "application/json", authorization: `Bearer ${ada}` },
      body: JSON.stringify({ runId: 42 }),
    });
    expect(badBody.status).toBe(400);
    const noAuth = await ctx.app.request("/v1/runs/open", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ runId: "anon" }),
    });
    expect(noAuth.status).toBe(401);
  });

  test("a run opened by one user cannot be submitted by another", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");
    expect((await openRun(ctx, ada, "stolen")).status).toBe(200);
    const run = await playUnopenedRun(ctx, bob, 1, "stolen");
    const { status, body } = await submit(ctx, bob, serializeRun(run));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/different user/);
  });

  test("a stale-but-real prefix still replays: the pool grew mid-run", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");

    // Ada's run is played but NOT yet submitted when Bob opens and takes his
    // first look at round 1 — so Bob's open-time floor is the bootstrap pool.
    const runA = await playSharedRun(ctx, ada, 1, "titan-a");
    expect((await openRun(ctx, bob, "stale")).status).toBe(200);
    const stalePool = await fetchMyPool(ctx, bob, 1);
    const staleChamp = (await fetchChampion(ctx)).champion;
    expect(stalePool.length).toBe(BASE);

    // Ada's submission lands her ghosts mid-Bob's-run…
    expect((await submit(ctx, ada, serializeRun(runA))).body).toMatchObject({ accepted: true, crowned: true });

    // …and Bob finishes against his stale round-1 view + fresh later views.
    let s = buy(initRun({ seed: 2, runId: "stale", pool: [TITAN], statuses: stressRegistry }), 0);
    for (let guard = 0; s.status === "active"; guard++) {
      if (guard > 200) throw new Error("stale run did not terminate");
      const round = s.round;
      const pool = round === 1 ? stalePool : await fetchMyPool(ctx, bob, round);
      const champ = round === 1 ? staleChamp : (await fetchChampion(ctx)).champion;
      const view: LadderStore = {
        poolAt: (r) => (r === round ? pool : []),
        addSnapshot: () => {},
        champion: () => champ,
        setChampion: () => {},
      };
      s = ladderFight(s, view);
    }

    const { status, body } = await submit(ctx, bob, serializeRun(s));
    expect(status).toBe(200); // round 1 claims the BASE-long prefix — exactly his open-time floor
    expect(body).toMatchObject({ accepted: true });
  });
});

// ---------------------------------------------------------------------------
// Submission cost bounds — the replay is synchronous
// ---------------------------------------------------------------------------

describe("submission cost bounds", () => {
  test("an oversized submission is rejected before replay", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const { status, body } = await submit(ctx, ada, "x".repeat(MAX_RUN_BYTES + 1));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/bytes/);
  });

  test("a log padded past the event cap is rejected before replay", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const run = await playSharedRun(ctx, ada, 1, "padded");
    const claimed = JSON.parse(serializeRun(run)) as Loose;
    (claimed["log"] as unknown[]).push(...Array.from({ length: MAX_RUN_LOG_EVENTS }, () => 0));
    const { status, body } = await submit(ctx, ada, JSON.stringify(claimed));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/events/);
  });
});

// ---------------------------------------------------------------------------
// The crown race: two runs legally beat the same champion
// ---------------------------------------------------------------------------

describe("crown race", () => {
  test("the first submission takes the spot; the second keeps its ghosts but not the crown", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");

    // Both play to a crown against the PRISTINE ladder (neither has submitted).
    const runA = await playSharedRun(ctx, ada, 1, "titan-a");
    const runB = await playSharedRun(ctx, bob, 2, "titan-b");
    expect(runA.endedBy).toBe("crown");
    expect(runB.endedBy).toBe("crown");

    expect((await submit(ctx, ada, serializeRun(runA))).body).toMatchObject({ accepted: true, crowned: true });
    const second = await submit(ctx, bob, serializeRun(runB));
    expect(second.status).toBe(200); // the run is legal — it replays cleanly
    expect(second.body).toMatchObject({ accepted: true, crowned: false }); // but the champion it beat is gone

    expect((await fetchChampion(ctx)).champion!.runId).toBe("titan-a"); // the seat held
    expect((await fetchPublicPool(ctx, 1)).map((g) => g.runId)).toContain("titan-b"); // the ghosts landed
  });
});
