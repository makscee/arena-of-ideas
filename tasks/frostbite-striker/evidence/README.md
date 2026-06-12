# Slice 2 e2e evidence — Claude Code headless creation adapter

These are the real, unattended creation-worker runs on a **clean checkout**
(`git clone` to a temp dir + `npm ci`), captured by Mason for PRD #013 slice 2.
The run logs are the worker's own machine-readable bounce log (the provenance
slice 4 will consume). Nothing here is sanitized — it is the raw output.

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
