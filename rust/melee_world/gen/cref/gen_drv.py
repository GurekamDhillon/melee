#!/usr/bin/env python3
"""C / Rust driver generator for the gr/ TU ports (shard 577-584).

  python rust/melee_world/gen/cref/run_gr.py <stem> [...]       (run_gr.py calls main() here)

For one game TU named in specs_gr.py it
  1. runs gen_tu.build (recording stubs for every callee, `ext` block of the port, fill_/hash_ walkers),
  2. writes gen/cref/tu/<stem>.c, the C oracle driver (`#include`s the game's .c verbatim), and
     tests/tu_<stem>.rs, the Rust half, op for op and draw for draw,
  3. runs the C driver (run_tu.py machinery) into tests/data/mtu_<stem>.txt.

The synthetic game state ("world") is the one every stage function sees: GO[4] gobjs, GR[4] Grounds
(gobj->user_data = &GR[i], gobj->hsd_obj = &JP[4i]), JP[16] jobjs linked as a tree, a CollData CD,
V3[4] vectors, plus the spec's own decls. All records are filled from the shared xorshift stream
(bitfields by value); the Ground union `u` is filled through the one view the function uses (the
members named after `u.` in its body, or spec `view`), pointers inside a view point into the pools.
Every function of the TU is an op (arguments drawn by type, or spec `args`), run on `cases` seeded
worlds; after the call the whole world is hashed with the return value and the call trace.

Spec keys (besides gen_tu's): types, decl_c / decl_r (extra statics), setup_c / setup_r (statements
after the world is linked), hash_c / hash_r (extra state hashed after every op), preg_c / preg_r
(extra pointer regions), args {fn: [(c_expr, r_expr), ...]}, view {fn: member}, cases {fn: n},
pre {fn: (c_stmts, r_stmts)} (run after the world is built, before the arguments are drawn),
data [(name, type, count)] (tables hashed by op "data"), skip {fn: reason}, ops [(name, c_body, r_body)]
(extra ops; the bodies set `ret`), ret_c {fn: expr} / ret_r (override the hashed return).
"""
import os, re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gen_tu  # noqa: E402
from gen_tu import ci, K, T, canon, ROOT, rust_ty, arg_class, is_fnptr, INTS  # noqa: E402

POOLS = {
    "HSD_GObj": ("&GO[fv() & 3]", "&raw mut GO[(fv() & 3) as usize]"),
    "_HSD_GObj": ("&GO[fv() & 3]", "&raw mut GO[(fv() & 3) as usize]"),
    "HSD_JObj": ("&JP[fv() % 16]", "&raw mut JP[(fv() % 16) as usize]"),
    "_HSD_JObj": ("&JP[fv() % 16]", "&raw mut JP[(fv() % 16) as usize]"),
    "Ground": ("&GR[fv() & 3]", "&raw mut GR[(fv() & 3) as usize]"),
    "CollData": ("&CD", "&raw mut CD"),
    "Vec": ("&V3[fv() & 3]", "&raw mut V3[(fv() & 3) as usize]"),
}
# pointer fields inside a Ground union view get linked to a pool (never NULL)
VIEW_POOLS = {k: (v[0], v[1].replace("&raw mut ", "&raw mut super::")) for k, v in POOLS.items()
              if k in ("HSD_GObj", "_HSD_GObj", "HSD_JObj", "_HSD_JObj", "Ground", "Vec")}
DEF_CASES = 300
ARG_POOLS = {}


