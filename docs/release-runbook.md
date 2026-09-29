# Release runbook: Season 1 fresh start

How arena.makscee.ru goes from the June image (`sha-4a276d3`) to a new build on
an empty database, with the June data archived and a way back
(makscee/void-board#467, mission #423). Maks decided on a fresh start: the June
database is kept, never migrated.

The steps are `ops/release/fresh-start.sh <step>`. It runs from any machine that
can `ssh root@mcow`. `MODE=rehearse` (the default) does everything on a copy of
the prod volume in its own container (`arena-rehearsal`) and volumes; prod's
`arena` container is never touched. `MODE=prod` is the real release, run only on
Maks's go after he accepts the mission.

## What stays where

| | June (today) | Season 1 |
|---|---|---|
| image | `sha-4a276d3` | the `sha-<7>` CI built from main |
| volume (compose `void`) | `void_arena_data`, kept as is | `void_arena_data_s1`, new and empty |
| homelab vars | `arena_image_tag=sha-4a276d3`, `arena_data_volume=arena_data` | `arena_image_tag=sha-<7>`, `arena_data_volume=arena_data_s1` |

The June archive is kept in three places: the untouched `void_arena_data` volume,
a tarball plus its sha256 in `/srv/backups/arena-june/` on mcow, and a restic
snapshot tagged `arena-june-final` in the nether repo. restic-daily's
`forget --keep-daily 7 --keep-weekly 4` has no `--group-by`, so it groups by host
and paths (checked on mcow 2026-09-29), and this snapshot is the only one with
its path, so it is never pruned. If that unit ever gains `--group-by host` or a
tag filter, add `--keep-tag arena-june-final` to it.

## Rehearsal (a copy of prod)

```sh
S=ops/release/fresh-start.sh
$S prepare                # copy void_arena_data, run the June image on the copy
$S backup                 # stop it; tarball + restic snapshot (a throwaway repo)
$S verify                 # restore both, sha256 match the volume, sqlite integrity ok
$S deploy sha-<7>         # the new build on a new, empty volume
$S check sha-<7>          # /healthz names the image; every table starts empty
$S rollback               # the June image on the June volume; June row counts back
$S cleanup                # remove the rehearsal container, volumes and files
```

## Release (prod, on Maks's go)

1. Pick the tag: the `sha-<7>` CI pushed for the merge commit on main.
2. `MODE=prod $S backup` stops `arena` (the site is down from here), writes the
   tarball, a per-file sha256 of the volume and the restic snapshot. Then
   `MODE=prod $S verify`. Stop if anything says MISMATCH or the integrity check
   isn't `ok`.
3. In homelab (with homelab #144 merged), set `arena_image_tag: "sha-<7>"` and
   `arena_data_volume: "arena_data_s1"` in
   `ansible/inventory/group_vars/void_platform.yml`, and merge it. Don't run any
   other playbook on mcow between steps 2 and 4.
4. `HOMELAB=<homelab clone at that commit> MODE=prod $S deploy sha-<7>`. It
   refuses unless group_vars says exactly that tag and volume (no `-e`
   overrides, so the next routine playbook run keeps them), and unless `arena`
   is still stopped and the June volume is unchanged since the backup. Then it
   runs the void-platform-mcow playbook, waits for `/healthz` and proves `arena`
   mounts `void_arena_data_s1`.
5. `MODE=prod $S check sha-<7>`: expect `OK: /healthz proves sha-<7> is live`.
   Then log in on https://arena.makscee.ru and start a run.

## Rollback

`HOMELAB=<homelab clone> MODE=prod $S rollback` runs the playbook with
`arena_image_tag=sha-4a276d3 arena_data_volume=arena_data` and checks that the
June image answers `{"ok":true}`, `arena` mounts `void_arena_data` and the June
rows are there. It uses `-e` so it works in an emergency; then set the two vars
back in homelab, so the next playbook run doesn't redeploy Season 1. The
Season 1 volume stays, so going forward again is the deploy step once more.

If the June volume itself were lost: `docker volume create void_arena_data`,
then untar `/srv/backups/arena-june/arena-june-<ts>.tgz` into it (or
`restic restore latest --tag arena-june-final`), then roll back.

## Build identity

`/healthz` names the image only when CI bakes it in: the build-image job passes
`ARENA_BUILD_COMMIT`, `ARENA_BUILD_IMAGE` and `ARENA_BUILD_TIME` (#467). Images
built before that, including `sha-14c2298`, say `unknown`, and `check` fails on
them.
