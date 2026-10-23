"""Pool-based state-oracle drivers for the melee_world::tu `if` ports (shard 625-632).

gen_if.py (a copy of gen_tu.py) writes the recording stubs and the field-by-field fill/hash of
every state record; this module adds, from the same spec, what the stage TUs wrote by hand:

  pools    {Type: n}   n records of Type (`P_<Type>[n]`), all filled, zeroed first; a pointer
                        field named by a `rules` entry points into a pool (`pick_<Type>(nullable)`)
  statics  [dict]      TU-owned (or extern) data that is filled and hashed every case
  ops      [..]        one entry per function: the C driver and the Rust test call it with
                        arguments drawn identically on both sides (see ARG TOKENS)

ARG TOKENS (per parameter; default by C type): v = fv(); i:N = rnd() % N; r:lo:hi; f = rg(); w = rw();
z = null / 0; p:T / pn:T = pool pick (nullable); k:VAL = literal; ("raw", c_expr, rs_expr).
Everything else (setup that links the pools into the shapes a function expects) is `setup_c` /
`setup_rs` of the spec, run after the fills and before the operation.
"""
import os
from pathlib import Path

import gen_if as G

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
N_CASES = 400


def cname(tu, tn):
    return ("struct " + tn) if tn in tu.tagged else tn


def pool_decls(tu, spec):
    c, r = [], []
    for tn, n in spec.get("pools", {}).items():
        cn = cname(tu, tn)
        c.append(f"static {cn} P_{tn}[{n}];")
        c.append(f"static {cn}* pick_{tn}(int nul) {{ if (nul) {{ u32 r = rnd(); if ((r & 3) == 0) return 0; }} "
                 f"return &P_{tn}[rnd() % {n}u]; }}")
        r.append(f"pub static mut P_{tn}: [{tn}; {n}] = unsafe {{ core::mem::zeroed() }};")
        r.append(f"pub fn pick_{tn}(nul: i32) -> *mut {tn} {{ unsafe {{ if nul != 0 && rnd() & 3 == 0 {{ "
                 f"return core::ptr::null_mut(); }} &raw mut P_{tn}[(rnd() % {n}) as usize] }} }}")
    return c, r


def _static_paths(st, data):
    """(C expression, Rust expression) of a static entry."""
    c = st["c"]
    rs = st.get("rs") or ("m::ext::" + c if st.get("ext") else "m::" + c)
    return c, rs


