#!/usr/bin/env python3
"""TU state oracle generator (group 6, shard 481-488).

  python rust/melee_world/oracle/tuo/gen_tuo.py <spec.py> <outdir>      (from the worktree root)

Given a spec (the C file, which functions to run, value domains, pointer links) this writes into
<outdir>:
  unit.c      `#include`s the game TU verbatim and adds visitors for every record type, globals,
              the call glue. Compiled with the shipping flags (contraction on) + lower.py.
  stubs.c     one logging stub per function the TU calls but does not define (plain C types).
  tuo.cfg     the case table + value domains + links (shared with the Rust test).
  <tu>_gen.rs the Rust twin: visitors, stubs, glue (tests/tuo_gen/).
  ext.rs.txt  the `pub mod ext` text for the port (callee hooks and extern globals).
See tuo_rt.c for the protocol. Needs Windows Python + libclang==18.1.1 (like gen_types.py).
"""
import importlib.util, os, re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
os.chdir(ROOT)
import gen_types as g  # noqa: E402

ci, K, T = g.ci, g.K, g.T
NAT = [a.replace("x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu") for a in g.NAT] + ["-mno-ms-bitfields"]
CL = os.environ.get("CLANG", r"C:\Program Files\LLVM\bin\clang.exe")
GEN_RS = (ROOT / "rust/melee_types/src/gen.rs").read_text(encoding="utf-8")

RUST_KW = g.RUST_KW
INTK = {T.CHAR_S: (8, 1), T.SCHAR: (8, 1), T.CHAR_U: (8, 0), T.UCHAR: (8, 0), T.SHORT: (16, 1),
        T.USHORT: (16, 0), T.INT: (32, 1), T.UINT: (32, 0), T.LONG: (32, 1), T.ULONG: (32, 0),
        T.LONGLONG: (64, 1), T.ULONGLONG: (64, 0), T.BOOL: (8, 0)}
RINT = {(8, 1): "i8", (8, 0): "u8", (16, 1): "i16", (16, 0): "u16", (32, 1): "i32", (32, 0): "u32",
        (64, 1): "i64", (64, 0): "u64"}
LIBC = {"sprintf", "snprintf", "memcpy", "memset", "memmove", "strcmp", "strlen", "strcpy", "strncpy",
        "strcat", "strncmp", "memcmp", "abs", "fabs", "sqrt", "sqrtf", "floorf", "ceilf", "sinf", "cosf",
        "atan2f", "fabsf", "malloc", "free", "printf", "vsprintf"}


def rid(n):
    return "r#" + n if n in RUST_KW else n


# ---------- melee_types introspection (Rust side names) ----------
RS_STRUCTS = {}
for m in re.finditer(r"^pub (?:struct|union) (\w+) \{\n(.*?)^\}", GEN_RS, re.M | re.S):
    flds = []
    for fm in re.finditer(r"^    pub (r#)?(\w+): (.+?),$", m.group(2), re.M):
        flds.append((fm.group(2), fm.group(3)))
    RS_STRUCTS[m.group(1)] = flds
RS_ALIAS = dict(re.findall(r"^pub type (\w+) = (\w+);", GEN_RS, re.M))
RS_NAMES = set(re.findall(r"^pub (?:struct|union|type|enum) (\w+)", GEN_RS, re.M))


def rs_elem(ty):
    """Rust type string with array brackets stripped."""
    while ty.startswith("["):
        ty = ty[1:ty.rindex(";")]
    return ty.strip()


class Spec:
    def __init__(self, path):
        sp = importlib.util.spec_from_file_location("spec", path)
        m = importlib.util.module_from_spec(sp)
        sp.loader.exec_module(m)
        self.m = m

    def get(self, k, d=None):
        return getattr(self.m, k, d)


# ---------- C type helpers ----------
def strip(t):
    while t.kind == T.ELABORATED:
        t = t.get_named_type()
    return t


def canon(t):
    return t.get_canonical()


def is_char_ptr(t):
    c = canon(t)
    if c.kind != T.POINTER:
        return False
    p = canon(c.get_pointee())
    return p.kind == T.CHAR_S  # plain `char*` only (u8*/s8* are buffers, not strings)


class Ctx:
    def __init__(self, spec):
        self.spec = spec
        self.tu_path = spec.get("TU")
        self.records = {}  # c name -> dict(cursor, rname)
        self.queue = []

    # name of a named record type as C spells it, and the Rust name
    def rec_names(self, t):
        t = strip(t)
        decl = canon(t).get_declaration()
        cname = None
        if t.kind == T.TYPEDEF:
            cname = t.spelling
        elif decl.spelling and not decl.is_anonymous_record_decl() and not decl.spelling.startswith("(unnamed"):
            cname = ("union " if decl.kind == K.UNION_DECL else "struct ") + decl.spelling
        return decl, cname

    def rust_rec(self, decl, cname):
        over = self.spec.get("RUST_TYPES", {})
        key = cname.split(" ")[-1] if cname else None
        if key in over:
            return over[key]
        cands = []
        if cname:
            cands.append(cname.split(" ")[-1])
        if decl.spelling:
            cands.append(decl.spelling)
        for c in cands:
            if c in RS_NAMES:
                return "melee_types::" + c
        return None


# ---------- type walking: emit C and Rust visitor statements together ----------
class Opaque(Exception):
    pass


class Emit:
    """Statement lists for one visitor body."""

    def __init__(self):
        self.c = []
        self.r = []


