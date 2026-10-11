// arena-mcp (#959): the bundled file, driven over stdio the way an agent's
// host drives it, against the real API on a real port: login by email code, a
// whole practice run by the tools, a ranked run it may only read, a battle and
// its trace in English and Russian.
import { serve } from "@hono/node-server";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { mkdtempSync, readFileSync, statSync, writeFileSync } from "node:fs";
import type { AddressInfo } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { createMvpApp } from "../../server/src/mvp/app.js";
import { mvpContent } from "../../server/src/mvp/content.js";
import { FakeMailer } from "../../server/src/mvp/email.js";
import { mvpRuntime } from "../../server/src/mvp/runtime.js";
import { mcpBundle } from "../../scripts/mcp-bundle.js";
import { MVP_RULES } from "../../src/mvp/contract.js";
import { ArenaClient, botDecision } from "../core/index.js";

const dir = mkdtempSync(join(tmpdir(), "arena-mcp-"));
const bundle = join(dir, "arena-mcp.mjs");
const sessionFile = join(dir, "session.json");

/** A host's side of the stdio pipe. */
class Host {
  private id = 0;
  private waiting = new Map<number, (m: any) => void>();
  readonly child: ChildProcessWithoutNullStreams;
  constructor(env: Record<string, string>) {
    this.child = spawn("node", [bundle], { env: { PATH: process.env.PATH ?? "", HOME: dir, ...env } });
    let buf = "";
    this.child.stdout.on("data", (d) => {
      buf += d;
      for (let i = buf.indexOf("\n"); i >= 0; i = buf.indexOf("\n")) {
        const m = JSON.parse(buf.slice(0, i));
        buf = buf.slice(i + 1);
        this.waiting.get(m.id)?.(m);
      }
    });
  }
  rpc(method: string, params?: unknown): Promise<any> {
    const id = ++this.id;
    return new Promise((resolve) => {
      this.waiting.set(id, resolve);
      this.child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, ...(params ? { params } : {}) }) + "\n");
    });
  }
  notify(method: string) {
    this.child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method }) + "\n");
  }
  /** A tool call's text, and whether it was an error. */
  async tool(name: string, args: Record<string, unknown> = {}): Promise<{ text: string; isError: boolean }> {
    const r = await this.rpc("tools/call", { name, arguments: args });
    return { text: r.result.content[0].text, isError: !!r.result.isError };
  }
}

