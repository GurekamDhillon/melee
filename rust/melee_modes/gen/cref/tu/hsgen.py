#!/usr/bin/env python3
"""Generate the by-value state hash / fill helpers for a TU oracle, in C and Rust, from one parse.

  python rust/melee_modes/gen/cref/tu/hsgen.py <stem>          (worktree root; Windows, libclang)

Reads `/* HS: Root1 Root2 ... */` from gen/cref/tu/<stem>.c (names of C records: tag or typedef)
and writes, for each root and every named record reachable from it,
  hs_<Rec>(const Rec*, u32 h) -> u32     fold every field into the hash BY VALUE
  fl_<Rec>(Rec*)                         fill every field from the shared xorshift (rnd/rg)
to gen/cref/tu/hs_<stem>.h (C) and tests/tu_<stem>_hs/mod.rs (Rust, over melee_types).

By value because the C records of this port carry bitfields: melee_types keeps them in GameCube
byte order (accessors), the native C compiler in its own, so a byte image would differ. Pointer
fields hash as null / non-null and are never filled; scalars hash as u32 words (signed types
sign-extended, 64-bit types as two words, floats as bits); arrays by element; unions and
anonymous members as raw bytes at their native offset (never filled). The C parse is
x86_64-windows-gnu with -mno-ms-bitfields, the layout run_tu.py compiles with, whose offsets are
melee_types' (rust/melee_types/gen/gen_types.py).
"""
import re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
import gen_types as g  # noqa: E402

ci, K, T = g.ci, g.K, g.T
import os  # noqa: E402

os.chdir(ROOT)

INTK = {T.CHAR_S: (1, True), T.SCHAR: (1, True), T.CHAR_U: (1, False), T.UCHAR: (1, False),
        T.SHORT: (2, True), T.USHORT: (2, False), T.INT: (4, True), T.UINT: (4, False),
        T.LONG: (4, True), T.ULONG: (4, False), T.BOOL: (1, False),
        T.LONGLONG: (8, True), T.ULONGLONG: (8, False)}


GEN_ = object.__new__(g.Gen)


class Ctx:
    def __init__(self, triple):
        args = [a.replace("x86_64-pc-windows-msvc", triple) for a in g.NAT] + ["-mno-ms-bitfields"]
        self.tu = g.parse(args)
        self.recs = {}      # (file,line,col) -> definition cursor
        self.byname = {}    # tag or typedef name -> definition cursor
        self.anon_td = {}   # loc -> typedef name of an anonymous record
        self.walk(self.tu.cursor)
        for c in self.tu.cursor.get_children():
            if c.kind == K.TYPEDEF_DECL:
                u = c.underlying_typedef_type
                while u.kind == T.ELABORATED:
                    u = u.get_named_type()
                if u.kind == T.RECORD:
                    d = u.get_declaration().get_definition()
                    if d is not None:
                        self.byname.setdefault(c.spelling, d)
                        if self.is_anon(d):
                            self.anon_td.setdefault(g.loc(d), c.spelling)

    def walk(self, c):
        for ch in c.get_children():
            if ch.kind in (K.STRUCT_DECL, K.UNION_DECL) and ch.is_definition():
                self.recs.setdefault(g.loc(ch), ch)
                if not self.is_anon(ch):
                    self.byname.setdefault(ch.spelling, ch)
                self.walk(ch)

    @staticmethod
    def is_anon(c):
        return g.Gen.is_anon(c)

    def name(self, d):
        """Rust name of a record == the C name used in generated C (struct tag, else typedef)."""
        if not self.is_anon(d):
            return d.spelling
        return self.anon_td.get(g.loc(d))

    def cname(self, d):
        n = self.name(d)
        return ("union " if d.kind == K.UNION_DECL else "struct ") + n if not self.is_anon(d) else n


