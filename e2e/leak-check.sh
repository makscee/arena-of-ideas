#!/usr/bin/env bash
# Must-fail-first proof for the e2e harness self-cleanup (orphan reaping).
#
# Starts `node e2e/run.mjs` on a DISTINCT test port (so a concurrent real run in
# another worktree/tab is never touched), waits until its orchestrated children
# (arena server + vite, and a probe's chromium if one is up) are alive, sends a
# signal to the ORCHESTRATOR, waits, then counts surviving descendants matched
# by the test ports + spawned command. Zero survivors = pass.
#
# Usage:  e2e/leak-check.sh <SIGNAL>     e.g.  e2e/leak-check.sh INT
#         (SIGNAL is the name passed to kill -<SIGNAL>: INT, TERM, ...)
#         e2e/leak-check.sh fault:throw   force an uncaughtException in run.mjs
#         e2e/leak-check.sh fault:reject  force an unhandledRejection in run.mjs
#
# In fault mode the orchestrator crashes ITSELF (no external signal) — the proof
# that teardown fires on the crash paths too. Run against CURRENT (leaks) and
# FIXED (clean).

set -u
SIG="${1:-INT}"
FAULT=""
case "$SIG" in
  fault:*) FAULT="${SIG#fault:}"; SIG="" ;;
esac
PORT=5390
SERVER_PORT=5395
HERE="$(cd "$(dirname "$0")/.." && pwd)"

echo "== leak-check (signal=$SIG, port=$PORT/$SERVER_PORT) =="

# Match any process whose command line mentions either test port or the spawned
# server/vite entrypoints. Excludes this script and grep itself.
descendants() {
  pgrep -f "($PORT|$SERVER_PORT|server/src/main.ts|vite --port $PORT)" \
    | while read -r pid; do
        [ "$pid" = "$$" ] && continue
        cmd=$(ps -o command= -p "$pid" 2>/dev/null) || continue
        case "$cmd" in
          *leak-check*) ;;                              # skip this script
          *"$PORT"*|*"$SERVER_PORT"*|*server/src/main.ts*)
            echo "$pid $cmd" ;;
        esac
      done
}

# Launch the orchestrator detached in its OWN session so our kill targets only it.
LOGTAG="${SIG:-$FAULT}"
AOI_E2E_PORT=$PORT AOI_E2E_SERVER_PORT=$SERVER_PORT AOI_E2E_FAULT="$FAULT" \
  node "$HERE/e2e/run.mjs" >/tmp/leak-run.$LOGTAG.log 2>&1 &
ORCH=$!
echo "orchestrator pid=$ORCH"

# Wait until vite is serving on the test port (children are then up; a probe's
# chromium typically appears shortly after). Cap at 60s.
for i in $(seq 1 120); do
  if curl -s "http://localhost:$PORT" >/dev/null 2>&1; then break; fi
  if ! kill -0 "$ORCH" 2>/dev/null; then
    # In fault mode an early self-exit IS the path under test — fall through to
    # the survivor count. In signal mode it means the harness failed to boot.
    [ -n "$FAULT" ] && break
    echo "orchestrator exited early; log:"; cat /tmp/leak-run.$LOGTAG.log; exit 3
  fi
  sleep 0.5
done

echo "-- children alive before fault/signal --"
descendants

if [ -n "$FAULT" ]; then
  echo "-- fault=$FAULT injected; orchestrator crashes itself --"
else
  echo "-- sending SIG$SIG to orchestrator $ORCH --"
  kill -"$SIG" "$ORCH" 2>/dev/null
fi
# Give the orchestrator time to handle the signal/crash (or to die uncleanly).
wait "$ORCH" 2>/dev/null
sleep 3

# The fault path can race: children may still be on their way up when the curl
# loop saw the early exit. Give the descendant matcher a moment to be sure we
# are not under-counting a slow-to-appear orphan.
[ -n "$FAULT" ] && sleep 1

echo "-- survivors after teardown --"
SURV=$(descendants)
if [ -n "$SURV" ]; then echo "$SURV"; fi
COUNT=$(printf "%s\n" "$SURV" | grep -c . )
echo "SURVIVOR_COUNT=$COUNT"

if [ "$COUNT" -eq 0 ]; then
  echo "RESULT: PASS (zero orphans)"
  exit 0
else
  echo "RESULT: FAIL ($COUNT orphan(s) leaked)"
  # Leave nothing behind regardless of pass/fail: reap the orphans so reruns are clean.
  echo "$SURV" | awk '{print $1}' | while read -r p; do kill -9 "$p" 2>/dev/null; done
  exit 1
fi
