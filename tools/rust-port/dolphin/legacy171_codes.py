# SPDX-License-Identifier: MIT
"""Dolphin oracle: the Gecko code list a Slippi 1.7.1 recording console ran, for playing a 1.x replay.

    python -I legacy171_codes.py REPLAY.slp OUT.bin [--asm SSBM_ASM_DIR] [--no-spawns] [--qol]

A Slippi 1.x replay carries no Gecko list. Modern Slippi playback then applies its built-in `legacy_code_list`
(SlippiPlayback.cpp: UCF 0.74 toggle handlers, static patches, PS preload, PAL handlers), which is not what a 2019
console ran. This builds the list from the console binaries committed in project-slippi/slippi-ssbm-asm at 496f0ae
("increment version for PS toggle", the 1.7.1 bump), `Output/Console/g_*.bin` (read with `git show`, not copied
into this repo). The patched oracle Dolphin plays OUT.bin in place of the built-in list when
MELEE_DOLPHIN_LEGACY_CODES=OUT.bin (notes/dolphin-oracle.md, "Slippi 1.7.1 replays").

What the console loaded (Slippi Nintendont 1.7 settings; the replay decides where it can):
  g_core.bin     always. Every code except the recording transport, see RECORDING and PLAYBACK_OWNS below.
  g_ucf.bin      the "UCF" controller-fix setting: UCF 0.73 for every port (Enable UCF Toggles writes 0x01010101
                 to ControllerFixOptions, rtoc-0xDD8). The recorded UCF toggle words must all be 1; otherwise the
                 console ran g_toggles.bin (per-player UCF/Arduino) and this script refuses.
  g_spawns.bin   20XX Neutral Spawns. The engine assumes it for every 1.x replay (gmvs.rs legacy_neutral_spawn);
                 --no-spawns leaves it out.
  g_pal.bin      only when the replay's PAL byte is set.
  g_qol.bin      only with --qol (Salty Runback, Disable FoD During Doubles, CSS/nametag cosmetics): the replay
                 does not say whether it was on.
Codes left out of g_core, and why (all are verbatim otherwise; nothing is edited):
  RECORDING      the five `bl` hooks into SlippiRecording and its 06 block at 0x8032ED94: they only send EXI
                 data to the Slippi device; 0x8006B0DC and 0x8016E74C are the sites of playback's own hooks.
  PLAYBACK_OWNS  sites the playback code set (Sys/GameSettings/GALE01r2.ini) already patches with Slippi's own
                 copy of the same code: Init Player Data 0x80068EEC and Init Stage Data 0x801C154C (the same
                 zeroing; the modern stage init also resets Dream Land wind / Japes logs, which a 2019 console
                 did not), Disable Special/Trophy Messages 0x8015D94C/0x8015D984 (identical words).
Prints one line.
"""
import argparse
import os
import struct
import subprocess
import sys

COMMIT = "496f0ae"
RECORDING = {(0x04, 0x8016E74C), (0x04, 0x8006B0DC), (0x04, 0x8006C5D4), (0x04, 0x801A5B14), (0x04, 0x802FED3C),
             (0x06, 0x8032ED94)}
PLAYBACK_OWNS = {(0xC2, 0x80068EEC), (0xC2, 0x801C154C), (0x04, 0x8015D94C), (0x04, 0x8015D984)}


def raw_events(d):
    i = d.index(b"raw[$U#l") + 12
    n = struct.unpack(">i", d[i - 4:i])[0]
    raw = d[i:i + n]
    assert raw[0] == 0x35
    size = {raw[k]: struct.unpack(">H", raw[k + 1:k + 3])[0] for k in range(2, raw[1] + 1, 3)}
    p = raw[1] + 1
    while p < len(raw):
        c = raw[p]
        if c not in size:
            break
        yield c, raw[p:p + size[c] + 1]
        p += size[c] + 1


def codes(g):
    """[(kind, address, bytes)] for a raw Gecko list (04/06/C2 and the rest by length)."""
    out, o = [], 0
    while o + 8 <= len(g):
        w0, w1 = struct.unpack(">II", g[o:o + 8])
        k = (w0 >> 24) & 0xFE
        if w0 == 0xFF000000 or k == 0xF0 or (w0 == 0 and w1 == 0):
            break
        extra = w1 if k in (0xC0, 0xC2) else (w1 + 7) // 8 if k == 0x06 else 1 if k == 0x08 else 0
        out.append((k, 0x80000000 | (w0 & 0x01FFFFFF), g[o:o + 8 * (1 + extra)]))
        o += 8 * (1 + extra)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("slp")
    ap.add_argument("out")
    ap.add_argument("--asm", default=os.path.join(os.environ.get("GW_WORKSPACE", r"E:\Projects\Melee Workspace"),
                                                  "_build", "slippi-src", "ssbm-asm"))
    ap.add_argument("--no-spawns", action="store_true")
    ap.add_argument("--qol", action="store_true")
    a = ap.parse_args()

    d = open(a.slp, "rb").read()
    gs, has_gecko = None, False
    for c, ev in raw_events(d):
        if c == 0x36 and gs is None:
            gs = ev
        elif c == 0x10:
            has_gecko = True
        elif c in (0x37, 0x38):
            break
    ver = tuple(gs[1:5])
    if ver[0] >= 2 or has_gecko:
        if os.path.exists(a.out):
            os.remove(a.out)
        print(f"legacy171: replay {'.'.join(map(str, ver[:3]))} has its own code list; not used")
        return 1
    ucf = struct.unpack(">8I", gs[0x141:0x161])
    pal = gs[0x1A1] if len(gs) > 0x1A1 else 0
    if set(ucf) != {1}:
        sys.exit(f"legacy171: UCF toggle words {ucf} are not all 1: the console ran g_toggles.bin, not handled")

    def bin_(name):
        return subprocess.run(["git", "-C", a.asm, "show", f"{COMMIT}:Output/Console/{name}"], check=True,
                              capture_output=True).stdout

    parts = ["g_core.bin", "g_ucf.bin"] + ([] if a.no_spawns else ["g_spawns.bin"]) + (["g_pal.bin"] if pal else []) \
        + (["g_qol.bin"] if a.qol else [])
    out, n, dropped = bytearray(), 0, 0
    for p in parts:
        for k, addr, body in codes(bin_(p)):
            if p == "g_core.bin" and ((k, addr) in RECORDING or (k, addr) in PLAYBACK_OWNS):
                dropped += 1
                continue
            if k not in (0x04, 0x06, 0xC2):
                sys.exit(f"legacy171: code type {k:02X} at {addr:08X} in {p}: playback applies only 04/06/C2")
            out += body
            n += 1
    out += bytes([0xFF] + [0] * 7)
    with open(a.out, "wb") as f:
        f.write(out)
    print(f"legacy171: {n} codes from {' '.join(parts)} @ {COMMIT} ({dropped} recording/playback-owned left out), "
          f"{len(out)} bytes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
