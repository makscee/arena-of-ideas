// Arena MVP HTTP API (mission #574, slice 1). The routes and shapes are the
// contract in src/mvp/contract.ts; this file only parses requests and calls
// the runtime (./runtime.ts): the run engine (./runs.ts), the day (./day.ts),
// the stats (./stats.ts), ideas (./ideas.ts) and the store. A slice fills in those modules, not
// these routes.
import { randomUUID } from "node:crypto";
import { Hono, type Context } from "hono";
import { bodyLimit } from "hono/body-limit";
import { MVP_API_PREFIX, MVP_API_VERSION, PLAYER_HEADER, TOKEN_HEADER, type Decision, type HomeView, type PlayerRef, type TelegramPoll, type TelegramStatus, type VoteRequest } from "../../../src/mvp/contract.js";
import { checkDecision, MvpBadDecision, MvpDecisionError, runView, type MvpRunState } from "../../../src/mvp/run.js";
import { creditsView, creditUnit, libraryView } from "./credits.js";
import { dayView, endDay, hiddenSlay } from "./day.js";
import { MvpNotYet } from "./errors.js";
import { declineOptions, myIdea, pickArchetype, pickReading } from "./idea-reading.js";
import { cancelIdea, grantIdea, IdeaRefused, ideasOf, myIdeas, writeIdea } from "./ideas.js";
import { isAdmin, isJoinCode, JoinRefused, joinOpen, NAME_RE, openJoin, redeemInvite, sessionPlayer } from "./invites.js";
import { abandon, currentRun, decide, preview, startRun } from "./runs.js";
import { isMvpRuntime, mvpRuntime, type MvpDeps, type MvpRuntime } from "./runtime.js";
import { servedContent } from "./pool.js";
import { statsView } from "./stats.js";
import { FakeTelegram, pollTelegramLogin, startTelegramLogin, TELEGRAM_STARTS_PER_HOUR, TelegramRefused } from "./telegram.js";
import { candidateScores, castVote, fakeVotes, nextCard, overnightCheck, seedCandidate, waitingForCheck } from "./votes.js";

/** New players the open join link (R4-20) makes per hour, server-wide. */
export const JOINS_PER_HOUR = 30;

