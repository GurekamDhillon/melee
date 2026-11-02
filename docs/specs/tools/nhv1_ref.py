#!/usr/bin/env python3
"""Reference implementation of NHV1 (docs/specs/nhv1.md): the words, in pure Python.

    python docs/specs/tools/nhv1_ref.py            self-test: the C oracle vectors and the golden CSV hash column
    python docs/specs/tools/nhv1_ref.py --worked   print the worked example of the spec (section 9)

The C oracle vectors are rust/melee_types/tests/data/nh_expected.txt (written by
rust/melee_types/gen/nh_oracle.py, which compiles src/melee/ft/fighter.c's own nh_* functions natively).
No disc data is read: the vectors and the goldens are hashes.
"""
import struct
import sys
from pathlib import Path

M32 = 0xFFFFFFFF
ROOT = Path(__file__).resolve().parents[3]


def mix(h, v):
    """nh_Mix: h ^= v; h *= 0x01000193; h ^= h >> 15 (all u32, wrapping)."""
    h = (h ^ (v & M32)) & M32
    h = (h * 0x01000193) & M32
    return h ^ (h >> 15)


def f32(x):
    """Round a Python float to the nearest IEEE-754 binary32 (what a C f32 holds)."""
    return struct.unpack("<f", struct.pack("<f", x))[0]


def bits(x):
    """The IEEE-754 bit pattern of an f32. No NaN or -0 canonicalisation (nhv1.md section 3)."""
    return struct.unpack("<I", struct.pack("<f", x))[0]


def u32(v):
    return v & M32


FIGHTER_FIELDS = ["slot+0x100", "motion_id", "pos.x", "pos.y", "pos.z", "vel.x", "vel.y", "percent", "facing",
                  "ground_or_air", "kb.x", "kb.y", "gr_vel", "hitlag", "x2219_b5", "shield", "cmd_frame", "jumps",
                  "x67F", "x680", "stocks"]

# the CPU flag word: bitfields in declaration order, first declared in the most significant bits (32 bits in all)
FLAG_WIDTHS = [1, 2, 2, 1, 1, 1] + [1] * 8 + [1, 1, 1, 2, 1, 1, 1] + [1] * 8


def fighter_inputs(f, slot):
    """The 21 base words of nh_Fighter, in order (f: C field values; floats as Python floats)."""
    return [u32(slot + 0x100), u32(f["motion_id"]), bits(f["pos"][0]), bits(f["pos"][1]), bits(f["pos"][2]),
            bits(f["vel"][0]), bits(f["vel"][1]), bits(f["percent"]), bits(f["facing"]), u32(f["ground_or_air"]),
            bits(f["kb"][0]), bits(f["kb"][1]), bits(f["gr_vel"]), bits(f["hitlag"]), u32(f["b5"]), bits(f["shield"]),
            bits(f["cmd_frame"]), u32(f["jumps"]), u32(f["x67F"]), u32(f["x680"]), u32(f["stocks"])]


def cpu_flags(vals):
    w = 0
    for v, n in zip(vals, FLAG_WIDTHS):
        w = ((w << n) | (v & ((1 << n) - 1))) & M32
    return w


def cpu_inputs(c):
    """The 27 words of the CPU block (nhv1.md section 4.2)."""
    u8 = lambda v: v & 0xFF
    return ([0x43505541, u32(c["buttons"]),
             (u8(c["lstick"][0]) << 24) | (u8(c["lstick"][1]) << 16) | (u8(c["cstick"][0]) << 8) | u8(c["cstick"][1]),
             (u8(c["ltrigger"]) << 8) | u8(c["rtrigger"]), u32(c["kind"]), u32(c["level"])]
            + [u32(v) for v in c["x14_34"]] + [bits(v) for v in c["x38_40"]] + [u32(v) for v in c["x7C_84"]]
            + [u8(c["xC8"]), u8(c["xEC"]), u32(c["command_duration"]), u32(c["cs_offset"]), cpu_flags(c["flags"])])


def fold(words, h=0x811C9DC5):
    for w in words:
        h = mix(h, w)
    return h


def fighter_word(f, slot, cpu=None):
    words = fighter_inputs(f, slot)
    if cpu is not None:
        words += cpu_inputs(cpu)
    return fold(words)


def item_word(kind, msid, pos, vel):
    """One item: the start value is 0x4954454D ("ITEM"), then kind, msid, pos xyz, vel xyz."""
    return fold([u32(kind), u32(msid)] + [bits(v) for v in list(pos) + list(vel)], 0x4954454D)


def game_word(seed, slot_words, item_count, item_sum):
    """slot_words: 12 entries (slot = player*2 + follower); 0 or None = absent."""
    h = mix(0x811C9DC5, 0x4E485631)
    h = mix(h, seed)
    for w in slot_words:
        if w:
            h = mix(h, w)
    h = mix(h, 0x4954434E)
    h = mix(h, item_count)
    return mix(h, item_sum & M32)


