// e2e orchestrator (`npm run e2e`): boots vite on its own port, waits for the
// ready line (bounded), runs every probe as a child with a hard timeout, and
// always tears the server down. The probes need real layout (wrap, collapse,
// elementFromPoint) that jsdom cannot see — this keeps the PRD #012 layout
// checks repeatable. Excluded from vitest: probes are plain scripts, not
// *.test.* files.

import { spawn, spawnSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const PORT = process.env.AOI_E2E_PORT ?? "5280";
const BASE = `http://localhost:${PORT}`;

const server = spawn("npx", ["vite", "--port", PORT, "--strictPort"], { cwd: root });
let serverOut = "";
server.stdout.on("data", (d) => (serverOut += d));
server.stderr.on("data", (d) => (serverOut += d));

const ready = await new Promise((resolve) => {
  const t0 = Date.now();
  const poll = setInterval(() => {
    if (serverOut.includes("Local:")) {
      clearInterval(poll);
      resolve(true);
    } else if (Date.now() - t0 > 30_000 || server.exitCode !== null) {
      clearInterval(poll);
      resolve(false);
    }
  }, 200);
});

let failed = !ready;
if (!ready) {
  console.error(`vite did not become ready on :${PORT} within 30s\n${serverOut}`);
} else {
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

server.kill();
process.exit(failed ? 1 : 0);
