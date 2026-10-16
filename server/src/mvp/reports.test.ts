// #1271: reports of a player's name or a player-made unit: stored, rate-limited, never hiding anything.
import { describe, expect, it } from "vitest";
import type { PlayerRef } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { poolContent, seedUnits } from "./pool.js";
import { REASON_MAX, REPORTS_PER_DAY } from "./reports.js";
import { mvpRuntime } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";

const NOW = new Date("2026-10-16T12:00:00Z");

function world(store: MvpStore) {
  let now = NOW;
  seedUnits(store, new Date("2026-10-08T08:00:00.000Z"));
  const rt = mvpRuntime({ content: poolContent(store), store, now: () => now });
  const app = createMvpApp(rt);
  const call = async (method: string, path: string, who?: PlayerRef, body?: unknown) => {
    const res = await app.request(`/api/v1${path}`, { method, headers: { "content-type": "application/json", ...(who ? { "X-Arena-Player": who.id } : {}) }, ...(body !== undefined ? { body: JSON.stringify(body) } : {}) });
    return { status: res.status, json: (await res.json()) as { ok?: true; error?: string } };
  };
  const player = (name: string, bot = false): PlayerRef => {
    const p = { id: `id-${name}`, name, bot };
    store.addPlayer(p);
    return p;
  };
  const maker = player("Maker");
  store.putUnit({ unitId: "hog", status: "live", row: { ...store.unit("rat")!.row, name: "Hog" }, authorId: maker.id, origin: "idea", parentId: null, createdAt: NOW.toISOString() });
  return { rt, store, call, player, maker, later: (ms: number) => (now = new Date(now.getTime() + ms)) };
}

describe.each([
  ["memory", () => new MemoryMvpStore()],
  ["sqlite", () => new SqliteMvpStore(":memory:")],
])("reports (#1271), %s store", (_name, make) => {
  it("stores a report of a name and of a player-made unit, with the reason, for the admin to list", async () => {
    const w = world(make());
    const kim = w.player("Kim");
    expect(await w.call("POST", "/reports", kim, { kind: "name", id: w.maker.id, reason: "  rude\n name " })).toEqual({ status: 200, json: { ok: true } });
    expect((await w.call("POST", "/reports", kim, { kind: "unit", id: "hog" })).status).toBe(200);
    const list = w.store.reports({ open: true });
    expect(list.map((r) => [r.reporterId, r.kind, r.targetId, r.reason ?? null, r.status, r.at])).toEqual([
      [kim.id, "name", w.maker.id, "rude name", "open", NOW.toISOString()],
      [kim.id, "unit", "hog", null, "open", NOW.toISOString()],
    ]);
  });

  it("hides nothing: the reported player and unit stay as they were", async () => {
    const w = world(make());
    const kim = w.player("Kim");
    await w.call("POST", "/reports", kim, { kind: "name", id: w.maker.id });
    await w.call("POST", "/reports", kim, { kind: "unit", id: "hog" });
    expect(w.store.player(w.maker.id)).toEqual(w.maker);
    expect(w.store.unit("hog")?.status).toBe("live");
  });

  it("takes the same report from one player once, and keeps the reason short", async () => {
    const w = world(make());
    const kim = w.player("Kim");
    await w.call("POST", "/reports", kim, { kind: "name", id: w.maker.id, reason: "x".repeat(REASON_MAX + 50) });
    expect((await w.call("POST", "/reports", kim, { kind: "name", id: w.maker.id, reason: "again" })).status).toBe(200);
    const list = w.store.reports();
    expect(list).toHaveLength(1);
    expect(list[0]!.reason).toHaveLength(REASON_MAX);
    // Another player's report of the same name is a second one.
    await w.call("POST", "/reports", w.player("Lee"), { kind: "name", id: w.maker.id });
    expect(w.store.reports()).toHaveLength(2);
  });

  it("refuses an unknown target, a bot, a seed unit, a bad body and an unknown player", async () => {
    const w = world(make());
    const kim = w.player("Kim");
    const bot = w.player("bot-Ash", true);
    const refused = async (body: unknown, status: number, who: PlayerRef | null = kim) => expect((await w.call("POST", "/reports", who ?? undefined, body)).status).toBe(status);
    await refused({ kind: "name", id: "nobody" }, 404);
    await refused({ kind: "name", id: bot.id }, 404);
    await refused({ kind: "unit", id: "rat" }, 404); // a seed unit has no player behind it
    await refused({ kind: "unit", id: "nope" }, 404);
    await refused({ kind: "idea", id: "hog" }, 400);
    await refused({ kind: "name" }, 400);
    await refused({ kind: "name", id: w.maker.id, reason: 5 }, 400);
    await refused({ kind: "name", id: w.maker.id }, 401, null);
    await refused({ kind: "name", id: w.maker.id }, 401, { id: "ghost", name: "Ghost", bot: false });
    expect(w.store.reports()).toEqual([]);
  });

  it(`allows ${REPORTS_PER_DAY} new reports a player in a day, counted from the last 24 hours`, async () => {
    const w = world(make());
    const kim = w.player("Kim");
    for (let i = 0; i < REPORTS_PER_DAY; i++) expect((await w.call("POST", "/reports", kim, { kind: "name", id: w.player(`Victim${i}`).id })).status).toBe(200);
    const extra = w.player("Extra");
    expect((await w.call("POST", "/reports", kim, { kind: "name", id: extra.id })).status).toBe(429);
    // The limit is Kim's own, and a report already filed still answers ok.
    expect((await w.call("POST", "/reports", w.player("Lee"), { kind: "name", id: extra.id })).status).toBe(200);
    expect((await w.call("POST", "/reports", kim, { kind: "name", id: "id-Victim0" })).status).toBe(200);
    expect(w.store.reports().filter((r) => r.reporterId === kim.id)).toHaveLength(REPORTS_PER_DAY);
    w.later(24 * 3_600_000 + 1000);
    expect((await w.call("POST", "/reports", kim, { kind: "name", id: extra.id })).status).toBe(200);
  });
});
