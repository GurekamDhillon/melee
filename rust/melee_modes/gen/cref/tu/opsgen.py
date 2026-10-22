#!/usr/bin/env python3
"""Generate the op table of a TU oracle, in C and Rust, from one spec.

  python rust/melee_modes/gen/cref/tu/opsgen.py <stem>         (worktree root; run by run_tu.py)

The driver gen/cref/tu/<stem>.c carries the spec in one or more `/* OPS: ... */` blocks, one op per line

    name(arg, arg, ...) -> ret [GROUP GROUP]

Argument kinds (evaluated left to right, each draws from the shared xorshift in the same order on both sides):
    T%N          (T) (rnd() % N)                       T@LO..HI    (T) (LO + rnd() % (HI-LO+1))
    T            (T) rnd()                             T=GEN       (T) GEN()   (GEN: ck ckp ckn mode)
    f32          rg()                                  bool        rnd() & 1
    &NAME        the address of the driver's state object NAME
    po0..po3     NULL or &OUT4[n] (one draw)
    {c|rust}     a C expression and the equivalent Rust expression
Return kinds: void, int s32 s8 s16 u8 u16 u32 bool (folded to a u32 as C converts), f32 (bits), str (hash of the
string, 0 for NULL), ptr (non-null), gx (GXColor as a word), selfptr:NAME (== the address of NAME as an s32),
ptreq:NAME (1 if the pointer is &NAME, 2 if another non-null pointer, else 0).
Groups name the state objects the driver fills before the call and hashes after it.

Writes gen/cref/tu/ops_<stem>.h (C: NOPS, op_groups(op), run_op(op)) and tests/tu_<stem>_ops/mod.rs (Rust: the same).
"""
import re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]

RT = {"s8": "i8", "u8": "u8", "s16": "i16", "u16": "u16", "s32": "i32", "u32": "u32", "int": "i32", "bool": "bool", "f32": "f32"}
SIGNED = {"s8", "s16", "s32", "int"}
GENS = {"ck": "ckgen", "ckp": "ckpgen", "ckn": "ckngen", "mode": "modegen"}


def split_args(s):
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "({[":
            depth += 1
        elif ch in ")}]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def arg(tok, i):
    """-> (c_decl, rust_decl) for local a<i>"""
    n = f"a{i}"
    m = re.fullmatch(r"\{(.*)\|(.*)\}", tok)
    if m:
        c, r = m.group(1), m.group(2)
        return f"__typeof__({c}) {n} = {c};", f"let {n} = {r};"
    if tok.startswith("&"):
        nm = tok[1:]
        return f"__typeof__(&{nm}) {n} = &{nm};", f"let {n} = addr_of_mut!({nm});"
    m = re.fullmatch(r"po([0-3])", tok)
    if m:
        k = m.group(1)
        return (f"u8* {n} = (rnd() & 1) ? &OUT4[{k}] : (u8*) 0;",
                f"let {n}: *mut u8 = if rnd() & 1 != 0 {{ addr_of_mut!(OUT4[{k}]) as *mut u8 }} else {{ null_mut() }};")
    if tok == "f32":
        return f"f32 {n} = rg();", f"let {n}: f32 = rg();"
    if tok == "bool":
        return f"bool {n} = (bool) (rnd() & 1);", f"let {n}: bool = (rnd() & 1) != 0;"
    m = re.fullmatch(r"(\w+)%(\d+)", tok)
    if m:
        t, k = m.groups()
        return f"{t} {n} = ({t}) (rnd() % {k}u);", f"let {n}: {RT.get(t, t)} = (rnd() % {k}) as {RT.get(t, t)};"
    m = re.fullmatch(r"(\w+)@(-?\d+)\.\.(-?\d+)", tok)
    if m:
        t, lo, hi = m.group(1), int(m.group(2)), int(m.group(3))
        w = hi - lo + 1
        return (f"{t} {n} = ({t}) ({lo} + (int) (rnd() % {w}u));",
                f"let {n}: {RT.get(t, t)} = ({lo} + (rnd() % {w}) as i32) as {RT.get(t, t)};")
    m = re.fullmatch(r"(\w+)=(\w+)", tok)
    if m:
        t, g = m.groups()
        return f"{t} {n} = ({t}) {GENS[g]}();", f"let {n}: {RT.get(t, t)} = {GENS[g]}() as {RT.get(t, t)};"
    m = re.fullmatch(r"(\w+)", tok)
    if m:
        t = m.group(1)
        return f"{t} {n} = ({t}) rnd();", f"let {n}: {RT.get(t, t)} = rnd() as {RT.get(t, t)};"
    raise SystemExit(f"opsgen: bad argument {tok}")


