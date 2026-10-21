#!/usr/bin/env python3
"""Generate native Rust structs + GameCube layout tables for the group-4 character types.

Inputs (both produced by clang 23 from the C headers, see run_gen.sh):
  ast.json   native (x86_64-pc-windows-msvc, -DTARGET_PC) `-ast-dump=json` of every
             src/melee/ft/kinds/*/types.h: decides the native field set and C types.
  gc.txt     ppc32 (powerpc-none-eabi, no TARGET_PC) `-fdump-record-layouts-complete`:
             gives the original GameCube offset/size/bit position of every field.
Output: src/types_gen.rs (one #[repr(C)] struct/union per C record declared under
src/melee/ft/kinds/ except ftCommon, plus placeholder types for records owned by other groups).

Native layout rules (RUST_PORT.md): ordinary little-endian structs, 8-byte pointers; C
bitfields become one plain integer field each (the GameCube bit position is kept in the
GcLayout table); C `bool` (MSL: int) stays i32; C enums are i32 (-fno-short-enums).

usage: gen_types.py ast.json gc.txt out.rs
"""
import json
import re
import sys

KW = {"type", "loop", "ref", "move", "match", "fn", "in", "box", "use", "mod", "impl", "where",
      "while", "for", "if", "else", "let", "mut", "pub", "struct", "enum", "trait", "true",
      "false", "return", "static", "const", "unsafe", "extern", "crate", "as", "break",
      "continue", "dyn", "abstract", "become", "do", "final", "macro", "override", "priv",
      "typeof", "unsized", "virtual", "yield", "async", "await", "try", "gen"}

SCALARS = {
    "char": "i8", "signed char": "i8", "unsigned char": "u8", "u8": "u8", "s8": "i8",
    "short": "i16", "signed short": "i16", "unsigned short": "u16", "s16": "i16", "u16": "u16",
    "int": "i32", "signed int": "i32", "unsigned int": "u32", "unsigned": "u32", "s32": "i32",
    "u32": "u32", "long": "i32", "unsigned long": "u32", "long long": "i64",
    "unsigned long long": "u64", "s64": "i64", "u64": "u64", "float": "f32", "f32": "f32",
    "double": "f64", "f64": "f64", "long double": "f64", "_Bool": "u8",
}


def norm_loc(s):
    return s.replace("\\", "/")


# ---------------------------------------------------------------- GameCube layout dump
def parse_gc(path):
    """-> {record key: {"size", "align", "fields": [(off, bits|None, ctype, name)]}}"""
    recs = {}
    cur = None
    for line in open(path, encoding="utf-8", errors="replace"):
        line = line.rstrip("\n")
        if line.startswith("*** Dumping AST Record Layout"):
            cur = None
            continue
        if "|" not in line:
            continue
        left, right = line.split("|", 1)
        left = left.strip()
        m = re.match(r"\[sizeof=(\d+), align=(\d+)", right.strip())
        if m and cur is not None:
            cur["size"], cur["align"] = int(m.group(1)), int(m.group(2))
            continue
        if cur is None:
            hm = re.match(r" (struct|union) (.*)$", right)
            if hm and left == "0":
                key = norm_loc(hm.group(2).strip())
                um = re.search(r"\(unnamed at ([^)]*)\)$", key)
                if um:
                    key = "@" + um.group(1)
                cur = {"kind": hm.group(1), "fields": []}
                if key not in recs:
                    recs[key] = cur
            continue
        # depth-1 field lines have exactly three spaces after '|'
        if not right.startswith("   ") or right.startswith("    "):
            continue
        body = right[3:]
        bm = re.match(r"(\d+)(?::(\d+)-(\d+))?$", left)
        if not bm:
            continue
        off = int(bm.group(1))
        bits = None
        if bm.group(2) is not None:
            b0, b1 = int(bm.group(2)), int(bm.group(3))
            bits = (b0, b1 - b0 + 1)
        if body.endswith(")") and "(unnamed" in body:  # anonymous member
            ctype, name = body, ""
        else:
            ctype, _, name = body.rpartition(" ")
        cur["fields"].append((off, bits, ctype.strip(), name))
    return recs


# ---------------------------------------------------------------- native AST
class Ctx:
    def __init__(self):
        self.file = ""
        self.typedefs = {}      # name -> qualType
        self.records = {}       # key -> record info
        self.order = []         # keys of group-owned records in declaration order
        self.rec_by_name = {}   # struct/union tag or typedef name -> key
        self.last_anon = None


def loc_file(node, ctx):
    loc = node.get("loc", {})
    for src in (loc, loc.get("expansionLoc", {}), loc.get("spellingLoc", {})):
        if "file" in src:
            ctx.file = norm_loc(src["file"])
            break
    return ctx.file


