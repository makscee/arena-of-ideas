#!/usr/bin/env node
// A protocol-faithful stand-in for an OpenAI-compatible /v1/chat/completions
// endpoint (DeepSeek, OpenAI, llama.cpp, …), used to drive the creation worker's
// chat-completions adapter (src/create/chat-completions.ts) end-to-end on this
// machine, where no real model/key is reachable. It plays the SAME role-model
// fake-claude-harness.mjs plays for the Claude Code adapter: faithful at the
// PROTOCOL layer, blind to the game.
//
// Faithful to the wire shape the adapter depends on:
//   - POST /v1/chat/completions with { model, messages, tools, stream:false },
//   - a 200 JSON body shaped { choices:[{ message, finish_reason }], usage },
//   - tool calls returned as choices[0].message.tool_calls (OpenAI function
//     calling), finish_reason "tool_calls" then "stop",
//   - choices[0].message.content for the final assistant text.
// It is stream=false only (documented stance — the adapter posts stream:false).
//
// As a "model" it does the task honestly through the adapter's two generic tools
// (read_file / write_file): it reads the README first, then writes a candidate,
// then stops. It does NOT know the answer up front — it starts intentionally
// overtuned and, ON A BOUNCE, parses the gate verdict the worker fed back into
// the user turn and adjusts magnitudes toward the band. Convergence is earned
// through the worker's bounce loop, exactly as a real model would earn it.
//
// It holds NO game rules: it knows "overtuned -> lower a number, underpowered ->
// raise it", which is generic optimisation feedback, not balance knowledge — the
// same stance fake-claude takes. The candidate's DSL shape it emits is copied
// structurally; the magnitudes are what the gate tunes. Cross-attempt magnitude
// state lives in <taskDir>/out/.stub-state.json (each worker attempt is a fresh
// HTTP conversation, like a real stateless endpoint).
//
// Usage:
//   node stub-openai-server.mjs            # ephemeral port, prints "PORT <n>"
//   STUB_PORT=8123 node stub-openai-server.mjs
//
// It exits 0 on SIGTERM/SIGINT. The adapter points at http://127.0.0.1:<port>.

