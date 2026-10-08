import { describe, expect, it } from "vitest";
import { MVP_RULES, type CandidateScore, type MyIdeasView, type PlayerRef, type VoteCard } from "../../../src/mvp/contract.js";
import { ROWS, type Row } from "../../../src/mvp/units.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { seedUnits } from "./pool.js";
import { mvpRuntime } from "./runtime.js";
import { MemoryMvpStore } from "./store.js";
import { candidateScores, nightDeadline, novelty, overnightCheck, qualified, typicalUnits, type Tuner } from "./votes.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const eva: PlayerRef = { id: "p2", name: "Eva", bot: false };

/** A tuner that passes (or fails) at once, its numbers bumped so the stored row shows it ran. */
const fakeTuner = (pass: boolean): Tuner => async (row) => ({ pass, reason: pass ? "fits" : "too strong in a damage team", row: { ...row, pwr: row.pwr + 1 } });

function world(tuner: Tuner = fakeTuner(true)) {
  let n = 7;
  const store = new MemoryMvpStore();
  const now = new Date("2026-10-08T08:00:00.000Z");
  seedUnits(store, now);
  const rt = mvpRuntime({ content: mvpContent(), store, seed: () => (n = (n * 1103515245 + 12345) >>> 0), now: () => now, dev: true, tuner: { night: tuner, dev: tuner } });
  for (const p of [maks, eva]) store.addPlayer(p);
  const app = createMvpApp(rt);
  const call = async <T>(method: string, path: string, player: PlayerRef | null, body?: unknown): Promise<{ status: number; json: T }> => {
    const headers: Record<string, string> = { "content-type": "application/json", ...(player ? { "X-Arena-Player": player.id } : {}) };
    const res = await app.request(`/api/v1${path}`, { method, headers, ...(body !== undefined ? { body: JSON.stringify(body) } : {}) });
    return { status: res.status, json: (await res.json()) as T };
  };
  return { rt, store, call };
}

/** Maks's dev candidate, through the overnight check. */
async function candidate(w: ReturnType<typeof world>) {
  const seeded = await w.call<MyIdeasView>("POST", "/dev/seed-candidate", maks);
  expect(seeded.json.sent[0]!.state).toBe("simulating");
  expect((await w.call<{ started: number }>("POST", "/dev/overnight-check", maks)).json).toEqual({ started: 1 });
  await overnightCheck(w.rt, w.rt.tuner.dev); // waits for the one the button started
  return w.store.ideas({ playerId: maks.id })[0]!;
}