class PtrWalk(gen_tu.Walk):
    """gen_tu.Walk plus: pointer fields inside a union view are linked to the pools."""
    inview = 0
    nulls = {}
    vpools = VIEW_POOLS

    def one(self, t, f, cexpr, rexpr, cf, ch, rf, rh, depth):
        c = canon(t)
        k = c.kind
        if k == T.RECORD:
            d = c.get_declaration()
            d = d.get_definition() or d
            if d.kind == K.UNION_DECL and self.unions.get(t.spelling.replace("union ", "")):
                PtrWalk.inview += 1
                try:
                    return super().one(t, f, cexpr, rexpr, cf, ch, rf, rh, depth)
                finally:
                    PtrWalk.inview -= 1
        if k == T.POINTER and not is_fnptr(t) and PtrWalk.inview:
            super().one(t, f, cexpr, rexpr, cf, ch, rf, rh, depth)
            pt = canon(c.get_pointee())
            if pt.kind == T.RECORD:
                nm = pt.get_declaration().spelling
                if nm in PtrWalk.vpools:
                    ce, re_ = PtrWalk.vpools[nm]
                    mod = PtrWalk.nulls.get(f.spelling) if f is not None else None
                    if mod:
                        cf.append(f"{cexpr} = (fv() % {mod} == 0) ? 0 : {ce};")
                        rf.append(f"{rexpr} = if fv() % {mod} == 0 {{ core::ptr::null_mut() }} else {{ {re_} as _ }};")
                    else:
                        cf.append(f"{cexpr} = {ce};")
                        rf.append(f"{rexpr} = {re_} as _;")
            return
        return super().one(t, f, cexpr, rexpr, cf, ch, rf, rh, depth)


    def gen(self, name):
        cdef, rdef = super().gen(name)
        # the hash walkers read the union view from USELH (fill: USEL), so an op can fill one view
        # and hash another
        ci_ = cdef.index("static void hash_")
        cdef = cdef[:ci_] + cdef[ci_:].replace("switch (USEL)", "switch (USELH)")
        ri_ = rdef.index("pub fn hash_")
        rdef = rdef[:ri_] + rdef[ri_:].replace("match unsafe { USEL }", "match unsafe { super::USELH }")
        # gen_tu writes base(&p) for a top-level anonymous member: that is the address of the reference
        rdef = rdef.replace("base(&p)", "base(&*p)")
        return cdef, rdef


gen_tu.Walk = PtrWalk

_orig_build = gen_tu.build


def _build(stem, spec, want_ext):
    """gen_tu.build, then: the generated Rust uses the test root's rgx() (mixed float draws) as rg()"""
    r = _orig_build(stem, spec, want_ext)
    f = ROOT / "rust/melee_world/tests/mtu" / f"{stem}_gen.rs"
    t = f.read_text(encoding="utf-8")
    t = t.replace("use crate::mtu_harness::*;", "use crate::mtu_harness::*;" + chr(10) + "use crate::rgx as rg;", 1)
    f.write_text(t, encoding="utf-8", newline=chr(10))
    return r


gen_tu.build = _build


def view_members(tu):
    """members of GroundVars (Ground::u) by name"""
    t = tu.types.get("Ground")
    if t is None:
        return []
    d = canon(t).get_declaration()
    d = d.get_definition() or d
    for f in d.get_children():
        if f.kind == K.FIELD_DECL and f.spelling == "u":
            ud = canon(f.type).get_declaration()
            ud = ud.get_definition() or ud
            return [c.spelling for c in ud.get_children() if c.kind == K.FIELD_DECL]
    return []


def fn_view(cur, members):
    """first member of Ground::u named after `u.` / `u ->` in the function's tokens"""
    toks = [x.spelling for x in cur.get_tokens()]
    for i in range(len(toks) - 2):
        if toks[i] == "u" and toks[i + 1] in (".",) and toks[i + 2] in members:
            return toks[i + 2]
    return None


def default_arg(ptype, fname, idx):
    c = canon(ptype)
    k = c.kind
    if k == T.POINTER and not is_fnptr(ptype):
        pt = canon(c.get_pointee())
        if pt.kind == T.RECORD:
            nm = pt.get_declaration().spelling
            if nm in ARG_POOLS:
                return ARG_POOLS[nm]
            if nm in POOLS:
                return POOLS[nm]
        raise SystemExit(f"{fname}: no default for arg {idx} of type {ptype.spelling}: give spec args")
    if k in INTS or k == T.ENUM:
        return (f"(({ptype.spelling}) (fv() % 8))", f"(fv() % 8) as {rust_ty(ptype)}")
    if k == T.FLOAT:
        return ("rg()", "rg()")
    if k in (T.DOUBLE, T.LONGDOUBLE):
        return ("(double) rg()", "rg() as f64")
    raise SystemExit(f"{fname}: no default for arg {idx} of type {ptype.spelling}: give spec args")


