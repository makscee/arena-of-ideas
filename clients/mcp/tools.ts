// The tools of arena-mcp (#959): what an agent does in Arena, over
// clients/core. Every tool answers in text; the client is an agent client
// (X-Arena-Client: agent/mcp, #958), so the server itself refuses ranked moves
// and `decide` says so in words before it asks. No Node imports: main.ts
// brings the session file and stdin.
import {
  ApiError,
  ArenaClient,
  apiBase,
  battleText,
  cardText,
  fightText,
  glossaryText,
  homeText,
  runText,
  traceText,
  unitName,
  type TextOpts,
} from "../core/index.js";
import { GLOSSARY, termDef, type FixedTermId } from "../../src/glossary.js";
import type { Decision, MvpContent, MvpRules, RunView } from "../../src/mvp/contract.js";

export const DEFAULT_URL = "https://arena.makscee.ru";
export const CLIENT = "agent/mcp";

/** What survives between runs of the server (main.ts keeps it in a 0600 file). */
export interface Session {
  token?: string;
  /** The address `login` mailed a code to, for `login_code`. */
  email?: string;
  name?: string;
}
export interface SessionStore {
  load(): Session;
  save(s: Session): void;
}

export interface ToolContext {
  /** The server's root URL (`ARENA_URL` may name the API itself, ".../api/v1"). */
  url: string;
  /** `ARENA_TOKEN`: wins over the saved session. */
  token?: string | undefined;
  store: SessionStore;
  fetch?: typeof fetch;
  /** X-Arena-Client, an agent's (default agent/mcp; the Claude Code mod's is agent/claude-code). */
  client?: string;
}

export interface Tool {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
  run(args: Record<string, unknown>): Promise<string>;
}

/** A refusal the agent should read and act on, not a crash. */
class Say extends Error {}

const str = (v: unknown): string => (typeof v === "string" ? v.trim() : "");
const langOf = (a: Record<string, unknown>): TextOpts => (a.lang === "ru" ? { lang: "ru" } : {});
const LANG = { type: "string", enum: ["en", "ru"], description: "Text language (default en)." };
const RUN_ID = { type: "string", description: "A run id (default: your practice run)." };

const DECISION = {
  description:
    'One move, as an object. {"kind":"buy","slot":N} (a SHOP number) · {"kind":"sell","index":N} (a LINE/BENCH number) · {"kind":"reroll"} · {"kind":"lock","slot":N} (toggles a shop offer) · {"kind":"reorder","from":N,"to":N} · {"kind":"fuse","first":N,"second":N} (two Awoken units) · {"kind":"gift","pick":N|null} · {"kind":"fight"}. Numbers are the [n] in the run text.',
  type: "object",
  properties: {
    kind: { type: "string", enum: ["buy", "sell", "reroll", "lock", "reorder", "fuse", "gift", "fight"] },
    slot: { type: "integer" },
    index: { type: "integer" },
    from: { type: "integer" },
    to: { type: "integer" },
    first: { type: "integer" },
    second: { type: "integer" },
    pick: { type: ["integer", "null"] },
  },
  required: ["kind"],
  additionalProperties: false,
} as const;

