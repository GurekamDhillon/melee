#!/usr/bin/env python3
"""Round-9 porting aid: port_r8.py plus `T* da = fp->ft_data->ext_attr;` locals.

  python rust/melee_chara/gen/port_r9.py      (from the worktree root; Windows, LLVM clang, cargo)

Adds to the round-8 subset:
  - `T* x = fp->ft_data->ext_attr;` (Master Hand, Crazy Hand, Link, Peach): `x->f` becomes a
    read-only pseudo field like round 4's dat_attrs locals, emitted in Rust as
    `(*((*(*fp).ft_data).ext_attr as *const melee_types::T)).f`. The oracle points `fp->ft_data`
    at an ftData whose ext_attr is a second attrs block (the dat_attrs block reversed), on both sides;
  - a function using such a local passes the "nothing new" filter;
  - calls into round-8 ports and this round's (fixpoint).
Output: src/chars/r9/<Char>.rs, gen/ported_r9.txt, gen/callbacks_r9.txt, src/chars/cb.rs rewritten
over rounds 3-9. Verified by gen/chref.py. Computed from C source only.

Round 8's docstring follows.
Round-8 porting aid: port_r7.py plus PAD_STACK forms and Player_GetPlayerSlotType.

  python rust/melee_chara/gen/port_r8.py      (from the worktree root; Windows, LLVM clang, cargo)

Adds to the round-7 subset:
  - `PAD_STACK(<expr>);`, `FORCE_PAD_STACK(<expr>);`, `FORCE_PAD_STACK_<n>;` statements are dropped
    (unused stack padding; no observable effect). Earlier rounds only knew `PAD_STACK(<ident>);`;
  - `Player_GetPlayerSlotType(fp->player_id)` (pl/player.c) through a TODO(group3) stub, so the
    Master/Crazy Hand `== Gm_PKind_Human` conditions translate;
  - `ftBossLib_8015BD20(gobj)` (ft/ftbosslib.c, empty body; ported as a no-op in stubs.rs);
  - calls into round-7 ports and this round's (fixpoint).
Output: src/chars/r8/<Char>.rs, gen/ported_r8.txt, gen/callbacks_r8.txt, src/chars/cb.rs rewritten
over rounds 3-8. Verified by gen/chref.py. Computed from C source only.

Round 7's docstring follows.
Round-7 porting aid: port_r6.py's translator with typed float arguments.

  python rust/melee_chara/gen/port_r7.py      (from the worktree root; Windows, LLVM clang, cargo)

Adds to the round-6 subset (see port_r6.py and earlier):
  - a `float` argument that is a Fighter field or a dat_attrs field (`fp->x`, `da->x`) of any
    arithmetic type becomes `(field as f32)`: C's implicit conversion to float (int -> float and
    double -> float both round to nearest, which is what Rust's `as f32` does; f32 -> f32 is a no-op).
    Round 6 emitted the bare field and rustc rejected every non-f32 one (62 functions);
  - Fighter bitfields x2222_b2 / x2220_b3 / x221E_b6 (added to the oracle's by-value set);
  - calls into round-6 ports and this round's (fixpoint).
Output: src/chars/r7/<Char>.rs, gen/ported_r7.txt, gen/callbacks_r7.txt (new callback keys), and
src/chars/cb.rs rewritten over rounds 3-7. Verified by gen/chref.py. Computed from C source only.
"""
import re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import port_r2 as r2  # noqa: E402
import port_r3 as r3  # noqa: E402
import port_r4 as r4  # noqa: E402
import port_r5 as r5  # noqa: E402
import port_r6 as r6  # noqa: E402
import port_r7 as r7  # noqa: E402
import port_r8 as r8  # noqa: E402
from port_r2 import Skip  # noqa: E402

ROOT, CRATE = r2.ROOT, r2.CRATE
BUILD = ROOT / "rust/target/port_r9"
OUT = CRATE / "src/chars/r9"

EA_DECL = (r"(?:const\s+)?(\w+)\s*\*\s*(\w+)\s*=\s*(?:\(\s*\w+\s*\*\s*\)\s*)?"
           r"fp\s*->\s*ft_data\s*->\s*ext_attr\s*;")
PAD = r8.PAD