/** The API on a runtime, or on a fresh one built from `deps`. */
export function createMvpApp(deps: MvpDeps | MvpRuntime): Hono {
  const rt = isMvpRuntime(deps) ? deps : mvpRuntime(deps);
  const { store } = rt;
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
  // the invite routes and a Telegram login: 401 "unknown player", so the device forgets it and
  // shows the invite screen.
  api.use("*", async (c, next) => {
    const token = c.req.header(TOKEN_HEADER);
    if (token && !/\/(invites|join|auth\/telegram\/(start|poll))(\/|$)/.test(c.req.path) && !sessionPlayer(store, token)) return unknownPlayer(c);
    await next();
  });
  // A decision's body is read only from a known player.
  for (const path of ["/runs/:runId/decisions", "/runs/:runId/preview"])
    api.post(path, async (c, next) => {
      if (!playerOf(c)) return unknownPlayer(c);
      await next();
    });

  api.get("/health", (c) => c.json({ ok: true, api: MVP_API_VERSION, contentVersion: rt.content.version, invites: rt.invites, open: rt.open, telegram: rt.telegram ? (rt.telegram instanceof FakeTelegram ? "fake" : "bot") : false }));
  api.get("/content", (c) => c.json(servedContent(rt)));

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

  // M4-6: log in with Telegram (./telegram.ts). Starts are counted per player,
  // or per address logged out (the nearest proxy's X-Forwarded-For entry), in memory.
  const tgStarts = new Map<string, number[]>();
  const addressOf = (c: Context) => {
    const hops = (c.req.header("x-forwarded-for") ?? "").split(",").map((h) => h.trim()).filter(Boolean);
    return hops.at(-1) ?? (c.env as { incoming?: { socket?: { remoteAddress?: string } } } | undefined)?.incoming?.socket?.remoteAddress ?? "local";
  };
  const tgStatus = (p: PlayerRef): TelegramStatus => ({ enabled: !!rt.telegram, linked: !!store.telegramLink({ playerId: p.id }) });
  api.post("/auth/telegram/start", (c) => {
    if (!rt.telegram) return bad(c, 404, "Telegram login is off");
    const p = playerOf(c);
    const key = p ? `player:${p.id}` : `address:${addressOf(c)}`;
    const now = rt.now();
    const recent = (tgStarts.get(key) ?? []).filter((t) => t > now.getTime() - 3_600_000);
    if (recent.length >= TELEGRAM_STARTS_PER_HOUR) return bad(c, 429, "Too many Telegram logins this hour. Try again later.");
    try {
      const start = startTelegramLogin(store, p, rt.telegram.bot, now);
      tgStarts.set(key, [...recent, now.getTime()]);
      return c.json(start);
    } catch (err) {
      if (err instanceof TelegramRefused) return bad(c, err.status, err.message);
      throw err;
    }
  });
  api.post("/auth/telegram/poll", async (c) => {
    const r = pollTelegramLogin(store, await codeOf(c), rt.now());
    if (!r) return bad(c, 404, "This login has expired or was used: start again");
    return c.json<TelegramPoll>(typeof r === "string" ? { status: r } : { status: "done", ...r });
  });
  api.get("/auth/telegram", (c) => {
    const p = playerOf(c);
    return p ? c.json(tgStatus(p)) : unknownPlayer(c);
  });
  api.post("/auth/telegram/unlink", (c) => {
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
    store.unlinkTelegram(p.id);
    return c.json(tgStatus(p));
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
      telegram: p ? tgStatus(p) : null,
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
    const body = (await c.req.json().catch(() => null)) as { text?: unknown; kind?: unknown; target?: unknown } | null;
    if (typeof body?.text !== "string") return bad(c, 400, "body must be { text }");
    const { text, kind, target } = body;
    if (kind !== undefined && kind !== "new" && kind !== "evolve") return bad(c, 400, 'kind is "new" or "evolve"');
    // M3-4: a new version of a Library unit.
    if (kind === "evolve" && typeof target !== "string") return bad(c, 400, "body must be { text, kind: \"evolve\", target }");
    return ideaCall(c, (p) => writeIdea(rt, p.id, text, kind === "evolve" ? { target: target as string } : undefined));
  });
  api.post("/ideas/:ideaId/cancel", (c) => ideaCall(c, (p) => cancelIdea(rt, p.id, c.req.param("ideaId"))));
  // M2-5: one idea with its options, and its author's picks (M2-6's screens).
  api.get("/ideas/:ideaId", (c) => {
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
    try {
      return c.json(myIdea(rt, p.id, c.req.param("ideaId")));
    } catch (err) {
      if (err instanceof IdeaRefused) return bad(c, err.status, err.message);
      throw err;
    }
  });
  const pick = (fn: typeof pickArchetype) => async (c: Context) => {
    const body = (await c.req.json().catch(() => null)) as { index?: unknown } | null;
    if (typeof body?.index !== "number") return bad(c, 400, "body must be { index }");
    const index = body.index;
    return ideaCall(c, (p) => fn(rt, p.id, c.req.param("ideaId")!, index));
  };
  api.post("/ideas/:ideaId/archetype", pick(pickArchetype));
  api.post("/ideas/:ideaId/reading", pick(pickReading));
  // M2-6: "None of these" at either pick.
  api.post("/ideas/:ideaId/none", (c) => ideaCall(c, (p) => declineOptions(rt, p.id, c.req.param("ideaId"))));

  // M2-8's either/or cards: a candidate against a typical live unit.
  api.get("/votes/next", (c) => {
    const p = playerOf(c);
    return p ? c.json({ card: nextCard(rt, p) }) : unknownPlayer(c);
  });
  api.post("/votes", async (c) => {
    const body = (await c.req.json().catch(() => null)) as Partial<VoteRequest> | null;
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
    if (typeof body?.candidateId !== "string" || typeof body.otherId !== "string" || !(body.pick === null || typeof body.pick === "string"))
      return bad(c, 400, "body must be { candidateId, otherId, pick }");
    try {
      castVote(rt, p, { candidateId: body.candidateId, otherId: body.otherId, pick: body.pick });
    } catch (err) {
      if (err instanceof IdeaRefused) return bad(c, err.status, err.message);
      throw err;
    }
    return c.json({ card: nextCard(rt, p) });
  });

  // Slice 10 owns this route and the store behind it; slice 11 only reads.
  api.get("/fusions", (c) => c.json(store.fusions()));

  api.get("/day", (c) => c.json(dayView(rt)));
  api.get("/stats", (c) => notYet(c, () => statsView(rt)));
  // M2-9: who each live unit's idea was, NEW, the caller's creator number;
  // and the units that have left.
  api.get("/credits", (c) => c.json(creditsView(rt, playerOf(c)?.id)));
  api.get("/library", (c) => c.json(libraryView(rt)));

  // The fake bot's side (ARENA_TELEGRAM_FAKE=1, which needs MVP_DEV=1): its
  // user sends /start <code>, as tapping the link in Telegram would, then taps
  // Yes (or No: `decline`). Before the dev gate: a logged-out device plays it
  // on an invite-only dev server too.
  api.post("/dev/telegram/accept", async (c) => {
    if (!(rt.telegram instanceof FakeTelegram)) return bad(c, 404, "no fake Telegram");
    const body = (await c.req.json().catch(() => ({}))) as { code?: unknown; decline?: unknown };
    return c.json({ reply: await rt.telegram.answer(typeof body.code === "string" ? body.code : "", body.decline !== true) });
  });
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
  api.post("/dev/credit-unit", async (c) => {
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
    const body = (await c.req.json().catch(() => null)) as { unitId?: unknown } | null;
    // No unitId: a tier I unit, so the shop offers it soon.
    const unitId = typeof body?.unitId === "string" ? body.unitId : rt.content.units.find((u) => u.tier === 1)?.id;
    if (!unitId || !creditUnit(rt, unitId, p.id)) return bad(c, 404, "no such live unit");
    return c.json(creditsView(rt, p.id));
  });

  // M2-8: the overnight check now (the quick meta check), a dev candidate,
  // fake votes, and the candidates' standing.
  api.post("/dev/overnight-check", (c) => {
    const started = waitingForCheck(store);
    overnightCheck(rt, rt.tuner.dev).catch((err) => console.error("[overnight] dev check failed", err));
    return c.json({ started });
  });
  api.post("/dev/seed-candidate", async (c) => {
    const body = (await c.req.json().catch(() => null)) as { unit?: unknown } | null;
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
    seedCandidate(rt, p.id, typeof body?.unit === "string" ? body.unit : undefined);
    return c.json(myIdeas(rt, p.id));
  });
  api.post("/dev/fake-votes", (c) => c.json(fakeVotes(rt)));
  api.get("/dev/candidates", (c) => c.json(candidateScores(rt)));

  const app = new Hono();
  app.route(MVP_API_PREFIX, api);
  return app;
}