def ret_stmt(rt, expr_c, expr_r):
    cls = arg_class(rt)
    if cls == "void":
        return (f"{expr_c};", f"{expr_r};")
    if cls == "i4":
        return (f"ret = (u32) {expr_c};", f"ret = ({expr_r}) as u32;")
    if cls == "i8":
        return (f"{{ u64_t_ r_ = (u64_t_) {expr_c}; ret = (u32) r_ ^ (u32) (r_ >> 32); }}",
                f"{{ let r_ = ({expr_r}) as u64; ret = (r_ as u32) ^ ((r_ >> 32) as u32); }}")
    if cls == "f4":
        return (f"ret = B({expr_c});", f"ret = ({expr_r}).to_bits();")
    if cls == "f8":
        return (f"HM64({expr_c});", f"hm64({expr_r});")
    if cls == "ptr":
        return (f"ret = PI((const void*) {expr_c});", f"ret = pi({expr_r});")
    if cls == "str":
        return (f"ret = SH((const char*) {expr_c});", f"ret = sh(({expr_r}) as *const u8);")
    if cls == "fp":
        return (f"ret = FI((const void*) {expr_c});", f"ret = fi({expr_r});")
    raise SystemExit("return type not supported: " + rt.spelling)


def prep(stem, spec):
    """spec + the walker types and the Ground::u views the TU's functions use -> (spec, fview, order)"""
    srcp = spec["src"]
    gen_tu.set_extra_main(spec.get("extra_main", ()))
    tu0 = gen_tu.Tu(gen_tu.patched_source(stem, spec), spec.get("args_clang", ()), (ROOT / srcp).parent)
    members = view_members(tu0)
    order = sorted(tu0.defs.items(), key=lambda kv: kv[1].extent.start.line)
    used = []
    fview = {}
    for n, cur in order:
        v = spec.get("view", {}).get(n) or fn_view(cur, members)
        h = spec.get("hview", {}).get(n)
        if v and v != "-":
            fview[n] = v
            if v not in used:
                used.append(v)
        elif v == "-":
            fview[n] = "-"
        if h and h != "-":
            if h not in used:
                used.append(h)
    for v in spec.get("views", ()):
        if v not in used:
            used.append(v)
    spec = dict(spec)
    types = list(spec.get("types", ()))
    for t in ("HSD_GObj", "HSD_JObj", "Ground", "CollData", "Vec"):
        if t not in types:
            types.append(t)
    spec["types"] = types
    if used:
        spec["unions"] = {"GroundVars": used}
    spec["_used"] = used
    PtrWalk.nulls = spec.get("view_null", {})
    PtrWalk.vpools = dict(VIEW_POOLS)
    for k, (ce, re_) in spec.get("pools", {}).items():
        PtrWalk.vpools[k] = (ce, re_.replace("&raw mut ", "&raw mut super::"))
    ARG_POOLS.clear()
    for k, (ce, re_) in spec.get("pools", {}).items():
        ARG_POOLS[k] = (ce, re_)
    return spec, fview, order


