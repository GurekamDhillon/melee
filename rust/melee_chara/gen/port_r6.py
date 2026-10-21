#!/usr/bin/env python3
"""Round-6 porting aid: port_r5.py's translator with more calls out of the character code.

  python rust/melee_chara/gen/port_r6.py      (from the worktree root; Windows, LLVM clang, cargo)

Adds to the round-5 subset (see port_r5.py and earlier):
  - 6 more stubbed calls (NEW_STUBS: TODO(group3) ftColl/ftAnim/ftCo, TODO(ef) efLib_DestroyAll,
    and itIsHeavy / ftAnim_80070C48 / 80070CC4 behind the ft/inlines.h inlines
    Fighter_OnItemVisible / Fighter_OnItemInvisible, written out in crate::stubs);
  - two group-3 functions group 3 has ported (ftCommon_Fall, ftCommon_ClampAirDrift:
    melee_fighter::ftcommon), the oracle runs their C (chref.py's existing ftcommon.c cut);
  - statement `fp->field++;` / `fp->field--;` (and prefix forms), rewritten as `+= 1` / `-= 1`
    before translation (same value for every C type: integer promotion then conversion back);
  - calls into round-5 ports and this round's (fixpoint).
Output: src/chars/r6/<Char>.rs, gen/ported_r6.txt, gen/callbacks_r6.txt (new callback keys), and
src/chars/cb.rs rewritten over rounds 3-6. Verified by gen/chref.py. Computed from C source only.
"""
import re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import port_r2 as r2  # noqa: E402
import port_r3 as r3  # noqa: E402
import port_r4 as r4  # noqa: E402
import port_r5 as r5  # noqa: E402
from port_r2 import Skip  # noqa: E402

ROOT, CRATE = r2.ROOT, r2.CRATE
BUILD = ROOT / "rust/target/port_r6"
OUT = CRATE / "src/chars/r6"

NEW_STUBS = {
    "ftColl_8007AEF8": ("ftColl_8007AEF8", "g", "v"),
    "efLib_DestroyAll": ("efLib_DestroyAll", "g", "v"),
    "ftCo_AirCatchHit_Coll": ("ftCo_AirCatchHit_Coll", "g", "v"),
    "ftAnim_ApplyPartAnim": ("ftAnim_ApplyPartAnim", "giif", "v"),
}
NEW_HELPERS = {
    "ftCommon_Fall": ("ftCommon_Fall", "Fff", "v"),
    "ftCommon_ClampAirDrift": ("ftCommon_ClampAirDrift", "F", "v"),
    "Fighter_OnItemVisible": ("Fighter_OnItemVisible", "gi", "v"),
    "Fighter_OnItemInvisible": ("Fighter_OnItemInvisible", "gi", "v"),
}
NEW = {**NEW_STUBS, **NEW_HELPERS}
r2.CALLS.update(NEW)
r5.NEW.update(NEW)  # Fn5.call marks these as new work

INCDEC = re.compile(r"(?:(\+\+|--)\s*(" + r4.FIELD + r")|(" + r4.FIELD + r")\s*(\+\+|--))\s*;")


def incdec(body):
    def rep(m):
        if m.group(1):
            return f"{m.group(2)} {m.group(1)[0]}= 1;"
        return f"{m.group(4)} {m.group(6)[0]}= 1;"
    return re.sub(r"(?<![\w>.])" + INCDEC.pattern, rep, body)


def known_calls(extra):
    calls = r5.known_calls(set())
    for tu, name in r3.read_list("ported_r5.txt"):
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r5::{d}::{stem}::{name}(gobj)", "v", False)
    for (tu, name) in extra:
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r6::{d}::{stem}::{name}(gobj)", "v", False)
    return calls


