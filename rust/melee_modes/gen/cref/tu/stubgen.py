#!/usr/bin/env python3
"""Generate recording stubs for a TU oracle (C), the TU's `mod ext` block and the Rust test hooks.

  python rust/melee_modes/gen/cref/tu/stubgen.py <stem>     (worktree root; run by run_tu.py)

The driver gen/cref/tu/<stem>.c lists the callees out of the TU in comment lines

    /* STUBS: name  name:ret=null  name:manual  name:default=crate::path::fn  ... */

(any number of such lines; modifiers separated by `;`). Prototypes are looked up in the game
headers (trapstubs.proto), so the generated C definition always matches the declaration the TU
sees. For every stub the generator writes

  gen/cref/tu/stubs_<stem>.h           C: `RET name(args) { T(id); T(<each arg>); return <policy>; }`
                                       (not for `manual`: the driver defines those itself)
  tests/tu_<stem>_stubs/mod.rs         Rust: `install()` sets every non-manual hook to the same trace
  src/tu/<stem>.rs, between the lines `// @ext-begin` and `// @ext-end`:
                                       `pub mod ext { ext_fns! { ... } }` for all stubs

Trace of an argument: integers as sign-/zero-extended u32 words (64-bit as two words), floats as
bits, `char*` as the string's hash, a pointer to a record whose by-value hash exists (the driver's
`/* HS: */` closure) as 0 for NULL else 1 + hs_<Record>(p, 0), any other pointer as null / non-null.
Return policy (default: `rnd` for integers, `bit` for bool, null for pointers, `float` for floats):
ret=rnd | bit | zero | null | small:N | float. The id of a stub is 1000 + its index in the list.
"""
import re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref/tu"))
import trapstubs  # noqa: E402

SCAL = {"s8": ("i8", 1, True), "u8": ("u8", 1, False), "s16": ("i16", 2, True), "u16": ("u16", 2, False),
        "s32": ("i32", 4, True), "u32": ("u32", 4, False), "s64": ("i64", 8, True), "u64": ("u64", 8, False),
        "int": ("i32", 4, True), "unsigned": ("u32", 4, False), "unsigned int": ("u32", 4, False),
        "short": ("i16", 2, True), "unsigned short": ("u16", 2, False), "char": ("i8", 1, True),
        "unsigned char": ("u8", 1, False), "long": ("i32", 4, True), "unsigned long": ("u32", 4, False),
        "ssize_t": ("isize", 8, True), "size_t": ("usize", 8, False), "enum_t": ("i32", 4, True),
        "CharacterKind": ("CharacterKind", 4, False), "StKind": ("StKind", 4, False),
        "bool": ("bool", 4, True), "BOOL": ("bool", 4, True), "float": ("f32", 4, True), "f32": ("f32", 4, True),
        "double": ("f64", 8, True), "f64": ("f64", 8, True)}


def parse_proto(ret, args):
    ps = [] if args.strip() in ("", "void") else [p.strip() for p in args.split(",")]
    out = []
    for i, p in enumerate(ps):
        if p == "...":
            raise ValueError("varargs")
        if "(*" in p:
            raise ValueError("function pointer parameter")
        # drop a trailing parameter name: the last identifier when there are >= 2 identifier words
        words = re.findall(r"[A-Za-z_]\w*|\*", p)
        idw = [w for w in words if w != "*" and w not in ("const", "struct", "enum", "unsigned", "signed")]
        if len(idw) >= 2 and not p.rstrip().endswith("*"):
            p = re.sub(r"\s*\b\w+\s*(\[\d*\])?\s*$", "", p)
        out.append(p.strip())
    return ret.strip(), out


def split_type(t):
    """('base type', n_pointers)"""
    t = re.sub(r"\b(const|volatile|struct|enum|union|extern)\b", "", t)
    n = t.count("*")
    base = " ".join(t.replace("*", " ").split())
    return base, n


