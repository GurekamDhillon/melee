#!/usr/bin/env python3
"""Scan every fighter subaction script (ftcmd) on one or more discs and report the opcode set.

Why: Geno reserves one ftcmd opcode as its escape (docs/geno.md, "Script escape"). That is only
safe if no shipped script uses it. This walks every Pl*.dat on each disc the way the engine
reaches scripts - the action table (ftData root +0x0C), the demo table (+0x14), the "ftcmd" row
table m-ex Kirby copy files export - and follows Subroutine/Goto targets, decoding with the
engine's own command lengths (ftaction.c ftAction_803C0870 + lbCommand's 0-9). It prints an
opcode histogram per disc and every script word whose opcode is >= 59 (outside the retail table).

    python pc/geno/tools/scan_ftcmd_opcodes.py "C:/iso/SSBM ACE Build v2.0.0.iso" ...

The decoder follows the Shadow analysis tool (experiment/Shadow/analysis/01_artifacts/scripts/
ftcmd.py); only the walk over whole discs is new. Reads mex_hsd.py from the root repo's
tools/mex_port (GW_ROOT, default: three levels above this file's melee worktree).
"""
import collections
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.environ.get("GW_ROOT") or r"C:/Users/Gurek/Desktop/GD's Melee"
sys.path.insert(0, os.path.join(ROOT, "tools", "mex_port"))
import mex_hsd  # noqa: E402

# words per command: lbCommand 0-9, then ftAction_803C0870 for 10-58
LEN = [1, 1, 1, 1, 1, 2, 1, 2, 1, 1] + [
    5, 5, 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 1, 7, 4, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 2, 1, 4]
assert len(LEN) == 59
GENO_OP = 0x3B  # the opcode Geno claims (pc/geno/geno.h GENO_FTCMD_OP)


def rows(ar, base, maxn=1024):
    """Action-table rows (0x18): yields script data offsets."""
    for i in range(maxn):
        o = base + i * 0x18
        if o + 0x18 > ar.data_size:
            return
        name = ar.u32(o)
        if name != 0 and o not in ar.reloc_set:
            return  # past the table
        if (o + 0xC) in ar.reloc_set:
            yield ar.u32(o + 0xC)


def walk(ar, starts, hist, odd, where):
    seen = set()
    todo = list(starts)
    while todo:
        o = todo.pop()
        if o in seen or not (0 <= o < ar.data_size):
            continue
        seen.add(o)
        n = 0
        while o + 4 <= ar.data_size and n < 2000:
            w = ar.u32(o)
            op = w >> 26
            hist[op] += 1
            if op >= len(LEN):
                odd.append((where, o, op, w))
                break
            if op in (5, 7) and (o + 4) in ar.reloc_set:
                todo.append(ar.u32(o + 4))
            o += 4 * LEN[op]
            n += 1
            if op in (0, 6, 7):
                break
    return len(seen)


def scan_disc(iso):
    g = mex_hsd.Gcm(iso)
    hist = collections.Counter()
    odd = []
    nfiles = nscripts = 0
    for name in sorted(g.files):
        base = name.rsplit("/", 1)[-1]
        if not (base.startswith("Pl") and base.endswith(".dat")):
            continue
        try:
            ar = mex_hsd.Archive(g.read(name)).relocate(0)
        except Exception:
            continue
        starts = []
        for sym, off in ar.publics:
            if sym.startswith("ftData") and not sym.startswith("ftDataKirbyCopy"):
                for k in (0x0C, 0x14):
                    if (off + k) in ar.reloc_set:
                        starts += list(rows(ar, ar.u32(off + k)))
            elif sym == "ftcmd":
                starts += list(rows(ar, off))
        if not starts:
            continue
        nfiles += 1
        nscripts += walk(ar, starts, hist, odd, base)
    return nfiles, nscripts, hist, odd


def main():
    isos = sys.argv[1:]
    if not isos:
        print(__doc__)
        return 2
    bad = 0
    for iso in isos:
        nfiles, nscripts, hist, odd = scan_disc(iso)
        top = max(hist) if hist else -1
        print(f"{os.path.basename(iso)}: {nfiles} fighter files, {nscripts} scripts, "
              f"{sum(hist.values())} commands, highest opcode {top}")
        hi = {op: c for op, c in sorted(hist.items()) if op >= 59}
        print(f"  opcodes >= 59: {hi if hi else 'none'}; Geno escape 0x{GENO_OP:02X} "
              f"({GENO_OP}) used {hist.get(GENO_OP, 0)} time(s)")
        for where, o, op, w in odd[:20]:
            print(f"    {where} data+0x{o:X}: opcode {op} word 0x{w:08X}")
        bad += hist.get(GENO_OP, 0)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
