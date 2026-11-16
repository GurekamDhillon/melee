#!/usr/bin/env python3
"""A loopback stand-in for the bracket server's upload routes (melee-autobracket docs/api.md), for the engine's bracket
client tests. Never a real server: binds 127.0.0.1 only.

  python test_server.py PORT OUT_DIR [EVENT] [SET] [GAME]

Answers GET / (200), GET /v1/me/next?event=EVENT ({"Play": {... set_id SET, game_no GAME ...}}) and
POST /v1/sets/EVENT/SET/games/GAME/replay (202, 200 for an identical re-upload), after checking every signed request
the way the server does: the four x-ab-* headers, the clock within 120 s, a fresh nonce, and the Ed25519 signature
over METHOD\\nPATH\\nTIME\\nNONCE\\nsha256(body). Each accepted upload is saved as OUT_DIR/<key prefix>-<n>.slp and logged
as one line to OUT_DIR/uploads.log: key prefix, path, bytes, sha256, x-ab-build.
"""
import hashlib, http.server, json, os, sys, time
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

PORT, OUT = int(sys.argv[1]), sys.argv[2]
EVENT, SET, GAME = (int(a) for a in (sys.argv[3:6] + ["7", "3", "1"][len(sys.argv[3:6]):]))
os.makedirs(OUT, exist_ok=True)
NONCES, STORED = set(), {}


class H(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def answer(self, code, obj=None):
        b = b"" if obj is None else json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("content-length", str(len(b)))
        self.end_headers()
        self.wfile.write(b)

    def check(self, body):
        h = {k: self.headers.get("x-ab-" + k) for k in ("key", "time", "nonce", "sig")}
        if None in h.values():
            return "missing signature headers"
        try:
            key = Ed25519PublicKey.from_public_bytes(bytes.fromhex(h["key"]))
            sig, t = bytes.fromhex(h["sig"]), int(h["time"])
            assert len(bytes.fromhex(h["nonce"])) == 16
        except Exception:
            return "malformed signature headers"
        if abs(time.time() - t) > 120:
            return "clock skew"
        msg = "\n".join([self.command, self.path, h["time"], h["nonce"], hashlib.sha256(body).hexdigest()]).encode()
        try:
            key.verify(sig, msg)
        except Exception:
            return "invalid signature"
        if (h["key"], h["nonce"]) in NONCES:
            return "nonce already used"
        NONCES.add((h["key"], h["nonce"]))
        self.key = h["key"]
        return None

    def do_GET(self):
        if self.path == "/":
            return self.answer(200, {"ok": True})
        err = self.check(b"")
        if err:
            return self.answer(401, {"error": err})
        if self.path == f"/v1/me/next?event={EVENT}":
            return self.answer(200, {"Play": {"set_id": SET, "opponent_tag": "peer", "role": "Host", "relay": "127.0.0.1:1",
                                              "room_code": None, "game_no": GAME, "wins": [0, 0], "best_of": 3, "stocks": 4,
                                              "timer_secs": 480, "awaiting_verdict": False}})
        self.answer(404, {"error": "no such route"})

    def do_POST(self):
        n = int(self.headers.get("content-length") or 0)
        body = self.rfile.read(n)
        err = self.check(body)
        if err:
            return self.answer(401, {"error": err})
        if self.path != f"/v1/sets/{EVENT}/{SET}/games/{GAME}/replay":
            return self.answer(404, {"error": "no such route"})
        if not body.startswith(b"{U\x03raw[$U#l"):
            return self.answer(400, {"error": "not a readable replay"})
        digest = hashlib.sha256(body).hexdigest()
        slot = (self.key, self.path)
        if STORED.get(slot) == digest:
            return self.answer(200)
        if slot in STORED:
            return self.answer(409, {"error": "a different file is stored for this slot"})
        STORED[slot] = digest
        k = self.key[:12]
        with open(os.path.join(OUT, f"{k}-{len(STORED)}.slp"), "wb") as f:
            f.write(body)
        with open(os.path.join(OUT, "uploads.log"), "a") as f:
            f.write(f"{k} {self.path} {len(body)} {digest} {self.headers.get('x-ab-build')}\n")
        self.answer(202)


print(f"bracket test server on 127.0.0.1:{PORT}", flush=True)
http.server.ThreadingHTTPServer(("127.0.0.1", PORT), H).serve_forever()
