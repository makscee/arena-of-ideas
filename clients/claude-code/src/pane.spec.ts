// The Claude Code mod (#960), without Claude Code: the shipped bundle
// (hooks/arena.mjs) is fresh, Claude's tools are arena-mcp's, and the pane's
// keys play a ranked round against a real server through the same `$.http.fetch`
// adapter the mod uses. The player's keys go without the agent header (they are
// the player's moves); Claude's tools are an agent's, refused a ranked move.
import { serve } from "@hono/node-server";
import { readFileSync } from "node:fs";
import type { AddressInfo } from "node:net";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { createMvpApp } from "../../../server/src/mvp/app.js";
import { mvpContent } from "../../../server/src/mvp/content.js";
import { FakeMailer } from "../../../server/src/mvp/email.js";
import { mvpRuntime } from "../../../server/src/mvp/runtime.js";
import { modBundle, MOD_BUNDLE } from "../../../scripts/mod-bundle.js";
import { CLIENT_HEADER, MVP_RULES, type Decision, type RunView } from "../../../src/mvp/contract.js";
import { arenaTools } from "../../mcp/tools.js";
import { ArenaClient, apiBase, battlePage, decisionText, hostFetch, keysFor, paneLines, press, type Pending, type Suggestion } from "./lib.js";

