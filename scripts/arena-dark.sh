#!/bin/bash
# Arena goes dark (#1044, mission 1043): arena.makscee.ru/arena shows a holding
# page instead of the game. Run it on m1 (or `ssh m1 zsh -lic '…'`):
#
#   scripts/arena-dark.sh [branch]      default: mission-1043-rules-v2
#
# What it does, in this order:
#   1. builds the holding server (holding/server.mjs + the phone client) in its
#      own clone ~/arena-holding, while the live game still runs;
#   2. stops the live game (launchd ru.makscee.arena-mvp), copies its database
#      files to ~/arena-mvp/data/dark-backup-<time>/ and moves its plist aside
#      to ~/arena-mvp/data/live.plist.aside, so a login doesn't start it again;
#   3. starts the holding server (launchd ru.makscee.arena-holding) on the same
#      port, 8791, so mcow's Caddy and tailscale serve /arena need no change;
#   4. checks /health says dark, locally and on the public URL. If the holding
#      server never answers, it puts the live game back and exits 1.
# The live world's data is never opened, wiped or migrated: the holding server
# has no database. The reopen (slice 10 of #1043) is on Maks's word:
#   npm run mvp:redeploy -- mvp-live    # it stops the holding server first
# Back out right now: cp ~/arena-mvp/data/live.plist.aside
#   ~/Library/LaunchAgents/ru.makscee.arena-mvp.plist; launchctl bootout
#   gui/$(id -u)/ru.makscee.arena-holding; launchctl bootstrap gui/$(id -u)
#   ~/Library/LaunchAgents/ru.makscee.arena-mvp.plist
set -euo pipefail
BRANCH="${1:-mission-1043-rules-v2}"
PORT="${ARENA_MVP_PORT:-8791}"
DIR="${ARENA_HOLDING_DIR:-$HOME/arena-holding}"
LIVE_DIR="$HOME/arena-mvp"
LABEL=ru.makscee.arena-holding
LIVE_LABEL=ru.makscee.arena-mvp
AGENTS="$HOME/Library/LaunchAgents"
PLIST="$AGENTS/$LABEL.plist"
LIVE_PLIST="$AGENTS/$LIVE_LABEL.plist"
ASIDE="$LIVE_DIR/data/live.plist.aside"
PUBLIC=https://arena.makscee.ru/arena/api/v1/health
TS=/Applications/Tailscale.app/Contents/MacOS/Tailscale
UIDN=$(id -u)
export PATH=/opt/homebrew/bin:/usr/local/bin:$PATH
NODE=$(command -v node)

gone() { # wait until launchd has forgotten a label
  for _ in $(seq 1 30); do launchctl print "gui/$UIDN/$1" >/dev/null 2>&1 || return 0; sleep 0.5; done
  echo "$1 is still loaded after 15s" >&2
  return 1
}

if [ ! -d "$DIR/.git" ]; then
  git clone --depth 1 --single-branch --branch "$BRANCH" https://github.com/makscee/arena-of-ideas.git "$DIR"
fi
cd "$DIR"
git fetch --depth 1 origin "$BRANCH"
git checkout -q -B holding FETCH_HEAD
BUILD=$(git rev-parse --short=8 HEAD)
if [ ! -f node_modules/.holding-lock ] || ! cmp -s package-lock.json node_modules/.holding-lock; then
  npm ci --no-audit --no-fund --loglevel=error
  cp package-lock.json node_modules/.holding-lock
fi
"$NODE" node_modules/vite/bin/vite.js build --config mobile/vite.config.ts --logLevel warn

mkdir -p "$DIR/data" "$AGENTS"
cat > "$PLIST.new" <<PL
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>$LABEL</string>
  <key>WorkingDirectory</key><string>$DIR</string>
  <key>ProgramArguments</key><array>
    <string>$NODE</string><string>holding/server.mjs</string>
  </array>
  <key>EnvironmentVariables</key><dict>
    <key>PORT</key><string>$PORT</string>
    <key>HOST</key><string>127.0.0.1</string>
    <key>BASE_PATH</key><string>/arena</string>
    <key>MVP_BUILD</key><string>$BUILD (holding)</string>
  </dict>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>$DIR/data/server.log</string>
  <key>StandardErrorPath</key><string>$DIR/data/server.log</string>
</dict></plist>
PL
mv "$PLIST.new" "$PLIST"

# The live game stops, its data is copied (the server is down, so the files are whole), and its plist moves aside.
if launchctl print "gui/$UIDN/$LIVE_LABEL" >/dev/null 2>&1; then
  launchctl bootout "gui/$UIDN/$LIVE_LABEL" || true
  gone "$LIVE_LABEL"
fi
if [ -f "$LIVE_PLIST" ]; then
  mkdir -p "$LIVE_DIR/data"
  mv "$LIVE_PLIST" "$ASIDE"
fi
if [ -f "$LIVE_DIR/data/arena-mvp.db" ]; then
  B="$LIVE_DIR/data/dark-backup-$(date +%Y%m%d-%H%M%S)"
  mkdir -p "$B"
  cp -p "$LIVE_DIR"/data/arena-mvp.db* "$B"/
  echo "live database copied to $B"
fi

launchctl bootout "gui/$UIDN/$LABEL" 2>/dev/null || true
gone "$LABEL"
launchctl bootstrap "gui/$UIDN" "$PLIST"
"$TS" serve --bg --set-path /arena "http://127.0.0.1:$PORT" >/dev/null

HEALTH=
for _ in $(seq 1 30); do
  HEALTH=$(curl -fsS --max-time 2 "http://127.0.0.1:$PORT/arena/api/v1/health" 2>/dev/null) && break
  HEALTH=
  sleep 1
done
if ! printf '%s' "$HEALTH" | grep -q '"dark":true'; then
  echo "the holding server didn't answer dark within 30s; putting the live game back" >&2
  tail -n 20 "$DIR/data/server.log" >&2 2>/dev/null || true
  launchctl bootout "gui/$UIDN/$LABEL" 2>/dev/null || true
  gone "$LABEL" || true
  [ -f "$ASIDE" ] && cp "$ASIDE" "$LIVE_PLIST" && launchctl bootstrap "gui/$UIDN" "$LIVE_PLIST"
  exit 1
fi
echo "dark: $HEALTH"
PUB=$(curl -sS --max-time 15 "$PUBLIC" 2>/dev/null || true)
if printf '%s' "$PUB" | grep -q '"dark":true'; then
  echo "public: $PUBLIC says dark"
else
  echo "public: $PUBLIC did not say dark (answer: ${PUB:0:120}); check mcow's Caddy and tailscale serve" >&2
  exit 1
fi
