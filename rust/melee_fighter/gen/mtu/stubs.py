#!/usr/bin/env python3
"""Recording-stub generator for the melee_fighter TU oracles.

  python rust/melee_fighter/gen/mtu/stubs.py <stem> <game .c> <port .rs> [--ret NAME=C|RUST ...] [--skip NAME ...]
                                              [--only NAME ...] [--notrace NAME:ARGIDX ...] [--outvec NAME:ARGIDX ...] [--libhook CNAME=STATIC ...]

From libclang's view of the game TU (every function it calls but does not define) and the port's `ext_fns!`
hook list (src/tu/<stem>.rs, after gen/mtu/hookify.py) it writes, with the same trace convention on both sides
(gen/cref/tu/mtu_common.h <-> tests/mtu_h/mod.rs: a stub does T(id) then T() of every argument, and returns a
value drawn from the shared xorshift stream):
  gen/cref/tu/<stem>_stubs.h        C: one stub definition per callee
  tests/<stem>_stubs/mod.rs         Rust: `pub fn install()` setting `ext::hook::<name>` for every callee the
                                    port also lists, with the same ids
Pointer arguments are traced by the pool registry (PI / pi), function pointers by null / non-null, strings by
hash, structs by value by a byte hash. Pointer return values are NULL unless --ret gives both expressions.
Computed from C source only: no disc data.
"""
import os, re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_items/gen/mtu"))
os.chdir(ROOT)
import gen_tu as gt  # noqa: E402

ci, K, T = gt.ci, gt.K, gt.T


HOOKS = {}
LIB_HOOKS = {}
LIB_ALIAS = {}


def load_lib_hooks():
    """lower-cased function name -> (STATIC, args, ret) of src/hooks.rs: its macro entries and explicit statics."""
    s = (ROOT / "rust/melee_fighter/src/hooks.rs").read_text(encoding="utf-8")
    for m in re.finditer(r"^\s*([A-Z0-9_]+) (\w+)\((.*?)\)(?: -> ([^=;]+?))?\s*=\s*[^;]*;$", s, re.M):
        stat, fn, args, ret = m.groups()
        LIB_HOOKS[fn.lower()] = (stat, split_args(args), ret.strip() if ret else None)
    for m in re.finditer(r"pub static mut ([A-Z0-9_]+): Option<fn\((.*?)\)(?: -> ([^>]+?))?> = None;", s):
        stat, args, ret = m.groups()
        LIB_HOOKS.setdefault(stat.lower().replace("_", ""), (stat, split_args(args), ret.strip() if ret else None))


def split_args(args):
    parts, depth, cur = [], 0, ""
    for ch in args:
        if ch in "<([":
            depth += 1
        elif ch in ">)]":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur)
    ps = []
    for p in parts:
        n, _, t = p.partition(":")
        ps.append((n.strip(), t.strip()))
    return ps


def rust_decls(path):
    """name -> ([(argname, type)], ret|None) from the `ext_fns! { fn NAME(ARGS) -> R = ...; }` lines of the port."""
    s = Path(path).read_text(encoding="utf-8")
    out = {}
    HOOKS.clear()
    # second seam form: `ext! { STATIC name(args) -> R; }` (the hook is the static itself)
    c = s.find("macro_rules!")
    if c >= 0:
        # entries of any `ext!` / `stubs!` / `extd!` style block: `STATIC name(args) -> R [= default];`
        for m in re.finditer(r"^\s*([A-Z][A-Z0-9_]+) (\w+)\((.*?)\)(?: -> (.+?))?(?: = .*)?;$", s[c:], re.M):
            stat, name, args, ret = m.groups()
            HOOKS[name] = "ext::" + stat
            out[name] = (split_args(args), ret.strip() if ret else None)
    # third seam form: a hand-written `pub static mut H_x: Option<fn(args) -> R> = None;` followed by `pub fn x(..)`
    for m in re.finditer(r"pub static mut (\w+): Option<fn\((.*?)\)(?: -> ([^>]+?))?> = None;\s*\n\s*pub fn (\w+)\(", s):
        stat, args, ret, name = m.groups()
        parts = [((n if t else f"a{i}"), (t or n)) for i, (n, t) in enumerate(split_args(args))]
        out[name] = (parts, ret.strip() if ret else None)
        HOOKS[name] = "ext::" + stat
    a = s.find("ext_fns! {")
    if a < 0:
        return out
    blk = s[a:]
    e = blk.find(chr(10) + "    }" + chr(10))
    if e > 0:
        blk = blk[:e]
    for m in re.finditer(r"^\s*(?:fn )?(\w+)\((.*?)\)(?: -> ([^=;]+?))?(?: =>? |;\s*$)", blk, re.M):
        name, args, ret = m.groups()
        parts, depth, cur = [], 0, ""
        for ch in args:
            if ch in "<([":
                depth += 1
            elif ch in ">)]":
                depth -= 1
            if ch == "," and depth == 0:
                parts.append(cur)
                cur = ""
            else:
                cur += ch
        if cur.strip():
            parts.append(cur)
        ps = []
        for p in parts:
            n, _, t = p.partition(":")
            ps.append((n.strip(), t.strip()))
        out[name] = (ps, ret.strip() if ret else None)
    return out


