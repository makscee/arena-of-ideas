// scripts/mvp-redeploy.sh (mission #574): the m1 script it prints
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
  bootout) [ -n "\${STUCK:-}" ] || rm -f "$HOME/loaded/$(basename "$2")"; [ -n "\${BREAK_DATA:-}" ] && chmod a-w "$HOME/arena-mvp/data"; exit 0 ;;
  bootstrap) touch "$HOME/loaded/$(label "$3")" ;;
esac`,
  git: `#!/bin/bash
[ "$1" = rev-parse ] && echo abc1234; exit 0`,
  npm: "#!/bin/bash\nexit 0",
  // NO_HEALTH: the new server never answers /health; STUB_HEALTH: what it says
  curl: `#!/bin/bash
case "$*" in *api/v1/health*) [ -n "\${NO_HEALTH:-}" ] && exit 7; D='{"ok":true,"invites":true}'; echo "\${STUB_HEALTH:-$D}"; exit 0 ;; esac
echo '{"ok":true}'`,
  // FUNNEL: Funnel is on
  tailscale: `#!/bin/bash
[ "$1" = funnel ] && [ -n "\${FUNNEL:-}" ] && echo "https://m1.twin-pogona.ts.net:8443 (Funnel on)"; exit 0`,
  // the waits for launchd don't wait here
  sleep: "#!/bin/bash\nexit 0",
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
function deploy(home: string, env: Record<string, string> = {}, args = ["--fresh"]) {
  const dry = spawnSync(join(ROOT, "scripts/mvp-redeploy.sh"), ["--dry-run", ...args], { encoding: "utf8", env: { ...process.env, ARENA_MVP_WIPE: "1", ...env } });
  expect(dry.status).toBe(0);
  const script = dry.stdout
    .replace("export PATH=/opt/homebrew/bin:/usr/local/bin:$PATH", `export PATH=${home}/bin:/usr/bin:/bin`)
    .replace("TS=/Applications/Tailscale.app/Contents/MacOS/Tailscale", `TS=${home}/bin/tailscale`);
  expect(script).toContain(`${home}/bin/tailscale`);
  const run = spawnSync("bash", [], { input: script, encoding: "utf8", env: { HOME: home, PATH: `${home}/bin:/usr/bin:/bin`, ...env } });
  const log = existsSync(join(home, "launchctl.log")) ? readFileSync(join(home, "launchctl.log"), "utf8") : "";
  return { status: run.status, stdout: run.stdout, stderr: run.stderr, log };
}

afterEach(() => {
  for (const h of homes.splice(0)) {
    chmodSync(join(h, "arena-mvp/data"), 0o755);
    rmSync(h, { recursive: true, force: true });
  }
});

describe("mvp-redeploy (--fresh and plain)", () => {
  it("refuses --fresh without ARENA_MVP_WIPE=1: the world holds real players", () => {
    const env = { ...process.env };
    delete env.ARENA_MVP_WIPE;
    const r = spawnSync(join(ROOT, "scripts/mvp-redeploy.sh"), ["--dry-run", "--fresh"], { encoding: "utf8", env });
    expect(r.status).toBe(2);
    expect(r.stderr).toContain("ARENA_MVP_WIPE=1");
  });

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

  it("an old server that doesn't stop fails the redeploy loudly: exit 1, no \"deployed\" line, nothing bootstrapped", { timeout: 15_000 }, () => {
    for (const args of [[], ["--fresh"]]) {
      const { home, db } = host();
      const r = deploy(home, { STUCK: "1" }, args); // launchd never lets the old job go
      expect(r.status, args.join(" ")).toBe(1);
      expect(r.stderr).toContain("didn't stop within 10s, so the old server still runs");
      // the checkout and build already happened: the message says so and what to run
      expect(r.stderr).toContain("is already at abc1234 (checkout, npm ci, build), so phones get the new client against the old server's API");
      expect(r.stderr).toContain(`Run the redeploy again: npm run mvp:redeploy -- mission-574-mvp${args.length ? " --fresh" : ""}`);
      expect(r.stdout).not.toContain("deployed");
      expect(r.log).not.toMatch(/bootstrap gui\/\d+ .*ru\.makscee\.arena-mvp\.plist/);
      expect(readFileSync(db, "utf8")).toBe("old world");
    }
  });

  it("a new server that never answers /health fails the redeploy: exit 1, no \"deployed\" line, the server log's end", { timeout: 15_000 }, () => {
    for (const args of [[], ["--fresh"]]) {
      const { home, dir } = host();
      writeFileSync(join(dir, "data/server.log"), "SyntaxError: boom\n");
      const r = deploy(home, { NO_HEALTH: "1" }, args);
      expect(r.status, args.join(" ")).toBe(1);
      expect(r.stdout).not.toContain("deployed");
      expect(r.stderr).toContain("the new server (abc1234, mission-574-mvp) never answered http://127.0.0.1:8791/arena/api/v1/health within 30s");
      expect(r.stderr).toContain("SyntaxError: boom");
      expect(r.log).toMatch(/bootstrap gui\/\d+ .*ru\.makscee\.arena-mvp\.plist/);
    }
  });

  it("a normal redeploy restarts the server on the same DB and reports the build", () => {
    const { home, db } = host();
    const r = deploy(home, {}, []);
    expect([r.status, r.stderr]).toEqual([0, ""]);
    expect(r.stdout).toContain("deployed abc1234 (mission-574-mvp)");
    expect(r.log).toMatch(/bootout gui\/\d+\/ru\.makscee\.arena-mvp[\s\S]*bootstrap gui\/\d+ .*ru\.makscee\.arena-mvp\.plist/);
    expect(readFileSync(db, "utf8")).toBe("old world");
  });

  it("refuses to report an open server as deployed: a /health without invites: true exits 1 (a pre-slice-13 ref)", () => {
    for (const health of ['{"ok":true}', '{"ok":true,"invites":false}']) {
      const { home } = host();
      const r = deploy(home, { STUB_HEALTH: health }, []);
      expect(r.status, health).toBe(1);
      expect(r.stdout).not.toContain("deployed");
      expect(r.stderr).toContain("is OPEN, not invite-only");
      expect(r.stderr).toContain("funnel --https=8443 off");
    }
  });

  it("ARENA_MVP_INVITES=0 deploys an open server with a warning, and refuses while Funnel is on", () => {
    const open = host();
    const r = deploy(open.home, { ARENA_MVP_INVITES: "0", STUB_HEALTH: '{"ok":true,"invites":false}' }, []);
    expect(r.status).toBe(0);
    expect(r.stdout).toContain("deployed abc1234");
    expect(r.stderr).toContain("WARNING: ARENA_MVP_INVITES=0 deploys an OPEN server");
    const funnel = host();
    const f = deploy(funnel.home, { ARENA_MVP_INVITES: "0", FUNNEL: "1" }, []);
    expect(f.status).toBe(1);
    expect(f.stderr).toContain("Tailscale Funnel is on");
    expect(f.log).not.toContain("bootout");
  });

  it("mvp-db-aside.sh --check refuses a DB path whose directory is missing, and moves nothing", () => {
    const { home, db } = host();
    const aside = join(ROOT, "scripts/mvp-db-aside.sh");
    expect(spawnSync(aside, ["--check", join(home, "nope/arena-mvp.db")]).status).toBe(1);
    expect(spawnSync(aside, ["--check", db], { encoding: "utf8" }).stdout).toContain("can move aside");
    expect(readFileSync(db, "utf8")).toBe("old world");
  });
});
