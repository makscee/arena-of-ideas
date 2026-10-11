// arena-mcp (#959): the stdio entry. Bundled into one file (scripts/mcp-bundle.ts),
// which the game server serves at /arena-mcp.mjs:
//
//   curl -fsSL https://arena.makscee.ru/arena-mcp.mjs -o ~/.arena-mcp.mjs
//   claude mcp add arena -- node ~/.arena-mcp.mjs
//
// Env: ARENA_URL (default https://arena.makscee.ru; the server's root or its
// .../api/v1), ARENA_TOKEN (a PlayerSession token: skips login),
// ARENA_SESSION_FILE (default ~/.config/arena-mcp/session.json, mode 0600,
// one entry per server URL).
import { chmodSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { createInterface } from "node:readline";
import { handleMessage } from "./rpc.js";
import { arenaTools, DEFAULT_URL, type Session, type SessionStore } from "./tools.js";

const url = (process.env.ARENA_URL || DEFAULT_URL).replace(/\/$/, "");
const file = process.env.ARENA_SESSION_FILE || join(homedir(), ".config", "arena-mcp", "session.json");

/** The session file: { "<server url>": Session }, so a dev server and the live one don't mix. */
const store: SessionStore = {
  load() {
    try {
      return (JSON.parse(readFileSync(file, "utf8")) as Record<string, Session>)[url] ?? {};
    } catch {
      return {};
    }
  },
  save(s) {
    let all: Record<string, Session> = {};
    try {
      all = JSON.parse(readFileSync(file, "utf8"));
    } catch {}
    all[url] = s;
    mkdirSync(dirname(file), { recursive: true, mode: 0o700 });
    writeFileSync(file, JSON.stringify(all, null, 2), { mode: 0o600 });
    chmodSync(file, 0o600);
  },
};

const tools = arenaTools({ url, token: process.env.ARENA_TOKEN || undefined, store });
const out = (m: unknown) => process.stdout.write(JSON.stringify(m) + "\n");

createInterface({ input: process.stdin }).on("line", async (line) => {
  if (!line.trim()) return;
  let msg: unknown;
  try {
    msg = JSON.parse(line);
  } catch {
    return out({ jsonrpc: "2.0", id: null, error: { code: -32700, message: "Parse error" } });
  }
  for (const m of Array.isArray(msg) ? msg : [msg]) {
    const res = await handleMessage(m, tools);
    if (res) out(res);
  }
});
