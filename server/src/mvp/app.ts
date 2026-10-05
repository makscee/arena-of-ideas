// Arena MVP HTTP API (mission #574, slice 1). The routes and shapes are the
// contract in src/mvp/contract.ts; this file only wires them to the pure run
// (src/mvp/run.ts) and a store.
import { randomUUID } from "node:crypto";
import { Hono, type Context } from "hono";
import {
  MVP_API_PREFIX,
  MVP_API_VERSION,
  MVP_RULES,
  PLAYER_HEADER,
  type Decision,
  type DecisionResponse,
  type Ghost,
  type HomeView,
  type MvpContent,
  type PlayerRef,
} from "../../../src/mvp/contract.js";
import { fuseCheck } from "../../../src/mvp/forms.js";
import { MvpDecisionError, applyMvpDecision, initMvpRun, runView, synthGhost, unitById, type DecisionContext, type MvpRunState } from "../../../src/mvp/run.js";
import { storedOrPortmanteau, type NameFusion } from "./fusions.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";

export interface MvpDeps {
  content: MvpContent;
  store?: MvpStore;
  /** Seed source; tests pin it. */
  seed?: () => number;
  now?: () => Date;
  /** Names a pair at fuse time, synchronously: the stored name, else a
   * deterministic portmanteau (default: storedOrPortmanteau). Slice 10 fills
   * the model's name into the store in the background. */
  nameFusion?: NameFusion;
}

const NAME_RE = /^[\p{L}\p{N}_\- ]{1,24}$/u;

function ghostOf(r: MvpRunState): Ghost {
  return { ghostId: `${r.runId}-r${r.round}`, runId: r.runId, player: r.player, round: r.round, line: structuredClone(r.line) };
}

export function createMvpApp(deps: MvpDeps): Hono {
  const { content } = deps;
  const store = deps.store ?? new MemoryMvpStore();
  const seed = deps.seed ?? (() => Math.floor(Math.random() * 2 ** 32));
  const now = deps.now ?? (() => new Date());
  const nameFusion = deps.nameFusion ?? storedOrPortmanteau(store);
  const api = new Hono();

  const bad = (c: Context, status: 400 | 401 | 404 | 409 | 501, error: string) => c.json({ error }, status);
  const playerOf = (c: Context): PlayerRef | undefined => {
    const id = c.req.header(PLAYER_HEADER);
    return id ? store.player(id) : undefined;
  };

  api.get("/health", (c) => c.json({ ok: true, api: MVP_API_VERSION, contentVersion: content.version }));
  api.get("/content", (c) => c.json(content));

  api.post("/players", async (c) => {
    const body = (await c.req.json().catch(() => null)) as { name?: unknown } | null;
    const name = typeof body?.name === "string" ? body.name.trim() : "";
    if (!NAME_RE.test(name)) return bad(c, 400, "name: 1–24 letters, digits, spaces, _ or -");
    const p: PlayerRef = { id: randomUUID(), name, bot: false };
    store.addPlayer(p);
    return c.json(p);
  });

  api.get("/home", (c) => {
    const p = playerOf(c);
    const t = now();
    const home: HomeView = {
      rules: MVP_RULES,
      // Slice 5 owns the day; until then there is no champion and the day is today's date.
      day: { day: t.toISOString().slice(0, 10), endsAt: t.toISOString(), champion: null, slayers: 0, lastPlayoff: null },
      rating: p ? { player: p, rating: MVP_RULES.ratingStart, runs: 0, slays: 0, daysAsChampion: 0, playoffWins: 0 } : null,
      activeRunId: p ? store.activeRun(p.id)?.runId ?? null : null,
    };
    return c.json(home);
  });

  api.post("/runs", (c) => {
    const p = playerOf(c);
    if (!p) return bad(c, 401, `unknown player: send ${PLAYER_HEADER} from POST /players`);
    const run = initMvpRun({ runId: randomUUID(), player: p, seed: seed(), content });
    store.putRun(run);
    return c.json(runView(run));
  });

  api.get("/runs/:runId", (c) => {
    const run = store.run(c.req.param("runId"));
    return run ? c.json(runView(run)) : bad(c, 404, "no such run");
  });

  api.post("/runs/:runId/decisions", async (c) => {
    const p = playerOf(c);
    const run = store.run(c.req.param("runId"));
    if (!run) return bad(c, 404, "no such run");
    if (!p || p.id !== run.player.id) return bad(c, 401, "not your run");
    const d = (await c.req.json().catch(() => null)) as Decision | null;
    if (!d || typeof d !== "object" || typeof d.kind !== "string") return bad(c, 400, "body must be a Decision");
    try {
      const ctx: DecisionContext = {};
      const first = d.kind === "fuse" ? run.line[d.first] : undefined;
      const second = d.kind === "fuse" ? run.line[d.second] : undefined;
      if (first && second && fuseCheck(first, second) === null) {
        ctx.fuse = nameFusion(unitById(content, first.unitId), unitById(content, second.unitId), run.player);
      }
      if (d.kind === "fight") {
        // Snapshot before the fight, so even a losing line becomes someone's ghost.
        const candidates = store.ghosts(run.round, run.runId);
        const pick = seed();
        const ghost =
          candidates.length > 0
            ? candidates[pick % candidates.length]!
            : synthGhost({ content, round: run.round, seed: pick, ghostId: `bot-${randomUUID()}` });
        if (run.line.length > 0) store.addGhost(ghostOf(run));
        ctx.fight = { ghost, battleId: randomUUID(), battleSeed: seed() };
      }
      const step = applyMvpDecision(run, d, content, ctx);
      store.putRun(step.state);
      if (step.battle) store.putBattle(step.battle);
      const res: DecisionResponse = { run: runView(step.state), ...(step.fight ? { fight: step.fight } : {}) };
      return c.json(res);
    } catch (err) {
      if (err instanceof MvpDecisionError) return bad(c, err.kind === "fuse" ? 501 : 409, err.message);
      throw err;
    }
  });

  api.get("/battles/:battleId", (c) => {
    const b = store.battle(c.req.param("battleId"));
    return b ? c.json(b) : bad(c, 404, "no such battle");
  });

  // Slice 10 owns this route and the store behind it; slice 11 only reads.
  api.get("/fusions", (c) => c.json(store.fusions()));

  api.get("/day", (c) => bad(c, 501, "the day arrives in slice 5"));
  api.post("/dev/end-day", (c) => bad(c, 501, "the day arrives in slice 5"));
  api.get("/stats", (c) => bad(c, 501, "stats arrive in slice 11"));

  const app = new Hono();
  app.route(MVP_API_PREFIX, api);
  return app;
}