def helpers(tu, spec, c_out, r_tail, data):
    """reset_state / hash_state / hash_init_state / preg_state (C and Rust).

    statics entries: c (C name), rs (Rust path; default m::<c>, m::ext::<c> when ext), ty (record type: filled and
    hashed field by field) or kind (ptr | int | f32 scalar, hashed by value / region id), n (array length), flat
    (C element type, Rust element type: a multi-dimensional array walked as n flat elements), fill (arg token),
    keep (constant table: never zeroed or filled), init (hashed once before the first case: checks the
    initializer), region (registered for pointer ids)."""
    pools = spec.get("pools", {})
    sts = spec.get("statics", [])
    c = ["static void reset_state(void)", "{", "    int i;", "    (void) i;"]
    r = ["pub unsafe fn reset_state() {"]
    h = ["static void hash_state(void)", "{", "    int i;", "    (void) i;"]
    rh = ["pub unsafe fn hash_state() {"]
    hi = ["static void hash_init_state(void)", "{", "    int i;", "    (void) i;"]
    rhi = ["pub unsafe fn hash_init_state() {"]
    pc = ["static void preg_state(void)", "{"]
    pr = ["pub unsafe fn preg_state() {"]
    for tn, n in pools.items():
        c.append(f"    zero(P_{tn}, sizeof P_{tn});")
        r.append(f"    P_{tn} = core::mem::zeroed();")
        pc.append(f"    preg(P_{tn}, sizeof P_{tn});")
        pr.append(f"    preg(&raw const P_{tn}, core::mem::size_of_val(&*(&raw const P_{tn})));")

    def elem(st, cn, rs, i):
        """-> (C lvalue, Rust lvalue) of element i (or the whole object when not an array)."""
        n = st.get("n")
        flat = st.get("flat")
        if flat:
            return f"(({flat[0]}*) {cn})[{i}]", f"(*(&raw mut {rs} as *mut {flat[1]}).add({i}))"
        if n:
            return f"{cn}[{i}]", f"{rs}[{i}]"
        return cn, rs

    for st in sts:
        cn, rs = _static_paths(st, data)
        n = st.get("n")
        if (st.get("ty") or st.get("zero") or st.get("kind")) and not st.get("keep"):
            c.append(f"    zero({'' if n else '&'}{cn}, sizeof {cn});")
            r.append(f"    {rs} = core::mem::zeroed();")
        if st.get("ty") or st.get("region"):
            pc.append(f"    preg({'' if n else '&'}{cn}, sizeof {cn});")
            pr.append(f"    preg(&raw const {rs}, core::mem::size_of_val(&*(&raw const {rs})));")
    for tn, n in pools.items():
        c.append(f"    for (i = 0; i < {n}; i++) fill_{tn}(&P_{tn}[i]);")
        r.append(f"    for i in 0..{n}usize {{ fill_{tn}(&mut P_{tn}[i]); }}")
        h.append(f"    for (i = 0; i < {n}; i++) hash_{tn}(&P_{tn}[i]);")
        rh.append(f"    for i in 0..{n}usize {{ hash_{tn}(&P_{tn}[i]); }}")
    for st in sts:
        cn, rs = _static_paths(st, data)
        n = st.get("n")
        ty = st.get("ty")
        kind = st.get("kind")
        keep = st.get("keep")
        if st.get("fill") and not keep:
            ce, re_ = _tok_expr(st["fill"], "i4", "i32", 0)
            re_ = re_.replace("g::pick_", "pick_")
            if n:
                lc, lr = elem(st, cn, rs, "i")
                c.append(f"    for (i = 0; i < {n}; i++) {lc} = ({ce});")
                r.append(f"    for i in 0..{n}usize {{ {lr} = ({re_}) as _; }}")
            else:
                c.append(f"    {cn} = ({ce});")
                r.append(f"    {rs} = ({re_}) as _;")
        if ty:
            if n:
                if not keep:
                    c.append(f"    for (i = 0; i < {n}; i++) fill_{ty}(&{cn}[i]);")
                    r.append(f"    for i in 0..{n}usize {{ fill_{ty}(&mut {rs}[i]); }}")
                hl = (f"    for (i = 0; i < {n}; i++) hash_{ty}(&{cn}[i]);", f"    for i in 0..{n}usize {{ hash_{ty}(&{rs}[i]); }}")
            else:
                if not keep:
                    c.append(f"    fill_{ty}(&{cn});")
                    r.append(f"    fill_{ty}(&mut {rs});")
                hl = (f"    hash_{ty}(&{cn});", f"    hash_{ty}(&{rs});")
            h.append(hl[0])
            rh.append(hl[1])
            if st.get("init"):
                hi.append(hl[0])
                rhi.append(hl[1])
        elif kind:
            hc = {"ptr": "HM(PI((const void*) {x}));", "int": "HM((u32) {x});", "f32": "HM(B({x}));",
                  "fnp": "HM(FI((const void*) {x}));"}[kind]
            hr = {"ptr": "hm(pi({x}));", "int": "hm({x} as u32);", "f32": "hm({x}.to_bits());",
                  "fnp": "hm(fi({x}));"}[kind]
            if n:
                lc, lr = elem(st, cn, rs, "i")
                hl = (f"    for (i = 0; i < {n}; i++) " + hc.format(x=lc), f"    for i in 0..{n}usize {{ " + hr.format(x=lr) + " }")
            else:
                hl = ("    " + hc.format(x=cn), "    " + hr.format(x=rs))
            h.append(hl[0])
            rh.append(hl[1])
            if st.get("init"):
                hi.append(hl[0])
                rhi.append(hl[1])
    he = spec.get("hash_extra")
    if he:
        h.append(he[0])
        rh.append(he[1])
    hi.append("}")
    rhi.append("}")
    c.append("}")
    r.append("}")
    h.append("}")
    rh.append("}")
    pc.append("}")
    pr.append("}")
    c_out.extend(c + h + hi + pc)
    r_tail.extend(r + rh + rhi + pr)


# ---------------------------------------------------------------------------- operations

