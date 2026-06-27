/**
 * Shared-ladder API contract: public leaderboard reads, authenticated run
 * submission with kernel re-derivation, tamper rejection, own-ghost exclusion
 * across users and across one user's runs, and the crown race.
 *
 * The test client plays the way the slice-3 remote backing will: open the
 * runId, then kernel transitions against per-fight views fetched through the
 * run-scoped play read (GET /v1/runs/:runId/pool/:round — pool + co-served
 * champion, recorded server-side), then submit the finished run in
 * serializeRun form. Content is injected as a one-unit Titan pool — the
 * kernel ladder tests' deterministic climber — so outcomes are pinned by
 * seed, not luck.
 */
import { describe, expect, test } from "vitest";
import type { Hono } from "hono";
import {
  BOOTSTRAP_RUN_ID,
  BOOTSTRAP_TEAMS,
  buy,
  challengeBoss,
  fight,
  InMemoryLadderStore,
  initRun,
  InvalidDecisionError,
  ladderFight,
  openLadder,
  serializeRun,
  STARTING_LIVES,
  stressAbilities,
  stressRegistry,
  TOWER_HEIGHT,
  UNIT_COST,
  type LadderStore,
  type RunEvent,
  type RunState,
  type TeamSnapshot,
  type UnitDef,
} from "../../src/index.js";
import { addGold, spawnUnit } from "../../web/dev-ops.js";
import { createApp } from "./app.js";
import { MAX_RUN_BYTES, MAX_RUN_LOG_EVENTS, RUN_OPEN_TTL_SECONDS } from "./runs.js";
import type { AuthEnv } from "./auth.js";
import { openDb } from "./db.js";
import { createMockMailClient, type MockMailClient } from "./mail.js";
import { createRateLimiter } from "./rate-limit.js";

const TITAN: UnitDef = { name: "Titan", base: { hp: 100, pwr: 50 }, ability: "Strike" };
const GOLIATH: UnitDef = { name: "Goliath", base: { hp: 200, pwr: 80 }, ability: "Strike" };
// Round-1's seeded pool size: the floor-1 climb teams PLUS the floor-1 boss-ghost
// (075-3 seeds a boss on every floor and leaves its team in the pool). All three
// climb floors happen to seed the same count, so BASE is the per-floor seeded size.
const BASE = BOOTSTRAP_TEAMS[0]!.length + 1;

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

interface Ctx {
  app: Hono<AuthEnv>;
  mailer: MockMailClient;
  /** Mutable server time (unix seconds) — TTL tests advance it. */
  now: { sec: number };
}

