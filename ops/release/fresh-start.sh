#!/usr/bin/env bash
# Backup-first fresh-start release of arena.makscee.ru (makscee/void-board#467).
#
# The June database is archived and kept, never migrated: the new image starts on
# a new, empty volume, and rollback is the June image on the untouched June volume.
# Runbook and the order of steps: docs/release-runbook.md.
#
#   MODE=rehearse (default) works on a COPY of the prod volume, in its own
#   container and volumes on mcow; prod's `arena` container is never touched.
#   MODE=prod works on the real thing. Run it only on Maks's go.
#
# usage: ops/release/fresh-start.sh <step> [args]
#   prepare              rehearse only: copy prod's volume, run the June image on it
#   backup               stop the arena, tarball + restic snapshot of the June volume
#   verify               restore both backups to a temp dir, prove they open and match
#   deploy <sha-tag>     start <sha-tag> on the new, empty volume
#   check <sha-tag>      /healthz answers with that build (the June tag: ok, no build)
#   rollback             the June image back on the June volume, data intact
#   cleanup              rehearse only: remove the rehearsal container, volumes and files
set -euo pipefail

MODE="${MODE:-rehearse}"
HOST="${HOST:-root@mcow}"
IMAGE="ghcr.io/makscee/arena-of-ideas"
JUNE_TAG="sha-4a276d3"
PROD_VOL="void_arena_data"
STAMP_FILE=".arena-release-stamp"

case "$MODE" in
  rehearse)
    CTR="arena-rehearsal"; OLD_VOL="arena_rehearsal_june"; NEW_VOL="arena_rehearsal_s1"
    ARCHIVE_DIR="/srv/backups/arena-rehearsal"
    # A throwaway restic repo next to the archive, so the rehearsal leaves nothing
    # in the real repo on nether. prod uses /etc/restic/env, as restic-daily does.
    RESTIC_ENV="export RESTIC_REPOSITORY=$ARCHIVE_DIR/restic RESTIC_PASSWORD_FILE=$ARCHIVE_DIR/restic.pass"
    HEALTH="docker exec $CTR wget -qO- http://127.0.0.1:8787/healthz" ;;
  prod)
    CTR="arena"; OLD_VOL="$PROD_VOL"; NEW_VOL="void_arena_data_s1"
    ARCHIVE_DIR="/srv/backups/arena-june"
    RESTIC_ENV="set -a; . /etc/restic/env; set +a; export HOME=/root"
    HEALTH="curl -fsS https://arena.makscee.ru/healthz" ;;
  *) echo "MODE must be rehearse or prod" >&2; exit 2 ;;
esac

remote() { ssh -o BatchMode=yes "$HOST" "set -euo pipefail; $1"; }
say() { printf '\n== %s\n' "$*"; }
need_tag() { [[ "${1:-}" =~ ^sha-[0-9a-f]{7}$ ]] || { echo "need an image tag like sha-1234abc" >&2; exit 2; }; }
rehearse_only() { [ "$MODE" = rehearse ] || { echo "$1 is a rehearsal step" >&2; exit 2; }; }

# Arena needs MAIL_* at boot. In a rehearsal no mail goes anywhere.
run_arena() { # <tag> <volume>
  remote "docker pull -q $IMAGE:$1 >/dev/null
    docker run -d --name $CTR --restart no \
    -e NODE_ENV=production -e PORT=8787 -e DB_PATH=/data/arena.db \
    -e MAIL_BASE_URL=http://127.0.0.1:9 -e MAIL_TOKEN=rehearsal \
    -v $2:/data $IMAGE:$1 >/dev/null"
}

wait_healthy() {
  for _ in $(seq 1 30); do
    if out=$(eval "$HEALTH_CMD" 2>/dev/null); then echo "$out"; return 0; fi
    sleep 1
  done
  echo "no /healthz answer after 30s" >&2
  remote "docker logs --tail 20 $CTR" >&2 || true
  return 1
}
HEALTH_CMD="ssh -o BatchMode=yes $HOST '$HEALTH'"
[ "$MODE" = prod ] && HEALTH_CMD="$HEALTH"

# Row counts of every table, read on a copy so the volume stays as it was.
counts() { # <volume>
  remote "d=\$(mktemp -d); cp -a \$(docker volume inspect $1 -f '{{.Mountpoint}}')/. \$d/
    for t in \$(sqlite3 \$d/arena.db .tables); do echo \"\$t \$(sqlite3 \$d/arena.db \"select count(*) from \$t\")\"; done | sort
    rm -rf \$d"
}

