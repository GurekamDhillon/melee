#!/usr/bin/env python3
"""Rewire the panicking callee stubs of melee_world to a native port.

  python rust/melee_world/gen/rewire/rewire.py [--dry] [ports.txt]     (from the worktree root)

ports.txt lines (blank lines and # comments ignored):

    NAME = crate::path::to::port [| mode]

mode: `cast` (default: every argument `as _`, the result `as _`), `raw` (arguments and result as they
are), `unit` (cast arguments, the port's result is dropped), or `expr:<template>` (a closure body;
`{0}`, `{1}`.. are the stub's parameters, `{all}` the list).

Every stub of NAME in rust/melee_world/src is rewired, in the forms the generators produced:

  * `hook!(H, NAME, (a: T, ..) -> R);`   -> `hook_or!(H, NAME, (..) -> R, |a: T, ..| body);`
  * a `fn NAME(a: T, ..) -> R;` line inside an `ext_fns! {..}` -> `fn NAME(..) -> R = |a: T, ..| body;`
    (the file's `ext_fns!` macro is upgraded to take the optional `= port` closure, as the if* files do)
  * `None => panic!("TODO: NAME is not ported")` in a one-line hand-written hook function -> `None => body`

The unset hook still wins (the oracles install loggers into the hooks); only the unset fallback changes.
"""
import re, sys
from pathlib import Path

CRLF = chr(13) + chr(10)
LF = chr(10)
ROOT = Path(__file__).resolve().parents[4]
SRC = ROOT / "rust/melee_world/src"

MACRO_OLD = (
    "    macro_rules! ext_fns {\n"
    "        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {\n"
    "            pub mod hook {\n"
    "                use super::*;\n"
    "                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*\n"
    "            }\n"
    "            $( pub fn $name($($a: $t),*) $(-> $r)? {\n"
    "                match unsafe { hook::$name } {\n"
    "                    Some(f) => f($($a),*),\n"
    "                    None => panic!(concat!(\"TODO: \", stringify!($name), \" is not ported\")),\n"
    "                }\n"
    "            } )*\n"
    "        };\n"
    "    }\n")

MACRO_NEW = (
    "    macro_rules! ext_none {\n"
    "        ($name:ident, ($($a:ident),*)) => { panic!(concat!(\"TODO: \", stringify!($name), \" is not ported\")) };\n"
    "        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };\n"
    "    }\n"
    "    macro_rules! ext_fns {\n"
    "        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {\n"
    "            pub mod hook {\n"
    "                use super::*;\n"
    "                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*\n"
    "            }\n"
    "            $( pub fn $name($($a: $t),*) $(-> $r)? {\n"
    "                match unsafe { hook::$name } {\n"
    "                    Some(f) => f($($a),*),\n"
    "                    None => ext_none!($name, ($($a),*) $(, $def)?),\n"
    "                }\n"
    "            } )*\n"
    "        };\n"
    "    }\n")

IF_ARM_OLD = "($name:ident, ($($a:ident),*), $def:path) => { $def($($a),*) };"
IF_ARM_NEW = "($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };"


def split_top(s):
    out, d, cur, i = [], 0, "", 0
    while i < len(s):
        c = s[i]
        if s[i:i + 2] == "->":
            cur += "->"
            i += 2
            continue
        if c in "<([":
            d += 1
        elif c in ">)]":
            d -= 1
        if c == "," and d == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += c
        i += 1
    if cur.strip():
        out.append(cur.strip())
    return out


def params_of(plist):
    ps = []
    for p in split_top(plist):
        n, _, t = p.partition(":")
        ps.append((n.strip(), t.strip()))
    return ps