/** The tools, on one connection. */
export function arenaTools(ctx: ToolContext): Tool[] {
  const root = ctx.url.replace(/\/api\/v1\/?$/, "");
  const client = new ArenaClient({ api: apiBase(root), client: ctx.client ?? CLIENT, token: ctx.token || ctx.store.load().token || null, ...(ctx.fetch ? { fetch: ctx.fetch } : {}) });
  let cache: { at: number; content: MvpContent; rules: MvpRules } | undefined;
  let me: string | undefined;

  /** The pool and rules, kept five minutes (the pool changes by the day). */
  const world = async () => {
    if (!cache || Date.now() - cache.at > 300_000) {
      const [content, home] = await Promise.all([client.content(), client.home()]);
      cache = { at: Date.now(), content, rules: home.rules };
    }
    return cache;
  };
  const loggedIn = () => {
    if (!client.conn.token) throw new Say("Not logged in. Call login {email}, then login_code {code}; or set ARENA_TOKEN.");
  };
  /** The id of a run: given, else the practice run going. */
  const runIdOf = async (a: Record<string, unknown>): Promise<string> => {
    const given = str(a.runId);
    if (given) return given;
    loggedIn();
    const home = await client.home();
    if (!home.practiceRunId) throw new Say("No practice run going: call start_run.");
    return home.practiceRunId;
  };
  const decisionOf = (v: unknown): Decision => {
    let d = v;
    if (typeof d === "string") {
      try {
        d = JSON.parse(d);
      } catch {
        throw new Say(`decision must be a JSON object like {"kind":"reroll"}; got: ${v}`);
      }
    }
    if (!d || typeof d !== "object" || typeof (d as { kind?: unknown }).kind !== "string") throw new Say('decision must be an object with a "kind", like {"kind":"buy","slot":0}.');
    return d as Decision;
  };
  /** After a move: the run, and the fight it started, if any. */
  const afterMove = async (run: RunView, fight: { battleId: string } | undefined, lang: TextOpts, fightLine?: string): Promise<string> => {
    const w = await world();
    const out = [];
    if (fightLine) out.push(fightLine, "");
    out.push(runText(run, w.content, w.rules, lang));
    if (fight) out.push("", `Read the fight: battle {battleId: "${fight.battleId}"}`);
    if (run.phase === "over") out.push("", run.mode === "practice" ? "Practice run over: no rating, no slay. start_run for another." : "Run over.");
    return out.join("\n");
  };

  return [
    {
      name: "login",
      description: "Log in to Arena with your email: mails a 6-digit code. Then call login_code. Skip when ARENA_TOKEN is set or you are already logged in (home tells).",
      inputSchema: {
        type: "object",
        properties: { email: { type: "string" }, name: { type: "string", description: "Player name, for a new player (can wait for login_code)." } },
        required: ["email"],
        additionalProperties: false,
      },
      async run(a) {
        const email = str(a.email);
        if (!email) throw new Say("email is required.");
        await client.emailStart(email, str(a.name) || undefined);
        ctx.store.save({ ...ctx.store.load(), email, ...(str(a.name) ? { name: str(a.name) } : {}) });
        return `A code was mailed to ${email} (it expires in minutes). Ask the player for it and call login_code {code}. A new player also needs a name.`;
      },
    },
    {
      name: "login_code",
      description: "Finish login with the code login mailed. The session is saved (file mode 0600) and kept between runs.",
      inputSchema: {
        type: "object",
        properties: { code: { type: "string" }, name: { type: "string", description: "Player name; only a new player needs one." } },
        required: ["code"],
        additionalProperties: false,
      },
      async run(a) {
        const s = ctx.store.load();
        if (!s.email) throw new Say("Call login {email} first.");
        const name = str(a.name) || s.name;
        const r = await client.emailVerify(s.email, str(a.code), name);
        if (r.status === "needsName") return "This email has no player yet. Call login_code again with the same code and a name.";
        ctx.store.save({ token: r.token });
        me = r.player.id;
        cache = undefined;
        return `Logged in as @${r.player.name}. Next: home, then start_run for a practice run.`;
      },
    },
    {
      name: "home",
      description: "Your rating, today's champion, your runs going (ranked and practice) and ideas held.",
      inputSchema: { type: "object", properties: { lang: LANG }, additionalProperties: false },
      async run(a) {
        const [home, w] = await Promise.all([client.home(), world()]);
        if (home.rating) me = home.rating.player.id;
        return homeText(home, w.content, langOf(a));
      },
    },
    {
      name: "start_run",
      description: "Starts a practice run (same shops, ghosts and Crown as a ranked run; no rating, no slay, no ghost saved), or returns the one already going. Agents never start ranked runs.",
      inputSchema: { type: "object", properties: { lang: LANG }, additionalProperties: false },
      async run(a) {
        loggedIn();
        const run = await client.startRun("practice");
        return afterMove(run, undefined, langOf(a));
      },
    },
    {
      name: "run",
      description: "A run as text: round, hearts, gold, the numbered SHOP, LINE and BENCH, the gift waiting, the next opponent. Reads ranked runs too.",
      inputSchema: { type: "object", properties: { runId: RUN_ID, lang: LANG }, additionalProperties: false },
      async run(a) {
        const run = await client.run(await runIdOf(a));
        return afterMove(run, undefined, langOf(a));
      },
    },
    {
      name: "decide",
      description: "Makes one move in a practice run. A ranked run needs the player's own tap on their phone: this tool says so and moves nothing. Use preview to coach on a ranked run.",
      inputSchema: { type: "object", properties: { runId: RUN_ID, decision: DECISION, lang: LANG }, required: ["decision"], additionalProperties: false },
      async run(a) {
        const d = decisionOf(a.decision);
        const runId = await runIdOf(a);
        const run = await client.run(runId);
        if (run.mode !== "practice")
          return `Run ${runId} is a ranked run: its moves count for rating, the Crown and the throne, so they need the player's own tap on their phone. Nothing was moved. Suggest the move in words, and use preview {runId, decision} to show what it would do.`;
        const r = await client.decide(runId, d);
        return afterMove(r.run, r.fight, langOf(a), r.fight ? fightText(r.fight) : `Done: ${JSON.stringify(d)}`);
      },
    },
    {
      name: "preview",
      description: "What a move would do, on any run, ranked included. Writes nothing (a fight can't be previewed).",
      inputSchema: { type: "object", properties: { runId: RUN_ID, decision: DECISION, lang: LANG }, required: ["decision"], additionalProperties: false },
      async run(a) {
        const d = decisionOf(a.decision);
        const r = await client.preview(await runIdOf(a), d);
        const w = await world();
        return [`PREVIEW of ${JSON.stringify(d)}: nothing was changed.`, "", runText(r.run, w.content, w.rules, langOf(a))].join("\n");
      },
    },
    {
      name: "battle",
      description: "A battle replayed as text. view \"summary\": the teams, the end, the row count; \"steps\" (default): every row turn by turn, each ending #<event id> for trace.",
      inputSchema: {
        type: "object",
        properties: { battleId: { type: "string" }, view: { type: "string", enum: ["summary", "steps"] }, lang: LANG },
        required: ["battleId"],
        additionalProperties: false,
      },
      async run(a) {
        const b = await client.battle(str(a.battleId));
        if (!me && client.conn.token) me = (await client.home()).rating?.player.id;
        const you = me && b.player.id === me ? "A" : me && b.opponent.id === me ? "B" : undefined;
        return battleText(b, { ...langOf(a), view: a.view === "summary" ? "summary" : "steps", ...(you ? { you } : {}) });
      },
    },
    {
      name: "trace",
      description: "Why an event of a battle happened: its cause chain, nearest first. Take the event id from a battle row's #<id>.",
      inputSchema: {
        type: "object",
        properties: { battleId: { type: "string" }, eventId: { type: "integer" }, lang: LANG },
        required: ["battleId", "eventId"],
        additionalProperties: false,
      },
      async run(a) {
        if (typeof a.eventId !== "number" || !Number.isInteger(a.eventId)) throw new Say("eventId is the number after # in a battle row.");
        return traceText(await client.battle(str(a.battleId)), a.eventId, langOf(a));
      },
    },
    {
      name: "codex",
      description: 'Unit cards and the glossary. query: part of a unit\'s name, line or emoji (EN or RU), or a term; "glossary" lists every term; empty lists the pool.',
      inputSchema: { type: "object", properties: { query: { type: "string" }, lang: LANG }, additionalProperties: false },
      async run(a) {
        const w = await world();
        const o = langOf(a);
        const q = str(a.query).toLowerCase();
        const all = w.content.units;
        if (!q) return all.map((u) => `T${u.tier} ${u.emoji} ${unitName(u.id, u.name, w.content, o)} ${u.base.pwr}/${u.base.hp}: ${(o.lang === "ru" && u.texts?.ru?.line) || u.archetype}`).join("\n") + "\n\ncodex {query} for a card; codex {query: \"glossary\"} for the terms.";
        if (q === "glossary" || q === "terms") return glossaryText(w.content, o);
        const hit = [...all, ...(w.content.left ?? [])].filter((u) => [u.name, u.emoji, u.archetype, u.texts?.ru?.name, u.texts?.ru?.line].some((s) => s?.toLowerCase().includes(q)));
        const terms = (Object.keys(GLOSSARY) as FixedTermId[]).map((id) => termDef(id, w.content.statuses, o.lang === "ru" ? "ru" : undefined)).filter((d) => d && d.label.toLowerCase().includes(q));
        const out = [...hit.slice(0, 8).map((u) => cardText(u, w.content, o)), ...terms.map((d) => `${d!.label}: ${d!.tip}`)];
        if (hit.length > 8) out.push(`… ${hit.length - 8} more units: narrow the query.`);
        return out.length ? out.join("\n\n") : `Nothing in the Codex matches "${q}".`;
      },
    },
  ];
}

/** A tool's failure as text the agent can act on. */
export function failureText(err: unknown): string {
  if (err instanceof Say) return err.message;
  if (err instanceof ApiError) {
    if (err.status === 401) return `${err.message} (401: not logged in, or the saved session is gone: login again)`;
    return `Arena answered ${err.status}: ${err.message}`;
  }
  return `Arena couldn't be reached or answered oddly: ${err instanceof Error ? err.message : String(err)}`;
}
