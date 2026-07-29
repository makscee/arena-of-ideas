// Remote ladder tests — the shared-ladder store behind the kernel's
// LadderStore seam, driven through a stub ArenaApi (no network, the run-store
// storage pattern). The bar that matters: a ladderFight against this store
// reads EXACTLY the served view — pool and champion from one serve response —
// and logs the Snapshotted seq the server recorded, because submission replay
// accepts nothing else (server/README.md, serve-time pinning).

import { describe, expect, test } from "vitest";
import { initRun, buy, challengeBoss, ladderFight, stressAbilities, stressRegistry, type TeamSnapshot, type UnitDef } from "../src/index.js";
import type { ArenaApi, ApiResult } from "./api.js";
import { RemoteLadder } from "./remote-ladder.js";

const TITAN: UnitDef = { name: "Titan", base: { hp: 100, pwr: 50 }, ability: "Strike" };
const WISP: UnitDef = { name: "Wisp", base: { hp: 1, pwr: 1 }, ability: "Strike" };

const snap = (runId: string, round: number, seq: number, team: UnitDef[]): TeamSnapshot => ({
  runId,
  round,
  seq,
  team,
});

/** An ArenaApi where every method fails loudly unless overridden — a test
 * touching an endpoint it didn't stub is a test asking the wrong question. */
function stubApi(overrides: Partial<ArenaApi>): ArenaApi {
  const die = (name: string) => () => {
    throw new Error(`unexpected api call: ${name}`);
  };
  return {
    startLogin: die("startLogin"),
    verifyLogin: die("verifyLogin"),
    me: die("me"),
    logout: die("logout"),
    setDisplayName: die("setDisplayName"),
    champion: die("champion"),
    pool: die("pool"),
    openRun: die("openRun"),
    servePool: die("servePool"),
    submitRun: die("submitRun"),
    ...overrides,
  } as ArenaApi;
}

const ok = <T,>(value: T): ApiResult<T> => ({ ok: true, value });