# prod: the homelab clone must already say what we deploy (no -e override), so
# the next routine playbook run keeps the same image on the same volume.
homelab_says() { # <tag> <volume key>
  : "${HOMELAB:?set HOMELAB to a makscee/homelab clone}"
  local gv="$HOMELAB/ansible/inventory/group_vars/void_platform.yml"
  grep -q '{{ arena_data_volume }}:/data' "$HOMELAB/ansible/playbooks/templates/compose.yml.j2" \
    || { echo "FAIL: $HOMELAB has no arena_data_volume in compose.yml.j2 (homelab #144 not in this clone)" >&2; exit 1; }
  grep -q "^arena_image_tag: \"$1\"" "$gv" && grep -q "^arena_data_volume: \"$2\"" "$gv" \
    || { echo "FAIL: $gv must say arena_image_tag: \"$1\" and arena_data_volume: \"$2\"" >&2; exit 1; }
}

# The arena container must run on exactly this volume.
mounted_on() { # <volume>
  local src
  src=$(remote "docker inspect $CTR -f '{{range .Mounts}}{{if eq .Destination \"/data\"}}{{.Name}}{{end}}{{end}}'")
  [ "$src" = "$1" ] && echo "OK: $CTR mounts $1 at /data" || { echo "FAIL: $CTR mounts '$src' at /data, expected $1" >&2; exit 1; }
}