function makeCtx(opts: { poolServeLimit?: number } = {}): Ctx {
  const { db } = openDb(":memory:");
  const mailer = createMockMailClient();
  const now = { sec: 1_750_000_000 };
  const clock = () => now.sec;
  const app = createApp({
    db,
    clock,
    mailClient: mailer,
    rateLimiters: {
      ipStart: createRateLimiter({ limit: 100, windowMs: 60_000, clock: () => clock() * 1000 }),
      emailStart: createRateLimiter({ limit: 100, windowMs: 60_000, clock: () => clock() * 1000 }),
      // Out of the way by default; the serve rate-limit tests tighten it.
      poolServe: createRateLimiter({ limit: opts.poolServeLimit ?? 10_000, windowMs: 60_000, clock: () => clock() * 1000 }),
    },
    content: { pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities },
  });
  return { app, mailer, now };
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

/** One per-fight view, as the run-scoped play read serves it: the
 * user-filtered pool plus the champion seated at serve time. The server
 * records exactly this — submission replays only against recorded views. */
interface RunView {
  pool: TeamSnapshot[];
  champion: TeamSnapshot;
}

/** THE play read — GET /v1/runs/:runId/pool/:round. */
async function fetchRunView(ctx: Ctx, token: string, runId: string, round: number): Promise<RunView> {
  const res = await ctx.app.request(`/v1/runs/${runId}/pool/${round}`, {
    headers: { authorization: `Bearer ${token}` },
  });
  expect(res.status).toBe(200);
  const body = (await res.json()) as { pool: TeamSnapshot[]; champion: TeamSnapshot };
  return { pool: body.pool, champion: body.champion };
}

/** The kernel view over one served per-fight view, writes staying local. */
function viewOf(round: number, v: RunView): LadderStore {
  return {
    poolAt: (r) => (r === round ? v.pool : []),
    addSnapshot: () => {},
    bossAt: (floor) => (v.champion !== null && v.champion.round === floor ? v.champion : null),
    setBoss: () => {},
    champion: () => v.champion,
  };
}

/** Play a whole run the remote-backing way: open the runId, then each fight
 * against a per-fight view served through the run-scoped play read. */
async function playSharedRun(ctx: Ctx, token: string, seed: number, runId: string): Promise<RunState> {
  expect((await openRun(ctx, token, runId)).status).toBe(200);
  return playOpenedRun(ctx, token, seed, runId);
}

/** One ladder move against a per-fight view: at the champion's floor (the fixed
 * tower's top) challenge the boss; below it, climb the same-floor pool, or — when
 * the climb is refused because the floor has no opponent left — challenge the
 * floor's boss. Mirrors the kernel/CLI policy: stop at the top and challenge,
 * never climb PAST it into a vacant floor (that overshoots — no crown). */
function ladderMove(s: RunState, view: LadderStore): RunState {
  const top = view.champion();
  if (top !== null && s.round >= top.round) {
    return challengeBoss(s, view);
  }
  try {
    return ladderFight(s, view);
  } catch (err) {
    if (err instanceof InvalidDecisionError && err.decision === "fight") {
      return challengeBoss(s, view);
    }
    throw err;
  }
}

/** The play loop after the open — each fight against a fresh served view. */
async function playOpenedRun(ctx: Ctx, token: string, seed: number, runId: string): Promise<RunState> {
  let s = buy(initRun({ seed, runId, pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
  for (let guard = 0; s.status === "active"; guard++) {
    if (guard > 200) throw new Error(`run ${runId} did not terminate`);
    const round = s.round;
    s = ladderMove(s, viewOf(round, await fetchRunView(ctx, token, runId, round)));
  }
  return s;
}

/** The play loop of a client that ignores the contract: no open, no run-scoped
 * reads — public/exclude=me views only (its submission must be rejected). */
async function playUnopenedRun(ctx: Ctx, token: string, seed: number, runId: string): Promise<RunState> {
  let s = buy(initRun({ seed, runId, pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
  for (let guard = 0; s.status === "active"; guard++) {
    if (guard > 200) throw new Error(`run ${runId} did not terminate`);
    const pool = await fetchMyPool(ctx, token, s.round);
    const champ = (await fetchChampion(ctx)).champion!;
    const round = s.round;
    s = ladderMove(s, viewOf(round, { pool, champion: champ }));
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
    expect(pool.length).toBe(BASE); // floor-1 climb teams + the floor-1 boss-ghost (075-3)
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
      if (e["type"] === "BossChallenged") e["boss"] = "phantom";
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
    const local = openLadder(new InMemoryLadderStore(), stressRegistry, stressAbilities);
    let s = buy(initRun({ seed: 1, runId: "goliath", pool: [GOLIATH], statuses: stressRegistry, abilities: stressAbilities }), 0);
    while (s.status === "active") s = ladderMove(s, local);
    const { status, body } = await submit(ctx, ada, serializeRun(s));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/content/);
  });

  test("explicit-opponent fights don't belong on the shared ladder", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    expect((await openRun(ctx, ada, "offline")).status).toBe(200);
    let s = buy(initRun({ seed: 1, runId: "offline", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    for (let i = 0; i < STARTING_LIVES; i++) s = fight(s, [{ name: "Wall", base: { hp: 9999, pwr: 9999 }, ability: "Strike" }]);
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

    const active = buy(initRun({ seed: 9, runId: "wip", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
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
// Dev cheats are structurally unsubmittable (#066 slice 4): the in-run DEV
// panel's mutations (web/dev-ops.ts) touch the run state OUTSIDE the decision
// log, so the re-derivation cannot reproduce them and the submission bounces.
// This is the real "local-only" guarantee — the architecture, not the client's
// skip-submission flag. The flag is hygiene; THIS is the boundary.
// ---------------------------------------------------------------------------

describe("a dev-cheated run is rejected by re-derivation", () => {
  test("addGold inflates gold off the decision log — the replay diverges", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");

    // A wholly legit run, played the contract way — it would be accepted as-is.
    const honest = await playSharedRun(ctx, ada, 1, "titan-a");
    expect(honest).toMatchObject({ status: "over" });

    // The dev cheat: the same +gold the DEV panel applies, on the finished run.
    const cheated = addGold(honest, 1_000);
    expect(cheated.gold).toBe(honest.gold + 1_000); // the cheat took locally…

    const { status, body } = await submit(ctx, ada, serializeRun(cheated));
    expect(status).toBe(422); // …but the server re-derives gold from the log and rejects it
    expect(body).toMatchObject({ accepted: false });
    expect(body["reason"]).toMatch(/diverges/);
    // Nothing entered the shared ladder.
    expect((await fetchPublicPool(ctx, 1)).map((g) => g.runId)).not.toContain("titan-a");
  });

  test("spawnUnit fabricates a line unit the log never bought — rejected too", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const honest = await playSharedRun(ctx, ada, 1, "titan-b");

    const cheated = spawnUnit(honest, TITAN, "team");
    expect(cheated.team.length).toBe(honest.team.length + 1); // the spawned unit is there locally

    const { status, body } = await submit(ctx, ada, serializeRun(cheated));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/diverges/);
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
    // round-1's pool held 0 ghosts, so the climb is refused and the run
    // challengeBosses at round 1. On the fixed tower floor 1 is not the champion
    // floor, so that challenge overshoots locally — but the point is the CLAIM:
    // on a bootstrap-seeded ladder the round-1 pool is never smaller than BASE, so
    // the server never served (and would never serve) that empty view.
    const champ = (await fetchChampion(ctx)).champion!;
    let s = buy(initRun({ seed: 1, runId: "forged", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    while (s.status === "active") s = ladderMove(s, viewOf(s.round, { pool: [], champion: champ }));
    expect(ofType(s.log, "OpponentDrawn")).toEqual([]); // an empty pool: no climb opponent ever drawn — the impossible claim

    const { status, body } = await submit(ctx, ada, serializeRun(s));
    expect(status).toBe(422);
    // Rejected either way: the empty-pool prefix was never served, OR (since floor 1
    // is not the champion floor) the overshoot the empty view forced doesn't replay
    // against the real seeded pool. Both are the server refusing the impossible claim.
    expect(body["reason"]).toMatch(/never served|does not replay/);
    // Nothing landed: no ghost, and the bootstrap champion still holds the seat.
    expect((await fetchPublicPool(ctx, 1)).length).toBe(BASE);
    expect((await fetchChampion(ctx)).champion!.runId).toBe(BOOTSTRAP_RUN_ID);
  });

  test("claiming a shorter-than-served prefix to cherry-pick the opponent draw is rejected", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");

    // Ada's accepted run lands a ghost in every round she fought: Bob's
    // visible round-1 pool is now BASE+1 long.
    const runA = await playSharedRun(ctx, ada, 1, "titan-a");
    expect((await submit(ctx, ada, serializeRun(runA))).body).toMatchObject({ accepted: true });

    // Forge: Bob opens AFTER Ada's ghosts landed and reads his views through
    // the run-scoped play read (so serves exist), but plays against views
    // truncated to the bootstrap-only prefix — claiming the pool as it stood
    // BEFORE Ada, cherry-picking the weaker deterministic draw. The server
    // served him the longer pool: the shorter claim matches no serve.
    expect((await openRun(ctx, bob, "cherry")).status).toBe(200);
    expect((await fetchRunView(ctx, bob, "cherry", 1)).pool.length).toBe(BASE + 1);
    let s = buy(initRun({ seed: 2, runId: "cherry", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    for (let guard = 0; s.status === "active"; guard++) {
      if (guard > 200) throw new Error("forged run did not terminate");
      const round = s.round;
      const served = await fetchRunView(ctx, bob, "cherry", round);
      const truncated = served.pool.filter((g) => g.runId === BOOTSTRAP_RUN_ID); // bootstrap rows are the pool's prefix
      s = ladderMove(s, viewOf(round, { pool: truncated, champion: served.champion }));
    }

    const { status, body } = await submit(ctx, bob, serializeRun(s));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/never served/);
    expect((await fetchPublicPool(ctx, 1)).map((g) => g.runId)).not.toContain("cherry");
  });
});

// ---------------------------------------------------------------------------
// The banked-open forgery (Cass's refutation of the second slice-2 build):
// an open whose accepted-prefix floor is pinned at open time and never
// expires can be BANKED — opened at ladder genesis, cashed in after the
// ladder has grown, replaying against genesis views to dodge every
// post-genesis ghost and grab a free challenge against TODAY's champion.
// Serve-time pinning kills it: a challenge replays only against the champion
// co-served with the claimed view, and what a banked open was served is the
// genesis world — not the present one.
// ---------------------------------------------------------------------------

/** Eve's offline forge: greedy buying (outlevels the honest lone titans), each
 * fight against a banked view, the challenge against `champion`. The tower GROWS
 * now (slice 6): the live champion has ascended past the genesis top floor, so a
 * banked run that only ever read floors 1..genesisTop can never honestly REACH
 * the live champion's floor. The forgery instead PINS the claimed champion at the
 * genesis top floor (`genesisTop`) — claiming the live champion's team sits where
 * the bootstrap one did — so the run challenges there with a banked view. That is
 * the exact claim serve-time pinning must reject: a round-`genesisTop` view was
 * never co-served with the live champion. */
function forgeRun(
  runId: string,
  seed: number,
  bankedPools: ReadonlyMap<number, TeamSnapshot[]>,
  champion: TeamSnapshot,
  genesisTop: number,
): RunState {
  const claimedChampion: TeamSnapshot = { ...champion, round: genesisTop };
  let s = initRun({ seed, runId, pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities });
  for (let guard = 0; s.status === "active"; guard++) {
    if (guard > 50) throw new Error("forged run did not terminate");
    while (s.gold >= UNIT_COST && s.offers.length > 0) s = buy(s, 0);
    const round = s.round;
    s = ladderMove(s, viewOf(round, { pool: bankedPools.get(round) ?? [], champion: claimedChampion }));
  }
  return s;
}

describe("banked-open forgeries are rejected", () => {
  /** Genesis bank + ladder growth: eve opens `banked` and reads (= banks)
   * rounds 1..4 through the play read while the ladder is pristine; then five
   * honest users play and submit, and the crown leaves the bootstrap seat. */
  async function bankAndGrow(ctx: Ctx) {
    const eve = await login(ctx, "eve@example.com");
    expect((await openRun(ctx, eve, "banked")).status).toBe(200);
    const genesisPools = new Map<number, TeamSnapshot[]>();
    let genesisChampion: TeamSnapshot | undefined;
    for (let r = 1; r <= 4; r++) {
      const v = await fetchRunView(ctx, eve, "banked", r);
      genesisPools.set(r, v.pool);
      genesisChampion = v.champion;
    }
    expect(genesisPools.get(1)!.length).toBe(BASE);
    expect(genesisPools.get(4)!.length).toBe(BASE); // genesis round 4: the champion floor is seeded too (075-3 uniform tower)
    expect(genesisChampion!.runId).toBe(BOOTSTRAP_RUN_ID); // the world eve's bank was served

    for (let i = 1; i <= 5; i++) {
      const u = await login(ctx, `user${i}@example.com`);
      const run = await playSharedRun(ctx, u, i, `titan-${i}`);
      expect((await submit(ctx, u, serializeRun(run))).body).toMatchObject({ accepted: true });
    }
    const seated = (await fetchChampion(ctx)).champion!;
    expect(seated.runId).not.toBe(BOOTSTRAP_RUN_ID);
    expect((await fetchPublicPool(ctx, 1)).length).toBeGreaterThan(BASE); // round 1 grew…
    expect((await fetchPublicPool(ctx, 4)).length).toBeGreaterThan(BASE); // …and round 4 grew past its seeded size
    return { eve, genesisPools, genesisChampion: genesisChampion!, seated };
  }

  test("a genesis open cashed in after the ladder grew cannot crown over the current champion", async () => {
    const ctx = makeCtx();
    const { eve, genesisPools, seated } = await bankAndGrow(ctx);

    // Eve forges against her banked genesis views: rounds 1-3 fight only the
    // bootstrap ghosts (dodging every post-genesis ghost), round 4 claims the
    // genesis round-4 pool and challenges the champion seated NOW — PINNED at the
    // genesis top floor (TOWER_HEIGHT), since the live champion has since ascended
    // higher than a banked floors-1..4 run could honestly reach — whom her banked
    // round-4 view was never served with.
    const s = forgeRun("banked", 7, genesisPools, seated, TOWER_HEIGHT);
    expect(s).toMatchObject({ endedBy: "crown", round: TOWER_HEIGHT }); // locally: crowned at the genesis top floor over the current champion's team
    // Each banked round was claimed at its genesis pool length (all BASE on the
    // uniform tower), so every snapshot — climbs and the round-4 challenge — sits at BASE.
    expect(ofType(s.log, "Snapshotted").map((e) => e.seq)).toEqual([BASE, BASE, BASE, BASE]); // the banked claims
    expect(ofType(s.log, "BossChallenged")[0]!.boss).toBe(seated.runId);

    const { status, body } = await submit(ctx, eve, serializeRun(s));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/never served with that champion seated/);
    // Nothing landed: the seat held, no banked ghosts entered the pools.
    expect((await fetchChampion(ctx)).champion!.runId).toBe(seated.runId);
    expect((await fetchPublicPool(ctx, 4)).map((g) => g.runId)).not.toContain("banked");
  });

  test("control: the same forgery from a fresh open has no genesis serves and is rejected outright", async () => {
    const ctx = makeCtx();
    const { eve, genesisPools, seated } = await bankAndGrow(ctx);

    // A fresh open is served TODAY's views — the genesis prefixes it claims
    // were never served for it, so the very first fight fails the record.
    expect((await openRun(ctx, eve, "control")).status).toBe(200);
    expect((await fetchRunView(ctx, eve, "control", 1)).pool.length).toBeGreaterThan(BASE);
    const s = forgeRun("control", 7, genesisPools, seated, TOWER_HEIGHT);
    const { status, body } = await submit(ctx, eve, serializeRun(s));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/never served/);
  });

  test("a banked open can cash in only the world it was served: the co-served champion, whose crown lapses", async () => {
    const ctx = makeCtx();
    const { eve, genesisPools, genesisChampion, seated } = await bankAndGrow(ctx);

    // Eve plays her banked views coherently end to end: the round-4 challenge
    // goes against the champion CO-SERVED at genesis (the bootstrap). That is
    // indistinguishable from a slow honest run, so it must replay — but the
    // crown race applies: the bootstrap seat is long gone, the crown lapses,
    // and only the ghosts land. Banking buys nothing an honest slow run
    // would not also get.
    const s = forgeRun("banked", 7, genesisPools, genesisChampion, TOWER_HEIGHT);
    expect(s).toMatchObject({ endedBy: "crown", round: TOWER_HEIGHT });
    expect(ofType(s.log, "BossChallenged")[0]!.boss).toBe(BOOTSTRAP_RUN_ID);

    const { status, body } = await submit(ctx, eve, serializeRun(s));
    expect(status).toBe(200);
    expect(body).toMatchObject({ accepted: true, crowned: false }); // ghosts land, the crown does not
    expect((await fetchChampion(ctx)).champion!.runId).toBe(seated.runId); // the seat held
    expect((await fetchPublicPool(ctx, 1)).map((g) => g.runId)).toContain("banked");
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

  test("a stale-but-served prefix still replays: the pool grew mid-run", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");

    // Ada's run is played but NOT yet submitted when Bob opens and is served
    // his round-1 view — the bootstrap pool, which the server now has on record.
    const runA = await playSharedRun(ctx, ada, 1, "titan-a");
    expect((await openRun(ctx, bob, "stale")).status).toBe(200);
    const staleView = await fetchRunView(ctx, bob, "stale", 1);
    expect(staleView.pool.length).toBe(BASE);

    // Ada's submission lands her ghosts mid-Bob's-run…
    expect((await submit(ctx, ada, serializeRun(runA))).body).toMatchObject({ accepted: true, crowned: true });

    // …and Bob finishes against his stale (served) round-1 view + fresh later views.
    let s = buy(initRun({ seed: 2, runId: "stale", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    for (let guard = 0; s.status === "active"; guard++) {
      if (guard > 200) throw new Error("stale run did not terminate");
      const round = s.round;
      const v = round === 1 ? staleView : await fetchRunView(ctx, bob, "stale", round);
      s = ladderMove(s, viewOf(round, v));
    }

    const { status, body } = await submit(ctx, bob, serializeRun(s));
    expect(status).toBe(200); // round 1 claims the BASE-long prefix — exactly the view he was served
    expect(body).toMatchObject({ accepted: true });
  });

  test("a re-read round replays against either served view — refreshing never bricks a submission", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");

    // Bob is served round 1 twice: once at the bootstrap length, once after
    // Ada's ghosts landed — both views are on record now.
    expect((await openRun(ctx, bob, "reread")).status).toBe(200);
    const first = await fetchRunView(ctx, bob, "reread", 1);
    expect(first.pool.length).toBe(BASE);
    const runA = await playSharedRun(ctx, ada, 1, "titan-a");
    expect((await submit(ctx, ada, serializeRun(runA))).body).toMatchObject({ accepted: true });
    const second = await fetchRunView(ctx, bob, "reread", 1);
    expect(second.pool.length).toBe(BASE + 1);

    // Bob plays round 1 against the LATEST view (the refresh) and the rest fresh.
    let s = buy(initRun({ seed: 2, runId: "reread", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    for (let guard = 0; s.status === "active"; guard++) {
      if (guard > 200) throw new Error("reread run did not terminate");
      const round = s.round;
      const v = round === 1 ? second : await fetchRunView(ctx, bob, "reread", round);
      s = ladderMove(s, viewOf(round, v));
    }
    expect((await submit(ctx, bob, serializeRun(s))).body).toMatchObject({ accepted: true });
  });

  test("the play read is run-scoped: no bearer, foreign runId, unopened runId, junk round", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");
    expect((await openRun(ctx, ada, "mine")).status).toBe(200);

    expect((await ctx.app.request("/v1/runs/mine/pool/1")).status).toBe(401);
    const foreign = await ctx.app.request("/v1/runs/mine/pool/1", { headers: { authorization: `Bearer ${bob}` } });
    expect(foreign.status).toBe(422); // bob does not own it — same answer as a runId that is not open
    expect(((await foreign.json()) as Loose)["reason"]).toMatch(/not open for this user/);
    const unopened = await ctx.app.request("/v1/runs/ghost/pool/1", { headers: { authorization: `Bearer ${ada}` } });
    expect(unopened.status).toBe(422);
    const badRound = await ctx.app.request("/v1/runs/mine/pool/zero", { headers: { authorization: `Bearer ${ada}` } });
    expect(badRound.status).toBe(400);
  });
});

// ---------------------------------------------------------------------------
// Serve-route write-amplification guards (deployment hardening): every serve
// can record a run_pool_serves row, so the route is frontier-bounded and
// per-session rate-limited — neither bound is reachable by honest play.
// ---------------------------------------------------------------------------

describe("the play read is bounded", () => {
  test("serves past the ladder frontier are refused; rounds honest play can reach still serve", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    expect((await openRun(ctx, ada, "mine")).status).toBe(200);

    // Fresh ladder: ghosts reach TOWER_HEIGHT (the top floor is seeded too). One
    // round past it is still well within the serve margin, so honest reads serve fine.
    const justPast = await ctx.app.request(`/v1/runs/mine/pool/${TOWER_HEIGHT + 1}`, {
      headers: { authorization: `Bearer ${ada}` },
    });
    expect(justPast.status).toBe(200);

    // A round sweep beyond frontier + margin records nothing and is 422.
    const far = await ctx.app.request(`/v1/runs/mine/pool/${TOWER_HEIGHT + 17}`, {
      headers: { authorization: `Bearer ${ada}` },
    });
    expect(far.status).toBe(422);
    expect(((await far.json()) as Loose)["reason"]).toMatch(/beyond the ladder's frontier/);
  });

  test("serves are rate-limited per session; another session is unaffected", async () => {
    const ctx = makeCtx({ poolServeLimit: 3 });
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");
    expect((await openRun(ctx, ada, "mine")).status).toBe(200);
    expect((await openRun(ctx, bob, "his")).status).toBe(200);

    for (let i = 0; i < 3; i++) {
      const ok = await ctx.app.request("/v1/runs/mine/pool/1", { headers: { authorization: `Bearer ${ada}` } });
      expect(ok.status).toBe(200);
    }
    const limited = await ctx.app.request("/v1/runs/mine/pool/1", { headers: { authorization: `Bearer ${ada}` } });
    expect(limited.status).toBe(429);
    expect(((await limited.json()) as Loose)["retryAfterMs"]).toBeGreaterThan(0);

    // The limiter keys on the session — bob's play reads still serve.
    const other = await ctx.app.request("/v1/runs/his/pool/1", { headers: { authorization: `Bearer ${bob}` } });
    expect(other.status).toBe(200);
  });
});

// ---------------------------------------------------------------------------
// Open expiry — the supplemental bound on banking. Generously above an honest
// run's lifetime: slow play survives it, only a parked open dies of it.
// ---------------------------------------------------------------------------

describe("run opens expire", () => {
  const DAY = 24 * 60 * 60;

  test("an expired open neither serves nor submits", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    expect((await openRun(ctx, ada, "parked")).status).toBe(200);
    const view = await fetchRunView(ctx, ada, "parked", 1); // banked while fresh

    ctx.now.sec += RUN_OPEN_TTL_SECONDS + 1;

    // Serving is refused…
    const res = await ctx.app.request("/v1/runs/parked/pool/1", { headers: { authorization: `Bearer ${ada}` } });
    expect(res.status).toBe(422);
    expect(((await res.json()) as Loose)["reason"]).toMatch(/expire/);

    // …and so is submitting a run played against the banked pre-expiry view
    // (round 1 vs the banked pool, then empty rounds to the champion — the
    // rejection happens at the TTL gate, before any view is even checked).
    let s = buy(initRun({ seed: 1, runId: "parked", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    s = ladderMove(s, viewOf(1, view));
    for (let guard = 0; s.status === "active"; guard++) {
      if (guard > 20) throw new Error("parked run did not terminate");
      s = ladderMove(s, viewOf(s.round, { pool: [], champion: view.champion }));
    }
    const { status, body } = await submit(ctx, ada, serializeRun(s));
    expect(status).toBe(422);
    expect(body["reason"]).toMatch(/expire/);
  });

  test("an honest slow run resumed days later still replays", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    expect((await openRun(ctx, ada, "slow")).status).toBe(200);

    // Round 1 today, the rest after a two-day pause — well inside the TTL.
    let s = buy(initRun({ seed: 1, runId: "slow", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    s = ladderMove(s, viewOf(s.round, await fetchRunView(ctx, ada, "slow", s.round)));
    ctx.now.sec += 2 * DAY;
    for (let guard = 0; s.status === "active"; guard++) {
      if (guard > 200) throw new Error("slow run did not terminate");
      const round = s.round;
      s = ladderMove(s, viewOf(round, await fetchRunView(ctx, ada, "slow", round)));
    }

    const { status, body } = await submit(ctx, ada, serializeRun(s));
    expect(status).toBe(200);
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