describe("the fight contract (serve-time pinning)", () => {
  test("ladderFight draws from the served pool and claims the served seq", async () => {
    const ghost = snap("rival", 1, 0, [WISP]);
    const champ = snap("queen", 4, 0, [TITAN]);
    const store = new RemoteLadder(
      stubApi({
        servePool: async () => ok({ round: 1, pool: [ghost], champion: champ }),
      }),
      "token",
    );
    expect((await store.serve("web-x", 1)).ok).toBe(true);

    let s = buy(initRun({ seed: 7, runId: "web-x", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    s = ladderFight(s, store);

    // The own ghost enters at the SERVED prefix length — what the server
    // recorded and what replay will demand of Snapshotted.seq.
    const snapshotted = s.log.find((e) => e.type === "Snapshotted");
    expect(snapshotted).toMatchObject({ seq: 1 });
    const drawn = s.log.find((e) => e.type === "OpponentDrawn");
    expect(drawn).toMatchObject({ opponent: "rival", seq: 0, candidates: 1 });
    // The fight view grew by the own ghost — the shop's "rivals" read sees it.
    expect(store.poolAt(1)).toHaveLength(2);
    expect(store.poolAt(1)[1]!.runId).toBe("web-x");
  });

  test("a served lower-floor boss stays distinct from the co-served champion", async () => {
    const lower = snap("lower", 1, 0, [WISP]);
    const champion = snap("queen", 4, 0, [TITAN]);
    const store = new RemoteLadder(
      stubApi({
        servePool: async () => ok({ round: 1, pool: [], boss: lower, champion }),
      }),
      "token",
    );
    expect((await store.serve("web-x", 1)).ok).toBe(true);

    const state = challengeBoss(
      buy(initRun({ seed: 7, runId: "web-x", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0),
      store,
    );

    expect(state.log.find((e) => e.type === "BossChallenged")).toMatchObject({ boss: "lower" });
    expect(state.endedBy).toBe("seated");
    expect(store.bossAt(1)).toMatchObject({ runId: "web-x", round: 1 });
    expect(store.champion()).toEqual(champion);
  });

  test("an empty served pool challenges the CO-SERVED champion, never a fresher read", async () => {
    // The boss of the floor being challenged (floor 1) carries that floor as its
    // round — challengeBoss reads ladder.bossAt(s.round), so the seat's round
    // must match the challenged floor.
    const servedChamp = snap("old-queen", 1, 0, [WISP]);
    const store = new RemoteLadder(
      stubApi({
        servePool: async () => ok({ round: 1, pool: [], champion: servedChamp }),
        // Display sync sees a NEWER champion — the fight must ignore it.
        champion: async () => ok({ champion: snap("new-queen", 5, 0, [TITAN]), holder: "Ada" }),
        pool: async () => ok({ round: 1, pool: [] as TeamSnapshot[] }),
      }),
      "token",
    );
    await store.sync();
    expect((await store.serve("web-x", 1)).ok).toBe(true);

    let s = buy(initRun({ seed: 7, runId: "web-x", pool: [TITAN], statuses: stressRegistry, abilities: stressAbilities }), 0);
    // An empty served pool: there is no climb opponent, so the move is to
    // challenge the floor's boss — the co-served champion, never a fresher read.
    s = challengeBoss(s, store);

    expect(s.log.find((e) => e.type === "BossChallenged")).toMatchObject({ boss: "old-queen" });
    expect(s.log.find((e) => e.type === "Snapshotted")).toMatchObject({ seq: 0 });
    // Titan beats the wisp champion: crowned — and the crown stays a LOCAL
    // display fact (the server decides the real one at submit).
    expect(s.endedBy).toBe("crown");
  });
});

describe("display reads (sync + overlay)", () => {
  test("sync fills every public floor seat and derives the same summit", async () => {
    const lower = snap("lower", 2, 0, [WISP]);
    const champion = snap("queen", 5, 0, [TITAN]);
    const store = new RemoteLadder(
      stubApi({
        champion: async () => ok({ champion, holder: "Ada", bosses: { "2": lower, "5": champion } }),
        pool: async (round: number) => ok({ round, pool: [] as TeamSnapshot[] }),
      }),
      "token",
    );

    expect(await store.sync()).toEqual({ ok: true });
    expect(store.bossAt(2)).toEqual(lower);
    expect(store.bossAt(4)).toBeNull();
    expect(store.bossAt(5)).toEqual(champion);
    expect(store.champion()).toEqual(champion);
  });

  test("a local overwrite of the synced summit immediately becomes champion", async () => {
    const old = snap("old-top", 5, 0, [WISP]);
    const replacement = snap("new-top", 5, 1, [TITAN]);
    const store = new RemoteLadder(
      stubApi({
        champion: async () => ok({ champion: old, holder: "Ada", bosses: { "5": old } }),
        pool: async (round: number) => ok({ round, pool: [] as TeamSnapshot[] }),
      }),
      "token",
    );
    await store.sync();

    store.setBoss(5, replacement);

    expect(store.bossAt(5)).toEqual(replacement);
    expect(store.champion()).toEqual(replacement);
  });

  test("sync fills public pools and the holder; own unsubmitted ghosts overlay", async () => {
    const pub = snap("rival", 1, 0, [WISP]);
    const store = new RemoteLadder(
      stubApi({
        champion: async () => ok({ champion: snap("queen", 4, 0, [TITAN]), holder: "Ada" }),
        pool: async (round: number) => ok({ round, pool: round === 1 ? [pub] : [] }),
      }),
      "token",
    );
    expect(await store.sync()).toEqual({ ok: true });
    expect(store.holder()).toBe("Ada");
    expect(store.champion()?.runId).toBe("queen");
    expect(store.poolAt(1)).toEqual([pub]);

    // A ghost the run snapshotted but the server hasn't re-derived yet still
    // shows — and clearLocal drops it (post-submit, the synced copy takes over).
    store.addSnapshot(snap("web-x", 2, 0, [TITAN]));
    expect(store.poolAt(2)).toHaveLength(1);
    store.clearLocal();
    expect(store.poolAt(2)).toHaveLength(0);
    expect(store.poolAt(1)).toEqual([pub]); // public state survives clearLocal
  });

  test("a failed sync leaves the previous display standing — stale beats blank", async () => {
    let dead = false;
    const store = new RemoteLadder(
      stubApi({
        champion: async () =>
          dead
            ? ({ ok: false, kind: "network", reason: "down" } as ApiResult<never>)
            : ok({ champion: snap("queen", 4, 0, [TITAN]), holder: null }),
        pool: async (round: number) => ok({ round, pool: [] as TeamSnapshot[] }),
      }),
      "token",
    );
    await store.sync();
    dead = true;
    const second = await store.sync();
    expect(second.ok).toBe(false);
    expect(store.champion()?.runId).toBe("queen");
  });
});

describe("the run gateway", () => {
  test("mintRunId: globally unique shape, inside the server's 1–128 bound", () => {
    const store = new RemoteLadder(stubApi({}), "token");
    const a = store.mintRunId();
    const b = store.mintRunId();
    expect(a).not.toBe(b);
    expect(a).toMatch(/^web-[0-9a-f-]{36}$/);
    expect(a.length).toBeLessThanOrEqual(128);
  });

  test("open clears the previous run's local view", async () => {
    const store = new RemoteLadder(
      stubApi({
        openRun: async () => ok({ opened: true as const, runId: "web-y" }),
        servePool: async () => ok({ round: 1, pool: [], champion: snap("queen", 4, 0, [WISP]) }),
      }),
      "token",
    );
    await store.serve("web-x", 1);
    store.addSnapshot(snap("web-x", 1, 0, [TITAN]));
    expect(store.poolAt(1)).toHaveLength(1);
    expect((await store.open("web-y")).ok).toBe(true);
    expect(store.poolAt(1)).toHaveLength(0);
    expect(store.champion()).toBeNull(); // no display sync yet, no fight view
  });

  test("failures map to player-shaped reasons, never raw transport", async () => {
    const store = new RemoteLadder(
      stubApi({
        servePool: async () => ({ ok: false, kind: "network", reason: "ECONNREFUSED" }) as ApiResult<never>,
        openRun: async () => ({ ok: false, kind: "unauthorized" }) as ApiResult<never>,
        submitRun: async () =>
          ({ ok: false, kind: "rejected", status: 422, reason: "run does not replay" }) as ApiResult<never>,
      }),
      "token",
    );
    const serve = await store.serve("web-x", 1);
    expect(serve).toMatchObject({ ok: false, reason: expect.stringContaining("unreachable") });
    const open = await store.open("web-x");
    expect(open).toMatchObject({ ok: false, reason: expect.stringContaining("log in") });
    const submit = await store.submit("{}");
    expect(submit).toEqual({ ok: false, kind: "rejected", reason: "run does not replay" });
  });

  test("an accepted submit reports the crown verdict and drops the local view", async () => {
    const store = new RemoteLadder(
      stubApi({
        submitRun: async () => ok({ runId: "web-x", endedBy: "crown", finalRound: 5, crowned: false }),
        servePool: async () => ok({ round: 1, pool: [], champion: snap("queen", 4, 0, [WISP]) }),
      }),
      "token",
    );
    await store.serve("web-x", 1);
    const res = await store.submit("…serialized…");
    expect(res).toEqual({ ok: true, crowned: false }); // the crown race, surfaced honestly
    expect(store.champion()).toBeNull(); // fight view gone with the submit
  });
});