def loc_key(node, ctx):
    loc = node.get("loc", {})
    if "expansionLoc" in loc:
        loc = loc["expansionLoc"]
    return "@%s:%s:%s" % (ctx.file, loc.get("line"), loc.get("col"))


def walk(node, ctx, parent=None):
    kind = node.get("kind")
    f = loc_file(node, ctx)
    # `range` may also carry file info for later siblings
    for k in ("begin", "end"):
        r = node.get("range", {}).get(k, {})
        if "file" in r:
            pass
    if kind == "TypedefDecl":
        qt = node["type"]["qualType"]
        ctx.typedefs[node["name"]] = qt
        m = re.match(r"^(struct|union) (\w+)$", qt)
        um = re.search(r"\((?:unnamed|anonymous)(?: struct| union)? at ([^)]*)\)", norm_loc(qt))
        if um:
            key = "@" + um.group(1)
            if key in ctx.records:
                ctx.records[key]["typedef"] = node["name"]
                ctx.rec_by_name[node["name"]] = key
        elif m and m.group(2) in ctx.rec_by_name:
            ctx.rec_by_name.setdefault(node["name"], ctx.rec_by_name[m.group(2)])
        elif m and ctx.last_anon and ctx.last_anon in ctx.records:
            # `typedef struct { ... } X;`: clang names the anonymous tag `struct X`
            ctx.records[ctx.last_anon]["typedef"] = node["name"]
            ctx.rec_by_name[node["name"]] = ctx.last_anon
        ctx.last_anon = None
        return
    if kind == "RecordDecl" and node.get("completeDefinition"):
        key = loc_key(node, ctx)
        info = {"tag": node.get("tagUsed"), "name": node.get("name"), "file": f,
                "fields": [], "key": key, "typedef": None, "parent": parent}
        ctx.records[key] = info
        if info["name"]:
            ctx.rec_by_name[info["name"]] = key
        for ch in node.get("inner", []):
            ck = ch.get("kind")
            if ck == "RecordDecl":
                walk(ch, ctx, parent=key)
            elif ck == "FieldDecl":
                loc_file(ch, ctx)
                width = None
                if ch.get("isBitfield"):
                    for e in ch.get("inner", []):
                        if "value" in e:
                            width = int(e["value"])
                info["fields"].append({"name": ch.get("name", ""),
                                       "qt": norm_loc(ch["type"]["qualType"]),
                                       "dqt": norm_loc(ch["type"].get("desugaredQualType", "")),
                                       "width": width})
        owned = "/melee/ft/kinds/" in f and "/ftCommon/" not in f
        info["owned"] = owned
        if owned:
            ctx.order.append(key)
        ctx.last_anon = key if not info["name"] and parent is None else None
        return
    for ch in node.get("inner", []):
        walk(ch, ctx, parent)