def _tok_expr(tok, cls, rty, idx):
    """-> (C expression, Rust expression) of one argument token."""
    if tok is None:
        tok = {"i4": "v", "i8": "v", "f4": "f", "f8": "f", "ptr": "z", "fp": "z", "str": "z"}.get(cls, "z")
    if isinstance(tok, tuple):
        assert tok[0] == "raw"
        return tok[1], tok[2]
    if tok == "v":
        return "fv()", "fv()"
    if tok == "f":
        return "rg()", "rg()"
    if tok == "w":
        return "rw()", "rw()"
    if tok == "z":
        return "0", f"core::mem::zeroed::<{rty}>()"
    if tok.startswith("i:"):
        n = int(tok[2:])
        return f"rnd() % {n}u", f"rnd() % {n}"
    if tok.startswith("r:"):
        _, lo, hi = tok.split(":")
        lo, hi = int(lo), int(hi)
        n = hi - lo + 1
        return f"({lo} + (int) (rnd() % {n}u))", f"({lo} + (rnd() % {n}) as i32)"
    if tok.startswith("k:"):
        return tok[2:], tok[2:]
    if tok.startswith("p:"):
        return f"pick_{tok[2:]}(0)", f"g::pick_{tok[2:]}(0)"
    if tok.startswith("pn:"):
        return f"pick_{tok[3:]}(1)", f"g::pick_{tok[3:]}(1)"
    raise SystemExit(f"bad arg token {tok}")


def op_code(tu, spec, op):
    """-> (name, c_stmts, rs_stmts) for one op entry."""
    if isinstance(op, str):
        op = dict(name=op)
    if "c" in op:
        return op.get("label", op["name"]), op["c"], op["rs"]
    name = op["name"]
    fn = tu.defs[name]
    params = list(fn.get_arguments())
    toks = list(op.get("args", [])) + [None] * (len(params) - len(op.get("args", [])))
    decl_c, asg_c, asg_r, names = [], [], [], []
    for i, p in enumerate(params):
        cls = G.arg_class(p.type)
        rty = G.rust_ty(p.type)
        if toks[i] is None and cls == "ptr":
            pt = G.canon(p.type).get_pointee()
            pn = G.canon(pt).get_declaration().spelling or pt.spelling.replace("const ", "").replace("struct ", "").strip()
            if pn in spec.get("pools", {}):
                toks[i] = "p:" + pn
        ce, re_ = _tok_expr(toks[i], cls, rty, i)
        decl_c.append(f"__typeof__({p.type.spelling}) a{i};")
        asg_c.append(f"a{i} = ({p.type.spelling}) ({ce});")
        if toks[i] == "z" or (toks[i] is None and cls in ("ptr", "fp", "str")):
            asg_r.append(f"let a{i}: {rty} = core::mem::zeroed();")
        elif cls in ("ptr",) and isinstance(toks[i], str) and toks[i][:2] in ("p:", "pn") or \
                (isinstance(toks[i], str) and toks[i].startswith(("p:", "pn:"))):
            asg_r.append(f"let a{i}: {rty} = {re_} as {rty};")
        else:
            asg_r.append(f"let a{i}: {rty} = ({re_}) as {rty};")
        names.append(f"a{i}")
    rt = fn.result_type
    rcls = G.arg_class(rt)
    callc = f"{name}({', '.join(names)})"
    callr = f"m::{name}({', '.join(names)})"
    if rcls == "void":
        c_call, r_call = callc + ";", callr + ";"
    elif rcls in ("i4", "i8"):
        c_call, r_call = f"ret = (u32) {callc};", f"ret = {callr} as u32;"
    elif rcls == "f4":
        c_call, r_call = f"ret = B({callc});", f"ret = {callr}.to_bits();"
    elif rcls == "ptr":
        c_call, r_call = f"ret = PI((const void*) {callc});", f"ret = pi({callr});"
    elif rcls == "f8":
        c_call, r_call = f"ret = B((float) {callc});", f"ret = ({callr} as f32).to_bits();"
    else:
        raise SystemExit(f"{name}: return class {rcls} needs a custom op")
    pre_c = op.get("pre_c", "")
    pre_r = op.get("pre_rs", "")
    c = " ".join(decl_c) + " " + pre_c + " " + " ".join(asg_c) + " " + c_call
    r = pre_r + " " + " ".join(asg_r) + " " + r_call
    return op.get("label", name), c, r


