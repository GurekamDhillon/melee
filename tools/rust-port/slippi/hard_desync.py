#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Would Slippi have ended this game? Compares the engine's Slippi desync checksum (`melee slpcheck --full
--slippi-checksum-out ENG.txt`) with the same checksum's low half rebuilt from the replay's own Post-Frame data.

Slippi's hard test (Online/Core/StartEngineLoop.asm FN_COMPUTE_CHECKSUM, melee_core::slippi_online::compare): the low
16 bits are fctiwz of the f32 sum of every present fighter's x, y and percent (slot order, main entity then sub entity);
two sides whose low halves differ by more than 1 (as s16) end the game. The high half needs the spawn number, which a
replay does not record, so only the hard test is rebuilt here.

    python -I hard_desync.py REPLAY.slp ENG.txt      -> one line: frames,first_hard,max_delta,version,frames_other_set (first_hard 'old' below Slippi 3.0)
"""
import struct
import sys

sys.path.insert(0, __import__("os").path.dirname(__file__))
from slp_rollbacks import events  # noqa: E402


def f32(v):
    return struct.unpack("<f", struct.pack("<f", v))[0]


def recorded_lo(path):
    d = open(path, "rb").read()
    k = d.find(b"raw[$U#l")
    ln = struct.unpack(">I", d[k + 8:k + 12])[0]
    frames = {}
    global VERSION, FINAL
    final = None
    for c, p in events(d[k + 12:k + 12 + ln]):
        if c == 0x36:
            VERSION = "%d.%d.%d" % (p[0], p[1], p[2])
        if c == 0x3C and len(p) >= 8:
            # Frame Bookend (3.7.0+): the latest finalized frame. Frames past the highest one were never confirmed
            # (a game that ends while a rollback is open keeps only their predicted copies)
            lf = struct.unpack(">i", p[4:8])[0]
            final = lf if final is None else max(final, lf)
        if c != 0x38:
            continue
        f = struct.unpack(">i", p[0:4])[0]
        port, follower = p[4], p[5]
        act = struct.unpack(">H", p[7:9])[0]
        x, y, _fc, pct = struct.unpack(">ffff", p[9:25])
        # the last copy of a frame wins (a rolled-back frame is recorded more than once)
        frames.setdefault(f, {})[(port, follower)] = (x, y, pct, act)
    FINAL = final
    global GAP
    fs = sorted(frames)
    GAP = max((b - a - 1 for a, b in zip(fs, fs[1:])), default=0)
    out = {}
    for f, fighters in frames.items():
        if final is not None and f > final:
            continue
        # DeadUp* (actions 3-10, the star / screen KO): the draw callback moves the fighter after the Post-Frame point
        # the replay records, and the checksum is taken after the frame's render (engine: its logic-side stand-in), so
        # those frames are not comparable here
        if any(3 <= v[3] <= 10 for v in fighters.values()):
            continue
        s = 0.0
        mask = 0
        for port, fol in fighters:
            mask |= 1 << (port * 2 + fol)
        for key in sorted(fighters):
            x, y, pct, _act = fighters[key]
            s = f32(s + x)
            s = f32(s + y)
            s = f32(s + pct)
        if s != s:
            v = -0x80000000
        else:
            v = int(max(min(s, 2147483647.0), -2147483648.0))
        out[f] = (v & 0xFFFF, mask)
    return out


# Slippi before 3.0 took Post-Frame at Fighter_procMap, partway through the frame: some actions move the fighter after
# it, so those replays do not hold the end-of-frame state the checksum is taken from. They are reported as "old".
VERSION = "?"
FINAL = None
GAP = 0  # the largest run of frames missing from the replay (a damaged file: not scored)


def s16(v):
    return v - 0x10000 if v & 0x8000 else v


def main():
    try:
        rec = recorded_lo(sys.argv[1])
    except (IndexError, struct.error, ValueError):
        print("0,error,0,?,0")  # unreadable replay (e.g. an empty raw block)
        return
    frames = 0
    first = ""
    worst = 0
    other_set = 0
    for line in open(sys.argv[2], encoding="ascii", errors="replace"):
        parts = line.split()
        if len(parts) < 2:
            continue
        # column 3: the checksum over the recorded fighters only (no inactive Zelda/Sheik form); column 4: which
        # fighters it covered. A frame where the replay records a different set is not comparable (counted apart).
        f, ck = int(parts[0]), int(parts[2], 16)
        r = rec.get(f)
        if r is None:
            continue
        r, rmask = r
        if len(parts) > 3 and int(parts[3], 16) != rmask:
            # columns 5/6: the same without Nana, for frames after a Nana death (the replay leaves her out)
            if len(parts) > 5 and int(parts[5], 16) == rmask:
                ck = int(parts[4], 16)
            else:
                other_set += 1
                continue
        frames += 1
        dlt = abs(s16(ck & 0xFFFF) - s16(r))
        worst = max(worst, dlt)
        if dlt > 1 and not first:
            first = str(f)
    if VERSION[0].isdigit() and int(VERSION.split(".")[0]) < 3:
        first = "old"
    elif GAP > 0:
        first = "gap"
    print(f"{frames},{first},{worst},{VERSION},{other_set}")


if __name__ == "__main__":
    main()
