#!/usr/bin/env python3
"""Round-12 porting aid (night 4 round 1): port_r11.py plus three semantics-preserving changes.

  python rust/melee_chara/gen/port_r12.py     (from the worktree root; Windows, LLVM clang, cargo)

  - the "already ported" filter is keyed by (tu, function), not by name: rounds 1-11 skipped every
    same-named static in other TUs (e.g. ftDonkey's two `setCallbacks` because ftCaptain's and
    ftMario's were ported), so those and their callers were never tried;
  - `Fighter* fp = gobj->user_data;` as the top-level fp becomes `Fighter* fp = GET_FIGHTER(gobj);`
    (ft/inlines.h: GET_FIGHTER is `HSD_GObjGetUserData(gobj)`, which returns `gobj->user_data`);
  - header `static inline void f(HSD_GObj* gobj)` helpers (ft/inlines.h, ft/kinds/*/*.h) whose body
    is `Fighter* fp = GET_FIGHTER(gobj);` (or `= gobj->user_data`) followed by declaration-free
    statements are expanded in place of `f(gobj);` where the caller's top-level fp is
    `GET_FIGHTER(gobj)` (same fp value; the statements reference only fp/gobj/globals).
Output: src/chars/r12/<Char>.rs, gen/ported_r12.txt, gen/callbacks_r12.txt, src/chars/cb.rs rewritten
over rounds 3-12. Verified by gen/chref.py. Computed from C source only.
"""
import re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import port_r11 as r11  # noqa: E402

r2, r3, r4, r5, r7, r10 = r11.r2, r11.r3, r11.r4, r11.r5, r11.r7, r11.r10
Skip = r11.Skip
ROOT, CRATE = r11.ROOT, r11.CRATE
BUILD = ROOT / "rust/target/port_r12"
OUT = CRATE / "src/chars/r12"
LISTS = ["ported.txt"] + [f"ported_r{i}.txt" for i in range(2, 12)]

USERDATA_TOP = re.compile(r"^(\s*(?:\w+\s+)*Fighter\s*\*\s*fp\s*=\s*)gobj\s*->\s*user_data\s*;")
HDR_FN = re.compile(r"^static\s+inline\s+void\s+(\w+)\s*\(\s*(?:HSD_GObj|Fighter_GObj)\s*\*\s*gobj\s*\)\s*\{", re.M)
HDR_FP = re.compile(r"^\s*Fighter\s*\*\s*fp\s*=\s*(?:GET_FIGHTER\(\s*gobj\s*\)|gobj\s*->\s*user_data)\s*;")


def header_inlines():
    out = {}
    files = sorted((ROOT / "src/melee/ft/kinds").glob("*/*.h")) + [ROOT / "src/melee/ft/inlines.h"]
    for f in files:
        src = f.read_text(encoding="utf-8", errors="replace")
        for m in HDR_FN.finditer(src):
            i, d = m.end(), 1
            while d and i < len(src):
                d += src[i] == "{"; d -= src[i] == "}"; i += 1
            body = r2.strip(src[m.end():i - 1])
            top = HDR_FP.match(body)
            if not top:
                continue
            rest = body[top.end():].strip()
            if not rest or r10.DECL.search(rest) or "return" in rest or "{" in rest:
                continue
            out[m.group(1)] = " ".join(rest.split())
    return out


HDR = header_inlines()


def rewrite(raw):
    body = USERDATA_TOP.sub(lambda m: m.group(1) + "GET_FIGHTER(gobj);", raw)
    if r10.FP_TOP.match(body):
        for name, stmts in HDR.items():
            body = re.sub(r"\b%s\(\s*gobj\s*\)\s*;" % name, lambda _m: stmts, body)
    b2, ch = r11.rewrite(body)
    return b2, ch or b2 != raw


def known_calls(extra):
    calls = r11.known_calls(set())
    for tu, name in r3.read_list("ported_r11.txt"):
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r11::{d}::{stem}::{name}(gobj)", "v", False)
    for (tu, name) in extra:
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r12::{d}::{stem}::{name}(gobj)", "v", False)
    return calls


def scan(drop, extra):
    ported = set()
    for lst in LISTS:
        ported |= {tuple(x) for x in r3.read_list(lst)}
    r5.Fn5.CALLS3 = known_calls(extra)
    tus = {}
    for f in sorted((ROOT / "src/melee/ft/kinds").glob("*/*.c")):
        if f.parent.name == "ftCommon":
            continue
        src = f.read_text(encoding="utf-8", errors="replace")
        rel = f.relative_to(ROOT).as_posix()
        tu = r4.TU4(rel)
        tu.ea = {}
        fns = []
        for m in re.finditer(r"^(static\s+)?(?:inline\s+)?void\s+(\w+)\s*\(\s*(?:HSD_GObj|Fighter_GObj)\s*\*\s*gobj\s*\)\s*\{", src, re.M):
            name = m.group(2)
            if (rel, name) in ported or (rel, name) in drop:
                continue
            i, d = m.end(), 1
            while d and i < len(src):
                d += src[i] == "{"; d -= src[i] == "}"; i += 1
            try:
                raw, fn_rw = rewrite(r2.strip(src[m.end():i - 1]))
                body = r11.incdec(r11.normalise(raw, tu))
                fn = r11.Fn11(tu, name, body)
                fn.cbs = set()
                fn.static = bool(m.group(1))
                fn.used_port = fn.assigned = fn.float_arg = fn.facing = fn.masked = False
                fn.padded = bool(r11.PAD.search(src[m.end():i - 1]))
                fn.rw = fn_rw
                fn.ea = bool(re.search(r11.EA_DECL, r4.norm_fp(raw)))
                rest = fn.block(body, "    ")
                if rest:
                    raise Skip("trailing")
                if not fn.lines:
                    raise Skip("empty")
                fns.append(fn)
            except Skip:
                continue
        if fns:
            tus[rel] = (tu, fns)
    return tus


