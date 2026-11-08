# SPDX-License-Identifier: MIT
"""Dolphin oracle: give the playback the recording console's own widescreen and polling-drift Gecko codes.

    python -I replay_codes.py REPLAY.slp USER_DIR

The recording console of many online replays ran a widescreen code. Its "Left/Right Camera Bound" (0x80030C7C,
0x80030C88) and bubble constants (0x804DDBxx) change the logic's off-screen test (ifMagnify_802FC998), so the 1%
offscreen tick (dmg.x1910 vs x7AC, Fighter_procAnim) fires on another frame. Slippi's playback code set does not
re-apply the replay's codes, so without this the console in Dolphin is not the recording console: the recorded
Post-Frame percent agrees with the engine (slpcheck applies the code, melee_data::slp::widescreen_code) and not with
playback (notes/dolphin-oracle.md, "Widescreen codes").

Reads the replay's Gecko list (Message Splitter 0x10 carrying command 0x3D), copies every code that writes one of the
addresses of Slippi's own "Widescreen 16:9" code verbatim (the replay's values, not Slippi's), and writes
USER_DIR/GameSettings/GALE01.ini with them as one enabled code. With no such code the file is removed. Prints one line.
No disc data is read or written.

The same goes for Slippi Online's "Polling Drift Fix" (0x801A4DA0, 0x801A4DB4, 0x80019860; in Slippi's
list_netplay.json as part of "Required: Slippi Online"). Without it the engine loop is paced by an OS timer at 60 Hz
(lb_80019880(OSSecondsToTicks(1/60)) in gm_801A4BD4) against the 59.94 Hz retrace, so every ~1001 frames the loop runs
two logic frames before one render. The render pass is what sets the fighters' off-camera flag (x221F_b0,
ftLib_UpdateScreenVisibility), so a skipped render moves the 1% offscreen tick by a frame (03b84423 slippi 7892,
21cd149c 3378: render skipped at 7831 and 3317, x7AC frames earlier), and the phase of the 1001-frame cycle depends on
how long playback waited before the match, which made playback nondeterministic (383fcf15). The recording console ran
the fix; the recorded Post-Frame agrees with the engine (notes/dolphin-oracle.md, "Polling drift").
"""
import os
import struct
import sys

# the addresses of Slippi's "$Optional: Widescreen 16:9" (Sys/GameSettings/GALE01r2.ini)
WIDE = {0x803BB05C, 0x8036A4A8, 0x804DDB58, 0x80086B24, 0x80030C7C, 0x80030C88, 0x804DDB30, 0x804DDB34,
        0x804DDB2C, 0x804DDB28, 0x804DDB4C, 0x802FCFC4, 0x804DDB84}
# Slippi Online's "Polling Drift Fix" (Sys/Slippi/InjectionLists/list_console_lag_pd.json)
DRIFT = {0x801A4DA0, 0x801A4DB4, 0x80019860}


def raw_events(d):
    i = d.index(b"raw[$U#l") + 12
    n = struct.unpack(">i", d[i - 4:i])[0]
    raw = d[i:i + n] if n > 0 else d[i:]  # 0: the recorder never closed the raw array (Slippi streams it)
    assert raw[0] == 0x35
    size = {}
    for k in range(2, raw[1] + 1, 3):
        size[raw[k]] = struct.unpack(">H", raw[k + 1:k + 3])[0]
    p = raw[1] + 1
    while p < len(raw):
        c = raw[p]
        if c not in size:
            break
        yield c, raw[p:p + size[c] + 1]
        p += size[c] + 1


def gecko_list(d):
    g = bytearray()
    for c, ev in raw_events(d):
        if c == 0x10 and len(ev) >= 0x205 and ev[0x203] == 0x3D:
            n = min(struct.unpack(">H", ev[0x201:0x203])[0], 512)
            g += ev[1:1 + n]
        elif c in (0x37, 0x38):
            break
    return bytes(g)


def codes(g):
    """(address, [(w0, w1), ...]) per code, the same walk as melee_data::slp::widescreen_code."""
    o = 0
    while o + 8 <= len(g):
        w0, w1 = struct.unpack(">II", g[o:o + 8])
        kind = w0 >> 24
        if kind == 0xF0 or (w0 == 0 and w1 == 0):
            break
        k = kind & 0xFE
        extra = w1 if k in (0xC0, 0xC2, 0xF2, 0xF4) else (w1 + 7) // 8 if k == 0x06 else 1 if k == 0x08 else 0
        total = 8 * (1 + extra)
        if o + total > len(g):
            break
        yield 0x80000000 | (w0 & 0x01FFFFFF), [struct.unpack(">II", g[o + 8 * j:o + 8 * j + 8]) for j in range(1 + extra)]
        o += total


def main():
    slp, user = sys.argv[1], sys.argv[2]
    g = gecko_list(open(slp, "rb").read())
    groups = {"widescreen": [], "polling drift fix": []}
    for addr, words in codes(g):
        name = "widescreen" if addr in WIDE else "polling drift fix" if addr in DRIFT else None
        if name is None:
            continue
        # a C2 code's last word is the branch back; the recording console's code handler had already patched it
        # (a 0x48xxxxxx `b`), Dolphin's handler writes its own into the 00000000 the code is written with
        if (words[0][0] >> 24) & 0xFE == 0xC2 and len(words) > 1 and words[-1][1] >> 26 == 18:
            words[-1] = (words[-1][0], 0)
        groups[name] += [f"{a:08X} {b:08X}" for a, b in words]
    # the standalone fix (3.x: C2 0x801A4DA0, 04 0x801A4DB4, 04 0x80019860) only. Slippi 3.19's online core carries a
    # C2 at 0x801A4DB4 instead, part of the online engine loop: it spins on flags in the core's own block
    # (r13-0x49E4 +0xDF/+0x90D) that the rest of the online core sets, so alone it hangs playback before the match
    # (notes/dolphin-oracle.md, "Round 3").
    if groups["polling drift fix"] and not any(a == 0x801A4DA0 for a, _ in codes(g)):
        print("replay_codes: the replay's 0x801A4DB4/0x80019860 codes are the online core's, not the standalone drift fix; not copied")
        groups["polling drift fix"] = []
    path = os.path.join(user, "GameSettings", "GALE01.ini")
    used = [(n, l) for n, l in groups.items() if l]
    if not used:
        if os.path.exists(path):
            os.remove(path)
        print("replay_codes: no widescreen or polling drift code in the replay")
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", newline="\n") as f:
        f.write("[Gecko]\n")
        for n, l in used:
            f.write(f"$Replay: recording console {n}\n" + "\n".join(l) + "\n")
        f.write("[Gecko_Enabled]\n")
        for n, _ in used:
            f.write(f"$Replay: recording console {n}\n")
    print("replay_codes: " + ", ".join(f"{len(l)} {n} line(s)" for n, l in used) + " from the replay")


if __name__ == "__main__":
    main()
