"""Renders a session timeline (timeline.ts) to audio the way the game mixes
it, so the sound can be judged in one listen (void-board#922).

    python3 -I render.py before|after <timeline.json> <sfx dir> <music dir> <out.mp3>

before: every cue plays (only the same cue within 70 ms is skipped); the
2022 home loop; music at the old curve (slider 0.4 squared).
after: admit() (GAP_MS, KEY_COOLDOWN_MS, MAX_VOICES), cuts, home on theme4,
levelled tracks at the linear curve. Both: default sliders (Sound 0.6
squared on the master), the shop's 700 Hz low-pass, 1 s crossfades.
"""
import json, os, sys
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from lufs import SR, load, voice

mode, tl_path, sfx_dir, music_dir, out = sys.argv[1:6]
tl = json.load(open(tl_path))
root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
tracks = json.load(open(os.path.join(root, "mobile", "ui", "tracks.json")))
MASTER = 0.6 ** 2
end = tl["battle"][1] / 1000
mix = np.zeros(int(end * SR) + SR)

# ---- music ----
if mode == "before":
    home, theme, level, tgain = "home.m4a", "theme1.m4a", 0.4 ** 2, {}
else:
    home, theme, level = tracks["home"]["file"], "theme1.m4a", 0.4
    tgain = {t["file"]: t["gain"] for t in [tracks["home"], *tracks["themes"]]}

def lowpass(x, hz):
    from scipy.signal import butter, lfilter
    b, a = butter(2, hz / (SR / 2))
    return lfilter(b, a, x)

def bed(file, t0, t1, muffle=False, offset=0.0):
    x = load(os.path.join(music_dir, file)) * level * tgain.get(file, 1.0)
    n0, n1 = int(t0 * SR), int(t1 * SR)
    seg = np.resize(x, int(offset * SR) + n1 - n0)[int(offset * SR):]
    if muffle:
        seg = lowpass(seg, 700)
    f = SR  # 1 s fades in and out
    env = np.ones(len(seg))
    env[:f] = np.linspace(0, 1, f)
    env[-f:] = np.linspace(1, 0, f)
    mix[n0:n1] += seg * env

h0, h1 = tl["home"][0] / 1000, tl["home"][1] / 1000
s1, b1 = tl["shop"][1] / 1000, tl["battle"][1] / 1000
if home == theme:
    bed(home, h0, h1 + 0.5)
else:
    bed(home, h0, h1 + 1)
bed(theme, h1, s1 + 0.5, muffle=True)
bed(theme, s1 - 0.5, b1, offset=s1 - h1 - 0.5)

# ---- sfx ----
GAP, COOL, VOICES = 90, 120, 3
cache = {}
voices = []  # sounding: [id, key, prio, at_ms, start_sample, end_sample, samples]
played = []  # every voice, for the stats
heard = 0
for i, ev in enumerate(sorted(tl["events"], key=lambda e: e["t"] + e["cue"].get("delay", 0))):
    c, t = ev["cue"], ev["t"] + ev["cue"].get("delay", 0)
    file = c.get("file", c["key"])
    if mode == "before":
        if any(v[1] == c["key"] and t - v[3] < 70 for v in voices):
            continue
        cut = []
    else:
        voices = [v for v in voices if v[5] > int(t / 1000 * SR)]
        if any(v[1] == c["key"] and t - v[3] < COOL for v in voices):
            continue
        close = [v for v in voices if t - v[3] < GAP]
        if any(v[2] >= c["prio"] for v in close):
            continue
        cut = [v[0] for v in close]
        left = sorted([v for v in voices if v[0] not in cut], key=lambda v: (v[2], v[3]))
        while len(left) >= VOICES:
            cut.append(left.pop(0)[0])
    n = int(t / 1000 * SR)
    for v in voices:
        if v[0] in cut:  # fades out over 40 ms from now, then silence
            off, f = n - v[4], int(0.04 * SR)
            seg = v[6][off:]
            kept = seg.copy()
            kept[:f] *= np.linspace(1, 0, f)[: len(kept[:f])]
            kept[f:] = 0
            mix[n:n + len(seg)] -= seg - kept
            v[5] = n + min(f, len(seg))
    x = cache.setdefault(file, load(os.path.join(sfx_dir, file + ".mp3")))
    y = voice(x, c["rate"], c.get("cut")) * c["gain"]
    m = min(len(y), len(mix) - n)
    mix[n:n + m] += y[:m]
    voices.append([i, c["key"], c.get("prio", 0), t, n, n + m, y[:m]])
    played.append(voices[-1])
    heard += 1

# How many sounds ring at once (a voice counts until it ends or is cut).
busy = np.zeros(len(mix), dtype=int)
for v in played:
    busy[v[4]:v[5]] += 1
mix *= MASTER
peak = np.abs(mix).max()
print(f"{mode}: {heard} of {len(tl['events'])} cues heard, at most {busy.max()} at once, 3+ at once {100*(busy>=3).mean():.1f}% of the time, peak {20*np.log10(peak):.1f} dBFS")
import subprocess
subprocess.run(["ffmpeg", "-v", "error", "-y", "-f", "f64le", "-ar", str(SR), "-ac", "1", "-i", "-", "-c:a", "libmp3lame", "-b:a", "128k", out], input=np.clip(mix, -1, 1).astype(np.float64).tobytes(), check=True)
