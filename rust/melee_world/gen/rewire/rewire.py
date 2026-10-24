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


class NoVariant(Exception):
    pass


def pick(spec, params):
    """A spec is one (path, mode) or a list of (arity, first-type-prefix, (path, mode)) variants."""
    if isinstance(spec, list):
        for ar, ft, sp in spec:
            fts = ft if isinstance(ft, list) else [ft]
            ok = ar is None or ar == len(params)
            for i, x in enumerate(fts):
                if x and not (len(params) > i and params[i][1].replace(" ", "").startswith(x)):
                    ok = False
            if ok:
                return sp
        raise NoVariant()
    return spec


def body_for(spec, params, has_ret):
    spec = pick(spec, params)
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
    spec = pick(spec, params)
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
        sp = (path.strip(), mode.strip() or "cast")
        name = name.strip()
        if "/" in name:
            name, _, var = name.partition("/")
            ar, _, ft = var.partition(":")
            specs.setdefault(name, []).append((int(ar) if ar else None, [x.replace(" ", "") or None for x in ft.split(":")], sp))
        else:
            specs[name] = sp
    return specs


def match_paren(s, i):
    """s[i] == '(' -> index of the matching ')' (skipping '->' arrows)."""
    d = 0
    while i < len(s):
        c = s[i]
        if c == "-" and s[i + 1:i + 2] == ">":
            i += 2
            continue
        if c == "(":
            d += 1
        elif c == ")":
            d -= 1
            if d == 0:
                return i
        i += 1
    return -1


