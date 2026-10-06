// Arena MVP HTTP API (mission #574, slice 1). The routes and shapes are the
// contract in src/mvp/contract.ts; this file only parses requests and calls
// the runtime (./runtime.ts): the run engine (./runs.ts), the day (./day.ts),
// the stats (./stats.ts) and the store. A slice fills in those modules, not
// these routes.
import { randomUUID } from "node:crypto";
import { Hono, type Context } from "hono";
import { MVP_API_PREFIX, MVP_API_VERSION, PLAYER_HEADER, type Decision, type HomeView, type PlayerRef } from "../../../src/mvp/contract.js";
import { MvpBadDecision, MvpDecisionError, runView, type MvpRunState } from "../../../src/mvp/run.js";
import { dayView, endDay, hiddenSlay } from "./day.js";
import { MvpNotYet } from "./errors.js";
import { abandon, currentRun, decide, preview, startRun } from "./runs.js";
import { isMvpRuntime, mvpRuntime, type MvpDeps, type MvpRuntime } from "./runtime.js";
import { statsView } from "./stats.js";

const NAME_RE = /^[\p{L}\p{N}_\- ]{1,24}$/u;

/** The API on a runtime, or on a fresh one built from `deps`. */
export function createMvpApp(deps: MvpDeps | MvpRuntime): Hono {
  const rt = isMvpRuntime(deps) ? deps : mvpRuntime(deps);
  const { content, store } = rt;
  const api = new Hono();

  const bad = (c: Context, status: 400 | 401 | 404 | 409 | 501, error: string) => c.json({ error }, status);
  /** A stub that a later slice fills in answers 501 until then. */
  const notYet = (c: Context, fn: () => unknown) => {
    try {
      return c.json(fn());
    } catch (err) {
      if (err instanceof MvpNotYet) return bad(c, 501, err.message);
      throw err;
    }
  };
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
    const home: HomeView = {
      rules: rt.rules,
      day: dayView(rt),
      rating: p ? store.rating(p.id) ?? { player: p, rating: rt.rules.ratingStart, runs: 0, slays: 0, daysAsChampion: 0, playoffWins: 0 } : null,
      activeRunId: p ? store.activeRun(p.id)?.runId ?? null : null,
    };
    return c.json(home);
  });

  api.post("/runs", (c) => {
    const p = playerOf(c);
    if (!p) return bad(c, 401, `unknown player: send ${PLAYER_HEADER} from POST /players`);
    return c.json(runView(startRun(rt, p)));
  });

  // Only the run's player reads it: a round fight's ghostId names the
  // opponent's run, and a slayer's run would show its hidden Crown team.
  api.get("/runs/:runId", (c) => {
    const run = store.run(c.req.param("runId"));
    if (!run) return bad(c, 404, "no such run");
    if (playerOf(c)?.id !== run.player.id) return bad(c, 401, "not your run");
    return c.json(runView(currentRun(rt, run)));
  });

  /** The caller's run and the Decision in `body`, or the error response.
   * Synchronous on purpose: await the body first, then call this and the run
   * engine with no await in between, so parallel decisions on one run apply
   * one after another instead of starting from the same state. */
  const runAndDecision = (c: Context, body: unknown): { run: MvpRunState; d: Decision } | Response => {
    const d = body as Decision | null;
    const p = playerOf(c);
    const run = store.run(c.req.param("runId") ?? "");
    if (!run) return bad(c, 404, "no such run");
    if (!p || p.id !== run.player.id) return bad(c, 401, "not your run");
    if (!d || typeof d !== "object" || typeof d.kind !== "string") return bad(c, 400, "body must be a Decision");
    return { run, d };
  };
  /** A decision the run engine turned down: 400 for one it can't read (an
   * unknown kind), 409 for one the rules refuse; anything else is a bug. */
  const refused = (c: Context, err: unknown) => {
    if (err instanceof MvpBadDecision) return bad(c, 400, err.message);
    if (err instanceof MvpDecisionError) return bad(c, 409, err.message);
    throw err;
  };

  api.post("/runs/:runId/decisions", async (c) => {
    const req = runAndDecision(c, await c.req.json().catch(() => null));
    if (req instanceof Response) return req;
    try {
      return c.json(decide(rt, req.run, req.d));
    } catch (err) {
      return refused(c, err);
    }
  });

  // Giving up: the run ends "abandoned", every heart left rated a lost fight
  // (runs.ts abandon). The owner only; 409 on a run that is already over.
  api.post("/runs/:runId/abandon", (c) => {
    const run = store.run(c.req.param("runId"));
    if (!run) return bad(c, 404, "no such run");
    if (playerOf(c)?.id !== run.player.id) return bad(c, 401, "not your run");
    try {
      return c.json(abandon(rt, run));
    } catch (err) {
      return refused(c, err);
    }
  });

  // A dry run of a shop decision for slice 8's result cards: no writes.
  api.post("/runs/:runId/preview", async (c) => {
    const req = runAndDecision(c, await c.req.json().catch(() => null));
    if (req instanceof Response) return req;
    if (req.d.kind === "fight") return bad(c, 400, "a fight can't be previewed");
    try {
      return c.json(preview(rt, req.run, req.d));
    } catch (err) {
      return refused(c, err);
    }
  });

  api.get("/battles/:battleId", (c) => {
    const b = store.battle(c.req.param("battleId"));
    // A Crown fight that slew today's champion shows its slayer's team: only
    // the slayer sees it before the day ends.
    if (!b || hiddenSlay(rt, b.battleId, playerOf(c)?.id)) return bad(c, 404, "no such battle");
    return c.json(b);
  });

  // Slice 10 owns this route and the store behind it; slice 11 only reads.
  api.get("/fusions", (c) => c.json(store.fusions()));

  api.get("/day", (c) => c.json(dayView(rt)));
  api.get("/stats", (c) => notYet(c, () => statsView(rt)));

  // Dev-only tools (MvpDeps.dev, MVP_DEV=1 in main.ts): 404 without it.
  api.use("/dev/*", async (c, next) => {
    if (!rt.dev) return bad(c, 404, "not found");
    await next();
  });
  api.post("/dev/end-day", (c) => notYet(c, () => endDay(rt)));

  const app = new Hono();
  app.route(MVP_API_PREFIX, api);
  return app;
}
