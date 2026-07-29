import Database from "better-sqlite3";
import { spawn, spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, test } from "vitest";
import { eq } from "drizzle-orm";
import { castIdeaVote, listIdeas } from "./ideas.js";
import { openDb } from "./db.js";
import { freezeSelection, rollSeason, stageBounce, stageShip } from "./governance.js";
import { readActiveContent, readSeasonPointer } from "./season-store.js";
import { openRun, servePool } from "./runs.js";
import { SqliteLadderStore } from "./ladder-store.js";
import { DEFAULT_SELECTION_TUNABLES, deserializeRun, initRun, serializeRun } from "../../src/index.js";
import { mint } from "./sessions.js";
import { contentVersions, ideaBuilds, ideaVotes, ideas, ladderChampions, ladderGhosts, runOpens, runPoolServes, runSubmissions, seasonArchives, seasonFreezes, users } from "./schema.js";
import { createApp } from "./app.js";
import { createMockMailClient } from "./mail.js";
import { createRateLimiter } from "./rate-limit.js";

const NOW = 1_800_000_000;
const FROST = JSON.parse(readFileSync(new URL("../../candidates/frostbite-striker.json", import.meta.url), "utf8"));
const REASON = "sim gauntlet: win rate above the allowed band";
const REPO_ROOT = fileURLToPath(new URL("../..", import.meta.url));
const dirs: string[] = [];
afterEach(() => { for (const dir of dirs.splice(0)) rmSync(dir, { recursive: true, force: true }); });
function tempDb(name: string): string { const dir = mkdtempSync(join(tmpdir(), "aoi62-governance-")); dirs.push(dir); return join(dir, name); }

function setup(path = ":memory:") {
  const opened = openDb(path);
  const deps = { ...opened, clock: () => NOW };
  for (const [id, name] of [["maks", "Maks"], ["glass", "Glass Smith"], ["later", "Later Voter"]] as const) {
    deps.db.insert(users).values({ id, email: `${id}@test.invalid`, displayName: name, createdAt: NOW, updatedAt: NOW }).run();
  }
  for (let i = 1; i <= 5; i++) deps.db.insert(users).values({ id: `v${i}`, email: `v${i}@test.invalid`, displayName: `Voter ${i}`, createdAt: NOW, updatedAt: NOW }).run();
  deps.db.insert(ideas).values([
    { id: "idea-ship", seq: 0, authorId: "maks", text: "Frostbite Striker", createdAt: NOW, status: "on-table" },
    { id: "idea-bounce", seq: 1, authorId: "glass", text: "Overpowered glass cannon", createdAt: NOW, status: "on-table" },
  ]).run();
  for (let i = 1; i <= 5; i++) {
    deps.db.insert(ideaVotes).values({ ideaId: "idea-ship", userId: `v${i}`, direction: "up", votedAt: NOW }).run();
    deps.db.insert(ideaVotes).values({ ideaId: "idea-bounce", userId: `v${i}`, direction: i === 5 ? "down" : "up", votedAt: NOW }).run();
  }
  const active = readActiveContent(deps.db);
  const team = JSON.stringify([active.pool[0]]);
  deps.db.insert(ladderGhosts).values({ round: 1, seq: 0, runId: "old-run", userId: "maks", team }).run();
  deps.db.insert(ladderChampions).values({ runId: "old-run", userId: "maks", round: 1, seq: 0, team }).run();
  deps.db.insert(runOpens).values({ runId: "stale-run", userId: "maks", contentVersion: 1, ghostWatermark: 1, openedAt: NOW }).run();
  deps.db.insert(runPoolServes).values({ runId: "stale-run", round: 1, servedLen: 1, championRunId: "old-run", servedAt: NOW }).run();
  return deps;
}

function governCli(path: string, args: string[]) {
  return spawnSync("npm", ["run", "govern", "--", ...args], {
    cwd: REPO_ROOT,
    env: { ...process.env, DB_PATH: path },
    encoding: "utf8",
    timeout: 30_000,
  });
}

