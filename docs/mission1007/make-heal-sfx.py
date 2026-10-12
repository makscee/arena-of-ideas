"""Synthesises mobile/public/sfx/heal.mp3 (mission 1007, #1011): a soft two-note
rising chime (G5 then D6, bell partials), 0.6 s, mono, peak-normalised.
The one battle sound that isn't from Duamo: it must not sound like buff.

    python3 -I docs/mission1007/make-heal-sfx.py
"""
import os, subprocess, tempfile, wave
import numpy as np

SR = 44100
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
OUT = os.path.join(ROOT, "mobile", "public", "sfx", "heal.mp3")

def note(freq, dur, start, total):
    t = np.arange(int(SR * dur)) / SR
    env = np.minimum(t / 0.008, 1) * np.exp(-t * 6.0)
    y = np.zeros_like(t)
    for k, amp in ((1, 1.0), (2, 0.35), (3.01, 0.12), (4.2, 0.05)):
        y += amp * np.sin(2 * np.pi * freq * k * t)
    out = np.zeros(int(SR * total))
    s = int(SR * start)
    out[s:s + len(y)] += y * env
    return out

x = note(784.0, 0.5, 0.0, 0.7) + 0.9 * note(1174.7, 0.55, 0.11, 0.7)
x *= 0.89 / np.abs(x).max()
with tempfile.TemporaryDirectory() as d:
    wav = os.path.join(d, "heal.wav")
    with wave.open(wav, "wb") as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(SR)
        w.writeframes((x * 32767).astype("<i2").tobytes())
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", wav, "-ac", "1", "-b:a", "96k", OUT], check=True)
print(OUT, os.path.getsize(OUT), "bytes")
