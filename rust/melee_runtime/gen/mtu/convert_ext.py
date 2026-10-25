#!/usr/bin/env python3
"""python gen/mtu/convert_ext.py <stem>: replaces the port's hand-written `pub mod ext { .. }` with the generated-ext
markers, harvesting the old `= default` expressions into gen/mtu/defaults_<stem>.json. Prints the old block's
non-hook items (statics, uses) so they can be re-added by hand."""
import json, re, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
stem = sys.argv[1]
p = ROOT / "rust/melee_runtime/src/tu" / f"{stem}.rs"
txt = p.read_text(encoding="utf-8").replace("\r\n", "\n")
if "// BEGIN GENERATED EXT" in txt:
    a = txt.index("// BEGIN GENERATED EXT"); b = txt.index("// END GENERATED EXT") + len("// END GENERATED EXT")
    old = txt[a:b]
else:
    m = re.search(r"(?m)^(\s*)pub mod ext\s*\{", txt)
    if not m:
        NL = chr(10)
        ls = txt.split(NL)
        last = max(i for i, l in enumerate(ls[:40]) if l.startswith("use ") or l.startswith("pub use "))
        # a multi-line use: advance to its terminating ';'
        while not ls[last].rstrip().endswith(";"):
            last += 1
        ls.insert(last + 1, "// BEGIN GENERATED EXT" + NL + "// END GENERATED EXT")
        p.write_text(NL.join(ls), encoding="utf-8", newline=NL)
        print("no ext module: markers inserted after line", last + 1)
        raise SystemExit(0)
    i = m.end(); d = 1
    while d:
        c = txt[i]
        if c == "{": d += 1
        elif c == "}": d -= 1
        i += 1
    a, b = m.start(), i
    old = txt[a:b]


def split_params(s):
    out, d, cur = [], 0, ""
    for ch in s:
        if ch in "(<[": d += 1
        elif ch in ")>]": d -= 1
        if ch == "," and d == 0:
            out.append(cur); cur = ""
        else:
            cur += ch
    if cur.strip(): out.append(cur)
    return [x.split(":")[0].strip() for x in out]


def upto_semi(s, i):
    d = 0
    j = i
    while j < len(s):
        c = s[j]
        if c in "({[": d += 1
        elif c in ")}]": d -= 1
        elif c == ";" and d == 0: return j
        j += 1
    return len(s)


defaults = {}
for m in re.finditer(r"fn\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*[^=;{]+?)?\s*=\s*", old):
    name = m.group(1)
    j = upto_semi(old, m.end())
    expr = old[m.end():j].strip()
    names = split_params(m.group(2))
    args = ", ".join(f"a{i}" for i in range(len(names)))
    defaults[name] = "{ let (" + ", ".join(names) + ("," if len(names) == 1 else "") + ") = (" + args + ("," if len(names) == 1 else "") + "); " + expr + " }" if names else expr
for m in re.finditer(r"hook_or!\(\s*\w+\s*,\s*(\w+)\s*,\s*\(([^)]*)\)[^,]*,\s*([^;]+?)\)\s*;", old):
    name, ps, port = m.group(1), m.group(2), m.group(3).strip()
    names = split_params(ps)
    defaults[name] = f"{port}({', '.join(f'a{i}' for i in range(len(names)))})"
for m in re.finditer(r"pub static mut (\w+)\s*:\s*(?:unsafe\s+)?fn\s*\([^=]*?=\s*([A-Za-z_][\w:]*)\s*;", old):
    defaults.setdefault(m.group(1), m.group(2) + "(@ARGS@)")
# lb_hook!(H_X, X, (a: T, ..) [-> R] [= default]);  and  pub use path::{A, B};  /  pub use path::A;  inside the old ext
def _split_args(s):
    return split_params(s)
for m in re.finditer(r"lb_hook!\(\s*\w+\s*,\s*(\w+)\s*,\s*\(([^)]*)\)[^=;]*?(?:=\s*([^;]+?))?\)\s*;", old):
    name, ps, dflt = m.group(1), m.group(2), m.group(3)
    if dflt and name not in ("OSReport",):
        defaults[name] = "(" + dflt.strip() + ")(@ARGS@)"
for m in re.finditer(r"pub use ([\w:]+)::\{([^}]*)\}\s*;", old):
    for nm in [x.strip() for x in m.group(2).split(",") if x.strip()]:
        if nm not in ("OSReport", "OsArg", "__assert"):
            defaults.setdefault(nm, m.group(1) + "::" + nm + "(@ARGS@)")
for m in re.finditer(r"pub use ([\w:]+)::(\w+)\s*;", old):
    if m.group(2) not in ("OSReport", "OsArg", "__assert"):
        defaults.setdefault(m.group(2), m.group(1) + "::" + m.group(2) + "(@ARGS@)")
txt = txt.replace("ext::OSReport(", "crate::tu::lbext::OSReport(").replace("ext::OsArg", "crate::tu::lbext::OsArg").replace("ext::__assert(", "crate::tu::lbext::__assert(")
if defaults:
    (HERE / f"defaults_{stem}.json").write_text(json.dumps(defaults, indent=1), encoding="utf-8")
new = "// BEGIN GENERATED EXT\n// END GENERATED EXT"
txt = txt[:a] + new + txt[b:]
# the old statics were fn pointers called as `(ext::X)(args)`: the generated ext has plain fns
txt, nsites = re.subn(r"\(ext::(\w+)\)\(", r"ext::\1(", txt)
print("rewrote", nsites, "call sites")
p.write_text(txt, encoding="utf-8", newline="\n")
print(f"{len(defaults)} defaults harvested")
print("---- old ext (non-fn lines) ----")
for l in old.splitlines():
    if re.search(r"static|pub use|const |type |struct", l):
        print(l)