# ---- the oracle fill (mirrors rust/melee_types/gen/nh_oracle.py fill(k)) ------------------------------------------
def oracle_fill(k):
    kf = float(k)
    pat = (0xA5C30F96 ^ ((k * 0x01010101) & M32)) & M32
    f = dict(motion_id=0x10 + k * 7, pos=[f32(kf * f32(1.5)), f32(-kf * f32(0.25)), f32(kf * f32(0.1))],
             vel=[f32(kf * f32(0.5)), f32(-1.0 - kf)], percent=f32(kf * f32(12.5)), facing=-1.0 if k & 1 else 1.0,
             ground_or_air=k & 1, kb=[f32(kf * 3.0), f32(kf * -2.0)], gr_vel=f32(kf * f32(0.75)), hitlag=kf,
             b5=k & 1, shield=f32(60.0 - kf), cmd_frame=f32(kf * 2.0), jumps=k & 0xFF, x67F=(k + 1) & 0xFF,
             x680=(k + 2) & 0xFF, stocks=3 + (k % 4))
    s8 = lambda v: v & 0xFF
    flags = []
    for n in FLAG_WIDTHS:
        flags.append(pat & ((1 << n) - 1))
        pat = ((pat >> n) | (pat << (32 - n))) & M32
    c = dict(buttons=0x1000 + k, lstick=[s8(-k), s8(k)], cstick=[s8(k * 9), s8(-k * 13)], ltrigger=(k * 10) & 0xFF,
             rtrigger=(255 - k) & 0xFF, kind=k, level=9, x14_34=[k * 100 + i for i in range(9)],
             x38_40=[f32(kf * f32(0.5) + a) for a in (0.0, 1.0, 2.0)], x7C_84=[-k, -k - 1, -k - 2], xC8=k & 0xFF,
             xEC=(k * 2) & 0xFF, command_duration=30 + k, cs_offset=k * 3, flags=flags)
    item = (k * 5, k + 3, [f32(kf * 4.0), f32(kf * f32(-0.5)), 0.0], [f32(kf * f32(0.125)), -kf, f32(kf * f32(1e-3))])
    return f, c, item


def geno_digest(la_i, la_f, ra_i, ra_f, move_i, move_f, action_time):
    """The optional geno<port> word (nhv1.md section 8.1) for an ACTIVE (non-inert) fighter.
    la_*/ra_*: 64 values each; move_*: 8 each; ints are s32 (taken as u32), floats are Python floats (binary32 bits)."""
    assert len(la_i) == len(la_f) == len(ra_i) == len(ra_f) == 64 and len(move_i) == len(move_f) == 8
    h = mix(0x811C9DC5, 0x474E5631)
    for v in la_i:
        h = mix(h, u32(v))
    for v in la_f:
        h = mix(h, bits(v))
    for v in ra_i:
        h = mix(h, u32(v))
    for v in ra_f:
        h = mix(h, bits(v))
    for v in move_i:
        h = mix(h, u32(v))
    for v in move_f:
        h = mix(h, bits(v))
    return mix(h, u32(action_time))


def geno_digest_zero(action_time):
    """geno_digest of an active fighter whose variable banks are all zero (only the action clock set)."""
    return geno_digest([0] * 64, [0.0] * 64, [0] * 64, [0.0] * 64, [0] * 8, [0.0] * 8, action_time)


def selftest():
    bad = 0
    p = ROOT / "rust/melee_types/tests/data/nh_expected.txt"
    for line in p.read_text().splitlines():
        if line.startswith("#") or not line.strip():
            continue
        k, cpu, flags, fw, iw = line.split()
        k = int(k)
        f, c, item = oracle_fill(k)
        got = (cpu_flags(c["flags"]), fighter_word(f, k, c if cpu == "1" else None), item_word(*item))
        want = (int(flags, 16), int(fw, 16), int(iw, 16))
        ok = got == want
        bad += not ok
        print("oracle k=%d cpu=%s flags/fighter/item %s" % (k, cpu, "OK" if ok else "MISMATCH %s != %s" % (got, want)))
    n = 0
    for csv in sorted((ROOT / "docs/rust-port/golden").glob("*.csv")):
        for line in csv.read_text().splitlines()[1:]:
            c = line.split(",")
            hx = lambda s: int(s, 16)
            got = game_word(hx(c[2]), [hx(x) for x in c[5:17]], int(c[3]), hx(c[4]))
            if got != hx(c[1]):
                bad += 1
                print("golden mismatch", csv.name, c[0])
            n += 1
    print("golden rows recomputed: %d" % n)
    return bad


def worked():
    f, c, item = oracle_fill(0)
    print("k=0 fighter (slot 0, non-CPU, stocks 3); fields from nh_oracle.py fill(0)")
    h = 0x811C9DC5
    print("  start                  %08X" % h)
    for name, w in zip(FIGHTER_FIELDS, fighter_inputs(f, 0)):
        h = mix(h, w)
        print("  %-13s %08X -> %08X" % (name, w, h))
    print("  fighter word           %08X" % h)
    ih = 0x4954454D
    print("item k=0 (kind 0, msid 3)")
    for name, w in zip(["kind", "msid", "pos.x", "pos.y", "pos.z", "vel.x", "vel.y", "vel.z"],
                       [item[0], item[1]] + [bits(v) for v in item[2] + item[3]]):
        ih = mix(ih, w)
        print("  %-13s %08X -> %08X" % (name, w, ih))
    row = (ROOT / "docs/rust-port/golden/pad_falcon_fox_fd.csv").read_text().splitlines()[1].split(",")
    seed, s0, s2 = int(row[2], 16), int(row[5], 16), int(row[7], 16)
    print("game word, golden frame 0 of pad_falcon_fox_fd.csv")
    h = mix(0x811C9DC5, 0x4E485631)
    print("  NHV1 tag               -> %08X" % h)
    h = mix(h, seed)
    print("  seed %08X         -> %08X" % (seed, h))
    h = mix(h, s0)
    print("  s0   %08X         -> %08X" % (s0, h))
    h = mix(h, s2)
    print("  s2   %08X         -> %08X" % (s2, h))
    h = mix(h, 0x4954434E)
    print("  ITCN 4954434E         -> %08X" % h)
    h = mix(h, 0)
    print("  items 0               -> %08X" % h)
    h = mix(h, 0)
    print("  itemsum 0             -> %08X   (csv hash column: %s)" % (h, row[1]))


if __name__ == "__main__":
    if "--worked" in sys.argv:
        worked()
    else:
        sys.exit(1 if selftest() else 0)
