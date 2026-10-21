#!/usr/bin/env python3
"""Round-11 porting aid: port_r10.py plus motion-flag mask locals and ftGetFacingDirInt.

  python rust/melee_chara/gen/port_r11.py     (from the worktree root; Windows, LLVM clang, cargo)

Adds to the round-10 subset (both semantics-preserving):
  - `static u32 const mf = <constant expression>;` (`static [const] T [const] x = e;`, T an integer
    type, `e` free of fp/gobj): the declaration is removed and every use of `x` becomes `((T) (e))`.
    The value is then evaluated by clang from the TU itself like every other constant argument
    (round 2's mechanism). A function using the name with `&` or as an lvalue is left alone;
  - `ftGetFacingDirInt(fp)` as an int argument (ft/inlines.h: `fp->facing_dir < 0.0f ? -1 : +1`)
    becomes `(if (*fp).facing_dir < 0.0f32 { -1i32 } else { 1i32 })` (NaN -> +1 on both sides);
  - conditions `fp->f & K` and `!(fp->f & K)` (integer field, constant K, e.g. held_buttons & HSD_PAD_B)
    -> `(((*fp).f as u32) & K) != 0` / `== 0`;
  - header inlines expanded in place: `Fighter_SetEffectHitlagCallbacks(fp)` (ft/inlines.h: stores
    efLib_PauseAll / efLib_ResumeAll into pre/post_hitlag_cb) and `ftKb_SpecialN_set_cbs(2)(gobj)`
    (ftKirby/inlines.h: death2_cb / take_dmg_cb stores, in its order; only where the caller's
    top-level `fp` is `GET_FIGHTER(gobj)`);
  - `ftCliffCommon_80081298(gobj)` (ft/ftcliffcommon.c, ledge grab check; returns bool) through a
    new TODO(group3) stub;
  - a function changed by either passes the "nothing new" filter when it emits a statement;
  - calls into round-10 ports and this round's (fixpoint).
Output: src/chars/r11/<Char>.rs, gen/ported_r11.txt, gen/callbacks_r11.txt, src/chars/cb.rs rewritten
over rounds 3-11. Verified by gen/chref.py. Computed from C source only.
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
import port_r9 as r9  # noqa: E402
import port_r10 as r10  # noqa: E402
from port_r2 import Skip  # noqa: E402

ROOT, CRATE = r2.ROOT, r2.CRATE
BUILD = ROOT / "rust/target/port_r11"
OUT = CRATE / "src/chars/r11"
EA_DECL, PAD = r10.EA_DECL, r10.PAD
normalise, da_rs, incdec = r10.normalise, r10.da_rs, r10.incdec
WHY = {}

# group-3 callee newly stubbed this round (TODO(group3) in src/stubs.rs, recorder id 46 in the oracle)
NEW_STUBS = {"ftCliffCommon_80081298": ("ftCliffCommon_80081298", "g", "i")}
r2.CALLS.update(NEW_STUBS)
r5.NEW.update(NEW_STUBS)  # Fn5.call marks these as new work

MASK = re.compile(r"static\s+(?:const\s+)?(u32|s32|int|u16|s16|u8|MotionFlags)\s+(?:const\s+)?([A-Za-z_]\w*)\s*=\s*([^;{}]+);")


def rewrite(raw):
    """round-11 mask inlining, then round 10's rewrites; returns (body, changed)"""
    body = raw
    for m in list(MASK.finditer(raw)):
        ty, name, e = m.group(1), m.group(2), m.group(3)
        if re.search(r"\b(fp|gobj)\b|->", e):
            continue
        rest = body.replace(m.group(0), "", 1)
        if re.search(r"&\s*%s\b|\b%s\s*(?:[-+*/|&^]?=(?!=)|\+\+|--)" % (name, name), rest):
            continue
        body = re.sub(r"\b%s\b" % name, lambda _m: f"(({ty}) ({' '.join(e.split())}))", rest)
    # header inlines whose body is two Fighter stores (ft/inlines.h, ftKirby/inlines.h), textually
    # expanded; the Kirby pair reads `GET_FIGHTER(gobj)`, so only where the caller's top-level fp is
    # that same value
    body = re.sub(r"\bFighter_SetEffectHitlagCallbacks\(\s*fp\s*\)\s*;",
                  "fp->pre_hitlag_cb = efLib_PauseAll; fp->post_hitlag_cb = efLib_ResumeAll;", body)
    if r10.FP_TOP.match(body):
        body = re.sub(r"\bftKb_SpecialN_set_cbs\(\s*gobj\s*\)\s*;",
                      "fp->death2_cb = ftKb_Init_800EE74C; fp->take_dmg_cb = ftKb_Init_800EE7B8;", body)
        body = re.sub(r"\bftKb_SpecialN_set_cbs2\(\s*gobj\s*\)\s*;",
                      "fp->take_dmg_cb = ftKb_Init_800EE7B8; fp->death2_cb = ftKb_Init_800EE74C;", body)
    body, ch = r10.rewrite(body)
    return body, ch or body != raw


