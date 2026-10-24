#!/usr/bin/env python3
"""TU state oracle generator.

  python rust/melee_modes/oracle_tu/tuoracle.py <stem> [--build] [--rust-only]    (worktree root)

For one game TU (config in cfgs.py) this writes
  * rust/target/tu-oracle/<stem>/wrap.c   the TU itself #included, plus a harness: mocks for every
    external function, seeded fill/hash walkers for every state object, one runner per function
  * rust/melee_modes/tests/tu_<stem>.rs   the same harness for the Rust port (generated, committed)
  * rust/melee_modes/tests/data/tu_<stem>.txt   with --build: the C results (committed)
and, with --build, compiles wrap.c exactly like the shipping build (contraction on, gwtool float
lowering) and runs it.

State oracle: each case seeds a stream, fills every state object (the TU's own globals, the
external globals it reads, the objects its external functions return a pointer to) field by field
with the same draws on both sides, calls the function with arguments drawn from the stream, and
compares (return value, hash of every external call and its arguments, hash of all state).
External functions are mocked identically on both sides; pointer-valued fields are NULL unless the
config wires them. Bitfields are read and written by member name (C) / accessor (Rust), so the
GameCube bit order of melee_types and the native order of the C compile do not matter.
"""
import json, os, re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
sys.path.insert(0, str(HERE))
os.chdir(ROOT)
import gen_types as g  # noqa: E402
from scan import parse_tu, NAT  # noqa: E402

ci, K, T = g.ci, g.K, g.T
OUT_ROOT = ROOT / "rust/target/tu-oracle"


# ---------------------------------------------------------------- naming (Rust types)
class Names(g.Gen):
    """gen_types.Gen over the all_types.c parse (the melee_types names), plus the TU-local records."""

    def local(self, tu, anon_vars=None):
        for c in tu.cursor.get_children():
            if anon_vars and c.kind == K.VAR_DECL and c.spelling in anon_vars:
                self._anon_var(c, anon_vars[c.spelling])
            if c.kind == K.TYPEDEF_DECL:
                u = c.underlying_typedef_type
                while u.kind == T.ELABORATED:
                    u = u.get_named_type()
                if u.kind in (T.RECORD, T.ENUM):
                    d = u.get_declaration()
                    if (d.is_anonymous() or "unnamed" in d.spelling or not d.spelling) and loc(d) not in self.rec_name:
                        self.rec_name[loc(d)] = c.spelling
            elif c.kind in (K.STRUCT_DECL, K.UNION_DECL, K.ENUM_DECL) and c.spelling and "unnamed" not in c.spelling:
                if g.loc(c) not in self.rec_name:
                    self.rec_name[g.loc(c)] = c.spelling


loc = g.loc


def _anon_var(self, c, spec):
    """cfg anon_vars: {var: "RustName"} or {var: ("RustName", {field: "FieldRecName"})}: name the anonymous
    record type of a variable (and of its anonymous-record fields) after the port's own record."""
    name, fields = (spec, {}) if isinstance(spec, str) else spec
    t = c.type.get_canonical()
    while t.kind == T.CONSTANTARRAY:
        t = t.element_type.get_canonical()
    d = t.get_declaration()
    self.rec_name[loc(d)] = name
    for f in d.get_children():
        if f.kind == K.FIELD_DECL and f.spelling in fields:
            ft = f.type.get_canonical()
            while ft.kind == T.CONSTANTARRAY:
                ft = ft.element_type.get_canonical()
            self.rec_name[loc(ft.get_declaration())] = fields[f.spelling]


Names._anon_var = _anon_var


def load_names(tu, anon_vars=None):
    ppc = g.parse(g.PPC)
    n = Names(ppc, {})
    n.collect()
    n.local(tu, anon_vars)
    return n


# ---------------------------------------------------------------- type classification
def strip(t):
    while t.kind == T.ELABORATED:
        t = t.get_named_type()
    return t


def classify(t):
    # uintptr_t/intptr_t are 4 bytes in the melee_types (GameCube) layout, 8 on the native C side:
    # draw and hash them as 32-bit on both
    tt = t
    while tt.kind in (T.ELABORATED, T.TYPEDEF):
        if tt.kind == T.TYPEDEF and tt.spelling in ("uintptr_t", "intptr_t"):
            return ("int", 4, tt.spelling == "intptr_t", True)
        tt = tt.get_named_type() if tt.kind == T.ELABORATED else tt.get_declaration().underlying_typedef_type
    c = t.get_canonical()
    k = c.kind
    if k in (T.CHAR_S, T.SCHAR, T.SHORT, T.INT, T.LONG, T.LONGLONG):
        return ("int", c.get_size(), True)
    if k in (T.CHAR_U, T.UCHAR, T.USHORT, T.UINT, T.ULONG, T.ULONGLONG):
        return ("int", c.get_size(), False)
    if k == T.BOOL:
        return ("bool",)
    if k == T.ENUM:
        ut = c.get_declaration().enum_type.get_canonical()
        return ("int", c.get_size(), ut.kind in (T.CHAR_S, T.SCHAR, T.SHORT, T.INT, T.LONG, T.LONGLONG))
    if k == T.FLOAT:
        return ("f32",)
    if k in (T.DOUBLE, T.LONGDOUBLE):
        return ("f64",)
    if k == T.VOID:
        return ("void",)
    if k == T.POINTER:
        p = c.get_pointee().get_canonical()
        if p.kind in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO):
            return ("fnptr", p)
        return ("ptr", c.get_pointee())
    if k == T.CONSTANTARRAY:
        return ("arr", c.element_type, c.element_count)
    if k in (T.INCOMPLETEARRAY,):
        return ("arr", c.element_type, 0)
    if k == T.RECORD:
        d = c.get_declaration()
        d = d.get_definition() or d
        return ("rec", d, d.kind == K.UNION_DECL)
    raise ValueError(f"unclassified type {t.spelling} ({k})")


def is_cstr_ptr(pointee):
    c = pointee.get_canonical()
    return c.kind in (T.CHAR_S, T.SCHAR, T.CHAR_U, T.UCHAR) and c.get_size() == 1 and pointee.is_const_qualified()


def rp_par_of(fr):
    return fr.rpartition(".")[0]


