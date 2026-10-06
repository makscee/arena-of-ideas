// scripts/mvp-redeploy.sh (mission #574): the m1 script it prints
// with --dry-run, run here against a temp HOME with launchctl, git, npm, curl
// and tailscale stubbed. Nothing touches m1 or this machine's services.
import { spawnSync } from "node:child_process";
import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterEach, describe, expect, it } from "vitest";

const ROOT = resolve(import.meta.dirname, "..");
/** What https://arena.makscee.ru/arena/api/v1/health answers today: the June
 * web client's title screen (its SPA fallback), cut to the markers. */
const JUNE = `<!doctype html>
<html lang="en">
  <head><title>Arena of Ideas</title></head>
  <body><main>
      <section id="title-view">
        <h1 class="title-name">Arena of Ideas</h1>`;
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
  // NO_HEALTH: the new server never answers /health; STUB_HEALTH: what it
  // says. PUBLIC_CODE / PUBLIC_HEALTH: what https://arena.makscee.ru/arena/
  // answers (default: the June page; 000 is no answer), PUBLIC_EXIT: curl's
  // exit after printing it (18: cut off mid-transfer); OWN_CODE: what this
  // host's tailnet /arena answers (default 502: the server is down, the serve
  // on), OWN_EXIT: curl failing there (7 refused, 6 DNS, 28 timeout: prints 000)
  curl: `#!/bin/bash
case "$*" in
  *arena.makscee.ru*)
    C="\${PUBLIC_CODE:-200}"; B="\${PUBLIC_HEALTH:-$JUNE}"
    [ "$C" = 000 ] && { printf '000'; exit 28; }
    printf '%s\\n%s' "$B" "$C"; exit "\${PUBLIC_EXIT:-0}" ;;
  *twin-pogona*) [ -n "\${OWN_EXIT:-}" ] && { printf '000'; exit "$OWN_EXIT"; }; printf '%s' "\${OWN_CODE:-502}"; exit 0 ;;
  *api/v1/health*) [ -n "\${NO_HEALTH:-}" ] && exit 7; [ -e "$HOME/loaded/ru.makscee.arena-mvp" ] || exit 7; D='{"ok":true,"build":"abc1234","invites":true}'; echo "\${STUB_HEALTH:-$D}"; exit 0 ;;
esac
echo '{"ok":true}'`,
  // SERVE_FAIL: \`serve --bg\` fails
  tailscale: `#!/bin/bash
echo "$*" >> "$HOME/tailscale.log"
[ -n "\${SERVE_FAIL:-}" ] && [ "$2" = --bg ] && { echo "serve config denied" >&2; exit 1; }
exit 0`,
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
  const run = spawnSync("bash", [], { input: script, encoding: "utf8", env: { HOME: home, PATH: `${home}/bin:/usr/bin:/bin`, JUNE, ...env } });
  const log = existsSync(join(home, "launchctl.log")) ? readFileSync(join(home, "launchctl.log"), "utf8") : "";
  const ts = existsSync(join(home, "tailscale.log")) ? readFileSync(join(home, "tailscale.log"), "utf8") : "";
  return { status: run.status, stdout: run.stdout, stderr: run.stderr, log, ts };
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
      // invite-only: its invites were never checked, so it doesn't stay loaded
      // to come up later unchecked
      expect(r.stderr).toContain("invites were never checked, so it is stopped and the /arena serve is off");
      expect(r.stderr).not.toContain("STILL LOADED");
      expect(existsSync(join(home, "loaded/ru.makscee.arena-mvp"))).toBe(false);
      expect(r.ts.trim().split("\n").at(-1)).toBe("serve --set-path /arena off");
    }
    // an open deploy (ARENA_MVP_INVITES=0) has nothing to check: launchd keeps retrying it
    const { home } = host();
    const r = deploy(home, { NO_HEALTH: "1", ARENA_MVP_INVITES: "0" }, []);
    expect(r.status).toBe(1);
    expect(r.stderr).not.toContain("invites were never checked");
    expect(existsSync(join(home, "loaded/ru.makscee.arena-mvp"))).toBe(true);
  });

  it("a normal redeploy restarts the server on the same DB and reports the build", () => {
    const { home, db } = host();
    const r = deploy(home, {}, []);
    expect([r.status, r.stderr]).toEqual([0, ""]);
    expect(r.stdout).toContain("deployed abc1234 (mission-574-mvp)");
    expect(r.log).toMatch(/bootout gui\/\d+\/ru\.makscee\.arena-mvp[\s\S]*bootstrap gui\/\d+ .*ru\.makscee\.arena-mvp\.plist/);
    expect(readFileSync(db, "utf8")).toBe("old world");
  });

  it("never leaves an open server running: a /health without invites: true stops it, turns /arena off, exits 1", { timeout: 15_000 }, () => {
    for (const health of ['{"ok":true}', '{"ok":true,"invites":false}']) {
      const { home } = host();
      const r = deploy(home, { STUB_HEALTH: health }, []);
      expect(r.status, health).toBe(1);
      expect(r.stdout).not.toContain("deployed");
      expect(r.stderr).toContain("is OPEN, not invite-only");
      expect(r.stderr).toContain("the /arena serve is off");
      expect(r.stderr).not.toContain("STILL LOADED");
      expect(existsSync(join(home, "loaded/ru.makscee.arena-mvp"))).toBe(false);
      expect(r.ts.trim().split("\n").at(-1)).toBe("serve --set-path /arena off");
    }
  });

  it("ARENA_MVP_INVITES=0 deploys an open server with a warning, and refuses while arena.makscee.ru serves this host", { timeout: 15_000 }, () => {
    const open = { ARENA_MVP_INVITES: "0", STUB_HEALTH: '{"ok":true,"build":"abc1234","invites":false}' };
    const off = host();
    const r = deploy(off.home, open, []);
    expect(r.status).toBe(0);
    expect(r.stdout).toContain("deployed abc1234");
    expect(r.stderr).toContain("WARNING: ARENA_MVP_INVITES=0 deploys an OPEN server");
    // The domain still points at another server (the June build): fine.
    const elsewhere = host();
    expect(deploy(elsewhere.home, { ...open, PUBLIC_HEALTH: '{"ok":true,"build":"4a276d3"}' }, []).status).toBe(0);
    const pub = host();
    const f = deploy(pub.home, { ...open, PUBLIC_HEALTH: open.STUB_HEALTH }, []);
    expect(f.status).toBe(1);
    expect(f.stderr).toContain("https://arena.makscee.ru/arena/ serves this host (build abc1234)");
    expect(f.log).not.toContain("bootout");
    const forced = host();
    expect(deploy(forced.home, { ...open, PUBLIC_HEALTH: open.STUB_HEALTH, ARENA_MVP_OPEN_PUBLIC: "1" }, []).status).toBe(0);
  });

  it("ARENA_MVP_INVITES=0 with this host's server down: allowed only when the domain clearly isn't this host, and always says why it refused", { timeout: 30_000 }, () => {
    const open = { ARENA_MVP_INVITES: "0", STUB_HEALTH: '{"ok":true,"build":"abc1234","invites":false}' };
    // the server is down before the deploy (a failed check booted it out, a crash loop)
    const down = () => {
      const h = host();
      rmSync(join(h.home, "loaded/ru.makscee.arena-mvp"));
      return h;
    };
    const allowed: [string, Record<string, string>][] = [
      ["the June page", {}],
      ["a 404 while this host's /arena answers 502", { PUBLIC_CODE: "404", PUBLIC_HEALTH: "" }],
    ];
    for (const [what, env] of allowed) {
      const { home } = down();
      const r = deploy(home, { ...open, ...env }, []);
      expect([r.status, r.stdout.includes("deployed abc1234")], what).toEqual([0, true]);
      expect(r.stderr, what).not.toContain("refused");
    }
    const refused: [string, Record<string, string>, string][] = [
      ["a 502", { PUBLIC_CODE: "502", PUBLIC_HEALTH: "bad gateway" }, "answers 502: that is what this host down behind mcow's proxy looks like"],
      ["a 504", { PUBLIC_CODE: "504", PUBLIC_HEALTH: "" }, "answers 504"],
      ["no answer", { PUBLIC_CODE: "000" }, "didn't answer within 10s"],
      ["Arena JSON", { PUBLIC_HEALTH: '{"ok":true,"build":"4a276d3"}' }, "answers with an Arena server's /health, and this host's server doesn't answer"],
      ["a 404 while this host's /arena is a 404 too", { PUBLIC_CODE: "404", PUBLIC_HEALTH: "", OWN_CODE: "404" }, "(its /arena serve is off)"],
      // this host can't read its own ts.net URL: curl prints 000 and fails
      ...(["7", "6", "28"] as const).map((e): [string, Record<string, string>, string] => [`a 404 while this host's own /arena fails (curl ${e})`, { PUBLIC_CODE: "404", PUBLIC_HEALTH: "", OWN_EXIT: e }, "this host can't read its own https://m1.twin-pogona.ts.net/arena/api/v1/health within 10s"]),
      ["the June page cut off mid-transfer", { PUBLIC_EXIT: "18" }, "or its answer was cut off"],
      ["HTML that isn't the June page", { PUBLIC_HEALTH: "<!doctype html><html><body>Bad gateway</body></html>" }, "answers 200 with neither the June page nor an Arena /health"],
    ];
    for (const [what, env, why] of refused) {
      const { home } = down();
      const r = deploy(home, { ...open, ...env }, []);
      expect(r.status, what).toBe(1);
      expect(r.stderr, what).toContain("ARENA_MVP_INVITES=0 refused: https://arena.makscee.ru/arena/");
      expect(r.stderr, what).toContain(why);
      expect(r.stderr, what).toContain("Switch the domain off first");
      expect(r.log, what).toBe("");
      expect(r.stdout, what).not.toContain("deployed");
    }
    // with this host's server up, a 5xx refuses too
    const up = host();
    expect(deploy(up.home, { ...open, PUBLIC_CODE: "502", PUBLIC_HEALTH: "" }, []).status).toBe(1);
    // ARENA_MVP_OPEN_PUBLIC=1 overrides, and says what it overrode
    const forced = down();
    const f = deploy(forced.home, { ...open, PUBLIC_CODE: "502", PUBLIC_HEALTH: "", ARENA_MVP_OPEN_PUBLIC: "1" }, []);
    expect(f.status).toBe(0);
    expect(f.stderr).toContain("ARENA_MVP_OPEN_PUBLIC=1: deploying open although https://arena.makscee.ru/arena/ answers 502");
  });

  it("a `tailscale serve` that fails stops the new server and turns /arena off: exit 1, with why", { timeout: 15_000 }, () => {
    for (const env of [{}, { ARENA_MVP_INVITES: "0", STUB_HEALTH: '{"ok":true,"build":"abc1234","invites":false}' }]) {
      const { home } = host();
      const r = deploy(home, { SERVE_FAIL: "1", ...env }, []);
      expect(r.status, JSON.stringify(env)).toBe(1);
      expect(r.stdout).not.toContain("deployed");
      expect(r.stderr).toContain("tailscale serve --set-path /arena didn't take (serve config denied)");
      expect(r.stderr).toContain("the /arena serve is off");
      expect(r.stderr).not.toContain("STILL LOADED");
      expect(existsSync(join(home, "loaded/ru.makscee.arena-mvp"))).toBe(false);
      expect(r.ts.trim().split("\n").at(-1)).toBe("serve --set-path /arena off");
    }
  });

  it("the failed invites check's message says to switch the domain off before an open rollback", () => {
    const { home } = host();
    const r = deploy(home, { STUB_HEALTH: '{"ok":true}' }, []);
    expect(r.stderr).toContain("roll back: switch the domain off first (homelab Caddyfile.j2, arena.makscee.ru stops proxying /arena* to m1), then ARENA_MVP_INVITES=0 npm run mvp:redeploy -- <ref>");
  });

  it("takes ARENA_MVP_INVITES 0 or 1 only", () => {
    for (const v of ["yes", "true", ""]) {
      const r = spawnSync(join(ROOT, "scripts/mvp-redeploy.sh"), ["--dry-run"], { encoding: "utf8", env: { ...process.env, ARENA_MVP_INVITES: v } });
      expect(v === "" ? r.status : [r.status, r.stderr.trim()], v).toEqual(v === "" ? 0 : [2, `ARENA_MVP_INVITES is 0 or 1, not '${v}'`]);
    }
  });

  it("mvp-db-aside.sh --check refuses a DB path whose directory is missing, and moves nothing", () => {
    const { home, db } = host();
    const aside = join(ROOT, "scripts/mvp-db-aside.sh");
    expect(spawnSync(aside, ["--check", join(home, "nope/arena-mvp.db")]).status).toBe(1);
    expect(spawnSync(aside, ["--check", db], { encoding: "utf8" }).stdout).toContain("can move aside");
    expect(readFileSync(db, "utf8")).toBe("old world");
  });
});
