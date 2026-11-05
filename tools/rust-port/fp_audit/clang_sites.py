#!/usr/bin/env python3
"""Per-function fused-multiply-add sites in the C PC build's front end, for every decomp TU in splits.txt.

Runs the shipping front end (fma_sites.py SHIP flags) with -g so each llvm.fmuladd gets its C line.
Two passes: `raw` (-disable-llvm-passes, what gwtool lowers; no inlining) and `inl` (LLVM -O2 inlining,
closer to MWCC's view of a function, which also inlines statics). Writes CSV rows
  tu,func,pass,type,line: type f32|f64 + square class (classify_square) + cmpd, line = file:line;
  call rows (raw pass): type=call, line=callee(argdesc,...) with L{calls} for the address of a local
  python tools/rust-port/fp_audit/clang_sites.py OUT.csv [--jobs 8] [--tu REGEX]
"""
import argparse, csv, re, subprocess, sys, tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import fma_sites  # noqa: E402

ROOT = fma_sites.ROOT


def tus():
    seen = []
    for line in (ROOT / "config/GALE01/splits.txt").read_text().splitlines():
        if line and not line[0].isspace() and line.endswith(":") and line != "Sections:":
            t = line[:-1]
            if t.startswith(("melee/", "sysdolphin/")) and (ROOT / "src" / t).exists() and t not in seen:
                seen.append(t)
    return seen


REF = re.compile(r"%[\w.]+")


def canon(defs, v, depth=0):
    """Structural form of an SSA value (allocas by name), so two loads of `p->x` compare equal."""
    if v not in defs or depth > 12:
        return v
    t = defs[v]
    if t.startswith("alloca"):
        return "alloca" + v
    return REF.sub(lambda m: canon(defs, m.group(0), depth + 1), t)


def classify_square(defs, esc, params, a, b):
    """How MWCC treats the product a*b when a and b are the same expression. MWCC (-O4,p,
    -fp_contract on) does not fuse a square of a memory or computed expression (`p->x * p->x`,
    `SQ(px - ax)`): measured over the DOL, 370 squares of loads are separate fmuls and 5 are fused.
    Squares of variables (`dx * dx`, or a field of a local struct MWCC keeps in registers) fuse.
    Returns "" (not a square), "sqvar" (fused), "sqmem" (not fused), "sqlocal{c1|c2}" (a field of a
    local struct whose address goes to the calls c1, c2: fused if MWCC inlined all of them) or
    "sqparam<i>" (through pointer parameter i: decided at the call site)."""
    if a != b and canon(defs, a) != canon(defs, b):
        return ""
    t = defs.get(a, "")
    m = re.match(r"load \w+, ptr (%[\w.]+)", t)
    if not m:
        if a in defs and not t.startswith(("fpext", "fptrunc", "sitofp", "uitofp", "call")):
            return "sqmem"  # computed operand written twice
        return "sqvar"
    ptr = m.group(1)
    d = defs.get(ptr, "")
    if d.startswith("alloca"):
        return "sqvar"
    g = re.match(r"getelementptr .*?, ptr (%[\w.]+)", d)
    base = g.group(1) if g else ptr
    bd = defs.get(base, "")
    if bd.startswith("alloca"):
        e = esc.get(base, set())
        if "store" in e:
            return "sqmem"
        return "sqlocal{" + "|".join(sorted(e)) + "}" if e else "sqvar"
    pm = re.match(r"load ptr, ptr (%[\w.]+)", bd)
    if pm and pm.group(1) in params:
        return f"sqparam{params[pm.group(1)]}"
    return "sqmem"


