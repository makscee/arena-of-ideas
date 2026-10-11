// #958 (mission #955, slice 2): practice runs and the assist-only rule. A
// practice run plays like a ranked one and counts for nothing: no rating, no
// Slay, no ghost, no stats, no fusion credit. An agent client
// (X-Arena-Client: agent/<name>) reads, previews, logs in and plays practice
// runs; every other write answers 403.
import { describe, expect, it } from "vitest";
import { botDecision } from "../../../clients/core/bot.js";
import type { Champion, DecisionResponse, HomeView, PlayerRef, RunView } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
import { AGENT_REFUSED, createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { FakeMailer } from "./email.js";
import { portmanteau } from "./fusions.js";
import { abandon, decide, preview, startRun, type RunHooks } from "./runs.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";
import { FakeTelegram } from "./telegram.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const botP: PlayerRef = { id: "b1", name: "bot-Ash", bot: true };
const AGENT = { "X-Arena-Client": "agent/test" };

function world(extra: Partial<MvpDeps> = {}) {
  let n = 7;
  const seen: string[] = [];
  const spy: RunHooks = { onDecision: () => seen.push("decision"), onFuse: () => seen.push("fuse"), onFight: () => seen.push("fight"), onRunEnd: () => seen.push("end") };
  const rt = mvpRuntime({ content: mvpContent(), seed: () => (n = (n * 1103515245 + 12345) >>> 0), hooks: [spy], ...extra });
  rt.store.addPlayer(maks);
  return { rt, seen };
}

/** Everything a run can feed besides itself: ghosts, Slays, ratings, stats, fusions. */
function derived(rt: ReturnType<typeof world>["rt"]) {
  const seq = rt.today().seq;
  return {
    ghosts: Array.from({ length: rt.rules.rounds }, (_, i) => rt.store.ghosts(i + 1, { excludePlayerId: "nobody" })).flat(),
    slays: rt.store.slays(seq),
    rating: rt.store.rating(maks.id),
    day: rt.store.dayTallies(seq),
    units: rt.store.unitTallies(rt.content.version),
    fusions: rt.store.fusions(),
  };
}

/** A champion that can't win: one 1 HP unit. */
function weakChampion(rt: ReturnType<typeof world>["rt"]): Champion {
  const u = rt.content.units[0]!;
  const c: Champion = { seq: rt.today().seq, day: rt.today().day, player: botP, line: [{ ...lineUnitOf(u, "c1"), stats: { pwr: 0, hp: 1 } }], since: "2026-10-05T00:00:00.000Z", contentVersion: rt.content.version };
  rt.store.putChampion(c);
  return c;
}

/** A practice run at its last shop round with the content's five biggest units, awoken. */
function practiceAt12(rt: ReturnType<typeof world>["rt"]): MvpRunState {
  const run = startRun(rt, maks, "practice");
  const big = [...rt.content.units].sort((a, b) => b.base.hp + b.base.pwr - (a.base.hp + a.base.pwr)).slice(0, 5);
  const at12: MvpRunState = { ...run, round: rt.rules.rounds, line: big.map((u, i) => lineUnitOf(u, `u${i + 1}`, 3)), nextUid: 6 };
  rt.store.putRun(at12);
  return at12;
}

describe("practice runs (#958)", () => {
  it("a player has one ranked and one practice run going; each start hands back its own", () => {
    const { rt } = world();
    const ranked = startRun(rt, maks);
    const practice = startRun(rt, maks, "practice");
    expect(ranked.mode).toBeUndefined();
    expect(practice.mode).toBe("practice");
    expect(practice.runId).not.toBe(ranked.runId);
    expect(startRun(rt, maks).runId).toBe(ranked.runId);
    expect(startRun(rt, maks, "practice").runId).toBe(practice.runId);
  });

  it("plays to the end and feeds nothing: no ghost, rating, stats, Slay or hook", () => {
    const { rt, seen } = world();
    const before = derived(rt);
    let run = startRun(rt, maks, "practice");
    for (let guard = 0; run.phase !== "over" && guard < 1000; guard++) {
      decide(rt, run, botDecision(run, rt.content, rt.rules, 9, () => true));
      run = rt.store.run(run.runId)!;
    }
    expect(run.phase).toBe("over");
    expect(run.fights.length).toBeGreaterThan(0);
    expect(run.rating).toBeNull();
    // Its battles are stored: a replay works.
    expect(rt.store.battle(run.fights[0]!.battleId)).toBeTruthy();
    expect(derived(rt)).toEqual(before);
    expect(seen).toEqual([]);
  });

  it("a ranked run still feeds them (the same play, for contrast)", () => {
    const { rt, seen } = world();
    const run = startRun(rt, maks);
    decide(rt, run, { kind: "buy", slot: 0 });
    decide(rt, rt.store.run(run.runId)!, { kind: "fight" });
    expect(derived(rt).ghosts).toHaveLength(1);
    expect(seen).toContain("fight");
  });

  it("a Crown win is no Slay and moves no rating", () => {
    const { rt, seen } = world();
    const champ = weakChampion(rt);
    const run = practiceAt12(rt);
    decide(rt, run, { kind: "fight" });
    const crown = rt.store.run(run.runId)!;
    expect(crown).toMatchObject({ phase: "crown", crownSeq: champ.seq });
    const won: DecisionResponse = decide(rt, crown, { kind: "fight" });
    expect(won.fight).toMatchObject({ kind: "crown", outcome: "win" });
    expect(won.run).toMatchObject({ phase: "over", endedBy: "crown-won", rating: null, mode: "practice" });
    expect(rt.store.slays(champ.seq)).toEqual([]);
    expect(rt.store.rating(maks.id)).toBeUndefined();
    expect(seen).toEqual([]);
  });

  it("abandoning moves no rating and calls no hook", () => {
    const { rt, seen } = world();
    const ended = abandon(rt, startRun(rt, maks, "practice"));
    expect(ended).toMatchObject({ phase: "over", endedBy: "abandoned", rating: null });
    expect(rt.store.rating(maks.id)).toBeUndefined();
    expect(seen).toEqual([]);
  });

  it("a fuse credits no one and records nothing; a stored pair shows its name, a bots' pair stays unclaimed", () => {
    const { rt } = world();
    const [a, b, c] = rt.content.units;
    const run = startRun(rt, maks, "practice");
    rt.store.putRun({ ...run, line: [lineUnitOf(a!, "u1", 3, rt.rules), lineUnitOf(b!, "u2", 3, rt.rules), lineUnitOf(c!, "u3", 3, rt.rules)], nextUid: 4 });
    const bots = portmanteau(a!.name, b!.name);
    rt.store.putFusion({ first: a!.id, second: b!.id, name: bots, discoveredBy: null, discoveredAt: "2026-10-06T00:00:00.000Z", nameSource: "fallback" });
    const stored = rt.store.fusions();

    // A new pair: a provisional name, no credit, nothing stored.
    expect(preview(rt, rt.store.run(run.runId)!, { kind: "fuse", first: 2, second: 1 }).run.line.find((u) => u.kind === "fused")).toMatchObject({ name: "", fusion: { discoveredBy: null } });
    const fresh = decide(rt, rt.store.run(run.runId)!, { kind: "fuse", first: 2, second: 1 }).run.line.find((u) => u.kind === "fused")!;
    expect(fresh.fusion).toMatchObject({ first: c!.id, second: b!.id, discoveredBy: null });
    expect(fresh.name).toBeTruthy();
    expect(rt.store.fusions()).toEqual(stored);

    // The bots' pair: its stored name, still no one's.
    const run2 = startRun(rt, { id: "p2", name: "Eva", bot: false }, "practice");
    rt.store.putRun({ ...run2, line: [lineUnitOf(a!, "u1", 3, rt.rules), lineUnitOf(b!, "u2", 3, rt.rules)], nextUid: 3 });
    const known = decide(rt, rt.store.run(run2.runId)!, { kind: "fuse", first: 0, second: 1 }).run.line.find((u) => u.kind === "fused");
    expect(known).toMatchObject({ name: bots, fusion: { name: bots, discoveredBy: null } });
    expect(rt.store.fusions()).toEqual(stored);
  });
});

describe("the assist-only rule over HTTP (#958)", () => {
  function api(extra: Partial<MvpDeps> = {}) {
    const mail = new FakeMailer(false);
    const { rt } = world({ dev: true, telegram: new FakeTelegram(), mail, ...extra });
    const app = createMvpApp(rt);
    const call = async <T = unknown>(method: string, path: string, headers: Record<string, string> = {}, body?: unknown) => {
      const res = await app.request(`/api/v1${path}`, { method, headers: { "content-type": "application/json", ...headers }, ...(body !== undefined ? { body: JSON.stringify(body) } : {}) });
      return { status: res.status, json: (await res.json().catch(() => null)) as T };
    };
    return { rt, call };
  }
  const as = (p: PlayerRef, agent = false) => ({ "X-Arena-Player": p.id, ...(agent ? AGENT : {}) });

  it("an agent starts and plays a practice run to the end; home shows it beside the ranked run", async () => {
    const { rt, call } = api();
    const ranked = await call<RunView>("POST", "/runs", as(maks));
    expect(ranked.status).toBe(200);
    expect(ranked.json.mode).toBeUndefined();
    const started = await call<RunView>("POST", "/runs", as(maks, true), { mode: "practice" });
    expect(started.status).toBe(200);
    expect(started.json.mode).toBe("practice");
    const home = await call<HomeView>("GET", "/home", as(maks, true));
    expect(home.json).toMatchObject({ activeRunId: ranked.json.runId, practiceRunId: started.json.runId, rating: { rating: 1000, runs: 0 } });

    const before = derived(rt);
    let run = started.json;
    for (let guard = 0; run.phase !== "over" && guard < 1000; guard++) {
      const d = botDecision(rt.store.run(run.runId)!, rt.content, rt.rules, 9, () => true);
      const res = await call<DecisionResponse>("POST", `/runs/${run.runId}/decisions`, as(maks, true), d);
      expect(res.status).toBe(200);
      run = res.json.run;
    }
    expect(run).toMatchObject({ phase: "over", rating: null });
    expect(derived(rt)).toEqual(before);
    // A replay of its battles is a read.
    expect((await call("GET", `/battles/${run.fights[0]!.battleId}`, as(maks, true))).status).toBe(200);
    expect((await call<HomeView>("GET", "/home", as(maks, true))).json).toMatchObject({ activeRunId: ranked.json.runId, practiceRunId: null, rating: { rating: 1000, runs: 0 } });
  });

  it("answers an agent 403 on ranked start, decide and abandon; reads and previews still work", async () => {
    const { call } = api();
    expect(await call("POST", "/runs", as(maks, true))).toEqual({ status: 403, json: { error: AGENT_REFUSED.ranked } });
    expect((await call("POST", "/runs", as(maks, true), { mode: "ranked" })).status).toBe(403);
    expect((await call("POST", "/runs", as(maks, true), { mode: "solo" })).status).toBe(400);
    const ranked = (await call<RunView>("POST", "/runs", as(maks))).json;
    expect(await call("POST", `/runs/${ranked.runId}/decisions`, as(maks, true), { kind: "buy", slot: 0 })).toEqual({ status: 403, json: { error: AGENT_REFUSED.ranked } });
    expect((await call("POST", `/runs/${ranked.runId}/abandon`, as(maks, true))).status).toBe(403);
    expect((await call("GET", `/runs/${ranked.runId}`, as(maks, true))).status).toBe(200);
    expect((await call("POST", `/runs/${ranked.runId}/preview`, as(maks, true), { kind: "buy", slot: 0 })).status).toBe(200);
    // The human's tap goes through.
    expect((await call("POST", `/runs/${ranked.runId}/decisions`, as(maks), { kind: "buy", slot: 0 })).status).toBe(200);
    // Without the header nothing changes: a ranked start, any value but agent/….
    expect((await call("POST", "/runs", { ...as(maks), "X-Arena-Client": "phone" })).status).toBe(200);
  });

  it("answers an agent 403 on idea writes, votes, linking a login and the dev tools", async () => {
    const { call } = api();
    const writes: [string, unknown?][] = [
      ["/ideas", { text: "A knight who guards the weakest ally" }],
      ["/ideas/i1/cancel"],
      ["/ideas/i1/archetype", { index: 0 }],
      ["/ideas/i1/reading", { index: 0 }],
      ["/ideas/i1/none"],
      ["/votes", { candidateId: "c", otherId: "o", pick: null }],
      ["/auth/email/attach", { email: "maks@example.com", code: "000000" }],
      ["/auth/telegram/unlink"],
      ["/auth/telegram/start"],
      ["/dev/end-day"],
      ["/dev/grant-idea"],
      ["/dev/fake-votes"],
      ["/dev/seed-candidate", {}],
      ["/dev/telegram/accept", { code: "x" }],
    ];
    for (const [path, body] of writes) {
      const res = await call("POST", path, as(maks, true), body);
      expect(res.status, path).toBe(403);
      expect(Object.values(AGENT_REFUSED), path).toContain((res.json as { error: string }).error);
    }
    // Reads stay open to it.
    for (const path of ["/ideas", "/votes/next", "/day", "/stats", "/fusions", "/credits", "/library", "/content", "/auth/email", "/auth/telegram"])
      expect((await call("GET", path, as(maks, true))).status, path).toBe(200);
    // The same writes from the human still reach their routes.
    expect((await call("POST", "/dev/grant-idea", as(maks))).status).toBe(200);
    expect((await call("POST", "/auth/telegram/start", as(maks))).status).toBe(200);
  });

  it("lets an agent log in as today: a device, an email code, a Telegram login without a session", async () => {
    const { call } = api();
    expect((await call<PlayerRef>("POST", "/players", AGENT, { name: "Coach" })).status).toBe(200);
    expect((await call("POST", "/auth/email/start", AGENT, { email: "eva@example.com", name: "Eva", lang: "en" })).status).toBe(200);
    const { code } = (await call<{ code: string }>("POST", "/dev/mail/last", AGENT, { email: "eva@example.com" })).json;
    const done = await call<{ status: string; token: string }>("POST", "/auth/email/verify", AGENT, { email: "eva@example.com", code, name: "Eva" });
    expect(done.json.status).toBe("done");
    const practice = await call<RunView>("POST", "/runs", { "X-Arena-Token": done.json.token, ...AGENT }, { mode: "practice" });
    expect(practice.json.mode).toBe("practice");
    expect((await call("POST", "/auth/telegram/start", AGENT)).status).toBe(200);
  });
});
