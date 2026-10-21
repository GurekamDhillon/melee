#!/usr/bin/env python3
"""Generate native Rust structs + GameCube layout tables for group 3 (fighter/pl) records.

  python rust/melee_fighter/gen/gen_types.py            (from the worktree root)

Parses gen/types_tu.c with libclang for the GameCube target (powerpc-unknown-eabi, big-endian,
ILP32) and writes rust/melee_fighter/src/types/generated.rs:

  * every struct/union *defined* under src/melee/ft or src/melee/pl, plus everything they embed by
    value (Vec3, Mtx, HSD_*...; those carry TODO(move-to-core)), as `#[repr(C)]` native structs:
    little-endian, 8-byte pointers, no guest memory;
  * a `melee_core::gc::GcLayout` impl per record with each field's GameCube offset/size (bitfields
    with PPC MSB-first bit numbers), for the DAT loader, netplay and the future m-ex offset map;
  * bitfield runs as `[u8; N]` storage with `name()` / `set_name()` accessors that use the same
    MSB-first numbering, so a GC byte image copies straight in;
  * records only reached through pointers as opaque `struct X { _opaque: [u8; 0] }`;
  * enums defined under ft/pl as `pub type Enum = i32/u32` + one `pub const` per enumerator.

Native layout is whatever repr(C) gives; it is never compared with the GC layout except through
the GcLayout table. Never hand-edit the output; re-run this script.
"""
import keyword, os, re, sys
from pathlib import Path
import clang.cindex as ci

ROOT = Path(__file__).resolve().parents[3]
LLVM = os.environ.get("LIBCLANG_PATH", r"C:\Program Files\LLVM\bin")
ci.Config.set_library_path(LLVM)
TK = ci.TypeKind
CK = ci.CursorKind

GALE = os.environ.get("GALE_INCLUDE", str(ROOT / "../../melee/build/GALE01/include"))
ARGS = ["--target=powerpc-unknown-eabi", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT",
        "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-Wno-everything",
        "-I", str(ROOT / "src"), "-I", str(ROOT / "src/melee"), "-I", str(ROOT / "pc"),
        "-I", str(ROOT / "pc/gameworld"), "-isystem", str(ROOT / "src/MSL"),
        "-isystem", str(ROOT / "libs/dolphin/include"), "-isystem", str(ROOT / "libs/dolphin/src"),
        "-isystem", GALE]
OWN_DIRS = [str((ROOT / "src/melee/ft").resolve()).lower(), str((ROOT / "src/melee/pl").resolve()).lower()]
RUST_KW = set("as break const continue crate else enum extern false fn for if impl in let loop match mod "
              "move mut pub ref return self Self static struct super trait true type unsafe use where while "
              "async await dyn abstract become box do final macro override priv typeof unsized virtual yield try "
              "union".split())

SCALAR = {TK.BOOL: "bool", TK.CHAR_S: "i8", TK.SCHAR: "i8", TK.CHAR_U: "u8", TK.UCHAR: "u8",
          TK.SHORT: "i16", TK.USHORT: "u16", TK.INT: "i32", TK.UINT: "u32", TK.LONG: "i32",
          TK.ULONG: "u32", TK.LONGLONG: "i64", TK.ULONGLONG: "u64", TK.FLOAT: "f32",
          TK.DOUBLE: "f64", TK.LONGDOUBLE: "f64"}
SIGNED = {"i8", "i16", "i32", "i64"}


def owned(cur):
    f = cur.location.file
    if f is None:
        return False
    p = str(Path(f.name).resolve()).lower()
    return any(p.startswith(d) for d in OWN_DIRS)


def fname(n):
    if n in ("_", ""):
        return None
    return "r#" + n if n in RUST_KW and n not in ("self", "Self", "super", "crate") else (n + "_" if n in ("self", "Self", "super", "crate") else n)


