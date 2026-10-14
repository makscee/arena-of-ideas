#!/bin/bash
# The night batch in the workshop (#1050, mission 1043): every night at 03:30
# the job designs fusions for tonight's pairs and stores them in the
# workshop's Codex as ready to claim (scripts/night-batch.ts). Run on m1.
#
#   scripts/arena-night.sh install     write and load launchd ru.makscee.arena-night (03:30 daily)
#   scripts/arena-night.sh run [args]  one run now, the way the job runs it (extra args go to mvp:night,
#                                      e.g. --pairs 2); the log is data/night/night.log
#   scripts/arena-night.sh uninstall   unload and remove the job
#
# It runs in the workshop clone (~/arena-workshop, whatever commit the
# workshop is at) against the workshop's database, with Opus and Sonnet
# through `claude -p` (ARENA_NIGHT_DESIGNER=claude, the workers' login on m1),
# and writes the report into ~/reports/arena-night-<day>.html. The live game
# stays dark: nothing runs there.
set -euo pipefail
# The workshop server runs /opt/homebrew/bin/node (arena-workshop.sh), and better-sqlite3 is built for it:
# that node must come first, ahead of ~/.local/bin (another node, and claude).
export PATH="/opt/homebrew/bin:$HOME/.local/bin:/usr/local/bin:$PATH"
DIR="${ARENA_WORKSHOP_DIR:-$HOME/arena-workshop}"
LABEL=ru.makscee.arena-night
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
DB="$DIR/data/workshop.db"
REPORTS="${ARENA_REPORTS_DIR:-$HOME/reports}"
NODE=$(command -v node)

case "${1:-}" in
run)
  shift
  cd "$DIR"
  mkdir -p data/night "$REPORTS"
  DAY=$(date +%F)
  echo "[$(date '+%F %T')] night batch $DAY starts" >> data/night/night.log
  env -u CLAUDECODE ARENA_NIGHT_DESIGNER="${ARENA_NIGHT_DESIGNER:-claude}" ARENA_CLAUDE_BIN="${ARENA_CLAUDE_BIN:-$(command -v claude)}" ARENA_CODEX=1 MVP_DB="$DB" \
    "$NODE" --import tsx/esm scripts/night-batch.ts --day "$DAY" --out data/night --report "$REPORTS/arena-night-$DAY.html" --parallel "${ARENA_NIGHT_PARALLEL:-3}" "$@" >> data/night/night.log 2>&1
  echo "[$(date '+%F %T')] night batch $DAY done: $REPORTS/arena-night-$DAY.html" >> data/night/night.log
  ;;
install)
  mkdir -p "$(dirname "$PLIST")" "$DIR/data/night"
  cat > "$PLIST" <<PL
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>$LABEL</string>
  <key>WorkingDirectory</key><string>$DIR</string>
  <key>ProgramArguments</key><array>
    <string>/bin/bash</string><string>$DIR/scripts/arena-night.sh</string><string>run</string>
  </array>
  <key>EnvironmentVariables</key><dict>
    <key>PATH</key><string>/opt/homebrew/bin:$HOME/.local/bin:/usr/local/bin:/usr/bin:/bin</string>
    <key>HOME</key><string>$HOME</string>
  </dict>
  <key>StartCalendarInterval</key><dict><key>Hour</key><integer>3</integer><key>Minute</key><integer>30</integer></dict>
  <key>RunAtLoad</key><false/>
  <key>StandardOutPath</key><string>$DIR/data/night/launchd.log</string>
  <key>StandardErrorPath</key><string>$DIR/data/night/launchd.log</string>
</dict></plist>
PL
  UIDN=$(id -u)
  launchctl bootout "gui/$UIDN/$LABEL" 2>/dev/null || true
  launchctl bootstrap "gui/$UIDN" "$PLIST"
  echo "installed $LABEL: 03:30 daily in $DIR against $DB, reports in $REPORTS"
  launchctl print "gui/$UIDN/$LABEL" | grep -E "state|calendar|runs" | head -5 || true
  ;;
uninstall)
  launchctl bootout "gui/$(id -u)/$LABEL" 2>/dev/null || true
  rm -f "$PLIST"
  echo "removed $LABEL"
  ;;
*)
  echo "usage: $0 install | run [mvp:night args] | uninstall" >&2
  exit 2
  ;;
esac
