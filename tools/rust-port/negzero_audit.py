#!/usr/bin/env python3
"""Sign-of-zero audit: console instructions whose exact-zero result has a sign the port can get wrong.

MWCC emits `b -= a*c` / `b - a*c` as fnmsubs, -(a*c - b), and `-(a*c) - b` as fnmadds, -(a*c + b). At an exact
cancellation both give -0.0; the port's `fmadds(-a, c, b)` / `fmsubs(-a, c, b)` give +0.0 (round-to-nearest).
The sign then reaches atan2f, divisions and the NHV1 hash (notes/fp-audit.md, "sign of zero"; the Dolphin oracle found
it in Fighter_procUpdate's knockback decay). The same class: `0.0 - x` (fsubs from a zero constant) vs `-x` (fneg).

  python tools/rust-port/negzero_audit.py [--dol main.dol] [--all]

Per DOL function with fnmsubs/fnmadds: the count, and the Rust function of the same name (case-insensitive, any crate
under rust/) with its count of `fnmsubs(`/`fnmadds(` calls and of suspect `fmadds(-`/`fmsubs(-` forms. A row is OPEN
when the Rust body has fewer fnm* calls than the DOL and has a suspect form; `--all` also prints the rest. The Rust
side's `0.0 - x` forms are listed at the end (the console's `-x` is fneg). Only addresses, names and mnemonics are
printed, never code bytes; the DOL path is never printed.
"""
import argparse, os, re, sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WS = ROOT.parents[1]
sys.path.insert(0, str(Path(__file__).resolve().parent))
from int_audit import load_dol, load_symbols  # noqa: E402

GAME_CRATES = ("melee_fighter", "melee_chara", "melee_items", "melee_world", "melee_runtime", "melee_engine", "melee_modes")


def dol_sites(dolp):
    read = load_dol(dolp)
    out = {}
    for addr, size, name in load_symbols():
        code = read(addr, size)
        if code is None:
            continue
        c = Counter()
        for k in range(0, size, 4):
            w = int.from_bytes(code[k:k + 4], "big")
            op, xo = w >> 26, (w >> 1) & 31
            if op == 59 and xo == 30:
                c["fnmsubs"] += 1
            elif op == 59 and xo == 31:
                c["fnmadds"] += 1
        if c:
            out[name] = (addr, c)
    return out


def rust_bodies():
    bodies = {}
    for crate in GAME_CRATES:
        for p in (ROOT / "rust" / crate / "src").rglob("*.rs"):
            s = p.read_text(encoding="utf-8", errors="replace")
            for m in re.finditer(r"\bfn\s+(\w+)\s*(?:<[^>]*>)?\s*\(", s):
                i = s.find("{", m.end())
                if i < 0:
                    continue
                semi = s.find(";", m.end())
                if 0 <= semi < i:
                    continue
                d, j = 0, i
                while j < len(s):
                    if s[j] == "{":
                        d += 1
                    elif s[j] == "}":
                        d -= 1
                        if d == 0:
                            break
                    j += 1
                line = s.count("\n", 0, m.start()) + 1
                bodies.setdefault(m.group(1).lower(), []).append((p.relative_to(ROOT).as_posix(), line, s[i:j + 1]))
    return bodies


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dol")
    ap.add_argument("--all", action="store_true")
    a = ap.parse_args()
    dolp = a.dol or os.environ.get("INT_AUDIT_DOL") or str(WS / "melee/orig/GALE01/sys/main.dol")
    if not Path(dolp).exists():
        sys.exit("negzero_audit: DOL not found (pass --dol)")
    dol = dol_sites(dolp)
    rb = rust_bodies()
    print("| addr | function | DOL fnmsubs+fnmadds | Rust location | Rust fnm* | Rust suspect | status |")
    print("|---|---|---|---|---|---|---|")
    stat = Counter()
    for name, (addr, c) in sorted(dol.items(), key=lambda kv: kv[1][0]):
        n = c["fnmsubs"] + c["fnmadds"]
        cands = rb.get(name.lower(), [])
        if not cands:
            stat["no rust fn"] += 1
            if a.all:
                print(f"| {addr:08X} | {name} | {n} | - | - | - | no Rust fn of this name |")
            continue
        loc, line, body = max(cands, key=lambda t: len(t[2]))
        # fnmsubs(a, c, b) / fnmadds, or the equivalent negated forms -(fmsubs(..)) / -fmadds(..)
        good = len(re.findall(r"\bfnm(?:subs|adds)\(|-\s*\(?\s*fm(?:adds|subs)\(", body))
        sus = len(re.findall(r"\bfm(?:adds|subs)\(\s*-", body))
        st = "ok" if good >= n else ("OPEN" if sus else "check (inlined or other form)")
        stat[st] += 1
        if a.all or st == "OPEN":
            print(f"| {addr:08X} | {name} | {n} | {loc}:{line} | {good} | {sus} | {st} |")
    print()
    print("summary:", dict(stat))
    zs = []
    for crate in GAME_CRATES:
        for p in (ROOT / "rust" / crate / "src").rglob("*.rs"):
            for k, l in enumerate(p.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
                if re.search(r"(?<![\w.])0\.0(?:f32)?\s*-\s*[\w(*]", l) and not l.lstrip().startswith("//"):
                    zs.append(f"{p.relative_to(ROOT).as_posix()}:{k}")
    print(f"Rust `0.0 - x` forms (fsubs from zero; the console's `-x` is fneg, differs at x = +0): {len(zs)}")
    if a.all:
        for z in zs:
            print("  ", z)


if __name__ == "__main__":
    main()
