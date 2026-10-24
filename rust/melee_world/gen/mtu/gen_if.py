#!/usr/bin/env python3
"""C-oracle generator for the per-TU ports in rust/melee_world/src/tu/.

  python rust/melee_world/gen/mtu/gen_tu.py <stem> [--ext]      (from the worktree root)

For the game TU named in specs.py this parses the C with libclang (the native flags of
melee_world/oracle/build_r10.sh: x86_64-pc-windows-gnu, -mno-ms-bitfields, so field offsets match
melee_types), finds every function / data object the TU's functions use but do not define, and
writes
  gen/mtu/tu/<stem>_gen.h        C: recording stubs for those callees, extern data definitions,
                                  and fill_<T>/hash_<T> for the state types in the spec
  tests/mtu/<stem>_gen.rs         Rust: the same stubs as `ext::hook` installers (same trace),
                                  and the same fill_<T>/hash_<T>, draw for draw
With --ext it also prints the `pub mod ext { ... }` block for the port file.

Trace convention (tests/mtu_harness/mod.rs mirrors gen/mtu/tu/mtu_common.h): a stub does
T(id) then T() of every argument (pointer -> PI, function pointer -> FI, string -> hash), and
returns a value from the shared xorshift stream, so a divergence in call order, arguments or
values shows in the trace hash. Computed from C source only: no disc data.
"""
import os, re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
sys.path.insert(0, str(HERE))
os.chdir(ROOT)
import gen_types as g  # noqa: E402

ci, K, T = g.ci, g.K, g.T
NAT = [a.replace("x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu") for a in g.NAT] + ["-mno-ms-bitfields"]
GEN_RS = (ROOT / "rust/melee_types/src/gen.rs").read_text(encoding="utf-8")
KNOWN = set(re.findall(r"^pub (?:struct|union|type) (\w+)", GEN_RS, re.M))

REC_FIELDS = {}
for _m in re.finditer(r"^pub (?:struct|union) (\w+) \{\n(.*?)^\}", GEN_RS, re.M | re.S):
    REC_FIELDS[_m.group(1)] = dict(re.findall(r"^\s+pub ([\w#]+): ([^\n]+),$", _m.group(2), re.M))


def strip_arr(t):
    """element type of a Rust array type text `[T; N]` (nested arrays stripped)"""
    t = t.strip()
    while t.startswith("["):
        t = t[1:t.rindex(";")].strip()
    return t


PRIM = g.PRIM
INTS = {T.CHAR_S, T.SCHAR, T.CHAR_U, T.UCHAR, T.SHORT, T.USHORT, T.INT, T.UINT, T.LONG, T.ULONG,
        T.LONGLONG, T.ULONGLONG, T.BOOL}


def sh(path):
    return os.path.relpath(path, ROOT).replace("\\", "/")


def patched_source(stem, spec):
    """The TU source the oracle compiles: the game's file verbatim, except for the literal
    replacements the spec lists (each must match exactly once; documented in the spec)."""
    src = spec["src"]
    if not spec.get("patch"):
        return src
    txt = (ROOT / src).read_bytes().decode("utf-8").replace(chr(13) + chr(10), chr(10))
    for a, b in spec["patch"]:
        assert txt.count(a) == 1, f"{stem}: patch text {a!r} matches {txt.count(a)} times"
        txt = txt.replace(a, b)
    out = ROOT / "rust/target/mtu" / f"{stem}_tu.c"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(txt.encode("utf-8"))
    return str(out.relative_to(ROOT)).replace("\\", "/")


