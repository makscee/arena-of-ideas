// What the Claude Code mod runs (#960), bundled into hooks/arena.mjs by
// scripts/mod-bundle.ts: an installed mod has only its own folder. Claude's
// tools are arena-mcp's (clients/mcp/tools.ts), not a copy.
export { arenaTools, failureText, DEFAULT_URL, type Session, type SessionStore, type Tool } from "../../mcp/tools.js";
export { ApiError, ArenaClient, apiBase, battleText } from "../../core/index.js";
export * from "./pane.js";
