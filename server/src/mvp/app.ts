// Arena MVP HTTP API (mission #574, slice 1). The routes and shapes are the
// contract in src/mvp/contract.ts; this file only parses requests and wires
// them to the run engine (./runs.ts) and a store.
import { randomUUID } from "node:crypto";
import { Hono, type Context } from "hono";
import {
  MVP_API_PREFIX,
  MVP_API_VERSION,
  MVP_RULES,
  PLAYER_HEADER,
  type Decision,
  type HomeView,
  type MvpContent,
  type PlayerRef,
} from "../../../src/mvp/contract.js";
import { MvpDecisionError, runView } from "../../../src/mvp/run.js";
import { storedOrPortmanteau, type NameFusion } from "./fusions.js";
import { decide, startRun, type RunDeps, type RunHooks } from "./runs.js";
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
  /** Observers of fights and run ends (slices 5 and 11). */
  hooks?: RunHooks[];
}

const NAME_RE = /^[\p{L}\p{N}_\- ]{1,24}$/u;

export function createMvpApp(deps: MvpDeps): Hono {
  const { content } = deps;
  const store = deps.store ?? new MemoryMvpStore();
  const seed = deps.seed ?? (() => Math.floor(Math.random() * 2 ** 32));
  const now = deps.now ?? (() => new Date());
  const runs: RunDeps = { store, content, seed, now, hooks: deps.hooks ?? [], nameFusion: deps.nameFusion ?? storedOrPortmanteau(store) };
  // Slice 5 owns the day counter (rollover, dev end-day); until then it is always day 1.
  const daySeq = (): number => 1;
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
    const seq = daySeq();
    const home: HomeView = {
      rules: MVP_RULES,
      // Slice 5 owns the day (date, rollover time, playoff); the champion is
      // whoever the store holds (slice 6 seeds day 1), none until then.
      day: {
        seq,
        day: t.toISOString().slice(0, 10),
        endsAt: t.toISOString(),
        champion: store.currentChampion() ?? null,
        slayers: new Set(store.slays(seq).map((s) => s.player.id)).size,
        lastPlayoff: null,
      },
      rating: p ? store.rating(p.id) ?? { player: p, rating: MVP_RULES.ratingStart, runs: 0, slays: 0, daysAsChampion: 0, playoffWins: 0 } : null,
      activeRunId: p ? store.activeRun(p.id)?.runId ?? null : null,
    };
    return c.json(home);
  });

  api.post("/runs", (c) => {
    const p = playerOf(c);
    if (!p) return bad(c, 401, `unknown player: send ${PLAYER_HEADER} from POST /players`);
    return c.json(runView(startRun(runs, p, daySeq())));
  });

  api.get("/runs/:runId", (c) => {
    const run = store.run(c.req.param("runId"));
    return run ? c.json(runView(run)) : bad(c, 404, "no such run");
  });

  api.post("/runs/:runId/decisions", async (c) => {
    // Await the body before reading the run: from here on the handler is
    // synchronous, so parallel decisions on one run apply one after another.
    const d = (await c.req.json().catch(() => null)) as Decision | null;
    const p = playerOf(c);
    const run = store.run(c.req.param("runId"));
    if (!run) return bad(c, 404, "no such run");
    if (!p || p.id !== run.player.id) return bad(c, 401, "not your run");
    if (!d || typeof d !== "object" || typeof d.kind !== "string") return bad(c, 400, "body must be a Decision");
    try {
      return c.json(decide(runs, run, d));
    } catch (err) {
      // Every fuse answers 501 until slice 2 makes fusion work; then 409 like the rest.
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