def write_cb(r12list, r12cbs):
    """cb.rs over rounds 3-12: port_r11.write_cb for 3-11 (+12's keys), then entries whose target is a
    round-12 port call it. callbacks_r3..r11.txt are kept; new keys go to callbacks_r12.txt."""
    old = set()
    for n in [f"callbacks_r{i}.txt" for i in range(3, 12)]:
        old |= r4.read_callbacks(n)
    keep = {n: (HERE / n).read_bytes() for n in ("callbacks_r9.txt", "callbacks_r10.txt", "callbacks_r11.txt")}
    r11cbs = r4.read_callbacks("callbacks_r11.txt")
    r11.write_cb([tuple(x) for x in r3.read_list("ported_r11.txt")], r11cbs | r12cbs)
    for n, b in keep.items():
        (HERE / n).write_bytes(b)
    rs = (CRATE / "src/chars/cb.rs").read_text(encoding="utf-8")
    for tu, name in r12list:
        d, stem = Path(tu).parent.name, Path(tu).stem
        key = r3.cb_name(tu, name)
        rs = re.sub(r"(fn %s\(g: \*mut HSD_GObj\) \{\n    )panic!\([^\n]*\);" % re.escape(key),
                    lambda m: m.group(1) + f"crate::chars::r12::{d}::{stem}::{name}(g);", rs)
    rs = rs.replace("gen/port_r10.py + gen/port_r11.py", "gen/port_r10.py + gen/port_r11.py + gen/port_r12.py", 1)
    (CRATE / "src/chars/cb.rs").write_text(rs, encoding="utf-8", newline="\n")
    out = sorted(" ".join(c) for c in r12cbs - old)
    (HERE / "callbacks_r12.txt").write_text(
        "# <setter tu> <defining tu | ext> <callback> stored or passed by round-12 ports (new ones only): gen/port_r12.py\n"
        + "".join(l + "\n" for l in out), encoding="utf-8", newline="\n")


def emit(tus):
    # port_r11.emit with the r12 paths: swap the module globals it reads, then restore
    saved = (r11.OUT, r11.write_cb)
    r11.OUT = OUT
    r11.write_cb = lambda lst, cbs: None
    try:
        linemap, dropped, n = r11.emit(tus)
    finally:
        r11.OUT, r11.write_cb = saved
    # r11.emit wrote its listing to ported_r11.txt: move it to ported_r12.txt, restore r11's
    lst = (HERE / "ported_r11.txt").read_text(encoding="utf-8").replace("Round-11", "Round-12").replace("port_r11.py", "port_r12.py")
    (HERE / "ported_r12.txt").write_text(lst, encoding="utf-8", newline="\n")
    (HERE / "ported_r11.txt").write_bytes(R11_LIST)
    for p in OUT.glob("*.rs"):
        t = p.read_text(encoding="utf-8").replace("port_r11.py", "port_r12.py").replace("round-11", "round-12")
        p.write_text(t, encoding="utf-8", newline="\n")
    write_cb([l.split() for l in lst.splitlines()[1:] if l.strip()], set(r3.CALLBACKS) | EMIT_CBS[0])
    return linemap, dropped, n


R11_LIST = (HERE / "ported_r11.txt").read_bytes()
EMIT_CBS = [set()]


def cargo_errors(linemap):
    import subprocess
    r = subprocess.run(["cargo", "build", "-p", "melee_chara", "--message-format=short"],
                       cwd=ROOT / "rust", capture_output=True, text=True)
    BUILD.mkdir(parents=True, exist_ok=True)
    (BUILD / "cargo_stderr.txt").write_text(r.stderr, encoding="utf-8")
    bad = set()
    for m in re.finditer(r"src[\\/]chars[\\/]r12[\\/](\w+)\.rs:(\d+):\d+: (?:error|warning: unreachable)", r.stderr):
        ln = int(m.group(2))
        name = linemap.get((m.group(1), ln))
        if name is None:
            cands = [n for (c, l), n in linemap.items() if c == m.group(1) and l >= ln]
            name = cands[0] if cands else None
        if name:
            bad.add(name)
    if r.returncode and not bad:
        sys.stderr.write(r.stderr[-4000:])
        raise SystemExit("cargo build failed outside round-12 functions")
    return bad


def main():
    drop, extra = set(), set()
    while True:
        tus = scan(drop, extra)
        names = {(rel, fn.name) for rel, (tu, fns) in tus.items() for fn in fns}
        if names - extra:
            extra |= names
            continue
        for m in (r2, r3, r4, r5, r11.r6, r7, r11.r8, r11.r9, r10):
            m.BUILD = BUILD
        r2.eval_consts(tus)
        r4.probe_assigns(tus)
        r4.probe_cmps(tus)
        EMIT_CBS[0] = {c for _, fns in tus.values() for fn in fns for c in fn.cbs}
        linemap, dropped, n = emit(tus)
        bad = dropped | cargo_errors(linemap)
        if not bad:
            break
        print(f"port_r12: dropping {len(bad)}: {sorted(bad)[:8]}...", flush=True)
        drop |= {(rel, fn.name) for rel, (tu, fns) in tus.items() for fn in fns if fn.name in bad}
        extra = {k for k in extra if k not in drop}
    print(f"port_r12: {n} functions in {len(tus)} TUs")


if __name__ == "__main__":
    main()
