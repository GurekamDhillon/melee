#!/usr/bin/env python3
"""Turn the plain `pub mod ext { pub fn NAME(args) -> R { body } ... }` seam of a src/tu/<stem>.rs port into the
settable-hook form the oracle stubs install into (same macro as src/tu/fighter.rs):

  python rust/melee_fighter/gen/mtu/hookify.py src/tu/ftaction.rs

Every one-line `pub fn` inside the ext module becomes `fn NAME(args) -> R = { body };` of an `ext_fns!` block, which
declares `ext::hook::NAME` (an `Option<fn(..)>`; None runs the body) next to the wrapper `ext::NAME`. Lines that are
not one-line `pub fn`s are kept as they are (and listed).
"""
import re, sys
from pathlib import Path

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


def main():
    p = Path(sys.argv[1])
    s = p.read_text(encoding="utf-8")
    crlf = "\r\n" in s
    s = s.replace("\r\n", "\n")
    a = s.index("pub mod ext {")
    end = s.index("\n}\n", a) + 3
    block = s[a:end]
    lines = block.split("\n")
    out, ents, kept = [], [], []
    for l in lines:
        m = re.match(r"^    pub fn (\w+)\((.*?)\)( -> [^{]+?)? \{ (.*) \}$", l)
        if m:
            name, args, ret, body = m.groups()
            ents.append(f"        fn {name}({args}){ret or ''} = {{ {body} }};")
        elif l.startswith("    pub fn "):
            kept.append(l)
            out.append(l)
        else:
            out.append(l)
    # place the macro + block before the first kept pub fn or at the end of the module
    idx = len(out) - 2  # before the closing "}" and trailing ""
    while out[idx].strip() == "" or out[idx] == "}":
        idx -= 1
    idx += 1
    ins = MACRO.split("\n")[:-1] + ["    ext_fns! {"] + ents + ["    }"]
    out[idx:idx] = ins
    s = s[:a] + "\n".join(out) + s[end:]
    p.write_text(s.replace("\n", "\r\n") if crlf else s, encoding="utf-8", newline="")
    print(f"hookify: {p.name}: {len(ents)} hooked, {len(kept)} kept")
    for k in kept:
        print("  KEPT", k.strip()[:100])


if __name__ == "__main__":
    main()
