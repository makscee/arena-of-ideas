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
# invites. Testers play at https://arena.makscee.ru/arena/: mcow's Caddy
# proxies /arena* there to this host's tailscale serve, so whatever this
# script serves is public once that switch is made. m1 itself stays
# tailnet-only.
#
# The redeploy fails (exit 1, no "deployed" line) unless the new server's
# /health says invites: true, so a pre-slice-13 ref or a broken setting is
# never served by accident: it then stops the new server and turns the /arena
# serve off before exiting. A new server that never answers /health is
# stopped the same way, since its invites were never checked.
# ARENA_MVP_INVITES takes 0 or 1 only. 0 deploys an OPEN server (names, no
# links: anyone who reaches it plays as anyone); it warns, and refuses unless
# https://arena.makscee.ru/arena/ clearly isn't this host: its /health shows
# another build, the June page (its title screen, not any HTML), or a 404
# while this host's own /arena answers a definite non-404 code. This host's
# build, a 5xx, no answer or a cut-off one, Arena JSON while this host's
# server is down, and a 404 while this host can't read its own /arena or gets
# a 404 there too all refuse, with the reason; ARENA_MVP_OPEN_PUBLIC=1
# overrides on purpose.
#
# Rollback to a pre-slice-13 build, or back up after a failed check: SWITCH
# THE DOMAIN OFF FIRST (homelab Caddyfile.j2, arena.makscee.ru stops proxying
# /arena* to m1), then
#   ARENA_MVP_INVITES=0 npm run mvp:redeploy -- <old ref>
#
# Run it from a checkout at the commit being deployed: the script builds the
# remote commands here, so a stale clone deploys new code with old checks.
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
case "$INVITES" in 0|1) ;; *) echo "ARENA_MVP_INVITES is 0 or 1, not '$INVITES'" >&2; exit 2 ;; esac
OPEN_PUBLIC="${ARENA_MVP_OPEN_PUBLIC:-0}"
# Open to all (Maks, 2026-10-08): on the invite-only server, anyone who comes
# without a link gets the join name screen (MVP_OPEN, R4-20's join code and
# limit). ARENA_MVP_OPEN=0 deploys it invite-only again.
OPEN="${ARENA_MVP_OPEN:-1}"
case "$OPEN" in 0|1) ;; *) echo "ARENA_MVP_OPEN is 0 or 1, not '$OPEN'" >&2; exit 2 ;; esac
# Who reads players' ideas (mission 2, M2-5; Maks: Claude, through Claude Code
# on m1): claude by default here, ARENA_MVP_IDEA_READER=fake turns it off.
IDEA_READER="${ARENA_MVP_IDEA_READER:-claude}"
case "$IDEA_READER" in claude|fake) ;; *) echo "ARENA_MVP_IDEA_READER is claude or fake, not '$IDEA_READER'" >&2; exit 2 ;; esac
# The pool's daily rotation at 04:00 (mission 2, M2-10): on, after a dry run on a
# copy of the live DB (npm run mvp:rotate -- --db <copy> --dry-run).
# ARENA_MVP_ROTATION=0 turns it off.
ROTATION="${ARENA_MVP_ROTATION:-1}"
case "$ROTATION" in 0|1) ;; *) echo "ARENA_MVP_ROTATION is 0 or 1, not '$ROTATION'" >&2; exit 2 ;; esac
PUBLIC_URL=https://arena.makscee.ru/arena/
TAILNET_URL=https://m1.twin-pogona.ts.net/arena/api/v1/health

