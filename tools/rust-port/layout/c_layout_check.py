#!/usr/bin/env python3
"""Layout check, part A: every melee_types record against the C build's own layout.

  python tools/rust-port/layout/c_layout_check.py [--json OUT] [--verbose]     (from the worktree root, Windows)

C side (the oracle): the CURRENT headers and TU sources, parsed by libclang with the shipping game
pipeline's flags (`_build/masstest/pipe_win.sh`: ppc32-none-eabi -DTARGET_PC -fno-short-enums
-fsigned-char -mlong-double-64 ...). Each record is flattened to leaves: (GC bit offset, element count)
-> scalar kind (i8/u8/i16/u16/i32/u32/i64/u64/f32/f64/ptr) and every bitfield to (bit offset, width,
signed). Record sizes/aligns are cross-checked against the shipping clang binary's
`-fdump-record-layouts-complete` (tools/rust-port/layout/c_dump_sizes.sh does it standalone).
Rust side: rust/melee_types/src/gen.rs, i.e. what the decoder, the GC image code and the hash use:
native field types (width/signedness/float), their GC offsets (the generated decode), GC_FIELDS bitfields.
Records are matched by C name; nested and anonymous records are reached through their parents, so an
upstream rename of an anonymous type does not hide a layout change. Field NAMES are not compared
(upstream renames them often); offsets, widths, signedness and bit positions are.
"""
import json, os, re, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gentab

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
import clang.cindex as ci  # noqa: E402

LLVM = Path(os.environ.get("LLVM_DIR", r"C:\Program Files\LLVM"))
ci.Config.set_library_file(str(LLVM / "bin" / "libclang.dll"))
K, T = ci.CursorKind, ci.TypeKind
GALE = os.environ.get("GALE_INCLUDE") or str((ROOT / "../../melee/build/GALE01/include").resolve())
# = _build/masstest/pipe_win.sh (the C build's per-TU compile), minus codegen-only flags
SHIP = ["--target=ppc32-none-eabi", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT", "-DTARGET_PC",
        "-fno-short-enums", "-fsigned-char", "-mlong-double-64", "-fno-strict-aliasing", "-fwrapv", "-fcommon",
        "-fgnu89-inline", "-w", "-Isrc", "-Ipc", "-Ipc/gameworld", "-isystem", "src/MSL", "-isystem",
        "libs/dolphin/include", "-isystem", "libs/dolphin/src", "-isystem", GALE, "-include", "src/MSL/math_ppc.h"]
GEN = ROOT / "rust/melee_types/src/gen.rs"
PROBE = ROOT / "rust/melee_types/gen/all_types.c"

PRIM = {T.CHAR_S: "i8", T.SCHAR: "i8", T.CHAR_U: "u8", T.UCHAR: "u8", T.SHORT: "i16", T.USHORT: "u16",
        T.INT: "i32", T.UINT: "u32", T.LONG: "i32", T.ULONG: "u32", T.LONGLONG: "i64", T.ULONGLONG: "u64",
        T.FLOAT: "f32", T.DOUBLE: "f64", T.LONGDOUBLE: "f64", T.BOOL: "u8"}


def sources():
    """The TU sources whose private records melee_types carries (gen_types.py / local_records.py lists)."""
    txt = (ROOT / "rust/melee_types/gen/gen_types.py").read_text(encoding="utf-8") + \
        (ROOT / "rust/melee_types/gen/local_records.py").read_text(encoding="utf-8")
    out = set(re.findall(r'"((?:src|pc|libs)/[\w/.-]+\.c)"', txt))
    for stem in re.findall(r'\("(\w+)", \[', txt):
        for d in ("src/melee/lb", "src/sysdolphin/baselib"):
            if (ROOT / d / f"{stem}.c").exists():
                out.add(f"{d}/{stem}.c")
    for stem in ("state", "synth"):
        out.add(f"src/sysdolphin/baselib/{stem}.c")
    out.add("src/melee/gr/ground.c")
    return sorted(s for s in out if (ROOT / s).exists())


def probe_text():
    """all_types.c as committed, minus headers upstream folded away (their records are in the TUs now)."""
    out, dropped = [], []
    for ln in PROBE.read_text(encoding="utf-8").split("\n"):
        m = re.match(r"#include <(.*)>", ln)
        if m and not any((ROOT / d / m.group(1)).exists() for d in
                         ("src", "pc", "pc/gameworld", "libs/dolphin/include", "libs/dolphin/src", "src/MSL")):
            dropped.append(m.group(1))
            continue
        out.append(ln)
    return "\n".join(out), dropped


def parse(path, text=None):
    idx = ci.Index.create()
    cwd = os.getcwd(); os.chdir(ROOT)
    try:
        uf = [(path, text)] if text is not None else None
        tu = idx.parse(path, args=SHIP, unsaved_files=uf, options=ci.TranslationUnit.PARSE_SKIP_FUNCTION_BODIES)
    finally:
        os.chdir(cwd)
    errs = [str(d) for d in tu.diagnostics if d.severity >= ci.Diagnostic.Error]
    return tu, errs


