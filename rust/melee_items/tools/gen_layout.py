#!/usr/bin/env python3
"""Generate native Rust structs + GameCube offset tables for port group 5 (it/ty/vi).

Input (both produced by clang from the real headers, see `gen_layout.sh`):
  * a `-fdump-record-layouts-complete` dump for --target=powerpc-none-eabi (the GC layout);
  * a `-ast-dump` grep of TypedefDecl/RecordDecl lines (typedef -> canonical type, and where
    each record is defined).

Output: one Rust file with, for every record defined under src/melee/{it,ty,vi} plus every
record those embed by value:
  * a native `#[repr(C)]` struct/union (little-endian, native pointer width);
  * a `melee_core::gc_layout!` table: every C field with its GameCube offset/size (and PPC
    MSB-first bit numbering for bitfields) - the m-ex "GC offset -> native field" record;
  * bitfield storage units as plain integers holding the bits in GC numbering, with
    getter/setter accessors per C bitfield.

The generator self-checks: it recomputes every record's GC layout from the field types using
the ppc32 EABI rules and fails if an offset or size disagrees with clang's dump.
"""
import re
import sys
from collections import OrderedDict

LAY, DECLS, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
MINE = re.compile(r"src[\\/]melee[\\/](it|ty|vi)[\\/]")

# ---------------------------------------------------------------- typedefs / record files
typedefs = {}       # name -> canonical C type string
rec_file = {}       # "struct Tag" -> file (definitions only)
cur_file = ""
for line in open(DECLS, encoding="utf-8", errors="replace"):
    m = re.search(r"<([A-Za-z]:?[^<>:]*?\.[ch]):\d+", line)
    if m:
        cur_file = m.group(1)
    m = re.match(r"\|-TypedefDecl .*? (?:referenced )?(\w+) '", line.rstrip())
    if m and " implicit " not in line:
        quoted = re.findall(r"'([^']*)'", line)
        typedefs[m.group(1)] = quoted[-1]
        continue
    m = re.match(r"\|-RecordDecl .*? (struct|union) (\w+) definition", line)
    if m:
        rec_file[f"{m.group(1)} {m.group(2)}"] = cur_file

# ---------------------------------------------------------------- layout dump
LINE = re.compile(r"^\s*(\d+)(?::(\d+)-(\d+))? \| (\s*)(.*?)\s*$")


class Node:
    def __init__(self, off, bits, depth, text):
        self.off, self.bits, self.depth, self.text = off, bits, depth, text
        self.children = []


blocks = OrderedDict()   # record key -> (root Node, size, align)


def rec_key(text):
    """'struct Item' / 'union X::(anonymous at f:l:c)' -> stable key."""
    text = text.strip()
    m = re.search(r"\((?:anonymous|unnamed)(?: struct| union)? at ([^)]*)\)\s*$", text)
    kind = text.split()[0]
    if m:
        return f"{kind} @{m.group(1).replace(chr(92), '/')}"
    return text


lines = open(LAY, encoding="utf-8", errors="replace").read().split("\n")
i = 0
while i < len(lines):
    if "*** Dumping AST Record Layout" not in lines[i]:
        i += 1
        continue
    i += 1
    stack = []
    root = None
    while i < len(lines):
        ln = lines[i]
        m = re.search(r"\[sizeof=(\d+),.*?align=(\d+)", ln)
        if m:
            blocks.setdefault(rec_key(root.text), (root, int(m.group(1)), int(m.group(2))))
            break
        m = LINE.match(ln)
        if m:
            depth = len(m.group(4)) // 2
            bits = (int(m.group(2)), int(m.group(3))) if m.group(2) else None
            n = Node(int(m.group(1)), bits, depth, m.group(5))
            while stack and stack[-1].depth >= depth:
                stack.pop()
            if stack:
                stack[-1].children.append(n)
            else:
                root = n
            stack.append(n)
        i += 1
    i += 1

