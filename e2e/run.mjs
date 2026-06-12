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

/** Spawn a child and collect its output; `ready` answers "is it up yet?". */
function boot(name, cmd, args, env) {
  const child = spawn(cmd, args, { cwd: root, env: { ...process.env, ...env } });
  child.out = "";
  child.stdout.on("data", (d) => (child.out += d));
  child.stderr.on("data", (d) => (child.out += d));
  child.tag = name;
  return child;
}

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

let failed = false;
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
    if (r.status !== 0) failed = true;
  }
}

vite.kill();
arena.kill();
rmSync(dbDir, { recursive: true, force: true });
process.exit(failed ? 1 : 0);
