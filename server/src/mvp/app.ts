// Arena MVP HTTP API (mission #574, slice 1). The routes and shapes are the
// contract in src/mvp/contract.ts; this file only parses requests and calls
// the runtime (./runtime.ts): the run engine (./runs.ts), the day (./day.ts),
// the stats (./stats.ts), ideas (./ideas.ts) and the store. A slice fills in those modules, not
// these routes.
import { randomUUID } from "node:crypto";
import { Hono, type Context } from "hono";
import { bodyLimit } from "hono/body-limit";
import { MVP_API_PREFIX, MVP_API_VERSION, PLAYER_HEADER, TOKEN_HEADER, type Decision, type HomeView, type PlayerRef } from "../../../src/mvp/contract.js";
import { checkDecision, MvpBadDecision, MvpDecisionError, runView, type MvpRunState } from "../../../src/mvp/run.js";
import { dayView, endDay, hiddenSlay } from "./day.js";
import { MvpNotYet } from "./errors.js";
import { cancelIdea, grantIdea, IdeaRefused, ideasOf, myIdeas, writeIdea } from "./ideas.js";
import { isAdmin, isJoinCode, JoinRefused, joinOpen, NAME_RE, openJoin, redeemInvite, sessionPlayer } from "./invites.js";
import { abandon, currentRun, decide, preview, startRun } from "./runs.js";
import { isMvpRuntime, mvpRuntime, type MvpDeps, type MvpRuntime } from "./runtime.js";
import { statsView } from "./stats.js";

/** New players the open join link (R4-20) makes per hour, server-wide. */
export const JOINS_PER_HOUR = 30;