def normalise(body, tu):
    """ext_attr locals -> pseudo fields `fp->_daK.` (K in tu.ea), then round 4's normalise"""
    body = r4.norm_fp(body)
    for m in list(re.finditer(EA_DECL, body)):
        ty, name = m.group(1), m.group(2)
        if ty not in tu.ea:
            tu.da_types.append(ty)
            tu.ea[ty] = len(tu.da_types) - 1
        k = tu.ea[ty]
        body = body.replace(m.group(0), ";", 1)
        body = re.sub(r"\b%s\s*->\s*" % name, f"fp->_da{k}.", body)
        if re.search(r"\b%s\b" % name, body):
            raise Skip("ext_attr local used otherwise")
    return r4.normalise(body, tu)


def da_rs(tu, text):
    def rep(m):
        k = int(m.group(1))
        ty = tu.da_types[k]
        if k in tu.ea.values():
            return f"(*((*(*fp).ft_data).ext_attr as *const melee_types::{ty}))."
        return f"(*((*fp).dat_attrs as *const melee_types::{ty}))."
    return re.sub(r"\(\*fp\)\._da(\d+)\.", rep, text)


incdec = r8.incdec

WHY = {}  # dropped function -> reason (emit-time Skip)

# Fighter members inside anonymous C unions: melee_types names the union `anon_<offset>`
# (non-bitfield members only; bitfields need the by-value handling of the oracle)
ANON_MAP = {"throw_flags": "anon_2210", "x594_s32": "anon_594"}
ANON = re.compile(r"\(\*fp\)\.(" + "|".join(ANON_MAP) + r")\b(?!_)")


ANON_MAP, ANON = r7.ANON_MAP, r7.ANON
Fn7 = r7.Fn7


def known_calls(extra):
    calls = r8.known_calls(set())
    for tu, name in r3.read_list("ported_r8.txt"):
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r8::{d}::{stem}::{name}(gobj)", "v", False)
    for (tu, name) in extra:
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r9::{d}::{stem}::{name}(gobj)", "v", False)
    return calls


def scan(drop, extra):
    ported = set()
    for lst in ("ported.txt", "ported_r2.txt", "ported_r3.txt", "ported_r4.txt", "ported_r5.txt", "ported_r6.txt", "ported_r7.txt", "ported_r8.txt"):
        ported |= {n for _, n in r3.read_list(lst)}
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
            if name in ported or name in drop:
                continue
            i, d = m.end(), 1
            while d and i < len(src):
                d += src[i] == "{"; d -= src[i] == "}"; i += 1
            try:
                raw = r2.strip(src[m.end():i - 1])
                body = incdec(normalise(raw, tu))
                fn = Fn7(tu, name, body)
                fn.cbs = set()
                fn.static = bool(m.group(1))
                fn.used_port = fn.assigned = fn.float_arg = False
                fn.padded = bool(PAD.search(src[m.end():i - 1]))
                fn.ea = bool(re.search(EA_DECL, r4.norm_fp(raw)))
                rest = fn.block(body, "    ")
                if rest:
                    raise Skip("trailing")
                if not (fn.used_port or fn.assigned or fn.float_arg or (fn.padded and fn.lines) or (fn.ea and fn.lines)):
                    raise Skip("nothing new")  # e.g. the empty callbacks
                fns.append(fn)
            except Skip:
                continue
        if fns:
            tus[rel] = (tu, fns)
    return tus


def write_cb(r9list, r9cbs):
    """cb.rs over rounds 3-9: port_r8.write_cb for 3-8 (+9's keys), then entries whose target
    is a round-9 port call it. callbacks_r3..r8.txt are kept; new keys go to callbacks_r9.txt."""
    old = set()
    for n in ("callbacks_r3.txt", "callbacks_r4.txt", "callbacks_r5.txt", "callbacks_r6.txt", "callbacks_r7.txt", "callbacks_r8.txt"):
        old |= r4.read_callbacks(n)
    keep8 = (HERE / "callbacks_r8.txt").read_bytes()
    r8cbs = r4.read_callbacks("callbacks_r8.txt")
    r8.write_cb([tuple(x) for x in r3.read_list("ported_r8.txt")], r8cbs | r9cbs)
    (HERE / "callbacks_r8.txt").write_bytes(keep8)
    rs = (CRATE / "src/chars/cb.rs").read_text(encoding="utf-8")
    for tu, name in r9list:
        d, stem = Path(tu).parent.name, Path(tu).stem
        key = r3.cb_name(tu, name)
        rs = re.sub(r"(fn %s\(g: \*mut HSD_GObj\) \{\n    )panic!\([^\n]*\);" % re.escape(key),
                    lambda m: m.group(1) + f"crate::chars::r9::{d}::{stem}::{name}(g);", rs)
    rs = rs.replace("gen/port_r7.py + gen/port_r8.py", "gen/port_r7.py + gen/port_r8.py + gen/port_r9.py", 1)
    (CRATE / "src/chars/cb.rs").write_text(rs, encoding="utf-8", newline="\n")
    out = sorted(" ".join(c) for c in r9cbs - old)
    (HERE / "callbacks_r9.txt").write_text(
        "# <setter tu> <defining tu | ext> <callback> stored or passed by round-9 ports (new ones only): gen/port_r9.py\n"
        + "".join(l + "\n" for l in out), encoding="utf-8", newline="\n")


