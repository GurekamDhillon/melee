#!/usr/bin/env python3
"""Round-14 porting aid (night 4 round 3): port_r13.py plus the Fighter_SetDamageCallback inline.

  python rust/melee_chara/gen/port_r14.py     (from the worktree root; Windows, LLVM clang, cargo)

  - `Fighter_SetDamageCallback(gobj, cb);` (ft/inlines.h: `fp = GET_FIGHTER(gobj); fp->take_dmg_cb = cb;
    fp->death2_cb = cb;`) expanded in place where the caller's top-level fp is GET_FIGHTER(gobj);
  - calls into round-13 ports and this round's (fixpoint); otherwise port_r13's translator.
Output: src/chars/r14/<Char>.rs, gen/ported_r14.txt, gen/callbacks_r14.txt, src/chars/cb.rs rewritten
over rounds 3-14. Verified by gen/chref.py. Computed from C source only.
"""
import re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import port_r13 as r13  # noqa: E402
r12 = r13.r12

r11 = r12.r11
r2, r3, r4, r5, r7, r10 = r12.r2, r12.r3, r12.r4, r12.r5, r12.r7, r12.r10
Skip = r12.Skip
ROOT, CRATE = r12.ROOT, r12.CRATE
BUILD = ROOT / "rust/target/port_r14"
OUT = CRATE / "src/chars/r14"
LISTS = ["ported.txt"] + [f"ported_r{i}.txt" for i in range(2, 14)]

SETDMG = re.compile(r"\bFighter_SetDamageCallback\(\s*gobj\s*,\s*(\w+)\s*\)\s*;")
# `{ Fighter* fpX = GET_FIGHTER(gobj);` / `= gobj->user_data;` opening a nested block
ALIAS = re.compile(r"\{\s*Fighter\s*\*\s*(\w+)\s*=\s*(?:GET_FIGHTER\(\s*gobj\s*\)|gobj\s*->\s*user_data)\s*;")
_R12_REWRITE = r12.rewrite


def _block_end(body, start):
    i, d = start + 1, 1
    while d and i < len(body):
        d += body[i] == "{"; d -= body[i] == "}"; i += 1
    return i


def flatten(body):
    """Nested blocks at depth 0 of a body whose top is `fp = GET_FIGHTER(gobj)`:
    - `{ Fighter* fpX = GET_FIGHTER(gobj); ... }` -> fpX renamed fp inside (same value: gobj is never
      assigned in these bodies, checked), then the block is removed;
    - bare `{ ... }` blocks that declare nothing -> their statements (a block without declarations
      has no scope effect in C)."""
    if re.search(r"\bgobj\s*=(?!=)", body):
        return body
    while True:
        hit = None
        for m in re.finditer(r"\{", body):
            pre = body[:m.start()]
            if pre.count("{") != pre.count("}") or not re.search(r"(?:^|[;}])\s*$", pre):
                continue
            e = _block_end(body, m.start())
            inner = body[m.start() + 1:e - 1]
            a = ALIAS.match(body, m.start())
            if a:
                name = a.group(1)
                rest = body[a.end():e - 1]
                if name != "fp" and re.search(r"\b%s\b" % re.escape(name), body[:m.start()] + body[e:]):
                    continue
                if name != "fp":
                    rest = re.sub(r"\b%s\b" % re.escape(name), "fp", rest)
                if r10.DECL.search(rest):
                    continue
                hit = body[:m.start()] + rest + body[e:]
                break
            if not r10.DECL.search(inner) and not re.search(r"\bcase\b|\bdefault\s*:|\bbreak\b|\bcontinue\b", inner):
                hit = body[:m.start()] + inner + body[e:]
                break
        if hit is None:
            return body
        body = hit


def rewrite(raw):
    body = raw
    top = r12.USERDATA_TOP.sub(lambda m: m.group(1) + "GET_FIGHTER(gobj);", raw)
    if r10.FP_TOP.match(top):
        body = SETDMG.sub(lambda m: f"fp->take_dmg_cb = {m.group(1)}; fp->death2_cb = {m.group(1)};", top)
        t = r10.FP_TOP.match(body)
        body = body[:t.end()] + flatten(body[t.end():])
    b2, ch = _R12_REWRITE(body)
    return b2, ch or b2 != raw


_R13_KNOWN = r13.known_calls


def known_calls(extra):
    calls = _R13_KNOWN(set())
    for tu, name in r3.read_list("ported_r13.txt"):
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r13::{d}::{stem}::{name}(gobj)", "v", False)
    for (tu, name) in extra:
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r14::{d}::{stem}::{name}(gobj)", "v", False)
    return calls


