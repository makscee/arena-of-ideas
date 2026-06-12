#!/usr/bin/env node
// A faithful stand-in for the Claude Code CLI, used to drive the creation
// worker end-to-end on a clean checkout when the live `claude -p` is
// unauthenticated (subscription OAuth is unavailable to a spawned headless
// process in this environment — see the worker's real-run evidence).
//
// It mimics the CLI's headless contract the adapter depends on:
//   - prompt is the final positional arg; flags (-p, --output-format json,
//     --permission-mode, --add-dir, --model) precede it,
//   - it emits a JSON *array* envelope on stdout ending in a type:"result"
//     object (session_id, is_error, num_turns), exits 0.
//
// As an "agent" it does the task honestly: it reads the prompt to find the
// task dir + out path, writes a candidate, and ON A BOUNCE it parses the
// gate numbers fed back in the prompt and ADJUSTS magnitudes toward the band —
// it does NOT know the answer up front. So convergence is earned via the
// bounce loop, exactly as a real agent would: it starts intentionally
// overtuned, and the gate's "overtuned" verdict drives it down into the band.
//
// State across attempts lives in <taskDir>/out/.fake-state.json (the worker
// re-spawns a fresh process each attempt, like the real CLI).

import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";

// The worker feeds the prompt on stdin (like the real adapter); fall back to the
// last argv element if stdin is empty.
let stdinText = "";
try { stdinText = readFileSync(0, "utf8"); } catch {}
const prompt = stdinText.trim() ? stdinText : process.argv[process.argv.length - 1];

// Find the task dir + out path from the prompt (the worker names absolute paths
// to the README and the out target — we parse them, we don't hardcode them).
const readmeMatch = prompt.match(/(\S+README\.md)/);
const outMatch = prompt.match(/(\S+out\/candidate\.json)/);
if (!outMatch) {
  emit({ is_error: true, result: "fake-harness: could not find out path in prompt" });
  process.exit(0);
}
const outPath = outMatch[1];
const taskDir = dirname(dirname(outPath)); // .../tasks/<id>/out/candidate.json -> .../tasks/<id>
const statePath = join(taskDir, "out", ".fake-state.json");

// A frost striker (expresses the README idea): a Shielded front body that
// poisons + curses what it strikes, a durable mid, a body. The magnitudes are
// what we tune on a bounce.
function buildCandidate(shield, poison) {
  const strike = (status, value) => ({
    whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
    selectors: [{ kind: "frontEnemy" }],
    effects: [{ kind: "applyStatus", status, stacks: { kind: "const", value } }],
  });
  return {
    _comment: `fake-harness frost striker (shield=${shield}, poison=${poison})`,
    units: [
      { name: "Frostbiter", base: { hp: 11, pwr: 2 }, statuses: [{ status: "Shield", stacks: shield }], abilities: [strike("Poison", poison), strike("Curse", 1)] },
      { name: "Glacier", base: { hp: 12, pwr: 2 } },
      { name: "Squire", base: { hp: 8, pwr: 2 } },
    ],
  };
}

// Read prior state (magnitudes) or start INTENTIONALLY too strong so the gate
// must bounce us at least once — convergence is earned, not pre-baked.
let state = { shield: 6, poison: 6 };
if (existsSync(statePath)) {
  try { state = JSON.parse(readFileSync(statePath, "utf8")); } catch {}
}

const isBounce = /did not pass|check output|bounced|verdict/i.test(prompt);
if (isBounce) {
  // Parse the gate JSON the worker fed back (verbatim) and adjust.
  const jsonLine = (prompt.match(/\{[\s\S]*"verdict"[\s\S]*\}/) || [])[0];
  let verdict = "overtuned";
  try { verdict = JSON.parse(jsonLine).gate.verdict; } catch {}
  if (verdict === "overtuned") {
    // Too strong: lower magnitudes a notch.
    if (state.poison > 3) state.poison -= 1;
    else if (state.shield > 3) state.shield -= 1;
    else state.poison = Math.max(1, state.poison - 1);
  } else if (verdict === "underpowered") {
    state.poison += 1;
  } else if (verdict === "counter-folded") {
    // Fold somewhere: nudge poison (the StatStack lever) without inflating.
    state.poison += 1;
  }
}

mkdirSync(dirname(outPath), { recursive: true });
writeFileSync(outPath, JSON.stringify(buildCandidate(state.shield, state.poison), null, 2) + "\n", "utf8");
writeFileSync(statePath, JSON.stringify(state), "utf8");

emit({ is_error: false, result: `wrote candidate (shield=${state.shield}, poison=${state.poison})`, num_turns: 1 });

function emit(resultObj) {
  const sid = "fake-" + Math.random().toString(36).slice(2, 10);
  const arr = [
    { type: "system", subtype: "init", session_id: sid, cwd: process.cwd() },
    { type: "result", subtype: resultObj.is_error ? "error" : "success", session_id: sid, ...resultObj },
  ];
  process.stdout.write(JSON.stringify(arr));
}