class Walker:
    def __init__(self, cx):
        self.cx = cx
        self.nbuf = 0
        self.prune = None   # list of allowed trails ("a.b[3].c") while expanding a pruned root type
        self.trail = ""

    def allowed(self, tr):
        if self.prune is None:
            return True
        for a in self.prune:
            if a == tr or a.startswith(tr + ".") or a.startswith(tr + "[") or tr.startswith(a + ".") or tr.startswith(a + "["):
                return True
        return False

    def fixed_indices(self):
        """Array indices named by the prune list for the array at the current trail (or None)."""
        if self.prune is None:
            return None
        idx = set()
        for a in self.prune:
            if a.startswith(self.trail + "[") and self.trail:
                idx.add(int(a[len(self.trail) + 1:].split("]")[0]))
            elif a == self.trail or a.startswith(self.trail + "."):
                return None
        return sorted(idx) if idx else None

    def leaf(self, e, t, cx_, rx_, path_c, path_r, rtype_hint=None):
        """Append the C and Rust statements for the value of canonical type t."""
        c = canon(t)
        k = c.kind
        if strip(t).kind == T.TYPEDEF and strip(t).spelling in self.cx.spec.get("SKIP_LEAF_TYPES", []):
            return
        if k in INTK or k == T.ENUM:
            if k == T.ENUM:
                u = canon(c.get_declaration().enum_type)
                bits, sg = INTK[u.kind]
            else:
                bits, sg = INTK[k]
                if k == T.BOOL:
                    bits, sg = 8, 0
            rt = RINT[(bits, sg)]
            e.c.append(f"if(tuo_mode==1) {cx_} = tuo_leaf_int({path_c}, {bits}, {sg}); else tuo_dump_{'i' if sg else 'u'}({path_c}, {cx_});")
            if k == T.BOOL:
                e.r.append(f"if v.set() {{ {rx_} = v.leaf_int({path_r}, {bits}, {sg}) != 0; }} else {{ v.dump_u({path_r}, ({rx_}) as u64); }}")
            else:
                cv = "i64" if sg else "u64"
                e.r.append(f"if v.set() {{ {rx_} = v.leaf_int({path_r}, {bits}, {sg}) as _; }} else {{ v.dump_{'i' if sg else 'u'}({path_r}, (({rx_}) as {rt}) as {cv}); }}")
            return
        if k == T.FLOAT:
            e.c.append(f"if(tuo_mode==1) {{ unsigned _b = tuo_leaf_f32({path_c}); __builtin_memcpy(&{cx_}, &_b, 4); }} else {{ unsigned _b; __builtin_memcpy(&_b, &{cx_}, 4); tuo_dump_f32({path_c}, _b); }}")
            e.r.append(f"if v.set() {{ {rx_} = f32::from_bits(v.leaf_f32({path_r})); }} else {{ v.dump_f32({path_r}, ({rx_}).to_bits()); }}")
            return
        if k in (T.DOUBLE, T.LONGDOUBLE):
            e.c.append(f"if(tuo_mode==1) {{ unsigned long long _b = tuo_leaf_f64({path_c}); double _d; __builtin_memcpy(&_d, &_b, 8); {cx_} = _d; }} else {{ double _d = {cx_}; unsigned long long _b; __builtin_memcpy(&_b, &_d, 8); tuo_dump_f64({path_c}, _b); }}")
            e.r.append(f"if v.set() {{ {rx_} = f64::from_bits(v.leaf_f64({path_r})); }} else {{ v.dump_f64({path_r}, ({rx_}).to_bits()); }}")
            return
        if k == T.POINTER:
            pt = canon(c.get_pointee())
            if pt.kind in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO):
                e.c.append(f"if(tuo_mode==1) {cx_} = (__typeof__({cx_})) tuo_leaf_ptr({path_c}); else tuo_dump_p({path_c}, (void*) {cx_});")
                e.r.append(f"if v.set() {{ {rx_} = unsafe {{ core::mem::transmute::<usize, _>(v.leaf_ptr({path_r})) }}; }} else {{ v.dump_p({path_r}, unsafe {{ core::mem::transmute_copy::<_, usize>(&{rx_}) }}); }}")
            else:
                # PTR_MASK = {record: [field]}: the pointer's low byte is aliased by a bitfield of another union view
                # (b0 is bit 7 in melee_types / on the GameCube, bit 0 in the native C): bits 0 and 7 are not compared
                masked = getattr(self, "rec", "") in self.cx.spec.get("PTR_MASK", {}) and cx_.rsplit(".", 1)[-1] in self.cx.spec.get("PTR_MASK", {})[self.rec]
                if masked:
                    e.c.append(f"if(tuo_mode==1) {cx_} = (__typeof__({cx_})) tuo_leaf_ptr({path_c}); else tuo_dump_p({path_c}, (const void*) ((unsigned long long) {cx_} & ~0x81ull));")
                    e.r.append(f"if v.set() {{ {rx_} = v.leaf_ptr({path_r}) as _; }} else {{ v.dump_p({path_r}, (({rx_}) as usize) & !0x81usize); }}")
                else:
                    e.c.append(f"if(tuo_mode==1) {cx_} = (__typeof__({cx_})) tuo_leaf_ptr({path_c}); else tuo_dump_p({path_c}, (const void*) {cx_});")
                    e.r.append(f"if v.set() {{ {rx_} = v.leaf_ptr({path_r}) as _; }} else {{ v.dump_p({path_r}, ({rx_}) as usize); }}")
            return
        if k == T.CONSTANTARRAY:
            n = c.get_array_size()
            st_ = strip(t)
            elt = st_.element_type if st_.kind == T.CONSTANTARRAY else c.element_type
            el = canon(elt)
            if el.kind in INTK and INTK[el.kind][0] == 8:
                e.c.append(f"tuo_bytes({path_c}, &{cx_}, sizeof({cx_}));")
                e.r.append(f"v.bytes({path_r}, &mut {rx_} as *mut _ as *mut u8, core::mem::size_of_val(&{rx_}));")
                return
            fixed = self.fixed_indices()
            if fixed is not None:
                for ix in fixed:
                    self.nbuf += 1
                    nb = self.nbuf
                    i = f"_i{nb}"
                    sub = Emit()
                    base = self.trail
                    self.trail = f"{base}[{ix}]"
                    self.leaf(sub, elt, f"{cx_}[{ix}]", f"{rx_}[{ix}]",
                              f"tuo_cati(_pb{nb}, {path_c}, \"\", {ix})", f"&format!(\"{{}}[{{}}]\", {self.strip_amp(path_r)}, {ix})", rtype_hint)
                    self.trail = base
                    e.c.append(f"{{ char _pb{nb}[600];")
                    e.c.extend(sub.c)
                    e.c.append("}")
                    e.r.append("{")
                    e.r.extend(sub.r)
                    e.r.append("}")
                return
            self.nbuf += 1
            nb = self.nbuf
            i = f"_i{nb}"
            sub = Emit()
            base = self.trail
            self.trail = base + "[]" if self.prune is not None else base
            self.leaf(sub, elt, f"{cx_}[{i}]", f"{rx_}[{i}]",
                      f"tuo_cati(_pb{nb}, {path_c}, \"\", {i})", f"&format!(\"{{}}[{{}}]\", {self.strip_amp(path_r)}, {i})", rtype_hint)
            self.trail = base
            e.c.append(f"{{ char _pb{nb}[600]; int {i}; for({i}=0;{i}<{n};{i}++) {{")
            e.c.extend(sub.c)
            e.c.append("} }")
            e.r.append(f"for {i} in 0..{n}usize {{")
            e.r.extend(sub.r)
            e.r.append("}")
            return
        if k == T.RECORD:
            decl = c.get_declaration()
            if decl.kind == K.UNION_DECL:
                uv = self.cx.spec.get("UNION_VIEW", {})
                _d, cname_ = self.cx.rec_names(t)
                key_ = (cname_ or "").split(" ")[-1]
                if key_ not in uv:
                    # anonymous union: keyed by "<enclosing record>.<field>"
                    key_ = f"{getattr(self, 'rec', '')}.{self.trail.split('.')[-1].split('[')[0]}"
                if key_ in uv:
                    # compare the union through one or more members (each gets domains/links like any record);
                    # UNION_VIEW[key] = "member" | {"member": [functions it is active for ([] = all)]}
                    spec_ = uv[key_]
                    if isinstance(spec_, str):
                        spec_ = {spec_: []}
                    for m, _fns in spec_.items():
                        p_c = f'tuo_cat(_pb, {path_c}, ".{m}")'
                        p_r = f'&format!("{{}}.{m}", {self.strip_amp(path_r)})'
                        sub = Emit()
                        if m.startswith("anon:"):
                            # the union's anonymous struct member: its members are members of the union in C;
                            # the Rust side reaches it as the field named after the colon
                            an_ = m[5:]
                            afld = next(f for f in (decl.get_definition() or decl).get_children()
                                        if f.kind == K.STRUCT_DECL and f.is_anonymous_record_decl())
                            self.record_fields(sub, afld, cx_, f"{rx_}.{an_}", p_c, p_r, None)
                        else:
                            fld = next(f for f in (decl.get_definition() or decl).get_children()
                                       if f.kind == K.FIELD_DECL and f.spelling == m)
                            self.leaf_field(sub, fld.type, f"{cx_}.{m}", f"{rx_}.{rid(m)}", p_c, p_r, None)
                        e.c.append(f'if (tuo_viewon("{key_}", "{m}")) {{')
                        e.c.extend(sub.c)
                        e.c.append("}")
                        e.r.append(f'if v.view_on("{key_}", "{m}") {{')
                        e.r.extend(sub.r)
                        e.r.append("}")
                    return
                e.c.append(f"tuo_bytes({path_c}, &{cx_}, sizeof({cx_}));")
                e.r.append(f"v.bytes({path_r}, &mut {rx_} as *mut _ as *mut u8, core::mem::size_of_val(&{rx_}));")
                return
            d, cname = self.cx.rec_names(t)
            rname = self.cx.rust_rec(d, cname) if cname else None
            if cname and rname and self.prune is not None:
                self.record_fields(e, d.get_definition() or d, cx_, rx_, path_c, path_r, rname)
                return
            if cname and rname:
                self.cx.need(cname, d, rname)
                e.c.append(f"vis_{self.mangle(cname)}({path_c}, &{cx_});")
                e.r.append(f"vis_{self.mangle(cname)}(v, {path_r}, &mut {rx_} as *mut _ as *mut u8);")
                return
            # anonymous / unnamed here: inline
            d = d.get_definition() or d
            self.record_fields(e, d, cx_, rx_, path_c, path_r, rtype_hint)
            return
        raise SystemExit(f"tuo: unsupported type {t.spelling} ({k})")

    @staticmethod
    def strip_amp(p):
        return p[1:] if p.startswith("&") else p

    @staticmethod
    def mangle(cname):
        return re.sub(r"\W", "_", cname)

    def record_fields(self, e, decl, cx_, rx_, path_c, path_r, rname):
        """Fields of record `decl` (inline). cx_/rx_ are the C / Rust places of the record."""
        prev_rec = getattr(self, "rec", "")
        if decl.spelling and not decl.spelling.startswith("(unnamed"):
            self.rec = decl.spelling
        try:
            return self._record_fields(e, decl, cx_, rx_, path_c, path_r, rname)
        finally:
            self.rec = prev_rec

    def _record_fields(self, e, decl, cx_, rx_, path_c, path_r, rname):
        fields = [f for f in decl.get_children() if f.kind == K.FIELD_DECL or
                  (f.kind in (K.STRUCT_DECL, K.UNION_DECL) and f.is_anonymous_record_decl())]
        rkey = rname.split("::")[-1] if rname else None
        while rkey in RS_ALIAS and rkey not in RS_STRUCTS:
            rkey = RS_ALIAS[rkey]
        rfields = list(RS_STRUCTS.get(rkey, [])) if rname and rname.startswith("melee_types::") else None
        anon_i = 0
        ranon = [n for (n, ty) in (rfields or []) if n.startswith("anon_")]
        rtys = dict(rfields or [])
        skip_bits = [b for k, bs in self.cx.spec.get("SKIP_BITS", {}).items() if k in getattr(self, "rec", "") for b in bs]  # key = substring of the record name
        for f in fields:
            if f.kind == K.FIELD_DECL and f.spelling in skip_bits:
                continue  # SKIP_BITS {record: [bitfield names]}: not generated/compared (aliased native bit order)
            if f.kind != K.FIELD_DECL:  # C11 anonymous member
                an = ranon[anon_i] if rfields is not None else None
                anon_i += 1
                if f.kind == K.UNION_DECL and f"{decl.spelling}.anon{anon_i - 1}" in self.cx.spec.get("UNION_VIEW", {}):
                    key_ = f"{decl.spelling}.anon{anon_i - 1}"
                    spec_ = self.cx.spec.get("UNION_VIEW", {})[key_]
                    if isinstance(spec_, str):
                        spec_ = {spec_: []}
                    for m, _fns in spec_.items():
                        fld = next(c for c in f.get_children() if c.kind == K.FIELD_DECL and c.spelling == m)
                        p_c = f'tuo_cat(_pb, {path_c}, ".{an}.{m}")'
                        p_r = f'&format!("{{}}.{an}.{m}", {self.strip_amp(path_r)})'
                        sub = Emit()
                        self.leaf_field(sub, fld.type, f"{cx_}.{m}", f"{rx_}.{an}.{rid(m)}", p_c, p_r, None)
                        e.c.append(f'if (tuo_viewon("{key_}", "{m}")) {{')
                        e.c.extend(sub.c)
                        e.c.append("}")
                        e.r.append(f'if v.view_on("{key_}", "{m}") {{')
                        e.r.extend(sub.r)
                        e.r.append("}")
                elif f.kind == K.UNION_DECL:
                    # bytes of the union: address from the Rust side; the C side takes the
                    # first named member (a bitfield first member cannot be addressed)
                    first = next((c for c in f.get_children() if c.kind == K.FIELD_DECL and c.spelling and not c.is_bitfield()), None)
                    if first is None or an is None:
                        raise SystemExit(f"tuo: cannot address anonymous union in {decl.spelling}")
                    p_c = f'tuo_cat(_pb, {path_c}, ".{an}")'
                    p_r = f'&format!("{{}}.{an}", {self.strip_amp(path_r)})'
                    e.c.append(f"tuo_bytes({p_c}, &{cx_}.{first.spelling}, {f.type.get_size()});")
                    e.r.append(f"v.bytes({p_r}, &mut {rx_}.{an} as *mut _ as *mut u8, core::mem::size_of_val(&{rx_}.{an}));")
                else:
                    # anonymous struct: its members are members of the parent in C
                    sub_r = rs_elem(rtys[an]) if an else None
                    self.record_fields(e, f, cx_, f"{rx_}.{an}", path_c, path_r, "melee_types::" + sub_r if sub_r else None)
                continue
            nm = f.spelling
            if nm and not self.allowed((self.trail + "." if self.trail else "") + nm):
                continue
            if f.is_bitfield():
                if not nm:
                    continue
                bt = canon(f.type)
                bits = f.get_bitfield_width()
                sg = INTK.get(bt.kind, (32, 0))[1] if bt.kind != T.ENUM else 0
                if bt.kind == T.ENUM:
                    sg = INTK[canon(bt.get_declaration().enum_type).kind][1]
                p_c = f'tuo_cat(_pb, {path_c}, ".{nm}")'
                p_r = f'&format!("{{}}.{nm}", {self.strip_amp(path_r)})'
                cv = "i64" if sg else "u64"
                isbool = bt.kind == T.BOOL
                n0c, n0r = len(e.c), len(e.r)
                e.c.append(f"if(tuo_mode==1) {cx_}.{nm} = tuo_leaf_int({p_c}, {bits}, {sg}); else tuo_dump_{'i' if sg else 'u'}({p_c}, {cx_}.{nm});")
                getter = f"{rx_}.{rid(nm)}()"
                if isbool:
                    e.r.append(f"if v.set() {{ {rx_}.set_{nm}(v.leaf_int({p_r}, {bits}, {sg}) != 0); }} else {{ v.dump_u({p_r}, ({getter}) as u64); }}")
                else:
                    e.r.append(f"if v.set() {{ {rx_}.set_{nm}(v.leaf_int({p_r}, {bits}, {sg}) as _); }} else {{ v.dump_{'i' if sg else 'u'}({p_r}, ({getter}) as {cv}); }}")
                ffb_ = [fn for k, fns in self.cx.spec.get("FIELD_FNS", {}).items() if k == f"{getattr(self, 'rec', '')}.{nm}" for fn in fns]
                if ffb_:
                    cond_ = "|".join(ffb_)
                    e.c.insert(n0c, f'if (tuo_fnin("{cond_}")) {{')
                    e.c.append("}")
                    e.r.insert(n0r, f'if v.fn_in("{cond_}") {{')
                    e.r.append("}")
                continue
            if not nm:
                continue
            ft = f.type
            p_c = f'tuo_cat(_pb, {path_c}, ".{nm}")'
            p_r = f'&format!("{{}}.{nm}", {self.strip_amp(path_r)})'
            rn = nm
            if rfields is not None:
                if nm not in rtys:
                    cand = [n for n in rtys if n.split("_")[0] == nm]
                    if not cand:
                        raise Opaque(f"Rust field {nm} missing in {rname}")
                    rn = cand[0]
                sub_rname = "melee_types::" + rs_elem(rtys[rn])
            else:
                sub_rname = None
            base = self.trail
            self.trail = (base + "." if base else "") + nm
            ff_ = [fn for k, fns in self.cx.spec.get("FIELD_FNS", {}).items() if k == f"{getattr(self, 'rec', '')}.{nm}" for fn in fns]
            if ff_:
                # FIELD_FNS {"<record>.<field>": [fns]}: the field is visited only while those functions run
                sub_ = Emit()
                self.leaf_field(sub_, ft, f"{cx_}.{nm}", f"{rx_}.{rid(rn)}", p_c, p_r, sub_rname)
                cond_ = "|".join(ff_)
                e.c.append(f'if (tuo_fnin("{cond_}")) {{')
                e.c.extend(sub_.c)
                e.c.append("}")
                e.r.append(f'if v.fn_in("{cond_}") {{')
                e.r.extend(sub_.r)
                e.r.append("}")
            else:
                self.leaf_field(e, ft, f"{cx_}.{nm}", f"{rx_}.{rid(rn)}", p_c, p_r, sub_rname)
            self.trail = base

    def leaf_field(self, e, ft, cx_, rx_, p_c, p_r, rname):
        # a path buffer per use: the C helpers return into _pb, which is reused sequentially, but
        # nested calls read it after the call returns, so every use gets its own copy
        e.c.append("{ char _pb[600];")
        e.r.append("{")
        sub = Emit()
        self.leaf(sub, ft, cx_, rx_, p_c, p_r, rname)
        e.c.extend(sub.c)
        e.r.extend(sub.r)
        e.c.append("}")
        e.r.append("}")


