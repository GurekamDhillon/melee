#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Mock Slippi matchmaking server plus a loopback UDP relay that records the netplay traffic.

Loopback only by default: every socket here binds 127.0.0.1. Nothing talks to a Slippi service.
`--bind ADDR` (a private address, e.g. the Windows host's WSL switch address) serves peers on other machines of that
network instead: the server and relay sockets bind ADDR, each player is reached at the address its WebSocket came from,
and the engines run with `--mm-lan` (docs/rust-port/notes/linux.md, cross-OS netplay).

    python -I tools/rust-port/slippi/mock_mm.py --port 43110 --capture OUT.cap [--log OUT.log]

Point Slippi Dolphin at it with SLIPPI_MM_URL=ws://127.0.0.1:43110/v1/queue (and SLIPPI_LOCAL_TEST=1, the local Dolphin
patch). The protocol is the one in Externals/SlippiRustExtensions/matchmaking (protocol.rs, client.rs) and
SlippiMatchmaking.cpp's match handler: `join` -> `joined{ticketId, token}` and `status` every 2 s; once two `join`s
are queued (any mode, any code) both get `match{matchId, isHost, players[...], stages}`.

The relay: each player is told its opponent is at 127.0.0.1:<relay port>, and the relay forwards datagrams between the
two real netplay ports, so both ENet peers talk through it unchanged. Every datagram is appended to the capture file as
`<t_ms> <A>B|B>A> <hex>` (decode with enet_decode.py).
"""
import argparse
import base64
import hashlib
import ipaddress
import json
import select
import socket
import struct
import threading
import time

LOOP = "127.0.0.1"
GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"


def log(msg):
    print(f"mock_mm: {msg}", flush=True)


class Conn:
    def __init__(self, sock):
        self.sock = sock
        self.buf = b""
        self.lock = threading.Lock()

    def handshake(self):
        while b"\r\n\r\n" not in self.buf:
            d = self.sock.recv(4096)
            if not d:
                raise EOFError
            self.buf += d
        head, self.buf = self.buf.split(b"\r\n\r\n", 1)
        key = ""
        for line in head.decode("latin1").split("\r\n"):
            if line.lower().startswith("sec-websocket-key:"):
                key = line.split(":", 1)[1].strip()
        acc = base64.b64encode(hashlib.sha1((key + GUID).encode()).digest()).decode()
        self.sock.sendall(("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n"
                           f"Sec-WebSocket-Accept: {acc}\r\n\r\n").encode())

    def _need(self, n):
        while len(self.buf) < n:
            d = self.sock.recv(4096)
            if not d:
                raise EOFError
            self.buf += d

    def recv(self):
        """Returns (opcode, payload)."""
        self._need(2)
        b0, b1 = self.buf[0], self.buf[1]
        n, p = b1 & 0x7F, 2
        if n == 126:
            self._need(4)
            n, p = struct.unpack(">H", self.buf[2:4])[0], 4
        elif n == 127:
            self._need(10)
            n, p = struct.unpack(">Q", self.buf[2:10])[0], 10
        mask = b""
        if b1 & 0x80:
            self._need(p + 4)
            mask, p = self.buf[p:p + 4], p + 4
        self._need(p + n)
        data = bytearray(self.buf[p:p + n])
        self.buf = self.buf[p + n:]
        if mask:
            for i in range(n):
                data[i] ^= mask[i % 4]
        return b0 & 0x0F, bytes(data)

    def send(self, obj, opcode=1):
        data = json.dumps(obj).encode() if opcode == 1 else obj
        n = len(data)
        hdr = bytes([0x80 | opcode]) + (bytes([n]) if n < 126 else bytes([126]) + struct.pack(">H", n))
        with self.lock:
            self.sock.sendall(hdr + data)


class Server:
    def __init__(self, port, capture, delay_ms, stages=None, lag_ms=0, loss_pct=0):
        self.port = port
        self.bind = LOOP
        self.capture = open(capture, "w") if capture else None
        self.queue = []
        self.lock = threading.Lock()
        self.t0 = time.time()
        self.delay_ms = delay_ms
        self.stages = stages or [0x2, 0x3, 0x8, 0x1C, 0x1F, 0x20]
        # relay impairment, both directions: each datagram is dropped with loss_pct %, else held lag_ms
        self.lag = lag_ms / 1000.0
        self.loss = loss_pct
        self.rng = 0x1234567

    def serve(self):
        ls = socket.socket()
        ls.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        ls.bind((self.bind, self.port))
        ls.listen(4)
        log(f"listening on ws://{self.bind}:{self.port}/v1/queue")
        while True:
            s, addr = ls.accept()
            threading.Thread(target=self.client, args=(s,), daemon=True).start()

    def client(self, s):
        c = Conn(s)
        try:
            c.handshake()
            n = 0
            while True:
                op, data = c.recv()
                if op == 8:
                    break
                if op == 9:
                    c.send(data, opcode=10)
                    continue
                if op != 1:
                    continue
                msg = json.loads(data)
                msg["_peer_ip"] = s.getpeername()[0]  # where this player's netplay port is reached
                kind = msg.get("type")
                if kind == "join":
                    n += 1
                    user = msg.get("user", {})
                    log(f"join from {user.get('connectCode')} mode {msg.get('search', {}).get('mode')} "
                        f"port {msg.get('port')} lan {msg.get('ipAddressLan')} appVersion {msg.get('appVersion')} "
                        f"keys {sorted(msg.keys())}")
                    c.send({"type": "joined", "ticketId": f"t{id(c)}", "token": "local"})
                    threading.Thread(target=self.status_loop, args=(c,), daemon=True).start()
                    with self.lock:
                        mate = next((q for q in self.queue if self.compatible(q[1], msg)), None)
                        if mate:
                            self.queue.remove(mate)
                            self.pair(mate, (c, msg))
                        else:
                            self.queue.append((c, msg))
                elif kind == "ping":
                    c.send({"type": "pong"})
                else:
                    log(f"message {kind}")
        except (EOFError, OSError, ValueError) as e:
            log(f"client closed ({type(e).__name__})")

    @staticmethod
    def compatible(a, b):
        """Direct (mode 2): one side searches for the other's code (Shift-JIS bytes = ASCII here). Others: any."""
        sa, sb = a.get("search", {}), b.get("search", {})
        if sa.get("mode") != sb.get("mode"):
            return False
        if sa.get("mode") != 2:
            return True
        code = lambda s: bytes(s.get("connectCode") or []).decode("latin1")
        ua, ub = a.get("user", {}).get("connectCode", ""), b.get("user", {}).get("connectCode", "")
        return code(sa) == ub or code(sb) == ua

    def status_loop(self, c):
        t = time.time()
        try:
            for _ in range(600):
                time.sleep(2)
                c.send({"type": "status", "secondsInQueue": time.time() - t})
        except OSError:
            pass

    def pair(self, a, b):
        (ca, ma), (cb, mb) = a, b
        pa, pb = (ma["_peer_ip"], int(ma["port"])), (mb["_peer_ip"], int(mb["port"]))
        # relay sockets: rx is what A believes is B, ry what B believes is A
        rx = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        rx.bind((self.bind, 0))
        ry = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        ry.bind((self.bind, 0))
        x, y = rx.getsockname()[1], ry.getsockname()[1]
        log(f"pairing: A netplay {pa[1]} -> relay {x} | B netplay {pb[1]} -> relay {y}")
        threading.Thread(target=self.relay, args=(rx, ry, pa, pb), daemon=True).start()

        def player(m, gport, local, ip_port):
            u = m.get("user", {})
            return {"uid": u.get("uid", ""), "displayName": u.get("displayName", ""),
                    "connectCode": u.get("connectCode", ""), "port": gport, "isLocalPlayer": local,
                    "ipAddress": ip_port, "ipAddressLan": ip_port, "isBot": False}

        mid = f"mode.direct-local-{int(time.time())}"
        stages = self.stages
        ca.send({"type": "match", "matchId": mid, "isHost": True, "stages": stages,
                 "players": [player(ma, 1, True, f"{pa[0]}:{pa[1]}"), player(mb, 2, False, f"{self.bind}:{x}")]})
        cb.send({"type": "match", "matchId": mid, "isHost": False, "stages": stages,
                 "players": [player(ma, 1, False, f"{self.bind}:{y}"), player(mb, 2, True, f"{pb[0]}:{pb[1]}")]})

    def drop(self):
        self.rng = (self.rng * 1103515245 + 12345) & 0x7FFFFFFF
        return (self.rng >> 8) % 100 < self.loss

    def relay(self, rx, ry, pa, pb):
        held = []  # (due, socket, data, dest)
        while True:
            now = time.time()
            while held and held[0][0] <= now:
                _, so, data, dest = held.pop(0)
                so.sendto(data, dest)
            r, _, _ = select.select([rx, ry], [], [], 0.002 if held else 0.5)
            for s in r:
                try:
                    data, src = s.recvfrom(65536)
                except ConnectionResetError:  # Windows: an ICMP port-unreachable from a peer that has gone
                    continue
                if s is rx:  # from A, for B
                    out, dest, d = ry, pb, "A>B"
                else:
                    out, dest, d = rx, pa, "B>A"
                if self.loss and self.drop():
                    d += "x"
                elif self.lag:
                    held.append((time.time() + self.lag, out, data, dest))
                else:
                    out.sendto(data, dest)
                if self.capture:
                    self.capture.write(f"{int((time.time() - self.t0) * 1000)} {d} {data.hex()}\n")
                    self.capture.flush()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=43110)
    ap.add_argument("--capture", default="")
    ap.add_argument("--delay-ms", type=int, default=0)
    ap.add_argument("--stages", default="", help="comma-separated external stage ids for the match (default the 1v1 list)")
    ap.add_argument("--lag-ms", type=int, default=0, help="relay: one-way delay per datagram")
    ap.add_argument("--loss-pct", type=int, default=0, help="relay: drop this % of datagrams")
    ap.add_argument("--bind", default=LOOP, help="listen/relay address (default loopback; a private address for --mm-lan peers)")
    a = ap.parse_args()
    ip = ipaddress.ip_address(a.bind)
    if not (ip.is_loopback or ip.is_private):
        ap.error("--bind must be a loopback or private address")
    stages = [int(x, 0) for x in a.stages.split(",") if x]
    srv = Server(a.port, a.capture, a.delay_ms, stages, a.lag_ms, a.loss_pct)
    srv.bind = a.bind
    srv.serve()


if __name__ == "__main__":
    main()
