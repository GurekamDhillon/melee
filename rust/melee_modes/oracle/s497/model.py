"""libclang model of one game TU for the shard-497-504 oracle: functions it defines, the callees
and file-scope variables it uses from elsewhere, and C -> Rust type spellings."""
import os, re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
os.chdir(ROOT)
import gen_types as g  # noqa: E402

ci, K, T = g.ci, g.K, g.T
# Same flags as the other group-6 oracles: gnu target + Itanium bitfield packing (GC offsets).
NAT = [a.replace("x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu") for a in g.NAT] + ["-mno-ms-bitfields"]

GEN_RS = (ROOT / "rust/melee_types/src/gen.rs").read_text(encoding="utf-8")
RS_NAMES = set(re.findall(r"^pub (?:struct|type|union) (\w+)", GEN_RS, re.M))

VARIADIC = {"OSReport", "HSD_SisLib_803A6B98", "lbArchive_80016DBC", "lb_80011E24"}


class Fn:
    def __init__(self, c, defined):
        self.c = c
        self.name = c.spelling
        self.defined = defined
        self.params = [(a.spelling or f"p{i}", a.type) for i, a in enumerate(c.get_arguments())]
        self.ret = c.result_type
        self.variadic = c.type.is_function_variadic()
        self.static = c.linkage == ci.LinkageKind.INTERNAL


def decl_name(t):
    """Name of the record/typedef melee_types uses for pointee type t."""
    d = t.get_declaration()
    n = d.spelling
    if not n:
        # anonymous record behind a typedef: walk the typedef chain of the non-canonical type
        tt = t
        while tt.kind == T.TYPEDEF:
            return tt.get_declaration().spelling
        return None
    return n


def rust_ty(t, ctx=""):
    """Rust spelling of C type t (as a parameter, return or field type)."""
    cn = t.get_canonical()
    k = cn.kind
    prim = {T.VOID: "()", T.BOOL: "bool", T.CHAR_S: "i8", T.SCHAR: "i8", T.UCHAR: "u8", T.CHAR_U: "u8",
            T.SHORT: "i16", T.USHORT: "u16", T.INT: "i32", T.UINT: "u32", T.LONG: "i32", T.ULONG: "u32",
            T.LONGLONG: "i64", T.ULONGLONG: "u64", T.FLOAT: "f32", T.DOUBLE: "f64", T.LONGDOUBLE: "f64"}
    if k in prim:
        return prim[k]
    if k == T.ENUM:
        return "u32" if cn.get_declaration().enum_type.kind in (T.UINT, T.ULONG) else "i32"
    if k == T.POINTER:
        pt = cn.get_pointee()
        pc = pt.get_canonical()
        mut = "*const" if pt.is_const_qualified() else "*mut"
        if pc.kind == T.VOID:
            return f"{mut} core::ffi::c_void"
        if pc.kind == T.FUNCTIONPROTO:
            args = ", ".join(rust_ty(a) for a in pc.argument_types())
            r = rust_ty(pc.get_result())
            return f"Option<unsafe extern \"C\" fn({args}){'' if r == '()' else ' -> ' + r}>"
        if pc.kind == T.RECORD:
            n = decl_name(pt) or decl_name(pc)
            return f"{mut} {rname(n)}"
        if pc.kind == T.CONSTANTARRAY:
            return f"{mut} [{rust_ty(pc.element_type)}; {pc.element_count}]"
        return f"{mut} {rust_ty(pt)}"
    if k == T.RECORD:
        n = decl_name(t) or decl_name(cn)
        return rname(n)
    if k == T.CONSTANTARRAY:
        return f"[{rust_ty(cn.element_type)}; {cn.element_count}]"
    if k == T.INCOMPLETEARRAY:
        return f"[{rust_ty(cn.element_type)}; 1]"
    raise ValueError(f"no Rust type for {t.spelling} ({k})")


def rname(n):
    if n is None or n not in RS_NAMES:
        raise ValueError(f"melee_types has no {n}")
    return n


def load(path):
    tu = ci.Index.create().parse(path, args=NAT)
    here = Path(path).resolve()

    def inhere(c):
        return bool(c.location.file) and Path(str(c.location.file)).resolve() == here

    defs, callees, ext_vars, own_vars = {}, {}, {}, {}
    for c in tu.cursor.get_children():
        if c.kind == K.FUNCTION_DECL and c.is_definition() and inhere(c):
            defs[c.spelling] = Fn(c, True)
        if c.kind == K.VAR_DECL and inhere(c):
            own_vars[c.spelling] = c
    for c in tu.cursor.walk_preorder():
        if not inhere(c):
            continue
        r = c.referenced
        if r is None:
            continue
        if r.kind == K.FUNCTION_DECL and c.kind in (K.CALL_EXPR, K.DECL_REF_EXPR) and r.spelling not in defs \
                and r.get_definition() is None:
            callees.setdefault(r.spelling, Fn(r, False))
        if r.kind == K.VAR_DECL and c.kind == K.DECL_REF_EXPR and r.semantic_parent.kind == K.TRANSLATION_UNIT \
                and r.spelling not in own_vars and r.get_definition() is None:
            ext_vars.setdefault(r.spelling, r)
    # header inlines this file uses (their bodies are compiled into the TU): their out-of-line callees count
    used, todo = {}, []
    for c in tu.cursor.walk_preorder():
        if inhere(c) and c.kind in (K.CALL_EXPR, K.DECL_REF_EXPR):
            r = c.referenced
            if r is not None and r.kind == K.FUNCTION_DECL:
                d = r.get_definition()
                if d is not None and not inhere(d) and r.spelling not in used:
                    used[r.spelling] = d
                    todo.append(d)
    while todo:
        d = todo.pop()
        for c in d.walk_preorder():
            if c.kind in (K.CALL_EXPR, K.DECL_REF_EXPR):
                r = c.referenced
                if r is not None and r.kind == K.FUNCTION_DECL:
                    dd = r.get_definition()
                    if dd is not None and not inhere(dd) and r.spelling not in used:
                        used[r.spelling] = dd
                        todo.append(dd)
                    elif dd is None and r.spelling not in defs:
                        callees.setdefault(r.spelling, Fn(r, False))
    return tu, defs, callees, ext_vars, own_vars


def find_type(tu, name):
    for c in tu.cursor.walk_preorder():
        if c.kind in (K.TYPEDEF_DECL, K.STRUCT_DECL) and c.spelling == name and (c.kind == K.TYPEDEF_DECL or c.is_definition()):
            return c.type
    raise KeyError(name)


SHARD_TUS = ("gmtiny", "gmsupersudden", "gmsinglebutton", "gmslomo", "gmstamina", "gmstaffroll", "gmscene", "gmscdata")
_MINE = {}


def mine():
    """name -> stem for every function and `*_States` table defined by a TU of this shard."""
    if not _MINE:
        for stem in SHARD_TUS:
            tu, defs, callees, ev, own = load(f"src/melee/gm/{stem}.c")
            for n in list(defs) + [v for v in own if v.endswith("_States")]:
                _MINE[n] = stem
    return _MINE
