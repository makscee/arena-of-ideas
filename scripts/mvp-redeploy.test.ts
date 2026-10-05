// scripts/mvp-redeploy.sh --fresh (mission #574): the m1 script it prints
// with --dry-run, run here against a temp HOME with launchctl, git, npm, curl
// and tailscale stubbed. Nothing touches m1 or this machine's services.
import { spawnSync } from "node:child_process";
import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, describe, expect, it } from "vitest";

const ROOT = resolve(import.meta.dirname, "..");
const homes: string[] = [];

const STUBS: Record<string, string> = {
  // per-label state: print answers whether the label is loaded
  launchctl: `#!/bin/bash
echo "$*" >> "$HOME/launchctl.log"
label() { basename "$1" .plist; }
case "$1" in
  print) [ -e "$HOME/loaded/$(basename "$2")" ] ;;
  bootout) rm -f "$HOME/loaded/$(basename "$2")"; [ -n "\${BREAK_DATA:-}" ] && chmod a-w "$HOME/arena-mvp/data"; exit 0 ;;
  bootstrap) touch "$HOME/loaded/$(label "$3")" ;;
esac`,
  git: `#!/bin/bash
[ "$1" = rev-parse ] && echo abc1234; exit 0`,
  npm: "#!/bin/bash\nexit 0",
  curl: "#!/bin/bash\necho '{\"ok\":true}'",
  tailscale: "#!/bin/bash\nexit 0",
};

/** A temp HOME with a checkout, a DB with a -wal, the server loaded, and stubs. */
function host(o: { asideScript?: boolean } = {}) {
  const home = mkdtempSync(join(tmpdir(), "mvp-redeploy-"));
  homes.push(home);
  const dir = join(home, "arena-mvp");
  for (const d of [".git", "scripts", "data"]) mkdirSync(join(dir, d), { recursive: true });
  mkdirSync(join(home, "loaded"));
  mkdirSync(join(home, "bin"));
  for (const [name, body] of Object.entries(STUBS)) writeFileSync(join(home, "bin", name), body, { mode: 0o755 });
  if (o.asideScript !== false) copyFileSync(join(ROOT, "scripts/mvp-db-aside.sh"), join(dir, "scripts/mvp-db-aside.sh"));
  writeFileSync(join(dir, "data/arena-mvp.db"), "old world");
  writeFileSync(join(dir, "data/arena-mvp.db-wal"), "old wal");
  writeFileSync(join(home, "loaded/ru.makscee.arena-mvp"), "");
  return { home, dir, db: join(dir, "data/arena-mvp.db") };
}

/** The script --dry-run prints for m1, run against `home` (PATH and the
 * Tailscale binary point at the stubs). */
function deploy(home: string, env: Record<string, string> = {}) {
  const dry = spawnSync(join(ROOT, "scripts/mvp-redeploy.sh"), ["--dry-run", "--fresh"], { encoding: "utf8" });
  expect(dry.status).toBe(0);
  const script = dry.stdout
    .replace("export PATH=/opt/homebrew/bin:/usr/local/bin:$PATH", `export PATH=${home}/bin:/usr/bin:/bin`)
    .replace("TS=/Applications/Tailscale.app/Contents/MacOS/Tailscale", `TS=${home}/bin/tailscale`);
  expect(script).toContain(`${home}/bin/tailscale`);
  const run = spawnSync("bash", [], { input: script, encoding: "utf8", env: { HOME: home, PATH: `${home}/bin:/usr/bin:/bin`, ...env } });
  const log = existsSync(join(home, "launchctl.log")) ? readFileSync(join(home, "launchctl.log"), "utf8") : "";
  return { status: run.status, stderr: run.stderr, log };
}

afterEach(() => {
  for (const h of homes.splice(0)) {
    chmodSync(join(h, "arena-mvp/data"), 0o755);
    rmSync(h, { recursive: true, force: true });
  }
});

describe("mvp-redeploy --fresh", () => {
  it("moves the DB and its -wal aside while the server is stopped, then starts it", () => {
    const { home, dir, db } = host();
    const r = deploy(home);
    expect(r.stderr).toBe("");
    expect(r.status).toBe(0);
    expect(existsSync(db)).toBe(false);
    expect(readdirSync(join(dir, "data")).filter((f) => f.includes(".bak-")).sort()).toEqual([expect.stringMatching(/\.db\.bak-\d{8}T\d{6}Z$/), expect.stringMatching(/\.db\.bak-\d{8}T\d{6}Z-wal$/)]);
    expect(r.log).toMatch(/bootout gui\/\d+\/ru\.makscee\.arena-mvp[\s\S]*bootstrap gui\/\d+ .*ru\.makscee\.arena-mvp\.plist/);
    expect(readFileSync(join(home, "Library/LaunchAgents/ru.makscee.arena-mvp.plist"), "utf8")).toContain(`<key>MVP_DB</key><string>${db}</string>`);
  });

  it("checks before it stops anything: without mvp-db-aside.sh it exits 1 and the server keeps running", () => {
    const { home, db } = host({ asideScript: false });
    const r = deploy(home);
    expect(r.status).toBe(1);
    expect(r.stderr).toContain("mvp-db-aside.sh is missing");
    expect(r.log).not.toContain("arena-mvp");
    expect(readFileSync(db, "utf8")).toBe("old world");
  });

  it("a move that fails starts the server on the old DB and exits 1", () => {
    const { home, db } = host();
    const r = deploy(home, { BREAK_DATA: "1" }); // the data dir goes read-only as the server stops
    expect(r.status).toBe(1);
    expect(r.stderr).toContain("--fresh failed: moving");
    expect(readFileSync(db, "utf8")).toBe("old world");
    expect(readFileSync(`${db}-wal`, "utf8")).toBe("old wal");
    expect(r.log).toMatch(/bootstrap gui\/\d+ .*ru\.makscee\.arena-mvp\.plist/);
    expect(existsSync(join(home, "loaded/ru.makscee.arena-mvp"))).toBe(true);
  });

  it("mvp-db-aside.sh --check refuses a DB path whose directory is missing, and moves nothing", () => {
    const { home, db } = host();
    const aside = join(ROOT, "scripts/mvp-db-aside.sh");
    expect(spawnSync(aside, ["--check", join(home, "nope/arena-mvp.db")]).status).toBe(1);
    expect(spawnSync(aside, ["--check", db], { encoding: "utf8" }).stdout).toContain("can move aside");
    expect(readFileSync(db, "utf8")).toBe("old world");
  });
});
