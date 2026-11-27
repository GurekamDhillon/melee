#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Would Slippi have ended this game? Compares the engine's Slippi desync checksum (`melee slpcheck --full
--slippi-checksum-out ENG.txt`) with the same checksum's low half rebuilt from the replay's own Post-Frame data.

Slippi's hard test (Online/Core/StartEngineLoop.asm FN_COMPUTE_CHECKSUM, melee_core::slippi_online::compare): the low
16 bits are fctiwz of the f32 sum of every present fighter's x, y and percent (slot order, main entity then sub entity);
two sides whose low halves differ by more than 1 (as s16) end the game. The high half needs the spawn number, which a
replay does not record, so only the hard test is rebuilt here.

    python -I hard_desync.py REPLAY.slp ENG.txt      -> one line: frames,first_hard,max_delta
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
    for c, p in events(d[k + 12:k + 12 + ln]):
        if c != 0x38:
            continue
        f = struct.unpack(">i", p[0:4])[0]
        port, follower = p[4], p[5]
        act = struct.unpack(">H", p[7:9])[0]
        x, y, _fc, pct = struct.unpack(">ffff", p[9:25])
        # the last copy of a frame wins (a rolled-back frame is recorded more than once)
        frames.setdefault(f, {})[(port, follower)] = (x, y, pct, act)
    out = {}
    for f, fighters in frames.items():
        # DeadUp* (actions 3-10, the star / screen KO): the draw callback moves the fighter after the Post-Frame point
        # the replay records, and the checksum is taken after the frame's render (engine: its logic-side stand-in), so
        # those frames are not comparable here
        if any(3 <= v[3] <= 10 for v in fighters.values()):
            continue
        s = 0.0
        for key in sorted(fighters):
            x, y, pct, _act = fighters[key]
            s = f32(s + x)
            s = f32(s + y)
            s = f32(s + pct)
        if s != s:
            v = -0x80000000
        else:
            v = int(max(min(s, 2147483647.0), -2147483648.0))
        out[f] = v & 0xFFFF
    return out


def s16(v):
    return v - 0x10000 if v & 0x8000 else v


def main():
    rec = recorded_lo(sys.argv[1])
    frames = 0
    first = ""
    worst = 0
    for line in open(sys.argv[2], encoding="ascii", errors="replace"):
        parts = line.split()
        if len(parts) < 2:
            continue
        # column 3 (when present): the checksum over the recorded fighters only (no inactive Zelda/Sheik form)
        f, ck = int(parts[0]), int(parts[-1], 16)
        r = rec.get(f)
        if r is None:
            continue
        frames += 1
        dlt = abs(s16(ck & 0xFFFF) - s16(r))
        worst = max(worst, dlt)
        if dlt > 1 and not first:
            first = str(f)
    print(f"{frames},{first},{worst}")


if __name__ == "__main__":
    main()
