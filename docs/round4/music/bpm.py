import sys, subprocess, numpy as np
SR=11025; HOP=128
def load(p):
    raw = subprocess.run(["ffmpeg","-v","quiet","-i",p,"-ac","1","-ar",str(SR),"-f","f32le","-"],capture_output=True).stdout
    return np.frombuffer(raw, np.float32)
def flux(y):
    n=1024; frames = np.lib.stride_tricks.sliding_window_view(y, n)[::HOP]*np.hanning(n)
    S=np.log1p(10*np.abs(np.fft.rfft(frames,axis=1)))
    f=np.maximum(np.diff(S,axis=0),0).sum(1)
    f=f-np.convolve(f,np.ones(64)/64,'same'); return np.maximum(f,0)
fps=SR/HOP
for p in sys.argv[1:]:
    y=load(p); f=flux(y); dur=len(y)/SR
    best=None
    for bpm in np.arange(70,170,0.02):
        per=fps*60/bpm
        # comb score with best phase
        idx=np.arange(0,len(f)-1,per)
        ph=np.arange(0,per,1.0)
        sc=[f[(idx+q).astype(int)[(idx+q)<len(f)]].mean() for q in ph[::2]]
        i=int(np.argmax(sc)); s=sc[i]
        if best is None or s>best[0]: best=(s,bpm,ph[::2][i]/fps)
    s,bpm,off=best
    print(f"{p.split('/')[-1]}\tdur={dur:.3f}\tbpm={bpm:.2f}\toffset={off:.3f}\tbeats={dur*bpm/60:.2f}",flush=True)
