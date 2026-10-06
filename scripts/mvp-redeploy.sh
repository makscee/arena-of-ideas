#!/usr/bin/env bash
# Redeploy the Arena MVP test instance on m1 (mission #574): tailnet only, at
# https://m1.twin-pogona.ts.net/arena/ → 127.0.0.1:$PORT. Since 2026-10-06 it
# holds REAL players' data (Maks plays it; a SQLite file,
# ~/arena-mvp/data/arena-mvp.db, kept across redeploys). Only checked builds go
# live: the orchestrator fast-forwards the branch mvp-live to a checked commit
# and redeploys that. Never run mvp:bot, mvp:phone or any e2e against it: they
# register players in the real world.
#
#   npm run mvp:redeploy -- mvp-live     # the last checked build (the usual one)
#   npm run mvp:redeploy                 # the mission branch head
#   npm run mvp:redeploy -- <branch>     # any pushed branch
#   ARENA_MVP_WIPE=1 npm run mvp:redeploy -- --fresh
#                                        # only on Maks's word: wipe the world, the DB moves
#                                        # aside to data/arena-mvp.db.bak-<time>
#                                        # while the server is stopped. Every
#                                        # check runs before it stops; if the
#                                        # move fails, the server starts on the
#                                        # old DB and the redeploy exits 1
#   npm run mvp:redeploy -- --dry-run    # print what would run on m1, run nothing
#
# Invite-only since slice 13 (MVP_INVITES=1): players come from invite links,
# made on m1 with `cd ~/arena-mvp && npm run mvp:invite -- add <name>` (see
# server/src/mvp/invite-cli.ts), and "End day now" shows only to admin
# invites. ARENA_MVP_INVITES=0 redeploys it open (names, no links).
#
# Runs from anywhere with `ssh m1`; on m1 itself it runs locally. The server
# is a launchd agent (ru.makscee.arena-mvp), so it restarts on crash and login.
# The deployed commit goes into its env (MVP_BUILD): `build` on /api/v1/health.
# It prints "deployed <sha>" only once the new server answers /health; an old
# server that won't stop, or a new one that never answers, exits 1.
set -euo pipefail
BRANCH=mission-574-mvp
FRESH=0
DRY=0
for arg in "$@"; do
  case "$arg" in
    --fresh) FRESH=1
      [ "${ARENA_MVP_WIPE:-}" = 1 ] || { echo "--fresh wipes real players' world; only on Maks's word, with ARENA_MVP_WIPE=1" >&2; exit 2; } ;;
    --dry-run) DRY=1 ;;
    -*) echo "unknown option $arg (--fresh, --dry-run)" >&2; exit 2 ;;
    *) BRANCH="$arg" ;;
  esac
done
PORT="${ARENA_MVP_PORT:-8791}"
# Slice 10's fusion namer (Qwen3-4B since round 2, R2-4): a local model behind an OpenAI-compatible
# endpoint (mlx_lm.server), its own launchd agent so a redeploy doesn't reload
# it. The server falls back to a portmanteau whenever it is down.
NAMER_PORT="${ARENA_NAMER_PORT:-8792}"
NAMER_MODEL="${ARENA_NAMER_MODEL:-mlx-community/Qwen3-4B-Instruct-2507-4bit}"
HOST_ALIAS="${ARENA_MVP_HOST:-m1}"
INVITES="${ARENA_MVP_INVITES:-1}"

