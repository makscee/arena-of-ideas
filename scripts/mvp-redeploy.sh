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
# Slice 10's fusion namer: a small local model behind an OpenAI-compatible
# endpoint (mlx_lm.server), its own launchd agent so a redeploy doesn't reload
# it. The server falls back to a portmanteau whenever it is down.
NAMER_PORT="${ARENA_NAMER_PORT:-8792}"
NAMER_MODEL="${ARENA_NAMER_MODEL:-mlx-community/Qwen2.5-1.5B-Instruct-4bit}"
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
launchctl bootstrap "gui/\$(id -u)" "\$PLIST"
TS=/Applications/Tailscale.app/Contents/MacOS/Tailscale
\$TS serve --bg --set-path /arena "http://127.0.0.1:$PORT" >/dev/null
for i in \$(seq 1 30); do
  curl -fsS --max-time 2 "http://127.0.0.1:$PORT/arena/api/v1/health" >/dev/null 2>&1 && break
  sleep 1
done
curl -fsS --max-time 5 "http://127.0.0.1:$NAMER_PORT/v1/models" >/dev/null 2>&1 && echo "namer up on :$NAMER_PORT" || echo "namer not up yet on :$NAMER_PORT (fusions use the portmanteau until it is)"
echo "deployed \$(git rev-parse --short HEAD) ($BRANCH): \$(curl -fsS --max-time 5 http://127.0.0.1:$PORT/arena/api/v1/health)"
SCRIPT
}

if [ "$(hostname -s 2>/dev/null)" = "m1" ] || [ "$(hostname 2>/dev/null)" = "m1.twin-pogona.ts.net" ]; then
  remote | bash
else
  remote | ssh -o ConnectTimeout=10 "$HOST_ALIAS" bash
fi
echo "open https://m1.twin-pogona.ts.net/arena/"
