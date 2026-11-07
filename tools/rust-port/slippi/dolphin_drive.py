#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Drive one local-test Slippi Dolphin (run_np_dolphin.ps1) through its pad pipe, closed loop on its state log.

No OS input: buttons go to the named pipe \\\\.\\pipe\\<Name>slippibot1 that Dolphin's pipe controller reads
(SLIPPI_PIPE_PREFIX). Progress is read back from MELEE_DOLPHIN_STATELOG (one line per frame: frame, the scene word at
0x80479D30, then the words of MELEE_DOLPHIN_STATELOG_ADDRS, which must be ADDRS below).

    python -I dolphin_drive.py --name A --state OUT/state_A.txt --char 2 [--pads padgen:SEED:PORT] [--frames N]

Steps: main menu -> 1P menu (Slippi's Online Play) -> Online submenu -> Direct -> online CSS -> the character
(cursor over its icon, A) -> START -> code entry (autocompleted from Slippi/direct-codes.json, else typed) -> START
(confirm) -> search; once the opponent is connected the game starts. In game, --pads feeds the session pad generator
(the same rb_gen as `melee slippi-netplay --padgen`) for this port, one pad per frame as the state log advances.
"""
import argparse
import struct
import sys
import time

# words logged per frame (after frame and scene): MenuFlow (cur_menu, prev_menu, hovered_selection),
# r13-0x5060 (online mode, app state), r13-0x49AC (CSS: door count, scene request, character selected), the global
# frame counter, HSD_PadMasterStatus[0], mnCharSel_804A0BC0[0] (port 1's CSSCursorData), and that cursor's x and y
# (+0xC, +0x10). The cursor is heap-allocated at the same address on every boot of this build (deterministic), so
# its fields can be logged by address; css_goto needs them as the last two words.
ADDRS = "804a04f0,804d6640,804d6cf4,80479d60,804c1fac,804a0bc0,81155a8c,81155a90"


def log(m):
    print(f"drive: {m}", flush=True)


class Dolphin:
    def __init__(self, name, state):
        self.pipe_name = rf"\\.\pipe\{name}slippibot1"
        self.state = state
        self.pipe = None
        self.fh = None
        self.last = None
        self.buf = ""

    def open(self, timeout=60):
        t = time.time()
        while time.time() - t < timeout:
            try:
                self.pipe = open(self.pipe_name, "wb", buffering=0)
                break
            except OSError:
                time.sleep(0.25)
        if not self.pipe:
            raise SystemExit("drive: pipe never appeared")
        while time.time() - t < timeout:
            try:
                self.fh = open(self.state)
                return
            except OSError:
                time.sleep(0.25)
        raise SystemExit("drive: no state log")

    def cmd(self, s):
        # bytes, LF only: a text-mode pipe writes CRLF, and Dolphin's pipe parser then drops every command
        self.pipe.write((s + "\n").encode())
        self.pipe.flush()

    def poll(self):
        """Newest state line as a list of ints (frame, scene, *ADDRS)."""
        d = self.fh.read()
        if d:
            self.buf += d
            lines = self.buf.split("\n")
            self.buf = lines[-1]
            for ln in reversed(lines[:-1]):
                p = ln.split()
                if len(p) >= 2:
                    self.last = [int(p[0])] + [int(x, 16) for x in p[1:]]
                    break
        return self.last

    def frames(self, n):
        s = self.poll()
        start = s[0] if s else 0
        t = time.time()
        while time.time() - t < 30:
            s = self.poll()
            if s and s[0] >= start + n:
                return s
            time.sleep(0.004)
        raise SystemExit("drive: Dolphin stopped advancing")

    def tap(self, btn, hold=3, after=8):
        self.cmd(f"PRESS {btn}")
        self.frames(hold)
        self.cmd(f"RELEASE {btn}")
        self.frames(after)

    def until(self, pred, what, timeout=60):
        t = time.time()
        while time.time() - t < timeout:
            s = self.poll()
            if s and pred(s):
                return s
            time.sleep(0.01)
        raise SystemExit(f"drive: timed out waiting for {what} (state {self.fmt()})")

    def fmt(self):
        s = self.last
        return "none" if not s else " ".join([str(s[0])] + [f"{x:08x}" for x in s[1:]])


def major(s):
    return s[1] >> 24


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--name", required=True)
    ap.add_argument("--state", required=True)
    ap.add_argument("--explore", default="", help="space-separated taps (A, B, START, D_DOWN...) then report")
    ap.add_argument("--play", default="", help="X,Y: the full flow, picking the character under the CSS hand at X,Y")
    ap.add_argument("--pads", type=int, default=1, help="seed of the scripted in-game pads")
    ap.add_argument("--frames", type=int, default=3600, help="in-game frames to play (the game may end first)")
    ap.add_argument("--timeout", type=int, default=600)
    a = ap.parse_args()
    d = Dolphin(a.name, a.state)
    d.open()
    d.until(lambda s: major(s) in (1, 8), "main menu")
    d.frames(30)
    log(f"main menu: {d.fmt()}")
    if a.play:
        play(d, a)
        return
    for t in a.explore.split():
        if t == "direct":
            to_direct(d)
        elif t.startswith("w"):
            d.frames(int(t[1:]))
        elif t.startswith("G:"):  # G:x,y  steer the CSS hand there (needs the cursor x/y words as the last two)
            x, y = map(float, t[2:].split(","))
            n = len(d.poll())
            css_goto(d, x, y, n - 2, n - 1)
        elif t.startswith("S:"):  # S:x,y:N  hold the main stick at (x, y) for N frames, then centre it
            _, xy, n = t.split(":")
            d.cmd("SET MAIN " + xy.replace(",", " "))
            d.frames(int(n))
            d.cmd("SET MAIN 0.5 0.5")
            d.frames(4)
        else:
            d.tap(t)
        log(f"after {t}: {d.fmt()}")


def f32(w):
    return struct.unpack(">f", struct.pack(">I", w))[0]


def css_goto(d, x, y, xi, yi):
    """Steer the CSS hand (CSSCursorData x at +0xC, y at +0x10; their words are columns xi, yi) to (x, y)."""
    for _ in range(400):
        s = d.frames(1)
        cx, cy = f32(s[xi]), f32(s[yi])
        ex, ey = x - cx, y - cy
        if abs(ex) < 0.6 and abs(ey) < 0.6:
            break
        k = lambda e: 0.5 + max(-0.5, min(0.5, e / 8.0))
        d.cmd(f"SET MAIN {k(ex):.3f} {k(ey):.3f}")
    d.cmd("SET MAIN 0.5 0.5")
    d.frames(4)


def in_game(s):
    # SCENE_ONLINE_IN_GAME: major 8, minor 2 (the scene word is cur major, pending major, prev major, minor)
    return major(s) == 8 and (s[1] & 0xFF) == 2


def play(d, a):
    to_direct(d)
    x, y = map(float, a.play.split(","))
    n = len(d.poll())
    css_goto(d, x, y, n - 2, n - 1)
    d.tap("A", after=20)
    if not d.poll()[4] & 0xFF:
        raise SystemExit(f"drive: no character under the hand at {a.play}")
    # START: lock in and open the code entry; Z takes the autocomplete suggestion (Slippi/direct-codes.json of
    # this throwaway user dir); START and A confirm; then the search runs
    d.tap("START", after=60)
    d.tap("Z", after=20)
    d.tap("START", after=20)
    d.tap("A", after=30)
    log(f"searching: {d.fmt()}")
    d.until(in_game, "the online game", timeout=a.timeout)
    s = d.poll()
    log(f"in game at log frame {s[0]}: {d.fmt()}")
    # scripted pads: a new random pad every 3-12 frames, from --pads (a plain LCG, so a run is repeatable)
    rng = a.pads * 2654435761 & 0xFFFFFFFF
    def r(n):
        nonlocal rng
        rng = (rng * 1103515245 + 12345) & 0x7FFFFFFF
        return (rng >> 8) % n
    buttons = ["A", "B", "X", "Z", "L", "R"]
    start = s[0]
    held = set()
    while True:
        s = d.poll()
        if not in_game(s):
            log(f"left the game at log frame {s[0]} ({s[0] - start} frames in): {d.fmt()}")
            break
        if s[0] - start >= a.frames:
            log(f"played {a.frames} frames")
            break
        for b in list(held):
            d.cmd(f"RELEASE {b}")
        held.clear()
        if r(3):
            b = buttons[r(len(buttons))]
            if b != "L" or r(4) == 0:  # few shields
                d.cmd(f"PRESS {b}")
                held.add(b)
        d.cmd(f"SET MAIN {r(101) / 100:.2f} {r(101) / 100:.2f}")
        d.cmd(f"SET C {0.5 if r(4) else r(101) / 100:.2f} 0.5")
        d.frames(3 + r(10))
    for b in held:
        d.cmd(f"RELEASE {b}")
    d.cmd("SET MAIN 0.5 0.5")


def to_direct(d):
    """Online submenu (MenuFlow.cur_menu 8) -> Direct (hovered 2) -> the online CSS in direct mode."""
    s = d.until(lambda s: (s[2] >> 24) == 8 and (s[3] >> 16) & 0xFF == 1, "online submenu, logged in")
    for _ in range(10):
        h = s[2] & 0xFFFF
        if h == 2:
            break
        d.tap("D_DOWN" if h < 2 else "D_UP", after=10)
        s = d.poll()
    d.tap("A")
    d.until(lambda s: major(s) == 8 and (s[3] >> 24) == 2, "online CSS, direct mode")
    d.frames(30)


if __name__ == "__main__":
    main()