class Tu:
    def __init__(self, src, extra_args=(), incdir=None, extra_main=()):
        self.extra_main = {(ROOT / x).resolve() for x in extra_main}
        self.src = src
        idx = ci.Index.create()
        inc = ["-I", str(incdir)] if incdir else []
        self.tu = idx.parse(src, args=NAT + inc + list(extra_args), options=ci.TranslationUnit.PARSE_DETAILED_PROCESSING_RECORD)
        errs = [d for d in self.tu.diagnostics if d.severity >= ci.Diagnostic.Error]
        if errs:
            raise SystemExit("clang errors:\n" + "\n".join(str(e) for e in errs[:20]))
        self._src = {}
        self.main = (ROOT / src).resolve()
        self.tagged, self.typedefd = set(), set()
        self.main_vars = set()
        self.defs = {}      # name -> cursor, defined in the TU's own file
        self.types = {}     # spelling -> type
        for c in self.tu.cursor.get_children():
            if c.kind == K.FUNCTION_DECL and c.is_definition() and self.in_main(c):
                self.defs[c.spelling] = c
            if c.kind == K.VAR_DECL and self.in_main(c) and c.storage_class != ci.StorageClass.EXTERN:
                self.main_vars.add(c.spelling)
            if c.kind in (K.TYPEDEF_DECL, K.STRUCT_DECL, K.UNION_DECL) and c.spelling:
                self.types.setdefault(c.spelling, c.type)
                if c.kind == K.TYPEDEF_DECL:
                    self.tagged.discard(c.spelling)
                    self.typedefd.add(c.spelling)
                elif c.spelling not in self.typedefd:
                    self.tagged.add(c.spelling)

    def in_main(self, c):
        f = c.location.file
        if f is None:
            return False
        p = Path(f.name).resolve()
        return p == self.main or p in self.extra_main

    def uses(self):
        """-> (stubs{name: [cursor, called, addr_taken]}, inlines{name}, data{name: cursor})"""
        stubs, inlines, data = {}, {}, {}
        for fn in self.defs.values():
            self.scan(fn, stubs, inlines, data)
        # initializers of the TU's own data (the GameModeState tables name functions and data)
        for c in self.tu.cursor.get_children():
            if c.kind == K.VAR_DECL and c.is_definition() and self.in_main(c):
                self.scan(c, stubs, inlines, data)
        return stubs, inlines, data

    def scan(self, root, stubs, inlines, data):
        def visit(c, parent):
            if c.kind == K.DECL_REF_EXPR:
                r = c.referenced
                if r is not None and r.kind == K.FUNCTION_DECL:
                    d = r.get_definition()
                    name = r.spelling
                    if d is None:
                        e = stubs.setdefault(name, [r, False, False])
                        called = parent is not None and parent.kind in (K.CALL_EXPR,) and \
                            next(iter(parent.get_children()), None) is not None
                        # a reference below a CALL_EXPR's first child, through implicit casts, is a call
                        if self.is_callee(c, parent):
                            e[1] = True
                        else:
                            e[2] = True
                    elif not self.in_main(d):
                        if name not in inlines:
                            inlines[name] = d
                            self.scan(d, stubs, inlines, data)
                elif r is not None and r.kind == K.VAR_DECL:
                    d = r.get_definition()
                    if d is None and r.linkage == ci.LinkageKind.EXTERNAL or \
                       (d is not None and not self.in_main(d) and not r.type.is_const_qualified()):
                        data[r.spelling] = r
            for ch in c.get_children():
                visit(ch, c)
        visit(root, None)

    def is_callee(self, ref, parent):
        # climb: parent chain not available; approximate with source text: next token is '('
        toks = list(ref.get_tokens())
        if not toks:
            return True
        ext = ref.extent
        f = ext.end.file
        key = Path(f.name).resolve()
        if key not in self._src:
            self._src[key] = key.read_bytes()
        off = ext.end.offset
        rest = self._src[key][off:off + 16].lstrip()
        return rest.startswith(b"(")


# ---------------------------------------------------------------------------- type mapping

def canon(t):
    return t.get_canonical()


def is_fnptr(t):
    c = canon(t)
    return c.kind == T.POINTER and canon(c.get_pointee()).kind in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO)


def rust_ty(t):
    """Rust spelling of C type t (as melee_types spells fields and ports spell parameters)."""
    c = canon(t)
    k = c.kind
    if k == T.ENUM:
        n = t.spelling
        if n in KNOWN:
            return n
        return PRIM[canon(c.get_declaration().enum_type).kind]
    if k in PRIM:
        n = t.spelling
        if k == T.BOOL:
            return "bool"
        return PRIM[k] if k != T.VOID else "()"
    if k == T.POINTER:
        pt = c.get_pointee()
        if canon(pt).kind in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO):
            fp = canon(pt)
            args = ", ".join(rust_ty(a) for a in fp.argument_types())
            r = fp.get_result()
            rs = "" if canon(r).kind == T.VOID else f" -> {rust_ty(r)}"
            return f'Option<unsafe extern "C" fn({args}){rs}>'
        mut = "*const" if pt.is_const_qualified() else "*mut"
        cp = canon(pt)
        if cp.kind == T.VOID:
            return f"{mut} c_void"
        if cp.kind in PRIM or cp.kind == T.ENUM:
            n = pt.spelling.replace("const ", "").strip()
            return f"{mut} {n if n in KNOWN else rust_ty(pt)}"
        if cp.kind == T.POINTER:
            return f"{mut} {rust_ty(pt)}"
        if cp.kind == T.RECORD:
            n = pt.spelling.replace("const ", "").replace("struct ", "").replace("union ", "").strip()
            if n in KNOWN:
                return f"{mut} {n}"
            dn = cp.get_declaration().spelling
            if dn in KNOWN:
                return f"{mut} {dn}"
            return f"{mut} c_void /* {n} */"
        return f"{mut} c_void"
    if k == T.CONSTANTARRAY:
        return f"[{rust_ty(c.element_type)}; {c.element_count}]"
    if k == T.INCOMPLETEARRAY:
        return f"[{rust_ty(c.element_type)}; 0]"
    if k == T.RECORD:
        n = t.spelling.replace("struct ", "").replace("union ", "")
        if n in KNOWN:
            return n
        dn = c.get_declaration().spelling
        if dn in KNOWN:
            return dn
        return f"c_void /* {n} */"
    return f"c_void /* {c.spelling} */"


