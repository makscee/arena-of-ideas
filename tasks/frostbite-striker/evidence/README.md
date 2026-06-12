# Slices 2 & 3 e2e evidence — the two creation adapters

These are the real, unattended creation-worker runs, captured by Mason for PRD
#013. The run logs are the worker's own machine-readable bounce log (the
provenance slice 4 will consume). Nothing here is sanitized — it is the raw
output.

- **Slice 2** (below): the Claude Code headless adapter (`src/create/claude-code.ts`).
- **Slice 3** (at the bottom): the raw chat-completions adapter
  (`src/create/chat-completions.ts`) — a second adapter behind the *same* worker
  bounce loop, proving harness-agnosticism.

## How they were produced

From the repo root of a fresh clone, after `npm ci`:

```
# Convergence (8-attempt budget):
npm run create -- tasks/frostbite-striker --bin <harness> --max-attempts 8

# Loud bounded failure (3-attempt budget, too few to converge):
npm run create -- tasks/frostbite-striker --bin <harness> --max-attempts 3
```

`npm run create` (src/create/cli.ts) drives the worker: it points the harness at
this task's README, collects `out/candidate.json`, runs the README's own
`check-candidate` gauntlet, and on a bounce feeds the gate's JSON numbers back
verbatim as the next prompt — bounded retries, then loud failure. The worker
holds no game rules; the gauntlet carries them.

## run-log-real-claude-401.jsonl — the live `claude` CLI, end-to-end

`npm run create -- tasks/frostbite-striker --max-attempts 1` against the **real**
`claude` binary (v2.1.175, default `--bin`). The adapter spawned
`claude -p --output-format json --permission-mode bypassPermissions --add-dir
<repo>`, fed the prompt on stdin, and parsed the JSON-array result envelope — the
CLI returned a `session_id` and a clean envelope, so the adapter wiring is
correct end to end. The run then failed **loudly** with the harness's own error:

```
"harness": { "ok": false, "handle": "api-error",
             "detail": "Failed to authenticate. API Error: 401 status code (no body)" }
```

This is an **environment auth blocker, not a code defect**: the headless `-p`
spawn cannot reach the Claude subscription credentials here (`apiKeySource:
"none"`; the platform's interactive OAuth/gateway auth is not available to a
spawned non-interactive process). On a machine where `claude -p` is
authenticated (a real `ANTHROPIC_API_KEY`, an `apiKeyHelper`, or working
subscription headless auth) this same command drives a real model. The worker
handled the failure exactly as designed: logged it as `api-error` and exited
non-zero.

## run-log-converge.jsonl — the full bounce loop converging

Because the live CLI is unauthenticated here, the loop was driven end-to-end by
`fake-claude-harness.mjs` — a faithful stand-in that obeys the same headless
contract (reads the prompt on stdin, writes `out/candidate.json`, emits the
JSON-array envelope) and acts as a real agent would: it **starts intentionally
overtuned and earns convergence from the gate's bounce numbers**, never knowing
the answer up front. It does not hardcode a passing candidate; it lowers
magnitudes whenever the gate says "overtuned".

The real `check-candidate` gauntlet ran as a subprocess each attempt. The
trajectory:

| attempt | verdict     | overall | Aggro / Sustain / StatStack |
|--------:|-------------|--------:|-----------------------------|
| 1       | overtuned   | 79.3%   | 100% / 64% / 74%            |
| 2       | overtuned   | 79.3%   | 100% / 64% / 74%            |
| 3       | overtuned   | 70.7%   | 100% / 64% / 48%            |
| 4       | overtuned   | 73.3%   | 100% / 64% / 56%            |
| 5       | overtuned   | 68.7%   | 100% / 64% / 42%            |
| 6       | **in-band** | 64.7%   | 100% / 64% / 30%            |

Converged at attempt 6/8, exit 0. Every matchup clears the 25% per-matchup
floor — the candidate is broadly viable, not a one-matchup hard-counter.

## run-log-loud-failure.jsonl — bounded loud failure

The same harness with `--max-attempts 3` — fewer than the 6 it takes to descend
into the band — fails loudly: three "overtuned" bounces, `converged: false`,
exit 1. The bound is honoured; nothing is smuggled through.

## fake-claude-harness.mjs