remote() {
  cat <<SCRIPT
set -euo pipefail
export PATH=/opt/homebrew/bin:/usr/local/bin:\$PATH
DIR="\$HOME/arena-mvp"
LABEL=ru.makscee.arena-mvp
[ -d "\$DIR/.git" ] || git clone -q https://github.com/makscee/arena-of-ideas.git "\$DIR"
cd "\$DIR"
git fetch -q origin "$BRANCH"
git checkout -q -B "$BRANCH" "origin/$BRANCH"
BUILD=\$(git rev-parse --short HEAD)
DB="\$DIR/data/arena-mvp.db"
mkdir -p "\$DIR/data"
if [ "$FRESH" = 1 ]; then
  # --fresh: every check before anything stops, so a DB that can't move aside
  # leaves the server running on it.
  [ -x scripts/mvp-db-aside.sh ] || { echo "--fresh: scripts/mvp-db-aside.sh is missing on $BRANCH; nothing stopped" >&2; exit 1; }
  scripts/mvp-db-aside.sh --check "\$DB" || { echo "--fresh: \$DB can't move aside; nothing stopped" >&2; exit 1; }
fi
npm ci --no-audit --no-fund --loglevel=error
npm run -s mvp:build
mkdir -p "\$HOME/Library/LaunchAgents"
NAMER=ru.makscee.arena-namer
NPLIST="\$HOME/Library/LaunchAgents/\$NAMER.plist"
cat > "\$NPLIST.new" <<PL
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>\$NAMER</string>
  <key>ProgramArguments</key><array>
    <string>/opt/homebrew/bin/uvx</string><string>--from</string><string>mlx-lm</string><string>mlx_lm.server</string>
    <string>--model</string><string>$NAMER_MODEL</string><string>--host</string><string>127.0.0.1</string><string>--port</string><string>$NAMER_PORT</string>
  </array>
  <key>EnvironmentVariables</key><dict><key>PATH</key><string>/opt/homebrew/bin:/usr/bin:/bin</string></dict>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>\$DIR/data/namer.log</string>
  <key>StandardErrorPath</key><string>\$DIR/data/namer.log</string>
</dict></plist>
PL
if ! cmp -s "\$NPLIST.new" "\$NPLIST" || ! launchctl print "gui/\$(id -u)/\$NAMER" >/dev/null 2>&1; then
  mv "\$NPLIST.new" "\$NPLIST"
  launchctl bootout "gui/\$(id -u)/\$NAMER" 2>/dev/null || true
  for i in \$(seq 1 20); do launchctl print "gui/\$(id -u)/\$NAMER" >/dev/null 2>&1 || break; sleep 0.5; done
  launchctl bootstrap "gui/\$(id -u)" "\$NPLIST"
else
  rm "\$NPLIST.new"
fi
PLIST="\$HOME/Library/LaunchAgents/\$LABEL.plist"
cat > "\$PLIST" <<PL
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>\$LABEL</string>
  <key>WorkingDirectory</key><string>\$DIR</string>
  <key>ProgramArguments</key><array>
    <string>/opt/homebrew/bin/node</string><string>--import</string><string>tsx/esm</string><string>server/src/mvp/main.ts</string>
  </array>
  <key>EnvironmentVariables</key><dict>
    <key>PORT</key><string>$PORT</string>
    <key>HOST</key><string>127.0.0.1</string>
    <key>BASE_PATH</key><string>/arena</string>
    <key>MVP_DEV</key><string>1</string>
    <key>MVP_INVITES</key><string>$INVITES</string>
    <key>MVP_DB</key><string>\$DB</string>
    <key>MVP_BUILD</key><string>\$BUILD</string>
    <key>ARENA_NAMER_URL</key><string>http://127.0.0.1:$NAMER_PORT/v1/chat/completions</string>
  </dict>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>\$DIR/data/server.log</string>
  <key>StandardErrorPath</key><string>\$DIR/data/server.log</string>
</dict></plist>
PL
# bootout returns before the job is gone; bootstrap too early fails with EIO.
launchctl bootout "gui/\$(id -u)/\$LABEL" 2>/dev/null || true
for i in \$(seq 1 20); do launchctl print "gui/\$(id -u)/\$LABEL" >/dev/null 2>&1 || break; sleep 0.5; done
# Still loaded after the wait: the old server keeps running. Say so and stop,
# never report a deploy that didn't happen. By now the checkout, npm ci and
# the build have already rewritten \$DIR, and the old server serves mobile/dist
# from disk, so phones get the new client against the old server's API.
if launchctl print "gui/\$(id -u)/\$LABEL" >/dev/null 2>&1; then
  echo "redeploy failed: \$LABEL didn't stop within 10s, so the old server still runs. \$DIR is already at \$BUILD (checkout, npm ci, build), so phones get the new client against the old server's API until it restarts. The DB stayed. Run the redeploy again: npm run mvp:redeploy -- $BRANCH$( [ "$FRESH" = 1 ] && echo " --fresh")" >&2
  exit 1
fi
FAILED=
if [ "$FRESH" = 1 ]; then
  # --fresh: the server is stopped; an empty DB on start, the bots reseed the
  # champion. If the move fails, the server starts on the old DB.
  if ! scripts/mvp-db-aside.sh "\$DB"; then
    FAILED="moving \$DB aside failed; the server starts on the old DB"
  fi
fi
launchctl bootstrap "gui/\$(id -u)" "\$PLIST"
TS=/Applications/Tailscale.app/Contents/MacOS/Tailscale
\$TS serve --bg --set-path /arena "http://127.0.0.1:$PORT" >/dev/null
HEALTH=
for i in \$(seq 1 30); do
  HEALTH=\$(curl -fsS --max-time 2 "http://127.0.0.1:$PORT/arena/api/v1/health" 2>/dev/null) && break
  HEALTH=
  sleep 1
done
curl -fsS --max-time 5 "http://127.0.0.1:$NAMER_PORT/v1/models" >/dev/null 2>&1 && echo "namer up on :$NAMER_PORT" || echo "namer not up yet on :$NAMER_PORT (fusions use the portmanteau until it is)"
# No answer in 30s: the new server is down (launchd keeps restarting it).
# Never report it deployed.
if [ -z "\$HEALTH" ]; then
  echo "redeploy failed: the new server (\$BUILD, $BRANCH) never answered http://127.0.0.1:$PORT/arena/api/v1/health within 30s, so the Arena isn't serving. The end of \$DIR/data/server.log:" >&2
  tail -n 20 "\$DIR/data/server.log" >&2 2>/dev/null || true
  [ -z "\$FAILED" ] || echo "--fresh failed too: \$FAILED" >&2
  echo "Fix it and redeploy (npm run mvp:redeploy -- <branch>), or go back to a branch that worked." >&2
  exit 1
fi
echo "deployed \$BUILD ($BRANCH): \$HEALTH"
if [ -n "\$FAILED" ]; then
  echo "--fresh failed: \$FAILED" >&2
  exit 1
fi
SCRIPT
}

if [ "$DRY" = 1 ]; then
  echo "# dry run: would run on $HOST_ALIAS (branch $BRANCH, fresh $FRESH):"
  remote
  exit 0
fi
if [ "$(hostname -s 2>/dev/null)" = "m1" ] || [ "$(hostname 2>/dev/null)" = "m1.twin-pogona.ts.net" ]; then
  remote | bash
else
  remote | ssh -o ConnectTimeout=10 "$HOST_ALIAS" bash
fi
echo "open https://m1.twin-pogona.ts.net/arena/"
