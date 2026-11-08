#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Dolphin oracle: turn the patched Slippi Dolphin's memory records (.nhraw, MELEE_DOLPHIN_NHV1) into an NHV1 CSV.

    python tools/rust-port/dolphin/raw2csv.py dolphin.nhraw out.csv

The words and the hash are docs/specs/tools/nhv1_ref.py's (fighter_inputs, fold, item_word, game_word); this file only
reads the GameCube fields at the offsets of nhv1.md 5.3 / 5.4. frame = slippi frame + 123 (nhv1.md 3, under a replay).
The CPU block (5.3 words 22-47) is added when ftCo_IsCpuControlled would answer true: Player_8003248C is CPU
(the slot's pkind is 1, or the fighter is a sub fighter of a character without a transformation, i.e. Nana) and cpu.kind != 5.
"""
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "docs/specs/tools"))
import nhv1_ref as N  # noqa: E402

FIGHTER, ITEM, CAMERA, STAGE = 0x23EC, 0xFCC, 0x39C, 0x748


def f32(b, o):
    return struct.unpack_from(">f", b, o)[0]


def u32(b, o):
    return struct.unpack_from(">I", b, o)[0]


def s32(b, o):
    return struct.unpack_from(">i", b, o)[0]


def fighter_fields(b, stocks):
    """struct Fighter (GC layout) -> the dict nhv1_ref.fighter_inputs takes (nhv1.md 5.3)."""
    return {
        "motion_id": s32(b, 0x10),
        "pos": (f32(b, 0xB0), f32(b, 0xB4), f32(b, 0xB8)),
        "vel": (f32(b, 0x80), f32(b, 0x84)),
        "percent": f32(b, 0x1830),
        "facing": f32(b, 0x2C),
        "ground_or_air": s32(b, 0xE0),
        "kb": (f32(b, 0x8C), f32(b, 0x90)),
        "gr_vel": f32(b, 0xEC),
        "hitlag": f32(b, 0x195C),
        "b5": (b[0x2219] >> (7 - 5)) & 1,  # x2219_b5, MSB = bit 0
        "shield": f32(b, 0x1998),
        "cmd_frame": f32(b, 0x3E8),
        "jumps": b[0x1968],
        "x67F": b[0x67F],
        "x680": b[0x680],
        "stocks": stocks,
    }


CPU = 0x1A88  # struct CpuFighter inside struct Fighter


def cpu_fields(b, fp_addr):
    c = CPU
    s8 = lambda o: struct.unpack_from(">b", b, c + o)[0]
    word = u32(b, c + 0xF8)
    flags, sh = [], 32
    for w in N.FLAG_WIDTHS:  # split the big-endian word back into the declared bitfields, MSB first
        sh -= w
        flags.append((word >> sh) & ((1 << w) - 1))
    csp, buf = u32(b, c + 0x450), fp_addr + c + 0x454
    return {
        "buttons": u32(b, c), "lstick": (s8(4), s8(5)), "cstick": (s8(6), s8(7)),
        "ltrigger": b[c + 8], "rtrigger": b[c + 9], "kind": u32(b, c + 0xC), "level": s32(b, c + 0x10),
        "x14_34": [s32(b, c + o) for o in range(0x14, 0x38, 4)],
        "x38_40": [f32(b, c + o) for o in (0x38, 0x3C, 0x40)],
        "x7C_84": [s32(b, c + o) for o in (0x7C, 0x80, 0x84)],
        "xC8": b[c + 0xC8], "xEC": b[c + 0xEC], "command_duration": u32(b, c + 0x44C),
        "cs_offset": (csp - buf) if buf <= csp <= buf + 0x100 else 0, "flags": flags,
    }


def is_cpu(b, pkind):
    sub = (b[0x221F] >> (7 - 4)) & 1  # is_sub_fighter, fp+221F:4
    # Player_8003248C answers CPU for a sub fighter only when the character has no transformation
    # (ftMapping_list[ckind].has_transformation: Zelda/Sheik's inactive form is a sub fighter, not a CPU)
    transforms = s32(b, 0x4) in (0x07, 0x13)  # Fighter.kind: Ft_Kind_Seak, Ft_Kind_Zelda
    return (pkind == 1 or (sub == 1 and not transforms)) and u32(b, CPU + 0xC) != 5


def records(data):
    assert data[:8] == b"NHRAW001", "not an NHRAW001 file"
    wide = struct.unpack_from("<I", data, 8)[0] & 1
    p = 12
    while p + 24 <= len(data):
        magic, frame, unk0, seed, nf, ni = struct.unpack_from("<IiIIII", data, p)
        assert magic == 0x4D524652, f"bad record at {p}"
        p += 24
        fts = []
        for _ in range(nf):
            slot, pkind, stocks, _t, fp = struct.unpack_from("<BBbBI", data, p)
            p += 8
            fts.append((slot, pkind, stocks, fp, data[p:p + FIGHTER]))
            p += FIGHTER
        its = []
        for _ in range(ni):
            p += 4
            its.append(data[p:p + ITEM])
            p += ITEM
        if wide:
            p += CAMERA + STAGE
        if p > len(data):
            # the last record is incomplete: playback was killed mid-write (a replay without Game End, whose
            # playback waits for the frame after the last one, ends on run_dolphin's timeout)
            print(f"raw2csv: dropped an incomplete last record (frame {frame})", file=sys.stderr)
            return
        yield frame, unk0, seed, fts, its


def main():
    src, dst = sys.argv[1], sys.argv[2]
    data = Path(src).read_bytes()
    rows = ["frame,hash,seed,items,itemsum," + ",".join(f"s{i}" for i in range(12))]
    for frame, _unk0, seed, fts, its in records(data):
        slots = [0] * 12
        for slot, pkind, stocks, fp, b in fts:
            words = N.fighter_inputs(fighter_fields(b, stocks), slot)
            if is_cpu(b, pkind):
                words += N.cpu_inputs(cpu_fields(b, fp))
            slots[slot] = N.fold(words)
        isum = 0
        for b in its:
            w = N.item_word(s32(b, 0x10), s32(b, 0x24), (f32(b, 0x4C), f32(b, 0x50), f32(b, 0x54)),
                            (f32(b, 0x40), f32(b, 0x44), f32(b, 0x48)))
            isum = (isum + w) & N.M32
        h = N.game_word(seed, slots, len(its), isum)
        rows.append(f"{frame + 123},{h:08X},{seed:08X},{len(its)},{isum:08X}," + ",".join(f"{w:08X}" for w in slots))
    Path(dst).write_text("\n".join(rows) + "\n", encoding="ascii", newline="\n")
    print(f"raw2csv: {len(rows) - 1} frames -> {dst}", file=sys.stderr)


if __name__ == "__main__":
    main()