/** The API on a runtime, or on a fresh one built from `deps`. */
export function createMvpApp(deps: MvpDeps | MvpRuntime): Hono {
  const rt = isMvpRuntime(deps) ? deps : mvpRuntime(deps);
  const { content, store } = rt;
  const api = new Hono();

  const bad = (c: Context, status: 400 | 401 | 403 | 404 | 409 | 413 | 429 | 501, error: string) => c.json({ error }, status);
  /** A stub that a later slice fills in answers 501 until then. */
  const notYet = (c: Context, fn: () => unknown) => {
    try {
      return c.json(fn());
    } catch (err) {
      if (err instanceof MvpNotYet) return bad(c, 501, err.message);
      throw err;
    }
  };
  // A session token (slice 13) names the player anywhere; the bare player id
  // only on an open server, since ids are public (the day, fusions, battles).
  const playerOf = (c: Context): PlayerRef | undefined => {
    const token = c.req.header(TOKEN_HEADER);
    if (token) return sessionPlayer(store, token);
    const id = rt.invites ? undefined : c.req.header(PLAYER_HEADER);
    return id ? store.player(id) : undefined;
  };
  const unknownPlayer = (c: Context) => bad(c, 401, rt.invites ? "unknown player: open your invite link" : `unknown player: send ${PLAYER_HEADER} from POST /players`);
  /** The dev tools: anyone on an open dev server, admin invites on an invite-only one. */
  const devFor = (c: Context) => rt.dev && (!rt.invites || isAdmin(store, playerOf(c)?.id));

  // Every body here is a name, a Decision or an idea: a few hundred bytes (an
  // idea's 400 characters, a few KB at most).
  api.use("*", bodyLimit({ maxSize: 16 * 1024, onError: (c) => bad(c, 413, "body too large") }));
  // A token that names no session (a revoked link) is no player anywhere but
  // the invite routes: 401 "unknown player", so the device forgets it and
  // shows the invite screen.
  api.use("*", async (c, next) => {
    const token = c.req.header(TOKEN_HEADER);
    if (token && !/\/(invites|join)(\/|$)/.test(c.req.path) && !sessionPlayer(store, token)) return unknownPlayer(c);
    await next();
  });
  // A decision's body is read only from a known player.
  for (const path of ["/runs/:runId/decisions", "/runs/:runId/preview"])
    api.post(path, async (c, next) => {
      if (!playerOf(c)) return unknownPlayer(c);
      await next();
    });

  api.get("/health", (c) => c.json({ ok: true, api: MVP_API_VERSION, contentVersion: content.version, invites: rt.invites, open: rt.open }));
  api.get("/content", (c) => c.json(content));

  api.post("/players", async (c) => {
    if (rt.invites) return bad(c, 403, "invite only: open your invite link");
    const body = (await c.req.json().catch(() => null)) as { name?: unknown } | null;
    const name = typeof body?.name === "string" ? body.name.trim() : "";
    if (!NAME_RE.test(name)) return bad(c, 400, "name: 1–24 letters, digits, spaces, _ or -");
    const p: PlayerRef = { id: randomUUID(), name, bot: false };
    store.addPlayer(p);
    return c.json(p);
  });

  // Invite codes come in the body, never the path: the proxies in front
  // (tailscale serve, mcow's Caddy) log every URI.
  const codeOf = async (c: Context) => {
    const body = (await c.req.json().catch(() => null)) as { code?: unknown } | null;
    return typeof body?.code === "string" ? body.code : "";
  };
  // Whose link this is, without opening it (the client asks before a device
  // switches from another player).
  api.post("/invites/lookup", async (c) => {
    const invite = store.invite(await codeOf(c));
    const player = invite && store.player(invite.playerId);
    return player ? c.json({ player }) : bad(c, 404, "no such invite");
  });
  // Opens an invite link: its player and a new token for this device.
  api.post("/invites/redeem", async (c) => {
    const session = redeemInvite(store, await codeOf(c), rt.now());
    return session ? c.json(session) : bad(c, 404, "no such invite");
  });

  // R4-20: the open join link. Anyone with its code picks a name and becomes a
  // new player (never admin), at most JOINS_PER_HOUR new players an hour for
  // the whole server, counted here in memory.
  // Open to all (rt.open): no code joins too, through the join code.
  const joins: number[] = [];
  const joinWith = (code: string) => (rt.open && code === "" ? openJoin(store) : code);
  api.post("/join/check", async (c) => (isJoinCode(store, joinWith(await codeOf(c))) ? c.json({ ok: true }) : bad(c, 404, "no such join link")));
  api.post("/join", async (c) => {
    const body = (await c.req.json().catch(() => null)) as { code?: unknown; name?: unknown } | null;
    const code = joinWith(typeof body?.code === "string" ? body.code : "");
    const name = typeof body?.name === "string" ? body.name : "";
    if (!isJoinCode(store, code)) return bad(c, 404, "no such join link");
    const now = rt.now();
    while (joins.length && joins[0]! <= now.getTime() - 3_600_000) joins.shift();
    if (joins.length >= JOINS_PER_HOUR) return bad(c, 429, "Too many new players this hour. Try again later.");
    try {
      const session = joinOpen(store, code, name, now);
      if (!session) return bad(c, 404, "no such join link");
      joins.push(now.getTime());
      return c.json(session);
    } catch (err) {
      if (err instanceof JoinRefused) return bad(c, err.status, err.message);
      throw err;
    }
  });

  api.get("/home", (c) => {
    const p = playerOf(c);
    const home: HomeView = {
      rules: rt.rules,
      day: dayView(rt),
      rating: p ? store.rating(p.id) ?? { player: p, rating: rt.rules.ratingStart, runs: 0, slays: 0, daysAsChampion: 0, playoffWins: 0 } : null,
      activeRunId: p ? store.activeRun(p.id)?.runId ?? null : null,
      dev: devFor(c),
      ideas: p ? ideasOf(rt, p.id) : null,
    };
    return c.json(home);
  });

  api.post("/runs", (c) => {
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
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
    // Kind and indexes before anything reads the run with them (runs.ts's
    // fuse naming indexes the line first).
    try {
      checkDecision(d);
    } catch (err) {
      if (err instanceof MvpBadDecision) return bad(c, 400, err.message);
      throw err;
    }
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

  // Mission 2's ideas (M2-4): each player reads and writes only their own;
  // no route ever returns another player's text.
  const ideaCall = (c: Context, fn: (p: PlayerRef) => void) => {
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
    try {
      fn(p);
    } catch (err) {
      if (err instanceof IdeaRefused) return bad(c, err.status, err.message);
      throw err;
    }
    return c.json(myIdeas(rt, p.id));
  };
  api.get("/ideas", (c) => ideaCall(c, () => {}));
  api.post("/ideas", async (c) => {
    const body = (await c.req.json().catch(() => null)) as { text?: unknown } | null;
    if (typeof body?.text !== "string") return bad(c, 400, "body must be { text }");
    const text = body.text;
    return ideaCall(c, (p) => writeIdea(rt, p.id, text));
  });
  api.post("/ideas/:ideaId/cancel", (c) => ideaCall(c, (p) => cancelIdea(rt, p.id, c.req.param("ideaId"))));

  // Slice 10 owns this route and the store behind it; slice 11 only reads.
  api.get("/fusions", (c) => c.json(store.fusions()));

  api.get("/day", (c) => c.json(dayView(rt)));
  api.get("/stats", (c) => notYet(c, () => statsView(rt)));

  // Dev-only tools (MvpDeps.dev, MVP_DEV=1 in main.ts): 404 without it, and
  // on an invite-only server for anyone but an admin invite's player.
  api.use("/dev/*", async (c, next) => {
    if (!devFor(c)) return bad(c, 404, "not found");
    await next();
  });
  api.post("/dev/end-day", (c) => notYet(c, () => endDay(rt)));
  api.post("/dev/grant-idea", (c) => {
    const p = playerOf(c);
    return p ? c.json(grantIdea(rt, p.id)) : unknownPlayer(c);
  });

  const app = new Hono();
  app.route(MVP_API_PREFIX, api);
  return app;
}