class Gen:
    def __init__(self, spec):
        self.spec = spec
        self.cx = Ctx(spec)
        self.cx.need = self.need
        self.w = Walker(self.cx)
        self.vis = {}   # cname -> (decl, rname)
        self.order = []

    def need(self, cname, decl, rname):
        if cname not in self.vis:
            d2 = decl.get_definition()
            if d2 is None:
                raise SystemExit(f"tuo: record {cname} has no definition")
            decl = d2
            if rname is None:
                raise SystemExit(f"tuo: record {cname} has no Rust name (add RUST_TYPES)")
            self.vis[cname] = (decl, rname)
            self.order.append(cname)

    def visitor(self, cname):
        decl, rname = self.vis[cname]
        e = Emit()
        mg = Walker.mangle(cname)
        self.w.prune = self.spec.get("PRUNE", {}).get(cname)
        self.w.trail = ""
        try:
            self.w.record_fields(e, decl, "(*p)", "(*p)", "path", "path", rname)
        except Opaque as ex:
            print(f"tuo: {cname} compared as bytes ({ex})")
            self.w.prune = None
            self.w.trail = ""
            e = Emit()
            e.c.append("tuo_bytes(path, p, sizeof(*p));")
            e.r.append("v.bytes(path, vp, core::mem::size_of_val(&*p));")
        c = [f"static void vis_{mg}(const char* path, void* vp) {{ {cname}* p = ({cname}*) vp; char _pb[600]; (void) _pb;"]
        c += e.c
        c.append("}")
        r = ["#[allow(unused_variables, unused_mut, unused_unsafe, unused_parens, non_snake_case)]",
             f"fn vis_{mg}(v: &mut V, path: &str, vp: *mut u8) {{ let p = vp as *mut {rname}; unsafe {{"]
        r += e.r
        r.append("} }")
        self.w.prune = None
        self.w.trail = ""
        return c, r


HOOK_MACRO = '''    macro_rules! hook {
        ($h:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $h: Option<unsafe extern "C" fn($($t),*) $(-> $r)?> = None;
            pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                match $h {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// Like `hook!`, but an unset hook calls an existing verified port.
    #[allow(unused_macros)]
    macro_rules! hook_or {
        ($h:ident, $name:ident, $port:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $h: Option<unsafe extern "C" fn($($t),*) $(-> $r)?> = None;
            pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                match $h {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
'''


def cfgn(cn):
    return cn.replace(' ', '~')