class Gen:
    def __init__(self, tu):
        self.tu = tu
        self.names = {}       # usr -> rust name
        self.used = set()
        self.queue = []       # record cursors to generate by value
        self.done = set()
        self.opaque = {}      # usr -> name
        self.out = []
        self.enums = {}       # usr -> (name, int type)

    # ---- naming
    def rec_name(self, decl, hint=None):
        usr = decl.get_usr() or str(decl.hash)
        if usr in self.names:
            return self.names[usr]
        n = decl.spelling
        if not n or "unnamed" in n or "anonymous" in n or "(" in n:
            n = hint or f"Anon_{len(self.names)}"
        n = re.sub(r"\W", "_", n)
        base, i = n, 2
        while n in self.used:
            n = f"{base}_{i}"; i += 1
        self.used.add(n)
        self.names[usr] = n
        return n

    # ---- types
    def rty(self, t, hint=None, byval=True):
        """Rust spelling of C type t. byval records are queued for full generation."""
        c = t.get_canonical()
        k = c.kind
        if k in SCALAR:
            return SCALAR[k]
        if k == TK.VOID:
            return "c_void"
        if k == TK.ENUM:
            d = c.get_declaration()
            it = SCALAR.get(d.enum_type.get_canonical().kind, "i32")
            if owned(d) and d.spelling and "unnamed" not in d.spelling:
                self.enums[d.get_usr()] = (re.sub(r"\W", "_", d.spelling), it, d)
                return re.sub(r"\W", "_", d.spelling)
            return it
        if k == TK.POINTER:
            p = c.get_pointee()
            pk = p.get_canonical().kind
            if pk in (TK.FUNCTIONPROTO, TK.FUNCTIONNOPROTO):
                return "Option<" + self.fnty(p.get_canonical()) + ">"
            if pk == TK.RECORD:
                d = p.get_canonical().get_declaration()
                if d.get_definition() is None or not self.wants_full(d):
                    return "*mut " + self.opaque_name(d, hint)
                return "*mut " + self.rty(p, hint, True)
            return "*mut " + self.rty(p, hint, True)
        if k == TK.CONSTANTARRAY:
            return f"[{self.rty(c.element_type, hint)}; {c.element_count}]"
        if k == TK.INCOMPLETEARRAY:
            return f"[{self.rty(c.element_type, hint)}; 0]"
        if k == TK.RECORD:
            d = c.get_declaration()
            dd = d.get_definition() or d
            name = self.rec_name(dd, hint or self.typedef_hint(t))
            if dd.get_usr() not in self.done:
                self.done.add(dd.get_usr())
                self.queue.append(dd)
            return name
        if k in (TK.FUNCTIONPROTO, TK.FUNCTIONNOPROTO):
            return self.fnty(c)
        raise SystemExit(f"gen_types: unmapped type {t.spelling} ({k})")

    def cty(self, t):
        """C type spelling for the GcField table, with clang's '(unnamed at path)' made stable."""
        def sub(m):
            return "<anon>"
        return re.sub(r"\((unnamed|anonymous)[^)]*\)", sub, t.spelling).replace("\\", "/")

    def typedef_hint(self, t):
        s = t.spelling.replace("struct ", "").replace("union ", "").replace("const ", "").strip()
        return re.sub(r"\W", "_", s) if re.fullmatch(r"[A-Za-z_]\w*", s) else None

    def wants_full(self, d):
        return True  # complete records reached by pointer are generated too (see opaque_name)

    def opaque_name(self, d, hint):
        dd = d.get_definition()
        if dd is not None and owned(dd):
            return self.rty(dd.type, hint, True)
        usr = (dd or d).get_usr()
        if usr in self.names and usr in self.done:
            return self.names[usr]
        n = self.rec_name(dd or d, self.typedef_hint(d.type))
        self.opaque[usr] = n
        return n

    def fnty(self, f):
        if f.kind == TK.FUNCTIONNOPROTO:
            return 'unsafe extern "C" fn()'
        args = [self.rty(a, None) for a in f.argument_types()]
        if f.is_function_variadic():
            args.append("...")
        r = f.get_result()
        ret = "" if r.get_canonical().kind == TK.VOID else " -> " + self.rty(r, None)
        return f'unsafe extern "C" fn({", ".join(args)}){ret}'

    # ---- records
    def members(self, rec):
        """[(kind, cursor)] in declaration order; anonymous member records become 'anon'."""
        out = []
        kids = list(rec.get_children())
        def base(t):
            t = t.get_canonical()
            while t.kind in (TK.CONSTANTARRAY, TK.INCOMPLETEARRAY, TK.POINTER):
                t = (t.element_type if t.kind != TK.POINTER else t.get_pointee()).get_canonical()
            return t.get_declaration().hash
        named_types = {base(c.type) for c in kids if c.kind == CK.FIELD_DECL}
        for c in kids:
            if c.kind == CK.FIELD_DECL:
                out.append(("field", c))
            elif c.kind in (CK.STRUCT_DECL, CK.UNION_DECL) and c.is_definition() and c.is_anonymous() \
                    and c.hash not in named_types:
                out.append(("anon", c))
        return out

    def leaf_names(self, rec):
        for k, c in self.members(rec):
            if k == "field":
                yield c.spelling
            else:
                yield from self.leaf_names(c)

    def first_offset_bits(self, rec):
        for k, c in self.members(rec):
            if k == "field":
                return c.get_field_offsetof(), c
            o, f = self.first_offset_bits(c)
            if f is not None:
                return o, f
        return None, None

    def emit_record(self, rec):
        name = self.names[rec.get_usr()]
        is_union = rec.kind == CK.UNION_DECL
        t = rec.type
        gsize, galign = t.get_size(), t.get_align()
        loc = rec.location
        src = Path(loc.file.name).resolve().relative_to(ROOT).as_posix() if loc.file else "?"
        lines, gcf, acc = [], [], []
        mem = self.members(rec)
        i, anon_i = 0, 0
        while i < len(mem):
            kind, c = mem[i]
            if kind == "anon":
                anon_i += 1
                sub = self.rec_name(c, f"{name}_anon{anon_i}")
                if c.get_usr() not in self.done:
                    self.done.add(c.get_usr()); self.queue.append(c)
                off = 0
                if not is_union:
                    for fs in self.leaf_names(c):
                        if fs:
                            a, b = t.get_offset(fs), c.type.get_offset(fs)
                            if a >= 0 and b >= 0:
                                off = (a - b) // 8
                                break
                    else:
                        raise SystemExit(f"gen_types: cannot place anonymous member of {name}")
                fn = f"anon{anon_i}"
                lines.append(f"    pub {fn}: {sub},")
                gcf.append(f'GcField::new("{fn}", {off:#x}, {c.type.get_size()}, "{self.cty(c.type)}")')
                i += 1
                continue
            if c.is_bitfield():
                run = []
                while i < len(mem) and mem[i][0] == "field" and mem[i][1].is_bitfield() and not is_union:
                    run.append(mem[i][1]); i += 1
                if is_union:
                    run = [c]; i += 1
                runs, cur = [], []
                for b in run:
                    if b.get_bitfield_width() == 0:
                        if cur: runs.append(cur); cur = []
                        continue
                    if cur and b.get_field_offsetof() < cur[-1].get_field_offsetof() + cur[-1].get_bitfield_width():
                        runs.append(cur); cur = []
                    cur.append(b)
                if cur: runs.append(cur)
                for r in runs:
                    b0 = r[0].get_field_offsetof() // 8
                    end = max(b.get_field_offsetof() + b.get_bitfield_width() for b in r)
                    nbytes = (end + 7) // 8 - b0
                    st = f"bf_{b0:x}"
                    lines.append(f"    /// Bitfields at GC +{b0:#x}, MSB-first (PPC) bit order.")
                    lines.append(f"    pub {st}: [u8; {nbytes}],")
                    for b in r:
                        w = b.get_bitfield_width()
                        rt = self.rty(b.type)
                        usz = b.type.get_canonical().get_size()
                        ubits = usz * 8
                        boff = b.get_field_offsetof()
                        ustart = (boff // ubits) * ubits
                        if boff + w > ustart + ubits:  # straddles (packed): describe by byte
                            ustart = (boff // 8) * 8; usz = (boff + w - ustart + 7) // 8
                        gcf.append(f'GcField::bitfield("{b.spelling}", {ustart // 8:#x}, {usz}, {boff - ustart}, {w}, "{self.cty(b.type)}")')
                        n = fname(b.spelling)
                        if n is None:
                            continue
                        acc.append((n, st, boff - b0 * 8, w, rt))
                continue
            n = fname(c.spelling) or f"_unused_{i}"
            rt = self.rty(c.type, f"{name}_{c.spelling}")
            off = c.get_field_offsetof() // 8
            lines.append(f"    pub {n}: {rt},")
            sz = c.type.get_size()
            gcf.append(f'GcField::new("{c.spelling}", {off:#x}, {max(sz, 0)}, "{self.cty(c.type)}")')
            i += 1
        kw = "union" if is_union else "struct"
        gname = name if (not rec.spelling or "(" in rec.spelling) else rec.spelling
        o = self.out
        o.append(f"/// `{self.cty(rec.type)}` ({src}:{loc.line}); GameCube size {gsize:#x}.")
        if not owned(rec):
            o.append("// TODO(move-to-core): shared type, generated here until melee_types owns it.")
        o.append("#[repr(C)]\n#[derive(Clone, Copy)]")
        if not lines:
            lines = ["    pub _empty: [u8; 0],"]
        o.append(f"pub {kw} {name} {{\n" + "\n".join(lines) + "\n}")
        o.append(f"impl GcLayout for {name} {{\n    const GC_NAME: &'static str = \"{gname}\";\n"
                 f"    const GC_SIZE: u32 = {gsize:#x};\n    const GC_ALIGN: u32 = {galign};\n"
                 f"    const GC_FIELDS: &'static [GcField] = &[\n" + "".join(f"        {g},\n" for g in gcf) + "    ];\n}")
        o.append(f"impl {name} {{\n    /// All-zero value (C `memset(0)`).\n    pub fn zeroed() -> Self {{ unsafe {{ core::mem::zeroed() }} }}")
        for n, st, bit, w, rt in acc:
            u0, u1 = ("unsafe { ", " }") if is_union else ("", "")
            o.append(f"    #[inline] pub fn {n}(&self) -> {rt} {{ {u0}bf_get!(self.{st}, {bit}, {w}, {rt}){u1} }}")
            o.append(f"    #[inline] pub fn set_{n.replace('r#', '')}(&mut self, v: {rt}) {{ {u0}bf_set!(self.{st}, {bit}, {w}, v){u1} }}")
        o.append("}\n")

    def run(self, roots):
        for r in roots:
            u = r.get_usr()
            self.rec_name(r)
            if u not in self.done:
                self.done.add(u); self.queue.append(r)
        while self.queue:
            self.emit_record(self.queue.pop(0))
        for usr, n in self.opaque.items():
            if usr in self.done:
                continue
            self.out.append(f"/// Opaque here (only reached by pointer). TODO(move-to-core)\n#[repr(C)]\npub struct {n} {{ _opaque: [u8; 0] }}\n")
        # enums
        seen = set()
        for c in self.tu.cursor.walk_preorder():
            if c.kind == CK.ENUM_DECL and c.is_definition() and owned(c):
                if c.spelling and "unnamed" not in c.spelling:
                    self.enums.setdefault(c.get_usr(), (re.sub(r"\W", "_", c.spelling), SCALAR.get(c.enum_type.get_canonical().kind, "i32"), c))
                for e in c.get_children():
                    if e.kind == CK.ENUM_CONSTANT_DECL and e.spelling not in seen:
                        seen.add(e.spelling)
                        it = SCALAR.get(c.enum_type.get_canonical().kind, "i32")
                        v = e.enum_value
                        if it == "u32": v &= 0xFFFFFFFF
                        self.out.append(f"pub const {e.spelling}: {it} = {v};")
        # typedef names of generated records (`typedef struct _StaticPlayer StaticPlayer;`)
        aliases = []
        for c in self.tu.cursor.walk_preorder():
            if c.kind == CK.TYPEDEF_DECL and owned(c) and c.spelling not in self.used:
                u = c.underlying_typedef_type.get_canonical()
                if u.kind == TK.RECORD:
                    d = u.get_declaration().get_definition() or u.get_declaration()
                    if d.get_usr() in self.names:
                        self.used.add(c.spelling)
                        aliases.append(f"pub type {c.spelling} = {self.names[d.get_usr()]};")
        self.out += sorted(aliases)
        for usr, (n, it, d) in sorted(self.enums.items(), key=lambda x: x[1][0]):
            if n in self.used:
                continue
            self.used.add(n)
            self.out.append(f"pub type {n} = {it};")


def main():
    idx = ci.Index.create()
    tu = idx.parse(str(ROOT / "rust/melee_fighter/gen/types_tu.c"), args=ARGS)
    errs = [d for d in tu.diagnostics if d.severity >= ci.Diagnostic.Error]
    if errs:
        for d in errs[:20]:
            print(d, file=sys.stderr)
        raise SystemExit("gen_types: parse errors")
    roots, seen = [], set()
    for c in tu.cursor.walk_preorder():
        if c.kind in (CK.STRUCT_DECL, CK.UNION_DECL) and c.is_definition() and owned(c) \
                and c.spelling and "unnamed" not in c.spelling and not c.is_anonymous() \
                and c.semantic_parent.kind == CK.TRANSLATION_UNIT and c.get_usr() not in seen:
            seen.add(c.get_usr()); roots.append(c)
    g = Gen(tu)
    # `typedef struct { ... } Name;` (StaticPlayer...): the record is anonymous, name it by the typedef
    for c in tu.cursor.walk_preorder():
        if c.kind == CK.TYPEDEF_DECL and owned(c):
            u = c.underlying_typedef_type.get_canonical()
            if u.kind == TK.RECORD:
                d = u.get_declaration()
                if d.is_definition() and d.get_usr() not in seen and (not d.spelling or "unnamed" in d.spelling or "(" in d.spelling):
                    seen.add(d.get_usr())
                    g.rec_name(d, c.spelling)
                    roots.append(d)
    g.run(roots)
    hdr = ("// @generated by rust/melee_fighter/gen/gen_types.py from the GameCube (ppc32) layout.\n"
           "// Do not edit. Native little-endian #[repr(C)] structs + GcLayout tables.\n"
           "#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]\n"
           "use core::ffi::c_void;\nuse melee_core::gc::{GcField, GcLayout};\n\n")
    dst = ROOT / "rust/melee_fighter/src/types/generated.rs"
    dst.write_text(hdr + "\n".join(g.out) + "\n", encoding="utf-8", newline="\n")
    print(f"gen_types: {len(g.done)} records ({len(roots)} owned roots), {len(g.opaque)} opaque, -> {dst}")


if __name__ == "__main__":
    main()