# ---------------------------------------------------------------- type mapping
class Gen:
    def __init__(self, ctx, gc):
        self.ctx, self.gc = ctx, gc
        self.ext_opaque = set()   # pointer-only external records
        self.ext_value = []       # by-value external records (generated in full)
        self.names = {}

    def rust_name(self, key):
        if key in self.names:
            return self.names[key]
        r = self.ctx.records[key]
        n = r["name"] or r["typedef"]
        if not n:
            p = r["parent"]
            pn = self.rust_name(p) if p else "Anon"
            sibs = [k for k, v in self.ctx.records.items() if v["parent"] == p and
                    not (v["name"] or v["typedef"])]
            n = "%s_anon%d" % (pn, sibs.index(key))
        self.names[key] = n
        return n

    def resolve_record(self, t):
        """C type spelling of a record -> record key or None."""
        t = t.replace("const ", "").replace("volatile ", "").strip()
        um = re.search(r"\((?:unnamed|anonymous)(?: struct| union)? at ([^)]*)\)", t)
        if um:
            return "@" + um.group(1)
        m = re.match(r"^(?:struct|union) (\w+)$", t)
        name = m.group(1) if m else t
        if name in self.ctx.rec_by_name:
            return self.ctx.rec_by_name[name]
        if not m and name in self.ctx.typedefs and self.ctx.typedefs[name] != t:
            return self.resolve_record(self.ctx.typedefs[name])
        return None

    def scalar(self, t, depth=0):
        t = t.replace("const ", "").replace("volatile ", "").strip()
        if t in SCALARS:
            return SCALARS[t]
        if t == "bool":
            return "i32"
        if t.startswith("enum "):
            return "i32"
        if t in self.ctx.typedefs and depth < 20:
            return self.scalar(self.ctx.typedefs[t], depth + 1)
        return None

    def map(self, t, by_value=True):
        t = t.strip()
        m = re.match(r"^(.*)\[(\d+)\]$", t)
        if m and "(*" not in t:
            # C "T[a][b]": peel the FIRST dimension as the outermost Rust array
            dims = re.findall(r"\[(\d+)\]", t[len(re.match(r"^[^\[]*", t).group(0)):])
            base = re.match(r"^[^\[]*", t).group(0).strip()
            inner = self.map(base, by_value)
            for d in reversed(dims):
                inner = "[%s; %s]" % (inner, d)
            return inner
        if "(*" in t or re.search(r"\)\s*\(", t):
            return "FnPtr"
        if t.endswith("*"):
            inner = t[:-1].strip()
            inner_c = inner.replace("const ", "").strip()
            if inner_c == "void":
                return "*mut c_void"
            return "*mut " + self.map(inner, by_value=False)
        s = self.scalar(t)
        if s:
            return s
        key = self.resolve_record(t)
        if key and key in self.ctx.records:
            r = self.ctx.records[key]
            if not r["owned"]:
                if by_value:
                    if key not in self.ext_value:
                        self.ext_value.append(key)
                        for f in r["fields"]:
                            self.map(f["qt"], True)
                else:
                    if key not in self.ext_value:
                        self.ext_opaque.add(self.rust_name(key))
            return self.rust_name(key)
        name = re.sub(r"^(struct|union) ", "", t.replace("const ", "")).strip()
        if name in self.ctx.typedefs and self.ctx.typedefs[name] not in (t, "struct " + name,
                                                                          "union " + name):
            return self.map(self.ctx.typedefs[name], by_value)
        name = re.sub(r"\W", "_", name)
        if by_value:
            raise SystemExit("unmappable by-value type %r" % t)
        self.ext_opaque.add(name)
        return name

    def gc_rec(self, key):
        r = self.ctx.records[key]
        for k in (r["name"], r["typedef"], key):
            if k and k in self.gc:
                return self.gc[k]
        return None


def fname(n, i):
    if not n:
        return "anon%d" % i
    if n == "_":
        return "_x"
    if n in ("self", "super", "Self", "crate"):
        return n + "_"
    if n in KW:
        return "r#" + n
    return n


def emit_record(g, key, out, hashes):
    r = g.ctx.records[key]
    name = g.rust_name(key)
    gcr = g.gc_rec(key)
    kind = "union" if r["tag"] == "union" else "struct"
    out.append("/// C `%s %s`%s." % (r["tag"], r["name"] or r["typedef"] or "(anonymous)",
                                     "" if r["owned"] else
                                     " (owned by another group; TODO(move-to-core))"))
    out.append("#[repr(C)]")
    out.append("#[derive(Clone, Copy)]")
    out.append("pub %s %s {" % (kind, name))
    table = []
    hashable = kind == "struct"
    rust_fields = []
    gcf = list(gcr["fields"]) if gcr else []
    for i, f in enumerate(r["fields"]):
        fn = fname(f["name"], i)
        ty = g.map(f["qt"], True)
        if f["width"] is not None:
            # bitfield -> plain integer of the declared type
            ty = g.scalar(f["qt"]) or "u32"
        comment = f["qt"] + ("" if f["width"] is None else " : %d" % f["width"])
        out.append("    pub %s: %s, // %s" % (fn, ty, comment))
        rust_fields.append((fn, ty))
        # GC entry: match by name (anonymous members by position)
        ge = None
        if f["name"]:
            for e in gcf:
                if e[3] == f["name"]:
                    ge = e
                    break
        else:
            anon = [e for e in gcf if e[3] == ""]
            idx = [x for x in r["fields"][:i] if not x["name"]]
            if len(idx) < len(anon):
                ge = anon[len(idx)]
        if ge:
            off, bits, cty, _ = ge
            size = gc_size(g, f, cty)
            cty_s = cty.replace('"', "'")
            if bits:
                b0, w = bits
                sz = (b0 + w + 7) // 8
                table.append('    %s @ 0x%X: %d "%s" bits %d + %d,' % (fn, off, sz, cty_s, b0, w))
            else:
                table.append('    %s @ 0x%X: %d "%s",' % (fn, off, size, cty_s))
        if "*" in ty or ty == "FnPtr":
            pass
    out.append("}")
    if gcr:
        out.append("melee_core::gc_layout!(%s, \"%s\", size 0x%X, align %d, [" %
                   (name, r["name"] or r["typedef"] or name, gcr["size"], gcr["align"]))
        out.extend(table)
        out.append("]);")
    else:
        out.append("// no GameCube layout (PC-only record)")
    if hashable:
        hashes.append((name, rust_fields))
    out.append("")