def run(cmd, **kw):
    r = subprocess.run(cmd, capture_output=True, text=True, **kw)
    if r.returncode:
        raise SystemExit(f"command failed: {' '.join(map(str, cmd))}\n{r.stderr[-3000:]}")
    return r.stdout


def main():
    spec = Spec(sys.argv[1])
    out = Path(sys.argv[2]).resolve()
    out.mkdir(parents=True, exist_ok=True)
    tu_path = spec.get("TU")
    tuname = Path(tu_path).stem
    # PATCH = [(old, new)]: textual fixes to the TU source for the NATIVE oracle only (PPC-isms such as
    # `(Ground*) ((s32) gp + 0)`, which truncates a 64-bit pointer); the patched copy is what is compiled
    incdir = (ROOT / tu_path).parent
    if spec.get("PATCH"):
        src = (ROOT / tu_path).read_text(encoding="utf-8", errors="replace")
        for o_, n_ in spec.get("PATCH"):
            if o_ not in src:
                raise SystemExit(f"tuo: PATCH text not found: {o_}")
            src = src.replace(o_, n_)
        pp = out / (tuname + "_patched.c")
        pp.write_bytes(src.replace(chr(13)+chr(10), chr(10)).encode("utf-8"))
        tu_path = pp.as_posix()
    (out / "incdir.txt").write_text(incdir.as_posix())
    (out / "extra_c.txt").write_text(" ".join(spec.get("EXTRA_C", [])))  # extra C sources linked into the oracle (real callees)
    G = Gen(spec)
    GALE = g.GALE
    flags = [f"--target=x86_64-pc-windows-gnu", "-mno-ms-bitfields", "-std=c99", "-nostdinc", "-fno-builtin",
             "-DLINT", "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-mlong-double-64",
             "-fno-strict-aliasing", "-fwrapv", "-fcommon", "-fgnu89-inline", "-ftrivial-auto-var-init=zero",
             "-ffp-contract=on", "-O2", "-Xclang", "-disable-llvm-passes", "-S", "-emit-llvm", "-w",
             "-I.", "-Isrc", "-Ipc", "-Ipc/gameworld", "-isystem", "src/MSL", "-isystem",
             "libs/dolphin/include", "-isystem", "libs/dolphin/src", "-isystem", GALE, "-include",
             "src/MSL/math_ppc.h", "-D_Static_assert(a,b)=", "-Dsize_t=gc_size_t"]
    # 1. IR of the bare TU: which functions/globals are external
    ll = out / "tu_only.ll"
    flags = flags + ["-I", incdir.as_posix()]
    run([CL] + flags + [tu_path, "-o", str(ll)])
    irt = ll.read_text()
    defined = set(re.findall(r"^define [^@]*@([\w.$]+)\(", irt, re.M))
    declared = [m for m in re.findall(r"^declare [^@]*@([\w.$]+)\(", irt, re.M) if not m.startswith("llvm.")]
    extvars = re.findall(r"^@([\w.$]+) = (?:external|common|extern_weak)[^\n]*", irt, re.M)
    # 2. libclang parse
    idx = ci.Index.create()
    tu = idx.parse(tu_path, args=NAT + ["-I", incdir.as_posix()], options=ci.TranslationUnit.PARSE_DETAILED_PROCESSING_RECORD)
    errs = [d for d in tu.diagnostics if d.severity >= ci.Diagnostic.Error]
    if errs:
        raise SystemExit("clang errors:\n" + "\n".join(str(x) for x in errs[:10]))
    tu_abs = Path(tu_path).resolve().as_posix().lower()

    inc_dir = incdir.resolve().as_posix().lower()

    def in_tu(c):
        # the TU file itself, or a `.inc` it includes from its own directory
        if c.location.file is None:
            return False
        f_ = Path(c.location.file.name).resolve().as_posix().lower()
        return f_ == tu_abs or (f_.endswith(".inc") and f_.rsplit("/", 1)[0] == inc_dir)

    fdecls, fdefs, vdecls = {}, {}, {}
    inl = set()
    for c in tu.cursor.walk_preorder():
        if c.kind == K.FUNCTION_DECL:
            fdecls.setdefault(c.spelling, c)
            if c.is_definition() and in_tu(c):
                fdefs[c.spelling] = c
                # `static inline` helpers are compiled into their callers: not ported as functions
                if "inline" in [tk.spelling for tk in list(c.get_tokens())[:8]]:
                    inl.add(c.spelling)
        elif c.kind == K.VAR_DECL and c.semantic_parent.kind == K.TRANSLATION_UNIT:
            vdecls.setdefault(c.spelling, []).append(c)

    # ---- globals ----
    glob = []  # (name, cursor, extern?)
    skipg = set(spec.get("SKIP_GLOBALS", []))
    for name, cs in vdecls.items():
        defs = [c for c in cs if (in_tu(c) or name in spec.get("EXTRA_GLOBALS", [])) and c.storage_class != ci.StorageClass.EXTERN]
        if defs and not defs[0].type.is_const_qualified() and name not in skipg:
            glob.append((name, defs[0], False))
    for name in extvars:
        if name in skipg or any(n == name for n, _, _ in glob) or name not in vdecls:
            continue
        if name in spec.get("EXTERN_CONST", []):
            continue
        glob.append((name, vdecls[name][0], True))
    # extern variables that are function-addresses etc are skipped by the filter above

    # ---- unit.c / rust text ----
    C, R = [], []
    gl_c, gl_r = [], []
    rust_mod = spec.get("RUST_MOD")
    extern_decl = []
    raw_g = set(spec.get("RAW_GLOBALS", []))
    nofill = set(spec.get("NOFILL", []))
    gsizes = spec.get("GLOBAL_TYPES", {})
    ext_globals_rs = []

    def rtype(t):
        return rust_type(G, t)

    for name, cur, is_ext in glob:
        t = cur.type
        ct = canon(t)
        init = False
        if not is_ext:
            init = any(tk.spelling == "=" for tk in cur.get_tokens())
        fill = (not init or name in spec.get("FORCE_FILL", [])) and name not in nofill
        cname = f"gv_{name}"
        is_raw = name in raw_g
        fallback_ty = None
        if not is_raw:
            try:
                e = Emit()
                G.w.leaf(e, t, f"(*p)", f"(*p)", "path", "path", spec.get("RUST_GLOBAL_TYPES", {}).get(name))
                rt = spec.get("RUST_GLOBAL_TYPES", {}).get(name) or rtype(t)
            except SystemExit as ex:
                if "no Rust name" not in str(ex):
                    raise
                print(f"tuo: global {name} compared as bytes ({ex})")
                is_raw = True
                fallback_ty = f"[u8; {t.get_size()}]"
        if is_raw:
            gl_c.append(f'  tuo_regglobal("{name}", (void*)&{name}, sizeof({name}), 0, {int(fill)});')
        else:
            C.append(f"static void {cname}(const char* path, void* vp) {{ __typeof__({name})* p = (__typeof__({name})*) vp; char _pb[600]; (void) _pb;")
            C.extend(e.c)
            C.append("}")
            R.append("#[allow(unused_variables, unused_mut, unused_unsafe, unused_parens, non_snake_case)]")
            R.append(f"fn {cname}(v: &mut V, path: &str, vp: *mut u8) {{ let p = vp as *mut {rt}; unsafe {{")
            R.extend(e.r)
            R.append("} }")
            gl_c.append(f'  tuo_regglobal("{name}", (void*)&{name}, sizeof({name}), {cname}, {int(fill)});')
        path = f"{rust_mod}::ext::{name}" if is_ext else f"{rust_mod}::{name}"
        over = spec.get("RUST_GLOBAL_PATH", {}).get(name)
        if over:
            path = over
        if is_raw:
            gl_r.append(f'    Global {{ name: "{name}", p: unsafe {{ &raw mut {path} as *mut u8 }}, size: unsafe {{ core::mem::size_of_val(&*(&raw const {path})) }}, vis: None, fill: {str(fill).lower()} }},')
        else:
            gl_r.append(f'    Global {{ name: "{name}", p: unsafe {{ &raw mut {path} as *mut u8 }}, size: unsafe {{ core::mem::size_of_val(&*(&raw const {path})) }}, vis: Some({cname}), fill: {str(fill).lower()} }},')
        if is_ext:
            ext_globals_rs.append((name, fallback_ty or cur))
            if ct.kind == T.INCOMPLETEARRAY:
                n = spec.get("EXTERN_ARRAY_LEN", {}).get(name)
                if n is None:
                    raise SystemExit(f"tuo: extern array {name} needs EXTERN_ARRAY_LEN")
                extern_decl.append(f"__typeof__({name}[0]) {name}[{n}];")
            else:
                extern_decl.append(f"__typeof__({name}) {name};")

    guest = []
    for addr, (gname, ctype, rtyp) in spec.get("GUEST_VARS", {}).items():
        sym = f"tuo_g_{gname}"
        extern_decl.append(f"{ctype} {sym};")
        signed = addr - (1 << 32) if addr >= (1 << 31) else addr
        guest.append(f"{signed} {sym}")
        gl_c.append(f'  tuo_regglobal("{gname}", (void*)&{sym}, sizeof({sym}), 0, 1);')
        path = f"{rust_mod}::ext::{gname}"
        gl_r.append(f'    Global {{ name: "{gname}", p: unsafe {{ &raw mut {path} as *mut u8 }}, size: core::mem::size_of::<{rtyp}>(), vis: None, fill: true }},')
        ext_globals_rs.append((gname, rtyp))
    (out / "guest.txt").write_text("\n".join(guest) + "\n")
    # ---- cases ----
    cases = spec.get("CASES", {})
    cfg = []
    glue = []
    rglue = []
    for fname, cs in cases.items():
        d = fdefs.get(fname)
        if d is None:
            raise SystemExit(f"tuo: {fname} not defined in {tu_path}")
        args = list(d.get_arguments())
        rt = canon(d.result_type)
        toks = []
        user = cs.get("args", [])
        for i, a in enumerate(args):
            if i < len(user) and user[i] is not None:
                toks.append(user[i])
                continue
            ac = canon(a.type)
            if ac.kind in INTK or ac.kind == T.ENUM:
                if ac.kind == T.ENUM:
                    b, s = INTK[canon(ac.get_declaration().enum_type).kind]
                else:
                    b, s = INTK[ac.kind]
                toks.append(f"i{b}{'s' if s else 'u'}")
            elif ac.kind == T.FLOAT:
                toks.append("f32")
            elif ac.kind in (T.DOUBLE,):
                toks.append("f64")
            elif ac.kind == T.POINTER:
                pt = canon(ac.get_pointee())
                if pt.kind == T.RECORD:
                    dd, cn = G.cx.rec_names(strip(a.type).get_pointee() if strip(a.type).kind == T.POINTER else a.type)
                    toks.append(f"p:{cfgn(cn)}")
                    rn = G.cx.rust_rec(dd, cn)
                    G.need(cn, dd, rn)
                elif pt.kind in INTK:
                    toks.append("buf:16")
                else:
                    toks.append("buf:64")
            else:
                raise SystemExit(f"tuo: arg {i} of {fname}: {a.type.spelling}")
        if rt.kind == T.VOID:
            rk = "v"
        elif rt.kind in INTK or rt.kind == T.ENUM:
            if rt.kind == T.ENUM:
                s = INTK[canon(rt.get_declaration().enum_type).kind][1]
            else:
                s = INTK[rt.kind][1]
            rk = "i" if s else "u"
        elif rt.kind == T.FLOAT:
            rk = "f32"
        elif rt.kind == T.DOUBLE:
            rk = "f64"
        elif rt.kind == T.POINTER:
            rk = "p"
        else:
            raise SystemExit(f"tuo: return type of {fname}: {d.result_type.spelling}")
        cfg.append(f"fn {fname} {cs.get('n', spec.get('N', 100))} {rk} {' '.join(toks)}".rstrip())
        # C glue
        decl_args, call_args = [], []
        for i, a in enumerate(args):
            ac = canon(a.type)
            if ac.kind == T.FLOAT:
                decl_args.append(f"float x{i}; {{ unsigned _b = (unsigned) a[{i}]; __builtin_memcpy(&x{i}, &_b, 4); }}")
            elif ac.kind == T.DOUBLE:
                decl_args.append(f"double x{i}; __builtin_memcpy(&x{i}, &a[{i}], 8);")
            else:
                decl_args.append(f"{a.type.spelling} x{i} = ({a.type.spelling}) a[{i}];" if "(*)" not in a.type.spelling else f"__typeof__(((__typeof__({fname})*)0)) _unused{i};")
            call_args.append(f"x{i}")
        call = f"{fname}({', '.join(call_args)})"
        if rk == "v":
            body = f"{call}; return 0;"
        elif rk == "f32":
            body = f"float r = {call}; unsigned b; __builtin_memcpy(&b, &r, 4); return b;"
        elif rk == "f64":
            body = f"double r = {call}; unsigned long long b; __builtin_memcpy(&b, &r, 8); return b;"
        elif rk == "p":
            body = f"return (unsigned long long)(unsigned long long) {call};"
        elif rk == "i":
            body = f"return (unsigned long long)(long long) {call};"
        else:
            body = f"return (unsigned long long) {call};"
        glue.append(f'  if(tuo_streq(fn, "{fname}")) {{ {" ".join(decl_args)} {body} }}')
        # Rust glue
        rargs = []
        for i, a in enumerate(args):
            ac = canon(a.type)
            if ac.kind == T.FLOAT:
                rargs.append(f"f32::from_bits(a[{i}] as u32)")
            elif ac.kind == T.DOUBLE:
                rargs.append(f"f64::from_bits(a[{i}])")
            elif ac.kind == T.POINTER and canon(ac.get_pointee()).kind in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO):
                rargs.append(f"unsafe {{ core::mem::transmute::<usize, _>(a[{i}] as usize) }}")
            elif ac.kind == T.POINTER:
                rargs.append(f"a[{i}] as usize as _")
            elif ac.kind == T.BOOL:
                rargs.append(f"(a[{i}] != 0) as _")
            else:
                rargs.append(f"a[{i}] as _")
        rcall = f"{rust_mod}::{fname}({', '.join(rargs)})"
        if rk == "v":
            rb = f"{{ {rcall}; 0 }}"
        elif rk == "f32":
            rb = f"{{ let r: f32 = {rcall}; r.to_bits() as u64 }}"
        elif rk == "f64":
            rb = f"{{ let r: f64 = {rcall}; r.to_bits() }}"
        elif rk == "p":
            rb = f"{{ let r = {rcall}; r as usize as u64 }}"
        elif rk == "i":
            rb = f"{{ let r = {rcall}; (r as i64) as u64 }}"
        else:
            rb = f"{{ let r = {rcall}; r as u64 }}"
        rglue.append(f'        "{fname}" => unsafe {rb},')
    if spec.get("LOWMEM"):
        cfg.append("lowmem 1")  # pool objects below 4 GB (u32 pointer slots)
    for k, v in spec.get("DOM", {}).items():
        cfg.append("dom " + k + " " + " ".join(str(x) for x in v))
    for k, v in spec.get("UNION_VIEW", {}).items():
        if isinstance(v, dict):
            for m, fns in v.items():
                cfg.append(f"view {k} {m} " + " ".join(fns))
    links = dict(spec.get("LINKS", {}))

    # ---- stubs ----
    stubs_c = ['#include <string.h>', 'extern void tuo_call_begin(const char*); extern void tuo_call_end(void);',
               'extern void tuo_arg_i(long long); extern void tuo_arg_u(unsigned long long); extern void tuo_arg_f32(unsigned);',
               'extern void tuo_arg_f64(unsigned long long); extern void tuo_arg_p(const void*); extern void tuo_arg_s(const char*); extern void tuo_arg_bytes(const void*, unsigned long long);',
               'extern unsigned long long tuo_ret_int(const char*, int, int); extern unsigned tuo_ret_f32(const char*);',
               'extern unsigned long long tuo_ret_f64(const char*); extern void* tuo_ret_ptr(const char*);',
               'extern void tuo_regfn(const char*, void*); extern void* tuo_leaf_ptr(const char*); extern unsigned long long tuo_leaf_int(const char*, int, int);',
               'extern unsigned tuo_leaf_f32(const char*); extern int sprintf(char*, const char*, ...);', '']
    stubs_reg = []
    rs_stub = []
    rs_install = []
    ext_hooks = []
    skip_stub = set(spec.get("SKIP_STUBS", []))
    OUTS = spec.get("OUTS", {})

    OUT_INTS = spec.get("OUT_INTS", {})
    OUT_F32S = spec.get("OUT_F32S", {})

    def outs_c(nm, n):
        a = " ".join(f'if (a{i}) *(void**) a{i} = tuo_leaf_ptr("out:{nm}#{i}");' for i in OUTS.get(nm, []))
        b = " ".join(f'if (a{i}) {{ unsigned long long _v = tuo_leaf_int("out:{nm}#{i}", {bits}, 0); __builtin_memcpy(a{i}, &_v, {bits // 8}); }}'
                     for i, bits in OUT_INTS.get(nm, {}).items())
        c3 = " ".join(f'if (a{i}) {{ int _k; for (_k = 0; _k < {cnt}; _k++) {{ char _pn[96]; unsigned _b; sprintf(_pn, "out:{nm}#{i}[%d]", _k); _b = tuo_leaf_f32(_pn); __builtin_memcpy((char*) a{i} + 4 * _k, &_b, 4); }} }}'
                      for i, cnt in OUT_F32S.get(nm, {}).items())
        return a + " " + b + " " + c3

    def outs_r(nm, n):
        a = " ".join(f'if !a{i}.is_null() {{ *(a{i} as *mut usize) = v.leaf_ptr("out:{nm}#{i}"); }}' for i in OUTS.get(nm, []))
        b = " ".join(f'if !a{i}.is_null() {{ let x: u64 = v.leaf_int("out:{nm}#{i}", {bits}, 0); core::ptr::copy_nonoverlapping(&x as *const u64 as *const u8, a{i} as *mut u8, {bits // 8}); }}'
                     for i, bits in OUT_INTS.get(nm, {}).items())
        r3 = " ".join(f'if !a{i}.is_null() {{ for k in 0..{cnt}usize {{ *(a{i} as *mut u32).add(k) = v.leaf_f32(&format!("out:{nm}#{i}[{{}}]", k)); }} }}'
                      for i, cnt in OUT_F32S.get(nm, {}).items())
        return a + " " + b + " " + r3

    custom = spec.get("CUSTOM_STUBS", {})
    for name in declared:
        if name in defined or name in LIBC or name in skip_stub:
            continue
        d = fdecls.get(name)
        if d is None:
            raise SystemExit(f"tuo: no prototype for callee {name}")
        if name in spec.get("FMT_VARIADIC", []):
            custom = dict(custom)
            custom[name] = fmt_variadic(name)
        if name in custom:
            stubs_c.append(custom[name]["c"])
            stubs_reg.append(f'  tuo_regfn("{name}", (void*){name});')
            if "rust" in custom[name]:
                rs_stub.append(custom[name]["rust"])
            rs_install.append(f'        tu::ext::H_{name} = Some(s_{name});')
            ext_hooks.append(custom[name]["ext"])
            continue
        vsp = spec.get("VARIADIC", {}).get(name)
        if vsp:
            kinds = vsp["args"]
            cdecl = ["const char* a0" if kinds[0][0] == "s" else "void* a0"] if kinds[0][0] in "sp" else [f"int a0"]
            body, logs, rl, rp = [], [], [], []
            for i, (kd, rt_) in enumerate(kinds):
                if i > 0:
                    body.append({"s": f"const char* a{i} = __builtin_va_arg(ap, const char*);", "p": f"void* a{i} = __builtin_va_arg(ap, void*);",
                                 "i": f"int a{i} = __builtin_va_arg(ap, int);"}[kd])
                logs.append({"s": f"tuo_arg_s(a{i});", "p": f"tuo_arg_p(a{i});", "i": f"tuo_arg_i(a{i});"}[kd])
                rl.append({"s": f"v.arg_s(a{i} as *const u8);", "p": f"v.arg_p(a{i} as usize);", "i": f"v.arg_i(a{i} as i64);"}[kd])
                rp.append(f"a{i}: {rt_}")
            rk = vsp["ret"]
            cret = {"v": "void", "p": "void*", "i": "int"}[rk]
            cb = {"v": "", "p": f'return tuo_ret_ptr("{name}");', "i": f'return (int) tuo_ret_int("{name}", 32, 1);'}[rk]
            rrt = {"v": "", "p": " -> " + vsp.get("ret_rust", ""), "i": " -> i32"}[rk]
            rb = {"v": "", "p": f'v.ret_ptr("{name}") as _', "i": f'v.ret_int("{name}", 32, 1) as _'}[rk]
            stubs_c.append(f"{cret} {name}({cdecl[0]}, ...) {{ __builtin_va_list ap; __builtin_va_start(ap, a0); {' '.join(body)} __builtin_va_end(ap); tuo_call_begin(\"{name}\"); {' '.join(logs)} tuo_call_end(); {outs_c(name, len(kinds))} {cb} }}")
            stubs_reg.append(f'  tuo_regfn("{name}", (void*){name});')
            rs_stub.append(f"unsafe extern \"C\" fn s_{name}({', '.join(rp)}){rrt} {{ let v = vv(); v.call_begin(\"{name}\"); {' '.join(rl)} v.call_end(); {outs_r(name, len(kinds))} {rb} }}")
            rs_install.append(f"        tu::ext::H_{name} = Some(s_{name});")
            ext_hooks.append((name, [f"a{i}" for i in range(len(kinds))], [t_ for _, t_ in kinds], vsp["ret_rust"] if rk == "p" else ("i32" if rk == "i" else None)))
            if rk == "p" and f"ret:{name}" not in links:
                links[f"ret:{name}"] = vsp.get("link", "buf:64")
            continue
        if d.type.is_function_variadic():
            raise SystemExit(f"tuo: callee {name} is variadic: add CUSTOM_STUBS")
        params = [canon(a.type) for a in d.get_arguments()]
        ptypes = [strip(a.type) for a in d.get_arguments()]
        ret = canon(d.result_type)
        cargs, logs, rparams, rlogs = [], [], [], []
        for i, (p, a) in enumerate(zip(params, d.get_arguments())):
            if p.kind in INTK or p.kind == T.ENUM:
                if p.kind == T.ENUM:
                    b, s = INTK[canon(p.get_declaration().enum_type).kind]
                else:
                    b, s = INTK[p.kind]
                cty = {(8, 1): "signed char", (8, 0): "unsigned char", (16, 1): "short", (16, 0): "unsigned short",
                       (32, 1): "int", (32, 0): "unsigned", (64, 1): "long long", (64, 0): "unsigned long long"}[(b, s)]
                if p.kind == T.BOOL:
                    cty = "unsigned char"
                cargs.append(f"{cty} a{i}")
                logs.append(f"tuo_arg_{'i' if s else 'u'}(a{i});")
                rparams.append(f"a{i}: {RINT[(b, s)] if p.kind != T.BOOL else 'bool'}")
                rlogs.append(f"v.arg_{'i' if s else 'u'}(a{i} as {'i64' if s else 'u64'});" if p.kind != T.BOOL else f"v.arg_u(a{i} as u64);")
            elif p.kind == T.FLOAT:
                cargs.append(f"float a{i}")
                logs.append(f"{{ unsigned b; memcpy(&b, &a{i}, 4); tuo_arg_f32(b); }}")
                rparams.append(f"a{i}: f32")
                rlogs.append(f"v.arg_f32(a{i}.to_bits());")
            elif p.kind == T.DOUBLE:
                cargs.append(f"double a{i}")
                logs.append(f"{{ unsigned long long b; memcpy(&b, &a{i}, 8); tuo_arg_f64(b); }}")
                rparams.append(f"a{i}: f64")
                rlogs.append(f"v.arg_f64(a{i}.to_bits());")
            elif p.kind == T.POINTER:
                cargs.append(f"void* a{i}")
                rparams.append(f"a{i}: {rust_type(G, a.type)}")
                pb = spec.get("PTR_BYTES", {}).get(name, {}).get(i)
                if pb:
                    logs.append(f"tuo_arg_bytes(a{i}, {pb});")
                    rlogs.append(f"v.arg_bytes(a{i} as *const u8, {pb});")
                elif is_char_ptr(a.type):
                    logs.append(f"tuo_arg_s((const char*) a{i});")
                    rlogs.append(f"v.arg_s(a{i} as *const u8);")
                else:
                    logs.append(f"tuo_arg_p(a{i});")
                    pk = canon(p.get_pointee()).kind
                    if pk in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO):
                        rlogs.append(f"v.arg_p(unsafe {{ core::mem::transmute_copy::<_, usize>(&a{i}) }});")
                    else:
                        rlogs.append(f"v.arg_p(a{i} as usize);")
            elif p.kind == T.RECORD and p.get_size() in (1, 2, 4, 8):
                # small struct by value (win64 passes it in an integer register): logged as one integer
                sz = p.get_size()
                cty = {1: "unsigned char", 2: "unsigned short", 4: "unsigned", 8: "unsigned long long"}[sz]
                cargs.append(f"{cty} a{i}")
                logs.append(f"tuo_arg_u(a{i});")
                rparams.append(f"a{i}: {rust_type(G, a.type)}")
                rlogs.append(f"v.arg_u(unsafe {{ core::mem::transmute_copy::<_, u{sz * 8}>(&a{i}) }} as u64);")
            else:
                raise SystemExit(f"tuo: callee {name} param {i}: {a.type.spelling}")
        if ret.kind == T.VOID:
            cret, rret, cbody, rbody = "void", "", "", ""
        elif ret.kind in INTK or ret.kind == T.ENUM:
            if ret.kind == T.ENUM:
                b, s = INTK[canon(ret.get_declaration().enum_type).kind]
            else:
                b, s = INTK[ret.kind]
            if ret.kind == T.BOOL:
                b, s = 8, 0
            cty = {(8, 1): "signed char", (8, 0): "unsigned char", (16, 1): "short", (16, 0): "unsigned short",
                   (32, 1): "int", (32, 0): "unsigned", (64, 1): "long long", (64, 0): "unsigned long long"}[(b, s)]
            cret = cty
            cbody = f"return ({cty}) tuo_ret_int(\"{name}\", {b}, {s});"
            rret = f" -> {RINT[(b, s)] if ret.kind != T.BOOL else 'bool'}"
            rbody = f"v.ret_int(\"{name}\", {b}, {s}) as _" if ret.kind != T.BOOL else f"v.ret_int(\"{name}\", 8, 0) != 0"
        elif ret.kind == T.FLOAT:
            cret = "float"
            cbody = f"{{ unsigned b = tuo_ret_f32(\"{name}\"); float r; memcpy(&r, &b, 4); return r; }}"
            rret = " -> f32"
            rbody = f"f32::from_bits(v.ret_f32(\"{name}\"))"
        elif ret.kind == T.DOUBLE:
            cret = "double"
            cbody = f"{{ unsigned long long b = tuo_ret_f64(\"{name}\"); double r; memcpy(&r, &b, 8); return r; }}"
            rret = " -> f64"
            rbody = f"f64::from_bits(v.ret_f64(\"{name}\"))"
        elif ret.kind == T.POINTER and name in spec.get("RET_FIELD", {}):
            # the real function returns a field of its first argument: do the same
            tyname, path_ = spec.get("RET_FIELD", {})[name]
            rdecl, _nm = find_record(tu, tyname)
            off, curt = 0, rdecl.type
            for fld in path_:
                off += curt.get_offset(fld) // 8
                curt = canon(next(f for f in (curt.get_declaration().get_definition() or curt.get_declaration()).get_children() if f.spelling == fld).type)
            cret = "void*"
            cbody = f"return *(void**)((char*)a0 + {off});"
            rret = f" -> {rust_type(G, d.result_type)}"
            rbody = f"(*a0).{'.'.join(path_)}"
        elif ret.kind == T.POINTER:
            cret = "void*"
            cbody = f"return tuo_ret_ptr(\"{name}\");"
            rret = f" -> {rust_type(G, d.result_type)}"
            rbody = f"v.ret_ptr(\"{name}\") as _"
            # default link: a pool object of the pointee record, or a small buffer
            pt = canon(ret.get_pointee())
            key = f"ret:{name}"
            if key not in links:
                if pt.kind == T.RECORD:
                    dd, cn = G.cx.rec_names(strip(d.result_type).get_pointee())
                    rn = G.cx.rust_rec(dd, cn) if cn else None
                    if cn and rn and canon(strip(d.result_type).get_pointee()).get_declaration().get_definition() is not None:
                        links[key] = f"new:{cfgn(cn)}"
                        G.need(cn, dd, rn)
                    else:
                        links[key] = "buf:64"
                elif pt.kind in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO):
                    links[key] = "null"
                else:
                    links[key] = "buf:16"
        else:
            raise SystemExit(f"tuo: callee {name} returns {d.result_type.spelling}")
        stubs_c.append(f"{cret} {name}({', '.join(cargs) if cargs else 'void'}) {{ tuo_call_begin(\"{name}\"); {' '.join(logs)} tuo_call_end(); {outs_c(name, len(params))} {cbody} }}")
        stubs_reg.append(f'  tuo_regfn("{name}", (void*){name});')
        rs_stub.append(f"unsafe extern \"C\" fn s_{name}({', '.join(rparams)}){rret} {{ let v = vv(); v.call_begin(\"{name}\"); {' '.join(rlogs)} v.call_end(); {outs_r(name, len(params))} {rbody} }}")
        rs_install.append(f"        tu::ext::H_{name} = Some(s_{name});")
        ext_hooks.append((name, [f"a{i}" for i in range(len(params))], [rust_type(G, a.type) for a in d.get_arguments()],
                          rust_type(G, d.result_type) if ret.kind != T.VOID else None))
    for xn, xd in spec.get("EXTRA_FNS", {}).items():
        stubs_c.append(xd["c"])
        stubs_reg.append(f'  tuo_regfn("{xn}", (void*){xn});')
        rs_stub.append(xd["rust"])
    stubs_c.append("void tuo_stubs_init(void) {")
    stubs_c.extend(stubs_reg)
    stubs_c.append("}")

    for k, v in links.items():
        cfg.append(f"link {k} {v}")
        if k in spec.get("LINK_GATE", {}):
            cfg.append(f"gate {k} " + " ".join(spec.get("LINK_GATE", {})[k]))
        if k in spec.get("LINK_FORCE", {}):
            cfg.append(f"force {k} " + " ".join(spec.get("LINK_FORCE", {})[k]))
        m = re.match(r"(?:new|maybe):(\w[\w ]*)", v)
        # types named by links
    # types named by LINKS targets / args
    for k, v in links.items():
        m = re.match(r"(?:new|maybe|ptrs):([^:]+)", v)
        if m:
            cn = m.group(1).replace("~", " ")
            self_decl = find_record(tu, cn)
            if self_decl is None:
                raise SystemExit(f"tuo: no record type {cn}")
            dd, nm = self_decl
            G.need(cn, dd, G.cx.rust_rec(dd, cn) or spec.get("RUST_TYPES", {}).get(cn))
    for fname, cs in cases.items():
        pass
    for cn in spec.get("TYPES", []):
        dd, nm = find_record(tu, cn)
        G.need(cn, dd, G.cx.rust_rec(dd, cn))
    for line in cfg:
        m = re.search(r" (?:p|pn):(\S+)", line)
        if line.startswith("fn ") and m:
            for tok in line.split()[4:]:
                mm = re.match(r"pn?:(.+)", tok)
                if mm:
                    cn = mm.group(1)
                    fd = find_record(tu, cn)
                    if fd:
                        G.need(cn, fd[0], G.cx.rust_rec(fd[0], cn))
    # visitors (the queue may grow while generating)
    vc, vr = [], []
    done = set()
    while True:
        todo = [c for c in G.order if c not in done]
        if not todo:
            break
        for cn in todo:
            done.add(cn)
            a, b = G.visitor(cn)
            vc.extend(a)
            vr.extend(b)
    for cn in G.order:
        cfg.append(f"tysize {cfgn(cn)} {G.vis[cn][0].type.get_size()}")
    # ---- write unit.c ----
    uc = [f'#include "{(ROOT / tu_path).as_posix()}"', '']
    uc.append("extern int tuo_mode; extern unsigned long long tuo_leaf_int(const char*, int, int);")
    uc.append("extern unsigned tuo_leaf_f32(const char*); extern unsigned long long tuo_leaf_f64(const char*);")
    uc.append("extern void* tuo_leaf_ptr(const char*); extern void tuo_dump_i(const char*, long long);")
    uc.append("extern void tuo_dump_u(const char*, unsigned long long); extern void tuo_dump_f32(const char*, unsigned);")
    uc.append("extern void tuo_dump_f64(const char*, unsigned long long); extern void tuo_dump_p(const char*, const void*);")
    uc.append("extern void tuo_bytes(const char*, void*, unsigned long long); extern int tuo_viewon(const char*, const char*); extern int tuo_fnin(const char*);")
    uc.append("extern char* tuo_cat(char*, const char*, const char*); extern char* tuo_cati(char*, const char*, const char*, int);")
    uc.append("extern void tuo_regtype(const char*, unsigned long long, unsigned long long, void(*)(const char*, void*));")
    uc.append("extern void tuo_regglobal(const char*, void*, unsigned long long, void(*)(const char*, void*), int);")
    uc.append("extern void tuo_regfn(const char*, void*); extern int tuo_streq(const char*, const char*);")
    uc.append("extern void tuo_stubs_init(void);")
    uc.append("")
    uc.extend(extern_decl_clean(extern_decl))
    uc.append("")
    for cn in G.order:
        uc.append(f"static void vis_{Walker.mangle(cn)}(const char* path, void* vp);")
    uc.extend(vc)
    uc.append("")
    uc.extend(C)
    uc.append("")
    uc.append("void tuo_unit_init(void) {")
    uc.append("  tuo_stubs_init();")
    for cn in G.order:
        mg = Walker.mangle(cn)
        uc.append(f'  tuo_regtype("{cfgn(cn)}", sizeof({cn}), _Alignof({cn}), vis_{mg});')
    uc.extend(gl_c)
    for fnm in fdefs:
        if fnm not in inl:
            uc.append(f'  tuo_regfn("{fnm}", (void*){fnm});')
    uc.append("}")
    uc.append("unsigned long long tuo_unit_expr(const char* n) {")
    for en, (cexpr, _rexpr) in spec.get("ARG_EXPRS", {}).items():
        uc.append(f'  if(tuo_streq(n, "{en}")) return (unsigned long long)(__SIZE_TYPE__)({cexpr});')
    uc.append("  return 0;")
    uc.append("}")
    uc.append("unsigned long long tuo_unit_call(const char* fn, unsigned long long* a) {")
    uc.extend(glue)
    uc.append("  return 0;")
    uc.append("}")
    (out / "unit.c").write_text("\n".join(uc) + "\n")
    (out / "stubs.c").write_text("\n".join(stubs_c) + "\n")
    (out / "tuo.cfg").write_text("\n".join(cfg) + "\n")
    # ---- rust ----
    rs = ["// Generated by rust/melee_world/oracle/tuo/gen_tuo.py: do not edit.",
          "#![allow(unused_imports, unused_unsafe, non_snake_case, non_camel_case_types, dead_code, static_mut_refs)]",
          "use crate::tuo_common::*;", f"use {rust_mod} as tu;", ""]
    rs.extend(vr)
    rs.extend(R)
    rs.extend(rs_stub)
    rs.append("pub fn glue() -> Glue {")
    rs.append("    Glue {")
    rs.append("        types: vec![")
    for cn in G.order:
        mg = Walker.mangle(cn)
        rname = G.vis[cn][1]
        rs.append(f'            TypeInfo {{ name: "{cfgn(cn)}", size: core::mem::size_of::<{rname}>(), align: core::mem::align_of::<{rname}>(), vis: vis_{mg} }},')
    rs.append("        ],")
    rs.append("        globals: vec![")
    rs.extend(gl_r)
    rs.append("        ],")
    rs.append("        fns: vec![")
    for fnm in fdefs:
        if fnm in spec.get("RUST_FN_SKIP", []) or fnm in inl:
            continue
        rs.append(f'            ("{fnm}", tu::{fnm} as *const () as usize),')
    for name in declared:
        if name in defined or name in LIBC or name in skip_stub:
            continue
        rs.append(f'            ("{name}", tu::ext::{name} as *const () as usize),')
    for xn in spec.get("EXTRA_FNS", {}):
        rs.append(f'            ("{xn}", s_{xn} as *const () as usize),')
    rs.append("        ],")
    rs.append("        size_ok: &[" + ", ".join(f'"{x}"' for x in spec.get("SIZE_OK", [])) + "],")
    rs.append("        install: || unsafe {")
    rs.extend(rs_install)
    rs.append("        },")
    rs.append("        expr: |n| match n {")
    for en, (_cexpr, rexpr) in spec.get("ARG_EXPRS", {}).items():
        rs.append(f'            "{en}" => unsafe {{ ({rexpr}) as usize as u64 }},')
    rs.append('            _ => panic!("no such expr {n}"),')
    rs.append("        },")
    rs.append("        call: |name, a| match name {")
    rs.extend(rglue)
    rs.append('            _ => panic!("no such function {name}"),')
    rs.append("        },")
    rs.append("    }")
    rs.append("}")
    (out / f"{tuname}_gen.rs").write_text("\n".join(rs) + "\n")
    # ---- ext text ----
    ex = ["pub mod ext {", "    //! Callees in TUs that are not ported (TODO stubs, named like the C function) and the globals",
          "    //! the C defines elsewhere. Generated by oracle/tuo/gen_tuo.py; every callee is a settable hook.",
          "    #![allow(non_snake_case, non_upper_case_globals, dead_code)]",
          HOOK_MACRO.rstrip(),
          "    // ---- callee hooks ----"]
    for h in ext_hooks:
        if isinstance(h, str):
            ex.append(h)
            continue
        nm, an, ts, rt_ = h
        ps = ", ".join(f"{a}: {t}" for a, t in zip(an, ts))
        port = spec.get("PORTED", {}).get(nm)
        if port:
            ex.append(f"    hook_or!(H_{nm}, {nm}, {port}, ({ps}){' -> ' + rt_ if rt_ else ''});")
        else:
            ex.append(f"    hook!(H_{nm}, {nm}, ({ps}){' -> ' + rt_ if rt_ else ''});")
    ex.append("    // ---- extern globals ----")
    for name, cur in ext_globals_rs:
        rt_ = cur if isinstance(cur, str) else rust_type(G, cur.type)
        ex.append(f"    pub static mut {name}: {rt_} = unsafe {{ core::mem::zeroed() }};")
    ex.append("}")
    (out / "ext.rs.txt").write_text("\n".join(ex) + "\n")
    print(f"unit.c {len(uc)} lines, {len(declared) - len(set(declared) & defined)} callees, {len(glob)} globals, {len(G.order)} types")


