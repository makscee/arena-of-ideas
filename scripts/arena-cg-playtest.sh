#!/bin/bash
# Arena's playtest frame (#1219, #1220, mission #1214). Since #1220 Arena plays in
# the crazy kit's shared CrazyGames preview, https://m1.twin-pogona.ts.net/r/crazy/cg/arena/,
# made from a checkout of makscee/crazy-kit on m1 or m4 (never tower-w1):
#
#   VITE_API_ORIGIN=https://m1.twin-pogona.ts.net/arena-workshop npm run mvp:build:cg     (here)
#   npm run cgpreview -- <arena>/mobile/dist-cg --media <covers+videos+STORE.md dir> --title "Arena of Ideas" --name arena   (in crazy-kit)
#
# Its mock SDK signs logins with the kit's TEST key; the workshop trusts its public
# half (kit-test-public.pem here) and game id "arena" (scripts/arena-workshop.sh).
# This script now only keeps the old address working: r/arena-cg/ redirects there.
#
#   scripts/arena-cg-playtest.sh          write the redirect at ~/reports/arena-cg/
#   scripts/arena-cg-playtest.sh keys     make an own test key (only for a ref without kit-test-public.pem)
set -euo pipefail
KEYDIR="${ARENA_CG_KEYDIR:-$HOME/arena-workshop/cg-test}"
OUT="${ARENA_CG_OUT:-$HOME/reports/arena-cg}"
TARGET=https://m1.twin-pogona.ts.net/r/crazy/cg/arena/

if [ "${1:-}" = keys ]; then
  if [ ! -f "$KEYDIR/public.pem" ] || [ ! -f "$KEYDIR/private.jwk" ]; then
    mkdir -p "$KEYDIR"
    KEYDIR="$KEYDIR" node -e '
      const { generateKeyPairSync } = require("node:crypto"); const fs = require("node:fs");
      const k = generateKeyPairSync("rsa", { modulusLength: 2048 });
      fs.writeFileSync(process.env.KEYDIR + "/public.pem", k.publicKey.export({ type: "spki", format: "pem" }));
      fs.writeFileSync(process.env.KEYDIR + "/private.jwk", JSON.stringify(k.privateKey.export({ format: "jwk" })), { mode: 0o600 });'
    echo "test key made in $KEYDIR"
  fi
  exit 0
fi

mkdir -p "$OUT"
rm -rf "$OUT/game" "$OUT/shots" "$OUT/arena-cg.zip"
cat > "$OUT/index.html" <<HTML
<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="refresh" content="0; url=$TARGET"><title>Arena of Ideas: moved</title></head>
<body style="background:#241d33;color:#fbf7f0;font:16px system-ui;padding:24px">Arena's CrazyGames frame moved to <a style="color:#f0c24b" href="$TARGET">$TARGET</a>.</body></html>
HTML
echo "r/arena-cg/ now redirects to $TARGET"
