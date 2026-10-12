#!/bin/bash
# The Arena lab (#1026): a scenes server on m1 for judging changes from
# #scene= links, tailnet only at https://m1.twin-pogona.ts.net/arena-lab/.
# It runs a branch (default main) from its own clone ~/arena-lab with
# MVP_DEV=1 ARENA_SCENES=1 MVP_DB=:memory: (a world built in memory at each
# start, never the live one), under launchd ru.makscee.arena-lab on :8796,
# and rewrites the index page ~/reports/arena-scenes.html (/r/arena-scenes.html).
#   scripts/arena-lab.sh [branch]     run on m1; e.g. a card's branch for its check
#   npm run mvp:lab -- [branch]
set -euo pipefail
BRANCH="${1:-main}"
PORT="${ARENA_LAB_PORT:-8796}"
DIR="${ARENA_LAB_DIR:-$HOME/arena-lab}"
LABEL=ru.makscee.arena-lab
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
TS=/Applications/Tailscale.app/Contents/MacOS/Tailscale
# launchd's node, so better-sqlite3 is built for the node that runs it.
export PATH=/opt/homebrew/bin:/usr/local/bin:$PATH
NODE=$(command -v node)

if [ ! -d "$DIR/.git" ]; then
  git clone --depth 1 --no-single-branch https://github.com/makscee/arena-of-ideas.git "$DIR"
fi
cd "$DIR"
git fetch --depth 1 origin "$BRANCH"
git checkout -q -B lab FETCH_HEAD
BUILD=$(git rev-parse --short=8 HEAD)
if [ ! -f node_modules/.lab-lock ] || ! cmp -s package-lock.json node_modules/.lab-lock; then
  npm ci --no-audit --no-fund
  cp package-lock.json node_modules/.lab-lock
fi
"$NODE" node_modules/vite/bin/vite.js build --config mobile/vite.config.ts --base /arena-lab/ --logLevel warn

mkdir -p "$DIR/data" "$(dirname "$PLIST")"
cat > "$PLIST" <<PL
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>$LABEL</string>
  <key>WorkingDirectory</key><string>$DIR</string>
  <key>ProgramArguments</key><array>
    <string>$NODE</string><string>--import</string><string>tsx/esm</string><string>server/src/mvp/main.ts</string>
  </array>
  <key>EnvironmentVariables</key><dict>
    <key>PORT</key><string>$PORT</string>
    <key>HOST</key><string>127.0.0.1</string>
    <key>BASE_PATH</key><string>/arena-lab</string>
    <key>MVP_DEV</key><string>1</string>
    <key>ARENA_SCENES</key><string>1</string>
    <key>MVP_DB</key><string>:memory:</string>
    <key>ARENA_TUNER</key><string>instant</string>
    <key>ARENA_IDEA_READER</key><string>fake</string>
    <key>MVP_BUILD</key><string>$BUILD ($BRANCH)</string>
  </dict>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>$DIR/data/server.log</string>
  <key>StandardErrorPath</key><string>$DIR/data/server.log</string>
</dict></plist>
PL
launchctl bootout "gui/$(id -u)/$LABEL" 2>/dev/null || true
# bootout returns before the old job is gone; bootstrap fails (5) until it is.
for _ in $(seq 1 30); do launchctl print "gui/$(id -u)/$LABEL" >/dev/null 2>&1 || break; sleep 1; done
launchctl bootstrap "gui/$(id -u)" "$PLIST"
"$TS" serve --bg --set-path /arena-lab "http://127.0.0.1:$PORT" >/dev/null

# Under load (m1 at 20+) the start takes minutes: it seeds the day's champion first.
for _ in $(seq 1 300); do
  curl -fsS --max-time 2 "http://127.0.0.1:$PORT/arena-lab/api/v1/health" >/dev/null 2>&1 && break
  sleep 1
done
curl -fsS --max-time 5 "http://127.0.0.1:$PORT/arena-lab/api/v1/health" >/dev/null || { echo "the lab didn't answer in 5 min: $DIR/data/server.log" >&2; exit 1; }
"$NODE" --import tsx/esm scripts/scenes-index.ts
echo "lab: $BRANCH at $BUILD on https://m1.twin-pogona.ts.net/arena-lab/ (index: https://m1.twin-pogona.ts.net/r/arena-scenes.html)"
