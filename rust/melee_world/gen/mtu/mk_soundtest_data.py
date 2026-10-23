#!/usr/bin/env python3
"""Rust spelling of the data objects src/melee/if/soundtest.c defines (the sound test / debug menu
tables: strings, string-pointer arrays, menu rows with callback and value pointers, the small
structs), written from the C initializers by libclang so the port's tables cannot drift from the C.

  python rust/melee_world/gen/mtu/mk_soundtest_data.py       (from the worktree root)

Rewrites the block between `// BEGIN GENERATED DATA` and `// END GENERATED DATA` in
rust/melee_world/src/tu/soundtest.rs and writes gen/mtu/tu/soundtest_data.json (name, kind, size)
which mk_soundtest.py uses to build the table-hash part of the oracle. The tables are checked
against the C build's by the oracle (op "tables").
"""
import json, re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gen_tu as G  # noqa: E402

ci, K, T = G.ci, G.K, G.T
SRC = "src/melee/if/soundtest.c"
PORT = G.ROOT / "rust/melee_world/src/tu/soundtest.rs"

# local record types -> Rust spelling (hand-written in the port file)
LOCAL_REC = {"SoundTestMenuData", "un_803FA258_t"}


def toks(c):
    return [t.spelling for t in c.get_tokens()]


def unescape(lit):
    """C string literal token -> bytes."""
    s = lit[1:-1]
    out = bytearray()
    i = 0
    while i < len(s):
        ch = s[i]
        if ch == "\\":
            i += 1
            e = s[i]
            if e == "n":
                out.append(10)
            elif e == "t":
                out.append(9)
            elif e == "r":
                out.append(13)
            elif e == "0":
                out.append(0)
            elif e in "\\\"'":
                out.append(ord(e))
            else:
                raise SystemExit(f"unhandled escape \\{e} in {lit}")
        else:
            out.append(ord(ch))
        i += 1
    return bytes(out)


class Conv:
    def __init__(self, tu):
        self.tu = tu
        self.vars = {}  # name -> (cursor, canonical type)
        for c in tu.tu.cursor.get_children():
            if c.kind == K.VAR_DECL and tu.in_main(c) and c.storage_class != ci.StorageClass.EXTERN:
                self.vars[c.spelling] = c

    # --- values
    def zero(self, t):
        c = G.canon(t)
        k = c.kind
        if k == T.CONSTANTARRAY:
            return "[" + ", ".join([self.zero(c.element_type)] * c.element_count) + "]" if c.element_count <= 4 else \
                f"[{self.zero(c.element_type)}; {c.element_count}]"
        if k == T.RECORD:
            return "unsafe { core::mem::zeroed() }"
        if k == T.POINTER:
            return "None" if G.is_fnptr(t) else "null_mut()"
        if k in (T.FLOAT, T.DOUBLE):
            return "0.0"
        return "0"

    def addr(self, ts, want):
        """tokens of `&name(.field|[i])*` or `name` -> Rust pointer expression of type `want` (a pointer type)."""
        s = "".join(ts)
        m = re.fullmatch(r"&?(\w+)((?:\.\w+|\[\d+\])*)", s)
        if not m:
            raise SystemExit(f"unhandled address expression {s!r}")
        name, rest = m.group(1), m.group(2)
        base = name if name in self.vars else f"ext::{name}"
        return f"(&raw mut {base}{rest}) as *mut _ as {want}"

    def value(self, c, t):
        """Rust constant expression for the initializer cursor c of type t."""
        ct = G.canon(t)
        k = ct.kind
        ts = toks(c)
        if k == T.CONSTANTARRAY and G.canon(ct.element_type).kind == T.CHAR_S:
            if len(ts) == 1 and ts[0].startswith('"'):
                b = unescape(ts[0])
                if len(b) + 1 > ct.element_count + (0):
                    if len(b) > ct.element_count:
                        raise SystemExit("string too long")
                return f"cstr::<{ct.element_count}>({bytestr(b)})"
            raise SystemExit(f"char array initializer {ts}")
        if k == T.CONSTANTARRAY:
            kids = [x for x in c.get_children()]
            elems = [self.value(x, ct.element_type) for x in kids]
            n = ct.element_count
            if len(elems) > n:
                raise SystemExit(f"too many initializers for array {ts[:6]}")
            elems += [self.zero(ct.element_type)] * (n - len(elems))
            return "[" + ", ".join(elems) + "]"
        if k == T.RECORD:
            decl = ct.get_declaration()
            fields = [f for f in ct.get_fields()]
            kids = [x for x in c.get_children()]
            vals = []
            for i, f in enumerate(fields):
                if i < len(kids):
                    vals.append(f"{f.spelling}: {self.value(kids[i], f.type)}")
                else:
                    vals.append(f"{f.spelling}: {self.zero(f.type)}")
            return f"{rec_name(t)} {{ " + ", ".join(vals) + " }"
        s = "".join(ts)
        if k == T.POINTER:
            if G.is_fnptr(t):
                return "None" if (not ts or s in ("NULL", "0")) else f"Some({s})"
            if not ts or s in ("NULL", "0", "((void*)0)"):
                return "null_mut()"
            return self.addr(ts, G.rust_ty(t))
        if k in (T.FLOAT, T.DOUBLE):
            s = re.sub(r"([0-9.]+)[fF]$", r"\1", s)
            if "." not in s and "e" not in s:
                s += ".0"
            return s
        # integers / enums
        s = s.replace("U", "").replace("L", "")
        return s

    # --- declarations
    def decl(self, name):
        c = self.vars[name]
        t = c.type
        ty = G.rust_ty(t)
        ct = G.canon(t)
        if ct.kind == T.CONSTANTARRAY and G.canon(ct.element_type).kind == T.CHAR_S:
            ty = f"[i8; {ct.element_count}]"
        if ct.kind == T.RECORD:
            ty = rec_name(t)
        init = [x for x in c.get_children() if x.kind not in (K.TYPE_REF, K.NAMESPACE_REF)]
        # the initializer is the last expression child
        exprs = [x for x in init if x.kind.is_expression()]
        if exprs:
            val = self.value(exprs[-1], t)
        else:
            val = self.zero(t)
        return ty, val


