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
    pools = spec.get("pools", {})
    sts = spec.get("statics", [])
    c = ["static void reset_state(void)", "{", "    int i;", "    (void) i;"]
    r = ["pub unsafe fn reset_state() {"]
    h = ["static void hash_state(void)", "{", "    int i;", "    (void) i;"]
    rh = ["pub unsafe fn hash_state() {"]
    pc = ["static void preg_state(void)", "{"]
    pr = ["pub unsafe fn preg_state() {"]
    for tn, n in pools.items():
        c.append(f"    zero(P_{tn}, sizeof P_{tn});")
        r.append(f"    P_{tn} = core::mem::zeroed();")
        pc.append(f"    preg(P_{tn}, sizeof P_{tn});")
        pr.append(f"    preg(&raw const P_{tn}, core::mem::size_of_val(&*(&raw const P_{tn})));")
    for st in sts:
        cn, rs = _static_paths(st, data)
        n = st.get("n")
        if st.get("ty") or st.get("zero") or st.get("kind"):
            if n:
                c.append(f"    zero({cn}, sizeof {cn});")
                r.append(f"    {rs} = core::mem::zeroed();")
            else:
                c.append(f"    zero(&{cn}, sizeof {cn});")
                r.append(f"    {rs} = core::mem::zeroed();")
        if st.get("ty") or st.get("region"):
            amp = "" if n else "&"
            pc.append(f"    preg({amp}{cn}, sizeof {cn});")
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
        if st.get("fill"):
            ce, re_ = _tok_expr(st["fill"], "i4", "i32", 0)
            re_ = re_.replace("g::pick_", "pick_")
            if n:
                c.append(f"    for (i = 0; i < {n}; i++) {cn}[i] = ({ce});")
                r.append(f"    for i in 0..{n}usize {{ {rs}[i] = ({re_}) as _; }}")
            else:
                c.append(f"    {cn} = ({ce});")
                r.append(f"    {rs} = ({re_}) as _;")
        if ty:
            if n:
                c.append(f"    for (i = 0; i < {n}; i++) fill_{ty}(&{cn}[i]);")
                r.append(f"    for i in 0..{n}usize {{ fill_{ty}(&mut {rs}[i]); }}")
                h.append(f"    for (i = 0; i < {n}; i++) hash_{ty}(&{cn}[i]);")
                rh.append(f"    for i in 0..{n}usize {{ hash_{ty}(&{rs}[i]); }}")
            else:
                c.append(f"    fill_{ty}(&{cn});")
                r.append(f"    fill_{ty}(&mut {rs});")
                h.append(f"    hash_{ty}(&{cn});")
                rh.append(f"    hash_{ty}(&{rs});")
        elif kind:
            hc = {"ptr": "HM(PI((const void*) {x}));", "int": "HM((u32) {x});", "f32": "HM(B({x}));"}[kind]
            hr = {"ptr": "hm(pi({x}));", "int": "hm({x} as u32);", "f32": "hm({x}.to_bits());"}[kind]
            if n:
                h.append(f"    for (i = 0; i < {n}; i++) " + hc.format(x=f"{cn}[i]"))
                rh.append(f"    for i in 0..{n}usize {{ " + hr.format(x=f"{rs}[i]") + " }")
            else:
                h.append("    " + hc.format(x=cn))
                rh.append("    " + hr.format(x=rs))
    c.append("}")
    r.append("}")
    h.append("}")
    rh.append("}")
    pc.append("}")
    pr.append("}")
    c_out.extend(c + h + pc)
    r_tail.extend(r + rh + pr)


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
        decl_c.append(f"{p.type.spelling} a{i};")
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
    c += ["    for (op = 0; op < %d; op++) {" % nops,
          "        static const int NC[] = {" + ", ".join(str(ncases.get(o[0], default_n)) for o in ops) + "};",
          "        for (k = 0; k < NC[op]; k++) {", "            u32 ret = 0;",
          "            tu_case(op, k);", "            reset_state();", "            setup_extra();", "            switch (op) {"]
    for i, (_, cs, _) in enumerate(ops):
        c.append(f"            case {i}: {{ {cs} }} break;")
    c += ["            }", "            hash_state();", '            printf("%d %d %08X %08X %08X\\n", op, k, ret, H, TR);',
          "        }", "    }", "    return 0;", "}", ""]
    (HERE / "tu" / f"{stem}.c").write_text("\n".join(c), encoding="utf-8", newline="\n")

    r = [f"//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/{stem}.c.",
         f"//! src/tu/{stem}.rs vs the C (the whole TU, shipping contraction; regenerate the expected file",
         f"//! with `python rust/melee_world/gen/mtu/run_if.py {stem}`).",
         "#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]",
         "mod mtu_harness;", "use core::ptr::null_mut;", "use melee_types::*;", f"use melee_world::tu::{stem} as m;",
         "use mtu_harness::*;", f'#[path = "mtu/{stem}_gen.rs"]', "mod g;", "", spec.get("rs_decl", ""), "",
         spec.get("rs_extra", ""), "",
         "unsafe fn setup_extra() {", "    let mut n: u32;", "    let mut i: usize;", spec.get("setup_rs", ""), "}", "",
         "#[test]", f"fn {stem}_matches_c() {{", "    unsafe {", "        g::install();", "        g::preg_state();",
         "        g::freg_all();"]
    for _, re_ in spec.get("extra_freg", []):
        r.append(f"        freg({re_} as *const () as usize);")
    r += [f'        let mut c = Check::new(include_str!("data/mtu_{stem}.txt"));',
          "        let nc: [u32; %d] = [" % nops + ", ".join(str(ncases.get(o[0], default_n)) for o in ops) + "];",
          f"        for op in 0..{nops}u32 {{", "            for k in 0..nc[op as usize] {", "                case(op, k);",
          "                g::reset_state();", "                setup_extra();",
          "                let mut ret = 0u32;", "                match op {"]
    for i, (_, _, rs) in enumerate(ops):
        r.append(f"                    {i} => {{ {rs} }}")
    r += ["                    _ => unreachable!(),", "                }", "                g::hash_state();",
          "                c.line(op, k, ret);", "            }", "        }",
          f"        assert_eq!(c.done(), {sum(ncases.get(o[0], default_n) for o in ops)});", "    }", "}", ""]
    (ROOT / "rust/melee_world/tests" / f"tu_{stem}.rs").write_text("\n".join(r), encoding="utf-8", newline="\n")
    print(f"if_drv: {stem}: {nops} ops")