def ret(kind):
    """-> (c_stmt using r_, rust_stmt using r) producing u32 `ret`"""
    if kind == "void":
        return "ret = 0;", "0u32"
    if kind in ("int", "s32", "s8", "s16"):
        return "ret = (u32) (s32) r_;", "r as i32 as u32"
    if kind in ("u8", "u16", "u32"):
        return "ret = (u32) r_;", "r as u32"
    if kind == "bool":
        return "ret = (u32) (r_ != 0);", "r as u32"
    if kind == "f32":
        return "ret = B(r_);", "r.to_bits()"
    if kind == "str":
        return "ret = r_ ? HBUF(r_, (unsigned) __builtin_strlen(r_)) + 1 : 0;", "if r.is_null() { 0 } else { let s = core::ffi::CStr::from_ptr(r as *const _).to_bytes(); hbuf(s.as_ptr(), s.len()).wrapping_add(1) }"
    if kind == "ptr":
        return "ret = (u32) (r_ != 0);", "!(r as *const u8).is_null() as u32"
    if kind == "gx":
        return "ret = ((u32) r_.r << 24) | ((u32) r_.g << 16) | ((u32) r_.b << 8) | (u32) r_.a;", "((r.r as u32) << 24) | ((r.g as u32) << 16) | ((r.b as u32) << 8) | r.a as u32"
    m = re.fullmatch(r"selfptr:(\w+)", kind)
    if m:
        nm = m.group(1)
        return f"ret = (u32) (r_ == (s32) (size_t) &{nm});", f"(r == addr_of_mut!({nm}) as usize as i32) as u32"
    m = re.fullmatch(r"ptreq:(\w+)", kind)
    if m:
        nm = m.group(1)
        return (f"ret = ((void*) r_ == (void*) &{nm}) ? 1 : (r_ ? 2 : 0);",
                f"if r as *const u8 == addr_of_mut!({nm}) as *const u8 {{ 1 }} else if !(r as *const u8).is_null() {{ 2 }} else {{ 0 }}")
    raise SystemExit(f"opsgen: bad return kind {kind}")


def main(stem):
    src = (HERE / f"{stem}.c").read_text(encoding="utf-8")
    ops = []
    for blk in re.finditer(r"/\*\s*OPS:(.*?)\*/", src, re.S):
        for line in blk.group(1).splitlines():
            line = line.strip()
            if not line:
                continue
            m = re.fullmatch(r"(\w+)\((.*)\)\s*->\s*(\S+)\s*(?:\[([^\]]*)\])?", line)
            if not m:
                raise SystemExit(f"opsgen: bad line {line}")
            ops.append((m.group(1), split_args(m.group(2)), m.group(3), (m.group(4) or "").split()))
    c = ["/* generated by opsgen.py from the OPS blocks of " + stem + ".c */", f"#define NOPS {len(ops)}",
         "static u32 op_groups(int op) { switch (op) {"]
    r = ["// generated by opsgen.py from the OPS blocks of gen/cref/tu/" + stem + ".c",
         "#![allow(non_snake_case, unused_variables, unused_unsafe, unused_mut, unused_parens, dead_code, unused_imports, clippy::all)]",
         "use crate::tu_gm_harness::*;", "use crate::*;", "use core::ptr::{addr_of_mut, null_mut};",
         "use melee_types::*;", f"use melee_modes::tu::{stem} as m;", f"pub const NOPS: u32 = {len(ops)};",
         "pub fn op_groups(op: u32) -> u32 { match op {"]
    for i, (name, args, rk, groups) in enumerate(ops, 1):
        gm = " | ".join(f"G_{g}" for g in groups) or "0"
        c.append(f"case {i}: return {gm};")
        r.append(f"{i} => {gm},")
    c += ["default: return 0; } }", "static u32 run_op(int op)", "{", "    u32 ret = 0;", "    switch (op) {"]
    r += ["_ => 0, } }", "pub unsafe fn run_op(op: u32) -> u32 {", "    let mut ret: u32 = 0;", "    match op {"]
    for i, (name, args, rk, groups) in enumerate(ops, 1):
        decls = [arg(a, j) for j, a in enumerate(args)]
        cs, rs = ret(rk)
        call_c = f"{name}({', '.join(f'a{j}' for j in range(len(args)))})"
        call_r = f"m::{name}({', '.join(f'a{j}' for j in range(len(args)))})"
        c.append(f"    case {i}: {{ " + " ".join(d[0] for d in decls) +
                 (f" __typeof__({call_c}) r_ = {call_c}; {cs}" if rk != "void" else f" {call_c};") + " } break;")
        r.append(f"        {i} => {{ " + " ".join(d[1] for d in decls) +
                 (f" let r = {call_r}; ret = {rs};" if rk != "void" else f" {call_r};") + " }")
    c += ["    default: ret = run_custom(op); break;", "    }", "    return ret;", "}"]
    r += ["        _ => {}", "    }", "    ret", "}"]
    (HERE / f"ops_{stem}.h").write_text("\n".join(c) + "\n", encoding="utf-8", newline="\n")
    outd = ROOT / f"rust/melee_modes/tests/tu_{stem}_ops"
    outd.mkdir(parents=True, exist_ok=True)
    (outd / "mod.rs").write_text("\n".join(r) + "\n", encoding="utf-8", newline="\n")
    print(f"opsgen: {stem}: {len(ops)} ops")


if __name__ == "__main__":
    for s in sys.argv[1:]:
        main(s)