def rust_type(t):
    base, n = split_type(t)
    if n == 0:
        if base in SCAL:
            return SCAL[base][0]
        return base
    if base in ("void",):
        inner = "c_void"
    elif base == "char":
        inner = "i8"
    elif base in SCAL:
        inner = SCAL[base][0]
    else:
        inner = base
    s = inner
    for _ in range(n):
        s = f"*mut {s}"
    return s


def find_def(name):
    pat = re.compile(r"^[A-Za-z_][\w \*]*[ \*]" + re.escape(name) + r"\s*\(", re.M)
    for d in ("src/melee", "src/sysdolphin", "pc"):
        for f in (ROOT / d).rglob("*.c"):
            try:
                t = f.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            if name in t and pat.search(t):
                return f.name
    return "?"


def local_proto(name, tu_text):
    """Prototype from a (possibly function-local) `extern` declaration in the TU's own source."""
    pat = re.compile(r"^\s*(?:extern\s+)?([A-Za-z_][\w \*]*?[\s\*])" + re.escape(name) + r"\s*\(([^;{]*?)\)\s*;", re.M | re.S)
    m = pat.search(tu_text)
    if not m:
        raise SystemExit(f"no prototype for {name}")
    return m.group(1).strip(), " ".join(m.group(2).split())


def main(stem, hs_known=None):
    src = (HERE / f"{stem}.c").read_text(encoding="utf-8")
    tu_path = ROOT / src.splitlines()[0].split("TU:")[1].split("*/")[0].strip()
    tu_text = tu_path.read_text(encoding="utf-8", errors="replace")
    specs = []
    for m in re.finditer(r"/\*\s*STUBS:(.*?)\*/", src, re.S):
        for tok in m.group(1).split():
            name, _, mods = tok.partition(":")
            d = {"name": name, "ret": None, "manual": False, "default": None, "proto": None}
            for md in [x for x in mods.split(";") if x]:
                if md == "manual":
                    d["manual"] = True
                elif md.startswith("ret="):
                    d["ret"] = md[4:]
                elif md == "boolint":
                    d["boolint"] = True
                elif md.startswith("proto="):
                    d["proto"] = md[6:].replace("~", " ")
                elif md.startswith("default="):
                    d["default"] = md[8:]
                else:
                    raise SystemExit(f"stubgen: bad modifier {md}")
            specs.append(d)
    if not specs:
        return
    # records with a by-value hash (the driver's HS closure): read back from the generated header
    hs = set()
    hp = HERE / f"hs_{stem}.h"
    if hp.exists():
        hs = set(re.findall(r"static u32 hs_(\w+)\(", hp.read_text(encoding="utf-8")))
    mod = stem
    c_out = [f"/* generated by stubgen.py from the STUBS lines of {stem}.c */"]
    r_hooks = []
    ext_lines = []
    pools, pool_total = [], 0
    for idx, s in enumerate(specs):
        name = s["name"]
        sid = 1000 + idx
        if s["proto"]:
            rr_, _, aa_ = s["proto"].partition("(")
            pr = (rr_.strip(), aa_.rstrip(")").strip())
        else:
            try:
                pr = trapstubs.proto(name)
            except SystemExit:
                pr = local_proto(name, tu_text)
        ret, args = parse_proto(*pr)
        rbase, rn = split_type(ret)
        rret = None if (ret == "void") else rust_type(ret)
        rargs = [("i32" if (s.get("boolint") and split_type(a) == ("bool", 0)) else rust_type(a)) for a in args]
        pnames = [f"a{i}" for i in range(len(args))]
        # ---- ext line
        deffile = find_def(name)
        if s["default"]:
            dflt = s["default"]
        else:
            ph = ", ".join(["_"] * len(args))
            dflt = f'|{ph}| todo!("TODO(port) {name} ({deffile})")' if args else f'|| todo!("TODO(port) {name} ({deffile})")'
        sig = ", ".join(f"{p}: {t}" for p, t in zip(pnames, rargs))
        ext_lines.append(f"        {name}({sig}){' -> ' + rret if rret else ''} = {dflt};")
        # ---- trace statements
        ctr, rtr = [f"T({sid});"], [f"t({sid});"]
        for p, a in zip(pnames, args):
            base, n = split_type(a)
            if n == 0:
                if base in SCAL and base not in ("f32", "float", "f64", "double"):
                    rt, sz, sg = SCAL[base]
                    if sz <= 4:
                        ctr.append(f"T((u32){'(s32)' if sg else ''}{p});")
                        rtr.append(f"t({'(' + p + ' as u32)' if base in ('bool', 'BOOL') else p + (' as i32' if sg else '')} as u32);")
                    else:
                        ctr.append(f"T((u32)(u64){p}); T((u32)((u64){p} >> 32));")
                        rtr.append(f"t({p} as u64 as u32); t(({p} as u64 >> 32) as u32);")
                elif base in ("f32", "float"):
                    ctr.append(f"T(B({p}));")
                    rtr.append(f"t({p}.to_bits());")
                elif base in ("f64", "double"):
                    ctr.append(f"{{ union {{ f64 f; u64 u; }} c_; c_.f = {p}; T((u32) c_.u); T((u32) (c_.u >> 32)); }}")
                    rtr.append(f"t({p}.to_bits() as u32); t(({p}.to_bits() >> 32) as u32);")
                else:  # enum typedef or unknown scalar
                    ctr.append(f"T((u32){p});")
                    rtr.append(f"t({p} as u32);")
            elif n == 1 and base == "char":
                ctr.append(f"T({p} ? HBUF({p}, (unsigned) __builtin_strlen({p})) + 1 : 0);")
                rtr.append(f"t(if {p}.is_null() {{ 0 }} else {{ let s = core::ffi::CStr::from_ptr({p} as *const _).to_bytes(); hbuf(s.as_ptr(), s.len()).wrapping_add(1) }});")
            elif n == 1 and base in hs:
                ctr.append(f"T({p} ? hs_{base}({p}, 0) + 1 : 0);")
                rtr.append(f"t(if {p}.is_null() {{ 0 }} else {{ hs_{base}(&*{p}, 0).wrapping_add(1) }});")
            else:
                ctr.append(f"T({p} != 0);")
                rtr.append(f"t(!({p} as *const u8).is_null() as u32);")
        # ---- return policy
        pol = s["ret"]
        if ret == "void":
            cret = rret_ = ""
        else:
            if pol is None:
                pol = "null" if rn > 0 else ("bit" if rbase in ("bool", "BOOL") else ("float" if rbase in ("f32", "float") else "rnd"))
            if pol == "null":
                cret, rret_ = "return 0;", "core::ptr::null_mut()"
            elif pol == "zero":
                cret, rret_ = "return 0;", ("false" if rret == "bool" else ("0.0" if rret in ("f32", "f64") else "0"))
            elif pol == "bit":
                cret, rret_ = f"return ({ret})(rnd() & 1);", ("(rnd() & 1) != 0" if rret == "bool" else f"(rnd() & 1) as {rret}")
            elif pol.startswith("small:"):
                n_ = int(pol[6:])
                cret, rret_ = f"return ({ret})(rnd() % {n_});", f"(rnd() % {n_}) as {rret}"
            elif pol == "str":
                cret, rret_ = f'return (rnd() & 1) ? ({ret}) "mexstr" : ({ret}) 0;', f'if rnd() & 1 != 0 {{ b"mexstr\\0".as_ptr() as {rret} }} else {{ core::ptr::null_mut() }}'
            elif pol == "ptrbit":
                cret, rret_ = f"return (rnd() & 1) ? ({ret}) (size_t) 1 : ({ret}) 0;", f"if rnd() & 1 != 0 {{ 1usize as {rret} }} else {{ core::ptr::null_mut() }}"
            elif pol.startswith("fpos:"):
                n_ = int(pol[5:])
                cret, rret_ = f"return (float) (rnd() % {n_}u) / 8.0f;", f"((rnd() % {n_}) as f32) / 8.0"
            elif pol.startswith("m1:"):
                n_ = int(pol[3:])
                cret, rret_ = f"return ({ret}) ((int) (rnd() % {n_}u) - 1);", f"((rnd() % {n_}) as i32 - 1) as {rret}"
            elif pol.startswith("pool:") or pol.startswith("pooln:"):
                nullable = pol.startswith("pooln:")
                pol = "pool:" + pol.split(":", 1)[1]
                parts = pol[5:].split(":")
                stride = int(parts[0])
                idx = None
                for pn, a_ in zip(pnames, args):
                    b_, n_ = split_type(a_)
                    if n_ == 0 and b_ in SCAL and b_ not in ("f32", "float", "f64", "double"):
                        idx = pn
                        break
                slots = int(parts[1]) if len(parts) > 1 else (64 if idx else 1)
                off = (pool_total + 15) // 16 * 16
                k_ = len(pools)
                pools.append((off, stride * slots))
                pool_total = off + stride * slots
                ix = f"(((u32)(s32){idx}) % {slots}u) * {stride}" if idx else "0"
                rx = f"((({idx} as i32 as u32) % {slots}) as usize) * {stride}" if idx else "0"
                if nullable:
                    cret = f"if (rnd() % 4u == 0) return ({ret}) 0; pool_touch({k_}); return ({ret})(POOLB + {off} + {ix});"
                    rret_ = f"{{ if rnd() % 4 == 0 {{ return core::ptr::null_mut(); }} pool_touch({k_}); POOLB.0.as_mut_ptr().add({off} + {rx}) as {rret} }}"
                else:
                    cret = f"pool_touch({k_}); return ({ret})(POOLB + {off} + {ix});"
                    rret_ = f"{{ pool_touch({k_}); POOLB.0.as_mut_ptr().add({off} + {rx}) as {rret} }}"
            elif pol == "float":
                cret, rret_ = "return rg();", "rg()"
            elif pol == "rnd":
                if rret in ("u64", "i64"):
                    cret = f"{{ u32 lo_ = rnd(); u32 hi_ = rnd(); return ({ret}) (((u64) hi_ << 32) | lo_); }}"
                    rret_ = f"{{ let lo = rnd() as u64; let hi = rnd() as u64; ((hi << 32) | lo) as {rret} }}"
                elif rret == "bool":
                    cret, rret_ = f"return ({ret})rnd();", "rnd() != 0"
                else:
                    cret, rret_ = f"return ({ret})rnd();", f"rnd() as {rret}"
            else:
                raise SystemExit(f"stubgen: bad ret policy {pol}")
        # ---- C definition
        if not s["manual"]:
            cargs = ", ".join(f"{a} {p}" for a, p in zip(args, pnames)) or "void"
            c_out.append(f"{ret} {name}({cargs}) {{ {' '.join(ctr)} {cret} }}")
            body = " ".join(rtr)
            closure_args = ", ".join(pnames)
            retv = f" {rret_} " if ret != "void" else ""
            r_hooks.append(f"        h::{name} = Some(|{closure_args}| {{ {body}{retv}}});")
    NP = len(pools)
    if NP:
        blk = ["/* lazily filled scratch memory behind the pointer-returning stubs: each region is filled from the case's generator at its first use */",
               f"static unsigned char POOLB[{pool_total}] __attribute__((aligned(16)));",
               f"static unsigned char POOLT[{NP}];",
               "static const unsigned POOL_OFF[%d] = { %s };" % (NP, ", ".join(str(o) for o, _ in pools)),
               "static const unsigned POOL_SZ[%d] = { %s };" % (NP, ", ".join(str(z) for _, z in pools)),
               f"static void pool_reset(void) {{ int i; for (i = 0; i < {NP}; i++) POOLT[i] = 0; }}",
               "static void pool_touch(int k) { if (!POOLT[k]) { POOLT[k] = 1; fillb(POOLB + POOL_OFF[k], POOL_SZ[k]); } }",
               f"static u32 pool_hash(void) {{ u32 h = 0; int i; for (i = 0; i < {NP}; i++) if (POOLT[i]) h = h * 31u + HBUF(POOLB + POOL_OFF[i], POOL_SZ[i]) + (u32) i; return h; }}"]
        c_out = c_out[:1] + blk + c_out[1:]
    (HERE / f"stubs_{stem}.h").write_text("\n".join(c_out) + "\n", encoding="utf-8", newline="\n")
    outd = ROOT / f"rust/melee_modes/tests/tu_{stem}_stubs"
    outd.mkdir(parents=True, exist_ok=True)
    rs = ["// generated by stubgen.py from the STUBS lines of gen/cref/tu/" + stem + ".c: the recording hooks",
          "#![allow(non_snake_case, unused_variables, unused_unsafe, unused_parens, dead_code, unused_imports, clippy::all)]",
          "use crate::tu_gm_harness::*;",
          f"use crate::tu_{stem}_hs::*;" if hs else "",
          "use melee_types::*;", "use core::ffi::c_void;",
          f"use melee_modes::tu::{mod}::ext::hook as h;",
          *(([
              "#[repr(C, align(16))] pub struct PoolB(pub [u8; %d]);" % pool_total,
              "pub static mut POOLB: PoolB = PoolB([0; %d]);" % pool_total,
              "pub static mut POOLT: [bool; %d] = [false; %d];" % (NP, NP),
              "const POOL_OFF: [usize; %d] = [%s];" % (NP, ", ".join(str(o) for o, _ in pools)),
              "const POOL_SZ: [usize; %d] = [%s];" % (NP, ", ".join(str(z) for _, z in pools)),
              "pub fn pool_reset() { unsafe { POOLT = [false; %d]; } }" % NP,
              "pub fn pool_touch(k: usize) { unsafe { if !POOLT[k] { POOLT[k] = true; for i in 0..POOL_SZ[k] { POOLB.0[POOL_OFF[k] + i] = (rnd() >> 11) as u8; } } } }",
              "pub fn pool_hash() -> u32 { let mut h = 0u32; unsafe { for i in 0..%d { if POOLT[i] { h = h.wrapping_mul(31).wrapping_add(hbuf(POOLB.0.as_ptr().add(POOL_OFF[i]), POOL_SZ[i])).wrapping_add(i as u32); } } } h }" % NP,
          ]) if NP else []),
          "pub fn install() {", "    unsafe {", *r_hooks, "    }", "}"]
    (outd / "mod.rs").write_text("\n".join(rs) + "\n", encoding="utf-8", newline="\n")
    # ---- ext block into the Rust TU between the markers
    tu = ROOT / f"rust/melee_modes/src/tu/{stem}.rs"
    if tu.exists():
        text = tu.read_text(encoding="utf-8")
        a, b = text.find("// @ext-begin"), text.find("// @ext-end")
        if a >= 0 and b > a:
            block = ("// @ext-begin (generated by gen/cref/tu/stubgen.py from the STUBS lines of the C oracle driver; "
                     "edit the driver's list, not this block)\n"
                     "/// Calls out of the TU. `ext::hook::<name>` overrides; unset = the port, or a TODO panic.\n"
                     "pub mod ext {\n    use crate::tu::ext_fns;\n    use core::ffi::c_void;\n    use melee_types::*;\n    ext_fns! {\n"
                     + "\n".join(ext_lines) + "\n    }\n}\n")
            text = text[:a] + block + text[b:]
            tu.write_text(text, encoding="utf-8", newline="\n")
    print(f"stubgen: {stem}: {len(specs)} stubs")


if __name__ == "__main__":
    for s in sys.argv[1:]:
        main(s)
