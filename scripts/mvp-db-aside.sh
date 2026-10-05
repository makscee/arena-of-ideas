#!/usr/bin/env bash
# Moves the Arena MVP SQLite DB (and its -wal and -shm files) aside to a
# timestamped backup next to it, so the server starts on an empty world (the
# bots reseed the champion). Stop the server first: scripts/mvp-redeploy.sh
# --fresh calls this between stopping and starting it (mission #574).
#
#   scripts/mvp-db-aside.sh data/arena-mvp.db
set -euo pipefail
DB="${1:?usage: mvp-db-aside.sh <path to the SQLite DB>}"
if [ ! -e "$DB" ]; then
  echo "no DB at $DB: nothing to move, the server starts fresh"
  exit 0
fi
BAK="$DB.bak-$(date -u +%Y%m%dT%H%M%SZ)"
for suffix in "" -wal -shm; do
  [ -e "$DB$suffix" ] && mv "$DB$suffix" "$BAK$suffix"
done
echo "moved $DB aside to $BAK"