def build_driver(stem, spec):
    spec, fview, order = prep(stem, spec)
    used = spec["_used"]
    # stubs / walkers / ext
    gen_tu.build(stem, spec, False)

    ops = []   # (name, usel, pre_c, pre_r, body_c, body_r, ncases)
    skipped = []
    for n, cur in order:
        if n in spec.get("skip", {}):
            skipped.append(n)
            continue
        ft = canon(cur.type)
        ptys = list(ft.argument_types())
        rt = ft.get_result()
        ov = spec.get("args", {}).get(n)
        draws_c, draws_r, names = [], [], []
        for i, p in enumerate(ptys):
            ce, re_ = ov[i] if ov else default_arg(p, n, i)
            draws_c.append(f"__auto_type a{i} = {ce};")
            draws_r.append(f"let a{i}: {rust_ty(p)} = {re_};")
            names.append(f"a{i}")
        callc = f"{n}({', '.join(names)})"
        callr = f"m::{n}({', '.join(names)})"
        sc, sr = ret_stmt(rt, callc, callr)
        pc, pr = spec.get("pre", {}).get(n, ("", ""))
        uf = -1 if fview.get(n) == "-" else (used.index(fview[n]) if n in fview else 0)
        hv = spec.get("hview", {}).get(n)
        uh = (-1 if hv == "-" else used.index(hv)) if hv else uf
        ops.append((n, (uf, uh), pc, pr,
                    "\n".join(draws_c + [sc]), "\n".join(draws_r + [sr]), spec.get("cases", {}).get(n, DEF_CASES)))
    for (n, bc, br, *rest) in spec.get("ops", ()):
        # an op may name its (fill, hash) Ground::u views by index into `used` (-1: none)
        ops.append((n, rest[0] if rest else (0, 0), "", "", bc, br, spec.get("cases", {}).get(n, DEF_CASES)))
    data = spec.get("data", [])
    if data:
        dc, dr = [], []
        for (nm, ty, cnt) in data:
            if ty == "raw":
                dc.append(f"HM(RH(&{nm}, sizeof {nm}));")
                dr.append(f"hm(rh(&m::{nm}));")
            elif cnt:
                dc.append(f"for (i = 0; i < {cnt}; i++) hash_{ty}(&{nm}[i]);")
                dr.append(f"for i in 0..{cnt} {{ g::hash_{ty}(&m::{nm}[i]); }}")
            else:
                dc.append(f"hash_{ty}(&{nm});")
                dr.append(f"g::hash_{ty}(&m::{nm});")
        ops.append(("data", (0, 0), "", "", "\n".join(dc), "\n".join(dr), 1))
    emit(stem, spec, ops, skipped)
    return ops, skipped


C_TEMPLATE = r"""/* TU: @SRC@ */
/* C oracle for src/tu/@STEM@.rs, generated by gen/cref/gen_drv.py: the whole TU included verbatim;
 * every callee out of it is a recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj GO[4];
static Ground GR[4];
static HSD_JObj JP[16];
static CollData CD;
static Vec V3[4];
static int USELH;
/* float draws: the shared grid values (rg) a quarter of the time replaced by full-mantissa values, so
 * fused and unfused products differ */
static float rf_(void) { return ((float) (rnd() >> 8) / 16777216.0f - 0.5f) * 4.0f; }
static float rgx_(void) { u32 sel = rnd() & 3; if (sel == 0) return rf_(); return rg(); }
#define rg() rgx_()
/* TV: trace a Vec; WV: fill one (stubs that write an out-parameter) */
static void TV(const Vec* p) { if (p) { T(B(p->x)); T(B(p->y)); T(B(p->z)); } }
static void WV(Vec* p) { if (p) { p->x = rg(); p->y = rg(); p->z = rg(); } }
@DECL@
#include "@STEM@_gen.h"

static void reset_world(void)
{
    int i;
    zero(GO, sizeof GO); zero(GR, sizeof GR); zero(JP, sizeof JP); zero(&CD, sizeof CD); zero(V3, sizeof V3);
    for (i = 0; i < 4; i++) fill_HSD_GObj(&GO[i]);
    for (i = 0; i < 4; i++) fill_Ground(&GR[i]);
    for (i = 0; i < 16; i++) { fill_HSD_JObj(&JP[i]); JP[i].flags &= ~JOBJ_USE_QUATERNION; }
    fill_CollData(&CD);
    for (i = 0; i < 4; i++) { V3[i].x = rw(); V3[i].y = rw(); V3[i].z = rw(); }
    for (i = 0; i < 4; i++) { GO[i].user_data = &GR[i]; GR[i].gobj = &GO[i]; GO[i].hsd_obj = &JP[i * 4]; }
    for (i = 0; i < 16; i++) {
        JP[i].child = (2 * i + 1 < 16) ? &JP[2 * i + 1] : 0;
        JP[i].next = (i % 4 != 3 && i + 1 < 16) ? &JP[i + 1] : 0;
        JP[i].parent = i > 0 ? &JP[(i - 1) / 2] : 0;
    }
@SETUP@
}

static void hash_world(void)
{
    int i;
    for (i = 0; i < 4; i++) hash_HSD_GObj(&GO[i]);
    for (i = 0; i < 4; i++) hash_Ground(&GR[i]);
    for (i = 0; i < 16; i++) hash_HSD_JObj(&JP[i]);
    hash_CollData(&CD);
    for (i = 0; i < 4; i++) { HM(B(V3[i].x)); HM(B(V3[i].y)); HM(B(V3[i].z)); }
@HASH@
}

static const int NCASES[] = { @NCASES@ };

int main(void)
{
    int op, k, i;
    (void) i;
    preg(GO, sizeof GO);
    preg(GR, sizeof GR);
    preg(JP, sizeof JP);
    preg(&CD, sizeof CD);
    preg(V3, sizeof V3);
@PREG@
    freg_all();
    for (op = 0; op < @NOPS@; op++) {
        for (k = 0; k < NCASES[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            switch (op) {
@CASES@
            }
            hash_world();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
            fflush(0);
        }
    }
    return 0;
}
"""

