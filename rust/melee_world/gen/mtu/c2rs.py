#!/usr/bin/env python3
"""c2rs.py: translate the functions of one game C file to native Rust with libclang.

  python rust/melee_world/gen/mtu/c2rs.py <src.c> <out.rs> [--hand DIR] [--only fn,fn] [--inline-hdr]

Used for the mechanical part of the shard ports (mp/mplib.c, mp/mpcoll.c, mp/mpisland.c). The AST
gives every implicit conversion, so integer/float/pointer semantics are the C ones by construction:
  * ints: plain Rust ops (the workspace builds with overflow-checks off, i.e. -fwrapv); division,
    remainder and variable shifts go through melee_runtime::cabi (gwtool's defined results);
  * floats: C types are kept (float math stays f32, mixed math widens exactly where C does);
    float->int casts use Rust's saturating `as` (what fptosi.sat gives), narrow targets via cabi;
  * contraction: a +/- whose left (else right) operand is a same-type multiply becomes
    fmadds / mul_add, exactly clang's -ffp-contract=on rule (see tools/rust-port/fma_sites.py,
    whose per-function counts this tool checks with --check);
  * `bool` is MSL's `int`;
  * functions with `goto` are not translated: the body is `todo!()` unless DIR/<name>.rs exists.
Callees defined elsewhere become `ext::name` hooks (unset: panic "TODO"), data defined elsewhere
`ext::` statics. Output is a starting point: the caller splices in the file header and data.
"""
import os, re, sys, struct, ctypes, subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
sys.path.insert(0, str(HERE))
os.chdir(ROOT)
import gen_tu as G  # noqa: E402

ci, K, T = G.ci, G.K, G.T
lib = G.g._lib
lib.clang_getCursorBinaryOperatorKind.argtypes = [ci.Cursor]
lib.clang_getCursorBinaryOperatorKind.restype = ctypes.c_int
lib.clang_getCursorUnaryOperatorKind.argtypes = [ci.Cursor]
lib.clang_getCursorUnaryOperatorKind.restype = ctypes.c_int
lib.clang_Cursor_Evaluate.argtypes = [ci.Cursor]
lib.clang_Cursor_Evaluate.restype = ctypes.c_void_p
lib.clang_EvalResult_getKind.argtypes = [ctypes.c_void_p]
lib.clang_EvalResult_getKind.restype = ctypes.c_int
lib.clang_EvalResult_getAsInt.argtypes = [ctypes.c_void_p]
lib.clang_EvalResult_getAsInt.restype = ctypes.c_int
lib.clang_EvalResult_getAsLongLong.argtypes = [ctypes.c_void_p]
lib.clang_EvalResult_getAsLongLong.restype = ctypes.c_longlong
lib.clang_EvalResult_isUnsignedInt.argtypes = [ctypes.c_void_p]
lib.clang_EvalResult_isUnsignedInt.restype = ctypes.c_uint
lib.clang_EvalResult_getAsUnsigned.argtypes = [ctypes.c_void_p]
lib.clang_EvalResult_getAsUnsigned.restype = ctypes.c_ulonglong
lib.clang_EvalResult_getAsDouble.argtypes = [ctypes.c_void_p]
lib.clang_EvalResult_getAsDouble.restype = ctypes.c_double
lib.clang_EvalResult_getAsStr.argtypes = [ctypes.c_void_p]
lib.clang_EvalResult_getAsStr.restype = ctypes.c_char_p
lib.clang_EvalResult_dispose.argtypes = [ctypes.c_void_p]

BIN = {3: "*", 4: "/", 5: "%", 6: "+", 7: "-", 8: "<<", 9: ">>", 11: "<", 12: ">", 13: "<=", 14: ">=", 15: "==", 16: "!=",
       17: "&", 18: "^", 19: "|", 20: "&&", 21: "||", 22: "=", 23: "*=", 24: "/=", 25: "%=", 26: "+=", 27: "-=", 28: "<<=",
       29: ">>=", 30: "&=", 31: "^=", 32: "|=", 33: ","}
UN = {1: "post++", 2: "post--", 3: "pre++", 4: "pre--", 5: "&", 6: "*", 7: "+", 8: "-", 9: "~", 10: "!"}

GEN_RS = G.GEN_RS
KNOWN = G.KNOWN
RUST_KW = G.g.RUST_KW

# ---- gen.rs index: record fields, bitfield accessors, consts ---------------------------------------------------------
RECS = {}
for m in re.finditer(r"^pub (?:struct|union) (\w+) \{\n(.*?)^\}", GEN_RS, re.M | re.S):
    RECS[m.group(1)] = dict(re.findall(r"^\s+pub (\w+): ([^,\n]+),", m.group(2), re.M))
BITF = {}
for m in re.finditer(r"^impl (\w+) \{\n(.*?)^\}", GEN_RS, re.M | re.S):
    BITF.setdefault(m.group(1), set()).update(re.findall(r"pub fn (set_)?(\w+)\(&", m.group(2)) and
                                              [x[1] for x in re.findall(r"pub fn (set_)?(\w+)\(&", m.group(2)) if not x[0]])
TYPEALIAS = dict(re.findall(r"^pub type (\w+) = (\w+);", GEN_RS, re.M))
CONSTS = {m.group(1): m.group(2) for m in re.finditer(r"^pub const (\w+): (\w+) =", GEN_RS, re.M)}


def prim_of_const(name):
    t = CONSTS.get(name)
    while t in TYPEALIAS:
        t = TYPEALIAS[t]
    return t


def ident(n):
    return G.g.ident(n)


def ctype(t):
    return t.get_canonical()


def norm(t):
    """canonical type with enums replaced by their underlying integer type"""
    c = t.get_canonical()
    if c.kind == T.ENUM:
        return c.get_declaration().enum_type.get_canonical()
    return c


INT_KINDS = {T.CHAR_S: ("i8", 8, True), T.SCHAR: ("i8", 8, True), T.CHAR_U: ("u8", 8, False), T.UCHAR: ("u8", 8, False),
             T.SHORT: ("i16", 16, True), T.USHORT: ("u16", 16, False), T.INT: ("i32", 32, True), T.UINT: ("u32", 32, False),
             T.LONG: ("i32", 32, True), T.ULONG: ("u32", 32, False), T.LONGLONG: ("i64", 64, True),
             T.ULONGLONG: ("u64", 64, False), T.BOOL: ("bool", 8, False)}
FLT_KINDS = {T.FLOAT: "f32", T.DOUBLE: "f64", T.LONGDOUBLE: "f64"}


def is_int(t):
    return norm(t).kind in INT_KINDS


def is_flt(t):
    return norm(t).kind in FLT_KINDS


def is_ptr(t):
    return norm(t).kind == T.POINTER


def is_arr(t):
    return norm(t).kind in (T.CONSTANTARRAY, T.INCOMPLETEARRAY)


def prim(t):
    k = norm(t).kind
    if k in INT_KINDS:
        return INT_KINDS[k][0]
    if k in FLT_KINDS:
        return FLT_KINDS[k]
    return None


def signed(t):
    k = norm(t).kind
    return INT_KINDS[k][2] if k in INT_KINDS else False


def bits(t):
    return INT_KINDS[norm(t).kind][1]


def is_fnptr(t):
    c = norm(t)
    return c.kind == T.POINTER and ctype(c.get_pointee()).kind in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO)


def ident_rec_is_scalarlike(nt):
    return False


ALL_EXTERN = False
VA_PTR = "*mut crate::tu::va::Va"


def is_va_list(t):
    return t.spelling.replace("const ", "").strip() in ("va_list", "__builtin_va_list", "__gnuc_va_list")


def rty(t):
    """Rust spelling of a declared type"""
    if is_va_list(t):
        return VA_PTR
    c = ctype(t)
    if c.kind == T.POINTER and ctype(c.get_pointee()).kind in (T.CONSTANTARRAY, T.INCOMPLETEARRAY):
        pt = ctype(c.get_pointee())
        mut = "*const" if c.get_pointee().is_const_qualified() else "*mut"
        return f"{mut} [{rty(pt.element_type)}; {pt.element_count if pt.kind == T.CONSTANTARRAY else 0}]"
    if c.kind == T.CONSTANTARRAY:
        return f"[{rty(c.element_type)}; {c.element_count}]"
    if c.kind == T.POINTER:
        pt = ctype(c.get_pointee())
        if pt.kind == T.RECORD and pt.get_declaration().hash in ANON:
            mut = "*const" if c.get_pointee().is_const_qualified() else "*mut"
            return f"{mut} {ANON[pt.get_declaration().hash]}"
        if pt.kind == T.RECORD and pt.get_declaration().spelling in MAIN_RECS:
            mut = "*const" if c.get_pointee().is_const_qualified() else "*mut"
            return f"{mut} {pt.get_declaration().spelling}"
    if c.kind == T.RECORD and c.get_declaration().hash in ANON:
        return ANON[c.get_declaration().hash]
    if c.kind == T.RECORD and c.get_declaration().spelling in MAIN_RECS:
        return c.get_declaration().spelling
    return G.rust_ty(t)


def pty(t):
    """Rust type of a parameter (arrays decay to element pointers)"""
    if is_va_list(t):
        return VA_PTR
    c = ctype(t)
    if c.kind in (T.CONSTANTARRAY, T.INCOMPLETEARRAY):
        return f"*mut {rty(c.element_type)}"
    return rty(t)


def f32lit(v):
    if v == 3.4028234663852886e+38:
        return "f32::MAX"
    f = struct.unpack("<f", struct.pack("<f", v))[0]
    s = repr(f)
    for p in range(1, 10):
        s = "%.*g" % (p, f)
        try:
            if struct.unpack("<f", struct.pack("<f", float(s)))[0] == f:
                break
        except OverflowError:
            pass
    if "e" not in s and "." not in s and "inf" not in s and "nan" not in s:
        s += ".0"
    return s


def f64lit(v):
    s = repr(float(v))
    if "e" not in s and "." not in s and "inf" not in s and "nan" not in s:
        s += ".0"
    return s


class Ex:
    __slots__ = ("code", "ty", "atom", "isbool", "null")

    def __init__(self, code, ty, atom=False, isbool=False, null=False):
        self.code, self.ty, self.atom, self.isbool, self.null = code, ty, atom, isbool, null

    def p(self):
        return self.code if self.atom else f"({self.code})"


WARN = []
MAIN_RECS = set()
PUB_ALL = False  # --pub-all: static functions are pub too (the state oracles call them)
ANON = {}  # decl hash -> Rust name of an anonymous struct declared by a `static struct {..} name;` variable
NL = chr(10)


def warn(msg):
    WARN.append(msg)
    print("c2rs: WARNING:", msg, file=sys.stderr)


def evaluate(c):
    r = lib.clang_Cursor_Evaluate(c)
    if not r:
        return None
    try:
        k = lib.clang_EvalResult_getKind(r)
        if k == 1:  # int
            if lib.clang_EvalResult_isUnsignedInt(r):
                return ("i", lib.clang_EvalResult_getAsUnsigned(r))
            return ("i", lib.clang_EvalResult_getAsLongLong(r))
        if k == 2:
            return ("f", lib.clang_EvalResult_getAsDouble(r))
        if k in (3, 4, 5):
            return ("s", lib.clang_EvalResult_getAsStr(r))
        return None
    finally:
        lib.clang_EvalResult_dispose(r)


def strip(c):
    """strip parens"""
    while c.kind == K.PAREN_EXPR:
        c = next(iter(c.get_children()))
    return c