ALIAS = {}
for _m in re.finditer(r"^pub type (\w+) = ([^;]+);", (ROOT / "rust/melee_types/src/gen.rs").read_text(encoding="utf-8"), re.M):
    ALIAS[_m.group(1)] = _m.group(2).strip()


def load_aliases(path):
    for m in re.finditer(r"^\s*(?:pub )?type (\w+) = ([^;]+);", Path(path).read_text(encoding="utf-8"), re.M):
        ALIAS[m.group(1)] = m.group(2).strip()


def rk(ty):
    t = ty.replace(" ", "")
    for _ in range(6):
        if t in ALIAS:
            t = ALIAS[t].replace(" ", "")
        else:
            break
    if t == "f32":
        return "f4"
    if t == "f64":
        return "f8"
    if t in ("&str", "&'staticstr"):
        return "str"
    if t.startswith("&"):
        return "ref"
    if t.startswith("*"):
        return "ptr"
    if "fn(" in t or "Option<" in t:
        return "fp"
    if t in ("i64", "u64"):
        return "i8"
    if t in ("bool", "i8", "u8", "i16", "u16", "i32", "u32", "isize", "usize", "c_int", "c_uint", "c_char"):
        return "i4"
    return "rec"


def c_trace(cls, a):
    return {"i4": f"T((u32) {a});", "i8": f"T((u32) {a}); T((u32) ((u64_t_) {a} >> 32));", "f4": f"T(B({a}));",
            "f8": f"T64({a});", "fp": f"T({a} != 0);", "str": f"T(SH((const char*) {a}));",
            "ptr": f"T(PI((const void*) {a}));", "ref": f"T(PI((const void*) {a}));",
            "rec": f"T(RH(&{a}, sizeof {a}));"}[cls]


def r_trace(cls, a):
    return {"i4": f"t({a} as u32);", "i8": f"t({a} as u32); t((({a} as u64) >> 32) as u32);", "f4": f"t({a}.to_bits());",
            "f8": f"t64({a});", "fp": f"t(fnn(&{a}));", "str": f"t(sh_str({a}));",
            "ptr": f"t(pi({a}));", "ref": f"t(pi({a} as *const _ as *const u8));", "rec": f"t(rh(&{a}));"}[cls]