class Fn11(r7.Fn7):
    def arg(self, a, k):
        if k == "i" and re.fullmatch(r"ftGetFacingDirInt\(\s*fp\s*\)", a.strip()):
            self.need_fp()
            self.facing = True
            return "(if (*fp).facing_dir < 0.0f32 { -1i32 } else { 1i32 })"
        return super().arg(a, k)

    def cond(self, c):
        # `fp->f & K` / `!(fp->f & K)` with K constant: C converts both sides to unsigned int
        # (usual arithmetic conversions; integer fields only), the result is tested against 0.
        # `as u32` does the same widening (zero- or sign-extending as C's promotion does).
        u = r4.unparen(c)
        neg = False
        if u.startswith("!"):
            inner = u[1:].strip()
            if inner.startswith("(") and r4.unparen(inner) != inner:
                neg, u = True, r4.unparen(inner)
        m = re.fullmatch(r4.FIELD + r"\s*&\s*([^=!<>&|]+)", u)
        if m and "->" not in m.group(1):
            self.need_fp()
            rhs = m.group(2).strip()
            if re.search(r"\b(fp|gobj)\b|->|\[", rhs) or re.search(r"(?!sizeof)\b\w+\s*\(", rhs):
                raise Skip("non-constant mask")
            self.masked = True
            k = self.tu.const(rhs, "u")
            return f"((((*fp).{m.group(1)} as u32) & {k}) {'==' if neg else '!='} 0)"
        return super().cond(c)


def known_calls(extra):
    calls = r10.known_calls(set())
    for tu, name in r3.read_list("ported_r10.txt"):
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r10::{d}::{stem}::{name}(gobj)", "v", False)
    for (tu, name) in extra:
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r11::{d}::{stem}::{name}(gobj)", "v", False)
    return calls


def scan(drop, extra):
    ported = set()
    for lst in ("ported.txt", "ported_r2.txt", "ported_r3.txt", "ported_r4.txt", "ported_r5.txt", "ported_r6.txt", "ported_r7.txt", "ported_r8.txt", "ported_r9.txt", "ported_r10.txt"):
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
                raw, fn_rw = rewrite(r2.strip(src[m.end():i - 1]))
                body = incdec(normalise(raw, tu))
                fn = Fn11(tu, name, body)
                fn.cbs = set()
                fn.static = bool(m.group(1))
                fn.used_port = fn.assigned = fn.float_arg = fn.facing = fn.masked = False
                fn.padded = bool(PAD.search(src[m.end():i - 1]))
                fn.rw = fn_rw
                fn.ea = bool(re.search(EA_DECL, r4.norm_fp(raw)))
                rest = fn.block(body, "    ")
                if rest:
                    raise Skip("trailing")
                if not (fn.used_port or fn.assigned or fn.float_arg or fn.facing or fn.masked or (fn.padded and fn.lines) or (fn.ea and fn.lines) or (fn.rw and fn.lines)):
                    raise Skip("nothing new")
                fns.append(fn)
            except Skip:
                continue
        if fns:
            tus[rel] = (tu, fns)
    return tus


