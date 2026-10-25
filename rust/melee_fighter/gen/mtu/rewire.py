#!/usr/bin/env python3
"""Route a port's direct calls through hookable `ext` seams so the C oracle's recording stubs can stand in for them.

  python rust/melee_fighter/gen/mtu/rewire.py <stem> <src/tu/file.rs>

Needs gen/cref/tu/<stem>_stubs.h and tests/<stem>_stubs/mod.rs from stubs.py. Every C stub that has no Rust hook yet
is looked up in the port: a call `path::fname(args)` whose name matches the C name (hex-named functions match exactly,
the others by suffix) becomes `ext::<CName>(args)`; the ext entry (an `ext_fns!` line with the original callee as the
default, so the running game is unchanged) is derived from the C signature and the callee's own Rust signature.
Pointers cross the seam as `*mut c_void` (crate::mtu_ptr::ptr turns refs / raw pointers into it).
Prints what it rewired and what it left alone.
"""
import re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
FD = ROOT / "rust/melee_fighter"
MACRO = '''    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $d:expr;)*) => {
            pub mod hook {
                use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(unreachable_code, unused_variables)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => $d,
                }
            } )*
        };
    }
'''
norm = lambda s: s.lower().replace("_", "")


def ctype_to_rust(t):
    t = t.strip()
    if "*" in t:
        return "*mut core::ffi::c_void"
    if t in ("float",):
        return "f32"
    if t == "double":
        return "f64"
    if t in ("unsigned int", "unsigned long", "u32", "unsigned"):
        return "u32"
    if t in ("unsigned char", "u8"):
        return "u8"
    if t in ("signed char", "s8", "char"):
        return "i8"
    if t in ("unsigned short", "u16"):
        return "u16"
    if t in ("short", "s16"):
        return "i16"
    if t == "void":
        return None
    return "i32"


def split_top(s):
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "([{<":
            depth += 1
        elif ch in ")]}>":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur)
    return [x.strip() for x in out]


def find_callee(fname):
    pat = re.compile(r"pub (?:unsafe )?(?:extern \"C\" )?fn " + re.escape(fname) + r"\s*(?:<[^>]*>)?\(")
    for p in (FD.parent).glob("*/src/**/*.rs"):
        try:
            t = p.read_text(encoding="utf-8")
        except Exception:
            continue
        m = pat.search(t)
        if m:
            i = m.end()
            depth, j = 1, i
            while depth:
                depth += {"(": 1, ")": -1}.get(t[j], 0)
                j += 1
            params = split_top(t[i:j - 1])
            rest = t[j:j + 200]
            rm = re.match(r"\s*->\s*([^{;]+?)\s*(?:\{|where|;)", rest)
            return params, (rm.group(1).strip() if rm else None)
    return None