describe("arena-mcp", () => {
  const mail = new FakeMailer(false);
  const rt = mvpRuntime({ content: mvpContent(), dev: true, mail });
  let server: ReturnType<typeof serve>;
  let url = "";
  const hosts: Host[] = [];

  beforeAll(async () => {
    writeFileSync(bundle, mcpBundle());
    server = serve({ fetch: createMvpApp(rt).fetch, port: 0, hostname: "127.0.0.1" });
    await new Promise((r) => server.once("listening", r));
    url = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
  });
  afterAll(() => {
    hosts.forEach((h) => h.child.kill());
    server.close();
  });
  const host = (env: Record<string, string> = {}) => {
    const h = new Host({ ARENA_URL: url, ARENA_SESSION_FILE: sessionFile, ...env });
    hosts.push(h);
    return h;
  };

  it("speaks MCP: initialize, then lists the tools", async () => {
    const h = host();
    const init = await h.rpc("initialize", { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "test", version: "0" } });
    expect(init.result).toMatchObject({ protocolVersion: "2025-06-18", serverInfo: { name: "arena-mcp" }, capabilities: { tools: {} } });
    h.notify("notifications/initialized");
    const list = await h.rpc("tools/list");
    expect(list.result.tools.map((t: { name: string }) => t.name)).toEqual(["login", "login_code", "home", "start_run", "run", "decide", "preview", "battle", "trace", "codex"]);
    for (const t of list.result.tools) expect(t.inputSchema.type).toBe("object");
    expect((await h.rpc("nope")).error.code).toBe(-32601);
    expect((await h.rpc("ping")).result).toEqual({});
  });

  it("asks to log in before anything that needs a player", async () => {
    const h = host();
    expect((await h.tool("start_run")).text).toMatch(/Not logged in/);
    expect((await h.tool("home")).text).toMatch(/Not logged in/);
  });

  it("logs in by email code and keeps the session in a 0600 file", async () => {
    const h = host();
    expect((await h.tool("login_code", { code: "123456" })).text).toMatch(/login \{email\} first/);
    expect((await h.tool("login", { email: "agent@example.com" })).text).toMatch(/code was mailed/);
    const code = mail.lastCode("agent@example.com")!;
    expect((await h.tool("login_code", { code })).text).toMatch(/no player yet/);
    expect((await h.tool("login_code", { code, name: "Coach" })).text).toMatch(/Logged in as @Coach/);
    expect(statSync(sessionFile).mode & 0o777).toBe(0o600);
    expect(JSON.parse(readFileSync(sessionFile, "utf8"))[url].token).toMatch(/\S{10}/);
    // A fresh process finds the session.
    expect((await host().tool("home")).text).toMatch(/^@Coach · rating \d+/);
  });

  it("plays a whole practice run by the tools, then reads the battle and its trace", async () => {
    const h = host();
    const token = JSON.parse(readFileSync(sessionFile, "utf8"))[url].token as string;
    const eyes = new ArenaClient({ api: `${url}/api/v1`, token }); // a human-side client, to read RunView JSON for the bot
    const content = await eyes.content();

    expect((await h.tool("run")).text).toMatch(/No practice run going: call start_run/);
    const start = await h.tool("start_run");
    expect(start.text).toMatch(/^Arena · Round 1\/\d+ · ♥\d+ · \d+g/);
    expect(start.text).toContain("SHOP\n  [0] ");
    expect((await h.tool("home")).text).toMatch(/Practice run: [0-9a-f-]{36}\n/);
    expect((await h.tool("decide", { decision: { kind: "buy", slot: 99 } })).isError).toBe(true);
    expect((await h.tool("decide", { decision: "not json" })).text).toMatch(/JSON object/);

    // A preview moves nothing.
    const before = (await h.tool("run")).text;
    const pv = await h.tool("preview", { decision: { kind: "reroll" } });
    expect(pv.text).toMatch(/^PREVIEW of \{"kind":"reroll"\}: nothing was changed\./);
    expect((await h.tool("run")).text).toBe(before);

    let run = await eyes.run((await eyes.home()).practiceRunId!);
    let rerolled = 0;
    let lastFight = "";
    for (let i = 0; i < 400 && run.phase !== "over"; i++) {
      const d = botDecision(run, content, MVP_RULES, rerolled);
      rerolled = d.kind === "reroll" ? rerolled + 1 : d.kind === "fight" ? 0 : rerolled;
      const r = await h.tool("decide", { runId: run.runId, decision: d });
      expect(r.isError, r.text).toBe(false);
      if (d.kind === "fight") expect(r.text).toMatch(/^(Round \d+|Crown|Playoff) vs @.+: (win|loss|draw)/);
      lastFight = /battle (\S+)/.exec(r.text)?.[1] ?? lastFight;
      run = await eyes.run(run.runId);
    }
    expect(run.phase).toBe("over");
    expect(run.mode).toBe("practice");
    expect((await h.tool("run")).text).toMatch(/No practice run going/);
    expect((await h.tool("run", { runId: run.runId })).text).toContain("LAST   ");
    expect((await h.tool("start_run")).text).toMatch(/^Arena · Round 1\//); // a new one after the old ended

    const en = await h.tool("battle", { battleId: lastFight });
    expect(en.text).toMatch(/^Battle .+\nA: .+\nB: .+\n/);
    expect(en.text).toMatch(/\nTurn 1\n/);
    expect(en.text).toMatch(/You (win|lose)|They win|draw/i);
    expect((await h.tool("battle", { battleId: lastFight, view: "summary" })).text.split("\n").length).toBe(5);
    const ru = await h.tool("battle", { battleId: lastFight, lang: "ru" });
    expect(ru.text).toMatch(/[а-я]/);
    expect(en.text).not.toMatch(/[а-я]/);
    const id = Number(/#(\d+)/.exec(en.text.split("Turn 1")[1]!)![1]);
    expect((await h.tool("trace", { battleId: lastFight, eventId: id })).text.length).toBeGreaterThan(0);
    expect((await h.tool("trace", { battleId: lastFight, eventId: id, lang: "ru" })).text.length).toBeGreaterThan(0);
    expect((await h.tool("trace", { battleId: lastFight })).text).toMatch(/eventId/);
    expect((await h.tool("battle", { battleId: "nope" })).text).toMatch(/Arena answered 404/);
  });

  it("reads a ranked run and previews on it, but decide says it needs the player's tap", async () => {
    const token = JSON.parse(readFileSync(sessionFile, "utf8"))[url].token as string;
    const human = new ArenaClient({ api: `${url}/api/v1`, token }); // no agent header: the phone
    const ranked = await human.startRun();
    expect(ranked.mode).toBeUndefined();
    const h = host();
    expect((await h.tool("run", { runId: ranked.runId })).text).toContain("SHOP");
    expect((await h.tool("preview", { runId: ranked.runId, decision: { kind: "reroll" } })).text).toMatch(/nothing was changed/);
    const refused = await h.tool("decide", { runId: ranked.runId, decision: { kind: "reroll" } });
    expect(refused.text).toMatch(/ranked run.*player's own tap on their phone\. Nothing was moved/s);
    expect((await human.run(ranked.runId)).gold).toBe(ranked.gold);
    expect((await h.tool("home")).text).toContain(`Ranked run: ${ranked.runId}`);
    // The server holds the line too, were the tool to ask.
    await expect(new ArenaClient({ api: `${url}/api/v1`, token, client: "agent/other" }).decide(ranked.runId, { kind: "reroll" })).rejects.toMatchObject({ status: 403 });
  });

  it("takes ARENA_TOKEN instead of a login, and says so when it is wrong", async () => {
    const token = JSON.parse(readFileSync(sessionFile, "utf8"))[url].token as string;
    expect((await host({ ARENA_TOKEN: token, ARENA_SESSION_FILE: join(dir, "none.json") }).tool("home")).text).toMatch(/^@Coach/);
    const bad = await host({ ARENA_TOKEN: "nope", ARENA_SESSION_FILE: join(dir, "none.json") }).tool("home");
    expect(bad.isError).toBe(true);
    expect(bad.text).toMatch(/401/);
  });

  it("looks things up in the Codex, English and Russian", async () => {
    const h = host();
    const content = mvpContent();
    const u = content.units[0]!;
    const list = (await h.tool("codex")).text;
    expect(list).toContain(u.name);
    expect((await h.tool("codex", { query: u.name })).text).toContain(`${u.emoji} ${u.name} · T${u.tier}`);
    expect((await h.tool("codex", { query: u.name, lang: "ru" })).text).toContain("Пробуждён: ");
    expect((await h.tool("codex", { query: "glossary" })).text).toMatch(/^PWR: Power/);
    expect((await h.tool("codex", { query: "zzzz-nothing" })).text).toMatch(/Nothing in the Codex/);
  });
});