# ---------------------------------------------------------------- C type model
SCALARS = {  # canonical C -> (rust, size, align, signed)
    "char": ("i8", 1, 1, True), "signed char": ("i8", 1, 1, True),
    "unsigned char": ("u8", 1, 1, False), "_Bool": ("u8", 1, 1, False),
    "short": ("i16", 2, 2, True), "unsigned short": ("u16", 2, 2, False),
    "int": ("i32", 4, 4, True), "unsigned int": ("u32", 4, 4, False),
    "long": ("i32", 4, 4, True), "unsigned long": ("u32", 4, 4, False),
    "long long": ("i64", 8, 8, True), "unsigned long long": ("u64", 8, 8, False),
    "float": ("f32", 4, 4, True), "double": ("f64", 8, 8, True),
    "long double": ("f64", 8, 8, True),
}


def split_field(text):
    """'HSD_JObj *[6] x10C_joint' -> ('HSD_JObj *[6]', 'x10C_joint'); anonymous -> (text, None)."""
    if text.endswith(")") and "(*)" not in text.split(")")[-1]:
        if re.search(r"\((anonymous|unnamed)[^)]*\)$", text):
            return text, None
    m = re.match(r"^(.*\S)\s+(\w+)$", text)
    if not m:
        return text, None
    return m.group(1), m.group(2)


class CType:
    """kind: scalar | ptr | fnptr | array | record | enum | void"""

    def __init__(self, kind, **kw):
        self.kind = kind
        self.__dict__.update(kw)


def strip_quals(t):
    t = re.sub(r"\b(const|volatile|restrict|__restrict)\b", "", t)
    return re.sub(r"\s+", " ", t).strip()


def parse_type(t, depth=0):
    t = strip_quals(t)
    if depth > 40:
        raise RuntimeError("typedef loop: " + t)
    if "(*)" in t or re.search(r"\(\*\s*\w*\)", t):
        # array of fn pointers: 'void (*[4])(X)'
        m = re.search(r"\(\*\[(\d+)\]\)", t)
        if m:
            return CType("array", elem=CType("fnptr", sig=t), n=int(m.group(1)))
        return CType("fnptr", sig=t)
    m = re.match(r"^(.*?)((?:\[\d+\])+)$", t)
    if m:
        dims = [int(x) for x in re.findall(r"\[(\d+)\]", m.group(2))]
        ct = parse_type(m.group(1), depth + 1)
        for n in reversed(dims):
            ct = CType("array", elem=ct, n=n)
        return ct
    if t.endswith("*"):
        return CType("ptr", to=parse_type(t[:-1], depth + 1))
    if t in SCALARS:
        return CType("scalar", c=t)
    if t == "void":
        return CType("void")
    m = re.match(r"^(struct|union) (.*)$", t)
    if m:
        return CType("record", key=rec_key(t))
    m = re.match(r"^enum (.*)$", t)
    if m:
        return CType("enum", name=m.group(1))
    if t in typedefs:
        ct = parse_type(typedefs[t], depth + 1)
        ct.alias = t
        return ct
    raise RuntimeError("unknown C type: " + repr(t))


def gc_size_align(ct):
    k = ct.kind
    if k == "scalar":
        return SCALARS[ct.c][1], SCALARS[ct.c][2]
    if k in ("ptr", "fnptr", "enum"):
        return 4, 4
    if k == "array":
        s, a = gc_size_align(ct.elem)
        return s * ct.n, a
    if k == "record":
        if ct.key not in blocks:
            raise RuntimeError("record without layout: " + ct.key)
        _, s, a = blocks[ct.key]
        return s, a
    raise RuntimeError("sizeless type " + k)