The stand-in, committed so the e2e is reproducible without live auth. It is test
scaffolding, not part of the shipped adapter — the real adapter is
`src/create/claude-code.ts`, exercised by the 401 run above and the unit tests.

---

# Slice 3 — raw chat-completions adapter

The second adapter, `src/create/chat-completions.ts`, drives ANY
OpenAI-compatible `/v1/chat/completions` endpoint (DeepSeek, OpenAI, a local
server) over a bare `fetch` HTTP loop. It sits behind the **same** worker
(`runLoop`) the Claude Code adapter sits behind — `npm run create` just builds a
different `Harness`. Because a bare completion endpoint has no file tools, the
adapter exposes two **generic, game-blind** tools — `read_file` / `write_file`,
jailed to the repo — so the model can read the README it is pointed at and write
the candidate, exactly as an agentic CLI does on its own. The contract lives in
the README the model reads; the adapter knows only how to move bytes and speak
the chat-completions wire format.

## The blocked real run (the one command Maks runs when a key exists)

No real model/key is reachable from this machine (no DeepSeek/OpenAI key;
`ANTHROPIC_*` exported but empty). The real-endpoint run is therefore **blocked,
not skipped**. On a machine with a key, this single command drives a live model
end to end — the adapter is complete and ready for it:

```
export DEEPSEEK_API_KEY=sk-...   # or pass --key
npm run create -- tasks/frostbite-striker --adapter=chat \
    --base-url=https://api.deepseek.com --model=deepseek-chat
```

(`--base-url`/`--key` also read `OPENAI_BASE_URL` / `OPENAI_API_KEY` /
`DEEPSEEK_API_KEY` from the env.)

## stub-openai-server.mjs — the protocol-faithful stand-in

To prove the adapter without a key, `stub-openai-server.mjs` is a local node
`http` server that speaks the OpenAI wire format faithfully (the response shape
incl. `choices[].message`/`tool_calls`, `finish_reason`, `usage`; `stream:false`
only — the adapter posts `stream:false`). It plays the **same role** fake-claude
plays for slice 2: faithful at the protocol layer, blind to the game. As a
"model" it reads the README, writes a candidate, and **starts intentionally
overtuned, earning convergence from the gate's bounce numbers** — it never knows
the answer up front. It is test scaffolding, not part of the shipped adapter.

## run-log-chat-converge.jsonl — the full bounce loop converging over HTTP

`npm run create -- tasks/frostbite-striker --adapter=chat
--base-url=http://127.0.0.1:<port> --model=stub-openai --max-attempts 8` against
the stub, with the **real `check-candidate` gauntlet** run as a subprocess each
attempt. Each attempt the adapter ran a multi-turn conversation over real HTTP:
`read_file(README)` → `write_file(out/candidate.json)` → stop. Trajectory:

| attempt | verdict     | overall | AggroVenom / SustainControl / StatStack |
|--------:|-------------|--------:|-----------------------------------------|
| 1       | overtuned   | 79.3%   | 100% / 64% / 74%                        |
| 2       | overtuned   | 79.3%   | 100% / 64% / 74%                        |
| 3       | overtuned   | 70.7%   | 100% / 64% / 48%                        |
| 4       | overtuned   | 73.3%   | 100% / 64% / 56%                        |
| 5       | overtuned   | 68.7%   | 100% / 64% / 42%                        |
| 6       | **in-band** | 64.7%   | 100% / 64% / 30%                        |

Converged at attempt 6/8, exit 0 — the **same trajectory** as the slice-2
Claude Code run (same task, same gauntlet, same band/floor), reached over a
different transport. Every matchup clears the 25% per-matchup floor.

## run-log-chat-loud-failure.jsonl — bounded loud failure

The same run with `--max-attempts 3` (fewer than the 6 it takes to descend into
the band): three "overtuned" bounces, `converged: false`, exit 1. The bound is
honoured.

## run-log-chat-transport-failure.jsonl — loud transport failure

`--adapter=chat --base-url=http://127.0.0.1:1` (a dead port) with
`--max-attempts 1`: the adapter's HTTP request fails and the attempt is logged
`"harness": { "ok": false, "handle": "network-error", … }`, outcome
`harness-error`, exit 1 — the same loud-degradation path the Claude Code
adapter's 401 took. (The unit tests in `src/create/chat-completions.test.ts`
also cover `http-<code>`, `malformed-body`, and the turn/token budgets with an
injected fetch.)