class Gen2:
    def __init__(self, names, cfg, tu):
        self.n = names
        self.cfg = cfg
        self.tu = tu
        self.cbs = {}  # signature key -> (index, fnproto type)

    def rty(self, t):
        return self.n.rty(t)

    def bit_union(self, d):
        """A union of one scalar byte and an anonymous struct of bitfields (UnkFlagStruct): the bit order
        differs between PPC (MSB first) and the native C, so it is walked bit by bit, by name."""
        kids = list(d.get_children())
        anon = [c for c in kids if c.kind == K.STRUCT_DECL and c.is_anonymous_record_decl()]
        if len(anon) != 1:
            return False
        fl = [c for c in anon[0].get_children() if c.kind == K.FIELD_DECL]
        return bool(fl) and all(c.is_bitfield() for c in fl)

    # ---- field iteration
    def members(self, d, cp, rp):
        """Yield (field cursor, C path, Rust path) for every named field of record d, flattening
        C11 anonymous members (the Rust record names them anon_<offset>). Of an anonymous union only
        the first member is walked (the others overlay it)."""
        kids = list(d.get_children())
        pd = self.n.records.get(loc(d))
        pkids = list(pd.get_children()) if pd is not None else None
        for idx, ch in enumerate(kids):
            if ch.kind == K.FIELD_DECL:
                yield ch, f"{cp}.{ch.spelling}" if ch.spelling else None, f"{rp}.{g.ident(ch.spelling)}" if ch.spelling else None
            elif ch.kind in (K.STRUCT_DECL, K.UNION_DECL) and ch.is_anonymous_record_decl():
                if pkids is None:
                    raise NotImplementedError(f"anonymous member in {d.spelling}: no GC record")
                pa = pkids[idx]
                off = self.n.anon_offset(pd, pa)
                nm = f"anon_{off:x}"
                if ch.kind == K.UNION_DECL:
                    # walk only the first member of the union
                    ukids = list(ch.get_children())
                    first = ukids[0]
                    if first.kind == K.FIELD_DECL:
                        yield first, f"{cp}.{first.spelling}", f"{rp}.{nm}.{g.ident(first.spelling)}"
                    else:
                        upd = self.n.records.get(loc(ch)) or pa
                        # recurse into the anonymous struct inside the union
                        sub = first
                        pkid2 = list(pa.get_children())
                        off2 = self.n.anon_offset(pa, pkid2[0])
                        yield from self.members(sub, cp, f"{rp}.{nm}.anon_{off2:x}")
                    continue
                yield from self.members(ch, cp, f"{rp}.{nm}")

    def rfield(self, f):
        return g.ident(f.spelling)

    def fieldrange(self, d, f):
        key = f"{self.n.rec_name.get(loc(d)) or d.spelling}.{f.spelling}"
        return self.cfg.get("fieldrange", {}).get(key)

    # ---- fill
    def fill(self, t, cp, rp, dep, stream="tuo_st", rstream="rt::ST"):
        """-> (c_lines, r_lines) filling the lvalue (cp / rp) of type t from the stream."""
        k = classify(t)
        pad = "    " * dep
        if k[0] == "int":
            if len(k) > 3:
                return ([f"{cp} = (__typeof__({cp})) (unsigned) tuo_draw_int(&{stream});"],
                        [f"{rp} = (rt::draw_int(&mut {rstream}) as u32) as _;"])  # pointer-width field: 32-bit draw on both sides
            return ([f"{cp} = (__typeof__({cp})) tuo_draw_int(&{stream});"],
                    [f"{rp} = rt::draw_int(&mut {rstream}) as _;"])
        if k[0] == "bool":
            return ([f"{cp} = (tuo_draw_int(&{stream}) & 1) != 0;"], [f"{rp} = (rt::draw_int(&mut {rstream}) & 1) != 0;"])
        if k[0] == "f32":
            return ([f"{cp} = tuo_draw_f32(&{stream});"], [f"{rp} = rt::draw_f32(&mut {rstream});"])
        if k[0] == "f64":
            return ([f"{cp} = tuo_draw_f64(&{stream});"], [f"{rp} = rt::draw_f64(&mut {rstream});"])
        if k[0] == "ptr":
            return ([f"{cp} = 0;"], [f"{rp} = 0usize as _;"])
        if k[0] == "fnptr":
            cb = self.cb(k[1])
            return ([f"{{ tuo_u64 d{dep} = tuo_next(&{stream}); {cp} = (d{dep} & 1) ? (__typeof__({cp})) tuo_cb{cb} : 0; }}"],
                    [f"{{ let d = rt::next(&mut {rstream}); {rp} = if d & 1 != 0 {{ Some(cb{cb}) }} else {{ None }}; }}"])
        if k[0] == "arr":
            i = f"i{dep}"
            cl, rl = self.fill(k[1], f"{cp}[{i}]", f"{rp}[{i}]", dep + 1, stream, rstream)
            return ([f"{{ int {i}; for ({i} = 0; {i} < {k[2]}; {i}++) {{"] + ["  " + x for x in cl] + ["} }"],
                    [f"for {i} in 0..{k[2]} {{"] + ["    " + x for x in rl] + ["}"])
        if k[0] == "rec":
            d = k[1]
            if k[2] and self.bit_union(d):
                pass  # a byte of flag bits: filled bit by bit, by name
            elif k[2]:  # union: raw bytes
                sz = t.get_canonical().get_size()
                i = f"i{dep}"
                return ([f"{{ int {i}; for ({i} = 0; {i} < {sz}; {i}++) ((unsigned char*)&{cp})[{i}] = (unsigned char) tuo_draw_int(&{stream}); }}"],
                        [f"{{ assert_eq!(core::mem::size_of_val(&{rp}), {sz}); for {i} in 0..{sz} {{ *(&mut {rp} as *mut _ as *mut u8).add({i}) = rt::draw_int(&mut {rstream}) as u8; }} }}"])
            cl, rl = [], []
            for f, fc, fr in self.members(d, cp, rp):
                if not f.spelling:
                    continue  # unnamed bitfield padding
                if k[2] and not f.is_bitfield():
                    continue
                if f.is_bitfield():
                    w = f.get_bitfield_width()
                    mask = (1 << w) - 1
                    cty = f.type.spelling
                    fk = classify(f.type)
                    if fk[0] == "bool":
                        rv = f"(rt::draw_int(&mut {rstream}) & 1) != 0"
                    else:
                        rv = f"(rt::draw_int(&mut {rstream}) & 0x{mask:X}) as _"
                    rng = self.fieldrange(d, f)
                    if rng and fk[0] != "bool":
                        lo, hi = map(lambda x: int(x, 0), rng.split(".."))
                        cl.append(f"{fc} = ({cty}) tuo_draw_range(&{stream}, {lo}, {hi});")
                        rl.append(f"{rp_par_of(fr)}.set_{f.spelling}(rt::draw_range(&mut {rstream}, {lo}, {hi}) as _);")
                        continue
                    cl.append(f"{fc} = ({cty}) (tuo_draw_int(&{stream}) & 0x{mask:X});")
                    rp_par, _, rp_name = fr.rpartition(".")
                    rl.append(f"{rp_par}.set_{f.spelling}({rv});")
                else:
                    rng = self.fieldrange(d, f)
                    fk = classify(f.type)
                    if fk[0] == "fnptr" and f"{self.n.rec_name.get(loc(d)) or d.spelling}.{f.spelling}" in self.cfg.get("fnptr_nonnull", ()):
                        cbi = self.cb(fk[1])
                        cl.append(f"{{ tuo_u64 d{dep} = tuo_next(&{stream}); (void) d{dep}; {fc} = (__typeof__({fc})) tuo_cb{cbi}; }}")
                        rl.append(f"{{ let _d = rt::next(&mut {rstream}); {fr} = Some(cb{cbi}); }}")
                        continue
                    if rng and fk[0] in ("int", "bool"):
                        lo, hi = map(lambda x: int(x, 0), rng.split(".."))
                        cl.append(f"{fc} = (__typeof__({fc})) tuo_draw_range(&{stream}, {lo}, {hi});")
                        rl.append(f"{fr} = rt::draw_range(&mut {rstream}, {lo}, {hi}) as _;")
                        continue
                    a, b = self.fill(f.type, fc, fr, dep, stream, rstream)
                    cl += a
                    rl += b
            return cl, rl
        raise ValueError(k)

    # ---- hash (into tuo_h / rt::H)
    def hash(self, t, cp, rp, dep, ptr_is_cstr=False):
        k = classify(t)
        if k[0] == "int":
            if len(k) > 3:
                return ([f"tuo_h_u((tuo_u64)(unsigned){cp});"], [f"rt::h_u(({rp} as u32) as u64);"])
            if k[2]:
                return ([f"tuo_h_i((tuo_i64){cp});"], [f"rt::h_i({rp} as i64);"])
            return ([f"tuo_h_u((tuo_u64){cp});"], [f"rt::h_u({rp} as u64);"])
        if k[0] == "bool":
            return ([f"tuo_h_u({cp} != 0);"], [f"rt::h_u({rp} as u64);"])
        if k[0] == "f32":
            return ([f"tuo_h_f32({cp});"], [f"rt::h_f32({rp});"])
        if k[0] == "f64":
            return ([f"tuo_h_f64({cp});"], [f"rt::h_f64({rp});"])
        if k[0] == "ptr":
            if is_cstr_ptr(k[1]):
                return ([f"tuo_h_p({cp}); tuo_h_cstr((const char*){cp});"],
                        [f"rt::h_p({rp} as *const u8); rt::h_cstr({rp} as *const u8);"])
            return ([f"tuo_h_p({cp});"], [f"rt::h_p({rp} as *const u8);"])
        if k[0] == "fnptr":
            return ([f"tuo_h_fn((const void*){cp});"], [f"rt::h_fn({rp}.map_or(0usize, |f| f as usize));"])
        if k[0] == "arr":
            i = f"i{dep}"
            cl, rl = self.hash(k[1], f"{cp}[{i}]", f"{rp}[{i}]", dep + 1)
            return ([f"{{ int {i}; for ({i} = 0; {i} < {k[2]}; {i}++) {{"] + ["  " + x for x in cl] + ["} }"],
                    [f"for {i} in 0..{k[2]} {{"] + ["    " + x for x in rl] + ["}"])
        if k[0] == "rec":
            d = k[1]
            if k[2] and not self.bit_union(d):
                sz = t.get_canonical().get_size()
                flds = [f for f in d.get_children() if f.kind == K.FIELD_DECL]
                if sz == 8 and flds and all(classify(f.type)[0] == "ptr" for f in flds):
                    # a union of pointers: hash the pointer's class (a wired pointer is an address)
                    return ([f"tuo_h_p(*(void* const*)&{cp});"],
                            [f"rt::h_p(*(&{rp} as *const _ as *const *const u8));"])
                return ([f"tuo_h_bytes(&{cp}, {sz});"],
                        [f"rt::h_bytes(&{rp} as *const _ as *const u8, {sz});"])
            cl, rl = [], []
            for f, fc, fr in self.members(d, cp, rp):
                if not f.spelling:
                    continue
                if k[2] and not f.is_bitfield():
                    continue
                if f.is_bitfield():
                    fk = classify(f.type)
                    if fk[0] == "bool":
                        cl.append(f"tuo_h_u({fc} != 0);")
                        rl.append(f"rt::h_u({fr}() as u64);")
                    elif fk[2]:
                        cl.append(f"tuo_h_i((tuo_i64){fc});")
                        rl.append(f"rt::h_i({fr}() as i64);")
                    else:
                        cl.append(f"tuo_h_u((tuo_u64){fc});")
                        rl.append(f"rt::h_u({fr}() as u64);")
                else:
                    a, b = self.hash(f.type, fc, fr, dep)
                    cl += a
                    rl += b
            return cl, rl
        raise ValueError(k)

    # ---- callbacks for fn-pointer fields
    def sigkey(self, p):
        return p.spelling

    def cb(self, p):
        key = self.sigkey(p)
        if key not in self.cbs:
            self.cbs[key] = (len(self.cbs), p)
        return self.cbs[key][0]