def rel(c):
    f = c.location.file
    if not f:
        return "?"
    try:
        return Path(f.name).resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return f.name


class CSide:
    def __init__(self):
        self.named = {}   # C name -> record cursor

    def add(self, tu, prefer_file=None):
        def walk(c):
            for ch in c.get_children():
                if ch.kind in (K.STRUCT_DECL, K.UNION_DECL) and ch.is_definition() and ch.spelling and \
                        "(unnamed" not in ch.type.spelling and "(anonymous" not in ch.type.spelling:
                    if prefer_file is None or rel(ch) == prefer_file:
                        if prefer_file or ch.spelling not in self.named:
                            self.named[ch.spelling] = ch
                if ch.kind == K.TYPEDEF_DECL:
                    u = ch.underlying_typedef_type.get_canonical()
                    if u.kind == T.RECORD:
                        d = u.get_declaration().get_definition()
                        if d is not None and (prefer_file is None or rel(d) == prefer_file):
                            if prefer_file or ch.spelling not in self.named:
                                self.named[ch.spelling] = d
                if ch.kind in (K.STRUCT_DECL, K.UNION_DECL):
                    walk(ch)
        walk(tu.cursor)

    def scalar(self, t):
        t = t.get_canonical()
        if t.kind in PRIM:
            return PRIM[t.kind]
        if t.kind == T.ENUM:
            return PRIM.get(t.get_declaration().enum_type.get_canonical().kind, "?enum")
        if t.kind in (T.POINTER, T.BLOCKPOINTER):
            return "ptr"
        return None

    def leaves(self, rec, base, out, bits, path, seen):
        for f in rec.get_children():
            if f.kind != K.FIELD_DECL:
                continue
            o = base + f.get_field_offsetof()
            if f.is_bitfield():
                k = self.scalar(f.type)
                bits[o] = (f.get_bitfield_width(), k, f"{path}.{f.spelling}")
                continue
            self.typ(f.type, o, out, bits, f"{path}.{f.spelling or '(anon)'}", seen)

    def typ(self, t, o, out, bits, path, seen, count=1):
        t = t.get_canonical()
        k = self.scalar(t)
        if k:
            out[(o, count)] = (k, path)
            return
        if t.kind == T.CONSTANTARRAY:
            n = t.element_count
            et = t.element_type.get_canonical()
            # arrays of scalars: one leaf with the total count; arrays of records: element 0 and the count
            if self.scalar(et) or et.kind == T.CONSTANTARRAY:
                inner = et
                while inner.kind == T.CONSTANTARRAY:
                    n *= inner.element_count
                    inner = inner.element_type.get_canonical()
                if self.scalar(inner):
                    out[(o, count * n)] = (self.scalar(inner), path)
                    return
                et = inner
            out[(o, -n)] = ("arr", path)
            self.typ(et, o, out, bits, path + "[0]", seen)
            return
        if t.kind == T.RECORD:
            d = t.get_declaration().get_definition()
            if d is None:
                out[(o, count)] = ("opaque", path)
                return
            if d.hash in seen:
                return
            self.leaves(d, o, out, bits, path, seen | {d.hash})
            return
        out[(o, count)] = (f"?{t.kind.name}", path)


class RustSide:
    def __init__(self, recs, aliases):
        self.recs, self.al = recs, aliases
        self.issues = []

    def scalar(self, t):
        t = gentab.resolve(t, self.al)
        if t in gentab.PRIMS:
            sz, s = gentab.PRIMS[t]
            return {"u": "u", "i": "i", "f": "f", "b": "u"}[s] + str(sz * 8) if s != "b" else "u8"
        if t.startswith("*mut") or t.startswith("*const") or t.startswith("Option<"):
            return "ptr"
        return None

    def typ(self, t, o, out, bits, path, seen):
        t = gentab.resolve(t, self.al)
        k = self.scalar(t)
        if k:
            out[(o, 1)] = (k, path)
            return
        a = gentab.split_array(t)
        if a:
            et, n = a
            n = int(n, 0) if re.fullmatch(r"\d+|0x[0-9a-fA-F]+", n) else None
            if n is None:
                out[(o, 0)] = ("?arraylen", path)
                return
            inner = gentab.resolve(et, self.al)
            while gentab.split_array(inner):
                ie, inn = gentab.split_array(inner)
                n *= int(inn, 0)
                inner = gentab.resolve(ie, self.al)
            if self.scalar(inner):
                out[(o, n)] = (self.scalar(inner), path)
                return
            out[(o, -n)] = ("arr", path)
            self.typ(inner, o, out, bits, path + "[0]", seen)
            return
        r = self.recs.get(t)
        if r is None:
            out[(o, 1)] = (f"?{t}", path)
            return
        if t in seen:
            return
        self.leaves(r, o, out, bits, path, seen | {t})

    def leaves(self, r, base, out, bits, path, seen):
        gcf = (r.get("gc") or {}).get("fields", [])
        for b in gcf:
            if b["bits"]:
                bit, w = b["bits"]
                unit = b["off"]
                bits[base * 8 + unit * 8 + bit] = (w, b["ctype"], f"{path}.{b['name']}")
        for name, t in r["fields"]:
            if name.startswith("bf_") and t.startswith("[u8;"):
                continue   # bitfield storage, compared through GC_FIELDS above
            if r["kind"] == "union":
                o = 0
            elif name in r["off"]:
                o = r["off"][name]
            elif any(g["name"] == name and not g["bits"] for g in gcf):
                # not decoded (function pointers, pads): its GC_FIELDS entry
                o = next(g["off"] for g in gcf if g["name"] == name and not g["bits"])
            elif re.fullmatch(r"(?:pad|x|unk|anon)_?([0-9A-Fa-f]+)", name):
                # native names the generator gives unnamed or padding slots carry their GC offset
                o = int(re.fullmatch(r"(?:pad|x|unk|anon)_?([0-9A-Fa-f]+)", name).group(1), 16)
            else:
                self.issues.append(f"{r['rust']}.{name}: no GC offset in the generated decode")
                continue
            self.typ(t, base + o, out, bits, f"{path}.{name}", seen)


