# One-off namer bench: loads one model, asks every pair with the real and the
# one-word prompt, SAMPLES times each, at the server's settings (temp 0.8,
# max_tokens 12). Writes JSONL of raw answers with latency, then a summary.
import json, sys, time, resource
import mlx.core as mx
from mlx_lm import load, generate
from mlx_lm.sample_utils import make_sampler

model_id, pairs_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
SAMPLES = int(sys.argv[4]) if len(sys.argv) > 4 else 2
VARIANTS = sys.argv[5].split(",") if len(sys.argv) > 5 else ["real", "one"]
pairs = json.load(open(pairs_path))
t0 = time.time()
model, tok = load(model_id)
load_s = time.time() - t0
sampler = make_sampler(temp=0.8)
lat = []
with open(out_path, "w") as f:
    for variant in VARIANTS:
        for p in pairs:
            prompt = tok.apply_chat_template(p[variant], add_generation_prompt=True, tokenize=False)
            for k in range(SAMPLES):
                t = time.time()
                text = generate(model, tok, prompt=prompt, max_tokens=12, sampler=sampler, verbose=False)
                dt = time.time() - t
                lat.append(dt)
                f.write(json.dumps({"model": model_id, "variant": variant, "first": p["first"], "second": p["second"], "a": p["a"], "b": p["b"], "k": k, "raw": text, "s": round(dt, 3)}) + "\n")
                f.flush()
lat.sort()
print(json.dumps({"model": model_id, "load_s": round(load_s, 1), "n": len(lat),
                  "lat_med": round(lat[len(lat)//2], 3), "lat_p90": round(lat[int(len(lat)*0.9)], 3), "lat_max": round(lat[-1], 3),
                  "peak_gpu_gb": round(mx.get_peak_memory() / 1e9, 2),
                  "maxrss_gb": round(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1e9, 2)}))