def fmt_variadic(name):
    """printf-style variadic callee: the stub walks the format string, logging each argument by its
    conversion (d/i int, u/x/X/c unsigned, s/p pointer id, f double). The port calls
    `ext::NAME(fmt, &[args as u64])` (ints sign-extended, floats as f64 bits, pointers as usize)."""
    c = r"""void NAME(const char* f, ...) { __builtin_va_list ap; const char* c; __builtin_va_start(ap, f);
 tuo_call_begin("NAME"); tuo_arg_s(f);
 for (c = f; *c; c++) if (*c == '%') { c++; while (*c && !strchr("diuxXcspf%", *c)) c++; if (!*c) break;
  switch (*c) { case 'd': case 'i': tuo_arg_i(__builtin_va_arg(ap, int)); break;
   case 'u': case 'x': case 'X': case 'c': tuo_arg_u(__builtin_va_arg(ap, unsigned)); break;
   case 's': case 'p': tuo_arg_p(__builtin_va_arg(ap, void*)); break;
   case 'f': { double d = __builtin_va_arg(ap, double); unsigned long long b; memcpy(&b, &d, 8); tuo_arg_f64(b); } break;
   default: break; } }
 tuo_call_end(); __builtin_va_end(ap); }""".replace("NAME", name).replace("\n", " ")
    r = r"""unsafe fn s_NAME(f: *const i8, a: &[u64]) { let v = vv(); v.call_begin("NAME"); v.arg_s(f as *const u8);
 let fs = core::ffi::CStr::from_ptr(f).to_bytes(); let mut k = 0usize; let mut i = 0usize;
 while i < fs.len() { if fs[i] == b'%' { i += 1; while i < fs.len() && !b"diuxXcspf%".contains(&fs[i]) { i += 1; } if i >= fs.len() { break; }
  match fs[i] { b'd' | b'i' => { v.arg_i(a[k] as i32 as i64); k += 1; }
   b'u' | b'x' | b'X' | b'c' => { v.arg_u(a[k] as u32 as u64); k += 1; }
   b's' | b'p' => { v.arg_p(a[k] as usize); k += 1; }
   b'f' => { v.arg_f64(a[k]); k += 1; } _ => {} } } i += 1; } v.call_end(); }""".replace("NAME", name).replace("\n", " ")
    ext = ("    pub static mut H_NAME: Option<unsafe fn(*const i8, &[u64])> = None;\n"
           "    pub unsafe fn NAME(f: *const i8, a: &[u64]) { match H_NAME { Some(h) => h(f, a), None => panic!(\"TODO: NAME is not ported\") } }").replace("NAME", name)
    return {"c": c, "rust": r, "ext": ext}