def gc_size(g, f, cty):
    """Byte size of a field on the GameCube, from the record dump / scalar table."""
    t = cty.strip()
    dims = re.findall(r"\[(\d+)\]", t)
    base = re.sub(r"\[\d+\]", "", t).strip()
    n = 1
    for d in dims:
        n *= int(d)
    if base.endswith("*") or "(*" in base:
        return 4 * n
    s = g.scalar(base)
    if s:
        return {"i8": 1, "u8": 1, "i16": 2, "u16": 2, "i32": 4, "u32": 4, "f32": 4,
                "i64": 8, "u64": 8, "f64": 8}[s] * n
    key = g.resolve_record(base)
    if key:
        gr = g.gc_rec(key)
        if gr:
            return gr["size"] * n
    return 0


def is_hashable(ty, hashable_names):
    base = ty
    while base.startswith("["):
        base = base[1:base.rindex(";")]
    if base.startswith("*mut") or base == "FnPtr":
        return False
    if base in ("i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "f32", "f64"):
        return True
    return base in hashable_names


def main():
    ast_path, gc_path, out_path = sys.argv[1:4]
    gc = parse_gc(gc_path)
    ctx = Ctx()
    walk(json.load(open(ast_path, encoding="utf-8")), ctx)
    g = Gen(ctx, gc)
    body, hashes = [], []
    for key in ctx.order:
        emit_record(g, key, body, hashes)
    ext = []
    done = set()
    i = 0
    while i < len(g.ext_value):  # emitting may discover more
        key = g.ext_value[i]
        i += 1
        if key in done:
            continue
        done.add(key)
        emit_record(g, key, ext, hashes)
    ext_names = {g.rust_name(k) for k in g.ext_value}
    hashable = set()
    changed = True
    cand = dict(hashes)
    while changed:  # fixpoint: struct hashable iff every field type is
        changed = False
        for n, fs in cand.items():
            if n not in hashable and all(is_hashable(t, hashable) for _, t in fs):
                hashable.add(n)
                changed = True
    out = ["// @generated by rust/melee_chara/gen/gen_types.py -- do not edit by hand.",
           "// Native little-endian structs for src/melee/ft/kinds/*/types.h (group 4) with",
           "// their GameCube (ppc32) layout recorded per field for loaders/netplay/m-ex.",
           "#![allow(non_camel_case_types, non_snake_case, dead_code, clippy::all)]",
           "use core::ffi::c_void;",
           "",
           "/// Any C function pointer. TODO(move-to-core): typed fn pointers once ft core lands.",
           "pub type FnPtr = Option<unsafe extern \"C\" fn()>;",
           ""]
    out.append("// ---- opaque records owned by other groups, only used behind pointers")
    out.append("// TODO(move-to-core): replace with the real types from melee_types/melee_fighter.")
    for n in sorted(g.ext_opaque - ext_names):
        out.append("#[repr(C)] pub struct %s { _opaque: [u8; 0] }" % n)
    out.append("")
    out.append("// ---- by-value records owned by other groups (TODO(move-to-core))")
    out.extend(ext)
    out.append("// ---- group-4 records")
    out.extend(body)
    out.append("// ---- StateHash: canonical field values in declaration order, pointers skipped")
    for n, fs in hashes:
        if n not in hashable:
            continue
        out.append("impl melee_core::hash::StateHash for %s {" % n)
        out.append("    fn state_hash(&self, h: &mut melee_core::hash::StateHasher) {")
        for fn, t in fs:
            out.append("        self.%s.state_hash(h);" % fn)
        if not fs:
            out.append("        let _ = h;")
        out.append("    }")
        out.append("}")
    out.append("")
    out.append("/// (name, GC sizeof, fields) of every generated record with a GameCube layout.")
    out.append("pub fn all_gc_tables() -> ::std::vec::Vec<(&'static str, u32, &'static [melee_core::gc::GcField])> {")
    out.append("    use melee_core::gc::GcLayout;")
    out.append("    vec![")
    for k in ctx.order + g.ext_value:
        if g.gc_rec(k):
            n = g.rust_name(k)
            out.append("        (%s::GC_NAME, %s::GC_SIZE, %s::GC_FIELDS)," % (n, n, n))
    out.append("    ]")
    out.append("}")
    out.append("/// Every generated record with a GameCube layout, for table-wide tests.")
    out.append("pub const GC_RECORD_COUNT: usize = %d;" %
               sum(1 for k in ctx.order + g.ext_value if g.gc_rec(k)))
    open(out_path, "w", encoding="utf-8", newline="\n").write("\n".join(out) + "\n")
    print("records: %d owned, %d external by value, %d opaque; %d hashable" %
          (len(ctx.order), len(g.ext_value), len(g.ext_opaque - ext_names), len(hashable)))


if __name__ == "__main__":
    main()