def has_ptr_deep(d, seen=None):
    """Does record d (or a record nested in it) hold a pointer? Its byte image then differs by layout."""
    seen = seen or set()
    if g.loc(d) in seen:
        return False
    seen.add(g.loc(d))
    for f in d.get_children():
        if f.kind in (K.STRUCT_DECL, K.UNION_DECL) and f.is_anonymous_record_decl():
            if has_ptr_deep(f, seen):
                return True
        if f.kind == K.FIELD_DECL:
            t = f.type.get_canonical()
            while t.kind in (T.CONSTANTARRAY, T.INCOMPLETEARRAY):
                t = t.element_type.get_canonical()
            if t.kind == T.POINTER:
                return True
            if t.kind == T.RECORD:
                dd = t.get_declaration().get_definition()
                if dd is not None and has_ptr_deep(dd, seen):
                    return True
    return False


def classify(t):
    """('i', size, signed) | ('f', size) | ('p',) | ('a', count, elem) | ('r', decl) | ('x',)"""
    c = t.get_canonical()
    if c.kind in INTK:
        sz, sg = INTK[c.kind]
        return ("i", sz, sg)
    if c.kind == T.ENUM:
        sz, sg = INTK.get(c.get_declaration().enum_type.get_canonical().kind, (4, True))
        return ("i", sz, sg)
    if c.kind in (T.FLOAT,):
        return ("f", 4)
    if c.kind in (T.DOUBLE, T.LONGDOUBLE):
        return ("f", 8)
    if c.kind == T.POINTER:
        return ("p",)
    if c.kind == T.CONSTANTARRAY:
        return ("a", c.element_count, c.element_type)
    if c.kind == T.RECORD:
        return ("r", c.get_declaration().get_definition())
    return ("x",)


