#!/usr/bin/env bash
# The CrazyGames build (mission #1214): mobile/dist-cg/ and mobile/arena-cg.zip
# (index.html at the zip's root, relative paths only). VITE_API_ORIGIN names
# the Arena server the zip talks to (no trailing slash; empty: its own base).
# Prints the file count and size against CrazyGames' limits (1500 files, 50 MB).
set -euo pipefail
cd "$(dirname "$0")/.."
npx vite build --config mobile/vite.config.ts --mode cg
cd mobile/dist-cg
rm -f ../arena-cg.zip
zip -qr -X ../arena-cg.zip .
files=$(find . -type f | wc -l | tr -d ' ')
kb=$(du -sk . | cut -f1)
echo "arena-cg.zip: $files files, ${kb} KB unzipped, $(( $(wc -c < ../arena-cg.zip) / 1024 )) KB zipped (limits: 1500 files, 50 MB)"
[ "$files" -le 1500 ] || { echo "too many files for CrazyGames" >&2; exit 1; }