remote() {
  cat <<SCRIPT
set -euo pipefail
export PATH=/opt/homebrew/bin:/usr/local/bin:\$PATH
DIR="\$HOME/arena-mvp"
LABEL=ru.makscee.arena-mvp
TS=/Applications/Tailscale.app/Contents/MacOS/Tailscale
if [ "$INVITES" != 1 ]; then
  # An open server: only while the public domain clearly isn't this host.
  # Checked before anything stops, and fail-closed: anything the check can't
  # read refuses, with the reason (ARENA_MVP_OPEN_PUBLIC=1 overrides).
  #   allowed: the public /health is JSON with another build than this host
  #     serves; the June page (its title screen markers, not any HTML); a 404
  #     while this host's own /arena answers a definite non-404 HTTP code (a
  #     proxied request would get that, not a 404)
  #   refused: this host's build; Arena JSON while this host's build is
  #     unknown (its server is down); a 5xx, no answer or a cut-off answer
  #     (curl failed: this host down behind mcow's proxy looks like that); a
  #     404 while this host's /arena is a 404 too (its serve is off) or can't
  #     be read (000); anything else
  MINE=
  if LOCAL=\$(curl -fsS --max-time 5 "http://127.0.0.1:$PORT/arena/api/v1/health" 2>/dev/null); then
    MINE=\$(printf '%s' "\$LOCAL" | sed -n 's/.*"build":"\([^"]*\)".*/\1/p')
  fi
  # A failed curl (no answer, a body cut off mid-transfer) is no answer: 000.
  PUB=\$(curl -sS --max-time 10 -w '\n%{http_code}' "${PUBLIC_URL}api/v1/health" 2>/dev/null) || PUB=000
  CODE=\$(printf '%s\n' "\$PUB" | tail -n 1)
  BODY=\$(printf '%s\n' "\$PUB" | sed '\$d')
  WHY=
  case "\$CODE" in
    200)
      # The June page: the old web client's title screen (web/index.html),
      # which the mobile client this host serves doesn't have.
      if printf '%s' "\$BODY" | grep -q '<section id="title-view">' && printf '%s' "\$BODY" | grep -q '<h1 class="title-name">Arena of Ideas</h1>'; then
        :
      elif printf '%s' "\$BODY" | grep -q '"build":"'; then
        if [ -z "\$MINE" ]; then
          WHY="$PUBLIC_URL answers with an Arena server's /health, and this host's server doesn't answer http://127.0.0.1:$PORT/arena/api/v1/health, so it can't tell whether the domain serves this host"
        elif printf '%s' "\$BODY" | grep -q "\"build\":\"\$MINE\""; then
          WHY="$PUBLIC_URL serves this host (build \$MINE)"
        fi
      else
        WHY="$PUBLIC_URL answers 200 with neither the June page nor an Arena /health, so it can't tell whether the domain serves this host"
      fi ;;
    404)
      # Only a definite non-404 HTTP code here tells a proxied request apart.
      OWN=\$(curl -sS --max-time 10 -o /dev/null -w '%{http_code}' "$TAILNET_URL" 2>/dev/null) || OWN=000
      case "\$OWN" in
        404) WHY="$PUBLIC_URL answers 404, and so does this host's own $TAILNET_URL (its /arena serve is off), so a 404 can't tell whether the domain serves this host" ;;
        [1-5][0-9][0-9]) : ;;
        *) WHY="$PUBLIC_URL answers 404, and this host can't read its own $TAILNET_URL within 10s, so a 404 can't tell whether the domain serves this host" ;;
      esac ;;
    5??) WHY="$PUBLIC_URL answers \$CODE: that is what this host down behind mcow's proxy looks like" ;;
    ''|000) WHY="$PUBLIC_URL didn't answer within 10s, or its answer was cut off: that can be this host down behind mcow's proxy" ;;
    *) WHY="$PUBLIC_URL answers \$CODE, so it can't tell whether the domain serves this host" ;;
  esac
  if [ -n "\$WHY" ]; then
    if [ "$OPEN_PUBLIC" != 1 ]; then
      echo "ARENA_MVP_INVITES=0 refused: \$WHY. An open server there would let anyone on the internet play as anyone. Switch the domain off first (homelab Caddyfile.j2, arena.makscee.ru stops proxying /arena* to m1), or ARENA_MVP_OPEN_PUBLIC=1 on purpose. Nothing stopped." >&2
      exit 1
    fi
    echo "ARENA_MVP_OPEN_PUBLIC=1: deploying open although \$WHY." >&2
  fi
  echo "WARNING: ARENA_MVP_INVITES=0 deploys an OPEN server (names, no links: anyone who reaches it plays as anyone). Keep $PUBLIC_URL off this host while it runs." >&2