def emit_driver(stem, spec, tu):
    ops = [op_code(tu, spec, o) for o in spec["ops"]]
    nops = len(ops)
    ncases = spec.get("ncases", {})
    default_n = spec.get("n_cases", N_CASES)
    src = spec["src"]
    c = [f"/* TU: {src} */",
         "/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a",
         " * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */",
         "#include TU_SRC", '#include "mtu_common.h"', spec.get("c_decl", ""), f'#include "{stem}_gen.h"', "",
         spec.get("c_extra", ""), "",
         "static void setup_extra(void)", "{", "    int i, n;", "    (void) i; (void) n;", spec.get("setup_c", ""), "}", "",
         "int main(void)", "{", "    int op, k;", "    preg_state();", "    freg_all();"]
    for ce, _ in spec.get("extra_freg", []):
        c.append(f"    freg((const void*) {ce});")
    has_init = any(st.get("init") for st in spec.get("statics", []))
    if has_init:
        c += ['    tu_case(99, 0); hash_init_state(); printf("%d %d %08X %08X %08X\\n", 99, 0, 0u, H, TR);']
    c += ["    for (op = 0; op < %d; op++) {" % nops,
          "        static const int NC[] = {" + ", ".join(str(ncases.get(o[0], default_n)) for o in ops) + "};",
          "        for (k = 0; k < NC[op]; k++) {", "            u32 ret = 0;",
          "            tu_case(op, k);", "            reset_state();", "            setup_extra();", "            switch (op) {"]
    for i, (_, cs, _) in enumerate(ops):
        c.append(f"            case {i}: {{ {cs} }} break;")
    c += ["            }", "            hash_state();", '            printf("%d %d %08X %08X %08X\\n", op, k, ret, H, TR);' + (" fflush((void*) 0);" if os.environ.get("MTU_FLUSH") else ""),
          "        }", "    }", "    return 0;", "}", ""]
    (HERE / "tu" / f"{stem}.c").write_text("\n".join(c), encoding="utf-8", newline="\n")

    r = [f"//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/{stem}.c.",
         f"//! src/tu/{stem}.rs vs the C (the whole TU, shipping contraction; regenerate the expected file",
         f"//! with `python rust/melee_world/gen/mtu/run_if.py {stem}`).",
         "#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]",
         "mod mtu_harness;", "use core::ffi::c_void;", "use core::ptr::null_mut;", "use melee_types::*;", f"use melee_world::tu::{stem} as m;",
         "use mtu_harness::*;", f'#[path = "mtu/{stem}_gen.rs"]', "mod g;", "", spec.get("rs_decl", ""), "",
         spec.get("rs_extra", ""), "",
         "unsafe fn setup_extra() {", "    let mut n: u32;", "    let mut i: usize;", spec.get("setup_rs", ""), "}", "",
         "#[test]", f"fn {stem}_matches_c() {{", "    unsafe {", "        g::install();", "        g::preg_state();",
         "        g::freg_all();"]
    for _, re_ in spec.get("extra_freg", []):
        r.append(f"        freg({re_} as *const () as usize);")
    r += [f'        let mut c = Check::new(include_str!("data/mtu_{stem}.txt"));',
          "        let nc: [u32; %d] = [" % nops + ", ".join(str(ncases.get(o[0], default_n)) for o in ops) + "];",
          ("        case(99, 0); g::hash_init_state(); c.line(99, 0, 0);" if has_init else ""),
          f"        for op in 0..{nops}u32 {{", "            for k in 0..nc[op as usize] {", "                case(op, k);",
          "                g::reset_state();", "                setup_extra();",
          "                let mut ret = 0u32;", "                match op {"]
    for i, (_, _, rs) in enumerate(ops):
        r.append(f"                    {i} => {{ {rs} }}")
    r += ["                    _ => unreachable!(),", "                }", "                g::hash_state();",
          "                c.line(op, k, ret);", "            }", "        }",
          f"        assert_eq!(c.done(), {sum(ncases.get(o[0], default_n) for o in ops) + (1 if has_init else 0)});", "    }", "}", ""]
    (ROOT / "rust/melee_world/tests" / f"tu_{stem}.rs").write_text("\n".join(r), encoding="utf-8", newline="\n")
    print(f"if_drv: {stem}: {nops} ops")