R_TEMPLATE = r"""//! tu::@STEM@ vs the C (gen/cref/tu/@STEM@.c: the whole TU, shipping contraction; generated by
//! gen/cref/gen_drv.py, regenerate with `python rust/melee_world/gen/cref/run_gr.py @STEM@`).
#![allow(static_mut_refs, non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
mod mtu_harness;
use core::ffi::c_void;
use melee_types::*;
use melee_world::tu::@STEM@ as m;
use mtu_harness::*;
use rgx as rg;
#[path = "mtu/@STEM@_gen.rs"]
mod g;

static mut GO: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };
static mut GR: [Ground; 4] = unsafe { core::mem::zeroed() };
static mut JP: [HSD_JObj; 16] = unsafe { core::mem::zeroed() };
static mut CD: CollData = unsafe { core::mem::zeroed() };
static mut V3: [Vec; 4] = unsafe { core::mem::zeroed() };
static mut USELH: i32 = 0;
pub fn tv(p: *const Vec) {
    if !p.is_null() {
        unsafe { t((*p).x.to_bits()); t((*p).y.to_bits()); t((*p).z.to_bits()); }
    }
}
pub fn wv(p: *mut Vec) {
    if !p.is_null() {
        unsafe { (*p).x = rgx(); (*p).y = rgx(); (*p).z = rgx(); }
    }
}
pub fn rgx() -> f32 {
    let sel = rnd() & 3;
    if sel == 0 { ((rnd() >> 8) as f32 / 16777216.0 - 0.5) * 4.0 } else { mtu_harness::rg() }
}
@DECL@

unsafe fn reset_world() {
    GO = core::mem::zeroed();
    GR = core::mem::zeroed();
    JP = core::mem::zeroed();
    CD = core::mem::zeroed();
    V3 = core::mem::zeroed();
    for i in 0..4 { g::fill_HSD_GObj(&mut GO[i]); }
    for i in 0..4 { g::fill_Ground(&mut GR[i]); }
    for i in 0..16 { g::fill_HSD_JObj(&mut JP[i]); JP[i].flags &= !(JOBJ_USE_QUATERNION as u32); }
    g::fill_CollData(&mut CD);
    for i in 0..4 { V3[i].x = rw(); V3[i].y = rw(); V3[i].z = rw(); }
    for i in 0..4 {
        GO[i].user_data = &raw mut GR[i] as *mut c_void;
        GR[i].gobj = &raw mut GO[i];
        GO[i].hsd_obj = &raw mut JP[i * 4] as *mut c_void;
    }
    for i in 0..16usize {
        JP[i].child = if 2 * i + 1 < 16 { &raw mut JP[2 * i + 1] } else { core::ptr::null_mut() };
        JP[i].next = if i % 4 != 3 && i + 1 < 16 { &raw mut JP[i + 1] } else { core::ptr::null_mut() };
        JP[i].parent = if i > 0 { &raw mut JP[(i - 1) / 2] } else { core::ptr::null_mut() };
    }
@SETUP@
}

unsafe fn hash_world() {
    for i in 0..4 { g::hash_HSD_GObj(&GO[i]); }
    for i in 0..4 { g::hash_Ground(&GR[i]); }
    for i in 0..16 { g::hash_HSD_JObj(&JP[i]); }
    g::hash_CollData(&CD);
    for i in 0..4 { hm(V3[i].x.to_bits()); hm(V3[i].y.to_bits()); hm(V3[i].z.to_bits()); }
@HASH@
}

const NCASES: [u32; @NOPS@] = [@NCASES@];

#[test]
fn @STEM@_matches_c() {
    unsafe {
        g::install();
        preg(&raw const GO, core::mem::size_of_val(&GO));
        preg(&raw const GR, core::mem::size_of_val(&GR));
        preg(&raw const JP, core::mem::size_of_val(&JP));
        preg(&raw const CD, core::mem::size_of_val(&CD));
        preg(&raw const V3, core::mem::size_of_val(&V3));
@PREG@
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_@STEM@.txt"));
        let mut total = 0;
        for op in 0..@NOPS@u32 {
            for k in 0..NCASES[op as usize] {
                total += 1;
                case(op, k);
                let mut ret = 0u32;
                match op {
@CASES@
                    _ => unreachable!(),
                }
                hash_world();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), total);
    }
}
"""