def main():
    argv = sys.argv[1:]
    stem, csrc, rsrc = argv[:3]
    notrace, outvec = set(), {}
    rets, skip, only = {}, {"memset", "memcpy", "memmove", "__frsqrte", "sinf", "cosf", "ProfActive", "ProfBegin", "ProfEnd", "ProfCount"}, set()
    i = 3
    while i < len(argv):
        if argv[i] == "--ret":
            n, _, v = argv[i + 1].partition("=")
            c, _, r = v.partition("|")
            rets[n] = (c, r)
            i += 2
        elif argv[i] == "--skip":
            skip.add(argv[i + 1]); i += 2
        elif argv[i] == "--notrace":  # NAME:ARGINDEX -- an address-valued argument (differs run to run): not traced
            n, _, a = argv[i + 1].partition(":")
            notrace.add((n, int(a))); i += 2
        elif argv[i] == "--outvec":  # NAME:ARGIDX -- an out parameter (Vec3*): the stub fills it with three drawn floats
            n, _, a = argv[i + 1].partition(":")
            outvec[n] = int(a); i += 2
        elif argv[i] == "--libhook":  # CNAME=STATIC -- the port reaches this callee through that crate::hooks settable
            n, _, st = argv[i + 1].partition("=")
            LIB_ALIAS[n] = st; i += 2
        elif argv[i] == "--only":
            only.add(argv[i + 1]); i += 2
        else:
            raise SystemExit("bad arg " + argv[i])
    rsrcs = rsrc.split(",")
    rsrc = rsrcs[0]
    load_aliases(rsrc)
    load_lib_hooks()
    load_aliases(str(ROOT / "rust/melee_fighter/src/hooks.rs"))
    decls = rust_decls(rsrc)
    decls_first = set(decls)
    first_hooks = dict(HOOKS)
    extra = []  # (alias, decls, hooks) of the further port modules: m2, m3, ...
    for _i, _p in enumerate(rsrcs[1:], 2):
        load_aliases(_p)
        _d = rust_decls(_p)
        extra.append((f"m{_i}", _d, dict(HOOKS)))
        for _k, _v in _d.items():
            decls.setdefault(_k, _v)
    HOOKS.clear(); HOOKS.update(first_hooks)
    for _a, _d, _h in extra:
        for _k, _v in _h.items():
            HOOKS.setdefault(_k, _v)
    lib_used = []
    tu = gt.Tu(csrc, (), (ROOT / csrc).parent)
    stubs, inlines, data = tu.uses()
    for n in ("__assert", "OSReport", "printf"):
        stubs.pop(n, None)
    names = sorted(n for n in stubs if n not in skip and (not only or n in only))
    c_out = [f"/* generated by gen/mtu/stubs.py for {csrc}: do not edit */"]
    dnames = []
    for dn in sorted(data):
        v = data[dn]
        if v.semantic_parent.kind != K.TRANSLATION_UNIT or dn in tu.defs or dn in tu.main_vars:
            continue
        if v.storage_class == ci.StorageClass.STATIC:  # a file static of an #included .inc: the TU defines it
            continue
        dnames.append(dn)
        toks = [x.spelling for x in v.get_tokens()]
        if toks and toks[0] == "extern":
            toks = toks[1:]
        txt = " ".join(toks).rstrip(";").strip()
        if "{" in toks:  # the declaration carries the struct definition: the headers already have it
            txt = v.type.spelling + " " + dn
            if v.type.kind == ci.TypeKind.CONSTANTARRAY:
                txt = v.type.element_type.spelling + " " + dn + "[%d]" % v.type.element_count
        if "unnamed" in v.type.spelling:  # an anonymous struct type: redeclare through the extern's own type
            txt = "__typeof__(" + dn + ") " + dn
        txt = txt.replace("[ ]", "[160]")
        c_out.append(txt + ";")
    r_out = [f"// generated by rust/melee_fighter/gen/mtu/stubs.py for {csrc}: do not edit",
             "#![allow(unused_variables, unused_parens, non_snake_case, unused_unsafe, static_mut_refs, clippy::all)]", "use crate::mtu_h::*;", ("use super::{" + ", ".join(["m"] + [a for a, _d, _h in extra]) + "};"), "use melee_fighter::hooks as h;", "use melee_types::*;", ""]
    hooks, warn, calls = [], [], []
    for idx, n in enumerate(names):
        decl = stubs[n][0]
        ft = gt.canon(decl.type)
        if ft.is_function_variadic():
            warn.append(f"{n}: variadic (skipped; define it by hand)")
            continue
        sid = 1000 + idx
        ptys = list(ft.argument_types())
        rt = ft.get_result()
        rcls = gt.arg_class(rt)
        params = ", ".join(f"__typeof__({p.spelling}) a{i}" for i, p in enumerate(ptys)) or "void"
        ret_t = "void" if rcls == "void" else f"__typeof__({rt.spelling})"
        rd = decls.get(n)
        if rd is None and n in LIB_ALIAS:
            for _k, _v in list(LIB_HOOKS.items()):
                if _v[0] == LIB_ALIAS[n]:
                    LIB_HOOKS[n.lower()] = _v
        if rd is None and n.lower() in LIB_HOOKS:
            # the port calls this callee through a crate::hooks settable (hooks.rs): install the stub there
            stat, hargs, hret = LIB_HOOKS[n.lower()]
            rd = (hargs, hret)
            HOOKS[n] = "h::" + stat
            lib_used.append(n)
        # C side classes: the Rust declaration decides str / ref / fp when it has one
        body = [f"T({sid});"]
        for i, p in enumerate(ptys):
            cc = gt.arg_class(p)
            if cc == "str":
                cc = "str" if (rd and i < len(rd[0]) and rk(rd[0][i][1]) == "str") else "ptr"
            if cc == "ptr" and rd and i < len(rd[0]) and rk(rd[0][i][1]) == "fp":
                cc = "fp"
            if (n, i) in notrace:
                continue
            body.append(c_trace(cc if cc in ("i4", "i8", "f4", "f8", "fp", "str", "ptr", "rec") else "ptr", f"a{i}"))
        ctype_spell = rt.spelling
        if rcls == "void":
            rc = None
        elif n in rets:
            rc = rets[n][0]
        elif rcls == "i4":
            rc = "(fv() & 1)" if ctype_spell == "bool" else "fv()"
        elif rcls == "i8":
            rc = "fv64()"
        elif rcls == "f4":
            rc = "rg()"
        elif rcls == "f8":
            rc = "(double) rg()"
        elif rcls in ("ptr", "fp", "str"):
            rc = "0"
        else:
            raise SystemExit(f"{n}: returns a struct by value; give --ret")
        if n in outvec:
            o = f"a{outvec[n]}"
            body.append(f"if ({o}) {{ ((Vec3*) {o})->x = rg(); ((Vec3*) {o})->y = rg(); ((Vec3*) {o})->z = rg(); }}")
        if rc is not None:
            body.append(f"return ({ret_t}) ({rc});")
        c_out.append(f"{ret_t} ({n})({params}) {{ " + " ".join(body) + " }")
        c_out.append(f"// stub {n} = {sid}")
        if rd is None:
            # no hookable seam in the port: a callable twin of the stub, for oracle hooks that stand in for an inline
            # (`ftanim_stubs::call_HSD_JObjSetMtxDirtySub(j)`)
            tmap = {"i4": "i32", "i8": "i64", "f4": "f32", "f8": "f64"}
            cps, ctr = [], []
            for i, p in enumerate(ptys):
                cc = gt.arg_class(p)
                cc = cc if cc in tmap else "ptr"
                cps.append(f"a{i}: {tmap.get(cc, '*const core::ffi::c_void')}")
                ctr.append(r_trace({"ptr": "ptr"}.get(cc, cc), f"a{i}") if cc != "ptr" else f"t(pi(a{i}));")
            crt = {"void": "", "i4": " -> i32", "i8": " -> i64", "f4": " -> f32", "f8": " -> f64"}.get(rcls, " -> *mut core::ffi::c_void")
            crv = {"void": "", "i4": "fv() as _", "i8": "fv64() as _", "f4": "rg()", "f8": "rg() as f64"}.get(rcls, "core::ptr::null_mut()")
            calls.append(f"pub fn call_{n}({', '.join(cps)}){crt} {{ t({sid}); " + " ".join(ctr) + f" {crv} }}")
            continue
        rps, rr = rd
        if len(rps) != len(ptys):
            warn.append(f"{n}: C takes {len(ptys)} args, the port's ext declaration {len(rps)}")
            continue
        for i, (an, ty) in enumerate(rps):
            cc, rc_ = gt.arg_class(ptys[i]), rk(ty)
            if {cc, rc_} & {"f4", "f8"} and cc != rc_ and not (cc == "f8" and rc_ == "f8"):
                warn.append(f"{n}: arg {i} is {cc} in C but {ty} in the port (a C conversion the caller must do)")
            elif (cc in ("i4", "i8")) != (rc_ in ("i4", "i8")) and "ptr" not in (cc, rc_) and "fp" not in (cc, rc_):
                warn.append(f"{n}: arg {i} is {cc} in C but {rc_} ({ty}) in the port")
        if rr is not None and rcls not in ("void",) and {rcls, rk(rr)} & {"f4", "f8"} and rcls != rk(rr):
            warn.append(f"{n}: return is {rcls} in C but {rr} in the port")
        rb = [f"t({sid});"]
        for i, (an, ty) in enumerate(rps):
            if (n, i) in notrace:
                continue
            rb.append(r_trace(rk(ty), f"a{i}"))
        if rr is not None:
            rkind = rk(rr)
            if n in rets:
                rv = rets[n][1]
            elif rr.replace(" ", "") == "bool":
                rv = "fv() & 1 != 0" if ctype_spell == "bool" else "fv() != 0"
            elif rkind == "i4":
                rv = "fv() as _"
            elif rkind == "i8":
                rv = "fv64() as _"
            elif rkind == "f4":
                rv = "rg()"
            elif rkind == "f8":
                rv = "rg() as f64"
            elif rkind == "ptr":
                rv = "core::ptr::null_mut()"
            elif rkind == "fp":
                rv = "None"
            else:
                raise SystemExit(f"{n}: Rust returns {rr}; give --ret")
            if n in outvec:  # the C stub fills the out vector before it evaluates the return value
                o = f"a{outvec[n]}"
                rb.append(f"if !{o}.is_null() {{ (*{o}).x = rg(); (*{o}).y = rg(); (*{o}).z = rg(); }}")
            rb.append(rv)
        elif rc is not None and rc != "0" and n not in rets:
            # the C stub returns a drawn value the port's declaration drops: draw it here too so the shared
            # xorshift stream stays in step
            rb.append({"fv()": "let _ = fv();", "(fv() & 1)": "let _ = fv();", "fv64()": "let _ = fv64();",
                       "rg()": "let _ = rg();", "(double) rg()": "let _ = rg();"}.get(rc, ""))
        if n in outvec and rr is None:
            o = f"a{outvec[n]}"
            rb.append(f"if !{o}.is_null() {{ (*{o}).x = rg(); (*{o}).y = rg(); (*{o}).z = rg(); }}")
        args = ", ".join(f"a{i}" for i in range(len(rps)))
        hp = HOOKS.get(n, 'ext::hook::' + n)
        hooks.append(f"        {hp if hp.startswith('h::') else 'm::' + hp} = Some(|{args}| {{ " + " ".join(rb) + " });")
        for _a, _d, _h in extra:
            if n in _d and (n not in decls_first):
                hooks[-1] = hooks[-1].replace("        m::", f"        {_a}::", 1)
            elif n in _d:
                hooks.append(hooks[-1].replace("        m::", f"        {_a}::", 1).replace(" = Some(|", " = Some(|", 1))
    r_out.extend(calls)
    r_out.append("pub fn install() {\n    unsafe {\n" + "\n".join(hooks) + "\n    }\n}\n")
    (HERE.parent / "cref/tu" / f"{stem}_stubs.h").write_text("\n".join(c_out) + "\n", encoding="utf-8", newline="\n")
    d = ROOT / "rust/melee_fighter/tests" / f"{stem}_stubs"
    d.mkdir(parents=True, exist_ok=True)
    (d / "mod.rs").write_text("\n".join(r_out), encoding="utf-8", newline="\n")
    print(f"stubs: {stem}: {len(names)} C stubs, {len(hooks)} Rust hooks; data defined in the C stub header: {dnames}")
    for w in warn:
        print("  WARN " + w)


if __name__ == "__main__":
    main()
