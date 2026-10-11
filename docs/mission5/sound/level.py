"""Levels the sounds (void-board#922): writes each cue's gain into
mobile/ui/sounds.json and each music track's gain into mobile/ui/tracks.json,
and prints the before/after table.

    python3 -I docs/mission5/sound/level.py [before-sfx-dir]

A cue's gain brings its M-max (as played: its file at its rate and cut) to its
tier's level; a track's brings its integrated loudness to MUSIC_LUFS.
"""
import json, os, sys
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from lufs import load, measure, voice

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
SFX = os.path.join(ROOT, "mobile", "public", "sfx")
MUSIC = os.path.join(ROOT, "mobile", "public", "music")
MUSIC_LUFS = -18.0

sounds_path = os.path.join(ROOT, "mobile", "ui", "sounds.json")
spec = json.load(open(sounds_path))
cache = {}
rows = []
for key, c in spec["cues"].items():
    x = cache.setdefault(c["file"], load(os.path.join(SFX, c["file"] + ".mp3")))
    v = voice(x, c["rate"], c.get("cut"))
    m, _ = measure(v)
    target = spec["tiers"][c["tier"]]
    # Never past -1 dBFS at full volume: a quiet file stays a little under its tier.
    peak = 20 * np.log10(np.abs(v).max())
    c["gain"] = round(min(10 ** ((target - m) / 20), 10 ** ((-1 - peak) / 20)), 2)
    rows.append((key, c["file"], m, c["gain"], round(m + 20 * np.log10(c["gain"]), 1)))
with open(sounds_path, "w") as f:
    f.write("{\n")
    f.write(f'  "files": {json.dumps(spec["files"])},\n')
    f.write(f'  "tiers": {json.dumps(spec["tiers"])},\n')
    f.write('  "cues": {\n')
    lines = [f'    {json.dumps(k)}: {json.dumps(c, separators=(", ", ": "))}' for k, c in spec["cues"].items()]
    f.write(",\n".join(lines) + "\n  }\n}\n")

tracks_path = os.path.join(ROOT, "mobile", "ui", "tracks.json")
tracks = json.load(open(tracks_path))
music_rows = []
for t in [tracks["home"], *tracks["themes"]]:
    _, i = measure(load(os.path.join(MUSIC, t["file"])))
    t["gain"] = round(10 ** ((MUSIC_LUFS - i) / 20), 2)
    music_rows.append((t["file"], i, t["gain"]))
with open(tracks_path, "w") as f:
    f.write("{\n")
    f.write(f'  "home": {json.dumps(tracks["home"], separators=(", ", ": "))},\n')
    f.write('  "themes": [\n' + ",\n".join(f'    {json.dumps(t, separators=(", ", ": "))}' for t in tracks["themes"]) + "\n  ]\n}\n")

print("| cue | file | M-max as played | gain | lands at |")
print("|---|---|---|---|---|")
for k, fl, m, g, t in rows:
    print(f"| {k} | {fl} | {m:.1f} | {g} | {t} |")
print()
print("| track | I (LUFS) | gain | lands at |")
print("|---|---|---|---|")
for fl, i, g in music_rows:
    print(f"| {fl} | {i:.1f} | {g} | {MUSIC_LUFS} |")