def extern_decl_clean(lst):
    return [x for x in lst if x]


def find_record(tu, cname):
    """(decl cursor, name) of a record by C spelling ('Name' typedef or 'struct tag')."""
    want = cname.split(" ")[-1]
    for c in tu.cursor.walk_preorder():
        if c.kind == K.TYPEDEF_DECL and c.spelling == want:
            d = canon(c.underlying_typedef_type).get_declaration()
            if d.kind in (K.STRUCT_DECL, K.UNION_DECL):
                return d.get_definition() or d, want
        if c.kind == K.STRUCT_DECL and c.spelling == want and c.is_definition():
            return c, want
    return None


def rust_type(G, t):
    s = strip(t)
    c = canon(t)
    k = c.kind
    if k in INTK:
        b, sg = INTK[k]
        if k == T.BOOL:
            return "bool"
        if s.kind == T.TYPEDEF and s.spelling == "bool":
            return "i32"
        return RINT[(b, sg)]
    if k == T.ENUM:
        b, sg = INTK[canon(c.get_declaration().enum_type).kind]
        return RINT[(b, sg)]
    if k == T.FLOAT:
        return "f32"
    if k in (T.DOUBLE, T.LONGDOUBLE):
        return "f64"
    if k == T.VOID:
        return "core::ffi::c_void"
    if k == T.POINTER:
        pt = c.get_pointee()
        pc = canon(pt)
        if pc.kind in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO):
            return rust_fn(G, pc)
        q = "*const" if pc.is_const_qualified() else "*mut"
        try:
            return f"{q} {rust_type(G, pt)}"
        except SystemExit as ex:
            if "no Rust name" not in str(ex):
                raise
            return f"{q} core::ffi::c_void"  # TODO(group0): record not generated by melee_types
    if k == T.CONSTANTARRAY:
        return f"[{rust_type(G, c.element_type)}; {c.get_array_size()}]"
    if k == T.INCOMPLETEARRAY:
        return f"[{rust_type(G, c.element_type)}; 0]"
    if k == T.RECORD:
        d, cname = G.cx.rec_names(t)
        rn = G.cx.rust_rec(d, cname) if cname else None
        if not rn:
            raise SystemExit(f"tuo: no Rust name for {t.spelling}")
        return rn
    raise SystemExit(f"tuo: rust_type {t.spelling} {k}")


def rust_fn(G, pc):
    ps = ", ".join(rust_type(G, a) for a in pc.argument_types()) if pc.kind == T.FUNCTIONPROTO else ""
    r = canon(pc.get_result())
    rr = "" if r.kind == T.VOID else f" -> {rust_type(G, r)}"
    return f"Option<unsafe extern \"C\" fn({ps}){rr}>"


if __name__ == "__main__":
    main()
