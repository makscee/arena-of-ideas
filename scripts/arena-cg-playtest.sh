#!/bin/bash
# Arena's playtest frame (#1219, mission #1214): the cg build in an iframe on a
# CrazyGames-like page, against the workshop server, at
#   https://m1.twin-pogona.ts.net/r/arena-cg/
# Maks's rule: every game is played in such a frame before he plays it. Run on
# m1 from a checkout (scripts/arena-workshop.sh runs it after each deploy):
#
#   scripts/arena-cg-playtest.sh          build the cg zip against the workshop, publish the page,
#                                         serve the game from its own origin (:8444)
#   scripts/arena-cg-playtest.sh keys     only make the test key the workshop trusts (once)
#
# The game's files sit at https://m1.twin-pogona.ts.net:8444/ (the same
# static server as /r, another port), so the iframe has its own origin and
# its own storage: a first open is a fresh guest, never Maks's workshop session.
# The SDK is scripts/arena-cg-playtest/cg-mock-sdk.js (the real one stays
# "disabled" off crazygames.com); it signs a login token with a test key kept in
# $ARENA_CG_KEYDIR (default ~/arena-workshop/cg-test), whose public half the
# workshop server takes with ARENA_CG_PUBLIC_KEY. Tailnet only.
set -euo pipefail
export PATH=/opt/homebrew/bin:/usr/local/bin:$PATH
cd "$(dirname "$0")/.."
KEYDIR="${ARENA_CG_KEYDIR:-$HOME/arena-workshop/cg-test}"
OUT="${ARENA_CG_OUT:-$HOME/reports/arena-cg}"
HOST=m1.twin-pogona.ts.net
API_ORIGIN="${ARENA_CG_API_ORIGIN:-https://$HOST/arena-workshop}"
GAME_PORT="${ARENA_CG_GAME_PORT:-8444}"
TS="${TAILSCALE:-/Applications/Tailscale.app/Contents/MacOS/Tailscale}"

if [ ! -f "$KEYDIR/public.pem" ] || [ ! -f "$KEYDIR/private.jwk" ]; then
  mkdir -p "$KEYDIR"
  KEYDIR="$KEYDIR" node -e '
    const { generateKeyPairSync } = require("node:crypto"); const fs = require("node:fs");
    const k = generateKeyPairSync("rsa", { modulusLength: 2048 });
    fs.writeFileSync(process.env.KEYDIR + "/public.pem", k.publicKey.export({ type: "spki", format: "pem" }));
    fs.writeFileSync(process.env.KEYDIR + "/private.jwk", JSON.stringify(k.privateKey.export({ format: "jwk" })), { mode: 0o600 });'
  echo "test key made in $KEYDIR"
fi
[ "${1:-}" = keys ] && exit 0

VITE_API_ORIGIN="$API_ORIGIN" bash scripts/build-cg.sh
rm -rf "$OUT/game"
mkdir -p "$OUT/game" "$OUT/shots"
cp -R mobile/dist-cg/. "$OUT/game/"
cp mobile/arena-cg.zip "$OUT/arena-cg.zip"
cp scripts/arena-cg-playtest/cg-mock-sdk.js "$OUT/game/"
printf 'window.__CG_MOCK_KEY = %s;\n' "$(cat "$KEYDIR/private.jwk")" > "$OUT/game/cg-mock-key.js"
# The mock SDK stands where the SDK's <script> is, ahead of the game.
SDK_URL=https://sdk.crazygames.com/crazygames-sdk-v3.js
grep -q "$SDK_URL" "$OUT/game/index.html" || { echo "the cg build's index.html has no SDK <script>" >&2; exit 1; }
sed -i.bak "s#<script src=\"$SDK_URL\"></script>#<script src=\"./cg-mock-key.js\"></script><script src=\"./cg-mock-sdk.js\"></script>#; s#<script src=\"$SDK_URL\">#<script src=\"./cg-mock-key.js\"></script><script src=\"./cg-mock-sdk.js\">#" "$OUT/game/index.html"
rm -f "$OUT/game/index.html.bak"
grep -q cg-mock-sdk.js "$OUT/game/index.html" || { echo "the SDK <script> wasn't replaced" >&2; exit 1; }
cp scripts/arena-cg-playtest/index.html "$OUT/index.html"

"$TS" serve --bg --https="$GAME_PORT" http://127.0.0.1:8787 >/dev/null
echo "playtest page: https://$HOST/r/arena-cg/   game origin: https://$HOST:$GAME_PORT/arena-cg/game/index.html"
for u in "https://$HOST/r/arena-cg/" "https://$HOST:$GAME_PORT/arena-cg/game/index.html"; do
  curl -fsS --max-time 10 -o /dev/null -w "%{http_code} $u\n" "$u" || echo "NOT reachable: $u" >&2
done
