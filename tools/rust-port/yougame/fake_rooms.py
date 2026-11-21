#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""A local fake of YouGame's rooms service and of the YouGame app's stdio bridge, for the online menu's tests.

Loopback only: the server binds 127.0.0.1; nothing talks to YouGame. The wire is the one our rooms client speaks
(rust/melee_net/src/yougame_room.rs, after Hero's yougame_room.h / find_match):
  GET /match?ticket=T      queue: `searching{waiting}`; the second searcher pairs with the first: both get
                           `matched{room}` (seats in queue order). `cancel` leaves the queue.
  GET /room/<code>?ticket=T room: `welcome{me, participants, host, revision}` to the joiner, `join{...}` to the room;
                           `msg{to,d}` is relayed as `msg{from,d}`; `begin` -> `start{matchId}` to all; `finish` ->
                           `result{matchId}` to all; `leave` -> `leave{...}` to the rest. A ticket names its user
                           (`<user>-...`): a rejoin takes the same user's seat.

As a library (`Rooms(port).start()`), or standalone: python -I fake_rooms.py --port P [--log FILE].
`App` plays the YouGame app on a game's stdio: it answers `mpTicket` with this server's URL and a ticket for its
user, and refuses everything else (saves etc.), as an app with no account features would.
"""
import argparse
import json
import socket
import threading
import time
import urllib.parse
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "slippi"))
from mock_mm import Conn  # noqa: E402  (the same minimal WebSocket server side)

LOOP = "127.0.0.1"


class Rooms:
    def __init__(self, port, logf=None):
        self.port = port
        self.lock = threading.Lock()
        self.waiting = []  # [(conn, user)]
        self.rooms = {}  # code -> {"seats": [user...], "conns": {user: (conn, cid)}, "rev": n, "match": str, "n": k}
        self.next_room = 0
        self.next_conn = 0
        self.logf = logf
        self.events = []

    def log(self, msg):
        line = f"fake_rooms: {msg}"
        self.events.append(line)
        if self.logf:
            self.logf.write(line + "\n")
            self.logf.flush()

    def start(self):
        ls = socket.socket()
        ls.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        ls.bind((LOOP, self.port))
        ls.listen(8)
        self.log(f"listening on ws://{LOOP}:{self.port}")
        threading.Thread(target=self._accept, args=(ls,), daemon=True).start()
        return self

    def _accept(self, ls):
        while True:
            s, _ = ls.accept()
            threading.Thread(target=self._client, args=(s,), daemon=True).start()

    def _roster(self, code):
        r = self.rooms[code]
        ps = []
        for slot, user in enumerate(r["seats"]):
            if user in r["conns"]:
                ps.append({"id": "p-" + user, "connectionId": r["conns"][user][1], "slot": slot, "localIndex": 0})
        host = ps[0]["connectionId"] if ps else ""
        return {"participants": ps, "host": host, "revision": r["rev"]}

    def _to_room(self, code, obj, but=None):
        for user, (c, _) in list(self.rooms[code]["conns"].items()):
            if user != but:
                try:
                    c.send(obj)
                except OSError:
                    pass

    def _client(self, s):
        c = Conn(s)
        user, code, cid = "", None, ""
        try:
            # the request line names the path before the handshake answer
            while b"\r\n\r\n" not in c.buf:
                d = s.recv(4096)
                if not d:
                    return
                c.buf += d
            path = c.buf.split(b" ", 2)[1].decode("latin1")
            c.handshake()
            u = urllib.parse.urlsplit(path)
            ticket = urllib.parse.parse_qs(u.query).get("ticket", [""])[0]
            user = ticket.split("-", 1)[0]
            with self.lock:
                self.next_conn += 1
                cid = f"c{self.next_conn}"
                if u.path == "/match":
                    c.send({"t": "searching", "waiting": len(self.waiting) + 1})
                    self.log(f"queue: {user} searching ({len(self.waiting) + 1} waiting)")
                    other = next((w for w in self.waiting if w[1] != user), None)
                    if other:
                        self.waiting.remove(other)
                        self.next_room += 1
                        code = f"room{self.next_room}"
                        self.rooms[code] = {"seats": [other[1], user], "conns": {}, "rev": 1, "match": "", "n": 0}
                        for w in (other[0], c):
                            w.send({"t": "matched", "room": code})
                        self.log(f"queue: matched {other[1]} + {user} into {code}")
                        code = None
                    else:
                        self.waiting.append((c, user))
                elif u.path.startswith("/room/"):
                    code = urllib.parse.unquote(u.path[6:])
                    r = self.rooms.get(code)
                    if r is None or user not in r["seats"]:
                        self.log(f"room {code}: refused {user}")
                        return
                    r["conns"][user] = (c, cid)
                    r["rev"] += 1
                    ro = self._roster(code)
                    c.send(dict(t="welcome", me=cid, **ro))
                    self._to_room(code, dict(t="join", **ro), but=user)
                    self.log(f"room {code}: {user} joined as {cid} (revision {r['rev']})")
                else:
                    return
            while True:
                op, data = c.recv()
                if op == 8:
                    break
                if op == 9:
                    c.send(data, opcode=10)
                    continue
                if op != 1:
                    continue
                m = json.loads(data)
                t = m.get("t")
                with self.lock:
                    if code is None:
                        if t == "cancel":
                            self.waiting = [w for w in self.waiting if w[0] is not c]
                            self.log(f"queue: {user} cancelled")
                        continue
                    r = self.rooms[code]
                    if t == "msg":
                        to = m.get("to")
                        for u2, (c2, id2) in r["conns"].items():
                            if id2 == to:
                                c2.send({"t": "msg", "from": cid, "d": m.get("d")})
                        if (m.get("d") or {}).get("kind") == "mu-cand":
                            self.log(f"room {code}: {user} sent its candidate")
                    elif t == "begin":
                        r["n"] += 1
                        r["match"] = f"{code}-m{r['n']}"
                        self._to_room(code, {"t": "start", "matchId": r["match"]})
                        self.log(f"room {code}: begin by {user} -> start {r['match']}")
                    elif t == "finish":
                        self._to_room(code, {"t": "result", "matchId": r["match"]})
                        self.log(f"room {code}: finish by {user}: {json.dumps(m)}")
                    elif t == "leave":
                        self.log(f"room {code}: {user} left")
                        break
        except (EOFError, OSError, ValueError):
            pass
        finally:
            with self.lock:
                self.waiting = [w for w in self.waiting if w[0] is not c]
                if code and code in self.rooms and self.rooms[code]["conns"].get(user, (None,))[0] is c:
                    del self.rooms[code]["conns"][user]
                    self.rooms[code]["rev"] += 1
                    self._to_room(code, dict(t="leave", **self._roster(code)))
            try:
                s.close()
            except OSError:
                pass


class Stun:
    """A loopback STUN server (RFC 5389 binding): answers XOR-MAPPED-ADDRESS with the request's source."""

    def __init__(self, port, logf=None):
        self.port, self.logf, self.answered = port, logf, 0

    def start(self):
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.bind((LOOP, self.port))
        threading.Thread(target=self._run, args=(s,), daemon=True).start()
        return self

    def _run(self, s):
        import struct
        while True:
            d, (ip, port) = s.recvfrom(512)
            if len(d) < 20 or d[:2] != bytes([0, 1]) or d[4:8] != bytes([0x21, 0x12, 0xA4, 0x42]):
                continue
            xip = bytes(a ^ b for a, b in zip(socket.inet_aton(ip), d[4:8]))
            attr = struct.pack(">HHBBH", 0x20, 8, 0, 1, port ^ 0x2112) + xip
            s.sendto(bytes([1, 1]) + struct.pack(">H", len(attr)) + d[4:20] + attr, (ip, port))
            self.answered += 1
            if self.logf:
                self.logf.write("fake_stun: answered a binding request" + chr(10))
                self.logf.flush()


class App:
    """The YouGame app's side of one game's stdio bridge."""

    def __init__(self, proc, user, rooms_port, logf):
        self.proc, self.user, self.port, self.logf = proc, user, rooms_port, logf
        self.n = 0
        self.tickets = 0
        threading.Thread(target=self._read, daemon=True).start()

    def _read(self):
        for raw in self.proc.stdout:
            line = raw.decode("utf-8", "replace").rstrip("\r\n")
            self.logf.write(f"app<{self.user}: {line}\n")
            self.logf.flush()
            if not line.startswith("yg "):
                continue
            try:
                q = json.loads(line[3:])
            except ValueError:
                continue
            if "id" not in q:
                continue
            if q.get("type") == "mpTicket":
                self.tickets += 1
                a = {"re": q["id"], "ok": True, "result": {"url": f"ws://{LOOP}:{self.port}", "ticket": f"{self.user}-t{self.tickets}"}}
            else:
                a = {"re": q["id"], "ok": False, "error": "not offered by the fake app"}
            try:
                self.proc.stdin.write(("yg " + json.dumps(a) + "\n").encode())
                self.proc.stdin.flush()
            except OSError:
                return


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=43190)
    ap.add_argument("--log")
    a = ap.parse_args()
    Rooms(a.port, open(a.log, "w") if a.log else sys.stdout).start()
    while True:
        time.sleep(3600)
