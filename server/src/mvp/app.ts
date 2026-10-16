// Arena MVP HTTP API (mission #574, slice 1). The routes and shapes are the
// contract in src/mvp/contract.ts; this file only parses requests and calls
// the runtime (./runtime.ts): the run engine (./runs.ts), the day (./day.ts),
// the stats (./stats.ts), ideas (./ideas.ts) and the store. A slice fills in those modules, not
// these routes.
import { checkpointRun } from "./checkpoint.js";
import { randomUUID } from "node:crypto";
import { Hono, type Context } from "hono";
import { bodyLimit } from "hono/body-limit";
import { MVP_API_PREFIX, MVP_API_VERSION, PLAYER_HEADER, TOKEN_HEADER, type Decision, type HomeView, type PlayerRef, type EmailVerify, type TelegramPoll, type TelegramStatus, type VoteRequest } from "../../../src/mvp/contract.js";
import { checkDecision, MvpBadDecision, MvpDecisionError, runView, type MvpRunState } from "../../../src/mvp/run.js";
import { creditsView, creditUnit, libraryView } from "./credits.js";
import { dayView, endDay, hiddenSlay } from "./day.js";
import { MvpNotYet } from "./errors.js";
import { declineOptions, myIdea, pickArchetype, pickReading } from "./idea-reading.js";
import { cancelIdea, grantIdea, IdeaRefused, ideasOf, myIdeas, writeIdea } from "./ideas.js";
import { CRUDE_NAME, isAdmin, isCrudePlayerName, isJoinCode, JoinRefused, joinOpen, NAME_RE, openJoin, redeemInvite, sessionPlayer } from "./invites.js";
import { abandon, currentRun, decide, preview, startRun } from "./runs.js";
import { codexPair, sortPair } from "../../../src/mvp/codex.js";
import { codexView, runForge, wish } from "./codex.js";
import { codexOverview } from "./codex-view.js";
import { isMvpRuntime, mvpRuntime, type MvpDeps, type MvpRuntime } from "./runtime.js";
import { servedContent } from "./pool.js";
import { championSvg, shareLang, sharePng, unitSvg } from "./share.js";
import { statsView } from "./stats.js";
import { attachEmail, EMAIL_STARTS_PER_ADDRESS_HOUR, EMAIL_STARTS_PER_HOUR, EmailRefused, emailRequired, emailRequiredFor, emailStatus, FakeMailer, startEmailLogin, verifyEmailLogin } from "./email.js";
import { FakeTelegram, pollTelegramLogin, startTelegramLogin, TELEGRAM_STARTS_PER_HOUR, TelegramRefused } from "./telegram.js";
import { buildScene, sceneById, sceneDesigns, SceneMissing, sceneList } from "./scenes.js";
import { withSceneFixtures } from "./scene-fixtures.js";
import { CrazyGamesRefused, guestLimiter, joinGuest, loginCrazyGames, verifyCgToken } from "./crazygames.js";
import { fileReport, ReportRefused } from "./reports.js";
import { candidateScores, castVote, fakeVotes, nextCard, overnightCheck, seedCandidate, waitingForCheck } from "./votes.js";

/** New players the open join link (R4-20) makes per hour, server-wide. */
export const JOINS_PER_HOUR = 30;

