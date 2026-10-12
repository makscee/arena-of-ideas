#!/usr/bin/env python3
"""The fusion namer, loaded only while it is used (void-board, 10-12).

The namer (mlx_lm.server, Qwen3-4B, ~2.2 GB) was asked a few minutes a day but
held its memory all day on m1, which then swapped. This stays on the namer's
port and starts the model on the first ask, forwards every request to it, and
stops it after IDLE seconds without one. While the model loads, a request
waits up to WAIT seconds, then gets a 503: the server's namer (fusions.ts)
waits a loading model out with its backoff, and names by portmanteau
meanwhile. Stdlib only.

  namer-on-demand.py --port 8792 --model M [--idle 600] [--wait 12]
"""
import argparse
import http.client
import http.server
import os
import signal
import subprocess
import sys
import threading
import time

ap = argparse.ArgumentParser()
ap.add_argument("--port", type=int, default=8792)
ap.add_argument("--model", required=True)
ap.add_argument("--idle", type=float, default=600)
ap.add_argument("--wait", type=float, default=12)
ap.add_argument("--inner-port", type=int, default=0)  # the model's own port; default port + 1
ap.add_argument("--cmd", default="/opt/homebrew/bin/uvx --from mlx-lm mlx_lm.server")  # tests pass a fake
args = ap.parse_args()
INNER = args.inner_port or args.port + 1

lock = threading.Lock()
child = None  # the model's process (its own process group)
last_use = 0.0


def log(msg):
    print(time.strftime("%Y-%m-%d %H:%M:%S"), "namer-on-demand:", msg, flush=True)


def ready():
    try:
        c = http.client.HTTPConnection("127.0.0.1", INNER, timeout=2)
        c.request("GET", "/v1/models")
        ok = c.getresponse().status == 200
        c.close()
        return ok
    except OSError:
        return False


def ensure_started():
    global child
    with lock:
        if child is None or child.poll() is not None:
            cmd = args.cmd.split() + ["--model", args.model, "--host", "127.0.0.1", "--port", str(INNER)]
            child = subprocess.Popen(cmd, start_new_session=True, stdout=sys.stdout, stderr=sys.stderr)
            log(f"started the model (pid {child.pid})")


def stop(reason):
    global child
    with lock:
        if child is not None and child.poll() is None:
            log(f"stopping the model ({reason})")
            try:
                os.killpg(child.pid, signal.SIGTERM)
                child.wait(10)
            except (ProcessLookupError, subprocess.TimeoutExpired):
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
        child = None


def reaper():
    while True:
        time.sleep(15)
        if child is not None and time.time() - last_use > args.idle:
            stop(f"idle {int(args.idle)} s")


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *a):
        pass

    def forward(self):
        global last_use
        last_use = time.time()
        body = self.rfile.read(int(self.headers.get("Content-Length") or 0))
        ensure_started()
        deadline = time.time() + args.wait
        while not ready():
            if time.time() > deadline:
                return self.send(503, b'{"error":"the namer is loading"}')
            time.sleep(0.3)
        try:
            c = http.client.HTTPConnection("127.0.0.1", INNER, timeout=120)
            hdrs = {k: v for k, v in self.headers.items() if k.lower() in ("content-type", "authorization", "accept")}
            c.request(self.command, self.path, body=body or None, headers=hdrs)
            r = c.getresponse()
            data = r.read()
            self.send(r.status, data, r.getheader("Content-Type", "application/json"))
            c.close()
        except OSError as e:
            self.send(502, f'{{"error":"{type(e).__name__}"}}'.encode())
        last_use = time.time()

    def send(self, status, data, ctype="application/json"):
        self.send_response(status)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    do_GET = do_POST = forward


def bye(*_):
    stop("exit")
    sys.exit(0)


signal.signal(signal.SIGTERM, bye)
signal.signal(signal.SIGINT, bye)
threading.Thread(target=reaper, daemon=True).start()
log(f"on :{args.port}, the model on :{INNER} only while used (idle {int(args.idle)} s)")
http.server.ThreadingHTTPServer(("127.0.0.1", args.port), Handler).serve_forever()
