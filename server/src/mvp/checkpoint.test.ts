// #1065: a checkpoint puts the caller's run one Fuse from discovering a pair.
import { describe, expect, it } from "vitest";
import { codexPair, pairState } from "../../../src/mvp/codex.js";
import type { PlayerRef } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { decide } from "./runs.js";
import { mvpRuntime } from "./runtime.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };

describe("POST /dev/checkpoint", () => {
  it("opens the shop with two Awoken of the pair; one Fuse discovers it", async () => {
    let n = 7;
    const rt = mvpRuntime({ content: mvpContent(), codex: true, dev: true, seed: () => (n = (n * 1103515245 + 12345) >>> 0) });
    rt.store.addPlayer(maks);
    const [a, b] = rt.content.units.filter((u) => u.forms?.awoken).map((u) => u.id) as [string, string];
    const [x, y] = a < b ? [a, b] : [b, a];
    const design = { name: "Test Fusion", lean: "twist" as const, recipe: structuredClone(rt.content.units[2]!.forms.awoken) };
    rt.store.putCodexPair({ ...codexPair(x, y), candidates: [design, design, design], preparedAt: rt.now().toISOString() });
    const app = createMvpApp(rt);
    const post = (body: unknown) => app.request("/api/v1/dev/checkpoint", { method: "POST", headers: { "content-type": "application/json", "X-Arena-Player": maks.id }, body: JSON.stringify(body) });
    expect((await post({ a, b: "nobody" })).status).toBe(400);
    const res = await post({ a: b, b: a });
    expect(res.status).toBe(200);
    const { runId } = (await res.json()) as { runId: string };
    const run = rt.store.run(runId)!;
    expect(run.phase).toBe("shop");
    expect(run.line.map((u) => [u.unitId, u.form])).toEqual([[b, "awoken"], [a, "awoken"]]);
    // a second checkpoint reuses the same run
    expect(((await (await post({ a, b })).json()) as { runId: string }).runId).toBe(runId);
    decide(rt, rt.store.run(runId)!, { kind: "fuse", first: 0, second: 1, pick: 1 });
    expect(pairState(rt.store.codexPair(x, y))).toBe("discovered");
    expect(rt.store.codexPair(x, y)!.claim!.by.id).toBe(maks.id);
  });
});