fi
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
    <key>MVP_OPEN</key><string>$OPEN</string>
    <key>MVP_DB</key><string>\$DB</string>
    <key>MVP_BUILD</key><string>\$BUILD</string>
    <key>ARENA_NAMER_URL</key><string>http://127.0.0.1:$NAMER_PORT/v1/chat/completions</string>
    <key>MVP_ROTATION</key><string>$ROTATION</string>
    <key>ARENA_IDEA_READER</key><string>$IDEA_READER</string>
    <key>ARENA_CLAUDE_BIN</key><string>\$HOME/.local/bin/claude</string>
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
# Stop the job (KeepAlive would restart it) and turn the /arena serve off,
# so nothing serves unchecked.
stop_unchecked() {
  launchctl bootout "gui/\$(id -u)/\$LABEL" 2>/dev/null || true
  for i in \$(seq 1 20); do launchctl print "gui/\$(id -u)/\$LABEL" >/dev/null 2>&1 || break; sleep 0.5; done
  \$TS serve --set-path /arena off >/dev/null 2>&1 || true
}
still_loaded() {
  if launchctl print "gui/\$(id -u)/\$LABEL" >/dev/null 2>&1; then
    echo "\$LABEL is STILL LOADED: stop it by hand (launchctl bootout gui/\$(id -u)/\$LABEL)." >&2
  fi
}
ROLLBACK="Redeploy an invite-only build, or roll back: switch the domain off first (homelab Caddyfile.j2, arena.makscee.ru stops proxying /arena* to m1), then ARENA_MVP_INVITES=0 npm run mvp:redeploy -- <ref>."
launchctl bootstrap "gui/\$(id -u)" "\$PLIST"
if ! TSOUT=\$(\$TS serve --bg --set-path /arena "http://127.0.0.1:$PORT" 2>&1); then
  # The old /arena serve may still point at this port: stop the new job
  # rather than leave it serving unchecked.
  stop_unchecked
  echo "redeploy failed: tailscale serve --set-path /arena didn't take (\$TSOUT), so the new server (\$BUILD, $BRANCH) is stopped and the /arena serve is off: the Arena is down. Fix tailscale serve on this host and redeploy. \$ROLLBACK" >&2
  still_loaded
  exit 1
fi
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
  if [ "$INVITES" = 1 ]; then
    # Invites were never checked: if it came up later it would serve
    # unchecked, so stop it.
    stop_unchecked
    echo "Its invites were never checked, so it is stopped and the /arena serve is off: the Arena is down. \$ROLLBACK" >&2
    still_loaded
  else
    echo "Fix it and redeploy (npm run mvp:redeploy -- <branch>), or go back to a branch that worked." >&2
  fi
  exit 1
fi
# Invite-only unless ARENA_MVP_INVITES=0 said otherwise: a server that
# doesn't say invites: true (a pre-slice-13 ref, a broken setting) is open.
if [ "$INVITES" = 1 ] && ! printf '%s' "\$HEALTH" | grep -q '"invites":true'; then
  stop_unchecked
  echo "redeploy failed: the new server (\$BUILD, $BRANCH) is OPEN, not invite-only: its /health says \$HEALTH. Anyone who reaches it would play as anyone, so it is stopped and the /arena serve is off: the Arena is down. \$ROLLBACK" >&2
  still_loaded
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
echo "open https://m1.twin-pogona.ts.net/arena/ (testers: $PUBLIC_URL once the domain points here)"
