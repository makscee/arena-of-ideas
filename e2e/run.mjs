// e2e orchestrator (`npm run e2e`): boots the arena server (MOCK_MODE, temp
// SQLite — #016 slice 3) and vite on their own ports, waits for both ready
// lines (bounded), runs every probe as a child with a hard timeout, and always
// tears both down. Vite proxies /v1 and /_mock to the server (vite.config.ts
// reads AOI_SERVER_URL), so probes talk to ONE origin exactly like a player.
// The probes need real layout (wrap, collapse, elementFromPoint) that jsdom
// cannot see — this keeps the PRD #012 layout checks repeatable. Excluded
// from vitest: probes are plain scripts, not *.test.* files.

import { spawn, spawnSync } from "node:child_process";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const PORT = process.env.AOI_E2E_PORT ?? "5280";
const SERVER_PORT = process.env.AOI_E2E_SERVER_PORT ?? "5285";
const BASE = `http://localhost:${PORT}`;

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
if (process.env.AOI_E2E_FAULT === "throw") throw new Error("injected fault (throw)");
if (process.env.AOI_E2E_FAULT === "reject") Promise.reject(new Error("injected fault (reject)"));

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
    const probes = readdirSync(here)
      .filter((f) => f.startsWith("probe-") && f.endsWith(".mjs"))
      .sort();
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