describe("the overnight check (M2-8)", () => {
  it("moves a passing idea to voting, its tuned row stored as a candidate unit", async () => {
    const w = world();
    const idea = await candidate(w);
    expect(idea.state).toBe("voting");
    const unit = w.store.unit(idea.data.unitId!)!;
    expect(unit).toMatchObject({ status: "candidate", authorId: maks.id, origin: "idea" });
    // The tuned row replaced the picked one; the name stays.
    const from = ROWS.find((r) => unit.row.name.startsWith(`${r.name} Echo`))!;
    expect(unit.row.pwr).toBe(from.pwr + 1);
    expect(idea.data.checkedAt).toBe(w.rt.now().toISOString());
    expect(w.store.ideas({ state: "simulating" })).toEqual([]);
  });

  it("moves a failing idea to failed with the reason and refunds it", async () => {
    const w = world(fakeTuner(false));
    const before = w.store.ideaCounts(maks.id).spent;
    w.store.putIdeaCounts(maks.id, { ...w.store.ideaCounts(maks.id), spent: before + 1 });
    const idea = await candidate(w);
    expect(idea).toMatchObject({ state: "failed", data: { failure: "It didn't pass the overnight check: too strong in a damage team." } });
    expect(w.store.unit(idea.data.unitId!)?.status).toBe("rejected");
    expect(w.store.ideaCounts(maks.id).spent).toBe(before);
  });

  it("tunes against the DB's current pool, not the code's ROWS", async () => {
    const seen: Row[][] = [];
    const w = world(async (row, pool) => (seen.push(pool), { pass: true, reason: "fits", row }));
    // The pool in the DB drops its first unit (as a rotation would).
    const cur = w.store.currentPool()!;
    w.store.putPool({ ...cur, version: "mvp-test", unitIds: cur.unitIds.slice(1), createdAt: w.rt.now().toISOString() });
    await candidate(w);
    expect(seen).toHaveLength(1);
    expect(seen[0]!.map((r) => r.name)).toEqual(cur.unitIds.slice(1).map((id) => w.store.unit(id)!.row.name));
    expect(seen[0]!.length).toBe(ROWS.length - 1);
  });

  it("leaves an idea alone whose unit isn't a candidate any more", async () => {
    const w = world();
    await w.call("POST", "/dev/seed-candidate", maks);
    const idea = w.store.ideas({ playerId: maks.id })[0]!;
    w.store.putUnit({ ...w.store.unit(idea.data.unitId!)!, status: "rejected" });
    expect(await overnightCheck(w.rt, w.rt.tuner.dev)).toBe(0);
    expect(w.store.idea(idea.ideaId)?.state).toBe("simulating");
  });

  it("puts on cards only candidates whose idea is in voting (M2-5's simulating ones wait)", async () => {
    const w = world();
    await w.call("POST", "/dev/seed-candidate", maks);
    expect((await w.call<{ card: VoteCard | null }>("GET", "/votes/next", eva)).json.card).toBeNull();
    expect(candidateScores(w.rt)).toEqual([]);
  });

  it("leaves a unit cut off at the deadline in simulating for the next night", async () => {
    const hang: Tuner = (_row, _pool, signal) => new Promise((resolve) => signal.addEventListener("abort", () => resolve(null)));
    const w = world(hang);
    await w.call("POST", "/dev/seed-candidate", maks);
    const judged = await overnightCheck(w.rt, hang, new Date(w.rt.now().getTime() + 20));
    expect(judged).toBe(0);
    expect(w.store.ideas({ state: "simulating" })).toHaveLength(1);
  });

  it("runs at night only: from the day end for NIGHT_HOURS", () => {
    // 04:00 Moscow is 01:00Z.
    expect(nightDeadline(new Date("2026-10-09T00:59:00Z"), MVP_RULES)).toBeNull();
    expect(nightDeadline(new Date("2026-10-09T01:00:00Z"), MVP_RULES)?.toISOString()).toBe("2026-10-09T06:00:00.000Z");
    expect(nightDeadline(new Date("2026-10-09T05:59:00Z"), MVP_RULES)?.toISOString()).toBe("2026-10-09T06:00:00.000Z");
    expect(nightDeadline(new Date("2026-10-09T06:00:00Z"), MVP_RULES)).toBeNull();
    expect(nightDeadline(new Date("2026-10-09T12:00:00Z"), MVP_RULES)).toBeNull();
  });

  it("is a dev tool only", async () => {
    const w = world();
    const rt = mvpRuntime({ content: mvpContent(), store: w.store });
    const res = await createMvpApp(rt).request("/api/v1/dev/overnight-check", { method: "POST", headers: { "X-Arena-Player": maks.id } });
    expect(res.status).toBe(404);
  });
});