def scan(drop, extra):
    ported = set()
    for lst in ("ported.txt", "ported_r2.txt", "ported_r3.txt", "ported_r4.txt", "ported_r5.txt"):
        ported |= {n for _, n in r3.read_list(lst)}
    r5.Fn5.CALLS3 = known_calls(extra)
    tus = {}
    for f in sorted((ROOT / "src/melee/ft/kinds").glob("*/*.c")):
        if f.parent.name == "ftCommon":
            continue
        src = f.read_text(encoding="utf-8", errors="replace")
        rel = f.relative_to(ROOT).as_posix()
        tu = r4.TU4(rel)
        fns = []
        for m in re.finditer(r"^(static\s+)?(?:inline\s+)?void\s+(\w+)\s*\(\s*(?:HSD_GObj|Fighter_GObj)\s*\*\s*gobj\s*\)\s*\{", src, re.M):
            name = m.group(2)
            if name in ported or name in drop:
                continue
            i, d = m.end(), 1
            while d and i < len(src):
                d += src[i] == "{"; d -= src[i] == "}"; i += 1
            try:
                body = incdec(r4.normalise(r2.strip(src[m.end():i - 1]), tu))
                fn = r5.Fn5(tu, name, body)
                fn.cbs = set()
                fn.static = bool(m.group(1))
                fn.used_port = fn.assigned = False
                rest = fn.block(body, "    ")
                if rest:
                    raise Skip("trailing")
                if not (fn.used_port or fn.assigned):
                    raise Skip("nothing new")
                fns.append(fn)
            except Skip:
                continue
        if fns:
            tus[rel] = (tu, fns)
    return tus


def write_cb(r6list, r6cbs):
    """cb.rs over rounds 3-6: port_r5.write_cb for 3-5 (+6's keys), then entries whose target
    is a round-6 port call it. callbacks_r3..r5.txt are kept; new keys go to callbacks_r6.txt."""
    old = set()
    for n in ("callbacks_r3.txt", "callbacks_r4.txt", "callbacks_r5.txt"):
        old |= r4.read_callbacks(n)
    keep5 = (HERE / "callbacks_r5.txt").read_bytes()
    r5cbs = r4.read_callbacks("callbacks_r5.txt")
    r5.write_cb([tuple(x) for x in r3.read_list("ported_r5.txt")], r5cbs | r6cbs)
    (HERE / "callbacks_r5.txt").write_bytes(keep5)
    rs = (CRATE / "src/chars/cb.rs").read_text(encoding="utf-8")
    for tu, name in r6list:
        d, stem = Path(tu).parent.name, Path(tu).stem
        key = r3.cb_name(tu, name)
        rs = re.sub(r"(fn %s\(g: \*mut HSD_GObj\) \{\n    )panic!\([^\n]*\);" % re.escape(key),
                    lambda m: m.group(1) + f"crate::chars::r6::{d}::{stem}::{name}(g);", rs)
    rs = rs.replace("gen/port_r4.py + gen/port_r5.py", "gen/port_r4.py + gen/port_r5.py + gen/port_r6.py", 1)
    (CRATE / "src/chars/cb.rs").write_text(rs, encoding="utf-8", newline="\n")
    out = sorted(" ".join(c) for c in r6cbs - old)
    (HERE / "callbacks_r6.txt").write_text(
        "# <setter tu> <defining tu | ext> <callback> stored or passed by round-6 ports (new ones only): gen/port_r6.py\n"
        + "".join(l + "\n" for l in out), encoding="utf-8", newline="\n")