def parse(ll, pas):
    files, scope_file, dbg = {}, {}, {}
    for m in re.finditer(r'^!(\d+) = !DIFile\(filename: "([^"]+)"', ll, re.M):
        files[m.group(1)] = m.group(2).replace("\\\\", "/").replace("\\", "/")
    for m in re.finditer(r"^!(\d+) = (?:distinct )?!DI(?:Subprogram|LexicalBlock\w*)\((.*)\)$", ll, re.M):
        f = re.search(r"[ (]file: !(\d+)", m.group(2))
        if f:
            scope_file[m.group(1)] = files.get(f.group(1), "?")
    for m in re.finditer(r"^!(\d+) = !DILocation\(line: (\d+), column: \d+, scope: !(\d+)", ll, re.M):
        dbg[m.group(1)] = f"{scope_file.get(m.group(3), '?')}:{m.group(2)}"
    out, fn, defs, pending, esc, params = [], None, {}, {}, {}, {}
    for line in ll.splitlines():
        d = re.match(r"\s+(%[\w.]+) = (.*?)(, !dbg.*|, align \d+.*)?$", line)
        if d:
            defs[d.group(1)] = re.sub(r"\s*#\d+$", "", d.group(2))
        m = re.match(r'define .*@"?([\w.$]+)"?\(', line)
        if m:
            fn = m.group(1); defs = {}; esc = {}; params = {}
            body = ll[ll.index(line):].split("\n}\n", 1)[0]
            nargs = len(re.findall(r"%\d+[,)]", line.split("(", 1)[1].split(")")[0] + ")"))
            for x in body.splitlines():
                c = re.search(r'call .*?@"?([\w.$]+)"?\((.*)\)', x)
                if c and not c.group(1).startswith("llvm.lifetime"):
                    for v in re.findall(r"ptr (?:noundef )?(?:nonnull )?(%[\w.]+)", c.group(2)):
                        esc.setdefault(v, set()).add(c.group(1))
                st = re.match(r"\s+store ptr (%[\w.]+), ptr (%[\w.]+)", x)
                if st:
                    if re.fullmatch(r"%\d+", st.group(1)) and int(st.group(1)[1:]) < nargs:
                        params[st.group(2)] = int(st.group(1)[1:])  # parameter spill slot
                    else:
                        esc.setdefault(st.group(1), set()).add("store")
        elif line.startswith("}"):
            fn = None
        elif fn and pas == "raw" and (m := re.search(r'call .*?@"?([\w.$]+)"?\((.*)\)', line)) and not m.group(1).startswith("llvm."):
            argd = []
            for arg in re.findall(r"(?:^|, )(\w+)[^,]*? (%[\w.]+|[\w.+-]+)(?=,|$)", m.group(2)):
                v = arg[1]
                if defs.get(v, "").startswith("alloca") and arg[0] == "ptr":
                    argd.append("L{" + "|".join(sorted(esc.get(v, set()) - {m.group(1)})) + "}")
                else:
                    argd.append("-")
            out.append((fn, "call", m.group(1) + "(" + ",".join(argd) + ")"))
        elif fn and "call" in line and "llvm.fmuladd" in line:
            t = "f32" if "fmuladd.f32" in line else "f64"
            ops = re.search(r"fmuladd\.f\d+\((?:float|double)(?: noundef)? ([^,]+), (?:float|double)(?: noundef)? ([^,]+),", line)
            if pas == "raw" and ops:
                t += classify_square(defs, esc, params, ops.group(1).strip(), ops.group(2).strip())
            d = re.search(r"!dbg !(\d+)", line)
            out.append((fn, t, dbg.get(d.group(1), "?") if d else "?"))
            res = re.match(r"\s+(%[\w.]+) =", line)
            add = re.search(r", (?:float|double)(?: noundef)? (%[\w.]+)\)", line)
            if res and add:
                pending[res.group(1)] = (len(out) - 1, add.group(1))
        elif fn and pas == "raw" and (st := re.match(r"\s+store (?:float|double) (%[\w.]+), ptr (%[\w.]+)", line)):
            # `mem += a*b` / `mem -= a*b` (informational: MWCC fuses most of these)
            if st.group(1) in pending:
                i, add = pending.pop(st.group(1))
                lm = re.match(r"(?:fneg \w+ )?(%[\w.]+)$", add)
                src = lm and defs.get(lm.group(1), "")
                if lm and src.startswith("fneg"):
                    src = defs.get(src.split()[-1], "")
                ld = re.match(r"load \w+, ptr (%[\w.]+)", src or "")
                if ld and canon(defs, ld.group(1)) == canon(defs, st.group(2)) and not defs.get(st.group(2), "").startswith("alloca"):
                    f, t, ln = out[i]
                    out[i] = (f, t + "cmpd", ln)
    return out


def run(tu):
    rows = []
    with tempfile.TemporaryDirectory() as td:
        for pas in ("raw", "inl"):
            flags = list(fma_sites.SHIP)
            if pas == "inl":
                i = flags.index("-Xclang"); del flags[i:i + 2]
            o = Path(td) / "x.ll"
            r = subprocess.run([fma_sites.CLANG, *flags, "-g", "src/" + tu, "-o", str(o)], cwd=ROOT,
                               capture_output=True, text=True)
            if r.returncode:
                rows.append((tu, "!error", pas, "", 0)); continue
            rows += [(tu, f, pas, t, ln) for f, t, ln in parse(o.read_text(), pas)]
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--tu", default=".")
    a = ap.parse_args()
    todo = [t for t in tus() if re.search(a.tu, t)]
    with ThreadPoolExecutor(a.jobs) as ex, open(a.out, "w", newline="") as f:
        w = csv.writer(f); w.writerow(["tu", "func", "pass", "type", "line"])
        for rows in ex.map(run, todo):
            w.writerows(rows)
    print(len(todo), "TUs")


if __name__ == "__main__":
    main()
