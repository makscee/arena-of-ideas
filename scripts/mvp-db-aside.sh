#!/usr/bin/env bash
# Moves the Arena MVP SQLite DB (and its -wal and -shm files) aside to a
# timestamped backup next to it, so the server starts on an empty world (the
# bots reseed the champion). Stop the server first: scripts/mvp-redeploy.sh
# --fresh calls this between stopping and starting it (mission #574), and
# with --check before it stops anything.
#
#   scripts/mvp-db-aside.sh data/arena-mvp.db          # move it aside
#   scripts/mvp-db-aside.sh --check data/arena-mvp.db  # only check it could
#
# A move that fails halfway puts back what it moved and exits non-zero: the
# DB is either all aside or all in place.
set -euo pipefail
CHECK=0
if [ "${1:-}" = "--check" ]; then CHECK=1; shift; fi
DB="${1:?usage: mvp-db-aside.sh [--check] <path to the SQLite DB>}"
DIR="$(dirname "$DB")"
[ -d "$DIR" ] || { echo "no directory $DIR for the DB $DB" >&2; exit 1; }
[ -w "$DIR" ] || { echo "can't write $DIR: the DB can't move aside" >&2; exit 1; }
if [ -e "$DB" ] && [ ! -f "$DB" ]; then echo "$DB is not a file" >&2; exit 1; fi
if [ "$CHECK" = 1 ]; then
  echo "ok: $DB can move aside"
  exit 0
fi
if [ ! -e "$DB" ]; then
  echo "no DB at $DB: nothing to move, the server starts fresh"
  exit 0
fi
BAK="$DB.bak-$(date -u +%Y%m%dT%H%M%SZ)"
undo() {
  for s in "" -wal -shm; do
    if [ -e "$BAK$s" ]; then mv "$BAK$s" "$DB$s" || true; fi
  done
}
for suffix in "" -wal -shm; do
  [ -e "$DB$suffix" ] || continue
  if ! mv "$DB$suffix" "$BAK$suffix"; then
    undo
    echo "couldn't move $DB$suffix aside: the DB stays where it was" >&2
    exit 1
  fi
done
echo "moved $DB aside to $BAK"
