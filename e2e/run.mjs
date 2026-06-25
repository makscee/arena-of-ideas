// e2e orchestrator (`npm run e2e`): boots the arena server (MOCK_MODE, temp
// SQLite — #016 slice 3) and vite on their own ports, waits for both ready
// lines (bounded), runs every probe as a child with a hard timeout, and always
// tears both down. Vite proxies /v1 and /_mock to the server (vite.config.ts
// reads AOI_SERVER_URL), so probes talk to ONE origin exactly like a player.
// The probes need real layout (wrap, collapse, elementFromPoint) that jsdom
// cannot see — this keeps the PRD #012 layout checks repeatable. Excluded
// from vitest: probes are plain scripts, not *.test.* files.

import { spawn, spawnSync } from "node:child_process";
import { mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const PORT = process.env.AOI_E2E_PORT ?? "5280";
const SERVER_PORT = process.env.AOI_E2E_SERVER_PORT ?? "5285";
const BASE = `http://localhost:${PORT}`;
const PIDFILE = join(root, ".e2e-serve.json");

// CLI shape: `--`-prefixed args are FLAGS, bare words are probe-filter TOKENS.
// `--serve` holds the warm stack open; `--down` reaps a held stack; tokens
// (when present, no serve/down flag) scope the suite to matching probes. The
// three are mutually exclusive in effect — serve/down ignore any tokens.
const args = process.argv.slice(2);
const flags = new Set(args.filter((a) => a.startsWith("--")));
const cliTokens = args.filter((a) => !a.startsWith("--"));
const envTokens = (process.env.PROBE ?? "").split(/[\s,]+/).filter(Boolean);
const tokens = [...cliTokens, ...envTokens];

/** Normalize a token to a substring match against a probe filename: strip a
 * leading `probe-` and trailing `.mjs` the user may have typed, lower-case it,
 * then test it as a substring of the (lower-cased) filename. */
function probeMatches(file, token) {
  const t = token.toLowerCase().replace(/^probe-/, "").replace(/\.mjs$/, "");
  return file.toLowerCase().includes(t);
}

/** All probe filenames, sorted — the suite's stable order. */
function allProbes() {
  return readdirSync(here)
    .filter((f) => f.startsWith("probe-") && f.endsWith(".mjs"))
    .sort();
}

// `--down`: reap a stack a previous `--serve` left warm, found via the pidfile
// (no terminal needed). We SIGTERM the serve process by its POSITIVE pid so its
// own handler runs teardown() — the same per-child group-kill that the normal
// run uses, never duplicated here. If it doesn't die within the grace, escalate
// to SIGKILL on the serve group AND each recorded child group (SIGKILL can't be
// caught, so the handler won't run — we reap the children directly). No pidfile
// → nothing to reap → clean exit 0.
if (flags.has("--down")) {
  let info;
  try {
    info = JSON.parse(readFileSync(PIDFILE, "utf8"));
  } catch {
    console.log("e2e: no warm server to stop (no pidfile)");
    process.exit(0);
  }
  const { pid, children = [] } = info;
  const alive = (p) => {
    try {
      process.kill(p, 0);
      return true;
    } catch {
      return false;
    }
  };
  try {
    process.kill(pid, "SIGTERM");
  } catch {}
  const deadline = Date.now() + 800;
  while (Date.now() < deadline && alive(pid)) spawnSync("sleep", ["0.05"]);
  if (alive(pid)) {
    for (const c of children) {
      try {
        process.kill(-c, "SIGKILL");
      } catch {}
    }
    try {
      process.kill(-pid, "SIGKILL");
    } catch {}
  }
  try {
    rmSync(PIDFILE, { force: true });
  } catch {}
  console.log(`e2e: warm server (pid ${pid}) stopped`);
  process.exit(0);
}

// Children we boot (arena, vite) outlive a clean run only if we let them: each
// spawns its own grandchildren (esbuild/tsx workers, and a probe under us may
// hold a chromium). `detached: true` makes each child a process-group LEADER,
// so signalling the NEGATIVE pid (`process.kill(-pid, sig)`) reaps the whole
// group — child AND grandchildren — in one shot. Without this, an interrupted
// run reparents its children to init (ppid=1) and they pin CPU.
const groups = [];

/** Spawn a child as its own group leader; collect output; track for teardown. */
function boot(name, cmd, args, env) {
  const child = spawn(cmd, args, {
    cwd: root,
    env: { ...process.env, ...env },
    detached: true,
  });
  child.out = "";
  child.stdout.on("data", (d) => (child.out += d));
  child.stderr.on("data", (d) => (child.out += d));
  child.tag = name;
  groups.push(child);
  return child;
}

// One idempotent teardown, invoked from EVERY exit path. Signals the whole
// group (negative pid), escalating SIGTERM → SIGKILL after a short grace so a
// wedged child still dies. Guarded so a second call (e.g. a signal mid-cleanup)
// is a no-op.
let cleanedUp = false;
let wroteServeFile = false;
function teardown() {
  if (cleanedUp) return;
  cleanedUp = true;
  const live = groups.filter((c) => c.pid && c.exitCode === null && !c.killed);
  for (const c of live) {
    try {
      process.kill(-c.pid, "SIGTERM");
    } catch {
      try {
        c.kill("SIGTERM");
      } catch {}
    }
  }
  // Block ~600ms for graceful exit, then SIGKILL the survivors' groups. A
  // synchronous wait keeps teardown usable from a signal handler without an
  // async race (the process is on its way out anyway).
  const deadline = Date.now() + 600;
  while (Date.now() < deadline && live.some((c) => c.exitCode === null && !c.killed)) {
    spawnSync("sleep", ["0.05"]);
  }
  for (const c of live) {
    if (c.exitCode !== null) continue;
    try {
      process.kill(-c.pid, "SIGKILL");
    } catch {
      try {
        c.kill("SIGKILL");
      } catch {}
    }
  }
  try {
    rmSync(dbDir, { recursive: true, force: true });
  } catch {}
  // In --serve mode this process owns the pidfile; clear it on the way out
  // (clean exit or signal) so --down never chases a dead pid.
  if (wroteServeFile) {
    try {
      rmSync(PIDFILE, { force: true });
    } catch {}
  }
}

// Wire teardown to the abnormal exit paths. SIGINT/SIGTERM re-exit with the
// conventional 128+signal code; crashes exit non-zero. The happy path calls
// teardown() directly in `finally` below.
for (const [sig, n] of [
  ["SIGINT", 2],
  ["SIGTERM", 15],
]) {
  process.on(sig, () => {
    teardown();
    process.exit(128 + n);
  });
}
process.on("uncaughtException", (err) => {
  console.error("e2e orchestrator uncaughtException:", err);
  teardown();
  process.exit(1);
});
process.on("unhandledRejection", (err) => {
  console.error("e2e orchestrator unhandledRejection:", err);
  teardown();
  process.exit(1);
});

function awaitReady(child, marker, timeoutMs = 30_000) {
  return new Promise((resolve) => {
    const t0 = Date.now();
    const poll = setInterval(() => {
      if (child.out.includes(marker)) {
        clearInterval(poll);
        resolve(true);
      } else if (Date.now() - t0 > timeoutMs || child.exitCode !== null) {
        clearInterval(poll);
        resolve(false);
      }
    }, 200);
  });
}

// The arena server: mock mailer (codes readable via /_mock/last-code), a
// throwaway DB per e2e run — every run starts from the bootstrap ladder.
const dbDir = mkdtempSync(join(tmpdir(), "aoi-e2e-"));
const arena = boot("arena-server", "npx", ["tsx", "server/src/main.ts"], {
  MOCK_MODE: "1",
  PORT: SERVER_PORT,
  DB_PATH: join(dbDir, "arena.db"),
});
const vite = boot("vite", "npx", ["vite", "--port", PORT, "--strictPort"], {
  AOI_SERVER_URL: `http://localhost:${SERVER_PORT}`,
});

const arenaReady = await awaitReady(arena, "arena server listening");
const viteReady = await awaitReady(vite, "Local:");

// Fault injection for the leak-check (no effect unless AOI_E2E_FAULT is set):
// `throw` exercises the uncaughtException path; `reject` the unhandledRejection
// path. Children are up here, so teardown must still reap them on a crash.
//
// The `reject` case must yield the event loop so the unhandledRejection handler
// actually fires before the synchronous probe loop begins. We schedule the
// rejection via setImmediate (guaranteeing it lands on the next event-loop turn)
// then await a never-resolving Promise — the event loop runs, the handler fires,
// teardown + process.exit(1) execute, and the probe loop is never reached.
if (process.env.AOI_E2E_FAULT === "throw") throw new Error("injected fault (throw)");
if (process.env.AOI_E2E_FAULT === "reject") {
  setImmediate(() => Promise.reject(new Error("injected fault (reject)")));
  await new Promise(() => {}); // suspend so the event loop turns and the handler fires
}

// --serve: the stack is up; hold it open instead of running probes, so an agent
// can fire scoped probes against the warm origin repeatedly. We write a pidfile
// (our own pid + ports) so `--down` can reap us without the terminal, then idle
// forever — only a signal (handlers above) or `--down`'s group-kill ends us, and
// teardown reaps the children + clears the pidfile on the way out.
if (flags.has("--serve")) {
  if (!arenaReady || !viteReady) {
    if (!arenaReady) console.error(`arena server did not become ready on :${SERVER_PORT} within 30s\n${arena.out}`);
    if (!viteReady) console.error(`vite did not become ready on :${PORT} within 30s\n${vite.out}`);
    teardown();
    process.exit(1);
  }
  // Record our pid AND the children's group-leader pids. Our children are each
  // their OWN process group (boot() spawns them detached), so reaping the warm
  // stack means signalling our pid (whose handler runs teardown → per-child
  // group-kill) and, as a SIGKILL fallback, each child's group directly.
  try {
    writeFileSync(
      PIDFILE,
      JSON.stringify({
        pid: process.pid,
        children: groups.map((c) => c.pid).filter(Boolean),
        port: Number(PORT),
        serverPort: Number(SERVER_PORT),
      }),
    );
    wroteServeFile = true;
  } catch {}
  console.log(`e2e server warm at ${BASE}`);
  console.log(`  vite :${PORT}  arena :${SERVER_PORT}  (pid ${process.pid})`);
  console.log(`  run a scoped probe: AOI_BASE_URL=${BASE} npm run probe <name>`);
  console.log(`  stop: npm run e2e:stop`);
  await new Promise(() => {}); // idle until a signal / --down reaps us
}

let failed = false;
try {
  if (!arenaReady) {
    failed = true;
    console.error(`arena server did not become ready on :${SERVER_PORT} within 30s\n${arena.out}`);
  }
  if (!viteReady) {
    failed = true;
    console.error(`vite did not become ready on :${PORT} within 30s\n${vite.out}`);
  }
  if (!failed) {
    // No tokens → full suite (sorted). Tokens → only probes matching ANY token
    // (generous substring). Tokens that match nothing is an error, not a vacuous
    // pass — list what's available and fail.
    let probes = allProbes();
    if (tokens.length > 0) {
      probes = probes.filter((f) => tokens.some((t) => probeMatches(f, t)));
      if (probes.length === 0) {
        console.error(`no probe matches ${JSON.stringify(tokens)}. available:`);
        for (const p of allProbes()) console.error(`  ${p}`);
        failed = true;
      }
    }
    for (const probe of probes) {
      console.log(`\n=== ${probe} ===`);
      const r = spawnSync("node", ["--import", "tsx/esm", join(here, probe)], {
        cwd: root,
        stdio: "inherit",
        timeout: 150_000,
        env: { ...process.env, AOI_BASE_URL: BASE },
      });
      // A timed-out probe (r.signal set) is a failure too — and its own group
      // is killed by spawnSync, but the orchestrated server/vite stay up for
      // the rest of the run; teardown in `finally` reaps them at the end.
      if (r.status !== 0 || r.signal) failed = true;
    }
  }
} finally {
  teardown();
}
process.exit(failed ? 1 : 0);