def arg_class(t):
    c = canon(t)
    k = c.kind
    if k == T.ENUM:
        return "i4"
    if k in INTS:
        sz = c.get_size()
        return "i8" if sz == 8 else "i4"
    if k == T.FLOAT:
        return "f4"
    if k in (T.DOUBLE, T.LONGDOUBLE):
        return "f8"
    if k == T.POINTER:
        if is_fnptr(t):
            return "fp"
        pk = canon(c.get_pointee()).kind
        if pk in (T.CHAR_S, T.SCHAR, T.CHAR_U, T.UCHAR) and c.get_pointee().is_const_qualified():
            return "str"
        return "ptr"
    if k == T.VOID:
        return "void"
    return "rec"


def c_trace(cls, a):
    return {"i4": f"T((u32) {a});", "i8": f"T((u32) {a}); T((u32) ((u64) {a} >> 32));",
            "f4": f"T(B({a}));", "f8": f"T64({a});", "fp": f"T(FI((const void*) {a}));",
            "str": f"T(SH((const char*) {a}));", "ptr": f"T(PI((const void*) {a}));",
            "rec": f"T(RH(&{a}, sizeof {a}));"}[cls]


def r_trace(cls, a):
    return {"i4": f"t({a} as u32);", "i8": f"t({a} as u32); t((({a} as u64) >> 32) as u32);",
            "f4": f"t({a}.to_bits());", "f8": f"t64({a});", "fp": f"t(fi({a}));",
            "str": f"t(sh({a} as *const u8));", "ptr": f"t(pi({a}));", "rec": f"t(rh(&{a}));"}[cls]


def c_ret(cls):
    return {"i4": "fv()", "i8": "fv64()", "f4": "rg()", "f8": "(double) rg()",
            "ptr": "0", "fp": "0", "str": "0"}.get(cls)


def r_ret(cls, t):
    return {"i4": "fv() as _", "i8": "fv64() as _", "f4": "rg()",
            "f8": "rg() as f64", "ptr": "core::ptr::null_mut()", "fp": "None", "str": "core::ptr::null()"}.get(cls)


# ---------------------------------------------------------------------------- state fill/hash

def gc4(t):
    """True when t is (a typedef of) a pointer-sized integer, which melee_types keeps at 4 bytes."""
    seen = 0
    while t.kind == T.TYPEDEF and seen < 8:
        if t.spelling in ("uintptr_t", "intptr_t", "size_t", "gc_size_t", "ptrdiff_t"):
            return True
        t = t.get_declaration().underlying_typedef_type
        seen += 1
    return False


def rust_field(n):
    return g.ident(n)


def leaves(rec, base_bits):
    for ch in rec.get_children():
        if ch.kind == K.FIELD_DECL and ch.spelling:
            yield ch, base_bits + rec.type.get_offset(ch.spelling)
        elif ch.kind in (K.STRUCT_DECL, K.UNION_DECL) and ch.is_anonymous_record_decl():
            yield from leaves(ch, base_bits + anon_offset_(rec, ch) * 8)


def anon_offset_(parent, anon):
    for lf, lo in leaves(anon, 0):
        return (parent.type.get_offset(lf.spelling) - lo) // 8
    return 0