def main(stem):
    src = (HERE / f"{stem}.c").read_text(encoding="utf-8")
    roots = []
    for m in re.finditer(r"/\*\s*HS:([^*]*)\*/", src):
        roots += m.group(1).split()
    if not roots:
        return
    ctx = Ctx("i686-pc-windows-gnu")    # the C oracle's layout (4-byte pointers, as on the GC)
    ctx64 = Ctx("x86_64-pc-windows-gnu")  # melee_types' native layout (the Rust side)
    order, seen = [], set()

    def need(d):
        if d is None or g.loc(d) in seen or d.kind != K.STRUCT_DECL or ctx.name(d) is None:
            return
        seen.add(g.loc(d))
        for f in d.get_children():
            if f.kind == K.FIELD_DECL and not f.is_bitfield():
                t = f.type
                k = classify(t)
                while k[0] == "a":
                    k = classify(k[2])
                if k[0] == "r":
                    need(k[1])
        order.append(d)

    for r in roots:
        d = ctx.byname.get(r)
        if d is None:
            raise SystemExit(f"hsgen: no record {r}")
        need(d)

    ch = ["/* generated by hsgen.py from the HS line of " + stem + ".c: by-value hash + fill */",
          "static u32 hm_(u32 h, u32 v) { return (h * 0x01000193u) ^ v; }",
          "static u32 hmb_(u32 h, const void* p, unsigned n) { const unsigned char* b = p; while (n--) h = hm_(h, *b++); return h; }"]
    rs = ["// generated by hsgen.py from the `/* HS: */` line of gen/cref/tu/" + stem + ".c: by-value hash + fill",
          "#![allow(non_snake_case, unused_variables, unused_mut, unused_parens, dead_code, unused_imports, clippy::all)]",
          "use crate::tu_gm_harness::{rg, rnd};",
          "use melee_types::*;",
          "pub fn hm(h: u32, v: u32) -> u32 { h.wrapping_mul(0x0100_0193) ^ v }",
          "pub fn hmb(mut h: u32, p: *const u8, n: usize) -> u32 { for i in 0..n { h = hm(h, unsafe { *p.add(i) } as u32); } h }"]

    def emit_val(k, ce, re_, h_c, h_r, depth, out_c, out_r, fill_c, fill_r, d_off, f_off, nat_size):
        """Append hash + fill statements for a value expression (C expr ce, Rust expr re_)."""
        t = k[0]
        if t == "i":
            sz, sg = k[1], k[2]
            if sz <= 4:
                out_c.append(f"h = hm_(h, (u32){'(s32)' if sg else ''}{ce});")
                out_r.append(f"h = hm(h, ({re_}){' as i32' if sg else ''} as u32);")
                fill_c.append(f"{ce} = rnd();")
                fill_r.append(f"{re_} = rnd() as _;")
            else:
                out_c.append(f"h = hm_(h, (u32)(u64){ce}); h = hm_(h, (u32)((u64){ce} >> 32));")
                out_r.append(f"h = hm(h, ({re_}) as u64 as u32); h = hm(h, (({re_}) as u64 >> 32) as u32);")
                fill_c.append(f"{{ u32 lo_ = rnd(); u32 hi_ = rnd(); {ce} = ((u64) hi_ << 32) | lo_; }}")
                fill_r.append(f"{{ let lo = rnd() as u64; let hi = rnd() as u64; {re_} = ((hi << 32) | lo) as _; }}")
        elif t == "f":
            if k[1] == 4:
                out_c.append(f"h = hm_(h, B({ce}));")
                out_r.append(f"h = hm(h, ({re_}).to_bits());")
                fill_c.append(f"{ce} = rg();")
                fill_r.append(f"{re_} = rg();")
            else:
                out_c.append(f"{{ union {{ f64 f; u64 u; }} c_; c_.f = {ce}; h = hm_(h, (u32) c_.u); h = hm_(h, (u32) (c_.u >> 32)); }}")
                out_r.append(f"{{ let u = ({re_}).to_bits(); h = hm(h, u as u32); h = hm(h, (u >> 32) as u32); }}")
                fill_c.append(f"{ce} = rg();")
                fill_r.append(f"{re_} = rg() as f64;")
        elif t == "p":
            out_c.append(f"h = hm_(h, {ce} != 0);")
            out_r.append(f"h = hm(h, (unsafe {{ core::ptr::read(&{re_} as *const _ as *const usize) }} != 0) as u32);")
        elif t == "a":
            n = k[1]
            v = f"i{depth}"
            ek = classify(k[2])
            sub_c, sub_r, sf_c, sf_r = [], [], [], []
            emit_val(ek, f"{ce}[{v}]", f"{re_}[{v}]", h_c, h_r, depth + 1, sub_c, sub_r, sf_c, sf_r, d_off, f_off, nat_size)
            out_c.append(f"{{ int {v}; for ({v} = 0; {v} < {n}; {v}++) {{ " + " ".join(sub_c) + " } }")
            out_r.append(f"for {v} in 0..{n} {{ " + " ".join(sub_r) + " }")
            if sf_c:
                fill_c.append(f"{{ int {v}; for ({v} = 0; {v} < {n}; {v}++) {{ " + " ".join(sf_c) + " } }")
                fill_r.append(f"for {v} in 0..{n} {{ " + " ".join(sf_r) + " }")
        elif t == "r":
            d = k[1]
            n = ctx.name(d)
            if d.kind == K.STRUCT_DECL and n is not None:
                out_c.append(f"h = hs_{n}(&{ce}, h);")
                out_r.append(f"h = hs_{n}(&{re_}, h);")
                fill_c.append(f"fl_{n}(&{ce});")
                fill_r.append(f"fl_{n}(&mut {re_});")
            else:
                out_c.append(f"h = hmb_(h, &{ce}, sizeof({ce}));")
                out_r.append(f"h = hmb(h, &{re_} as *const _ as *const u8, core::mem::size_of_val(&{re_}));")
        else:
            out_c.append(f"h = hmb_(h, &{ce}, sizeof({ce}));")
            out_r.append(f"h = hmb(h, &{re_} as *const _ as *const u8, core::mem::size_of_val(&{re_}));")

    for d in order:
        n = ctx.name(d)
        cn = ctx.cname(d)
        hc, hr, fc, fr = [], [], [], []
        for f in d.get_children():
            if f.kind in (K.STRUCT_DECL, K.UNION_DECL) and f.is_anonymous_record_decl():
                # C11 anonymous member: raw bytes at its native offset (never filled); skipped when its
                # byte image differs by layout (bitfields: GC vs native bit order; pointers: width)
                if GEN_.has_bf_deep(f) or has_ptr_deep(f):
                    continue
                off = g.Gen.anon_offset(GEN_, d, f)
                sz = f.type.get_size()
                d64 = ctx64.byname.get(ctx.name(d))
                anon64 = [c for c in d64.get_children() if c.kind in (K.STRUCT_DECL, K.UNION_DECL) and c.is_anonymous_record_decl()]
                anon32 = [c for c in d.get_children() if c.kind in (K.STRUCT_DECL, K.UNION_DECL) and c.is_anonymous_record_decl()]
                off64 = g.Gen.anon_offset(GEN_, d64, anon64[anon32.index(f)])
                hc.append(f"h = hmb_(h, (const unsigned char*) p + {off}, {sz});")
                hr.append(f"h = hmb(h, unsafe {{ (p as *const _ as *const u8).add({off64}) }}, {sz});")
                continue
            if f.kind != K.FIELD_DECL:
                continue
            name = f.spelling
            if not name:
                if f.is_bitfield():
                    continue
                continue
            rn = g.ident(name)
            if f.is_bitfield():
                canon = f.type.get_canonical()
                sg = INTK.get(canon.kind, (4, False))[1] if canon.kind != T.ENUM else True
                is_bool = canon.kind == T.BOOL
                hc.append(f"h = hm_(h, (u32){'(s32)' if sg and not is_bool else ''}p->{name});")
                if is_bool:
                    hr.append(f"h = hm(h, p.{rn}() as u32);")
                    fc.append(f"p->{name} = rnd() & 1;")
                    fr.append(f"p.set_{name}((rnd() & 1) != 0);")
                else:
                    hr.append(f"h = hm(h, p.{rn}(){' as i32' if sg else ''} as u32);")
                    fc.append(f"p->{name} = rnd();")
                    fr.append(f"p.set_{name}(rnd() as _);")
                continue
            k = classify(f.type)
            if k[0] == "r" and k[1] is not None and k[1].kind == K.UNION_DECL and (GEN_.has_bf_deep(k[1]) or has_ptr_deep(k[1])):
                continue
            if k[0] == "r" and (k[1] is None or k[1].kind == K.UNION_DECL or ctx.name(k[1]) is None):
                if k[1] is not None and (GEN_.has_bf_deep(k[1]) or has_ptr_deep(k[1])):
                    continue
                off = f.get_field_offsetof() // 8
                sz = f.type.get_size()
                d64 = ctx64.byname.get(ctx.name(d))
                f64 = next(c for c in d64.get_children() if c.kind == K.FIELD_DECL and c.spelling == name)
                off64 = f64.get_field_offsetof() // 8
                hc.append(f"h = hmb_(h, (const unsigned char*) p + {off}, {sz});")
                hr.append(f"h = hmb(h, unsafe {{ (p as *const _ as *const u8).add({off64}) }}, {sz});")
                continue
            emit_val(k, f"p->{name}", f"p.{rn}", None, None, 0, hc, hr, fc, fr, 0, 0, 0)
        ch.append(f"static u32 hs_{n}(const {cn}* p, u32 h) {{ " + " ".join(hc) + " return h; }")
        ch.append(f"static void fl_{n}({cn}* p) {{ " + " ".join(fc) + " }")
        rs.append(f"pub fn hs_{n}(p: &{n}, mut h: u32) -> u32 {{ " + " ".join(hr) + " h }")
        rs.append(f"pub fn fl_{n}(p: &mut {n}) {{ " + " ".join(fr) + " }")
    (HERE / f"hs_{stem}.h").write_text("\n".join(ch) + "\n", encoding="utf-8", newline="\n")
    outd = ROOT / f"rust/melee_modes/tests/tu_{stem}_hs"
    outd.mkdir(parents=True, exist_ok=True)
    (outd / "mod.rs").write_text("\n".join(rs) + "\n", encoding="utf-8", newline="\n")
    print(f"hsgen: {stem}: {len(order)} records")


if __name__ == "__main__":
    for s in sys.argv[1:]:
        main(s)