# ---------------------------------------------------------------- the generator for one TU
class TuGen:
    def __init__(self, stem, cfg):
        self.stem = stem
        self.cfg = cfg
        self.src = cfg["src"]
        self.mod = cfg["module"]
        txt = (ROOT / self.src).read_text(encoding="utf-8")
        for old, new in cfg.get("patch", []):
            assert old in txt, f"patch text not found: {old[:50]}"
            txt = txt.replace(old, new)
        pd = OUT_ROOT / stem
        pd.mkdir(parents=True, exist_ok=True)
        self.patched = pd / "tu_src.c"
        self.hdrs = []
        for hrel, edits in cfg.get("hdrpatch", []):
            ht = (ROOT / hrel).read_text(encoding="utf-8")
            for old, new in edits:
                assert old in ht, f"header patch text not found in {hrel}: {old[:50]}"
                ht = ht.replace(old, new)
            hp = pd / ("hp_" + Path(hrel).name)
            hp.write_text(ht, encoding="utf-8", newline=chr(10))
            self.hdrs.append(hp)
        pre = "".join(f'#include "{h.as_posix()}"' + chr(10) for h in self.hdrs)
        txt = pre + txt
        self.patched.write_text(txt, encoding="utf-8", newline=chr(10))
        self.tu = parse_tu(self.patched, ["-I" + str((ROOT / self.src).parent)])
        self.names = load_names(self.tu, cfg.get("anon_vars"))
        self.g2 = Gen2(self.names, cfg, self.tu)
        self.main = self.patched.resolve()
        self.scan()

    # ---- scan
    def in_tu_data(self, c):
        f = c.location.file
        if not f:
            return False
        p = Path(f.name).resolve()
        return p == self.main or p.name.endswith(".static.h") or p.suffix == ".inc"  # (.inc: text the TU includes)

    def scan(self):
        self.funcs = {}  # defined in main file
        self.fdefs = {}
        self.vars = {}  # TU-owned variables
        for c in self.tu.cursor.get_children():
            if c.kind == K.FUNCTION_DECL and c.is_definition():
                f = c.location.file
                if f and (Path(f.name).resolve() == self.main or Path(f.name).suffix == ".inc"):
                    self.funcs[c.spelling] = c
                self.fdefs[c.spelling] = c
            elif c.kind == K.VAR_DECL and self.in_tu_data(c) and c.storage_class != ci.StorageClass.EXTERN:
                self.vars[c.spelling] = c
        # references from main-file functions (and, transitively, header inlines they use)
        self.refs_f, self.refs_v = {}, {}
        seen = set()

        def walk(c):
            for ch in c.get_children():
                if ch.kind == K.DECL_REF_EXPR and ch.referenced is not None:
                    r = ch.referenced
                    if r.kind == K.FUNCTION_DECL:
                        if r.spelling not in self.funcs and r.spelling not in seen:
                            self.refs_f.setdefault(r.spelling, r)
                            d = self.fdefs.get(r.spelling)
                            if d is not None:
                                seen.add(r.spelling)
                                walk(d)
                    elif r.kind == K.VAR_DECL and r.semantic_parent.kind == K.TRANSLATION_UNIT:
                        self.refs_v.setdefault(r.spelling, r)
                walk(ch)
        for f in self.funcs.values():
            walk(f)
        for v in self.vars.values():
            walk(v)
        self.ext_funcs = {n: r for n, r in self.refs_f.items() if n not in self.fdefs and not n.startswith("__builtin")}
        self.ext_vars = {n: r for n, r in self.refs_v.items() if n not in self.vars
                         and (not self.in_tu_data(r) or r.storage_class == ci.StorageClass.EXTERN)
                         and r.location.file and "forward.h" not in r.location.file.name}
        # drop header-level constants (static const in headers) that are not data objects
        self.ext_vars = {n: r for n, r in self.ext_vars.items() if r.storage_class != ci.StorageClass.STATIC}

    def has_init(self, c):
        toks = [t.spelling for t in c.get_tokens()]
        return "=" in toks

    # ---- the pieces
    def build(self):
        cfg = self.cfg
        self.skip = cfg.get("skip", {})
        self.fcfg = cfg.get("funcs", {})
        names = [n for n in self.funcs if n not in self.skip and (not self.fcfg or n in self.fcfg or cfg.get("all", True))]
        self.test_funcs = sorted(n for n in names if self.fcfg.get(n, {}).get("test", True))
        # a function whose argument types the generator cannot draw (Mtx, ...) is reported and left out
        ok_funcs = []
        for n in self.test_funcs:
            try:
                for i, (a_, s_) in enumerate(self.arg_specs(self.funcs[n])):
                    self.arg_decl_c(i, a_, s_)
                self.runner_c(0, n)
                self.runner_rust(0, n)
                ok_funcs.append(n)
            except ValueError as e:
                print(f"tuoracle: skip {n}: {e}")
        self.test_funcs = ok_funcs
        self.objs = []  # (name, ctype_expr_t, rust_place, kind) built below
        self.build_objects()
        self.build_ext()
        c = self.emit_c()
        r = self.emit_rust()
        return c, r

    # object descriptors: dict(name, t (clang type), c_lv, r_lv, fill, c_size, init)
    def build_objects(self):
        cfg = self.cfg
        self.objs = []
        # 1. TU-owned variables: tables (initialised) hashed only; others filled + hashed
        for n in sorted(self.vars):
            v = self.vars[n]
            if n in cfg.get("ignore_vars", ()):
                continue
            self.objs.append(dict(name=n, t=v.type, c=n, r=f"(*core::ptr::addr_of_mut!(m::{g.ident(n)}))",
                                  r_ptr=f"core::ptr::addr_of_mut!(m::{g.ident(n)})",
                                  raw=cfg.get("raw", {}).get(n), fill=not self.has_init(v), own=True))
        # 2. external globals
        for n in sorted(self.ext_vars):
            v = self.ext_vars[n]
            t = v.type
            size = cfg.get("extsize", {}).get(n)
            if t.get_canonical().kind == T.INCOMPLETEARRAY:
                if size is None:
                    raise SystemExit(f"extern array {n} has no size: add cfg['extsize']['{n}']")
            const = t.is_const_qualified()
            rn = cfg.get("rust_vars", {}).get(n, n)
            self.objs.append(dict(name=n, t=t, c=n, r=f"(*core::ptr::addr_of_mut!(m::ext::{g.ident(rn)}))",
                                  r_ptr=f"core::ptr::addr_of_mut!(m::ext::{g.ident(rn)})",
                                  raw=cfg.get("raw", {}).get(n),
                                  fill=not const, own=False, ext=True, size=size, const=const))
        # 3. mock-return objects (filled in build_ext)

    def snap_objs(self):
        """Initialised TU variables (tables) the functions may write: restored to their initial bytes at the start of every case."""
        return [o for o in self.objs if o.get("own") and not o["fill"] and not o.get("bytes") and not o.get("const")
                and o["t"] is not None and not o["t"].is_const_qualified()]

    def mock_obj_for(self, fn):
        """Return-object spec for an external function returning a pointer, or None."""
        r = fn.result_type
        k = classify(r)
        if k[0] != "ptr":
            return None
        alias = self.cfg.get("mockobj", {}).get(fn.spelling, fn.spelling)
        return alias, k[1]

    def build_ext(self):
        self.ext = []
        mobj = {}
        for n in sorted(self.ext_funcs):
            fn = self.ext_funcs[n]
            if n in self.cfg.get("noext", ()):
                continue
            spec = dict(name=n, fn=fn, id=len(self.ext))
            self.ext.append(spec)
            mo = self.mock_obj_for(fn)
            if mo:
                alias, pt = mo
                spec["obj"] = alias
                if alias not in mobj:
                    mobj[alias] = pt
        for alias in sorted(mobj):
            pt = mobj[alias]
            ck = pt.get_canonical()
            if ck.kind == T.VOID or ck.get_size() <= 0:
                size = self.cfg.get("mocksize", {}).get(alias, 256)
                self.objs.append(dict(name=f"mo_{alias}", t=None, c=f"tuo_mo_{alias}", r=f"MO_{alias}", fill=True,
                                      own=False, mock=True, bytes=size))
            else:
                self.objs.append(dict(name=f"mo_{alias}", t=pt, c=f"tuo_mo_{alias}", r=f"(*core::ptr::addr_of_mut!(MO_{alias}))",
                                      r_ptr=f"core::ptr::addr_of_mut!(MO_{alias})", fill=True, own=False, mock=True, pt=pt))
        # cfg extra_objs {alias: "TypeName"}: further typed state objects (wired together by cfg fixups)
        for alias, tname in sorted(self.cfg.get("extra_objs", {}).items()):
            if alias in mobj:
                continue
            pt = self.find_type(tname)
            self.objs.append(dict(name=f"mo_{alias}", t=pt, c=f"tuo_mo_{alias}", r=f"(*core::ptr::addr_of_mut!(MO_{alias}))",
                                  r_ptr=f"core::ptr::addr_of_mut!(MO_{alias})", fill=True, own=False, mock=True, pt=pt))

    def forced_names(self):
        out = set()
        for fc in self.fcfg.values():
            out |= set(fc.get("force", {}))
        return sorted(out)

    def find_type(self, name):
        for c in self.tu.cursor.get_children():
            if c.kind == K.TYPEDEF_DECL and c.spelling == name:
                return c.type
            if c.kind == K.STRUCT_DECL and c.spelling == name and c.is_definition():
                return c.type
        raise SystemExit(f"extra_objs: type {name} not found in the TU")

    def obj_named(self, nm):
        for o in self.objs:
            if o["name"] == nm or o["name"] == "mo_" + nm:
                return o
        raise SystemExit(f"arg spec obj:{nm}: no such state object")

    # ---- args
    def arg_specs(self, fn):
        fcfg = self.fcfg.get(fn.spelling, {})
        specs = fcfg.get("args", [])
        out = []
        for i, a in enumerate(fn.get_arguments()):
            s = specs[i] if i < len(specs) else None
            out.append((a, s))
        return out

    # ---- emit C
    def cdecl_fn(self, fn, name):
        def decl(t, nm):
            sp = t.spelling
            if "(*)" in sp:  # function pointer: the name goes inside the parentheses
                return sp.replace("(*)", f"(*{nm})", 1)
            m = re.match(r"^(.*?)\s*((?:\[\d*\])+)$", sp)
            if m:  # array parameter
                return f"{m.group(1)} {nm}{m.group(2)}"
            return f"{sp} {nm}"
        args = [decl(a.type, f"a{i}") for i, a in enumerate(fn.get_arguments())]
        if fn.type.is_function_variadic():
            args.append("...")
        return f"{fn.result_type.spelling} {name}({', '.join(args) if args else 'void'})"

    def mock_c(self, spec):
        fn = spec["fn"]
        n = spec["name"]
        self._cur_ext = n
        lines = [self.cdecl_fn(fn, n) + " {", f"    tuo_t_begin({spec['id']});"]
        for i, a in enumerate(fn.get_arguments()):
            lines += ["    " + x for x in self.trace_arg(a.type, f"a{i}", "c")]
        lines += self.mock_ret_c(spec)
        lines.append("}")
        return lines

    def trace_arg(self, t, v, lang):
        k = classify(t)
        if lang == "c":
            if t.get_canonical().kind in (T.CONSTANTARRAY, T.INCOMPLETEARRAY):  # an array parameter is a pointer (Mtx)
                return [f"tuo_t_p({v});"]
            if k[0] in ("int", "bool"):
                return [f"tuo_t_i((tuo_i64){v});"]
            if k[0] == "f32":
                return [f"tuo_t_f32({v});"]
            if k[0] == "f64":
                return [f"tuo_t_f64({v});"]
            if k[0] == "fnptr":
                return [f"tuo_t_fn((const void*){v});"]
            if k[0] == "ptr":
                out = [f"tuo_t_p({v});"]
                pt = k[1]
                if is_cstr_ptr(pt) and self.cfg.get("cstr_nocls"):
                    # string literals may sit right after a registered const table in the C image
                    out = [f"tuo_t_i((tuo_i64)({v} != 0));"]
                if is_cstr_ptr(pt):
                    out.append(f"tuo_t_s((const char*){v});")
                else:
                    pk = classify(pt) if pt.get_canonical().kind not in (T.VOID,) else ("void",)
                    if pk[0] == "rec" and pk[1].is_definition() and getattr(self, "_cur_ext", None) not in self.cfg.get("no_arg_content", ()):
                        cl, _ = self.g2.hash(pt, f"(*{v})", "", 0)
                        out.append(f"if ({v}) {{ tuo_u64 sv = tuo_h; tuo_h = TUO_FNV0;")
                        out += ["  " + x for x in cl]
                        out.append("  tuo_trace = tuo_mix(tuo_trace, tuo_h); tuo_h = sv; }")
                    elif pk[0] in ("int", "f32", "f64") and False:
                        pass
                return out
            if k[0] == "rec":  # a record by value is not part of the trace (the call itself is)
                return []
            raise ValueError(f"mock arg {t.spelling}")
        raise ValueError(lang)

    def mock_ret_c(self, spec):
        fn = spec["fn"]
        n = spec["name"]
        rk = classify(fn.result_type)
        # out-parameters: non-const pointer to record is filled by the mock
        lines = []
        if n == "memzero":  # the real function: zero the block
            lines.append("    { int i; for (i = 0; i < (int)a1; i++) ((unsigned char*)a0)[i] = 0; }")
        for i, a in enumerate(fn.get_arguments()):
            ak = classify(a.type)
            if ak[0] == "ptr" and not ak[1].is_const_qualified() and ak[1].get_canonical().kind == T.RECORD \
                    and classify(ak[1])[1].is_definition() and not self.cfg.get("noout", {}).get(n):
                cl, _ = self.g2.fill(ak[1], f"(*a{i})", "", 1, "tuo_mst", "rt::MST")
                lines.append(f"    if (a{i}) {{")
                lines += ["        " + x for x in cl]
                lines.append("    }")
        rng = self.cfg.get("mock", {}).get(n)
        if rk[0] == "void":
            return lines
        if rk[0] in ("int", "bool") and n in self.forced_names():
            lines.append(f"    if (tuo_frc_{n} >= 0) return ({fn.result_type.spelling}) tuo_frc_{n};")
        if rk[0] in ("int", "bool"):
            if rng and "|" in rng:
                vals = rng.split("|")
                lines.append(f"    return ({fn.result_type.spelling}) ((const tuo_i64[]){{{', '.join(vals)}}})[tuo_next(&tuo_mst) % {len(vals)}];")
            elif rng:
                lo, hi = map(lambda x: int(x, 0), rng.split(".."))
                lines.append(f"    return ({fn.result_type.spelling}) tuo_draw_range(&tuo_mst, {lo}, {hi});")
            else:
                lines.append(f"    return ({fn.result_type.spelling}) tuo_draw_int(&tuo_mst);")
        elif rk[0] == "f32":
            lines.append("    return tuo_draw_f32(&tuo_mst);")
        elif rk[0] == "f64":
            lines.append("    return tuo_draw_f64(&tuo_mst);")
        elif rk[0] == "ptr":
            alias = spec["obj"]
            nullable = self.cfg.get("nullable", {}).get(n)
            if nullable:
                lines.append(f"    if (tuo_draw_range(&tuo_mst, 0, {nullable}) == 0) return 0;")
            ptc = rk[1].get_canonical()
            if any(o["name"] == f"mo_{alias}" and o.get("bytes") for o in self.objs):
                lines.append(f"    return ({fn.result_type.spelling}) tuo_mo_{alias};")
            else:
                lines.append(f"    return ({fn.result_type.spelling}) &tuo_mo_{alias};")
        elif rk[0] == "fnptr":
            lines.append("    return 0;")
        elif rk[0] == "rec" and not rk[2] and fn.result_type.get_canonical().get_size() <= 8:
            # small record by value: every field drawn from the mock stream
            cl, _ = self.g2.fill(fn.result_type, "r", "", 1, "tuo_mst", "rt::MST")
            lines.append(f"    {{ {fn.result_type.spelling} r; memset(&r, 0, sizeof r);")
            lines += ["      " + x for x in cl]
            lines.append("      return r; }")
        else:
            raise ValueError(f"mock ret {fn.result_type.spelling}")
        return lines

    def mock_rust(self, spec):
        fn = spec["fn"]
        n = spec["name"]
        self._cur_ext = n
        lines = [f"unsafe fn h_{n}(a: &[u64]) -> u64 {{", f"    rt::t_begin({spec['id']});"]
        for i, a in enumerate(fn.get_arguments()):
            lines += ["    " + x for x in self.trace_arg_rust(a.type, i)]
        # out-params
        for i, a in enumerate(fn.get_arguments()):
            ak = classify(a.type)
            if ak[0] == "ptr" and not ak[1].is_const_qualified() and ak[1].get_canonical().kind == T.RECORD \
                    and classify(ak[1])[1].is_definition() and not self.cfg.get("noout", {}).get(n):
                rt_ty = self.g2.rty(ak[1])
                _, rl = self.g2.fill(ak[1], "", f"(*p{i})", 1, "tuo_mst", "rt::MST")
                lines.append(f"    let p{i} = a[{i}] as usize as *mut {self.mt(rt_ty)};")
                lines.append(f"    if !p{i}.is_null() {{")
                lines += ["        " + x for x in rl]
                lines.append("    }")
        rk = classify(fn.result_type)
        rng = self.cfg.get("mock", {}).get(n)
        if n == "memzero":
            lines.append("    core::ptr::write_bytes(a[0] as usize as *mut u8, 0, a[1] as i64 as usize);")
        if rk[0] == "void":
            lines.append("    0")
        elif rk[0] in ("int", "bool"):
            if n in self.forced_names():
                lines.append(f"    if FRC_{n} >= 0 {{ return FRC_{n} as u64; }}")
            if rng and "|" in rng:
                vals = rng.split("|")
                lines.append(f"    [{', '.join(v + 'i64' for v in vals)}][(rt::next(&mut rt::MST) % {len(vals)}) as usize] as u64")
            elif rng:
                lo, hi = map(lambda x: int(x, 0), rng.split(".."))
                lines.append(f"    rt::draw_range(&mut rt::MST, {lo}, {hi}) as u64")
            else:
                lines.append("    rt::draw_int(&mut rt::MST) as u64")
        elif rk[0] == "f32":
            lines.append("    rt::draw_f32(&mut rt::MST).to_bits() as u64")
        elif rk[0] == "f64":
            lines.append("    rt::draw_f64(&mut rt::MST).to_bits()")
        elif rk[0] == "ptr":
            alias = spec["obj"]
            nullable = self.cfg.get("nullable", {}).get(n)
            if nullable:
                lines.append(f"    if rt::draw_range(&mut rt::MST, 0, {nullable}) == 0 {{ return 0; }}")
            if any(o["name"] == f"mo_{alias}" and o.get("bytes") for o in self.objs):
                lines.append(f"    core::ptr::addr_of_mut!(MO_{alias}) as *mut u8 as usize as u64")
            else:
                lines.append(f"    core::ptr::addr_of_mut!(MO_{alias}) as usize as u64")
        elif rk[0] == "fnptr":
            lines.append("    0")
        elif rk[0] == "rec" and not rk[2] and fn.result_type.get_canonical().get_size() <= 8:
            rt_ty = self.mt(self.g2.rty(fn.result_type))
            _, rl = self.g2.fill(fn.result_type, "", "r", 1, "tuo_mst", "rt::MST")
            lines.append(f"    let mut r: {rt_ty} = core::mem::zeroed();")
            lines += ["    " + x for x in rl]
            lines.append("    let mut v = 0u64;")
            lines.append("    core::ptr::copy_nonoverlapping(&r as *const _ as *const u8, &mut v as *mut u64 as *mut u8, core::mem::size_of_val(&r));")
            lines.append("    v")
        lines.append("}")
        return lines

    def mt(self, rty):
        """Qualify a Rust type name from Names.rty for the generated test (melee_types or the port)."""
        return self.qual(rty)

    def qual(self, s):
        # identifiers in a Rust type string -> qualified; primitives and punctuation kept
        prim = {"u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "f32", "f64", "usize", "isize", "bool", "c_void",
                "unsafe", "extern", "fn", "Option", "mut", "const"}

        def rep(m):
            w = m.group(0)
            if w in prim or w.startswith("r#"):
                return w
            return f"Q::{w}" if w not in self.local_names() else f"m::{w}"
        return re.sub(r"[A-Za-z_][A-Za-z0-9_]*", rep, s).replace('Q::"C"', '"C"')

    def rust_type_of(self, t):
        c = t
        dims = []
        while c.get_canonical().kind == T.CONSTANTARRAY:
            cc = c.get_canonical()
            dims.append(cc.element_count)
            c = cc.element_type
        r = self.g2.rty(c)
        for n in reversed(dims):
            r = f"[{r}; {n}]"
        return r

    def local_names(self):
        if not hasattr(self, "_local"):
            self._local = set(self.cfg.get("local_types", ()))
        return self._local

    def trace_arg_rust(self, t, i):
        k = classify(t)
        if t.get_canonical().kind in (T.CONSTANTARRAY, T.INCOMPLETEARRAY):  # an array parameter is a pointer (Mtx)
            return [f"rt::t_p(a[{i}] as usize as *const u8);"]
        if k[0] in ("int", "bool"):
            return [f"rt::t_i(a[{i}] as i64);"]
        if k[0] == "f32":
            return [f"rt::t_f32(f32::from_bits(a[{i}] as u32));"]
        if k[0] == "f64":
            return [f"rt::t_f64(f64::from_bits(a[{i}]));"]
        if k[0] == "fnptr":
            return [f"rt::t_fn(a[{i}] as usize);"]
        if k[0] == "ptr":
            out = [f"rt::t_p(a[{i}] as usize as *const u8);"]
            pt = k[1]
            if is_cstr_ptr(pt) and self.cfg.get("cstr_nocls"):
                out = [f"rt::t_i((a[{i}] != 0) as i64);"]
            if is_cstr_ptr(pt):
                out.append(f"rt::t_s(a[{i}] as usize as *const u8);")
            else:
                pk = classify(pt) if pt.get_canonical().kind != T.VOID else ("void",)
                if pk[0] == "rec" and pk[1].is_definition() and getattr(self, "_cur_ext", None) not in self.cfg.get("no_arg_content", ()):
                    _, rl = self.g2.hash(pt, "", f"(*p)", 0)
                    rty = self.mt(self.g2.rty(pt))
                    out.append(f"if a[{i}] != 0 {{ let p = a[{i}] as usize as *const {rty}; let sv = rt::H; rt::H = rt::FNV0;")
                    out += ["    " + x for x in rl]
                    out.append("    rt::TRACE = rt::mix(rt::TRACE, rt::H); rt::H = sv; }")
            return out
        if k[0] == "rec":  # a record by value is not part of the trace (the call itself is)
            return []
        raise ValueError(t.spelling)

    # ---- callbacks (for fn-pointer fields)
    def cb_c(self, idx, p):
        res = p.get_result()
        args = [a for a in p.argument_types()]
        cargs = [f"{a.spelling} a{i}" for i, a in enumerate(args)]
        lines = [f"static {res.spelling} tuo_cb{idx}({', '.join(cargs) or 'void'}) {{", f"    tuo_t_begin({1000 + idx});"]
        for i, a in enumerate(args):
            lines += ["    " + x for x in self.trace_arg(a, f"a{i}", "c")]
        rk = classify(res)
        if rk[0] in ("int", "bool"):
            lines.append(f"    return ({res.spelling}) tuo_draw_int(&tuo_mst);")
        elif rk[0] == "f32":
            lines.append("    return tuo_draw_f32(&tuo_mst);")
        elif rk[0] == "f64":
            lines.append("    return tuo_draw_f64(&tuo_mst);")
        elif rk[0] == "void":
            pass
        else:
            lines.append("    return 0;")
        lines.append("}")
        return lines

    def cb_rust(self, idx, p):
        res = p.get_result()
        args = list(p.argument_types())
        rargs = [f"a{i}: {self.mt(self.g2.rty(a))}" for i, a in enumerate(args)]
        rk = classify(res)
        rret = "" if rk[0] == "void" else f" -> {self.mt(self.g2.rty(res))}"
        lines = [f"unsafe extern \"C\" fn cb{idx}({', '.join(rargs)}){rret} {{", f"    rt::t_begin({1000 + idx});"]
        for i, a in enumerate(args):
            ak = classify(a)
            if ak[0] in ("int", "bool"):
                lines.append(f"    rt::t_i(a{i} as i64);")
            elif ak[0] == "f32":
                lines.append(f"    rt::t_f32(a{i});")
            elif ak[0] == "f64":
                lines.append(f"    rt::t_f64(a{i});")
            elif ak[0] == "ptr":
                lines.append(f"    rt::t_p(a{i} as *const u8);")
            elif ak[0] == "fnptr":
                lines.append(f"    rt::t_fn(a{i}.map_or(0usize, |f| f as usize));")
        if rk[0] in ("int", "bool"):
            lines.append("    rt::draw_int(&mut rt::MST) as _")
        elif rk[0] == "f32":
            lines.append("    rt::draw_f32(&mut rt::MST)")
        elif rk[0] == "f64":
            lines.append("    rt::draw_f64(&mut rt::MST)")
        elif rk[0] != "void":
            lines.append("    0 as _")
        lines.append("}")
        return lines

    # ---- object code
    def obj_c_addr(self, o):
        return o["c"]

    def obj_fill_c(self, o):
        if o.get("raw"):
            return [f"{{ int i; for (i = 0; i < {o['raw']}; i++) ((unsigned char*)&{o['c']})[i] = (unsigned char) tuo_draw_int(&tuo_st); }}"]
        if o.get("bytes"):
            return [f"{{ int i; for (i = 0; i < {o['bytes']}; i++) {o['c']}[i] = (unsigned char) tuo_draw_int(&tuo_st); }}"]
        t = o["t"]
        if o.get("size"):
            k = classify(t)
            cl, _ = self.g2.fill(k[1], f"{o['c']}[i0]", "x", 1)
            return [f"{{ int i0; for (i0 = 0; i0 < {o['size']}; i0++) {{"] + ["  " + x for x in cl] + ["} }"]
        cl, _ = self.g2.fill(t, o["c"], "", 0)
        return cl

    def obj_fill_r(self, o):
        if o.get("raw"):
            return [f"for i in 0..{o['raw']} {{ *({o['r_ptr']} as *mut u8).add(i) = rt::draw_int(&mut rt::ST) as u8; }}"]
        if o.get("bytes"):
            return [f"for i in 0..{o['bytes']} {{ {o['r']}[i] = rt::draw_int(&mut rt::ST) as u8; }}"]
        t = o["t"]
        if o.get("size"):
            k = classify(t)
            _, rl = self.g2.fill(k[1], "", f"{o['r']}[i0]", 1)
            return [f"for i0 in 0..{o['size']} {{"] + ["    " + x for x in rl] + ["}"]
        _, rl = self.g2.fill(t, "", o["r"], 0)
        return rl

    def obj_hash_c(self, o):
        if o.get("raw"):
            return [f"tuo_h_bytes(&{o['c']}, {o['raw']});"]
        if o.get("bytes"):
            return [f"tuo_h_bytes({o['c']}, {o['bytes']});"]
        t = o["t"]
        if o.get("size"):
            k = classify(t)
            cl, _ = self.g2.hash(k[1], f"{o['c']}[i0]", "x", 1)
            return [f"{{ int i0; for (i0 = 0; i0 < {o['size']}; i0++) {{"] + ["  " + x for x in cl] + ["} }"]
        cl, _ = self.g2.hash(t, o["c"], "", 0)
        return cl

    def obj_hash_r(self, o):
        if o.get("raw"):
            return [f"rt::h_bytes({o['r_ptr']} as *const u8, {o['raw']});"]
        if o.get("bytes"):
            return [f"rt::h_bytes({o['r']}.as_ptr(), {o['bytes']});"]
        t = o["t"]
        if o.get("size"):
            k = classify(t)
            _, rl = self.g2.hash(k[1], "", f"{o['r']}[i0]", 1)
            return [f"for i0 in 0..{o['size']} {{"] + ["    " + x for x in rl] + ["}"]
        _, rl = self.g2.hash(t, "", o["r"], 0)
        return rl

    def obj_c_decl(self, o):
        if o.get("own"):
            return None
        if o.get("ext"):
            t = o["t"]
            if o.get("size"):
                k = classify(t)
                return f"{k[1].spelling} {o['name']}[{o['size']}];"
            if o.get("const"):
                return f"__attribute__((used)) {t.spelling.replace('const ', 'const ', 1)} {o['name']} = {{0}};" \
                    if classify(t)[0] != "int" else f"__attribute__((used)) {t.spelling} {o['name']} = 0;"
            return f"{self.tspell(t, o['name'])};"
        if o.get("mock"):
            if o.get("bytes"):
                return f"static unsigned char {o['c']}[{o['bytes']}];"
            return f"static {self.tspell(o['t'], o['c']).replace('const ', '')};"  # (a mock the callee returns a const pointer to is still filled)
        return None

    def tspell(self, t, name):
        """C declaration `T name` handling arrays."""
        c = t
        dims = ""
        while c.get_canonical().kind == T.CONSTANTARRAY:
            cc = c.get_canonical()
            dims += f"[{cc.element_count}]"
            c = cc.element_type
        base = c.spelling
        return f"{base} {name}{dims}"

    # ---- driver code per function
    def consts(self):
        if hasattr(self, "_consts"):
            return self._consts
        vals = set()
        for f in self.funcs.values():
            for t in f.get_tokens():
                if t.kind == ci.TokenKind.LITERAL:
                    m = re.fullmatch(r"(0[xX][0-9a-fA-F]+|[0-9]+)[uUlL]*", t.spelling)
                    if m:
                        v = int(m.group(1), 0)
                        if v < (1 << 31):
                            vals.update((v - 1, v, v + 1))
        self._consts = sorted(vals)[:500]
        return self._consts

    def rn(self, n):
        return self.cfg.get("rust_names", {}).get(n, n)

    def func_cases(self):
        return self.cfg.get("cases", 60)

    def emit_c(self):
        L = []
        inc = self.patched.as_posix()
        L.append(f'#include "{inc}"')
        L.append(f'#include "{(HERE / "tu_oracle.h").as_posix()}"')
        # object declarations
        for o in self.objs:
            d = self.obj_c_decl(o)
            if d:
                L.append(d)
        # callbacks need forward decl before objects use them: emit before mocks
        # (callbacks are discovered while generating fills, so generate everything first)
        body = []
        # fill-all / hash-all
        for k, o in enumerate(self.snap_objs()):
            L.append(f"static unsigned char tuo_snap{k}[sizeof({o['c']})];")
        fa = ["static void tuo_fill_all(void) {"]
        for k, o in enumerate(self.snap_objs()):
            fa.append(f"    {{ unsigned char* d_ = (unsigned char*)&{o['c']}; unsigned i_; for (i_ = 0; i_ < sizeof({o['c']}); i_++) d_[i_] = tuo_snap{k}[i_]; }}")
        for o in self.objs:
            if o["fill"]:
                fa += ["    " + x for x in self.obj_fill_c(o)]
        for cs, rs in self.cfg.get("fixups", []):
            fa.append("    " + cs)
        fa.append("}")
        ha = ["static void tuo_hash_objs(void) {", "    tuo_u64 tot = TUO_FNV0; int k = 0;"]
        for o in self.objs:
            ha.append("    tuo_h = TUO_FNV0;")
            ha += ["    " + x for x in self.obj_hash_c(o)]
            ha.append("    tuo_obj_hash[k++] = tuo_h; tot = tuo_mix(tot, tuo_h);")
        ha.append("    tuo_h = tot;")
        ha.append("}")
        # init: registry
        ini = ["static void tuo_init(void) {"]
        for o in self.objs:
            sz = f"sizeof({o['c']})" if not o.get("size") else f"sizeof({o['c']})"
            ini.append(f'    tuo_reg_add(&{o["c"]}, {sz}, "{o["name"]}");' if not o.get("bytes") else
                       f'    tuo_reg_add({o["c"]}, {o["bytes"]}, "{o["name"]}");')
        for k, o in enumerate(self.snap_objs()):
            ini.append(f"    {{ const unsigned char* s_ = (const unsigned char*)&{o['c']}; unsigned i_; for (i_ = 0; i_ < sizeof({o['c']}); i_++) tuo_snap{k}[i_] = s_[i_]; }}")
        ini.append("    tuo_nstatic = tuo_nreg;")
        for i, v in enumerate(self.consts()):
            ini.append(f"    tuo_consts[{i}] = {v}LL;")
        ini.append(f"    tuo_nconsts = {len(self.consts())};")
        # function registry: defined funcs, ext funcs, callbacks (cbs appended after generation)
        for n in sorted(self.funcs):
            ini.append(f"    tuo_fn_add((const void*){n});")
        for s in self.ext:
            ini.append(f"    tuo_fn_add((const void*){s['name']});")
        self._ini_index = len(ini)
        ini.append("@@CBS@@")
        ini.append("}")
        # mocks
        mocks = [f"static tuo_i64 tuo_frc_{nm} = -1;" for nm in self.forced_names()]
        for s in self.ext:
            mocks += self.mock_c(s)
        # runners
        runners, table = [], []
        for idx, n in enumerate(self.test_funcs):
            runners += self.runner_c(idx, n)
            table.append(f'    "{n}",')
        # callbacks (now known)
        cbs = []
        cbreg = []
        # generating mocks/runners may have added callbacks
        for key, (i, p) in sorted(self.g2.cbs.items(), key=lambda kv: kv[1][0]):
            cbs += self.cb_c(i, p)
            cbreg.append(f"    tuo_fn_add((const void*)tuo_cb{i});")
        ini = [x if x != "@@CBS@@" else "\n".join(cbreg) for x in ini]
        # forward declarations of callbacks before fill_all
        fwd = []
        for key, (i, p) in sorted(self.g2.cbs.items(), key=lambda kv: kv[1][0]):
            args = ", ".join(a.spelling for a in p.argument_types()) or "void"
            fwd.append(f"static {p.get_result().spelling} tuo_cb{i}({args});")
        L += fwd
        L += ["", "/* ---- mocks of the external functions ---- */"] + mocks
        L += ["", "/* ---- callbacks ---- */"] + cbs
        L += ["", *fa, "", *ha, "", *ini]
        L += ["", "/* ---- runners ---- */", *runners]
        L += ["", "const char* const tuo_names[] = {", *table, "    0 };", f"int tuo_nfuncs = {len(self.test_funcs)};",
              "typedef void (*tuo_runner_t)(tuo_u64, tuo_u64*);", "extern const tuo_runner_t tuo_runners[];"]
        L += ["const tuo_runner_t tuo_runners[] = {"] + [f"    tuo_run_{n}," for n in self.test_funcs] + ["};"]
        tabs = [o for o in self.objs if o.get("own") and not o["fill"]]
        L += ["const char* const tuo_tab_names[] = {"] + [f'    "{o["name"]}",' for o in tabs] + ["    0 };"]
        L += ["const void* const tuo_tab_ptr[] = {"] + [f"    &{o['c']}," for o in tabs] + ["    0 };"]
        L += ["const unsigned tuo_tab_size[] = {"] + [f"    sizeof({o['c']})," for o in tabs] + ["    0 };"]
        L += ["int tuo_nobjs(void) { return tuo_nstatic; }",
              "const char* tuo_objname(int i) { return tuo_reg_name[i]; }",
              "tuo_u64 tuo_objhash(int i) { return tuo_obj_hash[i]; }",
              "tuo_u64 tuo_objsize(int i) { return tuo_reg_size[i]; }",
              "void tuo_setup(void) { tuo_init(); }"]
        return "\n".join(L) + "\n"

    def arg_decl_c(self, i, a, s):
        """-> (decl lines, call expression, extra state hash lines (C), pre lines)"""
        k = classify(a.type)
        t = a.type
        name = f"a{i}"
        if k[0] in ("int", "bool"):
            if s and ".." in s:
                lo, hi = map(lambda x: int(x, 0), s.split(".."))
                return ([f"{t.spelling} {name} = ({t.spelling}) tuo_draw_range(&tuo_st, {lo}, {hi});"], name, [])
            if s and "|" in s:
                vals = s.split("|")
                return ([f"{t.spelling} {name} = ({t.spelling}) ((const tuo_i64[]){{{', '.join(vals)}}})[tuo_next(&tuo_st) % {len(vals)}];"], name, [])
            if s and re.fullmatch(r"-?(0[xX][0-9a-fA-F]+|\d+)", s):
                return ([f"{t.spelling} {name} = ({t.spelling}) {s};"], name, [])
            return ([f"{t.spelling} {name} = ({t.spelling}) tuo_draw_int(&tuo_st);"], name, [])
        if k[0] == "f32":
            return ([f"float {name} = tuo_draw_f32(&tuo_st);"], name, [])
        if k[0] == "f64":
            return ([f"double {name} = tuo_draw_f64(&tuo_st);"], name, [])
        if k[0] == "fnptr":
            cb = self.g2.cb(k[1])
            decl = t.spelling.replace("(*)", f"(*{name})") if "(*)" in t.spelling else f"{t.spelling} {name}"
            return ([f"{decl} = (tuo_draw_int(&tuo_st) & 1) ? (({t.spelling}) tuo_cb{cb}) : 0;"], name, [])
        if k[0] == "ptr":
            pt = k[1]
            if s == "null":
                return ([f"{t.spelling} {name} = 0;"], name, [])
            if s and s.startswith("obj:"):
                return ([f"{t.spelling} {name} = ({t.spelling}) &{self.obj_named(s[4:])['c']};"], name, [])
            if s and s.startswith("idx:"):
                _, on, rg = s.split(":")
                lo, hi = map(lambda x: int(x, 0), rg.split(".."))
                return ([f"{t.spelling} {name} = ({t.spelling}) &{on}[tuo_draw_range(&tuo_st, {lo}, {hi})];"], name, [])
            if is_cstr_ptr(pt) or (s == "str"):
                return ([f"static char tuo_ab_{{F}}_{i}[16];",
                         f"{{ int j; for (j = 0; j < 15; j++) tuo_ab_{{F}}_{i}[j] = (char)('a' + tuo_draw_range(&tuo_st, 0, 25)); tuo_ab_{{F}}_{i}[15] = 0; }}",
                         f"{t.spelling} {name} = ({t.spelling}) tuo_ab_{{F}}_{i};"], name, [])
            cnt = 1
            lst = None
            if s and s.startswith("buf"):
                m = re.fullmatch(r"buf(?::(\d+))?", s)
                cnt = int(m.group(1)) if m and m.group(1) else 1
            if s and s.startswith("list:"):
                _, cn, vs = s.split(":")
                cnt, lst = int(cn), vs.split("|")
            ptc = pt.get_canonical()
            if ptc.kind == T.VOID:
                return ([f"static unsigned char tuo_ab_{{F}}_{i}[64];",
                         f"{{ int j; for (j = 0; j < 64; j++) tuo_ab_{{F}}_{i}[j] = (unsigned char) tuo_draw_int(&tuo_st); }}",
                         f"tuo_reg_add(tuo_ab_{{F}}_{i}, 64, \"arg{i}\");",
                         f"{t.spelling} {name} = (void*) tuo_ab_{{F}}_{i};"], name,
                        [f"tuo_h = TUO_FNV0; tuo_h_bytes(tuo_ab_{{F}}_{i}, 64); tuo_dyn(tuo_h);"])
            pk = classify(pt)
            cl, _ = self.g2.fill(pt, f"tuo_ab_{{F}}_{i}[e]", "", 2)
            if lst:
                cl = [f"tuo_ab_{{F}}_{i}[e] = (__typeof__(tuo_ab_{{F}}_{i}[e])) ((const tuo_i64[]){{{', '.join(lst)}}})[tuo_next(&tuo_st) % {len(lst)}];"]
            hl, _ = self.g2.hash(pt, f"tuo_ab_{{F}}_{i}[e]", "", 2)
            decl = [f"static {pt.get_canonical().spelling if pt.get_canonical().kind != T.RECORD else pt.spelling.replace('const ', '')} tuo_ab_{{F}}_{i}[{cnt}];" if False else
                    f"static {self.base_spell(pt)} tuo_ab_{{F}}_{i}[{cnt}];",
                    f"{{ int e; for (e = 0; e < {cnt}; e++) {{"] + ["    " + x for x in cl] + ["} }",
                    f"tuo_reg_add(tuo_ab_{{F}}_{i}, sizeof(tuo_ab_{{F}}_{i}), \"arg{i}\");",
                    f"{t.spelling} {name} = ({t.spelling}) tuo_ab_{{F}}_{i};"]
            post = [f"tuo_h = TUO_FNV0; {{ int e; for (e = 0; e < {cnt}; e++) {{"] + ["    " + x for x in hl] + ["} }", "tuo_dyn(tuo_h);"]
            return (decl, name, post)
        raise ValueError(f"arg type {t.spelling}")

    def base_spell(self, pt):
        s = pt.spelling
        return s.replace("const ", "")

    def runner_c(self, idx, n):
        fn = self.funcs[n]
        cases = []
        L = [f"static void tuo_run_{n}(tuo_u64 seed, tuo_u64* out) {{"]
        L += ["    tuo_st = seed; tuo_mst = seed ^ 0x5DEECE66DULL; tuo_trace = TUO_FNV0; tuo_nreg = tuo_nstatic; tuo_ndyn = 0;",
              "    tuo_fill_all();"]
        for nm, v in sorted(self.fcfg.get(n, {}).get("force", {}).items()):
            if "|" in str(v):
                vs = str(v).split("|")
                L.append(f"    tuo_frc_{nm} = ((const tuo_i64[]){{{', '.join(vs)}}})[tuo_next(&tuo_st) % {len(vs)}];")
            else:
                L.append(f"    tuo_frc_{nm} = {v};")
        call_args, posts = [], []
        for i, (a, s) in enumerate(self.arg_specs(fn)):
            decl, expr, post = self.arg_decl_c(i, a, s)
            L += ["    " + x.replace("{F}", n) for x in decl]
            call_args.append(expr)
            posts += [x.replace("{F}", n) for x in post]
        rk = classify(fn.result_type)
        call = f"{n}({', '.join(call_args)})"
        if rk[0] == "void":
            L.append(f"    {call};")
            L.append("    out[0] = 0;")
        elif rk[0] in ("int", "bool"):
            L.append(f"    {{ {fn.result_type.spelling} r = {call};")
            L.append("      out[0] = (tuo_u64)(tuo_i64)r; }" if rk[0] == "bool" or rk[2] else "      out[0] = (tuo_u64)r; }")
        elif rk[0] == "f32":
            L.append(f"    {{ union {{ float f; unsigned u; }} x; x.f = {call}; out[0] = x.u; }}")
        elif rk[0] == "f64":
            L.append(f"    {{ union {{ double f; tuo_u64 u; }} x; x.f = {call}; out[0] = x.u; }}")
        elif rk[0] == "ptr" and is_cstr_ptr(rk[1]):
            L.append(f"    {{ const char* p = {call}; tuo_h = TUO_FNV0; tuo_h_cstr(p); out[0] = tuo_mix(tuo_ptr_class(p), tuo_h); }}")
        elif rk[0] == "ptr":
            L.append(f"    out[0] = tuo_ptr_class((const void*){call});")
        else:
            raise ValueError(f"ret {fn.result_type.spelling}")
        for nm in sorted(self.fcfg.get(n, {}).get("force", {})):
            L.append(f"    tuo_frc_{nm} = -1;")
        L.append("    out[1] = tuo_trace;")
        L.append("    tuo_hash_objs();")
        L.append("    { tuo_u64 tot = tuo_h; int k;")
        L.append("      tuo_h = tot;")
        for p in posts:
            L.append("      " + p)
        L.append("      for (k = 0; k < tuo_ndyn; k++) tot = tuo_mix(tot, tuo_dynh[k]);")
        L.append("      out[2] = tot; }")
        L.append("}")
        return L

    # ---- Rust
    def emit_rust(self):
        mod = self.mod
        L = []
        L.append(f"//! State oracle for src/melee/gm TU `{self.stem}` ({self.src}): GENERATED by")
        L.append("//! rust/melee_modes/oracle_tu/tuoracle.py; do not edit. The C results are in")
        L.append(f"//! tests/data/tu_{self.stem}.txt (the TU compiled with the shipping flags, contraction on).")
        L.append("#![allow(non_snake_case, non_upper_case_globals, unused_variables, unused_mut, unused_unsafe, unused_imports, static_mut_refs, dead_code, function_casts_as_integer, clippy::all)]")
        L.append("#[path = \"common/tu_rt.rs\"]")
        L.append("mod rt;")
        L.append(f"use melee_modes::tu::{mod} as m;")
        L.append("use melee_modes::tu::ext_hook::EXT_HOOK;")
        L.append("use melee_types as Q;")
        L.append("use core::ffi::c_void;")
        L.append("#[repr(C, align(16))]")
        L.append("struct A16([u8; 64]);")
        L.append("")
        # mock object statics
        for o in self.objs:
            if o.get("mock"):
                if o.get("bytes"):
                    L.append(f"static mut {o['r']}: [u8; {o['bytes']}] = [0; {o['bytes']}];")
                else:
                    L.append(f"static mut MO_{o['name'][3:]}: {self.mt(self.g2.rty(o['pt']))} = unsafe {{ core::mem::zeroed() }};")
        L.append("")
        # arg buffers
        argbufs = []
        for n in self.test_funcs:
            fn = self.funcs[n]
            for i, (a, s) in enumerate(self.arg_specs(fn)):
                k = classify(a.type)
                if k[0] == "ptr" and s != "null" and not (s and s.startswith("obj:")):
                    pt = k[1]
                    if is_cstr_ptr(pt) or s == "str":
                        argbufs.append(f"static mut AB_{n}_{i}: [u8; 16] = [0; 16];")
                    elif pt.get_canonical().kind == T.VOID:
                        argbufs.append(f"static mut AB_{n}_{i}: A16 = A16([0; 64]);")
                    else:
                        cnt = 1
                        if s and (s.startswith("buf:") or s.startswith("list:")):
                            cnt = int(s.split(":")[1])
                        argbufs.append(f"static mut AB_{n}_{i}: [{self.mt(self.g2.rty(pt))}; {cnt}] = unsafe {{ core::mem::zeroed() }};")
        L += argbufs
        L.append("")
        # mocks
        for nm in self.forced_names():
            L.append(f"static mut FRC_{nm}: i64 = -1;")
        for s in self.ext:
            L += self.mock_rust(s)
        L.append("")
        L.append("unsafe fn hook(name: &'static str, a: &[u64]) -> u64 {")
        L.append("    match name {")
        for s in self.ext:
            L.append(f'        "{s["name"]}" => h_{s["name"]}(a),')
        L.append('        _ => panic!("unmocked ext function {name}"),')
        L.append("    }")
        L.append("}")
        # callbacks
        cbl = []
        for key, (i, p) in sorted(self.g2.cbs.items(), key=lambda kv: kv[1][0]):
            cbl += self.cb_rust(i, p)
        # fill_all / hash_objs
        L.append("")
        for k, o in enumerate(self.snap_objs()):
            L.append(f"static mut SNAP{k}: Vec<u8> = Vec::new();")
        L.append("unsafe fn fill_all() {")
        for k, o in enumerate(self.snap_objs()):
            L.append(f"    core::ptr::copy_nonoverlapping(SNAP{k}.as_ptr(), {o['r_ptr']} as *mut u8, SNAP{k}.len());")
        for o in self.objs:
            if o["fill"]:
                L += ["    " + x for x in self.obj_fill_r(o)]
        for cs, rs in self.cfg.get("fixups", []):
            L.append("    " + rs)
        L.append("}")
        L.append("unsafe fn hash_objs() {")
        L.append("    let mut tot = rt::FNV0;")
        L.append("    rt::OBJ_HASH.clear();")
        for o in self.objs:
            L.append("    rt::H = rt::FNV0;")
            L += ["    " + x for x in self.obj_hash_r(o)]
            L.append("    rt::OBJ_HASH.push(rt::H); tot = rt::mix(tot, rt::H);")
        L.append("    rt::H = tot;")
        L.append("}")
        L.append("unsafe fn init() {")
        for o in self.objs:
            if o.get("bytes"):
                L.append(f'    rt::reg_add({o["r"]}.as_ptr(), {o["bytes"]}, "{o["name"]}");')
            else:
                L.append(f'    rt::reg_add(core::ptr::addr_of!({o["r"]}), core::mem::size_of_val(&{o["r"]}), "{o["name"]}");')
        for k, o in enumerate(self.snap_objs()):
            L.append(f"    SNAP{k} = core::slice::from_raw_parts({o['r_ptr']} as *const u8, core::mem::size_of_val(&*{o['r_ptr']})).to_vec();")
        L.append("    rt::NSTATIC = rt::REGS.len();")
        L.append("    rt::CONSTS = vec![" + ", ".join(str(v) for v in self.consts()) + "];")
        for n in sorted(self.funcs):
            L.append(f"    rt::fn_add(m::{self.rn(n)} as usize);")
        for s in self.ext:
            L.append(f"    rt::fn_add(m::ext::{s['name']} as usize);")
        for key, (i, p) in sorted(self.g2.cbs.items(), key=lambda kv: kv[1][0]):
            L.append(f"    rt::fn_add(cb{i} as usize);")
        L.append("    EXT_HOOK = Some(hook);")
        L.append("}")
        # runners
        for idx, n in enumerate(self.test_funcs):
            L += self.runner_rust(idx, n)
        # callbacks (the runners may have added some)
        cbl = []
        for key, (i, p) in sorted(self.g2.cbs.items(), key=lambda kv: kv[1][0]):
            cbl += self.cb_rust(i, p)
        L += cbl
        L.append("")
        L.append("type Runner = unsafe fn(u64) -> [u64; 3];")
        L.append("fn runner(name: &str) -> Runner {")
        L.append("    match name {")
        for n in self.test_funcs:
            L.append(f'        "{n}" => run_{n},')
        L.append('        _ => panic!("no runner for {name}"),')
        L.append("    }")
        L.append("}")
        L.append("")
        L.append("#[test]")
        L.append(f"fn tu_{self.stem}_matches_c() {{")
        L.append(f'    let data = include_str!("data/tu_{self.stem}.txt");')
        L.append("    unsafe { init(); }")
        L.append("    let mut bad: Vec<String> = Vec::new();")
        L.append("    let mut by_fn: std::collections::BTreeMap<String, (usize, usize)> = Default::default();")
        L.append("    for line in data.lines() {")
        L.append("        let mut it = line.split_whitespace();")
        L.append("        let name = it.next().unwrap();")
        L.append("        if let Ok(f) = std::env::var(\"TU_FN\") { if f != name { continue; } }")
        L.append("        let seed = u64::from_str_radix(it.next().unwrap(), 16).unwrap();")
        L.append("        assert_eq!(it.next(), Some(\"=\"));")
        L.append("        let want: Vec<u64> = it.map(|x| u64::from_str_radix(x, 16).unwrap()).collect();")
        L.append("        let got = unsafe { runner(name)(seed) };")
        L.append("        let e = by_fn.entry(name.to_string()).or_default();")
        L.append("        e.0 += 1;")
        L.append("        if got[..] != want[..] {")
        L.append("            e.1 += 1;")
        L.append("            if e.1 <= 2 { bad.push(format!(\"{name} seed {seed:x}: C {want:x?} Rust {got:x?}\")); }")
        L.append("            if e.1 == 1 && std::env::var(\"TU_OBJS\").is_ok() { unsafe { eprintln!(\"MISMATCH {name} {seed:x}\"); for (k, h) in rt::OBJ_HASH.iter().enumerate() { eprintln!(\"  rust obj {} {:x} size {}\", rt::REGS[k].name, h, rt::REGS[k].size); } } }")
        L.append("        }")
        L.append("    }")
        L.append("    for (n, (t, b)) in &by_fn { if *b != 0 { eprintln!(\"FAIL {n}: {b}/{t} cases differ\"); } }")
        L.append("    assert!(bad.is_empty(), \"{} functions differ:\\n{}\", by_fn.values().filter(|v| v.1 != 0).count(), bad.join(\"\\n\"));")
        L.append("    eprintln!(\"{} functions, {} cases equal to the C\", by_fn.len(), data.lines().count());")
        L.append("}")
        return "\n".join(L) + "\n"

    def arg_rust(self, i, a, s, n):
        k = classify(a.type)
        name = f"a{i}"
        if k[0] in ("int", "bool"):
            if s and ".." in s:
                lo, hi = map(lambda x: int(x, 0), s.split(".."))
                return ([f"let {name} = rt::draw_range(&mut rt::ST, {lo}, {hi});"], f"{name} as _", [])
            if s and "|" in s:
                vals = s.split("|")
                return ([f"let {name}: i64 = [{', '.join(v + 'i64' for v in vals)}][(rt::next(&mut rt::ST) % {len(vals)}) as usize];"], f"{name} as _", [])
            if s and re.fullmatch(r"-?(0[xX][0-9a-fA-F]+|\d+)", s):
                return ([f"let {name}: i64 = {s};"], f"{name} as _", [])
            return ([f"let {name} = rt::draw_int(&mut rt::ST);"], f"{name} as _", [])
        if k[0] == "f32":
            return ([f"let {name} = rt::draw_f32(&mut rt::ST);"], name, [])
        if k[0] == "f64":
            return ([f"let {name} = rt::draw_f64(&mut rt::ST);"], name, [])
        if k[0] == "fnptr":
            cb = self.g2.cb(k[1])
            return ([f"let {name} = if rt::draw_int(&mut rt::ST) & 1 != 0 {{ Some(cb{cb} as _) }} else {{ None }};"], name, [])
        if k[0] == "ptr":
            pt = k[1]
            if s == "null":
                return ([f"let {name} = 0usize;"], f"{name} as _", [])
            if s and s.startswith("obj:"):
                o = self.obj_named(s[4:])
                return ([f"let {name} = {o['r_ptr']};"], f"{name} as _", [])
            if s and s.startswith("idx:"):
                _, on, rg = s.split(":")
                lo, hi = map(lambda x: int(x, 0), rg.split(".."))
                o = next(o for o in self.objs if o["name"] == on)
                return ([f"let {name} = &mut (*{o['r_ptr']})[rt::draw_range(&mut rt::ST, {lo}, {hi}) as usize] as *mut _;"], f"{name} as _", [])
            if is_cstr_ptr(pt) or s == "str":
                return ([f"for j in 0..15 {{ AB_{n}_{i}[j] = b'a' + rt::draw_range(&mut rt::ST, 0, 25) as u8; }} AB_{n}_{i}[15] = 0;",
                         f"let {name} = AB_{n}_{i}.as_mut_ptr();"], f"{name} as _", [])
            if pt.get_canonical().kind == T.VOID:
                return ([f"for j in 0..64 {{ AB_{n}_{i}.0[j] = rt::draw_int(&mut rt::ST) as u8; }}",
                         f"rt::reg_add(AB_{n}_{i}.0.as_ptr(), 64, \"arg{i}\");",
                         f"let {name} = AB_{n}_{i}.0.as_mut_ptr();"], f"{name} as _",
                        [f"rt::H = rt::FNV0; rt::h_bytes(AB_{n}_{i}.0.as_ptr(), 64); rt::DYN.push(rt::H);"])
            cnt = 1
            lst = None
            if s and s.startswith("buf:"):
                cnt = int(s.split(":")[1])
            if s and s.startswith("list:"):
                _, cn, vs = s.split(":")
                cnt, lst = int(cn), vs.split("|")
            _, cl = self.g2.fill(pt, "", f"AB_{n}_{i}[e]", 2)
            if lst:
                cl = [f"AB_{n}_{i}[e] = [{', '.join(v + 'i64' for v in lst)}][(rt::next(&mut rt::ST) % {len(lst)}) as usize] as _;"]
            _, hl = self.g2.hash(pt, "", f"AB_{n}_{i}[e]", 2)
            decl = [f"for e in 0..{cnt} {{"] + ["    " + x for x in cl] + ["}",
                    f"rt::reg_add(core::ptr::addr_of!(AB_{n}_{i}), core::mem::size_of_val(&AB_{n}_{i}), \"arg{i}\");",
                    f"let {name} = AB_{n}_{i}.as_mut_ptr();"]
            post = ["rt::H = rt::FNV0;", f"for e in 0..{cnt} {{"] + ["    " + x for x in hl] + ["}", "rt::DYN.push(rt::H);"]
            return (decl, f"{name} as _", post)
        raise ValueError(a.type.spelling)

    def runner_rust(self, idx, n):
        fn = self.funcs[n]
        L = [f"unsafe fn run_{n}(seed: u64) -> [u64; 3] {{",
             "    rt::ST = seed; rt::MST = seed ^ 0x5DEECE66D; rt::TRACE = rt::FNV0; rt::REGS.truncate(rt::NSTATIC); rt::DYN.clear();",
             "    fill_all();"]
        for nm, v in sorted(self.fcfg.get(n, {}).get("force", {}).items()):
            if "|" in str(v):
                vs = str(v).split("|")
                L.append(f"    FRC_{nm} = [{', '.join(x + 'i64' for x in vs)}][(rt::next(&mut rt::ST) % {len(vs)}) as usize];")
            else:
                L.append(f"    FRC_{nm} = {v};")
        args, posts = [], []
        for i, (a, s) in enumerate(self.arg_specs(fn)):
            decl, expr, post = self.arg_rust(i, a, s, n)
            L += ["    " + x for x in decl]
            args.append(expr)
            posts += post
        rk = classify(fn.result_type)
        call = f"m::{self.rn(n)}({', '.join(args)})"
        if rk[0] == "void":
            L.append(f"    {call};")
            L.append("    let ret = 0u64;")
        elif rk[0] in ("int", "bool"):
            L.append(f"    let r = {call};")
            L.append("    let ret = r as i64 as u64;" if (rk[0] == "bool" or rk[2]) else "    let ret = r as u64;")
        elif rk[0] == "f32":
            L.append(f"    let ret = ({call}).to_bits() as u64;")
        elif rk[0] == "f64":
            L.append(f"    let ret = ({call}).to_bits();")
        elif rk[0] == "ptr" and is_cstr_ptr(rk[1]):
            L.append(f"    let p = {call} as *const u8; rt::H = rt::FNV0; rt::h_cstr(p); let ret = rt::mix(rt::ptr_class(p), rt::H);")
        elif rk[0] == "ptr":
            L.append(f"    let ret = rt::ptr_class({call} as *const u8);")
        for nm in sorted(self.fcfg.get(n, {}).get("force", {})):
            L.append(f"    FRC_{nm} = -1;")
        L.append("    let trace = rt::TRACE;")
        L.append("    hash_objs();")
        L.append("    let mut tot = rt::H;")
        for p in posts:
            L.append("    " + p)
        L.append("    for h in rt::DYN.iter() { tot = rt::mix(tot, *h); }")
        L.append("    [ret, trace, tot]")
        L.append("}")
        return L