describe("votes (M2-8)", () => {
  it("never shows authors their own candidate; others get it against a typical live unit", async () => {
    const w = world();
    const idea = await candidate(w);
    expect((await w.call<{ card: VoteCard | null }>("GET", "/votes/next", maks)).json.card).toBeNull();
    const { card } = (await w.call<{ card: VoteCard | null }>("GET", "/votes/next", eva)).json;
    expect(card?.candidateId).toBe(idea.data.unitId);
    expect(card!.units.map((u) => u.id).sort()).toEqual([card!.candidateId, card!.otherId].sort());
    expect(w.rt.content.units.some((u) => u.id === card!.otherId)).toBe(true);
    // The candidate's sheet has what it needs.
    const cand = card!.units.find((u) => u.id === card!.candidateId)!;
    for (const a of cand.forms.awoken.does) expect(card!.pool.abilities[a]).toBeDefined();
    // Its own vote is refused too.
    expect((await w.call("POST", "/votes", maks, { candidateId: card!.candidateId, otherId: card!.otherId, pick: card!.candidateId })).status).toBe(409);
  });

  it("shows both orders over many cards", async () => {
    const w = world();
    await candidate(w);
    const firsts = new Set<boolean>();
    for (let i = 0; i < 20; i++) {
      const { card } = (await w.call<{ card: VoteCard | null }>("GET", "/votes/next", eva)).json;
      firsts.add(card!.units[0]!.id === card!.candidateId);
    }
    expect([...firsts].sort()).toEqual([false, true]);
  });

  it("takes one vote per player per pair, skips allowed, and never repeats a pair", async () => {
    const w = world();
    await candidate(w);
    const seen = new Set<string>();
    let { card } = (await w.call<{ card: VoteCard | null }>("GET", "/votes/next", eva)).json;
    for (let i = 0; card && i < 200; i++) {
      expect(seen.has(card.otherId)).toBe(false);
      seen.add(card.otherId);
      const vote = { candidateId: card.candidateId, otherId: card.otherId, pick: i % 3 === 0 ? null : card.candidateId };
      const res = await w.call<{ card: VoteCard | null }>("POST", "/votes", eva, vote);
      expect(res.status).toBe(200);
      expect((await w.call("POST", "/votes", eva, vote)).status).toBe(409);
      card = res.json.card;
    }
    expect(card).toBeNull();
    expect(seen.size).toBe(typicalUnits(w.rt).length);
    expect(w.store.votes({ playerId: eva.id }).filter((v) => v.pick === null).length).toBeGreaterThan(0);
  });

  it("refuses a pick of neither, an unknown pair and bots", async () => {
    const w = world();
    await candidate(w);
    const { card } = (await w.call<{ card: VoteCard }>("GET", "/votes/next", eva)).json;
    expect((await w.call("POST", "/votes", eva, { candidateId: card.candidateId, otherId: card.otherId, pick: "nope" })).status).toBe(400);
    expect((await w.call("POST", "/votes", eva, { candidateId: "nope", otherId: card.otherId, pick: null })).status).toBe(404);
    expect((await w.call("POST", "/votes", eva, { candidateId: card.candidateId })).status).toBe(400);
    expect((await w.call("POST", "/votes", null, { candidateId: card.candidateId, otherId: card.otherId, pick: null })).status).toBe(401);
    const bot: PlayerRef = { id: "b1", name: "Bot", bot: true };
    w.store.addPlayer(bot);
    expect((await w.call<{ card: VoteCard | null }>("GET", "/votes/next", bot)).json.card).toBeNull();
    expect((await w.call("POST", "/votes", bot, { candidateId: card.candidateId, otherId: card.otherId, pick: null })).status).toBe(409);
  });

  it("qualifies a candidate after 5 votes preferring it; the dev panel lists it first", async () => {
    const w = world();
    const idea = await candidate(w);
    const voters = Array.from({ length: 5 }, (_, i): PlayerRef => ({ id: `v${i}`, name: `V${i}`, bot: false }));
    for (const [i, v] of voters.entries()) {
      w.store.addPlayer(v);
      const { card } = (await w.call<{ card: VoteCard }>("GET", "/votes/next", v)).json;
      await w.call("POST", "/votes", v, { candidateId: card.candidateId, otherId: card.otherId, pick: card.candidateId });
      const scores = (await w.call<CandidateScore[]>("GET", "/dev/candidates", maks)).json;
      expect(scores[0]!.qualified).toBe(i === 4);
    }
    const scores = candidateScores(w.rt);
    expect(qualified(scores).map((s) => s.unitId)).toEqual([idea.data.unitId]);
    expect(scores[0]).toMatchObject({ votes: 5, won: 5, share: 1 });
  });

  it("dev fake votes: 5 a candidate, 4 for it, enough to qualify", async () => {
    const w = world();
    await candidate(w);
    const scores = (await w.call<CandidateScore[]>("POST", "/dev/fake-votes", maks)).json;
    expect(scores[0]).toMatchObject({ votes: 5, won: 4, share: 0.8, qualified: true });
  });

  it("doesn't qualify a candidate players prefer less, novelty or not", async () => {
    const w = world();
    const idea = await candidate(w);
    for (let i = 0; i < 6; i++) w.store.addVote({ playerId: `v${i}`, candidateId: idea.data.unitId!, otherId: "x", pick: i < 2 ? idea.data.unitId! : "x", createdAt: "" });
    const [s] = candidateScores(w.rt);
    expect(s).toMatchObject({ votes: 6, won: 2, qualified: false });
    expect(s!.score).toBeCloseTo(2 / 6 + 0.1 * s!.novelty, 3);
  });
});

describe("typical units and novelty (M2-8)", () => {
  it("picks the middle third by pick rate from the day tallies", () => {
    const w = world();
    const ids = w.rt.content.units.map((u) => u.id);
    expect(typicalUnits(w.rt)).toEqual(ids); // no tallies yet: every live unit
    const seq = w.rt.today().seq;
    w.store.addDayTallies(seq, { runs: 10, units: ids.map((unitId, i) => ({ unitId, fights: 0, wins: 0, runs: 0, picks: i })) });
    const typical = typicalUnits(w.rt);
    const third = Math.floor(ids.length / 3);
    expect(typical).toEqual(ids.slice(third, ids.length - third));
  });

  it("is high for parts the live pool rarely has, low for a copy of a common one", () => {
    const all = ROWS.map((r) => ({ r, n: novelty(r, ROWS) })).sort((a, b) => a.n - b.n);
    const [common, rare] = [all[0]!, all.at(-1)!];
    expect(rare.n).toBeGreaterThan(common.n + 0.2);
    expect(common.n).toBeGreaterThanOrEqual(0);
    expect(rare.n).toBeLessThanOrEqual(1);
    // The rare unit's When is shared by fewer live units than the common one's.
    const sharing = (r: Row) => ROWS.filter((x) => x.when === r.when).length;
    expect(sharing(rare.r)).toBeLessThanOrEqual(sharing(common.r));
  });
});
