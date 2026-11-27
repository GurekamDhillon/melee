"""Audit 2: fused multiply-add sites, retail main.dol vs the Rust port.

Per function in the online-relevant areas: the count of fused instructions in the DOL function
(fmadds/fmsubs/fnmadds/fnmsubs, double fmadd/fmsub/fnmadd/fnmsub; paired-single madds listed apart)
and the count of fused-helper call sites in the Rust function of the same name
(`fmadds`, `fmsubs`/`fmsubs_neg`, `fnmsubs`, `fnmadds`; double `.fmadd(`/`fmadd(`/`.mul_add(`).
A second Rust count folds in the Rust function's direct callees that are `#[inline]` or small
(<= SMALL_LINES lines) when the DOL function has no `bl` to a symbol of that name (Metrowerks inlined it),
recursively up to FOLD_DEPTH levels.

Prints counts and names only. Usage: python -I tools/rust-port/dol_audit/fusion.py [--all-areas]
       [--top N] [--func RE] [--tsv OUT]
"""
import argparse
import os
import re
import sys
from collections import Counter

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dol as D  # noqa: E402

SMALL_LINES = 12
FOLD_DEPTH = 4
SCALAR = ("fmadds", "fmsubs", "fnmadds", "fnmsubs")
DOUBLE = ("fmadd", "fmsub", "fnmadd", "fnmsub")
RUST_PATS = [
    ("fmadds", re.compile(r"\bfmadds\s*\(")),
    ("fmsubs", re.compile(r"\bfmsubs(?:_neg)?\s*\(")),
    ("fnmsubs", re.compile(r"\bfnmsubs\s*\(")),
    ("fnmadds", re.compile(r"\bfnmadds\s*\(")),
    ("fmadd", re.compile(r"(?:\.\s*fmadd\s*\(|(?<![\w.])fmadd\s*\(|\.\s*mul_add\s*\()")),
]
MACRO_RE = re.compile(r"(?:melee_core::)?(?:mod_hook|ct)!\s*\((?:[^()]|\((?:[^()]|\([^()]*\))*\))*\)\s*;?")
HELPERS = {"fmadds", "fmsubs", "fmsubs_neg", "fnmsubs", "fnmadds", "fmadd", "mul_add"}
CALL_RE = re.compile(r"(?<![\w.])((?:[A-Za-z_]\w*::)*)([A-Za-z_]\w*)\s*(?:::<[^>]*>)?\s*\(")
NOT_CALLS = {"if", "while", "for", "match", "return", "Some", "Ok", "Err", "loop", "as", "fn", "let", "unsafe",
             "move", "in", "box", "ref", "mut", "self", "Self", "super", "crate", "where", "impl", "dyn"}


MANUAL_RE = re.compile(r"//[^\n]*\b(?:fused|fmadds|fmsubs|fnmsubs|fnmadds|fnmadd|fnmsub|fmsub)\b", re.I)


def rust_counts(body, raw=None):
    """Helper call sites; `manual` counts lines whose comment says the site is fused but which call no
    helper (hand-written f64 forms such as `(a as f64 - b as f64 * c as f64) as f32`)."""
    c = Counter()
    for k, pat in RUST_PATS:
        n = len(pat.findall(body))
        if n:
            c[k] += n
    if raw:
        lines = raw.split("\n")
        for k, line in enumerate(lines):
            if not MANUAL_RE.search(line):
                continue
            code = line.split("//")[0]
            if not code.strip() and k + 1 < len(lines):
                code = lines[k + 1].split("//")[0]  # comment on its own line describes the next one
            if re.search(r"\bas f64\b", code) and not any(p.search(code) for _, p in RUST_PATS):
                c["manual"] += 1
    return c


def dol_bl_names(dol, f, by_addr):
    out = set()
    for a in range(f.addr, f.end, 4):
        w = dol.word(a)
        if w is not None and (w >> 26) == 18 and (w & 1):
            li = w & 0x03FFFFFC
            if li & 0x02000000:
                li -= 0x04000000
            t = (li if (w & 2) else a + li) & 0xFFFFFFFF
            g = by_addr.get(t)
            if g is not None:
                out.add(g.name)
    return out


_COUNT_CACHE = {}


