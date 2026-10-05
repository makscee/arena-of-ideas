#!/usr/bin/env bash
# Redeploy the Arena MVP test instance on m1 (mission #574): tailnet only, at
# https://m1.twin-pogona.ts.net/arena/ → 127.0.0.1:$PORT. It holds no real
# players' data (a SQLite file, ~/arena-mvp/data/arena-mvp.db, kept across
# redeploys), so any agent may redeploy it.
#
#   npm run mvp:redeploy                 # the mission branch
#   npm run mvp:redeploy -- <branch>     # any pushed branch
#
# Runs from anywhere with `ssh m1`; on m1 itself it runs locally. The server
# is a launchd agent (ru.makscee.arena-mvp), so it restarts on crash and login.
set -euo pipefail
BRANCH="${1:-mission-574-mvp}"
PORT="${ARENA_MVP_PORT:-8791}"
HOST_ALIAS="${ARENA_MVP_HOST:-m1}"

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
npm ci --no-audit --no-fund --loglevel=error
npm run -s mvp:build
mkdir -p "\$HOME/Library/LaunchAgents" "\$DIR/data"
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
launchctl bootstrap "gui/\$(id -u)" "\$PLIST"
TS=/Applications/Tailscale.app/Contents/MacOS/Tailscale
\$TS serve --bg --set-path /arena "http://127.0.0.1:$PORT" >/dev/null
for i in \$(seq 1 30); do
  curl -fsS --max-time 2 "http://127.0.0.1:$PORT/arena/api/v1/health" >/dev/null 2>&1 && break
  sleep 1
done
echo "deployed \$(git rev-parse --short HEAD) ($BRANCH): \$(curl -fsS --max-time 5 http://127.0.0.1:$PORT/arena/api/v1/health)"
SCRIPT
}

if [ "$(hostname -s 2>/dev/null)" = "m1" ] || [ "$(hostname 2>/dev/null)" = "m1.twin-pogona.ts.net" ]; then
  remote | bash
else
  remote | ssh -o ConnectTimeout=10 "$HOST_ALIAS" bash
fi
echo "open https://m1.twin-pogona.ts.net/arena/"
