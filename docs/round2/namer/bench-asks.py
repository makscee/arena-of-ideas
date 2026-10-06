# R2-4 (#594): runs asks.json (asks.mts) on one model, each ask at its own
# temperature, max_tokens 12 like the server. Writes JSONL rows like bench.py.
import json, sys, time
from mlx_lm import load, generate
from mlx_lm.sample_utils import make_sampler

model_id, asks_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
asks = json.load(open(asks_path))
model, tok = load(model_id)
with open(out_path, "w") as f:
    for a in asks:
        prompt = tok.apply_chat_template(a["messages"], add_generation_prompt=True, tokenize=False)
        t = time.time()
        text = generate(model, tok, prompt=prompt, max_tokens=12, sampler=make_sampler(temp=a["temp"]), verbose=False)
        row = {k: a.get(k) for k in ("variant", "first", "second", "a", "b", "k", "letter")}
        f.write(json.dumps({"model": model_id, **row, "raw": text, "s": round(time.time() - t, 3)}) + "\n")
        f.flush()