describe("Claude Code mod", () => {
  const mail = new FakeMailer(false);
  const rt = mvpRuntime({ content: mvpContent(), dev: true, mail });
  let server: ReturnType<typeof serve>;
  let url = "";
  /** Every request's agent header, as the server got it. */
  const seen: (string | null)[] = [];
  /** `$.http.fetch` as the host answers it: status, ok, text. */
  const hostHttp = async (u: string, init?: { method?: string; headers?: Record<string, string>; body?: string }) => {
    seen.push(init?.headers?.[CLIENT_HEADER] ?? null);
    const r = await fetch(u, init);
    return { status: r.status, ok: r.ok, text: await r.text() };
  };

  beforeAll(async () => {
    server = serve({ fetch: createMvpApp(rt).fetch, port: 0, hostname: "127.0.0.1" });
    await new Promise((r) => server.once("listening", r));
    url = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
  });
  afterAll(() => server.close());

  it("ships a fresh bundle (npm run mod:build) with no dynamic import", () => {
    const shipped = readFileSync(MOD_BUNDLE, "utf8");
    expect(shipped === modBundle(), "clients/claude-code/hooks/arena.mjs is stale: npm run mod:build").toBe(true);
    expect(shipped).not.toMatch(/\bimport\(/);
  });

  it("serves arena-mcp's tools, by the names register.tsx matches", () => {
    const names = arenaTools({ url, store: { load: () => ({}), save: () => {} } }).map((t) => t.name);
    const src = readFileSync(new URL("../hooks/register.tsx", import.meta.url), "utf8");
    const re = /tool: \/\^mcp__arena__\(([^)]+)\)\$\//.exec(src);
    expect(re?.[1]?.split("|")).toEqual(names);
  });

  it("plays a ranked round with keys, takes Claude's pick with y, replays the battle in Russian", async () => {
    const player = new ArenaClient({ api: apiBase(url), fetch: hostFetch(hostHttp) });
    await player.emailStart("pane@example.com");
    const v = await player.emailVerify("pane@example.com", mail.lastCode("pane@example.com")!, "Pane");
    expect(v.status).toBe("done");
    const token = player.conn.token!;
    const content = await player.content();
    const rules = (await player.home()).rules;
    expect(rules).toEqual(MVP_RULES);

    // No run: n starts a ranked one.
    expect(keysFor(null, rules, null, null, "run").map((k) => k.key)).toEqual(["n", "p", "t"]);
    expect(press("n", null, rules, null, null, "run")).toEqual({ do: "ranked" });
    let run: RunView = await player.startRun("ranked");
    expect(run.mode ?? "ranked").toBe("ranked");
    const lines = paneLines(run, content, rules);
    expect(lines[0]).toMatch(/^Arena · Round 1\/12 · ♥\d+ · \d+g · 0W 0L · vs @\S+ {3}\[ranked\]$/);
    expect(lines.find((l) => l.startsWith("SHOP"))).toMatch(/^SHOP {4}1 \S+/);
    expect(lines.find((l) => l.startsWith("LINE"))).toBe(`LINE    ${Array.from({ length: rules.lineSize }, (_, i) => `${i + 1} —`).join(" · ")}`);

    // A key is the player's move: 1 buys the first offer.
    const play = async (key: string, pending: Pending | null = null, suggestion: Suggestion | null = null) => {
      const p = press(key, run, rules, pending, suggestion, "run");
      if (!("decision" in p)) return p;
      const r = await player.decide(run.runId, p.decision);
      run = r.run;
      return r;
    };
    const gold = run.gold;
    await play("1");
    expect(run.line).toHaveLength(1);
    expect(run.gold).toBeLessThan(gold);
    // s then a slot sells; c cancels a key waiting for its number.
    expect(press("s", run, rules, null, null, "run")).toEqual({ pending: { kind: "sell" } });
    expect(press("c", run, rules, { kind: "sell" }, null, "run")).toEqual({ pending: null });
    expect(press("m", run, rules, null, null, "run")).toEqual({ pending: { kind: "move" } });
    expect(press("1", run, rules, { kind: "move" }, null, "run")).toEqual({ pending: { kind: "move2", first: 0 } });
    expect(press("3", run, rules, { kind: "move2", first: 0 }, null, "run")).toEqual({ decision: { kind: "reorder", from: 0, to: 2 } });
    expect(press("6", run, rules, { kind: "fuse", }, null, "run")).toEqual({ pending: { kind: "fuse2", first: 5 } });
    expect(press("9", run, rules, null, null, "run")).toEqual({ none: true });

    // Claude, as an agent, can't move the ranked run: its decide says so and moves nothing.
    const claude = arenaTools({ url, token, client: "agent/claude-code", fetch: hostFetch(hostHttp), store: { load: () => ({ token }), save: () => {} } });
    const decide = claude.find((t) => t.name === "decide")!;
    seen.length = 0;
    expect(await decide.run({ runId: run.runId, decision: { kind: "reroll" } })).toMatch(/ranked run/);
    expect(seen.every((h) => h === "agent/claude-code")).toBe(true);
    await expect(new ArenaClient({ api: apiBase(url), token, client: "agent/claude-code", fetch: hostFetch(hostHttp) }).decide(run.runId, { kind: "reroll" })).rejects.toMatchObject({ status: 403 });

    // Its suggestion shows in the pane; y sends it as the player's move.
    const pick: Decision = run.offers.length && run.gold >= run.offers[0]!.cost ? { kind: "buy", slot: 0 } : { kind: "reroll" };
    const s: Suggestion = { runId: run.runId, decision: pick, why: "test" };
    expect(keysFor(run, rules, null, s, "run").map((k) => k.key)).toContain("y");
    expect(decisionText(pick, run, rules, content)).toMatch(pick.kind === "buy" ? /^buy 1 \S/ : /^reroll$/);
    expect(press("y", run, rules, null, s, "run")).toEqual({ do: "accept" });
    seen.length = 0;
    const before = run.gold;
    run = (await player.decide(s.runId, s.decision)).run;
    expect(run.gold).toBeLessThan(before);
    expect(seen).toEqual([null]);

    // g fights; b replays the battle; RU reads Russian, headers too.
    seen.length = 0;
    const fought = await play("g");
    expect(seen).toEqual([null]);
    expect("fight" in fought && fought.fight).toBeTruthy();
    expect(run.fights).toHaveLength(1);
    expect(keysFor(run, rules, null, null, "run").map((k) => k.key)).toContain("b");
    expect(press("b", run, rules, null, null, "run")).toEqual({ do: "battle" });
    const battle = claude.find((t) => t.name === "battle")!;
    const ru = await battle.run({ battleId: run.fights[0]!.battleId, lang: "ru" });
    expect(ru).toMatch(/^Старт$/m);
    expect(ru).not.toMatch(/^(Start|Turns? \d)/m);
    const page = battlePage(ru, 0, 10);
    expect(page.lines).toHaveLength(10);
    expect(battlePage(ru, 99, 10).page).toBe(page.pages - 1);
    expect(paneLines(run, content, rules, { lang: "ru" })[0]).toMatch(/^Arena · Раунд 2\/12 .* \[рейтинг\]$/);
    expect(keysFor(run, rules, null, null, "run", { lang: "ru" }).map((k) => k.label)).toContain("в бой");
  });
});