def main():
    stem, rsrel = sys.argv[1], sys.argv[2]
    rs_path = FD / rsrel
    s = rs_path.read_text(encoding="utf-8")
    crlf = "\r\n" in s
    s = s.replace("\r\n", "\n")
    h = (FD / f"gen/cref/tu/{stem}_stubs.h").read_text(encoding="utf-8")
    m = (FD / f"tests/{stem}_stubs/mod.rs").read_text(encoding="utf-8")
    have = set(re.findall(r"\{ t\((\d+)\);", m))
    gaps = []
    for line in h.split("\n"):
        mm = re.match(r"(.*?)\((\w+)\)\((.*?)\) \{ T\((\d+)\);", line)
        if mm and mm.group(4) not in have:
            ret, name, args, i = mm.groups()
            args = [re.sub(r"__typeof__\((.*)\)", r"\1", a.strip().rsplit(" a", 1)[0]) for a in split_top(args)] if args != "void" else []
            args = [re.sub(r"^__typeof__\(|\)\s*$", "", a) if a.startswith("__typeof__") else a for a in args]
            ret = re.sub(r"__typeof__\((.*)\)", r"\1", ret.strip())
            gaps.append((name, args, ret))
    ent, done, left = [], [], []
    body_start = s.index("pub mod ext")
    for name, cargs, cret in gaps:
        cands = {}
        for fm in re.finditer(r"((?:[A-Za-z_]\w*::)*)([A-Za-z_]\w*)\(", s):
            full, f = fm.group(1) + fm.group(2), fm.group(2)
            if fm.start() < body_start:
                continue
            if full.startswith(("ext::", "super::ext::")):
                continue
            if norm(f) == norm(name) or (len(norm(f)) >= 7 and norm(name).endswith(norm(f))):
                if "::" in full or True:
                    cands.setdefault(full, f)
        if len(cands) != 1:
            left.append((name, "no unique candidate: " + ", ".join(cands)))
            continue
        full, f = list(cands.items())[0]
        info = find_callee(f)
        if info is None:
            left.append((name, f"callee {f} not found"))
            continue
        cparams, cretrs = info
        rparams = [ctype_to_rust(a) or "i32" for a in cargs]
        names = [f"_a{i}" for i in range(len(cargs))]
        conv = []
        for i, p in enumerate(cparams):
            pt = p.split(":", 1)[1].strip() if ":" in p else p
            a = names[i] if i < len(names) else "0"
            if pt.startswith("&mut "):
                conv.append(f"&mut *({a} as *mut {pt[5:].strip()})")
            elif pt.startswith("&"):
                conv.append(f"&*({a} as *const {pt[1:].strip()})")
            elif pt == "bool":
                conv.append(f"{a} != 0")
            else:
                conv.append(f"{a} as _")
        call = f"{full}({', '.join(conv)})"
        rret = ctype_to_rust(cret)
        outp = len(cargs) == len(cparams) + 1 and cretrs and cretrs != "bool" and rret is None
        if outp:
            # the C writes through a trailing out pointer where the port returns the value
            conv = conv[:len(cparams)]
            call = f"{full}({', '.join(conv)})"
            body = f"{{ unsafe {{ *({names[-1]} as *mut {cretrs}) = {call}; }} }}"
        elif rret is None:
            body = f"{{ unsafe {{ let _ = {call}; }} }}"
        else:
            body = f"{{ unsafe {{ ({call}) as _ }} }}"
        if outp:
            pass
        sig = ", ".join(f"{n}: {t}" for n, t in zip(names, rparams))
        ent.append(f"        fn {name}({sig})" + (f" -> {rret}" if rret else "") + f" = {body};")
        # rewrite the call sites
        pat = re.compile(re.escape(full) + r"\(")
        out, pos = [], 0
        while True:
            mm = pat.search(s, pos)
            if not mm:
                out.append(s[pos:])
                break
            if mm.start() < body_start:
                out.append(s[pos:mm.end()])
                pos = mm.end()
                continue
            i = mm.end()
            depth, j = 1, i
            while depth:
                depth += {"(": 1, ")": -1}.get(s[j], 0)
                j += 1
            args = split_top(s[i:j - 1])
            new = []
            for k, a in enumerate(args):
                ct = cargs[k] if k < len(cargs) else "int"
                if "*" in ct:
                    new.append(f"crate::mtu_ptr::ptr(&({a}))")
                elif ctype_to_rust(ct) == "f32":
                    new.append(f"({a}) as f32")
                elif ctype_to_rust(ct) == "u32":
                    new.append(f"({a}) as u32")
                else:
                    new.append(f"({a}) as i32")
            expr = f"ext::{name}({', '.join(new)})"
            if outp:
                expr = f"unsafe {{ let mut o: {cretrs} = core::mem::zeroed(); ext::{name}({', '.join(new + ['(&mut o as *mut ' + cretrs + ') as *mut core::ffi::c_void'])}); o }}"
            elif cretrs == "bool" and rret:
                expr = f"({expr} != 0)"
            elif cretrs and rret and cretrs not in ("i32",) and "*" not in (cretrs or ""):
                expr = f"({expr} as {cretrs})"
            elif rret == "*mut core::ffi::c_void":
                expr = f"({expr} as {cretrs if cretrs and '*' in cretrs else '_'})"
            out.append(s[pos:mm.start()] + expr)
            pos = j
        s = "".join(out)
        done.append(name)
    if ent:
        marker = "    ext_fns! {\n"
        if marker in s:
            s = s.replace(marker, marker + "\n".join(ent) + "\n", 1)
        else:
            # no macro yet: add one at the end of the ext module
            a = s.index("pub mod ext")
            end = s.index("\n}\n", a)
            s = s[:end] + "\n" + MACRO + "    ext_fns! {\n" + "\n".join(ent) + "\n    }" + s[end:]
    rs_path.write_text(s.replace("\n", "\r\n") if crlf else s, encoding="utf-8", newline="")
    print("rewired:", " ".join(done))
    for n, why in left:
        print("  LEFT", n, "-", why)


if __name__ == "__main__":
    main()