class Walk:
    anon_offset = staticmethod(anon_offset_)

    """Field-by-field fill/hash of one C record type (bitfields by value, pointers by id)."""

    def __init__(self, tu, rules=None, spec=None):
        self.tu = tu
        self.rules = rules or {}
        self.cur_key = None
        self.parent_rs = None
        spec = spec or {}
        self.walk2 = bool(spec.get("walk2"))
        self.uv_only = spec.get("uv_only", {})
        self.uv_keys = {}      # union key -> member names (declared once in gen.h / gen.rs)

    def rec_of(self, name):
        t = self.tu.types.get(name)
        if t is None:
            raise SystemExit(f"state type {name} not found in the TU")
        d = canon(t).get_declaration()
        return d.get_definition() or d

    def gen(self, name):
        decl = self.rec_of(name)
        cf, ch, rf, rh = [], [], [], []
        self.fields(decl, "p->", "p.", 0, cf, ch, rf, rh, "p", 0, rs=name)
        cn = ("struct " + name) if name in self.tu.tagged else name
        cdef = f"static void fill_{name}({cn}* p)\n{{\n    int i0, i1, i2, j0, j1, j2, j3;\n" + "\n".join("    " + x for x in cf) + \
            f"\n}}\nstatic void hash_{name}(const {cn}* p)\n{{\n    int i0, i1, i2, j0, j1, j2, j3;\n    if (HDBG) printf(\"B {name}%c\", 10);\n" + "\n".join("    " + x for x in ch) + "\n}\n"
        rdef = f"pub fn fill_{name}(p: &mut {name}) {{ unsafe {{\n" + "\n".join("    " + x for x in rf) + \
            f"\n}} }}\npub fn hash_{name}(p: &{name}) {{ unsafe {{\n    if HDBG {{ println!(\"B {name}\"); }}\n" + "\n".join("    " + x for x in rh) + "\n} }\n"
        return cdef, rdef

    def fields(self, decl, cp, rp, base_off, cf, ch, rf, rh, rootc, depth, rs=None):
        anon_i = 0
        for f in decl.get_children():
            if f.kind != K.FIELD_DECL:
                if self.walk2 and f.kind in (K.STRUCT_DECL, K.UNION_DECL) and f.is_anonymous_record_decl():
                    # walk2: the anonymous member by value (bitfields through their accessors); an
                    # anonymous union fills / hashes the one variant UV_<anon_NNN> names
                    flds = REC_FIELDS.get(rs, {})
                    anons = [x for x in flds if x.startswith("anon_")]
                    if anon_i >= len(anons):
                        raise SystemExit(f"walk2: no Rust anon field #{anon_i} in {rs}")
                    rname = anons[anon_i]
                    anon_i += 1
                    nrs = strip_arr(flds[rname])
                    if f.kind == K.UNION_DECL:
                        self.union(f, f"{rs}_{rname}", cp, rp + rname + ".", cf, ch, rf, rh, depth, nrs)
                    else:
                        self.fields(f, cp, rp + rname + ".", 0, cf, ch, rf, rh, None, depth, rs=nrs)
                    continue
                if f.kind in (K.STRUCT_DECL, K.UNION_DECL) and f.is_anonymous_record_decl():
                    # C11 anonymous member: raw bytes at its offset in the parent. (A bitfield
                    # overlay in it is not compared by value: melee_types keeps bitfields in GC bit
                    # order; a function that reads such a member needs its own check.)
                    off = self.anon_offset(decl, f)
                    sz = f.type.get_size()
                    cpar = "(*p)" if cp == "p->" else ("(*" + cp[:-2] + ")" if cp.endswith("->") else cp[:-1])
                    rpar = rp[:-1]
                    j = f"j{depth}"
                    cf.append(f"for ({j} = 0; {j} < {sz}; {j}++) ((unsigned char*) &{cpar})[{off} + {j}] = (unsigned char) rnd();")
                    ch.append(f"for ({j} = 0; {j} < {sz}; {j}++) HM(((const unsigned char*) &{cpar})[{off} + {j}]);")
                    barg = rpar if rpar == "p" else "&" + rpar  # `p` is already a reference to the record
                    rf.append(f"for {j} in 0..{sz} {{ unsafe {{ *(base({barg}) as *mut u8).add({off} + {j}) = rnd() as u8; }} }}")
                    rh.append(f"for {j} in 0..{sz} {{ unsafe {{ hm(*base({barg}).add({off} + {j}) as u32); }} }}")
                continue
            n = f.spelling
            if not n:
                raise SystemExit(f"unnamed field in {decl.spelling}")
            self.cur_key = f"{decl.spelling}.{n}"
            self.parent_rs = rs
            self.one(f.type, f, cp + n, rp + rust_field(n), cf, ch, rf, rh, depth,
                     rs=strip_arr(REC_FIELDS.get(rs, {}).get(rust_field(n), "")) if self.walk2 else None)

    def union(self, d, key, cprefix, rprefix, cf, ch, rf, rh, depth, rs):
        """walk2: a union fills / hashes ONE variant by value, chosen by the global UV_<key> (the
        member index; set per operation before the state is reset). uv_only[key] lists the members
        generated (default: all)."""
        members = [m for m in d.get_children() if m.kind == K.FIELD_DECL or
                   (m.kind in (K.STRUCT_DECL, K.UNION_DECL) and m.is_anonymous_record_decl())]
        anons = [x for x in REC_FIELDS.get(rs, {}) if x.startswith("anon_")]
        names, rnames, ai = [], [], 0
        for m in members:
            if m.kind == K.FIELD_DECL:
                names.append(m.spelling)
                rnames.append(None)
            else:
                names.append(f"anon{ai}")
                rnames.append(anons[ai])
                ai += 1
        self.uv_keys[key] = names
        only = self.uv_only.get(key)
        for idx, m in enumerate(members):
            if only is not None and names[idx] not in only:
                continue
            for L in (cf, ch):
                L.append(f"if (UV_{key} == {idx}) {{")
            for L in (rf, rh):
                L.append(f"if UV_{key} == {idx} {{")
            if m.kind == K.FIELD_DECL:
                self.cur_key = f"{d.spelling}.{m.spelling}"
                nrs = strip_arr(REC_FIELDS.get(rs, {}).get(rust_field(m.spelling), ""))
                self.parent_rs = rs
                self.one(m.type, m, cprefix + m.spelling, rprefix + rust_field(m.spelling), cf, ch, rf, rh, depth + 1, rs=nrs)
            else:
                nrs = strip_arr(REC_FIELDS[rs][rnames[idx]])
                if m.kind == K.UNION_DECL:
                    self.union(m, f"{rs}_{rnames[idx]}", cprefix, rprefix + rnames[idx] + ".", cf, ch, rf, rh, depth + 1, nrs)
                else:
                    self.fields(m, cprefix, rprefix + rnames[idx] + ".", 0, cf, ch, rf, rh, None, depth + 1, rs=nrs)
            for L in (cf, ch, rf, rh):
                L.append("}")

    def one(self, t, f, cexpr, rexpr, cf, ch, rf, rh, depth, rs=None):
        c = canon(t)
        k = c.kind
        prs = self.parent_rs
        rule = self.rules.get(self.cur_key)
        if rule is not None and (k not in (T.CONSTANTARRAY,) or rule[0] == "skip"):
            kind = rule[0]
            if kind == "skip":
                return
            if kind == "uptr" and k == T.RECORD:
                _, mem, tn, nul = rule
                cf.append(f"{cexpr}.{mem} = pick_{tn}({1 if nul else 0});")
                rf.append(f"unsafe {{ {rexpr}.{mem} = pick_{tn}({1 if nul else 0}) as _; }}")
                ch.append(f"HM(PI((const void*) {cexpr}.{mem}));")
                rh.append(f"hm(pi(unsafe {{ {rexpr}.{mem} }}));")
                return
            if kind == "expr" and k == T.POINTER:
                _, ce, re_ = rule
                cf.append(f"{cexpr} = (void*) ({ce});")
                rf.append(f"{rexpr} = ({re_}) as _;")
                ch.append(f"HM(PI((const void*) {cexpr}));")
                rh.append(f"hm(pi({rexpr}));")
                return
            if kind == "fn" and k == T.POINTER:
                _, fns, nul = rule
                n_ = len(fns)
                cs = " : ".join(f"q % {n_}u == {i} ? (void*) {c}" for i, (c, _) in enumerate(fns[:-1])) + \
                    (" : " if n_ > 1 else "") + f"(void*) {fns[-1][0]}"
                cf.append(f"{{ u32 r = rnd(); u32 q = rnd(); {cexpr} = (__typeof__({cexpr})) "
                          f"({'(r & 3) == 0 ? (void*) 0 : ' if nul else ''}({cs})); (void) r; }}")
                arms = " ".join(f"{i} => core::mem::transmute::<usize, _>({r_} as usize)," for i, (_, r_) in enumerate(fns))
                rf.append(f"{{ let r = rnd(); let q = rnd() % {n_}; {rexpr} = {'if r & 3 == 0 { unsafe { core::mem::zeroed() } } else ' if nul else ''}"
                          f"{{ unsafe {{ match q {{ {arms} _ => core::mem::zeroed() }} }} }}; let _ = r; }}")
                ch.append(f"HM(FI((const void*) {cexpr}));")
                rh.append(f"hm(fi({rexpr}));")
                return
            if kind == "ptr" and k == T.POINTER:
                _, tn, nul = rule
                cf.append(f"{cexpr} = (void*) pick_{tn}({1 if nul else 0});")
                rf.append(f"{rexpr} = pick_{tn}({1 if nul else 0}) as _;")
                ch.append(f"HM(PI((const void*) {cexpr}));")
                rh.append(f"hm(pi({rexpr}));")
                return
            if kind == "range" and (k in INTS or k == T.ENUM):
                _, lo, hi = rule
                n0 = len(cf)
                n1 = len(rf)
                self.cur_key = None
                self.rules_saved = self.rules
                self.rules = {}
                try:
                    self.one(t, f, cexpr, rexpr, cf, ch, rf, rh, depth)
                finally:
                    self.rules = self.rules_saved
                for i in range(n0, len(cf)):
                    cf[i] = cf[i].replace("fv()", f"({lo} + (int) (fv() % {hi - lo + 1}))")
                for i in range(n1, len(rf)):
                    rf[i] = rf[i].replace("fv() as _", f"({lo} + (fv() % {hi - lo + 1}) as i32) as _")
                return
        if f is not None and f.is_bitfield():
            n = f.spelling
            signed = PRIM.get(k, "u32") in ("i8", "i16", "i32", "i64")
            acc = rust_field(n)
            base = rexpr[: -len(acc)] if rexpr.endswith(acc) else rexpr
            cf.append(f"{cexpr} = fv();")
            rf.append(f"{base}set_{n}(fv() as _);")
            ch.append(f"HM((u32) {cexpr});")
            if k == T.BOOL:
                rh.append(f"hm({rexpr[:-len(acc)]}{acc}() as u32);")
            else:
                rh.append(f"hm({rexpr[:-len(acc)]}{acc}() as u32);")
            return
        if k == T.CONSTANTARRAY:
            iv = f"i{depth}"
            cf.append(f"for ({iv} = 0; {iv} < {c.element_count}; {iv}++) {{")
            ch.append(f"for ({iv} = 0; {iv} < {c.element_count}; {iv}++) {{")
            rf.append(f"for {iv} in 0..{c.element_count} {{")
            rh.append(f"for {iv} in 0..{c.element_count} {{")
            self.one(c.element_type, None, f"{cexpr}[{iv}]", f"{rexpr}[{iv}]", cf, ch, rf, rh, depth + 1, rs=rs)
            for L in (cf, ch, rf, rh):
                L.append("}")
            return
        if k == T.RECORD:
            d = c.get_declaration()
            d = d.get_definition() or d
            if d.kind == K.UNION_DECL and self.walk2:
                key = f"{prs}_{f.spelling}" if f is not None else f"{prs}_{(self.cur_key or 'u').split('.')[-1]}"
                self.union(d, key, cexpr + ".", rexpr + ".", cf, ch, rf, rh, depth, rs)
                return
            if d.kind == K.UNION_DECL:
                sz = c.get_size()
                cf.append(f"for (i{depth} = 0; i{depth} < {sz}; i{depth}++) ((unsigned char*) &{cexpr})[i{depth}] = (unsigned char) rnd();")
                ch.append(f"for (i{depth} = 0; i{depth} < {sz}; i{depth}++) HM(((const unsigned char*) &{cexpr})[i{depth}]);")
                rf.append(f"for i{depth} in 0..{sz} {{ unsafe {{ *(&mut {rexpr} as *mut _ as *mut u8).add(i{depth}) = rnd() as u8; }} }}")
                rh.append(f"for i{depth} in 0..{sz} {{ unsafe {{ hm(*(&{rexpr} as *const _ as *const u8).add(i{depth}) as u32); }} }}")
                return
            self.fields(d, cexpr + ".", rexpr + ".", 0, cf, ch, rf, rh, None, depth, rs=rs)
            return
        if k == T.POINTER:
            if is_fnptr(t):
                ch.append(f"HM(FI((const void*) {cexpr}));")
                rh.append(f"hm(fi({rexpr}));")
            else:
                ch.append(f"HM(PI((const void*) {cexpr}));")
                rh.append(f"hm(pi({rexpr}));")
            return
        if k in (T.FLOAT,):
            cf.append(f"{cexpr} = rg();")
            rf.append(f"{rexpr} = rg();")
            ch.append(f"HM(B({cexpr}));")
            rh.append(f"hm({rexpr}.to_bits());")
            return
        if k in (T.DOUBLE, T.LONGDOUBLE):
            cf.append(f"{cexpr} = (double) rg();")
            rf.append(f"{rexpr} = rg() as f64;")
            ch.append(f"HM64({cexpr});")
            rh.append(f"hm64({rexpr});")
            return
        if k == T.ENUM or k in INTS:
            sz = c.get_size()
            if gc4(t):  # uintptr_t & co are 4 bytes in melee_types (GC layout), 8 natively: compare 32 bits
                sz = 4
            if k == T.BOOL:
                cf.append(f"{cexpr} = fv() & 1;")
                rf.append(f"{rexpr} = fv() & 1 != 0;")
                ch.append(f"HM((u32) {cexpr});")
                rh.append(f"hm({rexpr} as u32);")
            elif sz == 8:
                cf.append(f"{cexpr} = fv64();")
                rf.append(f"{rexpr} = fv64() as _;")
                ch.append(f"HM((u32) {cexpr}); HM((u32) ((u64) {cexpr} >> 32));")
                rh.append(f"hm({rexpr} as u32); hm((({rexpr} as u64) >> 32) as u32);")
            else:
                cf.append(f"{cexpr} = fv();")
                rf.append(f"{rexpr} = fv() as _;")
                ch.append(f"HM((u32) {cexpr});")
                rh.append(f"hm({rexpr} as u32);")
            return
        raise SystemExit(f"state walker: unsupported field type {t.spelling} ({k})")