def fold(rf, fns, dol_names, bl_names, depth, seen):
    """Fused counts of the Rust function plus inlined-in-DOL callees. Returns (Counter, folded names)."""
    key = (rf.path, rf.line)
    if key not in _COUNT_CACHE:
        _COUNT_CACHE[key] = rust_counts(rf.body, rf.raw)
    c = Counter(_COUNT_CACHE[key])
    folded = []
    if depth <= 0:
        return c, folded
    # the mod-hook / coverage macros name the `__orig` body a second time: not a call site of its own
    body = MACRO_RE.sub(" ", rf.body)
    for m in CALL_RE.finditer(body):
        n = m.group(2)
        mod = m.group(1).rstrip(":").split("::")[-1] if m.group(1) else None
        if n in NOT_CALLS or n in HELPERS or n in bl_names:
            continue
        cands = fns.get(n)
        if not cands:
            continue
        cal = D.rust_tu_fn(fns, n, rf.path, mod)
        if cal is None or (cal.path, cal.line) in seen:
            continue
        small = cal.body.count("\n") <= SMALL_LINES
        # a callee that is its own DOL function with no bl to it here: MWCC inlined it (or the C used a macro);
        # folded when the Rust side is #[inline] or small. A Rust-only helper (no DOL symbol: `X__orig`
        # wrappers, crate helpers) is always folded.
        if (n not in dol_names) or cal.inline or small:
            sub, sf = fold(cal, fns, dol_names, bl_names, depth - 1, seen | {(cal.path, cal.line)})
            if sum(sub.values()):
                c += sub
                folded.append(n)
                folded += sf
    return c, folded


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dol", default=D.DOL_PATH)
    ap.add_argument("--symbols", default=D.SYM_PATH)
    ap.add_argument("--splits", default=D.SPLIT_PATH)
    ap.add_argument("--all-areas", action="store_true")
    ap.add_argument("--func")
    ap.add_argument("--top", type=int, default=0, help="limit the ranked disagreement list (0 = all)")
    ap.add_argument("--tsv")
    args = ap.parse_args()

    dol = D.Dol(args.dol)
    funcs = D.load_functions(args.symbols, args.splits)
    by_addr = {f.addr: f for f in funcs}
    dol_names = {f.name for f in funcs}
    fns = D.load_rust_functions()

    sel = funcs if args.all_areas else [f for f in funcs if f.area in D.ONLINE_AREAS]
    if args.func:
        sel = [f for f in sel if re.fullmatch(args.func, f.name)]

    tot_dol = Counter()
    tot_dol_area = {}
    tot_rust = Counter()
    tot_rust_fold = Counter()
    n_with_rust = n_dol_fused = n_dol_fused_no_rust = 0
    n_agree = n_agree_fold = n_agree_manual = 0
    ps_funcs = []
    rows = []
    for f in sel:
        dc = D.fused_counts(dol, f)
        ps = {k: v for k, v in dc.items() if k.startswith("ps_")}
        sc = Counter({k: v for k, v in dc.items() if not k.startswith("ps_")})
        tot_dol += dc
        tot_dol_area.setdefault(f.area, Counter()).update(dc)
        if ps:
            ps_funcs.append((f, ps))
        rf = D.rust_port_of(fns, f)
        if sum(sc.values()):
            n_dol_fused += 1
        if rf is None:
            if sum(sc.values()):
                n_dol_fused_no_rust += 1
            continue
        n_with_rust += 1
        rc = rust_counts(rf.body, rf.raw)
        bl = dol_bl_names(dol, f, by_addr)
        fc, folded = fold(rf, fns, dol_names, bl, FOLD_DEPTH, {(rf.path, rf.line)})
        tot_rust += rc
        tot_rust_fold += fc
        # double-precision kinds: Rust has one helper (fmadd / mul_add) for all four signs
        dd = sum(sc[k] for k in DOUBLE)
        ds = Counter({k: sc[k] for k in SCALAR if sc[k]})
        rs = Counter({k: rc[k] for k in SCALAR if rc[k]})
        fs = Counter({k: fc[k] for k in SCALAR if fc[k]})
        dtot = sum(ds.values()) + dd
        rtot = sum(rs.values()) + rc["fmadd"]
        ftot = sum(fs.values()) + fc["fmadd"]
        same = ds == rs and dd == rc["fmadd"]
        same_f = ds == fs and dd == fc["fmadd"]
        # hand-written f64 fused forms marked by a comment make up the difference: agreement by total
        same_m = (rc["manual"] or fc["manual"]) and (dtot == rtot + rc["manual"] or dtot == ftot + fc["manual"])
        n_agree += same
        n_agree_fold += same_f
        n_agree_manual += bool(same_m) and not (same or same_f)
        if same or same_f or same_m:
            continue
        # likelihood the disagreement is real: both counts disagree with the DOL in the same direction,
        # no folding involved, totals far apart
        diff = abs(dtot - ftot)
        direction_consistent = (dtot - rtot) * (dtot - ftot) > 0
        score = diff * (2 if direction_consistent else 1) + (3 if not folded else 0) + (2 if (ftot == 0) != (dtot == 0) else 0)
        if dtot == ftot:
            score = 0.5  # same total, different kinds (sign forms)
        rows.append((score, f, ds, dd, rs, rc["fmadd"], fs, fc["fmadd"], dtot, rtot, ftot, folded, rf, rc["manual"] + fc["manual"]))

    rows.sort(key=lambda r: (-r[0], r[1].addr))

    def fmt(c, d):
        parts = ["%s %d" % (k, c[k]) for k in SCALAR if c.get(k)]
        if d:
            parts.append("dbl %d" % d)
        return ", ".join(parts) or "0"

    print("functions in scope %d, with a Rust function of the same name %d" % (len(sel), n_with_rust))
    print("DOL functions with scalar/double fused instructions: %d (of which no Rust function of that name: %d)" % (
        n_dol_fused, n_dol_fused_no_rust))
    print("\nDOL fused totals (scope):", dict(sorted(tot_dol.items())))
    for a in D.AREA_ORDER:
        if a in tot_dol_area and sum(tot_dol_area[a].values()):
            print("  %-20s %s" % (a, dict(sorted(tot_dol_area[a].items()))))
    print("Rust helper call sites in those functions: %s;  with inlined callees folded: %s" % (
        dict(sorted(tot_rust.items())), dict(sorted(tot_rust_fold.items()))))
    print("functions with a Rust counterpart: counts agree %d, agree after folding %d, agree only by counting"
          " comment-marked hand-written f64 forms %d, disagree %d" % (n_agree, n_agree_fold, n_agree_manual, len(rows)))
    print("\nranked disagreements (score, DOL | Rust | Rust+folded):")
    lim = rows[:args.top] if args.top else rows
    for r in lim:
        score, f, ds, dd, rs, rd, fs, fd, dtot, rtot, ftot, folded, rf, man = r
        print("  %5.1f %08X %-36s %-8s DOL[%s] Rust[%s] fold[%s]%s%s" % (
            score, f.addr, f.name, f.area[:8], fmt(ds, dd), fmt(rs, rd), fmt(fs, fd),
            (" manual-f64 %d" % man) if man else "",
            ("  folded: " + ",".join(sorted(set(folded)))[:80]) if folded else ""))
    if ps_funcs:
        print("\npaired-single fused instructions (no Rust helper kind; listed for reference): %d functions" % len(ps_funcs))
        for f, ps in ps_funcs:
            print("  %08X %-36s %s" % (f.addr, f.name, ", ".join("%s %d" % kv for kv in sorted(ps.items()))))
    if args.tsv:
        with open(args.tsv, "w", encoding="utf-8") as o:
            o.write("score\taddr\tfunc\tarea\tdol\trust\trust_folded\tdol_total\trust_total\tfolded_total\tfolded\tport\n")
            for r in rows:
                score, f, ds, dd, rs, rd, fs, fd, dtot, rtot, ftot, folded, rf, man = r
                o.write("%.1f\t%08X\t%s\t%s\t%s\t%s\t%s\t%d\t%d\t%d\t%s\t%s:%d\n" % (
                    score, f.addr, f.name, f.area, fmt(ds, dd), fmt(rs, rd), fmt(fs, fd), dtot, rtot, ftot,
                    ",".join(sorted(set(folded))), rf.path, rf.line))


if __name__ == "__main__":
    main()