def emit(tus):
    by_char = {}
    for rel, (tu, fns) in sorted(tus.items()):
        by_char.setdefault(Path(rel).parent.name, []).append((rel, tu, fns))
    if OUT.exists():
        for p in OUT.glob("*.rs"):
            p.unlink()
    OUT.mkdir(parents=True, exist_ok=True)
    listing = ["# Round-6 character functions (gen/port_r6.py output): <tu> <function>, checked by gen/chref.py"]
    linemap, dropped = {}, set()
    r3.CALLBACKS.clear()
    cbs = set()
    for ch, items in sorted(by_char.items()):
        rs = [f"//! @generated by rust/melee_chara/gen/port_r6.py: round-6 callbacks of {ch}.",
              "//! Do not edit by hand: change the translator or move a function to a hand-written module.",
              "#![allow(non_snake_case, unused_unsafe, unused_variables, unused_parens, unused_comparisons, clippy::all)]", ""]
        for rel, tu, fns in items:
            body = []
            for fn in fns:
                before = set(r3.CALLBACKS)
                try:
                    out = []
                    for l in fn.lines:
                        l = re.sub(r"@C(\d+)@", lambda m: r2.rust_const(tu, int(m.group(1))), l)
                        l = re.sub(r"@K(\d+)@", lambda m: r4.rust_cmp(tu, int(m.group(1))), l)
                        m = re.search(r"@A(\d+)@", l)
                        if m:
                            l = l[:m.start()] + r4.rust_assign(tu, int(m.group(1)))
                        l = l.replace("st::ftCommon_GroundToAirStateChange(gobj, fp,", "st::ftCommon_GroundToAirStateChange(gobj, &mut *fp,")
                        l = l.replace("st::ftCommon_AirToGroundStateChange(gobj, fp,", "st::ftCommon_AirToGroundStateChange(gobj, &mut *fp,")
                        out.append(r4.da_rs(tu, l))
                except Skip:
                    dropped.add(fn.name)
                    r3.CALLBACKS.clear(); r3.CALLBACKS.update(before)
                    continue
                cbs |= fn.cbs
                body.append((fn, out))
            if not body:
                continue
            stem = Path(rel).stem
            rs.append(f"pub mod {stem} {{")
            rs += ["    #[allow(unused_imports)]", "    use crate::stubs as st;", "    #[allow(unused_imports)]", "    use melee_types::{Fighter, HSD_GObj};", ""]
            for fn, out in body:
                rs.append(f"    /// `{fn.name}`{' (static)' if fn.static else ''} ({rel}).")
                rs.append(f"    pub fn {fn.name}(gobj: *mut HSD_GObj) {{")
                rs.append("        unsafe {")
                for l in out:
                    rs.append("        " + l)
                    linemap[(ch, len(rs))] = fn.name
                rs.append("        }")
                rs += ["    }", ""]
                listing.append(f"{rel} {fn.name}")
            rs += ["}", ""]
        (OUT / f"{ch}.rs").write_text("\n".join(rs), encoding="utf-8", newline="\n")
    mods = ["//! @generated by gen/port_r6.py: round-6 translated character callbacks, one file per character.",
            *[f"pub mod {p.stem};" for p in sorted(OUT.glob("*.rs")) if p.stem != "mod"], ""]
    (OUT / "mod.rs").write_text("\n".join(mods), encoding="utf-8", newline="\n")
    (HERE / "ported_r6.txt").write_text("\n".join(listing) + "\n", encoding="utf-8", newline="\n")
    write_cb([l.split() for l in listing[1:]], set(r3.CALLBACKS) | cbs)
    return linemap, dropped, len(listing) - 1


def cargo_errors(linemap):
    r = subprocess.run(["cargo", "build", "-p", "melee_chara", "--message-format=short"],
                       cwd=ROOT / "rust", capture_output=True, text=True)
    bad = set()
    for m in re.finditer(r"src[\\/]chars[\\/]r6[\\/](\w+)\.rs:(\d+):\d+: (?:error|warning: unreachable)", r.stderr):
        ln = int(m.group(2))
        name = linemap.get((m.group(1), ln))
        if name is None:
            cands = [n for (c, l), n in linemap.items() if c == m.group(1) and l >= ln]
            name = cands[0] if cands else None
        if name:
            bad.add(name)
    if r.returncode and not bad:
        sys.stderr.write(r.stderr[-4000:])
        raise SystemExit("cargo build failed outside round-6 functions")
    return bad


def main():
    drop, extra = set(), set()
    while True:
        tus = scan(drop, extra)
        names = {(rel, fn.name) for rel, (tu, fns) in tus.items() for fn in fns}
        if names - extra:
            extra |= names
            continue
        r2.BUILD = r3.BUILD = r4.BUILD = r5.BUILD = BUILD
        r2.eval_consts(tus)
        r4.probe_assigns(tus)
        r4.probe_cmps(tus)
        linemap, dropped, n = emit(tus)
        bad = dropped | cargo_errors(linemap)
        if not bad:
            break
        print(f"port_r6: dropping {len(bad)}: {sorted(bad)[:8]}...", flush=True)
        drop |= bad
        extra = {k for k in extra if k[1] not in drop}
    print(f"port_r6: {n} functions in {len(tus)} TUs")


if __name__ == "__main__":
    main()