def synth_blocks():
    """Records that clang never dumped on their own (e.g. `struct Vec`): rebuild them from
    the nested expansion of a field that embeds them, computing size/align by ppc32 rules."""
    def walk(n):
        for ch in n.children:
            if ch.children:
                ty, _ = split_field(ch.text)
                try:
                    ct = parse_type(ty)
                except RuntimeError:
                    ct = None
                while ct is not None and ct.kind == "array":
                    ct = ct.elem
                if ct is not None and ct.kind == "record" and ct.key not in blocks:
                    kind = ct.key.split()[0]
                    r = Node(0, None, 0, ct.key if "@" not in ct.key else ty)
                    base = ch.off
                    def clone(src, dst, base):
                        for c in src.children:
                            m = Node(c.off - base, c.bits, c.depth, c.text)
                            dst.children.append(m)
                            clone(c, m, base)
                    # direct children only become fields; deeper ones keep their nesting
                    clone(ch, r, base)
                    if ct.key.startswith("union"):
                        r.text = "union " + ct.key.split(" ", 1)[1]
                    blocks[ct.key] = (r, None, None)
                walk(ch)
    for root, _, _ in list(blocks.values()):
        walk(root)
    def size_of(key):
        root, s, a = blocks[key]
        if s is not None:
            return s, a
        is_union = root.text.strip().startswith("union")
        end, al = 0, 1
        for c in root.children:
            ty, _ = split_field(c.text)
            cs, ca = gc_size_align(parse_type(ty))
            al = max(al, ca)
            if c.bits is not None:
                end = max(end, (c.off * 8 + c.bits[1]) // (cs * 8) * cs + cs)
            else:
                end = max(end, c.off + cs) if not is_union else max(end, cs)
        s = (end + al - 1) // al * al
        blocks[key] = (root, s, al)
        return s, al
    for k in list(blocks):
        if blocks[k][1] is None:
            size_of(k)
    return size_of


_size_of = None
_orig_gsa = gc_size_align


def gc_size_align(ct):
    if ct.kind == "record" and ct.key in blocks and blocks[ct.key][1] is None:
        return _size_of(ct.key)
    if ct.kind == "array":
        s, a = gc_size_align(ct.elem)
        return s * ct.n, a
    return _orig_gsa(ct)


_size_of = synth_blocks()

# ---------------------------------------------------------------- naming
RUST_KW = set("as break const continue crate else enum extern false fn for if impl in let loop "
              "match mod move mut pub ref return self Self static struct super trait true type "
              "unsafe use where while async await dyn abstract become box do final macro "
              "override priv typeof unsized virtual yield try union".split())
RENAME_TYPES = {"Vec": "Vec3", "Vec2": "Vec2", "Vec3": "Vec3"}
rust_names = {}   # key -> rust ident
used = set()


def rust_type_name(key):
    if key in rust_names:
        return rust_names[key]
    if "@" in key:
        kind, loc = key.split(" @")
        f, l, c = loc.rsplit(":", 2)
        base = re.sub(r"\W", "_", f.split("/")[-1].split(".")[0])
        name = f"Anon_{base}_{l}_{c}"
    else:
        name = key.split(" ", 1)[1]
        name = RENAME_TYPES.get(name, name)
        name = re.sub(r"\W", "_", name)
        if name[0].isdigit() or name.startswith("_"):
            name = "C" + name
    n, k = name, 2
    while n in used:
        n, k = f"{name}_{k}", k + 1
    used.add(n)
    rust_names[key] = n
    return n


def field_ident(name, taken):
    if name is None or name == "_" or name == "":
        name = "unk"
    if name in RUST_KW:
        name = name + "_"
    n, k = name, 2
    while n in taken:
        n, k = f"{name}_{k}", k + 1
    taken.add(n)
    return n


# ---------------------------------------------------------------- collect records
wanted = OrderedDict()
opaque = set()
for key, (root, size, align) in blocks.items():
    if "@" in key:
        loc = key.split("@")[1]
        mine = MINE.search(loc)
    else:
        f = rec_file.get(key, "")
        mine = MINE.search(f)
    if mine:
        wanted[key] = True


def note_type(ct, by_value):
    if ct.kind == "record":
        if by_value:
            if ct.key not in wanted:
                wanted[ct.key] = True
                visit(ct.key)
        elif ct.key not in blocks:
            opaque.add(ct.key)
    elif ct.kind == "array":
        note_type(ct.elem, by_value)
    elif ct.kind == "ptr":
        note_type(ct.to, False)


visited = set()


def visit(key):
    if key in visited:
        return
    visited.add(key)
    root, _, _ = blocks[key]
    for ch in root.children:
        ty, _ = split_field(ch.text)
        ct = parse_type(ty)
        note_type(ct, True)


for k in list(wanted):
    visit(k)
# pointers to records we don't generate become opaque markers
for key in list(blocks):
    pass

# ---------------------------------------------------------------- emit


def rust_ty(ct):
    k = ct.kind
    if k == "scalar":
        return SCALARS[ct.c][0]
    if k == "enum":
        return "i32"
    if k == "fnptr":
        return "FnPtr"
    if k == "void":
        return "c_void"
    if k == "array":
        return f"[{rust_ty(ct.elem)}; {ct.n}]"
    if k == "record":
        if ct.key in wanted:
            return rust_type_name(ct.key)
        return opaque_name(ct.key)
    if k == "ptr":
        to = ct.to
        if to.kind == "record" and to.key not in wanted:
            return f"*mut {opaque_name(to.key)}"
        if to.kind == "void":
            return "*mut c_void"
        return f"*mut {rust_ty(to)}"
    raise RuntimeError(k)


opaque_names = {}


def opaque_name(key):
    if key not in opaque_names:
        opaque_names[key] = rust_type_name(key) if key not in rust_names else rust_names[key]
    return opaque_names[key]


def c_spelling(ty):
    return ty.replace("\\", "/").replace('"', "'")


out = []
emit = out.append
problems = []

for key in wanted:
    root, size, align = blocks[key]
    is_union = root.text.strip().startswith("union")
    name = rust_type_name(key)
    taken = set()
    fields = []      # (ident, rust type, doc)
    gc_rows = []     # (cname, off, size, ctype, bits)
    accessors = []   # (cname ident, storage ident, shift, width, rust int, signed)
    units = {}       # (unit_off, unit_size) -> storage ident
    calc_off = 0
    calc_align = 1
    for ch in root.children:
        ty, fname = split_field(ch.text)
        ct = parse_type(ty)
        csz, cal = gc_size_align(ct)
        calc_align = max(calc_align, cal)
        if ch.bits is not None:
            unit_bits = csz * 8
            abs_bit = ch.off * 8 + ch.bits[0]
            width = ch.bits[1] - ch.bits[0] + 1
            unit_off = (abs_bit // unit_bits) * csz
            first = abs_bit - unit_off * 8
            uk = (unit_off, csz)
            if uk not in units:
                if is_union and units:
                    pass
                units[uk] = field_ident(f"bf_{unit_off:x}", taken)
                fields.append((units[uk], {1: "u8", 2: "u16", 4: "u32", 8: "u64"}[csz],
                               f"Bitfield storage at GC +0x{unit_off:X} ({csz} bytes), "
                               "bits in GC (MSB-first) numbering."))
            signed = ct.kind == "scalar" and SCALARS[ct.c][3] or ct.kind == "enum"
            gc_rows.append((fname, unit_off, csz, ty, (first, width)))
            accessors.append((field_ident(fname, set()), units[uk], unit_bits - first - width,
                              width, {1: "u8", 2: "u16", 4: "u32", 8: "u64"}[csz], signed,
                              fname, unit_off, first))
            calc_off = max(calc_off, unit_off + csz)
            continue
        if not is_union:
            exp = (calc_off + cal - 1) // cal * cal
            if exp != ch.off:
                problems.append(f"{key}.{fname}: computed +{exp} vs clang +{ch.off}")
            calc_off = ch.off + csz
        else:
            if ch.off != 0:
                problems.append(f"{key}.{fname}: union member at +{ch.off}")
            calc_off = max(calc_off, csz)
        ident = field_ident(fname if fname else f"anon_{ch.off:x}", taken)
        fields.append((ident, rust_ty(ct), f"GC +0x{ch.off:X}, {csz} bytes: `{c_spelling(ty)}`"))
        gc_rows.append((ident, ch.off, csz, ty, None))
    calc_size = (calc_off + calc_align - 1) // calc_align * calc_align
    if calc_size != size or calc_align != align:
        problems.append(f"{key}: computed size {calc_size}/align {calc_align} vs clang {size}/{align}")

    emit(f"/// `{c_spelling(root.text.strip())}`: GC sizeof 0x{size:X}, align {align}.")
    emit("#[repr(C)]")
    emit("#[derive(Clone, Copy)]")
    kw = "union" if is_union else "struct"
    emit(f"pub {kw} {name} {{")
    if not fields:
        emit("    pub _empty: [u8; 0],")
    for ident, rt, doc in fields:
        emit(f"    /// {doc}")
        emit(f"    pub {ident}: {rt},")
    emit("}")
    emit(f"impl {name} {{")
    emit("    /// All-zero value (valid: null pointers, `None` callbacks, zero numbers).")
    emit("    pub fn zeroed() -> Self {")
    emit("        // SAFETY: every field type of a generated record is valid as all-zero bytes.")
    emit("        unsafe { core::mem::zeroed() }")
    emit("    }")
    for (acc, store, shift, width, it, signed, cname, uoff, first) in accessors:
        mask = (1 << width) - 1
        emit(f"    /// C bitfield `{cname}`: GC +0x{uoff:X} bits {first}..{first + width - 1} (MSB=0).")
        emit("    #[inline]")
        if is_union:
            emit(f"    pub fn {acc}(&self) -> u32 {{ (unsafe {{ self.{store} }} as u32 >> {shift}) & 0x{mask:X} }}")
        else:
            emit(f"    pub fn {acc}(&self) -> u32 {{ (self.{store} as u32 >> {shift}) & 0x{mask:X} }}")
        if signed and width < 32:
            emit("    #[inline]")
            sh = 32 - width
            emit(f"    pub fn {acc}_signed(&self) -> i32 {{ ((self.{acc}() << {sh}) as i32) >> {sh} }}")
        emit("    #[inline]")
        body = (f"let m: {it} = (0x{mask:X} as {it}) << {shift}; "
                f"let s = (self.{store} & !m) | ((((v as u64) & 0x{mask:X}) as {it}) << {shift}); "
                f"self.{store} = s;")
        if is_union:
            body = "unsafe { " + body + " }"
        emit(f"    pub fn set_{acc}(&mut self, v: u32) {{ {body} }}")
    emit("}")
    emit(f"crate::gc_layout!({name}, \"{c_spelling(root.text.strip())}\", size 0x{size:X}, align {align}, [")
    for cname, off, sz, ty, bits in gc_rows:
        cname = re.sub(r"[^\w]", "_", cname or "unk")
        if cname in RUST_KW:
            cname += "_"
        b = f" bits {bits[0]} + {bits[1]}" if bits else ""
        emit(f"    {cname} @ 0x{off:X}: {sz} \"{c_spelling(ty)}\"{b},")
    emit("]);")
    emit("")

opq = []
for key, nm in sorted(opaque_names.items(), key=lambda x: x[1]):
    if key in wanted:
        continue
    opq.append(f"/// Opaque `{key}` (owned by another port group; only pointed to from here).")
    opq.append(f"#[repr(C)]\npub struct {nm} {{ _opaque: [u8; 0] }}")

hdr = [
    "// @generated by rust/melee_items/tools/gen_layout.py from clang's ppc32 record-layout dump",
    "// of the group-5 headers. Do not edit by hand: rerun tools/gen_layout.sh.",
    "// TODO(move-to-core): records owned by other groups (Vec3, HSD types, lb types...) are",
    "// generated here because group 5 embeds them by value; they move to melee_types later.",
    "#![allow(non_camel_case_types, non_snake_case, dead_code, clippy::all)]",
    "use core::ffi::c_void;",
    "",
    "/// Any C function pointer. TODO: typed signatures once callee crates exist.",
    "pub type FnPtr = Option<unsafe extern \"C\" fn()>;",
    "",
]
text = "\n".join(hdr + opq + [""] + out)
text = text.replace("crate::gc_layout!", "melee_core::gc_layout!")
open(OUT, "w", encoding="utf-8", newline="\n").write(text)
print(f"{len(wanted)} records, {len(opq)//2} opaque, {len(problems)} layout problems")
for p in problems[:50]:
    print("  " + p)
sys.exit(1 if problems else 0)