def rewire_line(line, name, spec, in_ext_fns, counter, local_extern=False):
    """Return the rewritten line or None."""
    st = line.lstrip()
    ind = line[: len(line) - len(st)]
    # F1: hook!(H, name, (params) -> R);
    m = re.match(r"hook!\((\w+),\s*" + re.escape(name) + r",\s*\(", st)
    if m:
        op = m.end() - 1
        cp = match_paren(st, op)
        rest = st[cp + 1:]
        rm = re.match(r"\s*(?:->\s*(.+?))?\);\s*$", rest)
        if cp > 0 and rm:
            plist = st[op + 1:cp]
            ret = rm.group(1)
            r = f" -> {ret}" if ret else ""
            counter[0] += 1
            if local_extern:
                ps = params_of(plist)
                types = ", ".join(t for _, t in ps)
                sig = ", ".join(f"{n}: {t}" for n, t in ps)
                args = ", ".join(n for n, _ in ps)
                h = m.group(1)
                b1 = f"{ind}pub static mut {h}: Option<unsafe extern \"C\" fn({types}){r}> = None;"
                b2 = f"{ind}pub unsafe extern \"C\" fn {name}({sig}){r} {{ match {h} {{ Some(f) => f({args}), None => {body_for(spec, ps, bool(ret))} }} }}"
                return b1 + LF + b2
            return f"{ind}hook_or!({m.group(1)}, {name}, ({plist}){r}, {closure(spec, params_of(plist), ret)});"
    # F2: fn name(params) -> R;   inside ext_fns!
    if in_ext_fns:
        m = re.match(r"fn " + re.escape(name) + r"\(", st)
        if m:
            op = m.end() - 1
            cp = match_paren(st, op)
            rest = st[cp + 1:]
            rm = re.match(r"\s*(?:->\s*([^=;]+?))?\s*;\s*$", rest)
            if cp > 0 and rm:
                plist = st[op + 1:cp]
                ret = rm.group(1)
                r = f" -> {ret}" if ret else ""
                counter[0] += 1
                return f"{ind}fn {name}({plist}){r} = {closure(spec, params_of(plist), ret)};"
    # F3: hand-written one-line hook function
    m = re.search(r"fn " + re.escape(name) + r"\(", st)
    if m and "None => panic!(\"TODO: " + name + " is not ported\")" in st:
        op = m.end() - 1
        cp = match_paren(st, op)
        plist = st[op + 1:cp]
        rm = re.match(r"\s*(?:->\s*([^{]+?))?\s*\{", st[cp + 1:])
        ret = rm.group(1) if rm else None
        counter[0] += 1
        return ind + st.replace("None => panic!(\"TODO: " + name + " is not ported\")", "None => " + body_for(spec, params_of(plist), bool(ret)))
    # F4: a one-line function whose whole body is panic!/unimplemented!/todo!
    m = re.search(r"fn " + re.escape(name) + r"\(", st)
    if m:
        op = m.end() - 1
        cp = match_paren(st, op)
        rm = re.match(r"\s*(?:->\s*([^{]+?))?\s*\{\s*(?:panic|unimplemented|todo)!\(.*\)\s*;?\s*\}\s*$", st[cp + 1:])
        if cp > 0 and rm:
            plist = st[op + 1:cp]
            ret = rm.group(1)
            head = st[: cp + 1] + (f" -> {ret}" if ret else "")
            counter[0] += 1
            return f"{ind}{head} {{ {body_for(spec, params_of(plist), bool(ret))} }}"
    return None


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    dry = "--dry" in sys.argv
    specs = load_specs(args[0] if args else Path(__file__).with_name("ports.txt"))
    total = {n: [0] for n in specs}
    for f in sorted(SRC.rglob("*.rs")):
        pf = f.as_posix()
        if "/raw/" in pf or "/c2r/" in pf:
            continue
        raw = f.read_bytes().decode("utf8")
        crlf = CRLF in raw
        lines = raw.replace(CRLF, LF).split(LF)
        o = list(lines)
        local_extern = bool(re.search(r"macro_rules! hook \{[^}]*?unsafe extern \"C\" fn \$name", LF.join(lines)))
        in_ext = False
        for i, line in enumerate(lines):
            s = line.strip()
            if s.startswith("ext_fns!") and s.endswith("{"):
                in_ext = True
                continue
            if in_ext and s == "}":
                in_ext = False
                continue
            if not (s.startswith("fn ") or s.startswith("hook!(") or "is not ported" in s or ("fn " in s and ("panic!(" in s or "unimplemented!(" in s or "todo!(" in s))):
                continue
            for name, spec in specs.items():
                if name not in s:
                    continue
                if "is not ported" in s and "fn " not in s and i > 0 and ("fn " + name + "(") in lines[i - 1] and lines[i - 1].rstrip().endswith("{"):
                    joined = lines[i - 1].rstrip() + " " + s
                    try:
                        r = rewire_line(joined, name, spec, in_ext, total[name], local_extern)
                    except NoVariant:
                        r = None
                    if r is not None:
                        lines[i - 1] = r
                        lines[i] = None
                        break
                    continue
                try:
                    r = rewire_line(line, name, spec, in_ext, total[name], local_extern)
                except NoVariant:
                    r = None
                if r is not None:
                    lines[i] = r
                    break
        if True:
            t = LF.join(l for l in lines if l is not None)
            if "hook_or!(" in t and "macro_rules! hook_or" not in t and not re.search(r"hookmac::(\{[^}]*\bhook_or\b[^}]*\}|hook_or\b)", t):
                t2 = re.sub(r"use crate::tu::hookmac::hook;", "use crate::tu::hookmac::{hook, hook_or};", t, count=1)
                if t2 == t:
                    t2 = re.sub(r"use crate::tu::hookmac::\{([^}]*)\};", lambda mm: "use crate::tu::hookmac::{" + mm.group(1).rstrip() + ", hook_or};", t, count=1)
                if t2 == t:
                    print("WARNING: hook_or not importable in", f)
                t = t2
            if "macro_rules! ext_fns" in t and "$def:expr" not in t:
                if "$(= $def:path)?" in t:
                    t = t.replace("$(= $def:path)?", "$(= $def:expr)?").replace(IF_ARM_OLD, IF_ARM_NEW)
                elif MACRO_OLD in t:
                    t = t.replace(MACRO_OLD, MACRO_NEW)
                else:
                    print("WARNING: ext_fns macro not upgradable in", f)
            elif "$(= $def:path)?" in t:
                t = t.replace("$(= $def:path)?", "$(= $def:expr)?").replace(IF_ARM_OLD, IF_ARM_NEW)
            if t != raw.replace(CRLF, LF):
                if not dry:
                    f.write_bytes((t.replace(LF, CRLF) if crlf else t).encode("utf8"))
                print("rewrote", f.relative_to(ROOT).as_posix())
    for n, c in total.items():
        print(f"{n}: {c[0]} sites")


if __name__ == "__main__":
    main()
