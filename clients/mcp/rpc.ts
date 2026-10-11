// The MCP side of arena-mcp (#959): JSON-RPC 2.0 over stdio lines, just the
// parts of the Model Context Protocol a tool server needs (initialize, ping,
// tools/list, tools/call). By hand rather than the SDK: one bundled file with
// no dependencies is the install ("curl it, node it").
import { failureText, type Tool } from "./tools.js";

export const SERVER_INFO = { name: "arena-mcp", version: "0.1.0" } as const;
/** Newest first; an initialize that asks for one of these gets it back. */
const PROTOCOLS = ["2025-06-18", "2025-03-26", "2024-11-05"];

export const INSTRUCTIONS = [
  "Arena of Ideas: an async auto-battler. You coach, or play practice runs for the player; ranked runs need the player's own tap.",
  "Start: home (login first if it says so), then start_run. run shows the numbered SHOP, LINE and BENCH; decide takes one move as a JSON object using those numbers.",
  "After a fight, battle {battleId} replays it as text and trace {battleId, eventId} explains a row's #id. codex looks up unit cards and terms. Pass lang \"ru\" for Russian.",
].join("\n");

type Id = string | number | null;
interface Req {
  jsonrpc?: string;
  id?: Id;
  method?: string;
  params?: Record<string, unknown>;
}
type Res = { jsonrpc: "2.0"; id: Id; result: unknown } | { jsonrpc: "2.0"; id: Id; error: { code: number; message: string } };

const ok = (id: Id, result: unknown): Res => ({ jsonrpc: "2.0", id, result });
const err = (id: Id, code: number, message: string): Res => ({ jsonrpc: "2.0", id, error: { code, message } });

/** Answers one JSON-RPC message; a notification (no id) gets undefined. */
export async function handleMessage(msg: unknown, tools: Tool[]): Promise<Res | undefined> {
  if (!msg || typeof msg !== "object" || Array.isArray(msg)) return err(null, -32600, "Invalid Request");
  const req = msg as Req;
  if (req.id === undefined || req.method === undefined) return req.method === undefined && req.id !== undefined ? err(req.id, -32600, "Invalid Request") : undefined;
  const id = req.id;
  switch (req.method) {
    case "initialize": {
      const asked = typeof req.params?.protocolVersion === "string" ? req.params.protocolVersion : "";
      return ok(id, { protocolVersion: PROTOCOLS.includes(asked) ? asked : PROTOCOLS[0], capabilities: { tools: {} }, serverInfo: SERVER_INFO, instructions: INSTRUCTIONS });
    }
    case "ping":
      return ok(id, {});
    case "tools/list":
      return ok(id, { tools: tools.map(({ name, description, inputSchema }) => ({ name, description, inputSchema })) });
    case "tools/call": {
      const tool = tools.find((t) => t.name === req.params?.name);
      if (!tool) return err(id, -32602, `Unknown tool: ${String(req.params?.name)}`);
      const args = req.params?.arguments;
      try {
        const text = await tool.run(args && typeof args === "object" ? (args as Record<string, unknown>) : {});
        return ok(id, { content: [{ type: "text", text }] });
      } catch (e) {
        return ok(id, { content: [{ type: "text", text: failureText(e) }], isError: true });
      }
    }
    default:
      return err(id, -32601, `Method not found: ${req.method}`);
  }
}