def write_cb(r11list, r11cbs):
    """cb.rs over rounds 3-11: port_r10.write_cb for 3-10 (+11's keys), then entries whose target
    is a round-11 port call it. callbacks_r3..r10.txt are kept; new keys go to callbacks_r11.txt."""
    old = set()
    for n in ("callbacks_r3.txt", "callbacks_r4.txt", "callbacks_r5.txt", "callbacks_r6.txt", "callbacks_r7.txt", "callbacks_r8.txt", "callbacks_r9.txt", "callbacks_r10.txt"):
        old |= r4.read_callbacks(n)
    keep = {n: (HERE / n).read_bytes() for n in ("callbacks_r9.txt", "callbacks_r10.txt")}
    r10cbs = r4.read_callbacks("callbacks_r10.txt")
    r10.write_cb([tuple(x) for x in r3.read_list("ported_r10.txt")], r10cbs | r11cbs)
    for n, b in keep.items():
        (HERE / n).write_bytes(b)
    rs = (CRATE / "src/chars/cb.rs").read_text(encoding="utf-8")
    for tu, name in r11list:
        d, stem = Path(tu).parent.name, Path(tu).stem
        key = r3.cb_name(tu, name)
        rs = re.sub(r"(fn %s\(g: \*mut HSD_GObj\) \{\n    )panic!\([^\n]*\);" % re.escape(key),
                    lambda m: m.group(1) + f"crate::chars::r11::{d}::{stem}::{name}(g);", rs)
    rs = rs.replace("gen/port_r9.py + gen/port_r10.py", "gen/port_r9.py + gen/port_r10.py + gen/port_r11.py", 1)
    (CRATE / "src/chars/cb.rs").write_text(rs, encoding="utf-8", newline="\n")
    out = sorted(" ".join(c) for c in r11cbs - old)
    (HERE / "callbacks_r11.txt").write_text(
        "# <setter tu> <defining tu | ext> <callback> stored or passed by round-11 ports (new ones only): gen/port_r11.py\n"
        + "".join(l + "\n" for l in out), encoding="utf-8", newline="\n")


def emit(tus):
    by_char = {}
    for rel, (tu, fns) in sorted(tus.items()):
        by_char.setdefault(Path(rel).parent.name, []).append((rel, tu, fns))
    if OUT.exists():
        for p in OUT.glob("*.rs"):
            p.unlink()
    OUT.mkdir(parents=True, exist_ok=True)
    listing = ["# Round-11 character functions (gen/port_r11.py output): <tu> <function>, checked by gen/chref.py"]
    linemap, dropped = {}, set()
    r3.CALLBACKS.clear()
    cbs = set()
    for ch, items in sorted(by_char.items()):
        rs = [f"//! @generated by rust/melee_chara/gen/port_r11.py: round-11 callbacks of {ch}.",
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
    mods = ["//! @generated by gen/port_r11.py: round-11 translated character callbacks, one file per character.",
            *[f"pub mod {p.stem};" for p in sorted(OUT.glob("*.rs")) if p.stem != "mod"], ""]
    (OUT / "mod.rs").write_text("\n".join(mods), encoding="utf-8", newline="\n")
    (HERE / "ported_r11.txt").write_text("\n".join(listing) + "\n", encoding="utf-8", newline="\n")
    write_cb([l.split() for l in listing[1:]], set(r3.CALLBACKS) | cbs)
    return linemap, dropped, len(listing) - 1


def cargo_errors(linemap):
    r = subprocess.run(["cargo", "build", "-p", "melee_chara", "--message-format=short"],
                       cwd=ROOT / "rust", capture_output=True, text=True)
    BUILD.mkdir(parents=True, exist_ok=True)
    (BUILD / "cargo_stderr.txt").write_text(r.stderr, encoding="utf-8")
    bad = set()
    for m in re.finditer(r"src[\\/]chars[\\/]r11[\\/](\w+)\.rs:(\d+):\d+: (?:error|warning: unreachable)", r.stderr):
        ln = int(m.group(2))
        name = linemap.get((m.group(1), ln))
        if name is None:
            cands = [n for (c, l), n in linemap.items() if c == m.group(1) and l >= ln]
            name = cands[0] if cands else None
        if name:
            bad.add(name)
    if r.returncode and not bad:
        sys.stderr.write(r.stderr[-4000:])
        raise SystemExit("cargo build failed outside round-11 functions")
    return bad


def main():
    drop, extra = set(), set()
    while True:
        tus = scan(drop, extra)
        names = {(rel, fn.name) for rel, (tu, fns) in tus.items() for fn in fns}
        if names - extra:
            extra |= names
            continue
        r2.BUILD = r3.BUILD = r4.BUILD = r5.BUILD = r6.BUILD = r7.BUILD = r8.BUILD = r9.BUILD = r10.BUILD = BUILD
        r2.eval_consts(tus)
        r4.probe_assigns(tus)
        r4.probe_cmps(tus)
        linemap, dropped, n = emit(tus)
        bad = dropped | cargo_errors(linemap)
        if not bad:
            break
        print(f"port_r11: dropping {len(bad)}: {sorted(bad)[:8]}...", flush=True)
        drop |= bad
        extra = {k for k in extra if k[1] not in drop}
    print(f"port_r11: {n} functions in {len(tus)} TUs")


if __name__ == "__main__":
    main()