def body_for(spec, params, has_ret):
    path, mode = spec
    names = [n for n, _ in params]
    if mode.startswith("expr:"):
        t = mode[5:]
        for i, n in enumerate(names):
            t = t.replace("{%d}" % i, n)
        return t.replace("{all}", ", ".join(names))
    if mode == "raw":
        return f"unsafe {{ {path}({', '.join(names)}) }}"
    call = f"{path}({', '.join(n + ' as _' for n in names)})"
    if mode == "unit" or not has_ret:
        return f"unsafe {{ let _ = {call}; }}"
    return f"unsafe {{ {call} as _ }}"


def closure(spec, params, ret):
    b = body_for(spec, params, bool(ret))
    return "|" + ", ".join(f"{n}: {t}" for n, t in params) + "| " + ("{ " + b + " }" if ";" in b else b)


def load_specs(p):
    specs = {}
    for line in Path(p).read_text().splitlines():
        if "#" in line and not line.split("=", 1)[-1].lstrip().startswith("expr"):
            line = line.split("#", 1)[0]
        line = line.strip()
        if not line:
            continue
        name, _, rhs = line.partition("=")
        path, _, mode = rhs.strip().partition("|")
        specs[name.strip()] = (path.strip(), mode.strip() or "cast")
    return specs


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    dry = "--dry" in sys.argv
    specs = load_specs(args[0] if args else Path(__file__).with_name("ports.txt"))
    total = {n: 0 for n in specs}
    for f in sorted(SRC.rglob("*.rs")):
        pf = f.as_posix()
        if "/raw/" in pf or "/c2r/" in pf:
            continue
        raw = f.read_bytes().decode("utf8")
        crlf = CRLF in raw
        t = raw.replace(CRLF, LF)
        o = t
        for name, spec in specs.items():
            en = re.escape(name)

            def f1(m):
                total[name] += 1
                ret = m.group(4)
                r = f" -> {ret}" if ret else ""
                return f"hook_or!({m.group(1)}, {name}, ({m.group(3)}){r}, {closure(spec, params_of(m.group(3)), ret)});"
            t = re.sub(r"hook!\((\w+),\s*(" + en + r"),\s*\(([^\n]*?)\)(?:\s*->\s*([^;\n]+?))?\);", f1, t)
            if "macro_rules! ext_fns" in t:
                def f2(m):
                    total[name] += 1
                    ret = m.group(3)
                    r = f" -> {ret}" if ret else ""
                    return f"{m.group(1)}fn {name}({m.group(2)}){r} = {closure(spec, params_of(m.group(2)), ret)};"
                t = re.sub(r"^([ \t]*)fn " + en + r"\(([^\n]*?)\)(?:[ \t]*->[ \t]*([^;\n=]+?))?;[ \t]*$", f2, t, flags=re.M)

            def f3(m):
                total[name] += 1
                return m.group(1) + "None => " + body_for(spec, params_of(m.group(2)), bool(m.group(3)))
            t = re.sub(r"(fn " + en + r"\(([^\n]*?)\)(?:\s*->\s*([^{\n]+?))?\s*\{[^\n]*?)None => panic!\(\"TODO: " + en + r" is not ported\"\)", f3, t)
        if t != o:
            if "macro_rules! ext_fns" in t and "$def:expr" not in t:
                if "$(= $def:path)?" in t:
                    t = t.replace("$(= $def:path)?", "$(= $def:expr)?").replace(IF_ARM_OLD, IF_ARM_NEW)
                elif MACRO_OLD in t:
                    t = t.replace(MACRO_OLD, MACRO_NEW)
                else:
                    print("WARNING: ext_fns macro not upgradable in", f)
            elif "$(= $def:path)?" in t:
                t = t.replace("$(= $def:path)?", "$(= $def:expr)?").replace(IF_ARM_OLD, IF_ARM_NEW)
            if not dry:
                f.write_bytes((t.replace(LF, CRLF) if crlf else t).encode("utf8"))
            print("rewrote", f.relative_to(ROOT).as_posix())
    for n, c in total.items():
        print(f"{n}: {c} sites")


if __name__ == "__main__":
    main()
