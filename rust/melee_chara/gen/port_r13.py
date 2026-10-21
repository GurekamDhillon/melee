#!/usr/bin/env python3
"""Round-13 porting aid (night 4 round 2): port_r12.py plus 9 newly callable group-3 ports.

  python rust/melee_chara/gen/port_r13.py     (from the worktree root; Windows, LLVM clang, cargo)

  - calls `ftColl_8007AEE0/8007AF10(gobj)` (group 3 round 13, ft0df.rs) and `ft_80085004/85088/
    850B4/85154/851C0/851D0/85204(gobj)` (group 3 round 12, ft084e.rs) go through new `stub_or!`
    hooks in src/stubs.rs (default: group 3's verified port; the oracle installs recorders 47-55);
  - calls into round-12 ports and this round's (fixpoint); otherwise port_r12's translator.
Output: src/chars/r13/<Char>.rs, gen/ported_r13.txt, gen/callbacks_r13.txt, src/chars/cb.rs rewritten
over rounds 3-13. Verified by gen/chref.py. Computed from C source only.
"""
import re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import port_r12 as r12  # noqa: E402

r11 = r12.r11
r2, r3, r4, r5, r7, r10 = r12.r2, r12.r3, r12.r4, r12.r5, r12.r7, r12.r10
Skip = r12.Skip
ROOT, CRATE = r12.ROOT, r12.CRATE
BUILD = ROOT / "rust/target/port_r13"
OUT = CRATE / "src/chars/r13"
LISTS = ["ported.txt"] + [f"ported_r{i}.txt" for i in range(2, 13)]

NEW_STUBS = {n: (n, "g", "v") for n in (
    "ftColl_8007AEE0", "ftColl_8007AF10", "ft_80085004", "ft_80085088", "ft_800850B4",
    "ft_80085154", "ft_800851C0", "ft_800851D0", "ft_80085204")}
r2.CALLS.update(NEW_STUBS)
r5.NEW.update(NEW_STUBS)


_R12_KNOWN = r12.known_calls


def known_calls(extra):
    calls = _R12_KNOWN(set())
    for tu, name in r3.read_list("ported_r12.txt"):
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r12::{d}::{stem}::{name}(gobj)", "v", False)
    for (tu, name) in extra:
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r13::{d}::{stem}::{name}(gobj)", "v", False)
    return calls


def scan(drop, extra):
    saved = (r12.LISTS, r12.known_calls)
    r12.LISTS, r12.known_calls = LISTS, known_calls
    try:
        return r12.scan(drop, extra)
    finally:
        r12.LISTS, r12.known_calls = saved


R11_LIST = (HERE / "ported_r11.txt").read_bytes()
R12_LIST = (HERE / "ported_r12.txt").read_bytes()
EMIT_CBS = [set()]


def write_cb(r13list, r13cbs):
    old = set()
    for n in [f"callbacks_r{i}.txt" for i in range(3, 13)]:
        old |= r4.read_callbacks(n)
    keep = {n: (HERE / n).read_bytes() for n in ("callbacks_r9.txt", "callbacks_r10.txt", "callbacks_r11.txt", "callbacks_r12.txt")}
    r12cbs = r4.read_callbacks("callbacks_r12.txt")
    r12.write_cb([tuple(x) for x in r3.read_list("ported_r12.txt")], r12cbs | r13cbs)
    for n, b in keep.items():
        (HERE / n).write_bytes(b)
    rs = (CRATE / "src/chars/cb.rs").read_text(encoding="utf-8")
    for tu, name in r13list:
        d, stem = Path(tu).parent.name, Path(tu).stem
        key = r3.cb_name(tu, name)
        rs = re.sub(r"(fn %s\(g: \*mut HSD_GObj\) \{\n    )panic!\([^\n]*\);" % re.escape(key),
                    lambda m: m.group(1) + f"crate::chars::r13::{d}::{stem}::{name}(g);", rs)
    rs = rs.replace("gen/port_r11.py + gen/port_r12.py", "gen/port_r11.py + gen/port_r12.py + gen/port_r13.py", 1)
    (CRATE / "src/chars/cb.rs").write_text(rs, encoding="utf-8", newline="\n")
    out = sorted(" ".join(c) for c in r13cbs - old)
    (HERE / "callbacks_r13.txt").write_text(
        "# <setter tu> <defining tu | ext> <callback> stored or passed by round-13 ports (new ones only): gen/port_r13.py\n"
        + "".join(l + "\n" for l in out), encoding="utf-8", newline="\n")


def emit(tus):
    saved = (r11.OUT, r11.write_cb)
    r11.OUT = OUT
    r11.write_cb = lambda lst, cbs: None
    try:
        linemap, dropped, n = r11.emit(tus)
    finally:
        r11.OUT, r11.write_cb = saved
    lst = (HERE / "ported_r11.txt").read_text(encoding="utf-8").replace("Round-11", "Round-13").replace("port_r11.py", "port_r13.py")
    (HERE / "ported_r13.txt").write_text(lst, encoding="utf-8", newline="\n")
    (HERE / "ported_r11.txt").write_bytes(R11_LIST)
    (HERE / "ported_r12.txt").write_bytes(R12_LIST)
    for p in OUT.glob("*.rs"):
        t = p.read_text(encoding="utf-8").replace("port_r11.py", "port_r13.py").replace("round-11", "round-13")
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
    for m in re.finditer(r"src[\\/]chars[\\/]r13[\\/](\w+)\.rs:(\d+):\d+: (?:error|warning: unreachable)", r.stderr):
        ln = int(m.group(2))
        name = linemap.get((m.group(1), ln))
        if name is None:
            cands = [n for (c, l), n in linemap.items() if c == m.group(1) and l >= ln]
            name = cands[0] if cands else None
        if name:
            bad.add(name)
    if r.returncode and not bad:
        sys.stderr.write(r.stderr[-4000:])
        raise SystemExit("cargo build failed outside round-13 functions")
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
        print(f"port_r13: dropping {len(bad)}: {sorted(bad)[:8]}...", flush=True)
        drop |= {(rel, fn.name) for rel, (tu, fns) in tus.items() for fn in fns if fn.name in bad}
        extra = {k for k in extra if k not in drop}
    print(f"port_r13: {n} functions in {len(tus)} TUs")


if __name__ == "__main__":
    main()