def children(c):
    return list(c.get_children())


def expr_children(c):
    return [x for x in c.get_children() if x.kind not in (K.TYPE_REF, K.NAMESPACE_REF, K.TEMPLATE_REF)]


class Tr:
    def __init__(self, src, hand=None, sibs=()):
        self.tu = G.Tu(src)
        self.sib_fns, self.sib_vars = {}, {}
        for st in sibs:
            stem, path = st.split(":")
            if Path(path).resolve() == Path(src).resolve():
                continue
            t2 = G.Tu(path)
            for n, fn in t2.defs.items():
                if fn.storage_class != ci.StorageClass.STATIC:
                    self.sib_fns.setdefault(n, stem)
            for n, v in [(c.spelling, c) for c in t2.tu.cursor.get_children() if c.kind == K.VAR_DECL and t2.in_main(c) and c.storage_class not in (ci.StorageClass.EXTERN, ci.StorageClass.STATIC)]:
                self.sib_vars.setdefault(n, stem)
        self.src = src
        self.hand = Path(hand) if hand else None
        self.defs = self.tu.defs
        # functions of this TU whose address a sibling TU takes need `extern "C"` too
        self.sib_taken = set()
        for st in sibs:
            stem, path = st.split(":")
            if Path(path).resolve() == Path(src).resolve():
                continue
            t2 = Tr(path, hand, ())
            for fn in t2.defs.values():
                try:
                    t2.emit_fn(fn)
                except Exception:
                    pass
            self.sib_taken |= t2.fnaddr | set(t2.ext_cb)
        self.ext_fns = {}      # name -> FunctionDecl cursor
        self.ext_vars = {}     # name -> VarDecl cursor
        self.inlines = {}      # name -> cursor (header-defined, translated into the module)
        self.fnaddr = set()    # functions whose address is taken (need extern "C")
        self.ext_cb = {}      # ext functions used as values (C-ABI forwarders in `ext::cb`)
        self.main_vars = {}    # name -> cursor (variables defined in this TU)
        for c in self.tu.tu.cursor.get_children():
            if c.kind in (K.STRUCT_DECL, K.UNION_DECL) and c.is_definition() and self.tu.in_main(c) and c.spelling and c.spelling not in KNOWN and "unnamed" not in c.spelling and "anonymous" not in c.spelling:
                MAIN_RECS.add(c.spelling)
        self.local_structs = {}
        self.static_locals = {}
        self.extra_statics = []
        self.fma = {}
        self.cur = None
        self.loops = []
        self.tmp = 0
        for c in self.tu.tu.cursor.get_children():
            if c.kind == K.VAR_DECL and self.tu.in_main(c) and c.storage_class != ci.StorageClass.EXTERN:
                self.main_vars[c.spelling] = c
        # `static struct { ... } name;` : an anonymous record is named after its variable
        self.anon_var_structs = []
        for n, v in self.main_vars.items():
            et = ctype(v.type)
            if et.kind == T.CONSTANTARRAY:
                et = ctype(et.element_type)
            if et.kind == T.RECORD:
                d = et.get_declaration()
                if (not d.spelling or "unnamed" in d.spelling or "anonymous" in d.spelling) and d.is_definition() and d.hash not in self.local_structs:
                    nm = f"{n}_t"
                    self.local_structs[d.hash] = nm
                    ANON[d.hash] = nm
                    self.anon_var_structs.append((nm, d))
        self.FORCE_EXT = {"HSD_StateInitTev", "HSD_StateAssignTev"}
        self.ALIAS = {
            "sqrtf": "melee_runtime::gekko::sqrtf",
            "__frsqrte": "melee_core::fp::frsqrte",
            "sinf": "melee_runtime::gekko::sinf",
            "cosf": "melee_runtime::gekko::cosf",
            "lbVector_Add": "lbVector_Add",
            "lbVector_Diff": "lbVector_Diff",
            "PSMTXMultVec": "PSMTXMultVec",
            "__fpclassifyf": "__fpclassifyf",
            "__fpclassifyd": "__fpclassifyd",
        }

    # ---- helpers --------------------------------------------------------------------------------------------------
    def newtmp(self):
        self.tmp += 1
        return f"__t{self.tmp}"

    def is_main(self, c):
        return self.tu.in_main(c)

    def ext_cb_ref(self, d, code):
        """an ext (other-TU) function used as a value needs a C-ABI item: `ext::cb::name` forwards to the hook"""
        if code.startswith("ext::") and not code.startswith("ext::cb::") and "::" not in code[5:]:
            self.ext_cb.setdefault(d.spelling, d)
            return "ext::cb::" + code[5:]
        return code

    def fn_ref(self, d):
        """Rust path of a function declaration"""
        name = d.spelling
        if name in self.ALIAS:
            return self.ALIAS[name]
        df = d.get_definition()
        if name.startswith("GX") or name in self.FORCE_EXT:
            self.ext_fns.setdefault(name, d)
            return f"ext::{ident(name)}"
        if name in self.sib_fns:
            return f"super::{self.sib_fns[name]}::{ident(name)}"
        if df is not None and self.is_main(df):
            return ident(name)
        if df is not None:  # header inline: translate it too
            self.inlines.setdefault(name, df)
            return ident(name)
        self.ext_fns.setdefault(name, d)
        return f"ext::{ident(name)}"

    # ---- type conversion -------------------------------------------------------------------------------------------
    def conv(self, ex, to):
        if ex.isbool:
            ex = Ex(f"({ex.code}) as i32", self.int_t(), False)
        f, t = norm(ex.ty), norm(to)
        if ex.null and t.kind == T.POINTER:
            return Ex("None" if is_fnptr(to) else "null_mut()", to, True)
        if t.kind == T.VOID:
            if re.fullmatch(r"[\w.\*\(\)]+( as \w+)?", ex.code) and "(" not in ex.code.replace("(*", ""):
                return Ex("()", to, True)
            return Ex(ex.code, to, ex.atom)
        fk, tk = f.kind, t.kind
        if fk in INT_KINDS and tk in INT_KINDS:
            if INT_KINDS[fk][:3] == INT_KINDS[tk][:3]:
                return Ex(ex.code, to, ex.atom)
            return Ex(f"{ex.p()} as {INT_KINDS[tk][0]}", to, False)
        if fk in INT_KINDS and tk in FLT_KINDS:
            return Ex(f"{ex.p()} as {FLT_KINDS[tk]}", to, False)
        if fk in FLT_KINDS and tk in FLT_KINDS:
            if FLT_KINDS[fk] == FLT_KINDS[tk]:
                return Ex(ex.code, to, ex.atom)
            return Ex(f"{ex.p()} as {FLT_KINDS[tk]}", to, False)
        if fk in FLT_KINDS and tk in INT_KINDS:
            n, w, s = INT_KINDS[tk]
            if w >= 32:
                return Ex(f"{ex.p()} as {n}", to, False)
            return Ex(f"melee_runtime::cabi::f2{'i' if s else 'u'}{w}({ex.p()} as f64)", to, True)
        if fk == T.POINTER and tk == T.POINTER:
            if is_fnptr(ex.ty) and is_fnptr(to):
                if rty(ex.ty) != rty(to):  # incompatible function pointer types: the same address
                    return Ex(f"core::mem::transmute::<{rty(ex.ty)}, {rty(to)}>({ex.code})", to, True)
                return Ex(ex.code, to, ex.atom)
            if is_fnptr(ex.ty) != is_fnptr(to):
                return Ex(f"core::mem::transmute::<{rty(ex.ty)}, {rty(to)}>({ex.code})", to, True)
            if rty(ex.ty) == rty(to):
                return Ex(ex.code, to, ex.atom)
            return Ex(f"{ex.p()} as {rty(to)}", to, False)
        if fk in INT_KINDS and tk == T.POINTER:
            if is_fnptr(to):
                return Ex(f"core::mem::transmute::<usize, {rty(to)}>({ex.p()} as usize)", to, True)
            return Ex(f"{ex.p()} as usize as {rty(to)}", to, False)
        if fk == T.POINTER and tk in INT_KINDS:
            warn(f"{self.cur}: pointer->int cast {ex.code!r} (hand check)")
            return Ex(f"{ex.p()} as usize as {INT_KINDS[tk][0]}", to, False)
        if fk in (T.CONSTANTARRAY, T.INCOMPLETEARRAY) and tk == T.POINTER and ex.code.startswith('c"'):
            return Ex(ex.code, to, False)
        if fk in (T.CONSTANTARRAY, T.INCOMPLETEARRAY) and tk == T.POINTER:
            ep = "*mut " + rty(f.element_type)
            if ep == rty(to):
                return Ex(f"{ex.p()}.as_mut_ptr()", to, False)
            return Ex(f"{ex.p()}.as_mut_ptr() as {rty(to)}", to, False)
        if fk in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO) and tk == T.POINTER:
            return ex
        if fk == tk and fk == T.RECORD:
            return ex
        warn(f"{self.cur}: unhandled conversion {f.spelling} -> {t.spelling} for {ex.code!r}")
        return ex

    def int_t(self):
        if not hasattr(self, "_int"):
            tu = self.tu.tu
            self._int = ctype(next(c for c in tu.cursor.walk_preorder() if c.kind == K.PARM_DECL and c.type.kind == T.INT).type)
        return self._int

    def cond(self, c):
        """C condition -> Rust bool code"""
        e = self.E(c)
        return self.to_cond(e)

    def to_cond(self, e):
        if e.isbool:
            return e.code
        t = norm(e.ty)
        if t.kind in INT_KINDS:
            return f"{e.p()} != 0"
        if t.kind in FLT_KINDS:
            return f"{e.p()} != 0.0"
        if t.kind == T.POINTER:
            if is_fnptr(e.ty):
                return f"{e.p()}.is_some()"
            return f"!{e.p()}.is_null()"
        warn(f"{self.cur}: cond of {t.spelling}")
        return e.code

    # ---- expressions -----------------------------------------------------------------------------------------------
    def E(self, c):
        k = c.kind
        if k == K.BINARY_OPERATOR and norm(c.type).kind in INT_KINDS and any(x.kind == K.CXX_UNARY_EXPR for x in c.walk_preorder()):
            v = evaluate(c)
            if v and v[0] == "i":
                return self.int_lit(v[1], c.type)
        if k in (K.PAREN_EXPR, K.BINARY_OPERATOR, K.UNARY_OPERATOR) and norm(c.type).kind in INT_KINDS:
            toks = list(c.get_tokens())
            if len(toks) == 1 and toks[0].spelling in CONSTS and toks[0].spelling.isupper() or (
                    len(toks) == 1 and toks[0].spelling in CONSTS):
                return self.int_lit(0, c.type, toks[0].spelling)
        if k == K.PAREN_EXPR:
            e = self.E(children(c)[0])
            return Ex(e.code, e.ty, e.atom, e.isbool, e.null) if e.atom else Ex(e.code, e.ty, False, e.isbool, e.null)
        if k == K.UNEXPOSED_EXPR:
            ch = expr_children(c)
            if len(ch) == 1:
                return self.cast_node(c, ch[0])
            return self.unexposed(c, ch)
        if k == K.CSTYLE_CAST_EXPR:
            ch = expr_children(c)
            return self.cast_node(c, ch[-1])
        if k == K.INTEGER_LITERAL:
            v = evaluate(c)
            return self.int_lit(v[1] if v else 0, c.type, self.macro_name(c))
        if k == K.FLOATING_LITERAL:
            v = evaluate(c)
            t = c.type
            s = (f32lit(v[1]) + ("f32" if f32lit(v[1]) != "f32::MAX" else "")) if prim(t) == "f32" else f64lit(v[1]) + "f64"
            return Ex(s, t, True)
        if k == K.STRING_LITERAL:
            return Ex(f'c"{c.spelling[1:-1]}".as_ptr() as *mut i8', c.type, True)
        if k == K.DECL_REF_EXPR:
            return self.declref(c)
        if k == K.MEMBER_REF_EXPR:
            return self.member(c)
        if k == K.ARRAY_SUBSCRIPT_EXPR:
            b, i = expr_children(c)
            bb = b
            while bb.kind in (K.UNEXPOSED_EXPR, K.PAREN_EXPR) and len(expr_children(bb)) == 1 and not is_arr(bb.type):
                bb = expr_children(bb)[0]
            if is_arr(bb.type):
                be = self.E(bb)
            else:
                be = self.E(b)
            ie = self.E(i)
            ie = self.conv(ie, self.int_t()) if ie.isbool else ie
            if is_arr(be.ty):
                return Ex(f"{be.p()}[{ie.p()} as usize]", c.type, True)
            return Ex(f"*{be.p()}.offset({ie.p()} as isize)", c.type, False)
        if k == K.UNARY_OPERATOR:
            return self.unary(c)
        if k == K.BINARY_OPERATOR:
            return self.binary(c)
        if k == K.COMPOUND_ASSIGNMENT_OPERATOR:
            return self.compound(c, as_stmt=False)
        if k == K.CONDITIONAL_OPERATOR:
            a, b, d = expr_children(c)
            cc = self.cond(a)
            be, de = self.E(b), self.E(d)
            if norm(c.type).kind != T.VOID:
                be, de = self.conv(be, c.type), self.conv(de, c.type)
            return Ex(f"if {cc} {{ {be.code} }} else {{ {de.code} }}", c.type, False)
        if k == K.CALL_EXPR:
            return self.call(c)
        if k == K.CXX_UNARY_EXPR:
            v = evaluate(c)
            return Ex(f"{v[1]}_{prim(c.type) or 'usize'}" if v else "0", c.type, True)
        if k == K.INIT_LIST_EXPR:
            warn(f"{self.cur}: init list in expression")
            return Ex("/*INITLIST*/ unsafe { core::mem::zeroed() }", c.type, False)
        warn(f"{self.cur}: unhandled expr {k.name}")
        return Ex(f"/*{k.name}*/ todo!()", c.type, False)

    def macro_name(self, c):
        toks = list(c.get_tokens())
        if len(toks) == 1 and toks[0].spelling in CONSTS:
            return toks[0].spelling
        return None

    def int_lit(self, v, t, name=None):
        n = prim(t) or "i32"
        if name:
            pn = CONSTS[name]
            while pn in TYPEALIAS:
                pn = TYPEALIAS[pn]
            code = name if pn == n else f"{name} as {n}"
            return Ex(code, t, pn == n)
        if n in ("f32", "f64"):
            return Ex(f"{v}.0{n}", t, True)
        if n == "i32":
            if v == -(1 << 31) % (1 << 64):
                return Ex("i32::MIN", t, True)
            if v >= (1 << 63):
                v -= 1 << 64
            return Ex(str(v), t, True) if v >= 0 and abs(v) < (1 << 31) else (Ex(f"({v}_i32)", t, True) if v < 0 else Ex(f"{v}_i32", t, True))
        if v >= (1 << 63) and n.startswith("i"):
            v -= 1 << 64
        return Ex(f"{v}_{n}", t, v >= 0)

    def cast_node(self, c, child):
        ex = self.E(child)
        to = c.type
        cc = strip(child)
        # null pointer constant
        if norm(to).kind == T.POINTER and cc.kind == K.INTEGER_LITERAL:
            v = evaluate(cc)
            if v and v[1] == 0:
                return Ex("None" if is_fnptr(to) else "null_mut()", to, True, null=True)
        if norm(to).kind == T.POINTER and cc.kind == K.CSTYLE_CAST_EXPR:
            pass
        if norm(to).kind == T.POINTER:
            q = cc
            while q.kind in (K.UNEXPOSED_EXPR, K.PAREN_EXPR) and len(expr_children(q)) == 1:
                q = strip(expr_children(q)[0])
            if q.kind == K.BINARY_OPERATOR and lib.clang_getCursorBinaryOperatorKind(q) == 6:
                l, r = expr_children(q)
                for a, b in ((l, r), (r, l)):
                    aa = strip(a)
                    while aa.kind in (K.UNEXPOSED_EXPR, K.PAREN_EXPR) and len(expr_children(aa)) == 1:
                        aa = strip(expr_children(aa)[0])
                    if aa.kind == K.CSTYLE_CAST_EXPR and is_ptr(expr_children(aa)[-1].type) and is_int(aa.type):
                        pe = self.E(expr_children(aa)[-1])
                        be = self.E(b)
                        return Ex(f"({pe.p()} as *mut u8).offset({be.p()} as isize) as {rty(to)}", to, False)
        # function name decaying to pointer
        r = cc.referenced if cc.kind == K.DECL_REF_EXPR else None
        if r is not None and r.kind == K.FUNCTION_DECL and norm(to).kind == T.POINTER:
            self.fnaddr.add(r.spelling)
            ft = norm(r.type)
            src = "unsafe extern \"C\" fn(" + ", ".join(pty(x) for x in ft.argument_types()) + ")" +                 ("" if norm(ft.get_result()).kind == T.VOID else " -> " + rty(ft.get_result()))
            dst = rty(to)
            fcode = self.ext_cb_ref(r, ex.code)
            if dst.startswith("Option<") and dst[7:-1] != src:
                # a function whose address is stored through an incompatible function pointer type
                return Ex(f"Some(core::mem::transmute::<{src}, {dst[7:-1]}>({fcode}))", to, True)
            return Ex(f"Some({fcode})", to, True)
        return self.conv(ex, to)

    def unexposed(self, c, ch):
        warn(f"{self.cur}: unexposed expr with {len(ch)} children")
        return self.E(ch[0]) if ch else Ex("todo!()", c.type)

    def declref(self, c):
        r = c.referenced
        if r is None:
            warn(f"{self.cur}: unresolved ref {c.spelling}")
            return Ex(c.spelling, c.type, True)
        if r.kind == K.ENUM_CONSTANT_DECL:
            n = r.spelling
            v = r.enum_value
            t = c.type
            if n in CONSTS:
                pn = prim_of_const(n)
                want = prim(t) or "i32"
                return Ex(n if pn == want else f"{n} as {want}", t, pn == want)
            return self.int_lit(v, t)
        if r.kind == K.FUNCTION_DECL:
            return Ex(self.fn_ref(r), c.type, True)
        if r.kind in (K.VAR_DECL, K.PARM_DECL):
            name = r.spelling
            if r.kind == K.VAR_DECL and r.semantic_parent.kind == K.TRANSLATION_UNIT:
                if name in self.sib_vars:
                    return Ex(f"super::{self.sib_vars[name]}::{ident(name)}", c.type, True)
                if name not in self.main_vars:
                    self.ext_vars.setdefault(name, r)
                    return Ex(f"ext::{ident(name)}", c.type, True)
            if r.hash in self.static_locals:
                return Ex(self.static_locals[r.hash], c.type, True)
            return Ex(ident(name), c.type, True)
        warn(f"{self.cur}: declref kind {r.kind.name}")
        return Ex(c.spelling, c.type, True)

    def rec_name(self, t):
        c = ctype(t)
        n = t.spelling.replace("const ", "").replace("struct ", "").replace("union ", "").strip()
        if n in RECS:
            return n
        dn = c.get_declaration().spelling
        return dn if dn in RECS else None

    def expr_rec(self, c):
        """Rust record name of the (struct / pointer to struct) value of expression c"""
        t = c.type
        n = norm(t)
        if n.kind == T.POINTER:
            t = n.get_pointee()
        r = self.rec_name(t)
        if r:
            return r
        c = strip(c)
        while c.kind == K.UNEXPOSED_EXPR and len(expr_children(c)) == 1:
            c = strip(expr_children(c)[0])
        if c.kind == K.MEMBER_REF_EXPR:
            ch = expr_children(c)
            pr = self.expr_rec(ch[0])
            if pr:
                path = self.find_path(pr, c.spelling)
                if path:
                    cur = pr
                    for hop in path:
                        ft = RECS.get(cur, {}).get(hop)
                        if ft is None:
                            return None
                        ft = ft.strip()
                        if ft.startswith("[") and ft.endswith("]") and ";" in ft:
                            ft = ft[1:ft.rindex(";")]
                        if ft.startswith("*mut "):
                            ft = ft[5:]
                        cur = ft.strip()
                    return cur if cur in RECS else None
        return None

    def find_path(self, rec, field, seen=0):
        """path of Rust field names from record `rec` to member `field` (through anonymous members)"""
        flds = RECS.get(rec, {})
        if field in flds:
            return [field]
        if field in BITF.get(rec, ()):
            return [field]
        for fn, ft in flds.items():
            ft = ft.strip()
            if "_anon_" in ft and ft in RECS:
                p = self.find_path(ft, field)
                if p:
                    return [fn] + p
        for ft in BITF and []:
            pass
        return None

    def member(self, c):
        ch = expr_children(c)
        # members of anonymous struct / union members are reached through the enclosing record
        while True:
            b0 = strip(ch[0])
            while b0.kind == K.UNEXPOSED_EXPR and len(expr_children(b0)) == 1:
                b0 = strip(expr_children(b0)[0])
            if b0.kind == K.MEMBER_REF_EXPR and b0.referenced is not None and b0.referenced.kind == K.FIELD_DECL and (
                    not b0.referenced.spelling or "anonymous" in b0.referenced.spelling or "(unnamed" in b0.referenced.spelling):
                ch = expr_children(b0)
                continue
            break
        base = self.E(ch[0])
        bt = norm(ch[0].type)
        arrow = bt.kind == T.POINTER
        rt = bt.get_pointee() if arrow else ch[0].type
        fld = c.referenced
        name = c.spelling
        rec = self.rec_name(rt) or self.expr_rec(ch[0])
        if rec is None and ctype(rt).kind == T.RECORD and ctype(rt).get_declaration().location.file is not None and \
                self.is_main(ctype(rt).get_declaration()) and not ctype(rt).get_declaration().is_anonymous():
            pre = f"(*{base.p()})" if arrow else base.p()
            if fld is not None and fld.is_bitfield():
                return Ex(pre + f".{ident(name)}()", c.type, True)
            return Ex(pre + f".{ident(name)}", c.type, True)
        if rec is None:
            # maybe a local anonymous struct
            ln = self.local_structs.get(ctype(rt).get_declaration().hash)
            if ln:
                path = [ident(name)]
                code = f"(*{base.p()}).{path[0]}" if arrow else f"{base.p()}.{path[0]}"
                return Ex(code, c.type, True)
            warn(f"{self.cur}: member {name} of unknown record {rt.spelling}")
            return Ex(f"{base.p()}.{ident(name)}", c.type, True)
        path = self.find_path(rec, name)
        if path is None:
            warn(f"{self.cur}: member {name} not found in {rec}")
            path = [name]
        isbf = fld is not None and fld.is_bitfield()
        pre = f"(*{base.p()})" if arrow else base.p()
        if isbf:
            code = pre + "".join("." + ident(x) for x in path[:-1]) + f".{ident(path[-1])}()"
            return Ex(code, c.type, True)
        return Ex(pre + "".join("." + ident(x) for x in path), c.type, True)

    def unary(self, c):
        op = UN.get(lib.clang_getCursorUnaryOperatorKind(c), "?")
        ch = expr_children(c)
        if op == "&":
            tgt = strip(ch[0])
            e = self.E(ch[0])
            # &arr[0], &*p etc: use addr_of_mut for places
            if tgt.kind == K.UNARY_OPERATOR and UN.get(lib.clang_getCursorUnaryOperatorKind(tgt)) == "*":
                inner = self.E(expr_children(tgt)[0])
                return self.conv(inner, c.type) if norm(inner.ty).kind == T.POINTER else Ex(f"{inner.p()} as {rty(c.type)}", c.type)
            if tgt.kind == K.MEMBER_REF_EXPR and tgt.referenced is not None and tgt.referenced.is_bitfield():
                warn(f"{self.cur}: address of bitfield")
            if tgt.kind == K.MEMBER_REF_EXPR and tgt.referenced is not None and not tgt.referenced.is_bitfield():
                # `&((T*) 0)->member` (offsetof): the byte offset in the native record
                kids = expr_children(tgt)
                bt = norm(kids[0].type) if kids else None
                if kids and bt is not None and bt.kind == T.POINTER and getattr(self.E(kids[0]), "null", False):
                    rt0 = bt.get_pointee()
                    rec0 = self.rec_name(rt0)
                    path0 = self.find_path(rec0, tgt.spelling) if rec0 else None
                    if rec0 and path0:
                        return Ex(f"(core::mem::offset_of!({rec0}, {'.'.join(ident(x) for x in path0)}) as {rty(c.type)})", c.type, False)
            if is_fnptr(c.type) or norm(ch[0].type).kind in (T.FUNCTIONPROTO,):
                return e
            if tgt.kind == K.ARRAY_SUBSCRIPT_EXPR:
                # `&arr[i]` forms an address only (C allows any i; the index is not bounds-checked, nor dereferenced)
                b, i = expr_children(tgt)
                bb = b
                while bb.kind in (K.UNEXPOSED_EXPR, K.PAREN_EXPR) and len(expr_children(bb)) == 1 and not is_arr(bb.type):
                    bb = expr_children(bb)[0]
                if is_arr(bb.type):
                    be = self.E(bb)
                    ie = self.E(i)
                    ie = self.conv(ie, self.int_t()) if ie.isbool else ie
                    el = rty(norm(bb.type).element_type)
                    return Ex(f"core::ptr::addr_of_mut!({be.code}).cast::<{el}>().wrapping_offset({ie.p()} as isize) as {rty(c.type)}", c.type, False)
            return Ex(f"core::ptr::addr_of_mut!({e.code}) as {rty(c.type)}", c.type, False)
        if op == "*":
            e = self.E(ch[0])
            if is_fnptr(ch[0].type):
                return Ex(e.code, ch[0].type, e.atom)
            return Ex(f"*{e.p()}", c.type, False)
        if op in ("+",):
            return self.E(ch[0])
        if op == "-":
            e = self.E(ch[0])
            if e.isbool:
                e = self.conv(e, self.int_t())
            v = evaluate(c)
            if v and not is_flt(c.type) and v[0] == "i":
                return self.int_lit(v[1], c.type)
            return Ex(f"-{e.p()}", c.type, False)
        if op == "~":
            e = self.E(ch[0])
            return Ex(f"!{e.p()}", c.type, False)
        if op == "!":
            e = self.E(ch[0])
            return Ex(f"!({self.to_cond(e)})", c.type, False, isbool=True)
        if op in ("post++", "post--", "pre++", "pre--"):
            return self.incdec(c, op, ch[0], as_stmt=False)
        warn(f"{self.cur}: unary {op}")
        return Ex("todo!()", c.type)

    def is_simple_place(self, c):
        c = strip(c)
        for x in c.walk_preorder():
            if x.kind in (K.CALL_EXPR, K.COMPOUND_ASSIGNMENT_OPERATOR) or (
                    x.kind == K.BINARY_OPERATOR and BIN.get(lib.clang_getCursorBinaryOperatorKind(x), "") in ("=",)) or (
                    x.kind == K.UNARY_OPERATOR and UN.get(lib.clang_getCursorUnaryOperatorKind(x), "").startswith(("post", "pre"))):
                return False
        return True

    def lvalue(self, c):
        """-> (getter, setter(valcode), prelude). bitfields use accessor/setter"""
        cc = strip(c)
        while cc.kind == K.UNEXPOSED_EXPR and len(expr_children(cc)) == 1 and norm(cc.type).kind == norm(expr_children(cc)[0].type).kind:
            cc = strip(expr_children(cc)[0])
        pre = ""
        if cc.kind == K.MEMBER_REF_EXPR and cc.referenced is not None and cc.referenced.is_bitfield():
            e = self.member(cc)
            g = e.code
            s = g[:-2]  # strip "()"
            # path.name -> path.set_name
            i = s.rfind(".")
            setter = s[:i + 1] + "set_" + s[i + 1:]
            ty = prim(cc.type) or "u8"
            return g, (lambda v: f"{setter}(({v}) as {ty});"), "", cc.type
        e = self.E(cc)
        if self.is_simple_place(cc):
            return e.code, (lambda v: f"{e.code} = {v};"), "", cc.type
        tmp = self.newtmp()
        pre = f"let {tmp} = core::ptr::addr_of_mut!({e.code}); "
        return f"*{tmp}", (lambda v: f"*{tmp} = {v};"), pre, cc.type

    def incdec(self, c, op, operand, as_stmt):
        g, s, pre, lt = self.lvalue(operand)
        d = "+" if "++" in op else "-"
        if is_ptr(lt):
            new = f"{g}.offset({'1' if d == '+' else '-1'})"
        elif is_flt(lt):
            new = f"{g} {d} 1.0"
        else:
            new = f"{g} {d} 1"
        if as_stmt:
            return pre + s(new)
        if op.startswith("pre"):
            return Ex(f"{{ {pre}{s(new)} {g} }}", c.type, True)
        t = self.newtmp()
        return Ex(f"{{ {pre}let {t} = {g}; {s(new)} {t} }}", c.type, True)

    def fma_try(self, c, op, l, r, rt):
        """clang -ffp-contract=on: + / - with a same-type multiply operand (or its negation)"""
        def mulparts(x):
            x = strip(x)
            neg = False
            if x.kind == K.UNARY_OPERATOR and UN.get(lib.clang_getCursorUnaryOperatorKind(x)) == "-":
                inner = strip(expr_children(x)[0])
                if norm(x.type).kind == norm(rt).kind:
                    x, neg = inner, True
            if (x.kind == K.BINARY_OPERATOR and lib.clang_getCursorBinaryOperatorKind(x) == 3 and norm(x.type).kind == norm(rt).kind
                    and norm(x.type).kind in FLT_KINDS):
                a, b = expr_children(x)
                return a, b, neg
            return None
        t = FLT_KINDS[norm(rt).kind]

        def fm(a, b, cc):
            self.fma[self.cur] = self.fma.get(self.cur, 0) + 1
            return f"fmadds({a}, {b}, {cc})" if t == "f32" else f"({a}).mul_add({b}, {cc})"

        def negc(e, flip):
            return f"-{e.p()}" if flip else e.code
        mp = mulparts(l)
        if mp:
            a, b, neg = mp
            ae, be = self.E(a), self.E(b)
            re_ = self.E(r)
            return Ex(fm(negc(ae, neg), be.code, negc(re_, op == "-")), rt, True)
        mp = mulparts(r)
        if mp:
            a, b, neg = mp
            ae, be = self.E(a), self.E(b)
            le = self.E(l)
            flip = neg if op == "+" else (not neg)
            return Ex(fm(negc(ae, flip), be.code, le.code), rt, True)
        return None

    def binary(self, c):
        opk = lib.clang_getCursorBinaryOperatorKind(c)
        op = BIN.get(opk, "?")
        ch = expr_children(c)
        l, r = ch[0], ch[1]
        if op == "=":
            return self.assign(c, as_stmt=False)
        if op == ",":
            le = self.E(l)
            re_ = self.E(r)
            return Ex(f"{{ {le.code}; {re_.code} }}", c.type, True, isbool=re_.isbool)
        if op in ("&&", "||"):
            a, b = self.to_cond(self.E(l)), self.to_cond(self.E(r))
            return Ex(f"({a}) {op} ({b})", c.type, False, isbool=True)
        # constant folding of integer constants keeps the output readable
        if op in ("+", "-") and is_flt(c.type):
            f = self.fma_try(c, op, l, r, c.type)
            if f is not None:
                return f
        le, re_ = self.E(l), self.E(r)
        lt, rt_ = norm(l.type), norm(r.type)
        if op in ("<", ">", "<=", ">=", "==", "!="):
            if le.isbool:
                le = self.conv(le, self.int_t())
            if re_.isbool:
                re_ = self.conv(re_, self.int_t())
            if le.null or re_.null:
                x = re_ if le.null else le
                if is_fnptr(x.ty):
                    return Ex(f"{x.p()}.{'is_none' if op == '==' else 'is_some'}()", c.type, False, isbool=True)
                return Ex(f"{'!' if op == '!=' else ''}{x.p()}.is_null()", c.type, False, isbool=True)
            return Ex(f"{le.p()} {op} {re_.p()}", c.type, False, isbool=True)
        if le.isbool:
            le = self.conv(le, self.int_t())
        if re_.isbool:
            re_ = self.conv(re_, self.int_t())
        return self.arith(op, le, re_, c.type, lt, rt_)

    def arith(self, op, le, re_, rtype, lt, rt_):
        # pointer arithmetic
        if lt.kind == T.POINTER and rt_.kind in INT_KINDS:
            s = f"{le.p()}.offset({re_.p()} as isize)" if op == "+" else f"{le.p()}.offset(-({re_.p()} as isize))"
            return Ex(s, rtype, False)
        if rt_.kind == T.POINTER and lt.kind in INT_KINDS and op == "+":
            return Ex(f"{re_.p()}.offset({le.p()} as isize)", rtype, False)
        if lt.kind == T.POINTER and rt_.kind == T.POINTER and op == "-":
            return Ex(f"{le.p()}.offset_from({re_.p()}) as {prim(rtype)}", rtype, False)
        k = norm(rtype).kind
        if k in FLT_KINDS:
            return Ex(f"{le.p()} {op} {re_.p()}", rtype, False)
        if k in INT_KINDS:
            n = INT_KINDS[k][0]
            if op in ("+", "-", "*", "&", "|", "^"):
                return Ex(f"{le.p()} {op} {re_.p()}", rtype, False)
            if op == "/":
                rv = re_.code
                if re.fullmatch(r"\d+(_\w+)?", rv) and int(re.match(r"\d+", rv).group()) != 0:
                    return Ex(f"{le.p()} / {re_.p()}", rtype, False)
                return Ex(f"melee_runtime::cabi::{'sdiv' if INT_KINDS[k][2] else 'udiv'}({le.code}, {re_.code})", rtype, True) if n in ("i32", "u32") else Ex(f"{le.p()} / {re_.p()}", rtype, False)
            if op == "%":
                rv = re_.code
                if re.fullmatch(r"\d+(_\w+)?", rv) and int(re.match(r"\d+", rv).group()) != 0:
                    return Ex(f"{le.p()} % {re_.p()}", rtype, False)
                return Ex(f"melee_runtime::cabi::{'srem' if INT_KINDS[k][2] else 'urem'}({le.code}, {re_.code})", rtype, True) if n in ("i32", "u32") else Ex(f"{le.p()} % {re_.p()}", rtype, False)
            if op in ("<<", ">>"):
                rv = re_.code
                if re.fullmatch(r"\d+(_\w+)?", rv) and int(re.match(r"\d+", rv).group()) < INT_KINDS[k][1]:
                    return Ex(f"{le.p()} {op} {re_.p()}", rtype, False)
                if n == "u32":
                    return Ex(f"melee_runtime::cabi::{'shl' if op == '<<' else 'lshr'}({le.code}, {re_.p()} as u32)", rtype, True)
                if n == "i32":
                    if op == "<<":
                        return Ex(f"melee_runtime::cabi::shl({le.p()} as u32, {re_.p()} as u32) as i32", rtype, False)
                    return Ex(f"melee_runtime::cabi::ashr({le.code}, {re_.p()} as u32)", rtype, True)
                return Ex(f"{le.p()} {op} {re_.p()}", rtype, False)
        warn(f"{self.cur}: arith {op} on {norm(rtype).spelling}")
        return Ex(f"{le.p()} {op} {re_.p()}", rtype, False)

    def assign(self, c, as_stmt):
        ch = expr_children(c)
        lhs, rhs = ch[0], ch[1]
        g, s, pre, lt = self.lvalue(lhs)
        rv = self.E(rhs)
        rv = self.conv(rv, lt) if not (norm(lt).kind == T.RECORD) else rv
        if as_stmt:
            return pre + s(rv.code)
        return Ex(f"{{ {pre}{s(rv.code)} {g} }}", lt, True)

    def usual(self, a, b):
        """usual arithmetic conversions (for compound assignment)"""
        a, b = norm(a), norm(b)
        if a.kind in FLT_KINDS or b.kind in FLT_KINDS:
            order = [T.FLOAT, T.DOUBLE, T.LONGDOUBLE]
            ka = order.index(a.kind) if a.kind in FLT_KINDS else -1
            kb = order.index(b.kind) if b.kind in FLT_KINDS else -1
            return a if ka >= kb else b
        def promote(t):
            n, w, s = INT_KINDS[t.kind]
            return t if w >= 32 else self.int_t()
        a, b = promote(a), promote(b)
        wa, wb = INT_KINDS[a.kind][1], INT_KINDS[b.kind][1]
        sa, sb = INT_KINDS[a.kind][2], INT_KINDS[b.kind][2]
        if wa > wb:
            return a
        if wb > wa:
            return b
        if sa == sb:
            return a
        return a if not sa else b

    def compound(self, c, as_stmt):
        opk = lib.clang_getCursorBinaryOperatorKind(c)
        op = BIN[opk][:-1]
        ch = expr_children(c)
        lhs, rhs = ch[0], ch[1]
        g, s, pre, lt = self.lvalue(lhs)
        rv = self.E(rhs)
        if rv.isbool:
            rv = self.conv(rv, self.int_t())
        rt_ = rhs.type
        le = Ex(g, lt, True)
        if is_ptr(lt):
            new = self.arith(op, le, rv, lt, norm(lt), norm(rt_))
            val = new.code
        else:
            ct = self.usual(lt, rt_)
            if op in ("<<", ">>"):
                ct = self.usual(lt, lt)
            if is_flt(ct) and op in ("+", "-"):
                # clang fuses a multiply on the right of a compound add/sub
                rr = strip(rhs)
                if (rr.kind == K.BINARY_OPERATOR and lib.clang_getCursorBinaryOperatorKind(rr) == 3 and
                        norm(rr.type).kind == norm(ct).kind and norm(lt).kind == norm(ct).kind):
                    a, b = [self.E(x) for x in expr_children(rr)]
                    self.fma[self.cur] = self.fma.get(self.cur, 0) + 1
                    t = FLT_KINDS[norm(ct).kind]
                    if op == "+":
                        val = f"fmadds({a.code}, {b.code}, {g})" if t == "f32" else f"({a.code}).mul_add({b.code}, {g})"
                    else:
                        val = f"fmadds(-{a.p()}, {b.code}, {g})" if t == "f32" else f"(-{a.p()}).mul_add({b.code}, {g})"
                    return (pre + s(val)) if as_stmt else Ex(f"{{ {pre}{s(val)} {g} }}", lt, True)
            lc = self.conv(le, ct)
            rc = self.conv(rv, ct if op not in ("<<", ">>") else rt_)
            new = self.arith(op, lc, rc, ct, norm(ct), norm(ct))
            val = self.conv(new, lt).code
        if as_stmt:
            return pre + s(val)
        return Ex(f"{{ {pre}{s(val)} {g} }}", lt, True)

    # ---- calls -----------------------------------------------------------------------------------------------------
    def call(self, c):
        ch = expr_children(c)
        callee = strip(ch[0])
        args = ch[1:]
        inner = callee
        while inner.kind == K.UNEXPOSED_EXPR and expr_children(inner):
            inner = strip(expr_children(inner)[0])
        d = inner.referenced if inner.kind == K.DECL_REF_EXPR else None
        name = d.spelling if d is not None and d.kind == K.FUNCTION_DECL else None
        if name in ("__assert",):
            return Ex(f'panic!("{self.cur}: assertion failed")', c.type, True)
        if name == "OSReport":
            return self.osreport(c, args)
        if name == "__builtin_va_start":
            return Ex(f"{self.E(args[0]).code} = &raw mut va_store", c.type, True)
        if name == "__builtin_va_end":
            return Ex("()", c.type, True)
        if name == "__va_arg":
            return Ex(f"(*{self.E(args[0]).p()}).arg({self.E(args[1]).code})", c.type, True)
        if name == "memzero":
            a, b = [self.E(x) for x in args]
            return Ex(f"core::ptr::write_bytes({a.p()} as *mut u8, 0, {b.p()} as usize)", c.type, True)
        if name == "memset":
            a, b, n = [self.E(x) for x in args]
            return Ex(f"core::ptr::write_bytes({a.p()} as *mut u8, {b.p()} as u8, {n.p()} as usize)", c.type, True)
        if name in ("memcpy", "memmove"):
            a, b, n = [self.E(x) for x in args]
            return Ex(f"core::ptr::copy({b.p()} as *const u8, {a.p()} as *mut u8, {n.p()} as usize)", c.type, True)
        if name == "PSVECNormalize":
            a, b = [self.E(x) for x in args]
            return Ex(f"{{ let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 {{ x: (*{a.p()}).x, y: (*{a.p()}).y, z: (*{a.p()}).z }}); (*{b.p()}).x = __n.x; (*{b.p()}).y = __n.y; (*{b.p()}).z = __n.z; }}", c.type, True)
        if name is not None:
            fe = self.fn_ref(d)
            av = []
            nfixed = len(list(d.get_arguments())) if d.type.is_function_variadic() else len(args)
            for a in args[:nfixed]:
                e = self.E(a)
                if e.isbool:
                    e = self.conv(e, a.type)
                av.append(e.code)
            if d.type.is_function_variadic():
                # a variadic callee other than OSReport takes its varargs as one `&[*const c_void]` (pointer / integer
                # arguments, each widened to a pointer-sized word; a float vararg is not supported)
                vv = []
                for a in args[nfixed:]:
                    e = self.E(a)
                    k = norm(a.type).kind
                    if k == T.POINTER:
                        vv.append(f"{e.code} as *const c_void")
                    elif k in INT_KINDS:
                        vv.append(f"({e.code}) as isize as *const c_void")
                    else:
                        warn(f"{self.cur}: float vararg of {name} (hand check)")
                        vv.append(f"({e.code}).to_bits() as isize as *const c_void")
                av.append("&[" + ", ".join(vv) + "]")
            return Ex(f"{fe}({', '.join(av)})", c.type, True)
        # indirect call through a function pointer value
        fe = self.E(callee)
        av = []
        for a in args:
            e = self.E(a)
            if e.isbool:
                e = self.conv(e, a.type)
            av.append(e.code)
        return Ex(f"{fe.p()}.unwrap()({', '.join(av)})", c.type, True)

    def osreport(self, c, args):
        fmt = None
        a0 = strip(args[0])
        while a0.kind == K.UNEXPOSED_EXPR:
            a0 = strip(expr_children(a0)[0])
        if a0.kind == K.STRING_LITERAL:
            fmt = a0.spelling[1:-1]
        rest = [self.E(a) for a in args[1:]]
        if fmt is None:
            return Ex("()", c.type, True)
        specs = list(re.finditer(r"%([-+ #0]*)(\d*)(?:\.(\d+))?(l|h|ll)?([diuxXfgeEscp%])", fmt))
        out, pos, ai = "", 0, 0
        argc = []
        for m in specs:
            out += fmt[pos:m.start()].replace("{", "{{").replace("}", "}}")
            pos = m.end()
            ch = m.group(5)
            if ch == "%":
                out += "%"
                continue
            flags, width, prec = m.group(1), m.group(2), m.group(3)
            e = rest[ai]
            if ch in "diuxX" and norm(args[1 + ai].type).kind == T.POINTER:
                # a pointer printed as an integer (PPC: 32-bit address)
                e = Ex(f"(({e.code}) as usize as u32)", args[1 + ai].type, True)
            ai += 1
            spec = ""
            if "0" in flags and width:
                spec = "0" + width
            elif width:
                spec = (">" if "-" not in flags else "<") + width if False else width
            if ch in "diu":
                argc.append(e.code)
                out += "{:" + spec + "}" if spec else "{}"
            elif ch in "xX":
                argc.append(e.code)
                out += "{:" + spec + ("x" if ch == "x" else "X") + "}"
            elif ch in "feEg":
                argc.append(e.code)
                out += "{:" + spec + (f".{prec}" if prec else ".6") + "}"
            elif ch == "c":
                argc.append(f"({e.code} as u8 as char)")
                out += "{}"
            elif ch == "p":
                argc.append(e.code)
                out += "{:p}"
            else:
                argc.append(f"if ({e.p()} as *const core::ffi::c_char).is_null() {{ std::borrow::Cow::Borrowed(\"(null)\") }} else {{ core::ffi::CStr::from_ptr({e.p()} as *const core::ffi::c_char).to_string_lossy() }}")
                out += "{}"
        out += fmt[pos:].replace("{", "{{").replace("}", "}}")
        out = out.replace("\n", "\\n")
        self.uses_osreport = True
        return Ex(f'ext::OSReport(&format!("{out}"{"".join(", " + a for a in argc)}))', c.type, True)

    # ---- statements ------------------------------------------------------------------------------------------------
    def S(self, c, ind):
        """braced block text (opening brace not indented, closing brace at indent `ind`)"""
        p = "    " * ind
        return "{\n" + self.body_lines(c, ind + 1) + p + "}"

    def line(self, c, ind):
        t = self.stmt(c, ind)
        if not t:
            return ""
        return "    " * ind + t + "\n"

    def body_lines(self, c, ind):
        if c.kind == K.COMPOUND_STMT:
            return self.seq(children(c), ind)
        return self.seq([c], ind)

    # ---- goto support: structured regions over a sibling list --------------------------------------------------------
    def gotos_in(self, c):
        out = set()
        for x in c.walk_preorder():
            if x.kind == K.GOTO_STMT:
                for y in x.get_children():
                    if y.kind == K.LABEL_REF:
                        out.add(y.spelling)
        return out

    def body_label(self, f):
        """label at the top of a for loop's body"""
        body = children(f)[-1]
        if body.kind == K.COMPOUND_STMT:
            for x in children(body):
                if x.kind == K.LABEL_STMT:
                    return x.spelling
                if x.kind != K.DECL_STMT:
                    return None
        return None

    def plain(self, stmts, ind):
        out = ""
        for st in stmts:
            if st.kind == K.LABEL_STMT:
                kids = children(st)
                out += "".join(self.line(x, ind) for x in kids)
            else:
                out += self.line(st, ind)
        return out

    def seq(self, stmts, ind):
        n = len(stmts)
        if n == 0:
            return ""
        p = "    " * ind
        if not any(x.kind in (K.LABEL_STMT, K.GOTO_STMT) or True for x in stmts[:0]):
            pass
        tg = [self.gotos_in(x) for x in stmts]
        if not any(tg):
            return self.plain(stmts, ind)
        # pattern A: for loop whose body starts with a label that a later sibling jumps to
        for i, st in enumerate(stmts):
            if st.kind == K.FOR_STMT:
                L = self.body_label(st)
                if L and L not in self.consumed and any(L in tg[j] for j in range(i + 1, n)):
                    e = max(j for j in range(i + 1, n) if L in tg[j])
                    self.consumed.add(L)
                    self.tmp += 1
                    pl = f"'p{self.tmp}"
                    self.gmap[L] = ("continue", pl)
                    try:
                        head = self.seq(stmts[:i], ind)
                        init, cnd, inc, body = self.for_parts(st)
                        pre = (p + "    " + self.stmt(init, ind + 1) + NL) if init is not None else ""
                        lp_text = self.loop(st, ind + 2, skip_init=True)
                        tail = self.seq(stmts[i + 1:e + 1], ind + 2)
                        txt = (p + "{" + NL + pre + p + "    " + pl + ": loop {" + NL + "    " * (ind + 2) + lp_text + NL +
                               tail + "    " * (ind + 2) + "break;" + NL + p + "    }" + NL + p + "}" + NL)
                    finally:
                        del self.gmap[L]
                    return head + txt + self.seq(stmts[e + 1:], ind)
        labs = {x.spelling: i for i, x in enumerate(stmts) if x.kind == K.LABEL_STMT and x.spelling not in self.consumed}
        regions = []  # (kind, label, start, end)
        for L, b in labs.items():
            idx = [j for j in range(n) if L in tg[j]]
            if not idx:
                continue
            if all(j < b for j in idx):
                regions.append(["fwd", L, min(idx), b - 1])
            elif all(j >= b for j in idx):
                regions.append(["back", L, b, max(idx)])
            else:
                raise NotImplementedError("label with gotos on both sides")
        # a forward jump to the statement right after a backward loop leaves the loop
        brk = {}
        for r in list(regions):
            if r[0] == "back":
                for r2 in list(regions):
                    if r2[0] == "fwd" and r2[3] == r[3] and r2[2] >= r[2]:
                        regions.remove(r2)
                        brk.setdefault(r[1], []).append(r2[1])
        fw = [r for r in regions if r[0] == "fwd"]
        changed = True
        while changed:
            changed = False
            for r1 in fw:
                for r2 in fw:
                    if r1 is not r2 and r1[2] < r2[2] <= r1[3] < r2[3]:
                        r2[2] = r1[2]
                        changed = True
        if not regions:
            return self.plain(stmts, ind)
        regions.sort(key=lambda r: (r[2], -r[3]))
        kind, L, a, e = regions[0]
        head = self.seq(stmts[:a], ind)
        self.tmp += 1
        self.consumed.add(L)
        if kind == "fwd":
            bl = f"'b{self.tmp}"
            self.gmap[L] = ("break", bl)
            try:
                inner = self.seq(stmts[a:e + 1], ind + 1)
            finally:
                del self.gmap[L]
            txt = p + f"{bl}: {{" + NL + inner + p + "}" + NL
        else:
            wl = f"'w{self.tmp}"
            self.gmap[L] = ("continue", wl)
            for L2 in brk.get(L, []):
                self.gmap[L2] = ("break", wl)
                self.consumed.add(L2)
            try:
                inner = self.seq(stmts[a:e + 1], ind + 1)
            finally:
                del self.gmap[L]
                for L2 in brk.get(L, []):
                    del self.gmap[L2]
            txt = p + f"{wl}: loop {{" + NL + inner + p + "    break;" + NL + p + "}" + NL
        return head + txt + self.seq(stmts[e + 1:], ind)

    def block(self, c, ind):
        return self.S(c, ind)

    def stmt(self, c, ind):
        c = strip(c)
        k = c.kind
        p = "    " * ind
        if k == K.COMPOUND_STMT:
            return self.S(c, ind)
        if k == K.NULL_STMT:
            return ""
        if k == K.DO_STMT:
            dch = children(c)
            v = evaluate(dch[1])
            if v and v[0] == "i" and v[1] == 0 and dch[0].kind == K.COMPOUND_STMT and not any(
                    x.kind in (K.BREAK_STMT, K.CONTINUE_STMT) for x in dch[0].walk_preorder()):
                if all(x.kind == K.DECL_STMT for x in children(dch[0])):
                    return ""  # PAD_STACK
                return self.S(dch[0], ind)
        if k == K.DECL_STMT:
            return self.declstmt(c, ind)
        if k == K.RETURN_STMT:
            ch = expr_children(c)
            if not ch:
                return "return;"
            e = self.conv(self.E(ch[0]), self.cur_ret)
            return f"return {e.code};"
        if k == K.IF_STMT:
            ch = children(c)
            cond = self.cond(ch[0])
            s = f"if {cond} " + self.block(ch[1], ind)
            if len(ch) > 2:
                if ch[2].kind == K.IF_STMT:
                    s += " else " + self.stmt(ch[2], ind)
                else:
                    s += " else " + self.block(ch[2], ind)
            return s
        if k in (K.FOR_STMT, K.WHILE_STMT, K.DO_STMT):
            return self.loop(c, ind)
        if k == K.SWITCH_STMT:
            return self.switch(c, ind)
        if k == K.BREAK_STMT:
            lp = self.loops[-1]
            return f"break {lp[0]};"
        if k == K.CONTINUE_STMT:
            lp = [x for x in self.loops if x[2] == "loop"][-1]
            return f"break {lp[1]};"
        if k == K.GOTO_STMT:
            lab = [y.spelling for y in c.get_children() if y.kind == K.LABEL_REF][0]
            if lab not in self.gmap:
                raise NotImplementedError("goto " + lab)
            kind, rl = self.gmap[lab]
            return f"{kind} {rl};"
        if k == K.LABEL_STMT:
            raise NotImplementedError("label")
        if k == K.BINARY_OPERATOR:
            opk = lib.clang_getCursorBinaryOperatorKind(c)
            if BIN.get(opk) == "=":
                return self.assign(c, as_stmt=True)
            if BIN.get(opk) == ",":
                a, b = expr_children(c)
                return self.stmt(a, ind) + " " + self.stmt(b, ind)
            e = self.E(c)
            return e.code + ";"
        if k == K.COMPOUND_ASSIGNMENT_OPERATOR:
            return self.compound(c, as_stmt=True)
        if k == K.UNARY_OPERATOR:
            op = UN.get(lib.clang_getCursorUnaryOperatorKind(c))
            if op in ("post++", "post--", "pre++", "pre--"):
                return self.incdec(c, op, expr_children(c)[0], as_stmt=True)
            e = self.E(c)
            return f"let _ = {e.code};"
        if k == K.CONDITIONAL_OPERATOR:
            a, b, d = expr_children(c)
            return f"if {self.cond(a)} {{ {self.stmt(b, ind)} }} else {{ {self.stmt(d, ind)} }}"
        e = self.E(c)
        if k == K.CALL_EXPR and norm(c.type).kind != T.VOID and not e.code.startswith("panic"):
            return f"let _ = {e.code};"
        if k == K.UNEXPOSED_EXPR:
            return e.code + ";"
        return e.code + ";"

    def uses_continue(self, c):
        def walk(x, depth):
            for ch in x.get_children():
                if ch.kind == K.CONTINUE_STMT and depth == 0:
                    return True
                d = depth + (1 if ch.kind in (K.FOR_STMT, K.WHILE_STMT, K.DO_STMT) else 0)
                if walk(ch, d):
                    return True
            return False
        return walk(c, 0)

    def loop(self, c, ind, skip_init=False):
        p = "    " * ind
        k = c.kind
        ch = children(c)
        uc = self.uses_continue(c)
        self.tmp += 1
        lab = f"'l{self.tmp}"
        cont = f"'c{self.tmp}" if uc else None
        self.loops.append((lab, cont, "loop"))
        try:
            init = inc = None
            if k == K.WHILE_STMT:
                cond, body = self.cond(ch[0]), ch[1]
            elif k == K.DO_STMT:
                body, cond = ch[0], self.cond(ch[1])
            else:
                init, cnd, inc, body = self.for_parts(c)
                cond = self.cond(cnd) if cnd is not None else "true"
            lp = f"{lab}: " if lab else ""

            def inner(i):
                if uc:
                    return "    " * i + f"{cont}: {{" + NL + self.body_lines(body, i + 1) + "    " * i + "}" + NL
                return self.body_lines(body, i)
            if k == K.DO_STMT:
                brk = f"break {lab};" if lab else "break;"
                return f"{lp}loop {{" + NL + inner(ind + 1) + p + f"    if !({cond}) {{ {brk} }}" + NL + p + "}"
            if skip_init:
                init = None
            if init is not None:
                i2 = ind + 1
                q = "    " * i2
                incs = ("    " * (i2 + 1) + self.stmt(inc, i2 + 1) + NL) if inc is not None else ""
                text = f"{lp}while {cond} {{" + NL + inner(i2 + 1) + incs + q + "}"
                return "{" + NL + q + self.stmt(init, i2) + NL + q + text + NL + p + "}"
            incs = ("    " * (ind + 1) + self.stmt(inc, ind + 1) + NL) if inc is not None else ""
            return f"{lp}while {cond} {{" + NL + inner(ind + 1) + incs + p + "}"
        finally:
            self.loops.pop()

    def switch(self, c, ind):
        p = "    " * ind
        ch = children(c)
        cond = self.E(ch[0])
        if cond.isbool:
            cond = self.conv(cond, self.int_t())
        body = ch[1]
        stmts = children(body) if body.kind == K.COMPOUND_STMT else [body]
        groups = []  # [labels, [stmts]]
        cur = [None]

        def case_pat(x):
            if x.kind == K.DEFAULT_STMT:
                return "_"
            ce = strip(expr_children(x)[0])
            v = evaluate(ce)
            if v is None:
                raise NotImplementedError("case value")
            return str(v[1]) if int(v[1]) >= 0 else "(" + str(v[1]) + ")"

        def add(x):
            if x.kind in (K.CASE_STMT, K.DEFAULT_STMT):
                if cur[0] is None or cur[0][1]:
                    cur[0] = [[], []]
                    groups.append(cur[0])
                cur[0][0].append(case_pat(x))
                kids = children(x)
                inner = kids[-1] if x.kind == K.CASE_STMT else (kids[0] if kids else None)
                if inner is not None and inner.kind in (K.CASE_STMT, K.DEFAULT_STMT):
                    add(inner)
                elif inner is not None:
                    cur[0][1].append(inner)
            else:
                if cur[0] is None:
                    raise NotImplementedError("statement before first case")
                cur[0][1].append(x)
        for x in stmts:
            add(x)
        self.tmp += 1
        lab = "'s" + str(self.tmp)
        self.loops.append((lab, None, "switch"))
        try:
            arms = []
            # a labelled block that ends a case group and is jumped to from earlier groups (`case A: x = 1; goto L;
            # ... case C: x = 3; L: { ...; break; }`): the arms set a flag and the block runs once after the match
            # `L: stmt; break;` at the end of a group is the same as a labelled block that ends the group
            groups = [(pj, (sj[:-1] if (len(sj) >= 2 and sj[-1].kind == K.BREAK_STMT and sj[-2].kind in (K.LABEL_STMT, K.GOTO_STMT)) else sj)) for pj, sj in groups]
            labtail = {}   # group index -> (label stmt, flag)
            gotoflag = {}  # group index -> flag
            for gi, (pats, sts) in enumerate(groups):
                if sts and sts[-1].kind == K.LABEL_STMT:
                    L = sts[-1].spelling
                    users = [gj for gj, (pj, sj) in enumerate(groups) if gj != gi and any(L in self.gotos_in(x) for x in sj)]
                    if users:
                        if any(gj > gi for gj in users):
                            raise NotImplementedError("switch: goto into an earlier case label")
                        self.tmp += 1
                        flag = f"jl{self.tmp}"
                        labtail[gi] = (sts[-1], flag)
                        for gj in users:
                            lastj = groups[gj][1][-1]
                            if lastj.kind != K.GOTO_STMT or any(L in self.gotos_in(x) for x in groups[gj][1][:-1]):
                                raise NotImplementedError("switch: goto not at the end of its case")
                            gotoflag[gj] = flag
            for gi, (pats, sts) in enumerate(groups):
                last = sts[-1] if sts else None
                term = (K.BREAK_STMT, K.RETURN_STMT, K.CONTINUE_STMT, K.GOTO_STMT)
                ends = last is not None and last.kind in term
                if last is not None and last.kind == K.COMPOUND_STMT and children(last):
                    ends = ends or children(last)[-1].kind in term
                if gi in labtail:
                    ends = True
                if not ends and gi != len(groups) - 1:
                    raise NotImplementedError("switch fallthrough")
                body_st = sts[:-1] if (last is not None and last.kind == K.BREAK_STMT) else sts
                setf = None
                if gi in labtail:
                    body_st = sts[:-1]
                    setf = labtail[gi][1]
                elif gi in gotoflag:
                    body_st = sts[:-1]
                    setf = gotoflag[gi]
                arms.append((pats, body_st, setf))
            has_default = any("_" in a[0] for a in arms)
            txt = lab + ": {" + NL
            for gi in sorted(labtail):
                txt += p + f"    let mut {labtail[gi][1]} = false;" + NL
            txt += p + "    match " + cond.code + " {" + NL
            for pats, body_st, setf in arms:
                pat = "_" if "_" in pats else " | ".join(pats)
                txt += p + "        " + pat + " => {" + NL
                txt += self.seq(body_st, ind + 3)
                if setf:
                    txt += p + f"            {setf} = true;" + NL
                txt += p + "        }" + NL
            if not has_default:
                txt += p + "        _ => {}" + NL
            txt += p + "    }" + NL
            for gi in sorted(labtail):
                lst, flag = labtail[gi]
                txt += p + f"    if {flag} {{" + NL + self.seq(children(lst), ind + 2) + p + "    }" + NL
            txt += p + "}"
            return txt
        finally:
            self.loops.pop()

    def for_parts(self, c):
        """(init, cond, inc, body) of a FOR_STMT using source tokens to find which parts exist"""
        ch = children(c)
        body = ch[-1]
        rest = ch[:-1]
        # find the two semicolons in the for header to classify parts
        toks = [t for t in c.get_tokens()]
        depth, semis, i = 0, [], 0
        for t in toks[2:]:
            s = t.spelling
            if s == "(":
                depth += 1
            elif s == ")":
                if depth == 0:
                    break
                depth -= 1
            elif s == ";" and depth == 0:
                semis.append(t.extent.start.offset)
        init = cnd = inc = None
        for x in rest:
            off = x.extent.start.offset
            if semis and off < semis[0]:
                init = x
            elif len(semis) > 1 and off < semis[1]:
                cnd = x
            else:
                inc = x
        return init, cnd, inc, body

    def declstmt(self, c, ind):
        out = []
        for d in children(c):
            if d.kind == K.STRUCT_DECL or d.kind == K.UNION_DECL:
                out.append(self.local_struct(d, ind))
                continue
            if d.kind != K.VAR_DECL:
                continue
            name = ident(d.spelling)
            t = d.type
            init = self.var_init(d)
            if d.storage_class == ci.StorageClass.STATIC:
                nm2 = f"{self.cur}_{d.spelling}"
                self.static_locals[d.hash] = nm2
                iv = self.var_init(d)
                if iv:
                    e0 = self.conv(self.E(iv[-1]), d.type)
                    initv = e0.code
                else:
                    initv = self.zero(d.type)
                self.extra_statics.append(f"static mut {nm2}: {rty(d.type)} = {initv};")
                continue
            tys = self.local_ty(t)
            if init:
                ie = init[-1]
                if ie.kind == K.INIT_LIST_EXPR:
                    out.append(f"let mut {name}: {tys} = {self.initlist(ie, t)};")
                else:
                    e = self.conv(self.E(ie), t)
                    out.append(f"let mut {name}: {tys} = {e.code};")
            else:
                out.append(f"let mut {name}: {tys} = {self.zero(t)};")
        return ("\n" + "    " * ind).join(out)

    def local_ty(self, t):
        d = ctype(t).get_declaration()
        if ctype(t).kind == T.RECORD and d.hash in self.local_structs:
            return self.local_structs[d.hash]
        if ctype(t).kind == T.CONSTANTARRAY and ctype(ctype(t).element_type).kind == T.RECORD:
            ed = ctype(ctype(t).element_type).get_declaration()
            if ed.hash in self.local_structs:
                return f"[{self.local_structs[ed.hash]}; {ctype(t).element_count}]"
        return rty(t)

    def local_struct(self, d, ind):
        if not d.is_definition():
            return ""
        nm = f"Local{len(self.local_structs)}"
        self.local_structs[d.hash] = nm
        flds = []
        for f in d.get_children():
            if f.kind == K.FIELD_DECL:
                flds.append(f"{ident(f.spelling)}: {rty(f.type)}")
        kw = "union" if d.kind == K.UNION_DECL else "struct"  # a local union must stay a union (type punning)
        return f"#[derive(Clone, Copy)] #[repr(C)] {kw} {nm} {{ {', '.join(flds)} }}"

    def zero(self, t):
        n = norm(t)
        if n.kind in INT_KINDS:
            return "0"
        if n.kind in FLT_KINDS:
            return "0.0"
        if n.kind == T.POINTER:
            return "None" if is_fnptr(t) else "null_mut()"
        if n.kind == T.CONSTANTARRAY:
            el = n.element_type
            if norm(el).kind in INT_KINDS or norm(el).kind in FLT_KINDS or norm(el).kind == T.POINTER:
                return f"[{self.zero(el)}; {n.element_count}]"
        return "unsafe { core::mem::zeroed() }"

    def agg(self, t, ch, pos, leaf, braced):
        """aggregate initialiser of type t from the flat/braced initialisers ch[pos:] (C brace elision);
        -> (code, next pos). `leaf(x, ty)` converts a scalar initialiser, `braced(x, ty)` an explicit {..}."""
        n = norm(t)

        def one(x, ty, pos):
            nt = norm(ty)
            xs = strip(x)
            while xs.kind == K.UNEXPOSED_EXPR and len(expr_children(xs)) == 1 and strip(expr_children(xs)[0]).kind == K.INIT_LIST_EXPR:
                xs = strip(expr_children(xs)[0])
            if xs.kind == K.INIT_LIST_EXPR:
                return braced(xs, ty), pos + 1
            if xs.kind == K.STRING_LITERAL and nt.kind == T.CONSTANTARRAY:
                return leaf(x, ty), pos + 1
            if nt.kind in (T.RECORD, T.CONSTANTARRAY) and not (nt.kind == T.RECORD and ident_rec_is_scalarlike(nt)):
                # brace elision: the aggregate takes its members from the following initialisers, unless this one
                # already is an aggregate value of the right type
                if nt.kind == T.RECORD and norm(xs.type).kind == T.RECORD:
                    return leaf(x, ty), pos + 1
                return self.agg(ty, ch, pos, leaf, braced)
            return leaf(x, ty), pos + 1

        if n.kind == T.RECORD:
            decl = n.get_declaration()
            decl = decl.get_definition() or decl
            flds = [f for f in decl.get_children() if f.kind == K.FIELD_DECL]
            parts = []
            for f in flds:
                if pos >= len(ch):
                    break
                code, pos = one(ch[pos], f.type, pos)
                parts.append(f"{ident(f.spelling)}: {code}")
            tail = "" if len(parts) == len(flds) else ", ..unsafe { core::mem::zeroed() }"
            return f"{rty(t)} {{ " + ", ".join(parts) + tail + " }", pos
        el = n.element_type
        items = []
        while len(items) < n.element_count and pos < len(ch):
            code, pos = one(ch[pos], el, pos)
            items.append(code)
        while len(items) < n.element_count:
            items.append(self.zero(el))
        return "[" + ", ".join(items) + "]", pos

    def initlist(self, ie, t):
        ch = expr_children(ie)
        n = norm(t)
        if ch and all((evaluate(x) or (None, 1))[1] == 0 and x.kind in (K.INTEGER_LITERAL, K.FLOATING_LITERAL, K.UNEXPOSED_EXPR) and
                      norm(x.type).kind != T.RECORD for x in ch):
            return self.zero(t)
        if n.kind in (T.RECORD, T.CONSTANTARRAY):
            return self.agg(t, ch, 0, lambda x, ty: self.conv(self.E(x), ty).code, lambda x, ty: self.initlist(x, ty))[0]
        warn(f"{self.cur}: non-trivial init list; zeroed")
        return self.zero(t)

    # ---- static data initialisers -------------------------------------------------------------------------------
    def cexpr(self, c, t):
        """const expression text of initialiser node c converted to type t"""
        c = strip(c)
        while c.kind == K.UNEXPOSED_EXPR and len(expr_children(c)) == 1:
            ch = strip(expr_children(c)[0])
            if ch.kind == K.INIT_LIST_EXPR:
                c = ch
                break
            c = ch
        n = norm(t)
        if c.kind == K.INIT_LIST_EXPR:
            ch = expr_children(c)
            if n.kind in (T.RECORD, T.CONSTANTARRAY):
                return self.agg(t, ch, 0, lambda x, ty: self.cexpr(x, ty), lambda x, ty: self.cexpr(x, ty))[0]
            warn(f"{self.cur}: init list for {n.spelling}")
            return self.zero(t)
        if c.kind == K.STRING_LITERAL and n.kind == T.CONSTANTARRAY:
            v = evaluate(c)
            b = list(v[1]) if v else []
            b = b[:n.element_count] + [0] * (n.element_count - len(b))
            return "[" + ", ".join(str(x if x < 128 else x - 256) for x in b) + "]"
        if n.kind in INT_KINDS or n.kind in FLT_KINDS:
            v = evaluate(c)
            if v:
                if n.kind in FLT_KINDS:
                    return (f32lit(v[1]) + ("f32" if f32lit(v[1]) != "f32::MAX" else "")) if prim(t) == "f32" else f64lit(v[1]) + "f64"
                return self.int_lit(int(v[1]) & ((1 << bits(t)) - 1) if not signed(t) else int(v[1]), t).code
        if n.kind == T.POINTER:
            if c.kind == K.INTEGER_LITERAL:
                return "None" if is_fnptr(t) else "null_mut()"
            if c.kind == K.UNARY_OPERATOR and UN.get(lib.clang_getCursorUnaryOperatorKind(c)) == "&":
                tg = strip(expr_children(c)[0])
                if tg.kind == K.DECL_REF_EXPR:
                    return f"core::ptr::addr_of_mut!({self.declref(tg).code})"
            if c.kind == K.DECL_REF_EXPR and c.referenced is not None and c.referenced.kind == K.VAR_DECL and is_arr(c.referenced.type):
                return f"core::ptr::addr_of_mut!({self.declref(c).code}) as {rty(t)}"
            if c.kind == K.DECL_REF_EXPR and c.referenced is not None and c.referenced.kind == K.FUNCTION_DECL:
                self.fnaddr.add(c.referenced.spelling)
                return f"Some({self.ext_cb_ref(c.referenced, self.fn_ref(c.referenced))})"
        e = self.E(c)
        return self.conv(e, t).code

    def var_init(self, d):
        toks = [t.spelling for t in d.get_tokens()]
        depth = 0
        has = False
        for t in toks:
            if t in "([":
                depth += 1
            elif t in ")]":
                depth -= 1
            elif t == "=" and depth == 0:
                has = True
                break
        if not has:
            return []
        return [x for x in children(d) if x.kind not in (K.TYPE_REF, K.NAMESPACE_REF, K.UNEXPOSED_ATTR, K.STRUCT_DECL, K.UNION_DECL) and
                x.extent.start.offset > d.location.offset]

    # ---- functions -------------------------------------------------------------------------------------------------
    def sig(self, fn):
        params = []
        for a in fn.get_arguments():
            params.append(f"mut {ident(a.spelling) or '_'}: {pty(a.type)}")
        if fn.type.is_function_variadic():
            params.append("args: &[*const c_void]")
        r = fn.result_type
        rs = "" if norm(r).kind == T.VOID else f" -> {rty(r)}"
        return ", ".join(params), rs

    def has_goto(self, fn):
        return any(x.kind in (K.GOTO_STMT, K.LABEL_STMT) for x in fn.walk_preorder())

    def emit_fn(self, fn, force_extern=False):
        name = fn.spelling
        self.cur = name
        self.cur_ret = fn.result_type
        self.loops = []
        self.gmap = {}
        self.consumed = set()
        params, rs = self.sig(fn)
        vis = "fn" if fn.storage_class == ci.StorageClass.STATIC else "pub fn"
        ext_c = force_extern or name in self.fnaddr or name in self.sib_taken or (ALL_EXTERN and not fn.type.is_function_variadic())
        qual = f'unsafe extern "C" {vis}' if False else None
        pubs = "pub " if (fn.storage_class != ci.StorageClass.STATIC or PUB_ALL) else ""
        exs = 'extern "C" ' if ext_c else ""
        decl = f"{pubs}unsafe {exs}fn {name}({params}){rs}"
        if self.hand and (self.hand / f"{name}.rs").exists():
            return (self.hand / f"{name}.rs").read_text(encoding="utf8").rstrip() + "\n"
        body = [x for x in children(fn) if x.kind == K.COMPOUND_STMT][-1]
        try:
            txt = self.S(body, 0)
        except NotImplementedError as ex:
            warn(f"{name}: goto not handled ({ex}); body left as todo!() (hand port)")
            return decl + " {" + NL + '    todo!("' + name + ': goto, hand port")' + NL + "}" + NL
        if fn.type.is_function_variadic():
            txt = "{" + NL + "    let mut va_store = crate::tu::va::Va::new(args);" + txt[1:]
        # gwtool: uninitialised locals are 0 (done at declaration)
        doc = f"/// `{name}`\n"
        return f"{doc}{decl} {txt}\n"