def bytestr(b):
    out = 'b"'
    for x in b:
        if x == 10:
            out += "\\n"
        elif x == 34:
            out += '\\"'
        elif x == 92:
            out += "\\\\"
        elif 32 <= x < 127:
            out += chr(x)
        else:
            out += f"\\x{x:02x}"
    return out + '"'


def rec_name(t):
    n = G.canon(t).get_declaration().spelling
    return n


def main():
    tu = G.Tu(SRC, (), G.ROOT / "src/melee/if")
    cv = Conv(tu)
    lines = []
    meta = []
    order = []
    for c in tu.tu.cursor.get_children():
        if c.kind == K.VAR_DECL and tu.in_main(c) and c.storage_class != ci.StorageClass.EXTERN:
            if c.spelling not in order:
                order.append(c.spelling)
    for n in order:
        c = cv.vars[n]
        # a tentative definition's cursor has no initializer; take the one that has (the later one)
        defs = [d for d in tu.tu.cursor.get_children() if d.kind == K.VAR_DECL and d.spelling == n and tu.in_main(d)
                and d.storage_class != ci.StorageClass.EXTERN]
        d = next((x for x in defs if x.is_definition()), defs[-1])
        cv.vars[n] = d
        ty, val = cv.decl(n)
        ct = G.canon(d.type)
        pub = "pub "
        lines.append(f"{pub}static mut {n}: {ty} = unsafe {{ {val} }};")
        kind = ("chars" if ct.kind == T.CONSTANTARRAY and G.canon(ct.element_type).kind == T.CHAR_S else
                "ptrs" if ct.kind == T.CONSTANTARRAY and G.canon(ct.element_type).kind == T.POINTER else
                "rows" if ct.kind == T.CONSTANTARRAY and G.canon(ct.element_type).kind == T.RECORD else
                "ints" if ct.kind == T.CONSTANTARRAY else
                "rec" if ct.kind == T.RECORD else "scalar")
        meta.append(dict(name=n, kind=kind, ctype=d.type.spelling, static=d.storage_class == ci.StorageClass.STATIC,
                         n=ct.element_count if ct.kind == T.CONSTANTARRAY else 1,
                         elem=(G.canon(ct.element_type).get_declaration().spelling or G.canon(ct.element_type).kind.name)
                         if ct.kind == T.CONSTANTARRAY else (ct.get_declaration().spelling or ct.kind.name)))
    block = "// BEGIN GENERATED DATA (rust/melee_world/gen/mtu/mk_soundtest_data.py from src/melee/if/soundtest.c)\n" + \
        "\n".join(lines) + "\n// END GENERATED DATA"
    if PORT.exists():
        txt = PORT.read_text(encoding="utf-8")
        if "// BEGIN GENERATED DATA" in txt:
            a = txt.index("// BEGIN GENERATED DATA")
            b = txt.index("// END GENERATED DATA") + len("// END GENERATED DATA")
            txt = txt[:a] + block + txt[b:]
        else:
            txt = txt.replace("@@DATA@@", block)
        PORT.write_text(txt, encoding="utf-8", newline="\n")
    (HERE / "tu" / "soundtest_data.json").write_text(json.dumps(meta, indent=1), encoding="utf-8", newline="\n")
    print(f"mk_soundtest_data: {len(meta)} objects")


if __name__ == "__main__":
    main()