function cliOutput(result: ReturnType<typeof governCli>): string {
  return `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
}

function snapshot(deps: ReturnType<typeof setup>) {
  return {
    pointer: readSeasonPointer(deps.db),
    ideas: deps.db.select().from(ideas).all(),
    ghosts: deps.db.select().from(ladderGhosts).all(),
    champions: deps.db.select().from(ladderChampions).all(),
    archives: deps.db.select().from(seasonArchives).all(),
    freezes: deps.db.select().from(seasonFreezes).all(),
    builds: deps.db.select().from(ideaBuilds).all(),
    opens: deps.db.select().from(runOpens).all(),
    serves: deps.db.select().from(runPoolServes).all(),
    submissions: deps.db.select().from(runSubmissions).all(),
    contentRows: deps.db.select().from(contentVersions).all(),
    content: readActiveContent(deps.db),
  };
}

describe("AOI-62 operator governance and atomic season boundary", () => {
  test("freezes current 5/0.6/3 slate, ships authoritative Frostbiter, bounces with carried votes, then rolls atomically", () => {
    const deps = setup();
    const oldTeam = deps.db.select().from(ladderChampions).all()[0]!.team;
    deps.db.insert(ladderChampions).values({ runId: "top-run", userId: "glass", round: 3, seq: 0, team: oldTeam }).run();
    const expected = { season: 1, contentVersion: 1 };
    const frozen = freezeSelection(deps, expected);
    expect(frozen.selection.selected.map((r) => r.idea.id)).toEqual(["idea-ship", "idea-bounce"]);
    expect(frozen.selection.selected.map((r) => r.tally.ratio)).toEqual([1, 0.8]);
    expect(() => freezeSelection(deps, expected)).toThrow(/already frozen/);
    const frozenReceipt = deps.db.select().from(seasonFreezes).all()[0]!.selectionReceipt;
    expect(deps.db.select().from(ideaBuilds).all().map((r) => r.status)).toEqual(["building", "building"]);
    expect(listIdeas(deps).map((i) => [i.id, i.status])).toEqual([["idea-ship", "selected"], ["idea-bounce", "selected"]]);

    const shipped = stageShip(deps, expected, "idea-ship", FROST, "frostbite fixture");
    expect(shipped).toMatchObject({ candidateId: "frostbite-striker", authorUserId: "maks", creatorDisplayName: "Maks" });
    expect(shipped.units).toContain("Frostbiter");
    expect(() => stageShip(deps, expected, "idea-ship", FROST, "duplicate")).toThrow(/duplicate staging|already has outcome/);
    expect(() => stageBounce(deps, expected, "idea-bounce", "   ")).toThrow(/requires a non-empty reason/);
    stageBounce(deps, expected, "idea-bounce", REASON);

    const rolled = rollSeason(deps, expected);
    expect(rolled).toMatchObject({ archivedSeason: 1, season: 2, contentVersion: 2 });
    expect(readSeasonPointer(deps.db)).toEqual({ season: 2, contentVersion: 2 });
    const content = readActiveContent(deps.db);
    const frost = content.pool.find((unit) => unit.name === "Frostbiter");
    expect(frost?._creator).toBe("Maks");
    expect(content.abilities.Frostbite).toBeDefined();
    expect(deps.db.select().from(ladderGhosts).all()).toEqual([]);
    expect(deps.db.select().from(ladderChampions).all()).toEqual([]);
    expect(deps.db.select().from(runPoolServes).all()).toEqual([]);
    expect(deps.db.select().from(runOpens).all()[0]).toMatchObject({ runId: "stale-run", contentVersion: 1 });
    expect(servePool({ db: deps.db, store: new SqliteLadderStore(deps.db), clock: deps.clock, contentVersion: 2 }, "maks", "stale-run", 1))
      .toMatchObject({ served: false, reason: expect.stringMatching(/season changed; start a fresh run/) });

    const archive = deps.db.select().from(seasonArchives).all()[0]!;
    expect(archive.contentVersion).toBe(1);
    expect(JSON.parse(archive.finalTower).bosses["1"].runId).toBe("old-run");
    expect(JSON.parse(archive.finalTower).bosses["3"].runId).toBe("top-run");
    expect(archive.selectionReceipt).toBe(frozenReceipt);
    expect(JSON.parse(archive.selectionReceipt).selected.map((r: any) => r.ideaId)).toEqual(["idea-ship", "idea-bounce"]);
    expect(JSON.parse(archive.outcomeReceipt)[0]).toMatchObject({ authorUserId: "maks", creatorDisplayName: "Maks" });
    deps.db.update(users).set({ displayName: "Renamed Later", updatedAt: NOW + 1 }).where(eq(users.id, "maks")).run();
    expect(readActiveContent(deps.db).pool.find((unit) => unit.name === "Frostbiter")?._creator).toBe("Maks");
    expect(JSON.parse(deps.db.select().from(seasonArchives).all()[0]!.outcomeReceipt)[0]).toMatchObject({ authorUserId: "maks", creatorDisplayName: "Maks" });

    const after = listIdeas(deps);
    expect(after.find((i) => i.id === "idea-ship")).toMatchObject({ status: "shipped", authorDisplayName: "Renamed Later" });
    const bounced = after.find((i) => i.id === "idea-bounce")!;
    expect(bounced).toMatchObject({ status: "bounced", bounceReason: REASON, tally: { up: 4, total: 5, ratio: 0.8 } });
    castIdeaVote(deps, "later", "idea-bounce", "up");
    expect(listIdeas(deps).find((i) => i.id === "idea-bounce")!.tally).toEqual({ up: 5, total: 6, ratio: 5 / 6 });
    expect(() => rollSeason(deps, expected)).toThrow(/stale season\/content/);
  }, 30_000);

  test("an empty slate has one authoritative freeze receipt and cannot roll unfrozen", () => {
    const opened = openDb(":memory:");
    const deps = { ...opened, clock: () => NOW };
    const expected = { season: 1, contentVersion: 1 };
    expect(() => rollSeason(deps, expected)).toThrow(/selection is not frozen/);
    expect(readSeasonPointer(deps.db)).toEqual(expected);
    expect(() => freezeSelection(deps, { season: 2, contentVersion: 1 })).toThrow(/stale season\/content/);

    const frozen = freezeSelection(deps, expected);
    expect(frozen.selection.selected).toEqual([]);
    const receipt = deps.db.select().from(seasonFreezes).all()[0]!;
    expect(JSON.parse(receipt.selectionReceipt)).toEqual({ tunables: DEFAULT_SELECTION_TUNABLES, selected: [] });
    const afterFreeze = snapshot(deps as ReturnType<typeof setup>);
    expect(() => freezeSelection(deps, expected)).toThrow(/already frozen/);
    expect(snapshot(deps as ReturnType<typeof setup>)).toEqual(afterFreeze);

    rollSeason(deps, expected);
    const archive = deps.db.select().from(seasonArchives).all()[0]!;
    expect(archive.selectionReceipt).toBe(receipt.selectionReceipt);
    expect(JSON.parse(archive.outcomeReceipt)).toEqual([]);
    expect(readSeasonPointer(deps.db)).toEqual({ season: 2, contentVersion: 2 });
  });

  test("the actual CLI rejects no-freeze/stale/duplicate empty transitions and archives the exact receipt", () => {
    const path = tempDb("empty-cli.db");
    const noFreeze = governCli(path, ["roll", "--season", "1", "--version", "1"]);
    expect(noFreeze.status, cliOutput(noFreeze)).toBe(1);
    expect(cliOutput(noFreeze)).toMatch(/selection is not frozen/);

    const stale = governCli(path, ["freeze", "--season", "2", "--version", "1"]);
    expect(stale.status, cliOutput(stale)).toBe(1);
    expect(cliOutput(stale)).toMatch(/stale season\/content/);

    const first = governCli(path, ["freeze", "--season", "1", "--version", "1"]);
    expect(first.status, cliOutput(first)).toBe(0);
    expect(JSON.parse(first.stdout.slice(first.stdout.indexOf("{")))).toMatchObject({ season: 1, contentVersion: 1, selection: { selected: [] } });
    const inspect = new Database(path);
    const receipt = (inspect.prepare("SELECT selection_receipt_json AS receipt FROM season_freezes WHERE season=1").get() as { receipt: string }).receipt;
    inspect.close();

    const duplicate = governCli(path, ["freeze", "--season", "1", "--version", "1"]);
    expect(duplicate.status, cliOutput(duplicate)).toBe(1);
    expect(cliOutput(duplicate)).toMatch(/already frozen/);
    const staleRoll = governCli(path, ["roll", "--season", "1", "--version", "2"]);
    expect(staleRoll.status, cliOutput(staleRoll)).toBe(1);
    expect(cliOutput(staleRoll)).toMatch(/stale season\/content/);

    const rolled = governCli(path, ["roll", "--season", "1", "--version", "1"]);
    expect(rolled.status, cliOutput(rolled)).toBe(0);
    const reopened = governCli(path, ["status"]);
    expect(reopened.status, cliOutput(reopened)).toBe(0);
    expect(JSON.parse(reopened.stdout.slice(reopened.stdout.indexOf("{")))).toEqual({ season: 2, contentVersion: 2 });
    const archived = new Database(path);
    expect((archived.prepare("SELECT selection_receipt_json AS receipt FROM season_archives WHERE season=1").get() as { receipt: string }).receipt).toBe(receipt);
    expect(archived.prepare("SELECT season, content_version FROM season_state WHERE id=1").get()).toEqual({ season: 2, content_version: 2 });
    archived.close();
  }, 30_000);

  test("invalid transitions, collisions, stale inputs, and failed roll validation mutate nothing", () => {
    const deps = setup();
    const expected = { season: 1, contentVersion: 1 };
    expect(() => stageBounce(deps, expected, "idea-ship", "nope")).toThrow(/selection is not frozen/);
    expect(() => freezeSelection(deps, { season: 1, contentVersion: 99 })).toThrow(/stale season\/content/);
    freezeSelection(deps, expected);
    const frozenReceipt = deps.db.select().from(seasonFreezes).all()[0]!.selectionReceipt;
    deps.db.update(seasonFreezes).set({ selectionReceipt: JSON.stringify({ tunables: DEFAULT_SELECTION_TUNABLES, selected: [] }) }).run();
    expect(() => stageShip(deps, expected, "idea-ship", FROST, "mismatched receipt")).toThrow(/does not match its frozen selection receipt/);
    deps.db.update(seasonFreezes).set({ selectionReceipt: frozenReceipt }).run();
    const collision = structuredClone(FROST);
    const pool = readActiveContent(deps.db).pool;
    collision.units.forEach((unit: any, i: number) => { unit.name = pool[i]!.name; });
    expect(() => stageShip(deps, expected, "idea-ship", collision, "collision")).toThrow(/collides|introduces no new/);
    stageBounce(deps, expected, "idea-bounce", REASON);
    const before = snapshot(deps);
    expect(() => rollSeason(deps, expected)).toThrow(/still building/);
    expect(snapshot(deps)).toEqual(before);
  }, 30_000);

  test("a failure at the first archive write rolls back the entire boundary", () => {
    const deps = setup();
    const expected = { season: 1, contentVersion: 1 };
    freezeSelection(deps, expected);
    stageShip(deps, expected, "idea-ship", FROST, "frostbite fixture");
    stageBounce(deps, expected, "idea-bounce", REASON);
    deps.sqlite.exec("CREATE TRIGGER fail_roll_archive BEFORE INSERT ON season_archives BEGIN SELECT RAISE(ABORT, 'injected archive failure'); END");
    const before = snapshot(deps);
    expect(() => rollSeason(deps, expected)).toThrow(/injected archive failure/);
    expect(snapshot(deps)).toEqual(before);
    deps.sqlite.exec("DROP TRIGGER fail_roll_archive");
  }, 30_000);

  test("a failure at the final pointer write rolls back archive/content/status/reset atomically", () => {
    const deps = setup();
    const expected = { season: 1, contentVersion: 1 };
    freezeSelection(deps, expected);
    stageShip(deps, expected, "idea-ship", FROST, "frostbite fixture");
    stageBounce(deps, expected, "idea-bounce", REASON);
    deps.sqlite.exec("CREATE TRIGGER fail_roll_pointer BEFORE UPDATE ON season_state BEGIN SELECT RAISE(ABORT, 'injected pointer failure'); END");
    const before = snapshot(deps);
    expect(() => rollSeason(deps, expected)).toThrow(/injected pointer failure/);
    expect(snapshot(deps)).toEqual(before);
    deps.sqlite.exec("DROP TRIGGER fail_roll_pointer");
  }, 30_000);

  test("a second-connection tower/run write committed before roll is archived, never lost between snapshot and reset", async () => {
    const path = tempDb("concurrent-roll.db");
    const deps = setup(path);
    const expected = { season: 1, contentVersion: 1 };
    freezeSelection(deps, expected);
    stageShip(deps, expected, "idea-ship", FROST, "frostbite fixture");
    stageBounce(deps, expected, "idea-bounce", REASON);

    const workerSource = `
      (async () => {
        const { openDb } = await import(${JSON.stringify(new URL("./db.ts", import.meta.url).href)});
        const { rollSeason } = await import(${JSON.stringify(new URL("./governance.ts", import.meta.url).href)});
        const opened = openDb(${JSON.stringify(path)});
        process.send({ type: "ready" });
        process.once("message", () => {
          process.send({ type: "starting" });
          try {
            const result = rollSeason({ ...opened, clock: () => ${NOW} }, { season: 1, contentVersion: 1 });
            opened.sqlite.close();
            process.send({ type: "result", result }, () => process.disconnect());
          } catch (error) {
            opened.sqlite.close();
            process.send({ type: "error", error: error instanceof Error ? error.stack : String(error) }, () => process.disconnect());
          }
        });
      })().catch((error) => process.send({ type: "error", error: error instanceof Error ? error.stack : String(error) }, () => process.disconnect()));
    `;
    const worker = spawn(process.execPath, ["--import", "tsx/esm", "-e", workerSource], {
      cwd: REPO_ROOT,
      stdio: ["ignore", "ignore", "pipe", "ipc"],
    });
    type WorkerMessage = { type: "ready" | "starting" } | { type: "result"; result: unknown } | { type: "error"; error: string };
    const waitFor = (type: WorkerMessage["type"]) => new Promise<WorkerMessage>((resolve, reject) => {
      const onMessage = (message: WorkerMessage) => {
        if (message.type === "error") { cleanup(); reject(new Error(message.error)); }
        else if (message.type === type) { cleanup(); resolve(message); }
      };
      const onError = (error: Error) => { cleanup(); reject(error); };
      const cleanup = () => { worker.off("message", onMessage); worker.off("error", onError); };
      worker.on("message", onMessage);
      worker.on("error", onError);
    });
    await waitFor("ready");

    const writer = new Database(path);
    writer.pragma("busy_timeout = 10000");
    writer.exec("BEGIN IMMEDIATE");
    const team = JSON.stringify([readActiveContent(deps.db).pool[0]]);
    writer.prepare("INSERT INTO ladder_ghosts (round,seq,run_id,user_id,team) VALUES (2,0,'concurrent-run','later',?)").run(team);
    writer.prepare("INSERT INTO run_submissions (run_id,user_id,content_version,seed,ended_by,final_round,submitted_at) VALUES ('concurrent-run','later',1,7,'loss',2,?)").run(NOW);

    const starting = waitFor("starting");
    const result = waitFor("result");
    worker.send("roll");
    await starting;
    await new Promise((resolve) => setTimeout(resolve, 100));
    writer.exec("COMMIT");
    await result;
    writer.close();
    await new Promise<void>((resolve, reject) => {
      if (worker.exitCode !== null) return worker.exitCode === 0 ? resolve() : reject(new Error(`roll worker exited ${worker.exitCode}: ${worker.stderr?.read()?.toString() ?? ""}`));
      worker.once("exit", (code) => code === 0 ? resolve() : reject(new Error(`roll worker exited ${code}: ${worker.stderr?.read()?.toString() ?? ""}`)));
    });

    const archive = JSON.parse(deps.db.select().from(seasonArchives).all()[0]!.finalTower);
    expect(archive.pools["2"]).toEqual([expect.objectContaining({ runId: "concurrent-run", round: 2, seq: 0 })]);
    expect(deps.db.select().from(ladderGhosts).where(eq(ladderGhosts.runId, "concurrent-run")).all()).toEqual([]);
    expect(deps.db.select().from(runSubmissions).where(eq(runSubmissions.runId, "concurrent-run")).all()).toHaveLength(1);
    expect(readSeasonPointer(deps.db)).toEqual({ season: 2, contentVersion: 2 });
    deps.sqlite.close();
    const reopened = openDb(path);
    expect(readActiveContent(reopened.db).pool.find((unit) => unit.name === "Frostbiter")?._creator).toBe("Maks");
    reopened.sqlite.close();
  }, 30_000);

  test("active v2 content is served by value and new run opens pin v2 while stale/missing versions fail", async () => {
    const deps = setup();
    const expected = { season: 1, contentVersion: 1 };
    freezeSelection(deps, expected);
    stageShip(deps, expected, "idea-ship", FROST, "frostbite fixture");
    stageBounce(deps, expected, "idea-bounce", REASON);
    rollSeason(deps, expected);
    const active = readActiveContent(deps.db);
    const state = initRun({ seed: 9, runId: "roundtrip", pool: active.pool, statuses: active.statuses, abilities: active.abilities });
    expect(deserializeRun(serializeRun(state))).toEqual(state);
    expect(state.offers.map((u) => u.name)).toContain("Frostbiter");
    expect(state.abilities.Frostbite).toBeDefined();
    const limiter = () => createRateLimiter({ limit: 100, windowMs: 60_000, clock: () => NOW * 1000 });
    const app = createApp({ db: deps.db, clock: deps.clock, mailClient: createMockMailClient(), rateLimiters: { ipStart: limiter(), emailStart: limiter(), poolServe: limiter() } });
    const contentResponse = await app.request("/v1/content");
    expect(contentResponse.status).toBe(200);
    expect(await contentResponse.json()).toMatchObject({ season: 2, contentVersion: 2, pool: expect.arrayContaining([expect.objectContaining({ name: "Frostbiter", _creator: "Maks" })]), abilities: { Frostbite: expect.any(Object) } });
    const token = mint(deps.db, { userId: "maks", label: "test", lifetimeDays: 1 }, deps.clock).token;
    const postOpen = (body: unknown) => app.request("/v1/runs/open", { method: "POST", headers: { authorization: `Bearer ${token}`, "content-type": "application/json" }, body: JSON.stringify(body) });
    expect((await postOpen({ runId: "missing-version" })).status).toBe(400);
    expect((await postOpen({ runId: "http-stale", contentVersion: 1 })).status).toBe(422);
    expect((await postOpen({ runId: "http-v2", contentVersion: 2 })).status).toBe(200);
    const store = new SqliteLadderStore(deps.db);
    expect(openRun({ db: deps.db, store, clock: deps.clock, contentVersion: 2 }, "maks", "v2-run", 2)).toMatchObject({ opened: true });
    expect(openRun({ db: deps.db, store, clock: deps.clock, contentVersion: 2 }, "maks", "old-version", 1))
      .toMatchObject({ opened: false, reason: expect.stringMatching(/season changed/) });
  }, 30_000);

  test("ordinary player HTTP authority exposes no governance mutation route", async () => {
    const deps = setup();
    const clock = () => NOW;
    const limiter = () => createRateLimiter({ limit: 100, windowMs: 60_000, clock: () => NOW * 1000 });
    const app = createApp({ db: deps.db, clock, mailClient: createMockMailClient(), rateLimiters: { ipStart: limiter(), emailStart: limiter(), poolServe: limiter() } });
    for (const path of ["/v1/governance/freeze", "/v1/governance/ship", "/v1/governance/roll"]) {
      expect((await app.request(path, { method: "POST", headers: { authorization: "Bearer player-token", "content-type": "application/json" }, body: "{}" })).status).toBe(404);
    }
  });
});
