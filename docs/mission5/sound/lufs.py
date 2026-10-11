"""Loudness by ITU-R BS.1770 (K-weighted, 48 kHz), numpy + ffmpeg only.

M-max: the loudest 400 ms window (the file padded with 400 ms of silence, so
a 0.1 s tick still gets a window): how loud a short sound lands.
I: gated integrated loudness: how loud a music loop sits.
"""
import subprocess
import numpy as np

SR = 48000


def load(path):
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", path, "-ac", "1", "-ar", str(SR), "-f", "f32le", "-"], capture_output=True, check=True).stdout
    return np.frombuffer(raw, dtype=np.float32).astype(np.float64)


def _biquad(x, b, a):
    try:
        from scipy.signal import lfilter
        return lfilter(b, a, x)
    except ImportError:
        pass
    # Without scipy: the same filter, slowly.
    y = np.zeros_like(x)
    x1 = x2 = y1 = y2 = 0.0
    for i, v in enumerate(x):
        o = b[0] * v + b[1] * x1 + b[2] * x2 - a[1] * y1 - a[2] * y2
        x2, x1, y2, y1 = x1, v, y1, o
        y[i] = o
    return y


def kweight(x):
    x = _biquad(x, (1.53512485958697, -2.69169618940638, 1.19839281085285), (1, -1.69065929318241, 0.73248077421585))
    return _biquad(x, (1.0, -2.0, 1.0), (1, -1.99004745483398, 0.99007225036621))


def measure(x):
    """(M-max, I) in LUFS."""
    y = kweight(np.concatenate([x, np.zeros(int(SR * 0.4))]))
    w, hop = int(0.4 * SR), int(0.1 * SR)
    ms = np.array([np.mean(y[i:i + w] ** 2) for i in range(0, max(1, len(y) - w + 1), hop)]) + 1e-12
    loud = -0.691 + 10 * np.log10(ms)
    gated = ms[loud > -70]
    if len(gated) == 0:
        return float(loud.max()), -70.0
    rel = -0.691 + 10 * np.log10(gated.mean()) - 10
    gated = gated[(-0.691 + 10 * np.log10(gated)) > rel]
    return float(loud.max()), float(-0.691 + 10 * np.log10(gated.mean()))


def voice(x, rate=1.0, cut=None, fade=0.06):
    """What the engine plays: x at playbackRate `rate` (pitch and speed),
    stopped after `cut` seconds with a short fade."""
    if rate != 1:
        n = int(len(x) / rate)
        x = np.interp(np.arange(n) * rate, np.arange(len(x)), x)
    if cut:
        n = int(cut * SR)
        if n < len(x):
            x = x[:n].copy()
            f = min(int(fade * SR), n)
            x[-f:] *= np.linspace(1, 0, f)
    return x
