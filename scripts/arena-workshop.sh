#!/bin/bash
# The Arena workshop (#1044, mission 1043): a persistent invite-only world on
# the tailnet where Maks plays each unit batch of rules v2, at
# https://m1.twin-pogona.ts.net/arena-workshop/ . Run on m1.
#
#   scripts/arena-workshop.sh <commit-or-branch>   point the workshop at it
#                                                   (the orchestrator runs this after each checked merge)
#   scripts/arena-workshop.sh <ref> --fresh        also start a new world: the old database moves
#                                                   aside to data/workshop.db.bak-<time>
#   scripts/arena-workshop.sh invite <name>        make <name>'s invite and print its link; --admin lets
#                                                   them use the dev tools
#   scripts/arena-workshop.sh list                 every invite with its link (the way back to a link made earlier)
#   npm run mvp:workshop -- <ref>                  the same, from a checkout
#
# It runs its own clone ~/arena-workshop (npm ci when the lock changed, the
# client built for /arena-workshop/) under launchd ru.makscee.arena-workshop on
# :8794, with MVP_DEV=1 (End day now, fake votes), MVP_INVITES=1 and MVP_OPEN=0
# (invite links only), ARENA_TUNER=instant and a fake idea reader, on
# ~/arena-workshop/data/workshop.db, with the Codex on (ARENA_CODEX=1: the
# forge, wishes, the night batch's candidates; scripts/arena-night.sh runs
# the batch nightly). The world persists across commits (the
# server migrates it at start); --fresh is for a commit whose migration can't.
# The world plays the rules v2 pool (MVP_POOL=v2, src/mvp/units-v2.ts; set
# MVP_POOL=grid for the old name grammar's). A world seeded on one pool keeps
# it: use --fresh to start on the other.
# Since #1219 it also serves the CrazyGames playtest frame's players: ARENA_CG=1
# (guest and CrazyGames joins, 100 new players an hour per address), logins
# checked with a test key (ARENA_CG_PUBLIC_KEY, ~/arena-workshop/cg-test) and
# game id "playtest", CORS for the frame's game origin; once it is up,
# scripts/arena-cg-playtest.sh publishes the frame at
# https://m1.twin-pogona.ts.net/r/arena-cg/ . Refs from before the cg code skip all of it.
# Never the live game's database, never the public domain: tailnet only.
set -euo pipefail
export PATH=/opt/homebrew/bin:/usr/local/bin:$PATH
PORT="${ARENA_WORKSHOP_PORT:-8794}"
DIR="${ARENA_WORKSHOP_DIR:-$HOME/arena-workshop}"
LABEL=ru.makscee.arena-workshop
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
TS=/Applications/Tailscale.app/Contents/MacOS/Tailscale
PUBLIC_URL=https://m1.twin-pogona.ts.net/arena-workshop/
REPO=https://github.com/makscee/arena-of-ideas.git
NODE=$(command -v node)
DB="$DIR/data/workshop.db"

if [ "${1:-}" = invite ]; then
  [ -n "${2:-}" ] || { echo "usage: $0 invite <name> [--admin]" >&2; exit 2; }
  cd "$DIR"
  shift
  MVP_DB="$DB" MVP_PUBLIC_URL="$PUBLIC_URL" "$NODE" --import tsx/esm server/src/mvp/invite-cli.ts add "$@"
  exit 0
fi

if [ "${1:-}" = list ]; then # every invite with its link (a name that has one is refused by `invite`)
  cd "$DIR"
  MVP_DB="$DB" MVP_PUBLIC_URL="$PUBLIC_URL" "$NODE" --import tsx/esm server/src/mvp/invite-cli.ts list
  exit 0
fi

REF="${1:?usage: $0 <commit-or-branch> [--fresh] | invite <name> [--admin] | list}"
FRESH=0
[ "${2:-}" = --fresh ] && FRESH=1

if [ ! -d "$DIR/.git" ]; then
  git init -q "$DIR"
  git -C "$DIR" remote add origin "$REPO"