def emit(tus):
    by_char = {}
    for rel, (tu, fns) in sorted(tus.items()):
        by_char.setdefault(Path(rel).parent.name, []).append((rel, tu, fns))
    if OUT.exists():
        for p in OUT.glob("*.rs"):
            p.unlink()
    OUT.mkdir(parents=True, exist_ok=True)
    listing = ["# Round-9 character functions (gen/port_r9.py output): <tu> <function>, checked by gen/chref.py"]
    linemap, dropped = {}, set()
    r3.CALLBACKS.clear()
    cbs = set()
    for ch, items in sorted(by_char.items()):
        rs = [f"//! @generated by rust/melee_chara/gen/port_r9.py: round-9 callbacks of {ch}.",
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
                        l = r7.ANON.sub(lambda m: f"(*fp).{r7.ANON_MAP[m.group(1)]}.{m.group(1)}", l)
                        out.append(da_rs(tu, l))
                except Skip as e:
                    WHY[fn.name] = str(e)
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
    mods = ["//! @generated by gen/port_r9.py: round-9 translated character callbacks, one file per character.",
            *[f"pub mod {p.stem};" for p in sorted(OUT.glob("*.rs")) if p.stem != "mod"], ""]
    (OUT / "mod.rs").write_text("\n".join(mods), encoding="utf-8", newline="\n")
    (HERE / "ported_r9.txt").write_text("\n".join(listing) + "\n", encoding="utf-8", newline="\n")
    write_cb([l.split() for l in listing[1:]], set(r3.CALLBACKS) | cbs)
    return linemap, dropped, len(listing) - 1


def cargo_errors(linemap):
    r = subprocess.run(["cargo", "build", "-p", "melee_chara", "--message-format=short"],
                       cwd=ROOT / "rust", capture_output=True, text=True)
    BUILD.mkdir(parents=True, exist_ok=True)
    (BUILD / "cargo_stderr.txt").write_text(r.stderr, encoding="utf-8")
    bad = set()
    for m in re.finditer(r"src[\\/]chars[\\/]r9[\\/](\w+)\.rs:(\d+):\d+: (?:error|warning: unreachable)", r.stderr):
        ln = int(m.group(2))
        name = linemap.get((m.group(1), ln))
        if name is None:
            cands = [n for (c, l), n in linemap.items() if c == m.group(1) and l >= ln]
            name = cands[0] if cands else None
        if name:
            bad.add(name)
    if r.returncode and not bad:
        sys.stderr.write(r.stderr[-4000:])
        raise SystemExit("cargo build failed outside round-9 functions")
    return bad


def main():
    drop, extra = set(), set()
    while True:
        tus = scan(drop, extra)
        names = {(rel, fn.name) for rel, (tu, fns) in tus.items() for fn in fns}
        if names - extra:
            extra |= names
            continue
        r2.BUILD = r3.BUILD = r4.BUILD = r5.BUILD = r6.BUILD = r7.BUILD = r8.BUILD = BUILD
        r2.eval_consts(tus)
        r4.probe_assigns(tus)
        r4.probe_cmps(tus)
        linemap, dropped, n = emit(tus)
        bad = dropped | cargo_errors(linemap)
        if not bad:
            break
        print(f"port_r9: dropping {len(bad)}: {sorted(bad)[:8]}...", flush=True)
        drop |= bad
        extra = {k for k in extra if k[1] not in drop}
    print(f"port_r9: {n} functions in {len(tus)} TUs")


if __name__ == "__main__":
    main()