/** The API on a runtime, or on a fresh one built from `deps`. */
export function createMvpApp(deps: MvpDeps | MvpRuntime): Hono {
  const rt = isMvpRuntime(deps) ? deps : mvpRuntime(deps);
  const { store } = rt;
  const api = new Hono();

  const bad = (c: Context, status: 400 | 401 | 403 | 404 | 409 | 413 | 429 | 501 | 502, error: string) => c.json({ error }, status);
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
    if (token && !/\/(invites|join|players\/(guest|crazygames)|auth\/telegram\/(start|poll)|auth\/email\/(start|verify))(\/|$)/.test(c.req.path) && !sessionPlayer(store, token)) return unknownPlayer(c);
    await next();
  });
  // A decision's body is read only from a known player.
  for (const path of ["/runs/:runId/decisions", "/runs/:runId/preview"])
    api.post(path, async (c, next) => {
      if (!playerOf(c)) return unknownPlayer(c);
      await next();
    });

  api.get("/health", (c) => c.json({ ok: true, api: MVP_API_VERSION, contentVersion: rt.content.version, invites: rt.invites, open: rt.open, codex: rt.codex, cg: rt.cg, telegram: rt.telegram ? (rt.telegram instanceof FakeTelegram ? "fake" : "bot") : false, mail: rt.mail ? (rt.mail instanceof FakeMailer ? "fake" : "smtp") : false }));
  // A scenes server also serves the rules v2 fixtures' words (#1045, scene-fixtures.ts).
  api.get("/content", (c) => c.json(rt.scenes ? withSceneFixtures(servedContent(rt)) : servedContent(rt)));

  api.post("/players", async (c) => {
    if (rt.invites) return bad(c, 403, "invite only: open your invite link");
    const body = (await c.req.json().catch(() => null)) as { name?: unknown } | null;
    const name = typeof body?.name === "string" ? body.name.trim() : "";
    if (!NAME_RE.test(name)) return bad(c, 400, "name: 1–24 letters, digits, spaces, _ or -");
    if (isCrudePlayerName(name)) return bad(c, 400, CRUDE_NAME);
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
    if (emailRequired(rt)) return bad(c, 403, "Sign up with your email.");
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

  // The client's address: the header the front proxy sets (ARENA_CLIENT_IP_HEADER,
  // live: mcow's Caddy, since tailscale serve rewrites X-Forwarded-For), only
  // when the server is told to trust it; else the nearest proxy's entry.
  const addressOf = (c: Context) => {
    const told = rt.clientIpHeader ? c.req.header(rt.clientIpHeader)?.trim() : undefined;
    if (told) return told;
    const hops = (c.req.header("x-forwarded-for") ?? "").split(",").map((h) => h.trim()).filter(Boolean);
    return hops.at(-1) ?? (c.env as { incoming?: { socket?: { remoteAddress?: string } } } | undefined)?.incoming?.socket?.remoteAddress ?? "local";
  };
  // #929: email login (./email.ts). Codes mailed are counted per address
  // and for the whole server, an hour, in memory; new players share the join
  // link's hourly limit.
  const mailStarts = new Map<string, number[]>();
  let allMailStarts: number[] = [];
  const hourOf = (list: number[], now: number) => list.filter((t) => t > now - 3_600_000);
  const emailCall = async (c: Context, fn: () => unknown | Promise<unknown>) => {
    try {
      return c.json((await fn()) as object);
    } catch (err) {
      if (err instanceof EmailRefused) return bad(c, err.status, err.message);
      throw err;
    }
  };
  const emailBody = async (c: Context) => (await c.req.json().catch(() => null)) as { email?: unknown; code?: unknown; name?: unknown; lang?: unknown; join?: unknown } | null;
  const str = (x: unknown) => (typeof x === "string" ? x : "");
  api.post("/auth/email/start", async (c) => {
    const mail = rt.mail;
    if (!mail) return bad(c, 404, "Email login is off");
    const body = await emailBody(c);
    const email = str(body?.email);
    const name = str(body?.name).trim();
    if (name && !NAME_RE.test(name)) return bad(c, 400, "name: 1–24 letters, digits, spaces, _ or -");
    const lang = body?.lang === "ru" || body?.lang === "en" ? body.lang : shareLang(undefined, c.req.header("Accept-Language"));
    const now = rt.now().getTime();
    const key = addressOf(c);
    const mine = hourOf(mailStarts.get(key) ?? [], now);
    allMailStarts = hourOf(allMailStarts, now);
    if (mine.length >= EMAIL_STARTS_PER_ADDRESS_HOUR || allMailStarts.length >= EMAIL_STARTS_PER_HOUR) return bad(c, 429, "Too many codes this hour. Try again later.");
    return emailCall(c, async () => {
      const r = await startEmailLogin(store, mail, email, playerOf(c), lang, rt.now());
      mailStarts.set(key, [...mine, now]);
      allMailStarts.push(now);
      return { ok: true, ...r };
    });
  });
  api.post("/auth/email/verify", async (c) => {
    if (!rt.mail) return bad(c, 404, "Email login is off");
    const body = await emailBody(c);
    const name = str(body?.name);
    // A new player: anyone on an open server, the join code's holder on an
    // invite-only one (open to all: no code needed, as for the join link).
    const canSignUp = !rt.invites || isJoinCode(store, joinWith(str(body?.join)));
    const now = rt.now();
    const admit = () => {
      while (joins.length && joins[0]! <= now.getTime() - 3_600_000) joins.shift();
      if (joins.length >= JOINS_PER_HOUR) return false;
      joins.push(now.getTime());
      return true;
    };
    return emailCall(c, (): EmailVerify => verifyEmailLogin(store, str(body?.email), str(body?.code), name || undefined, now, { canSignUp, admit }));
  });
  api.post("/auth/email/attach", async (c) => {
    if (!rt.mail) return bad(c, 404, "Email login is off");
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
    const body = await emailBody(c);
    return emailCall(c, () => attachEmail(rt, p, str(body?.email), str(body?.code), rt.now()));
  });
  api.get("/auth/email", (c) => {
    const p = playerOf(c);
    return p ? c.json(emailStatus(rt, p.id)) : unknownPlayer(c);
  });

  // #1218: CrazyGames players (./crazygames.ts), on a server with ARENA_CG=1
  // (rt.cg), whatever its invites: a guest at once, or the player of a
  // verified CrazyGames account. New players here have their own hourly
  // limits, per address and for the whole server (rt.guestJoins), apart from
  // the join link's.
  const guestJoins = guestLimiter(rt.guestJoins);
  const admitGuest = (c: Context) => guestJoins.admit(addressOf(c), rt.now().getTime());
  const tooManyGuests = (c: Context) => bad(c, 429, "Too many new players from here this hour. Try again later.");
  const cgOff = (c: Context) => bad(c, 404, "CrazyGames play is off");
  api.post("/players/guest", (c) => (!rt.cg ? cgOff(c) : admitGuest(c) ? c.json(joinGuest(store, rt.now())) : tooManyGuests(c)));
  api.post("/players/crazygames", async (c) => {
    if (!rt.cg) return cgOff(c);
    const body = (await c.req.json().catch(() => null)) as { token?: unknown } | null;
    const token = str(body?.token);
    if (!token) return bad(c, 400, "token: the CrazyGames user token");
    let claims;
    try {
      claims = await verifyCgToken(token, rt.cgKeys, rt.now(), rt.cgGameId);
    } catch (err) {
      if (err instanceof CrazyGamesRefused) return bad(c, err.status, err.message);
      console.error("crazygames login:", err instanceof Error ? err.message : err);
      return bad(c, 502, "CrazyGames login is unavailable: try again later");
    }
    // This device's guest only by its session token: a bare player id
    // (playerOf on an open server) is public and would hand over anyone's player.
    const deviceToken = c.req.header(TOKEN_HEADER);
    const r = loginCrazyGames(store, claims, deviceToken ? sessionPlayer(store, deviceToken) : undefined, rt.now(), () => admitGuest(c));
    return r ? c.json(r) : tooManyGuests(c);
  });

  // #1271: a report of a player's name or a player-made unit (./reports.ts): stored, nothing hidden.
  api.post("/reports", async (c) => {
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
    try {
      fileReport(rt, p, (await c.req.json().catch(() => null)) as { kind?: unknown; id?: unknown; reason?: unknown } | null);
      return c.json({ ok: true });
    } catch (err) {
      if (err instanceof ReportRefused) return bad(c, err.status, err.message);
      throw err;
    }
  });

  // M4-6: log in with Telegram (./telegram.ts). Starts are counted per player,
  // or per address logged out (the nearest proxy's X-Forwarded-For entry), in memory.
  const tgStarts = new Map<string, number[]>();
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
      email: p ? emailStatus(rt, p.id) : null,
    };
    return c.json(home);
  });

  api.post("/runs", (c) => {
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
    if (emailRequiredFor(rt, p.id) && !p.bot && !store.emailLink({ playerId: p.id })) return bad(c, 403, "Add your email first.");
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

  // Rules v2's Codex (#1049, ./codex.ts): the forge of a run (every pair it
  // can fuse now, in its state; a ready pair's three designs), a pair's view
  // (sealed), and a wish on an unforged pair. 404 while the Codex is off.
  api.get("/runs/:runId/forge", (c) => {
    if (!rt.codex) return bad(c, 404, "the Codex is off");
    const run = store.run(c.req.param("runId"));
    if (!run) return bad(c, 404, "no such run");
    if (playerOf(c)?.id !== run.player.id) return bad(c, 401, "not your run");
    return c.json(runForge(rt, run));
  });
  api.get("/codex", (c) => (rt.codex ? c.json(codexOverview(rt, playerOf(c))) : bad(c, 404, "the Codex is off")));
  api.get("/codex/:a/:b", (c) => (rt.codex ? c.json(codexView(rt, c.req.param("a"), c.req.param("b"), playerOf(c))) : bad(c, 404, "the Codex is off")));
  api.post("/codex/wish", async (c) => {
    if (!rt.codex) return bad(c, 404, "the Codex is off");
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
    const body = (await c.req.json().catch(() => null)) as { a?: unknown; b?: unknown } | null;
    if (!body || typeof body.a !== "string" || typeof body.b !== "string") return bad(c, 400, "body must be { a, b }: two unit ids");
    try {
      return c.json(wish(rt, p, body.a, body.b));
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

  // The fake mailer's inbox (ARENA_MAIL_FAKE=1, which needs MVP_DEV=1): the
  // last code mailed to an email. Before the dev gate, like the fake bot: a
  // logged-out device signs up with it on an invite-only dev server too.
  api.post("/dev/mail/last", async (c) => {
    if (!(rt.mail instanceof FakeMailer)) return bad(c, 404, "no fake mail");
    const body = await emailBody(c);
    const m = rt.mail.last(str(body?.email));
    return m ? c.json({ code: rt.mail.lastCode(m.to), subject: m.subject, text: m.text }) : bad(c, 404, "no mail to this email");
  });

  // M4-7: share cards, PNGs for link previews and the daily post. Rendered
  // once per content (share.ts caches by the SVG's hash, the ETag here).
  const png = (c: Context, svg: string | null) => {
    if (!svg) return bad(c, 404, "nothing to share");
    const { png: body, hash } = sharePng(svg);
    const etag = `"${hash}"`;
    const headers = { "Content-Type": "image/png", "Cache-Control": "public, max-age=600", ETag: etag };
    if (c.req.header("If-None-Match") === etag) return c.body(null, 304, headers);
    return c.body(new Uint8Array(body), 200, headers);
  };
  const langOf = (c: Context) => shareLang(c.req.query("lang"), c.req.header("Accept-Language"));
  api.get("/share/champion/:file", (c) => {
    const m = /^(\d{1,9})\.png$/.exec(c.req.param("file"));
    return png(c, m ? championSvg(rt, Number(m[1]), langOf(c)) : null);
  });
  api.get("/share/unit/:file", (c) => {
    const m = /^(.{1,80})\.png$/.exec(c.req.param("file"));
    return png(c, m ? unitSvg(rt, m[1]!, langOf(c)) : null);
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
  // #1026: the test scenes, on a scenes server only (ARENA_SCENES=1, main.ts).
  api.get("/dev/scenes", (c) => (rt.scenes ? c.json(sceneList()) : bad(c, 404, "not found")));
  api.post("/dev/scenes/:id", async (c) => {
    const scene = rt.scenes ? sceneById(c.req.param("id")) : undefined;
    if (!scene) return bad(c, 404, rt.scenes ? "no such scene" : "not found");
    try {
      return c.json(await buildScene(rt, scene));
    } catch (err) {
      if (err instanceof SceneMissing) return bad(c, 409, err.message);
      throw err;
    }
  });
  api.get("/dev/candidates", (c) => c.json(candidateScores(rt)));
  // #1049: a pair's Codex state by hand (the scenes' stand-in designs), so an
  // e2e can fuse under rules v2 before a night batch ran.
  api.post("/dev/codex-pair", async (c) => {
    if (!rt.codex) return bad(c, 404, "the Codex is off");
    const body = (await c.req.json().catch(() => null)) as { a?: unknown; b?: unknown; state?: unknown } | null;
    const [x, y] = [body?.a, body?.b];
    const state = body?.state === "discovered" ? "discovered" : "ready";
    if (typeof x !== "string" || typeof y !== "string" || x === y || !rt.content.units.some((u) => u.id === x) || !rt.content.units.some((u) => u.id === y)) return bad(c, 400, "a and b: two live unit ids");
    const [a, b] = sortPair(x, y);
    const at = rt.now().toISOString();
    rt.store.putCodexPair({ ...codexPair(a, b), candidates: sceneDesigns(rt, a, b), preparedAt: at });
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
    // "discovered": claimed by the caller (pick 0).
    if (state === "discovered") rt.store.claimCodexPair(a, b, { pick: 0, by: p, at });
    return c.json(codexView(rt, a, b, playerOf(c)));
  });

  // #1065: the caller's run at the moment before discovering a+b (./checkpoint.ts).
  api.post("/dev/checkpoint", async (c) => {
    const p = playerOf(c);
    if (!p) return unknownPlayer(c);
    const body = (await c.req.json().catch(() => null)) as { a?: unknown; b?: unknown } | null;
    const run = typeof body?.a === "string" && typeof body?.b === "string" ? checkpointRun(rt, p, body.a, body.b) : undefined;
    return run ? c.json({ runId: run.runId }) : bad(c, 400, "a and b: two live unit ids");
  });

  const app = new Hono();
  app.route(MVP_API_PREFIX, api);
  return app;
}