def scan(drop, extra):
    saved = (r12.LISTS, r12.known_calls, r12.rewrite)
    r12.LISTS, r12.known_calls, r12.rewrite = LISTS, known_calls, rewrite
    try:
        return r12.scan(drop, extra)
    finally:
        r12.LISTS, r12.known_calls, r12.rewrite = saved


R_LISTS = {n: (HERE / n).read_bytes() for n in ("ported_r11.txt", "ported_r12.txt", "ported_r13.txt")}
EMIT_CBS = [set()]


def write_cb(r14list, r14cbs):
    old = set()
    for n in [f"callbacks_r{i}.txt" for i in range(3, 14)]:
        old |= r4.read_callbacks(n)
    keep = {n: (HERE / n).read_bytes() for n in [f"callbacks_r{i}.txt" for i in range(9, 14)]}
    r13cbs = r4.read_callbacks("callbacks_r13.txt")
    r13.write_cb([tuple(x) for x in r3.read_list("ported_r13.txt")], r13cbs | r14cbs)
    for n, b in keep.items():
        (HERE / n).write_bytes(b)
    rs = (CRATE / "src/chars/cb.rs").read_text(encoding="utf-8")
    for tu, name in r14list:
        d, stem = Path(tu).parent.name, Path(tu).stem
        key = r3.cb_name(tu, name)
        rs = re.sub(r"(fn %s\(g: \*mut HSD_GObj\) \{\n    )panic!\([^\n]*\);" % re.escape(key),
                    lambda m: m.group(1) + f"crate::chars::r14::{d}::{stem}::{name}(g);", rs)
    rs = rs.replace("gen/port_r12.py + gen/port_r13.py", "gen/port_r12.py + gen/port_r13.py + gen/port_r14.py", 1)
    (CRATE / "src/chars/cb.rs").write_text(rs, encoding="utf-8", newline="\n")
    out = sorted(" ".join(c) for c in r14cbs - old)
    (HERE / "callbacks_r14.txt").write_text(
        "# <setter tu> <defining tu | ext> <callback> stored or passed by round-14 ports (new ones only): gen/port_r14.py\n"
        + "".join(l + "\n" for l in out), encoding="utf-8", newline="\n")


def emit(tus):
    saved = (r11.OUT, r11.write_cb)
    r11.OUT = OUT
    r11.write_cb = lambda lst, cbs: None
    try:
        linemap, dropped, n = r11.emit(tus)
    finally:
        r11.OUT, r11.write_cb = saved
    lst = (HERE / "ported_r11.txt").read_text(encoding="utf-8").replace("Round-11", "Round-14").replace("port_r11.py", "port_r14.py")
    (HERE / "ported_r14.txt").write_text(lst, encoding="utf-8", newline="\n")
    for k, b in R_LISTS.items():
        (HERE / k).write_bytes(b)
    for p in OUT.glob("*.rs"):
        t = p.read_text(encoding="utf-8").replace("port_r11.py", "port_r14.py").replace("round-11", "round-14")
        p.write_text(t, encoding="utf-8", newline="\n")
    write_cb([l.split() for l in lst.splitlines()[1:] if l.strip()], set(r3.CALLBACKS) | EMIT_CBS[0])
    return linemap, dropped, n


def cargo_errors(linemap):
    import subprocess
    r = subprocess.run(["cargo", "build", "-p", "melee_chara", "--message-format=short"],
                       cwd=ROOT / "rust", capture_output=True, text=True)
    BUILD.mkdir(parents=True, exist_ok=True)
    (BUILD / "cargo_stderr.txt").write_text(r.stderr, encoding="utf-8")
    bad = set()
    for m in re.finditer(r"src[\\/]chars[\\/]r14[\\/](\w+)\.rs:(\d+):\d+: (?:error|warning: unreachable)", r.stderr):
        ln = int(m.group(2))
        name = linemap.get((m.group(1), ln))
        if name is None:
            cands = [n for (c, l), n in linemap.items() if c == m.group(1) and l >= ln]
            name = cands[0] if cands else None
        if name:
            bad.add(name)
    if r.returncode and not bad:
        sys.stderr.write(r.stderr[-4000:])
        raise SystemExit("cargo build failed outside round-14 functions")
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
        print(f"port_r14: dropping {len(bad)}: {sorted(bad)[:8]}...", flush=True)
        drop |= {(rel, fn.name) for rel, (tu, fns) in tus.items() for fn in fns if fn.name in bad}
        extra = {k for k in extra if k not in drop}
    print(f"port_r14: {n} functions in {len(tus)} TUs")


if __name__ == "__main__":
    main()