def rust_literal(tg, t, data, off):
    k = classify(t)
    if k[0] == "int":
        n = k[1]
        v = int.from_bytes(data[off:off + n], "little", signed=k[2])
        return str(v) if v >= 0 or not k[2] else f"{v}"
    if k[0] == "bool":
        return "true" if data[off] else "false"
    if k[0] == "f32":
        return f"f32::from_bits(0x{int.from_bytes(data[off:off+4], 'little'):08X})"
    if k[0] == "f64":
        return f"f64::from_bits(0x{int.from_bytes(data[off:off+8], 'little'):016X})"
    if k[0] == "ptr":
        if int.from_bytes(data[off:off + 8], "little") != 0:
            raise ValueError("non-null pointer in table")
        return "core::ptr::null_mut()"
    if k[0] == "fnptr":
        raise ValueError("function pointer in table")
    if k[0] == "arr":
        es = k[1].get_canonical().get_size()
        return "[" + ", ".join(rust_literal(tg, k[1], data, off + i * es) for i in range(k[2])) + "]"
    if k[0] == "rec":
        if k[2]:
            raise ValueError("union in table")
        d = k[1]
        parts = []
        for f in d.get_children():
            if f.kind != K.FIELD_DECL:
                raise ValueError("anonymous member in table")
            if not f.spelling:
                continue
            if f.is_bitfield():
                raise ValueError("bitfield in table")
            parts.append(f"{tg.g2.rfield(f)}: " + rust_literal(tg, f.type, data, off + f.get_field_offsetof() // 8))
        return f"{tg.g2.rty(t)} {{ " + ", ".join(parts) + " }"
    raise ValueError(k)


def write_tables(tg, text):
    out = ["//! GENERATED by rust/melee_modes/oracle_tu/tuoracle.py from the C initialisers (the TU compiled",
           "//! natively, bytes read back and re-spelled field by field). Do not edit.",
           "#![allow(non_upper_case_globals, clippy::all)]", "use super::*;", ""]
    n = 0
    for line in text.splitlines():
        name, hexs = line.split(" ")
        o = next(o for o in tg.objs if o["name"] == name)
        data = bytes.fromhex(hexs)
        try:
            lit = rust_literal(tg, o["t"], data, 0)
        except ValueError as e:
            out.append(f"// {name}: not generated ({e})")
            continue
        out.append(f"pub static mut {g.ident(name)}: {tg.rust_type_of(o['t'])} = {lit};")
        n += 1
    if n == 0:
        print(f"{tg.stem}: no table could be generated: " + "; ".join(x for x in out if x.startswith("// ")))
        return
    d = ROOT / f"rust/melee_modes/src/tu/{tg.mod}"
    d.mkdir(exist_ok=True)
    (d / "data.rs").write_text("\n".join(out) + "\n", newline="\n")
    print(f"{tg.stem}: {n} tables -> {d / 'data.rs'}")


def driver_main_c(stem):
    return r'''#include <stdio.h>
#include <string.h>
#include <stdlib.h>
typedef unsigned long long u64;
extern const char* const tuo_names[];
extern int tuo_nfuncs;
typedef void (*runner_t)(u64, u64*);
extern const runner_t tuo_runners[];
extern void tuo_setup(void);
extern int tuo_nobjs(void);
extern const char* tuo_objname(int);
extern u64 tuo_objhash(int);
extern u64 tuo_objsize(int);
int main(int argc, char** argv) {
    char name[256]; unsigned long long seed; u64 out[3];
    tuo_setup();
    if (argc > 1 && !strcmp(argv[1], "--list")) {
        for (int i = 0; i < tuo_nfuncs; i++) printf("%s\n", tuo_names[i]);
        return 0;
    }
    if (argc > 1 && !strcmp(argv[1], "--tables")) {
        extern const char* const tuo_tab_names[]; extern const void* const tuo_tab_ptr[]; extern const unsigned tuo_tab_size[];
        for (int i = 0; tuo_tab_names[i]; i++) {
            const unsigned char* p = (const unsigned char*)tuo_tab_ptr[i];
            printf("%s ", tuo_tab_names[i]);
            for (unsigned k = 0; k < tuo_tab_size[i]; k++) printf("%02x", p[k]);
            printf("\n");
        }
        return 0;
    }
    if (argc > 3 && !strcmp(argv[1], "--dump")) {
        int i;
        for (i = 0; i < tuo_nfuncs; i++) if (!strcmp(tuo_names[i], argv[2])) {
            seed = strtoull(argv[3], 0, 16);
            tuo_runners[i](seed, out);
            printf("%s %llx = %llx %llx %llx\n", argv[2], seed, out[0], out[1], out[2]);
            for (int k = 0; k < tuo_nobjs(); k++) printf("  obj %s %llx size %llu\n", tuo_objname(k), tuo_objhash(k), tuo_objsize(k));
            return 0;
        }
        return 2;
    }
    while (scanf("%255s %llx", name, &seed) == 2) {
        int i, ok = 0;
        for (i = 0; i < tuo_nfuncs; i++) if (!strcmp(tuo_names[i], name)) { ok = 1; break; }
        if (!ok) { fprintf(stderr, "no function %s\n", name); return 1; }
        tuo_runners[i](seed, out);
        printf("%s %llx = %llx %llx %llx\n", name, seed, out[0], out[1], out[2]);
    }
    return 0;
}
'''


def build_and_run(stem, tg, cdir):
    CL = os.environ.get("CLANG", r"C:\Program Files\LLVM\bin\clang.exe")
    GALE = g.GALE
    F = ["--target=x86_64-pc-windows-gnu", "-mno-ms-bitfields", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT",
         "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-mlong-double-64", "-fno-strict-aliasing", "-fwrapv",
         "-fcommon", "-fgnu89-inline", "-ftrivial-auto-var-init=zero", "-ffp-contract=on", "-O2", "-Xclang",
         "-disable-llvm-passes", "-S", "-emit-llvm", "-w", "-I.", "-Isrc", "-Ipc", "-Ipc/gameworld", "-isystem",
         "src/MSL", "-isystem", "libs/dolphin/include", "-isystem", "libs/dolphin/src", "-isystem", GALE,
         "-include", "src/MSL/math_ppc.h", "-D_Static_assert(a,b)=", "-Dsize_t=gc_size_t",
         "-I" + str((ROOT / tg.src).parent)]
    ll = cdir / "wrap.ll"
    r = subprocess.run([CL, *F, str(cdir / "wrap.c"), "-o", str(ll)], cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        print(r.stderr[:6000])
        raise SystemExit("wrap.c did not compile")
    lo = cdir / "wrap.lo.ll"
    subprocess.run([sys.executable, str(ROOT / "rust/melee_runtime/oracle/lower.py"), str(ll), str(lo)], check=True)
    (cdir / "main.c").write_text(driver_main_c(stem))
    exe = cdir / "oracle.exe"
    r = subprocess.run([CL, "-O1", "-w", str(cdir / "main.c"), str(lo), "-o", str(exe), "-Wl,/STACK:268435456", "-Wl,/OPT:NOICF"], cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        print(r.stderr[:6000])
        raise SystemExit("link failed")
    return exe


def main():
    a = sys.argv[1:]
    stem = a[0]
    from cfgs import CFGS
    cfg = CFGS[stem]
    tg = TuGen(stem, cfg)
    c, r = tg.build()
    cdir = OUT_ROOT / stem
    cdir.mkdir(parents=True, exist_ok=True)
    (cdir / "wrap.c").write_text(c, newline="\n")
    (ROOT / f"rust/melee_modes/tests/tu_{stem}.rs").write_text(r.replace(".__", "._x0"), newline="\n")
    # regenerate runner Rust after callbacks known (emit_c ran first, emit_rust second: both complete)
    print(f"{stem}: {len(tg.test_funcs)} functions, {len(tg.objs)} state objects, {len(tg.ext)} external functions mocked")
    # ext signatures for the port file
    lines = []
    for s in tg.ext:
        fn = s["fn"]
        args = ", ".join(f"a{i}: {tg.g2.rty(x.type)}" for i, x in enumerate(fn.get_arguments()))
        rk = classify(fn.result_type)
        ret = "" if rk[0] == "void" else f" -> {tg.g2.rty(fn.result_type)}"
        lines.append(f"    fn {fn.spelling}({args}){ret};")
    (cdir / "ext_decls.rs.txt").write_text("\n".join(lines) + "\n")
    if "--build" in a:
        exe = build_and_run(stem, tg, cdir)
        inp = []
        for n in tg.test_funcs:
            cases = tg.fcfg.get(n, {}).get("cases", cfg.get("cases", 60))
            for i in range(cases):
                seed = (int.from_bytes(n.encode()[:8].ljust(8, b"\0"), "little") * 0x9E3779B97F4A7C15 + i * 0x1234567) & ((1 << 64) - 1)
                inp.append(f"{n} {seed:x}")
        # one process per function: a function whose C side faults on the synthetic state is reported and left out
        # of the data (the Rust test only runs the functions that have rows)
        out_lines = []
        for n in tg.test_funcs:
            fin = [x for x in inp if x.split()[0] == n]
            try:
                res1 = subprocess.run([str(exe)], input=chr(10).join(fin) + chr(10), capture_output=True, text=True, cwd=ROOT, timeout=300)
            except subprocess.TimeoutExpired:
                print(f"tuoracle: C side of {n} hangs: left out")
                continue
            if res1.returncode:
                print(f"tuoracle: C side of {n} faults (rc={res1.returncode:#x}): left out")
                continue
            out_lines.append(res1.stdout)
        (ROOT / f"rust/melee_modes/tests/data/tu_{stem}.txt").write_text("".join(out_lines), newline=chr(10))
        print(f"{stem}: {len(inp)} C cases written")
        tr = subprocess.run([str(exe), "--tables"], capture_output=True, text=True, cwd=ROOT)
        if tr.returncode == 0 and tr.stdout.strip():
            write_tables(tg, tr.stdout)


if __name__ == "__main__":
    main()
