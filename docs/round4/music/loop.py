# usage: loop.py in.ogg out.m4a bpm offset beats  -> cut `beats` beats from the first beat, bake a 0.5 s crossfade seam, AAC 96k
import sys, subprocess, numpy as np
src,out,bpm,off,beats=sys.argv[1],sys.argv[2],float(sys.argv[3]),float(sys.argv[4]),int(sys.argv[5])
SR=44100
raw=subprocess.run(["ffmpeg","-v","quiet","-i",src,"-ac","2","-ar",str(SR),"-f","f32le","-"],capture_output=True).stdout
a=np.frombuffer(raw,np.float32).reshape(-1,2)
t0=int(round(off*SR)); L=int(round(beats*60/bpm*SR)); X=int(0.5*SR)
seg=a[t0:t0+L+X].copy(); assert len(seg)==L+X, (len(seg),L+X)
r=np.linspace(0,1,X,dtype=np.float32)[:,None]
o=seg[:L].copy(); o[:X]=seg[:X]*np.sqrt(r)+seg[L:L+X]*np.sqrt(1-r)
p=subprocess.run(["ffmpeg","-v","error","-y","-f","f32le","-ar",str(SR),"-ac","2","-i","-","-c:a","aac","-b:a","96k","-movflags","+faststart",out],input=o.tobytes())
print(out, L/SR)