step="${1:-}"; shift || true
case "$step" in
  prepare)
    rehearse_only prepare
    say "copy prod's $PROD_VOL (read-only) to $OLD_VOL"
    remote "docker volume inspect $OLD_VOL >/dev/null 2>&1 && { echo '$OLD_VOL exists: run cleanup first' >&2; exit 1; }
      docker volume create $OLD_VOL >/dev/null; docker pull -q alpine >/dev/null
      docker run --rm -v $PROD_VOL:/from:ro -v $OLD_VOL:/to alpine sh -c 'cp -a /from/. /to/ && ls -la /to'"
    say "run the June image on the copy, as prod runs it today"
    run_arena "$JUNE_TAG" "$OLD_VOL"
    wait_healthy
    counts "$OLD_VOL" ;;

  backup)
    say "stop $CTR, so the SQLite files (db + wal + shm) are at rest"
    remote "docker stop $CTR >/dev/null && docker ps -a --filter name=^/$CTR\$ --format '{{.Names}} {{.Status}}'"
    say "tarball of $OLD_VOL to $ARCHIVE_DIR (off the volume)"
    remote "mkdir -p $ARCHIVE_DIR && chmod 700 $ARCHIVE_DIR
      ts=\$(date -u +%Y%m%dT%H%M%SZ); tgz=$ARCHIVE_DIR/arena-june-\$ts.tgz
      docker run --rm -v $OLD_VOL:/data:ro alpine tar czf - -C /data . > \$tgz
      (cd $ARCHIVE_DIR && sha256sum \$(basename \$tgz) > \$(basename \$tgz).sha256)
      (cd \$(docker volume inspect $OLD_VOL -f '{{.Mountpoint}}') && find . -type f | sort | xargs sha256sum) > $ARCHIVE_DIR/arena-june-\$ts.files.sha256
      echo \$ts > $ARCHIVE_DIR/$STAMP_FILE; ls -la $ARCHIVE_DIR"
    say "restic snapshot of $OLD_VOL, tagged arena-june-final"
    remote "$RESTIC_ENV
      if [ '$MODE' = rehearse ] && [ ! -f $ARCHIVE_DIR/restic.pass ]; then
        head -c 32 /dev/urandom | base64 > $ARCHIVE_DIR/restic.pass; chmod 600 $ARCHIVE_DIR/restic.pass
        restic init -q; fi
      restic backup -q --tag arena-june-final \$(docker volume inspect $OLD_VOL -f '{{.Mountpoint}}')
      restic snapshots --tag arena-june-final --compact" ;;

  verify)
    say "restore the tarball and the restic snapshot to temp dirs; both must match the volume and open cleanly"
    remote "$RESTIC_ENV
      ts=\$(cat $ARCHIVE_DIR/$STAMP_FILE); tgz=arena-june-\$ts.tgz
      (cd $ARCHIVE_DIR && sha256sum -c \$tgz.sha256)
      vol=\$(docker volume inspect $OLD_VOL -f '{{.Mountpoint}}')
      t=\$(mktemp -d); r=\$(mktemp -d)
      tar xzf $ARCHIVE_DIR/\$tgz -C \$t
      restic restore -q latest --tag arena-june-final --target \$r
      for f in \$(cd \$vol && find . -type f); do
        a=\$(sha256sum < \$vol/\$f); b=\$(sha256sum < \$t/\$f); c=\$(sha256sum < \$r\$vol/\$f)
        [ \"\$a\" = \"\$b\" ] && [ \"\$a\" = \"\$c\" ] && echo \"match  \$f\" || { echo \"MISMATCH \$f\"; exit 1; }
      done
      echo \"tarball integrity: \$(sqlite3 \$t/arena.db 'PRAGMA integrity_check')  users: \$(sqlite3 \$t/arena.db 'select count(*) from users')\"
      echo \"restic  integrity: \$(sqlite3 \$r\$vol/arena.db 'PRAGMA integrity_check')  users: \$(sqlite3 \$r\$vol/arena.db 'select count(*) from users')\"
      rm -rf \$t \$r" ;;

  deploy)
    need_tag "${1:-}"
    [ "$MODE" = prod ] && homelab_says "$1" arena_data_s1
    say "the June volume must be at rest and unchanged since the backup"
    # A playbook run between backup and deploy could have restarted the June arena.
    remote "[ \"\$(docker inspect $CTR -f '{{.State.Running}}' 2>/dev/null)\" != true ] || { echo 'FAIL: $CTR is running again; run backup again' >&2; exit 1; }
      ts=\$(cat $ARCHIVE_DIR/$STAMP_FILE)
      cd \$(docker volume inspect $OLD_VOL -f '{{.Mountpoint}}') && sha256sum --quiet -c $ARCHIVE_DIR/arena-june-\$ts.files.sha256 && echo 'unchanged since backup'"
    if [ "$MODE" = prod ]; then
      say "deploy $1 on the new volume through the homelab playbook"
      (cd "$HOMELAB/ansible" && ansible-playbook -i inventory/homelab.yml playbooks/void-platform-mcow.yml)
    else
      say "start $1 on the new, empty $NEW_VOL"
      remote "docker volume inspect $NEW_VOL >/dev/null 2>&1 && { echo '$NEW_VOL exists: a fresh start needs a new volume' >&2; exit 1; }
        docker volume create $NEW_VOL >/dev/null; docker rm -f $CTR >/dev/null"
      run_arena "$1" "$NEW_VOL"
    fi
    wait_healthy
    mounted_on "$NEW_VOL" ;;

  check)
    need_tag "${1:-}"
    out=$(wait_healthy); echo "$out"
    if [ "$1" = "$JUNE_TAG" ]; then
      [ "$out" = '{"ok":true}' ] && echo "OK: the June image answers" || { echo "FAIL: expected the June {\"ok\":true}" >&2; exit 1; }
    else
      echo "$out" | grep -q "\"image\":\"$IMAGE:$1\"" && echo "OK: /healthz proves $1 is live" \
        || { echo "FAIL: /healthz does not name $IMAGE:$1" >&2; exit 1; }
      if [ "$MODE" = rehearse ]; then say "the new volume starts empty"; counts "$NEW_VOL"; fi
    fi ;;

  rollback)
    if [ "$MODE" = prod ]; then
      : "${HOMELAB:?set HOMELAB to a makscee/homelab clone}"
      say "roll back to $JUNE_TAG on the June volume"
      (cd "$HOMELAB/ansible" && ansible-playbook -i inventory/homelab.yml playbooks/void-platform-mcow.yml \
        -e arena_image_tag="$JUNE_TAG" -e arena_data_volume=arena_data)
    else
      say "roll back: the June image on the June volume ($OLD_VOL), new volume kept for a look"
      remote "docker rm -f $CTR >/dev/null"
      run_arena "$JUNE_TAG" "$OLD_VOL"
    fi
    "$0" check "$JUNE_TAG"
    mounted_on "$OLD_VOL"
    say "June data after rollback"; counts "$OLD_VOL"
    [ "$MODE" = prod ] && echo "NOW set arena_image_tag: \"$JUNE_TAG\" and arena_data_volume: \"arena_data\" back in homelab group_vars"
    true ;;

  cleanup)
    rehearse_only cleanup
    remote "docker rm -f $CTR >/dev/null 2>&1 || true
      docker volume rm $OLD_VOL $NEW_VOL >/dev/null 2>&1 || true
      rm -rf $ARCHIVE_DIR; echo cleaned" ;;

  *) sed -n '2,20p' "$0"; exit 2 ;;
esac