import { createServer } from "node:http";
import { readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";

// --- the "model": pick magnitudes, build a candidate -----------------------

function buildCandidate(shield, poison) {
  const strike = (status, value) => ({
    whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
    selectors: [{ kind: "frontEnemy" }],
    effects: [{ kind: "applyStatus", status, stacks: { kind: "const", value } }],
  });
  return {
    _comment: `stub-openai frost striker (shield=${shield}, poison=${poison})`,
    units: [
      { name: "Frostbiter", base: { hp: 11, pwr: 2 }, statuses: [{ status: "Shield", stacks: shield }], abilities: [strike("Poison", poison), strike("Curse", 1)] },
      { name: "Glacier", base: { hp: 12, pwr: 2 } },
      { name: "Squire", base: { hp: 8, pwr: 2 } },
    ],
  };
}

// Read the bounce verdict (if any) out of the conversation's first user turn and
// adjust magnitudes — generic optimisation feedback, no balance knowledge.
function decideMagnitudes(statePath, userTurn) {
  let state = { shield: 6, poison: 6 }; // start intentionally overtuned
  if (existsSync(statePath)) {
    try { state = JSON.parse(readFileSync(statePath, "utf8")); } catch {}
  }
  const isBounce = /did not pass|check output|bounced|verdict/i.test(userTurn);
  if (isBounce) {
    const jsonLine = (userTurn.match(/\{[\s\S]*"verdict"[\s\S]*\}/) || [])[0];
    let verdict = "overtuned";
    try { verdict = JSON.parse(jsonLine).gate.verdict; } catch {}
    if (verdict === "overtuned") {
      if (state.poison > 3) state.poison -= 1;
      else if (state.shield > 3) state.shield -= 1;
      else state.poison = Math.max(1, state.poison - 1);
    } else if (verdict === "underpowered") {
      state.poison += 1;
    } else if (verdict === "counter-folded") {
      state.poison += 1;
    }
  }
  return state;
}

// --- the wire layer: respond to one chat-completions POST ------------------

function handleCompletion(body) {
  const messages = body.messages || [];
  const userTurn = messages.find((m) => m.role === "user")?.content ?? "";

  // Find the out path the task names (the README is read into a tool result; the
  // adapter relays it). We never hardcode paths — we parse them from the prompt
  // (the worker's user turn names the absolute out target).
  const outMatch = userTurn.match(/(\S+out\/candidate\.json)/);

  // Has a write_file already happened in THIS conversation? If a prior assistant
  // turn called write_file (and the tool returned OK), we are done — stop.
  const wroteAlready = messages.some(
    (m) => m.role === "tool" && typeof m.content === "string" && m.content.startsWith("OK: wrote"),
  );
  const readAlready = messages.some(
    (m) => m.role === "tool" && typeof m.content === "string" && m.content.includes("Self-test"),
  );

  if (wroteAlready) {
    return completion({ content: "Wrote the candidate and it should satisfy the task's self-test.", finish_reason: "stop" }, 24);
  }

  // First turn: read the README (faithful agent behaviour — gather the contract).
  if (!readAlready) {
    const readmeMatch = userTurn.match(/(\S+README\.md)/);
    const path = readmeMatch ? readmeMatch[1] : "tasks/frostbite-striker/README.md";
    return completion({
      tool_calls: [toolCall("call_read", "read_file", { path })],
      finish_reason: "tool_calls",
    }, 12);
  }

  // We have read the README; now write the candidate (magnitudes from any bounce).
  if (!outMatch) {
    return completion({ content: "ERROR: could not find the output path in the task.", finish_reason: "stop" }, 8);
  }
  const outPath = outMatch[1];
  const taskDir = dirname(dirname(outPath));
  const statePath = join(taskDir, "out", ".stub-state.json");
  const state = decideMagnitudes(statePath, userTurn);
  mkdirSync(dirname(statePath), { recursive: true });
  writeFileSync(statePath, JSON.stringify(state), "utf8");
  const candidate = buildCandidate(state.shield, state.poison);
  return completion({
    tool_calls: [toolCall("call_write", "write_file", { path: outPath, content: JSON.stringify(candidate, null, 2) + "\n" })],
    finish_reason: "tool_calls",
  }, 40);
}

let toolCounter = 0;
function toolCall(idBase, name, args) {
  return { id: `${idBase}_${++toolCounter}`, type: "function", function: { name, arguments: JSON.stringify(args) } };
}

function completion(message, completionTokens) {
  return {
    id: "chatcmpl-stub-" + Math.random().toString(36).slice(2, 10),
    object: "chat.completion",
    model: "stub-openai",
    choices: [
      {
        index: 0,
        message: { role: "assistant", content: message.content ?? null, ...(message.tool_calls ? { tool_calls: message.tool_calls } : {}) },
        finish_reason: message.finish_reason,
      },
    ],
    usage: { prompt_tokens: 100, completion_tokens: completionTokens, total_tokens: 100 + completionTokens },
  };
}

// --- the server ------------------------------------------------------------

const server = createServer((req, res) => {
  if (req.method !== "POST" || !req.url.endsWith("/chat/completions")) {
    res.writeHead(404, { "content-type": "application/json" });
    res.end(JSON.stringify({ error: { message: "not found" } }));
    return;
  }
  let raw = "";
  req.on("data", (c) => (raw += c));
  req.on("end", () => {
    let body;
    try { body = JSON.parse(raw); } catch {
      res.writeHead(400, { "content-type": "application/json" });
      res.end(JSON.stringify({ error: { message: "bad json" } }));
      return;
    }
    const out = handleCompletion(body);
    res.writeHead(200, { "content-type": "application/json" });
    res.end(JSON.stringify(out));
  });
});

const port = Number(process.env.STUB_PORT) || 0;
server.listen(port, "127.0.0.1", () => {
  const actual = server.address().port;
  process.stdout.write(`PORT ${actual}\n`);
});

for (const sig of ["SIGTERM", "SIGINT"]) {
  process.on(sig, () => server.close(() => process.exit(0)));
}