def indent(s, n):
    return "\n".join((" " * n + l) if l else l for l in s.split("\n"))


def emit(stem, spec, ops, skipped):
    ncases = ", ".join(str(o[6]) for o in ops)
    casesc, casesr = [], []
    for i, (n, usel, pc, pr, bc, br, nc) in enumerate(ops):
        casesc.append(f"            case {i}: /* {n} */ {{\n                int i;\n                (void) i;\n                USEL = {usel[0]}; USELH = {usel[1]};\n"
                      f"                reset_world();\n                if (TRACEON) {{ u32 hs_ = H; HN(\"PRE\"); hash_world(); H = hs_; }}\n"
                      f"{indent(pc, 16) + chr(10) if pc else ''}{indent(bc, 16)}\n                break;\n            }}")
        casesr.append(f"                    {i} => {{ // {n}\n                        USEL = {usel[0]}; USELH = {usel[1]};\n                        reset_world();\n"
                      f"                        if TRACEON {{ let hs = H; hn(\"PRE\"); hash_world(); H = hs; }}\n"
                      f"{indent(pr, 24) + chr(10) if pr else ''}{indent(br, 24)}\n                    }}")
    sub = {
        "@STEM@": stem, "@SRC@": spec["src"],
        "@DECL@": "", "@SETUP@": "", "@HASH@": "", "@PREG@": "",
        "@NCASES@": ncases, "@NOPS@": str(len(ops)), "@CASES@": "\n".join(casesc),
    }
    c = C_TEMPLATE
    subc = dict(sub, **{"@DECL@": spec.get("decl_c", ""), "@SETUP@": indent(spec.get("setup_c", ""), 4),
                        "@HASH@": indent(spec.get("hash_c", ""), 4), "@PREG@": indent(spec.get("preg_c", ""), 4)})
    for k, v in subc.items():
        c = c.replace(k, v)
    (HERE / "tu" / f"{stem}.c").write_text(c, encoding="utf-8", newline="\n")
    r = R_TEMPLATE
    subr = dict(sub, **{"@DECL@": spec.get("decl_r", ""), "@SETUP@": indent(spec.get("setup_r", ""), 4),
                        "@HASH@": indent(spec.get("hash_r", ""), 4), "@PREG@": indent(spec.get("preg_r", ""), 8),
                        "@CASES@": "\n".join(casesr)})
    for k, v in subr.items():
        r = r.replace(k, v)
    (ROOT / "rust/melee_world/tests" / f"tu_{stem}.rs").write_text(r, encoding="utf-8", newline="\n")


def main(stems):
    import run_tu
    import specs_gr
    gen_only = False
    if stems and stems[0] == "--gen-only":
        gen_only, stems = True, stems[1:]
    for n in stems:
        spec = specs_gr.SPECS[n]
        ops, skipped = build_driver(n, spec)
        print(f"gen_drv: {n}: {len(ops)} ops, skipped: {skipped}")
        if gen_only:
            continue
        # run_tu.main() runs gen_tu.build(n, SPECS[n]) itself: hand it the augmented spec
        specs_gr.SPECS[n] = prep(n, spec)[0]
        saved = sys.argv
        sys.argv = ["run_tu", n]
        try:
            run_tu.main()
        finally:
            sys.argv = saved
