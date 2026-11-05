#!/usr/bin/env python3
"""Write the walker's type table (Rust) from rust/melee_types/src/gen.rs.

  python tools/rust-port/layout/gen_walk_table.py OUT.rs

For every generated record: its native size (`size_of`), GC size, and per field the native offset
(`offset_of!`, so the table cannot drift from the compiled struct), the GC offset (from the generated
decode, else GC_FIELDS), and a type descriptor. Pointer fields whose decode allocates an array carry the
count expression copied verbatim from the generated decode (`d.ptr_array::<T>(at + off, n)`), or the
terminator word (`ptr_array_until_word`). The output is build output: never committed.
"""
import re, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gentab

ROOT = Path(__file__).resolve().parents[3]
GEN = ROOT / "rust/melee_types/src/gen.rs"
PRIM = {"u8": (1, 1, 0, 0), "i8": (1, 1, 1, 0), "u16": (2, 2, 0, 0), "i16": (2, 2, 1, 0), "u32": (4, 4, 0, 0),
        "i32": (4, 4, 1, 0), "u64": (8, 8, 0, 0), "i64": (8, 8, 1, 0), "f32": (4, 4, 0, 1), "f64": (8, 8, 0, 1),
        "bool": (1, 1, 0, 0)}


def main(out):
    text = GEN.read_text(encoding="utf-8")
    recs, al = gentab.parse(GEN)
    # per record: field -> the full decode statement (counts / terminators)
    stmt = {}
    cur = None
    for ln in text.split("\n"):
        m = gentab.DECODE_HEAD.match(ln)
        if m and m.group(1) in recs:
            cur = m.group(1)
            continue
        if cur and ln.startswith("impl "):
            cur = None
        if cur:
            m = re.search(r"v\.((?:r#)?\w+) = ", ln)
            if m:
                f = m.group(1)
                f = f[2:] if f.startswith("r#") else f
                stmt.setdefault(cur, {}).setdefault(f, ln.strip())
    tys, tyidx = [], {}
    names = sorted(recs)
    ridx = {n: i for i, n in enumerate(names)}

    def ty(t, depth=0):
        t = gentab.resolve(t.strip(), al)
        if t in tyidx:
            return tyidx[t]
        if depth > 30:
            d = "Ty::Void"
        elif t in PRIM:
            nw, gw, s, f = PRIM[t]
            d = f"Ty::Prim {{ w: {nw}, signed: {str(bool(s)).lower()}, float: {str(bool(f)).lower()} }}"
        elif t in ("usize", "isize"):
            d = "Ty::Usize"
        elif t.startswith("Option<"):
            d = "Ty::Fn"
        elif t.startswith("*mut ") or t.startswith("*const "):
            inner = t.split(" ", 1)[1]
            tyidx[t] = len(tys); tys.append(None)       # recursion guard (self-referential lists)
            i = tyidx[t]
            tys[i] = f"Ty::Ptr({ty(inner, depth + 1)})"
            return i
        elif gentab.split_array(t):
            e, n = gentab.split_array(t)
            if not re.fullmatch(r"\d+|0x[0-9a-fA-F]+", n):
                d = "Ty::Void"
            else:
                tyidx[t] = len(tys); tys.append(None)
                i = tyidx[t]
                tys[i] = f"Ty::Arr({ty(e, depth + 1)}, {int(n, 0)})"
                return i
        elif t in recs and recs[t].get("gc"):
            d = f"Ty::Rec({ridx[t]})"
        else:
            d = "Ty::Void"
        tyidx[t] = len(tys)
        tys.append(d)
        return tyidx[t]

    rows = []
    for n in names:
        r = recs[n]
        gc = r.get("gc") or {}
        fl = []
        gcf = {g["name"]: g for g in gc.get("fields", []) if not g["bits"]}
        for fname, ft in r["fields"]:
            if fname in r["off"]:
                goff = r["off"][fname]
            elif fname in gcf:
                goff = gcf[fname]["off"]
            elif r["kind"] == "union":
                goff = 0
            else:
                m = re.fullmatch(r"(?:pad|x|unk|anon|bf)_?([0-9A-Fa-f]+)", fname)
                if not m:
                    continue
                goff = int(m.group(1), 16)
            if r["kind"] == "union":
                goff = 0
            fid = ("r#" + fname) if fname in RUST_KW else fname
            noff = "0" if r["kind"] == "union" else f"core::mem::offset_of!({n}, {fid})"
            st = stmt.get(n, {}).get(fname, "")
            count, until = "None", "None"
            m = re.match(r"\{ let n = (.*); v\.\S+ = d\.ptr_array::<.*>\(at \+ 0x[0-9A-F]+, n\)", st)
            if m:
                count = f"Some(|d: &mut GcDecoder<'_>, at: u32| -> usize {{ let _ = at; {m.group(1)} }})"
            m = re.search(r"d\.ptr_array_until_word::<.*>\(at \+ 0x[0-9A-F]+, (0x[0-9A-F]+|\d+)\)", st)
            if m:
                until = f"Some({m.group(1)})"
            # the element type the decode allocates, when it differs from the field's declared type (arrays.tsv 3rd column)
            m = re.search(r"d\.(ptr_array|ptr_array_until_word)::<(.+?)>\(at \+", st)
            fty = ty(ft)
            if m and m.group(2) != "T":
                fty = ty("*mut " + m.group(2))
            decoded = "true" if fname in stmt.get(n, {}) else "false"
            fl.append(f'Fd {{ name: "{fname}", noff: {noff}, goff: 0x{goff:X}, ty: {fty}, count: {count}, until: {until}, decoded: {decoded} }}')
        rows.append(f'Rd {{ name: "{n}", gsize: 0x{gc.get("size", 0):X}, nsize: core::mem::size_of::<{n}>(), union: {str(r["kind"] == "union").lower()}, '
                    f'fields: &[{", ".join(fl)}] }}')
    for n in names:
        if recs[n].get("gc"):
            ty(n)   # every record has a type slot (the --identical listing)
    body = ["// @generated by tools/rust-port/layout/gen_walk_table.py from melee_types gen.rs. Build output; not committed.",
            "#[allow(unused_imports)] use melee_types::*;", "#[allow(unused_imports)] use melee_core::gcdecode::{GcDecode, GcDecoder};",
            "#[allow(unused_imports)] use core::ffi::c_void;",
            f"pub static TYS: [Ty; {len(tys)}] = [" + ", ".join(tys) + "];",
            f"pub static RECS: [Rd; {len(rows)}] = [", *[r + "," for r in rows], "];",
            "pub static ALIASES: &[(&str, &str)] = &[" + ", ".join(
                f'("{a}", "{gentab.resolve(a, al)}")' for a in sorted(al) if gentab.resolve(a, al) in recs) + "];"]
    Path(out).write_text("\n".join(body) + "\n", encoding="utf-8")
    print(f"walk table: {len(rows)} records, {len(tys)} types -> {out}")


RUST_KW = set("as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub ref "
              "return self Self static struct super trait true type unsafe use where while async await dyn abstract become "
              "box do final macro override priv typeof unsized virtual yield try gen".split())

if __name__ == "__main__":
    main(sys.argv[1])