fi
cd "$DIR"
git fetch --depth 1 origin "$REF"
git checkout -q -B workshop FETCH_HEAD
BUILD=$(git rev-parse --short=8 HEAD)
if [ ! -f node_modules/.workshop-lock ] || ! cmp -s package-lock.json node_modules/.workshop-lock; then
  npm ci --no-audit --no-fund --loglevel=error
  cp package-lock.json node_modules/.workshop-lock
fi
"$NODE" node_modules/vite/bin/vite.js build --config mobile/vite.config.ts --base /arena-workshop/ --logLevel warn

CG_ENV=
if [ -f server/src/mvp/crazygames.ts ] && [ -f scripts/arena-cg-playtest.sh ]; then
  export ARENA_CG_KEYDIR="$DIR/cg-test"
  bash scripts/arena-cg-playtest.sh keys
  CG_ENV="<key>ARENA_CG</key><string>1</string>
    <key>ARENA_CG_GAME_ID</key><string>playtest</string>
    <key>ARENA_CG_PUBLIC_KEY</key><string>$ARENA_CG_KEYDIR/public.pem</string>
    <key>MVP_CORS_ORIGINS</key><string>https://m1.twin-pogona.ts.net:8444</string>
    <key>ARENA_GUEST_JOINS_PER_IP</key><string>100</string>"
fi

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
    <key>BASE_PATH</key><string>/arena-workshop</string>
    <key>MVP_DEV</key><string>1</string>
    <key>MVP_INVITES</key><string>1</string>
    <key>MVP_OPEN</key><string>0</string>
    <key>MVP_ROTATION</key><string>0</string>
    <key>ARENA_TUNER</key><string>instant</string>
    <key>ARENA_IDEA_READER</key><string>fake</string>
    <key>ARENA_CODEX</key><string>1</string>
    <key>MVP_DB</key><string>$DB</string>
    <key>MVP_BUILD</key><string>$BUILD (workshop)</string>
    <key>MVP_POOL</key><string>${MVP_POOL:-v2}</string>
    $CG_ENV
  </dict>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>$DIR/data/server.log</string>
  <key>StandardErrorPath</key><string>$DIR/data/server.log</string>
</dict></plist>
PL
UIDN=$(id -u)
launchctl bootout "gui/$UIDN/$LABEL" 2>/dev/null || true
for _ in $(seq 1 30); do launchctl print "gui/$UIDN/$LABEL" >/dev/null 2>&1 || break; sleep 1; done
if [ "$FRESH" = 1 ] && [ -f "$DB" ]; then
  T=$(date +%Y%m%d-%H%M%S)
  for f in "$DB" "$DB-wal" "$DB-shm"; do [ -f "$f" ] && mv "$f" "${f/workshop.db/workshop.db.bak-$T}" || true; done
  echo "old world moved aside to workshop.db.bak-$T"
fi
launchctl bootstrap "gui/$UIDN" "$PLIST"
"$TS" serve --bg --set-path /arena-workshop "http://127.0.0.1:$PORT" >/dev/null

# Seeding the day's champion takes a while under load.
HEALTH=
for _ in $(seq 1 180); do
  HEALTH=$(curl -fsS --max-time 2 "http://127.0.0.1:$PORT/arena-workshop/api/v1/health" 2>/dev/null) && break
  HEALTH=
  sleep 1
done
if ! printf '%s' "$HEALTH" | grep -q '"invites":true'; then
  echo "the workshop didn't answer invite-only within 3 min (answer: ${HEALTH:-none}): the end of $DIR/data/server.log:" >&2
  tail -n 20 "$DIR/data/server.log" >&2 2>/dev/null || true
  exit 1
fi
echo "workshop: $REF at $BUILD on $PUBLIC_URL : $HEALTH"
echo "invite link: $0 invite <name> --admin"
if [ -n "$CG_ENV" ]; then
  bash scripts/arena-cg-playtest.sh || echo "the playtest frame wasn't published (see above); the workshop itself is up" >&2
fi