def ppc_layout(src, name):
    """GameCube (ppc32) size / align / field offsets of the record `name` defined in `src`"""
    idx = ci.Index.create()
    tu = idx.parse(src, args=G.g.PPC, options=ci.TranslationUnit.PARSE_DETAILED_PROCESSING_RECORD)
    for c in tu.cursor.get_children():
        if c.kind in (K.STRUCT_DECL, K.UNION_DECL) and c.spelling == name and c.is_definition():
            t = c.type
            fl = []
            for f in c.get_children():
                if f.kind == K.FIELD_DECL and not f.is_bitfield():
                    fl.append((f.spelling, f.get_field_offsetof() // 8, f.type.get_size(), f.type.spelling))
            return t.get_size(), t.get_align(), fl
    return None


def main():
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("src")
    ap.add_argument("out")
    ap.add_argument("--hand")
    ap.add_argument("--only")
    ap.add_argument("--sib", default="")
    ap.add_argument("--head")
    ap.add_argument("--data")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--pub-all", action="store_true")
    ap.add_argument("--all-extern", action="store_true", help="every function is `extern \"C\"` (a test may store any of them as a callback)")
    a = ap.parse_args()
    global PUB_ALL, ALL_EXTERN
    PUB_ALL = a.pub_all
    ALL_EXTERN = a.all_extern
    tr = Tr(a.src, a.hand, [x for x in a.sib.split(',') if x])
    # pass 1: find functions whose address is taken (need extern "C"), via a dry run
    names = [n for n in tr.defs]
    only = set(a.only.split(",")) if a.only else None
    for _ in range(2):
        outs = []
        tr.ext_fns.clear(); tr.ext_vars.clear(); tr.inlines.clear(); tr.fma.clear(); tr.ext_cb.clear(); WARN.clear()
        for n, fn in tr.defs.items():
            if only and n not in only:
                continue
            outs.append(tr.emit_fn(fn))
        # translate header inlines (may add more)
        done = set()
        while True:
            todo = [(n, d) for n, d in tr.inlines.items() if n not in done]
            if not todo:
                break
            for n, d in todo:
                done.add(n)
                outs.append(tr.emit_fn(d))
        # the static data initialisers take function addresses too
        tr.cur = "<data>"
        for n, v in tr.main_vars.items():
            iv = tr.var_init(v)
            if iv:
                tr.cexpr(iv[-1], v.type)
    # ext block
    ext = ["pub mod ext {", "    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]", "    use super::*;",
           "    macro_rules! ext_fns {",
           "        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {",
           "            pub mod hook {", "                use super::*;",
           "                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*", "            }",
           "            $( pub fn $name($($a: $t),*) $(-> $r)? {",
           "                match unsafe { hook::$name } {", "                    Some(f) => f($($a),*),",
           "                    None => panic!(concat!(\"TODO: \", stringify!($name), \" is not ported\")),",
           "                }", "            } )*", "        };", "    }", "    ext_fns! {"]
    for n, d in sorted(tr.ext_fns.items()):
        ps = ", ".join(f"a{i}: {pty(x.type)}" for i, x in enumerate(d.get_arguments()))
        r = d.result_type
        rs = "" if norm(r).kind == T.VOID else f" -> {rty(r)}"
        if d.type.is_function_variadic():
            ps += ", " if ps else ""
            ps += f"a{len(list(d.get_arguments()))}: &[*const c_void]"
        ext.append(f"    fn {ident(n)}({ps}){rs};")
    if getattr(tr, "uses_osreport", False):
        ext.append("    fn OSReport(a0: &str);")
    ext.append("    }")
    if tr.ext_cb:
        ext.append("    pub mod cb {")
        ext.append("        use super::*;")
        for n, d in sorted(tr.ext_cb.items()):
            ps = ", ".join(f"a{i}: {pty(x.type)}" for i, x in enumerate(d.get_arguments()))
            r = d.result_type
            rs = "" if norm(r).kind == T.VOID else f" -> {rty(r)}"
            call = ", ".join(f"a{i}" for i in range(len(list(d.get_arguments()))))
            ext.append(f"        pub unsafe extern \"C\" fn {ident(n)}({ps}){rs} {{ super::{ident(n)}({call}) }}")
        ext.append("    }")
    for n, v in sorted(tr.ext_vars.items()):
        vt = rty(v.type)
        if ctype(v.type).kind == T.INCOMPLETEARRAY:
            vt = f"[{rty(ctype(v.type).element_type)}; 1024]"  # `extern T x[]`: the owner TU sizes it; room for any index
        ext.append(f"    pub static mut {ident(n)}: {vt} = unsafe {{ core::mem::zeroed() }};")
    ext.append("}")
    pre = list(dict.fromkeys(tr.extra_statics))
    for c in tr.tu.tu.cursor.get_children():
        if c.kind in (K.STRUCT_DECL, K.UNION_DECL) and c.is_definition() and tr.is_main(c) and c.spelling and c.spelling not in KNOWN and "unnamed" not in c.spelling and "anonymous" not in c.spelling:
            fl = [f"    pub {ident(f.spelling)}: {rty(f.type)}," for f in c.get_children() if f.kind == K.FIELD_DECL]
            kw = "struct" if c.kind == K.STRUCT_DECL else "union"
            pre.append("#[repr(C)]" + NL + "#[derive(Clone, Copy)]" + NL + f"pub {kw} {c.spelling} {{" + NL + NL.join(fl) + NL + "}" + NL)
            lay = ppc_layout(a.src, c.spelling)
            if lay:
                size, align, flds = lay
                body = ", ".join(f'{fn} @ {off:#x}: {sz} "{cty}"' for fn, off, sz, cty in flds)
                pre.append(f'melee_core::gc_layout!({c.spelling}, "{c.spelling}", size {size:#x}, align {align}, [{NL}    {body},{NL}]);{NL}')
    for nm, d in tr.anon_var_structs:
        fl = [f"    pub {ident(f.spelling)}: {rty(f.type)}," for f in d.get_children() if f.kind == K.FIELD_DECL]
        pre.append("#[repr(C)]" + NL + "#[derive(Clone, Copy)]" + NL + f"pub struct {nm} {{" + NL + NL.join(fl) + NL + "}" + NL)
    tr.cur = "<data>"
    for n, v in tr.main_vars.items():
        vis = "pub "  # statics too: the mtu state oracles set and hash them
        iv = tr.var_init(v)
        init = tr.cexpr(iv[-1], v.type) if iv else tr.zero(v.type)
        mu = "" if v.type.is_const_qualified() else "mut "
        pre.append(f"{vis}static {mu}{ident(n)}: {tr.local_ty(v.type)} = {init};")
    head = Path(a.head).read_text(encoding="utf8") if a.head else ""
    if a.data:
        pre = [Path(a.data).read_text(encoding="utf8")]
    Path(a.out).write_text(head + NL + NL.join(ext) + NL + NL + NL.join(pre) + NL + NL + NL.join(outs), encoding="utf8")
    print(f"c2rs: wrote {a.out}: {len(outs)} fns, {len(WARN)} warnings", file=sys.stderr)
    if a.check:
        r = subprocess.run([sys.executable, "tools/rust-port/fma_sites.py", a.src], capture_output=True, text=True)
        want = {}
        for ln in r.stdout.splitlines():
            parts = ln.split()
            if len(parts) == 4 and (int(parts[2]) or int(parts[3])):
                want[parts[1]] = int(parts[2]) + int(parts[3])
        for fnm in sorted(set(want) | set(tr.fma)):
            if want.get(fnm, 0) != tr.fma.get(fnm, 0):
                print(f"c2rs: FMA MISMATCH {fnm}: clang {want.get(fnm, 0)} ours {tr.fma.get(fnm, 0)}", file=sys.stderr)


if __name__ == "__main__":
    main()