# ---------------------------------------------------------------------------- output

def build(stem, spec, want_ext):
    import if_drv
    global pool_decls, helpers
    pool_decls, helpers = if_drv.pool_decls, if_drv.helpers
    tu = Tu(patched_source(stem, spec), spec.get("args", ()), (ROOT / spec["src"]).parent, spec.get("extra_main", ()))
    stubs, inlines, data = tu.uses()
    for n in ("__assert", "OSReport", "printf"):
        stubs.pop(n, None)
    for n in spec.get("skip_stubs", ()):
        stubs.pop(n, None)
    for n in list(data):
        if n in tu.defs or n in tu.main_vars or n in spec.get("skip_data", ()):
            data.pop(n)
    # variables defined in the TU file itself are the port's own statics
    c_out = [f'/* generated by gen/mtu/gen_tu.py for {spec["src"]}: do not edit */']
    r_out = [f"// generated by rust/melee_world/gen/mtu/gen_tu.py for {spec['src']}: do not edit",
             "#![allow(unused_variables, unused_parens, non_snake_case, unused_unsafe, static_mut_refs, clippy::all)]",
             "use crate::mtu_harness::*;", "use super::m;", "use melee_types::*;", "use super::m::*;", "use core::ffi::c_void;", "use core::ptr::null_mut;", ""]
    names = sorted(stubs)
    sid = {n: 1000 + i for i, n in enumerate(names)}
    ext_fns, ext_cb, ext_data = [], [], []
    warn = []

    # data
    for n in sorted(data):
        v = data[n]
        t = v.type
        c = canon(t)
        csize = spec.get("data_size", {}).get(n)
        if c.kind == T.INCOMPLETEARRAY:
            cs = csize or 1
            c_out.append(f"{canon(c.element_type).spelling if False else c.element_type.spelling} {n}[{cs}];")
            ext_data.append(f"    pub static mut {n}: [{rust_ty(c.element_type)}; {cs}] = unsafe {{ core::mem::zeroed() }};")
        elif c.kind == T.CONSTANTARRAY:
            c_out.append(f"{c.element_type.spelling} {n}[{c.element_count}];")
            ext_data.append(f"    pub static mut {n}: {rust_ty(t)} = unsafe {{ core::mem::zeroed() }};")
        else:
            c_out.append(f"{t.spelling} {n};")
            ext_data.append(f"    pub static mut {n}: {rust_ty(t)} = unsafe {{ core::mem::zeroed() }};")

    # stubs
    ov = {**spec.get("common_ret", {}), **spec.get("ret", {})}
    eff = spec.get("effects", {})
    for n in names:
        decl, called, addr = stubs[n]
        ft = canon(decl.type)
        ptys = list(ft.argument_types())
        rt = ft.get_result()
        rcls = arg_class(rt)
        va = ft.is_function_variadic()
        realcb = spec.get("real_cb", {})  # address-taken stubs whose address the port takes from a real Rust port
        if va:
            cs = spec.get("custom_stubs", {}).get(n)
            if cs:  # a variadic callee the spec models by hand (fixed-arity on the Rust side)
                c_out.append(cs["c"].replace("@ID@", str(sid[n])))
                ext_fns.append("    " + cs["ext"])
                r_out.append(("@@HOOK@@", f"        m::ext::hook::{n} = Some({cs['hook'].replace('@ID@', str(sid[n]))});"))
            continue  # other variadics (OSReport): defined in mtu_common.h
        params = ", ".join(f"__typeof__({p.spelling}) a{i}" for i, p in enumerate(ptys)) or "void"
        ret_t = "void" if rcls == "void" else f"__typeof__({rt.spelling})"
        f32a = spec.get("fn32_args", {}).get(n, [])
        body_c = [f"T({sid[n]});"] + [(f"T(FI32((u32) a{i}));" if i in f32a else c_trace(arg_class(p), f"a{i}"))
                                      for i, p in enumerate(ptys)]
        rparams = ", ".join(f"a{i}: {rust_ty(p)}" for i, p in enumerate(ptys))
        rret = "" if rcls == "void" else f" -> {rust_ty(rt)}"
        body_r = [f"t({sid[n]});"] + [(f"t(fi32(a{i} as u32));" if i in f32a else r_trace(arg_class(p), f"a{i}"))
                                      for i, p in enumerate(ptys)]
        for di in spec.get("deref", {}).get(n, ()):
            body_c.append(f"T(RH((const void*) a{di}, sizeof *a{di}));")
            body_r.append(f"t(rh(unsafe {{ &*a{di} }}));")
        if n in eff:
            body_c.append(eff[n][0])
            body_r.append(eff[n][1])
        if rcls != "void":
            if n in ov:
                rc, rr = ov[n]
            else:
                rc, rr = c_ret(rcls), r_ret(rcls, rt)
                if rcls in ("ptr", "fp", "str") and n not in spec.get("null_ret_ok", ()):
                    warn.append(f"{n}: returns a pointer, NULL by default")
                if rcls == "rec":
                    raise SystemExit(f"{n}: returns a struct by value; give a ret override")
            body_c.append(f"return ({ret_t}) ({rc});")
            body_r.append(f"({rr}) as _" if n in ov and not rr.rstrip().endswith("as _") else rr)
        c_out.append(f"{ret_t} ({n})({params}) {{ " + " ".join(body_c) + " }")  # parenthesised: a function-like macro of the TU must not expand here
        argnames = ", ".join(f"a{i}" for i in range(len(ptys)))
        argl = ", ".join(f"a{i}" for i in range(len(ptys)))
        r_out_hook = f"        m::ext::hook::{n} = Some(|{argl}| {{ " + " ".join(body_r) + " });"
        r_out.append(("@@HOOK@@", r_out_hook))
        dflt = spec.get("defaults", {}).get(n)
        ext_fns.append(f"    fn {n}({rparams}){rret}" + (f" = {dflt}" if dflt else "") + ";")
        if addr and n not in realcb:
            ext_cb.append((n, rparams, rret, argnames))
    # fill / hash
    walker = Walk(tu, spec.get("rules"), spec)
    crate = spec.get("crate", "melee_world")
    local_types = []
    fwd = []
    all_types = list(spec.get("types", ()))
    for tn in list(spec.get("pools", {})) + [x["ty"] for x in spec.get("statics", ()) if x.get("ty")]:
        if tn not in all_types:
            all_types.append(tn)
    spec = dict(spec, types=all_types)
    for tn in spec.get("types", ()):
        cn_ = ("struct " + tn) if tn in tu.tagged else tn
        fwd.append(f"static void fill_{tn}({cn_}* p);")
        fwd.append(f"static void hash_{tn}(const {cn_}* p);")
    pool_c, pool_r = pool_decls(tu, spec)
    c_out[1:1] = pool_c + fwd
    for tn in spec.get("types", ()):
        cdef, rdef = walker.gen(tn)
        c_out.append(cdef)
        r_out.append(("@@RAW@@", rdef))
        if tu.in_main(walker.rec_of(tn)):
            local_types.append(tn)
    for uk in walker.uv_keys:   # walk2 union selectors (declared ahead of the fill / hash code)
        c_out.insert(1, f"static int UV_{uk};")
        r_out.append(("@@RAW@@", f"pub static mut UV_{uk}: i32 = 0;"))
    spec = dict(spec, _uv_keys=walker.uv_keys)
    if local_types:
        # records the .c defines shadow a same-named opaque melee_types record
        r_out.insert(5, "use super::m::{" + ", ".join(local_types) + "};")

    # the function table: stubs address-taken + the TU's own functions, sorted by name
    fpn = [n for n in names if stubs[n][2] and not stubs[n][0].type.is_function_variadic()]
    own = sorted(n for n in tu.defs if not tu.defs[n].storage_class == ci.StorageClass.NONE or True)
    c_out.append("static void freg_all(void)\n{")
    for n in fpn:
        c_out.append(f"    freg((const void*) {n});")
    for n in own:
        c_out.append(f"    freg((const void*) {n});")
    c_out.append("}")
    c_out.extend(f"// stub {n} = {sid[n]}" for n in names)
    r_tail = []
    helpers(tu, spec, c_out, r_tail, data)

    cfile = HERE / "tu" / f"{stem}_gen.h"
    cfile.write_text("\n".join(c_out) + "\n", encoding="utf-8", newline="\n")

    rl = r_out[:9 if False else 8]
    hooks = [x[1] for x in r_out if isinstance(x, tuple) and x[0] == "@@HOOK@@"]
    raws = [x[1] for x in r_out if isinstance(x, tuple) and x[0] == "@@RAW@@"]
    head = [x for x in r_out if isinstance(x, str)]
    rtxt = "\n".join(head) + "\n\n"
    rtxt += "pub fn install() {\n    use m::ext::hook::*;\n    unsafe {\n" + "\n".join(h.replace("m::ext::hook::", "") for h in hooks) + "\n    }\n}\n\n"
    rtxt += "pub fn freg_all() {\n"
    for n in fpn:
        rtxt += f"    freg({spec.get('real_cb', {}).get(n, 'm::ext::cb::' + n)} as *const () as usize);\n"
    for n in own:
        rtxt += f"    freg(m::{n} as *const () as usize);\n"
    rtxt += "}\n\n" + "\n".join(raws) + "\n" + "\n".join(pool_r) + "\n" + "\n".join(r_tail) + "\n"
    rfile = ROOT / "rust" / crate / "tests/mtu" / f"{stem}_gen.rs"
    rfile.parent.mkdir(parents=True, exist_ok=True)
    rfile.write_text(rtxt, encoding="utf-8", newline="\n")

    ext = ["// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)",
           "pub mod ext {",
           "    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]",
           "    use super::*;",
           "    use core::ffi::c_void;",
           "    macro_rules! ext_none {",
           "        ($name:ident, ($($a:ident),*)) => { panic!(concat!(\"TODO: \", stringify!($name), \" is not ported\")) };",
           "        ($name:ident, ($($a:ident),*), $def:path) => { $def($($a),*) };",
           "    }",
           "    macro_rules! ext_fns {",
           "        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:path)?;)*) => {",
           "            pub mod hook {",
           "                use super::*;",
           "                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*",
           "            }",
           "            $( pub fn $name($($a: $t),*) $(-> $r)? {",
           "                match unsafe { hook::$name } {",
           "                    Some(f) => f($($a),*),",
           "                    None => ext_none!($name, ($($a),*) $(, $def)?),",
           "                }",
           "            } )*",
           "        };",
           "    }",
           "    ext_fns! {"] + ext_fns + ["    }"]
    ext += ["    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.",
            "    pub mod cb {", "        use super::*;"]
    for n, rparams, rret, argn in ext_cb:
        ext.append(f"        pub unsafe extern \"C\" fn {n}({rparams}){rret} {{ super::{n}({argn}) }}")
    ext.append("    }")
    ext += ["    // Data defined in other TUs (the tests set them)."] + ext_data + ["}", "// END GENERATED EXT"]
    ext_txt = "\n".join(ext) + "\n"
    (ROOT / "rust/target").mkdir(exist_ok=True)
    (HERE / "tu" / f"{stem}_ext.rs.txt").write_text(ext_txt, encoding="utf-8", newline="\n")
    port = ROOT / "rust" / crate / "src/tu" / f"{stem}.rs"
    if port.exists():
        txt = port.read_text(encoding="utf-8")
        if "@@EXT@@" in txt:
            txt = txt.replace("@@EXT@@", ext_txt.rstrip("\n"))
        elif "// BEGIN GENERATED EXT" in txt:
            a = txt.index("// BEGIN GENERATED EXT")
            b = txt.index("// END GENERATED EXT") + len("// END GENERATED EXT")
            txt = txt[:a] + ext_txt.rstrip("\n") + txt[b:]
        port.write_text(txt, encoding="utf-8", newline="\n")
    if spec.get("ops"):
        if_drv.emit_driver(stem, spec, tu)
    print(f"gen_tu: {stem}: {len(names)} stubs, {len(data)} data, {len(inlines)} header inlines")
    if inlines:
        print("  header inlines used (port them in the file): " + ", ".join(sorted(inlines)))
    for w in warn:
        print("  WARN " + w)
    return ext_txt


def main():
    from specs_if import SPECS
    stem = sys.argv[1]
    build(stem, SPECS[stem], "--ext" in sys.argv)


if __name__ == "__main__":
    main()