def bits_signed_rust(ctype_kind):
    return ctype_kind and ctype_kind.startswith("i")


def main():
    verbose = "--verbose" in sys.argv
    recs, aliases = gentab.parse(GEN)
    c = CSide()
    text, dropped = probe_text()
    tu, errs = parse("rust/melee_types/gen/all_types.c", text)
    if errs:
        print("probe parse errors (first 10):\n  " + "\n  ".join(errs[:10]))
    c.add(tu)
    tu_errs = {}
    for s in sources():
        t2, e2 = parse(s)
        if e2:
            tu_errs[s] = e2[:3]
        c.add(t2, prefer_file=s)
    rs = RustSide(recs, aliases)
    report = {"dropped_headers": dropped, "tu_errors": tu_errs, "records": {}}
    n_ok = n_bad = n_missing = 0
    for rn, r in sorted(recs.items()):
        cname = r.get("c_name")
        if not cname or "(unnamed" in cname or "(anonymous" in cname:
            continue
        cname = re.sub(r"^(struct|union) ", "", cname)
        cr = c.named.get(cname)
        if cr is None:
            n_missing += 1
            report["records"][rn] = {"status": "no C record of that name (renamed or removed upstream)"}
            continue
        problems = []
        csz, cal = cr.type.get_size(), cr.type.get_align()
        if r.get("gc") and (r["gc"].get("size") != csz or r["gc"].get("align") != cal):
            problems.append(f"size/align GC table {r['gc'].get('size'):#x}/{r['gc'].get('align')} vs C {csz:#x}/{cal}")
        cl, cb, rl, rb = {}, {}, {}, {}
        c.leaves(cr, 0, cl, cb, cname, {cr.hash})
        rs.leaves(r, 0, rl, rb, rn, {rn})
        rl = {(o * 8, n): v for (o, n), v in rl.items()}
        for key in sorted(set(cl) | set(rl)):
            a, b = cl.get(key), rl.get(key)
            if a and b:
                ka, kb = a[0], b[0]
                if ka != kb and not (ka == "arr" and kb == "arr"):
                    problems.append(f"+{key[0] // 8:#x} x{key[1]}: C {ka} {a[1]} vs Rust {kb} {b[1]}")
            elif a and not b:
                problems.append(f"+{key[0] // 8:#x} x{key[1]}: C {a[0]} {a[1]} has no Rust leaf")
            else:
                problems.append(f"+{key[0] // 8:#x} x{key[1]}: Rust {b[0]} {b[1]} has no C leaf")
        for bo in sorted(set(cb) | set(rb)):
            a, b = cb.get(bo), rb.get(bo)
            if not a or not b or a[0] != b[0]:
                problems.append(f"bitfield @bit {bo}: C {a} vs Rust {b}")
        if problems:
            n_bad += 1
            report["records"][rn] = {"status": "MISMATCH", "problems": problems}
        else:
            n_ok += 1
            report["records"][rn] = {"status": "ok"}
    report["summary"] = {"ok": n_ok, "mismatch": n_bad, "no_c_record": n_missing, "rust_issues": rs.issues[:50]}
    if "--json" in sys.argv:
        Path(sys.argv[sys.argv.index("--json") + 1]).write_text(json.dumps(report, indent=1))
    for rn, v in report["records"].items():
        if v["status"] == "MISMATCH":
            print(f"MISMATCH {rn}: " + ("; ".join(v["problems"]) if verbose else f"{len(v['problems'])} problem(s): " + v["problems"][0]))
    print(f"layout: {n_ok} records match, {n_bad} mismatch, {n_missing} with no C record of that name; "
          f"{len(dropped)} folded headers dropped; {len(tu_errs)} TU parse errors")
    return 1 if n_bad else 0


if __name__ == "__main__":
    sys.exit(main())
