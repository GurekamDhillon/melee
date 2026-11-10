#!/usr/bin/env python3
"""State oracle for the native character TUs (rust/melee_chara/src/tu/<stem>.rs).

  python rust/melee_chara/gen/oracle/oracle.py <stem> [...] [-v FUNC]        (worktree root, Windows)

For each spec (gen/oracle/spec_<stem>.py) this
  1. parses the game's .c with libclang (native x86_64 MSVC layout, the shipping flags): struct field
     kinds for the fill, every callee outside the TU, the prototypes of the functions under test;
  2. builds a C driver that #includes the game's own .c, compiled the way the shipping build compiles
     game code (clang -ffp-contract=on, -O2 -Xclang -disable-llvm-passes, gwtool's lowering redone by
     melee_runtime/oracle/lower.py), with a generated stub for every callee outside the TU. A stub logs
     its arguments (pointers as region id + offset) and answers from the seeded generator;
  3. runs each function on N synthetic game states (struct fill by field kind), hashing the call
     log, return value and the whole state afterwards, one FNV-1a per function
     -> tests/data/oracle_<stem>.txt  (computed from C source only: no disc data);
  4. writes tests/oracle_<stem>.rs, which repeats the fill and the stubs against the Rust port
     (tests/oracle_common) and compares the hashes.
-v FUNC makes the C driver print every case of FUNC (log line and region hashes), the Rust test does
the same with ORACLE_VERBOSE=FUNC, so a mismatch can be diffed.
"""
import importlib.util, os, re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
CRATE = HERE.parents[1]
BUILD = ROOT / "rust/target/oracle"
CLANG = os.environ.get("CLANG", r"C:\Program Files\LLVM\bin\clang.exe")
NM = str(Path(CLANG).with_name("llvm-nm.exe"))
LLVM = Path(CLANG).parent.parent
GALE = os.environ.get("GALE_INCLUDE", str((ROOT / "../../melee/build/GALE01/include").resolve()))
COMMON = ["--target=x86_64-pc-windows-msvc", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT",
          "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-fno-strict-aliasing", "-fwrapv",
          "-fcommon", "-fgnu89-inline", "-fno-fast-math", "-ftrivial-auto-var-init=zero",
          "-D_Static_assert(a,b)=", "-Dsize_t=gc_size_t", "-mlong-double-64", "-Wno-everything",
          "-I", str(ROOT / "src"), "-I", str(ROOT / "src/melee"), "-I", str(ROOT / "pc"),
          "-I", str(ROOT / "pc/gameworld"), "-I", str(BUILD), "-I", str(HERE),
          "-isystem", str(ROOT / "tools/rust-port/include"), "-isystem", str(ROOT / "src/MSL"),
          "-isystem", str(ROOT / "libs/dolphin/include"), "-isystem", str(ROOT / "libs/dolphin/src"),
          "-isystem", GALE, "-include", str(ROOT / "src/MSL/math_ppc.h")]
SHIP_IR = ["-O2", "-Xclang", "-disable-llvm-passes", "-ffp-contract=on", "-S", "-emit-llvm"]
LOWER = ROOT / "rust/melee_runtime/oracle/lower.py"

import clang.cindex as ci
ci.Config.set_library_file(str(LLVM / "bin" / "libclang.dll"))
import ctypes
_lib = ctypes.CDLL(str(LLVM / "bin" / "libclang.dll"))
_lib.clang_Cursor_isAnonymousRecordDecl.argtypes = [ci.Cursor]
_lib.clang_Cursor_isAnonymousRecordDecl.restype = ctypes.c_uint
ci.Cursor.is_anonymous_record_decl = lambda self: bool(_lib.clang_Cursor_isAnonymousRecordDecl(self))
K = ci.CursorKind
T = ci.TypeKind

DEFAULT_INCLUDES = ["ft/types.h", "ft/forward.h", "ft/fighter.h", "ft/inlines.h", "ft/ftcommon.h",
                    "sysdolphin/baselib/gobj.h", "sysdolphin/baselib/jobj.h"]
N_CASES = 400


def run(cmd, what):
    r = subprocess.run([str(c) for c in cmd], cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        sys.stderr.write(" ".join(map(str, cmd))[:600] + "\n" + r.stdout[-3000:] + r.stderr[-6000:])
        raise SystemExit(f"oracle: {what} failed")
    return r


# ------------------------------------------------------------------ libclang helpers
class Probe:
    def __init__(self, src_c, includes):
        BUILD.mkdir(parents=True, exist_ok=True)
        self.path = BUILD / "probe.c"
        self.path.write_text("".join(f'#include "{i}"\n' for i in includes) + f'#include "{src_c}"\n')
        idx = ci.Index.create()
        args = [a for a in COMMON if a != "-Wno-everything"] + ["-w"]
        self.tu = idx.parse(str(self.path), args=args, options=ci.TranslationUnit.PARSE_DETAILED_PROCESSING_RECORD)
        errs = [d for d in self.tu.diagnostics if d.severity >= ci.Diagnostic.Error]
        if errs:
            raise SystemExit("oracle: probe parse errors:\n" + "\n".join(str(d) for d in errs[:10]))
        self.types, self.funcs, self.vars = {}, {}, {}
        for c in self.tu.cursor.get_children():
            if c.kind == K.TYPEDEF_DECL:
                self.types.setdefault(c.spelling, c.underlying_typedef_type)
            elif c.kind in (K.STRUCT_DECL, K.UNION_DECL) and c.spelling and c.is_definition():
                self.types.setdefault(c.spelling, c.type)
            elif c.kind == K.FUNCTION_DECL:
                prev = self.funcs.get(c.spelling)
                if prev is None or (c.is_definition() and not prev.is_definition()):
                    self.funcs[c.spelling] = c
            elif c.kind == K.VAR_DECL:
                self.vars.setdefault(c.spelling, c)

    def type(self, name):
        return self.types[name]

    def anon_offset(self, parent, anon):
        for lf, lo in self.leaves(anon, 0):
            return (parent.type.get_offset(lf.spelling) - lo) // 8
        return 0

    def leaves(self, rec, base_bits):
        for ch in rec.get_children():
            if ch.kind == K.FIELD_DECL and ch.spelling:
                yield ch, base_bits + rec.type.get_offset(ch.spelling)
            elif ch.kind in (K.STRUCT_DECL, K.UNION_DECL) and ch.is_anonymous_record_decl():
                yield from self.leaves(ch, base_bits + self.anon_offset(rec, ch) * 8)

    def descs(self, tname, prefer=()):
        """[(byte offset, size, kind)] over every leaf of the record, in fill order (members whose
        path is a prefix of a `prefer` entry come after their siblings, so they win overlaps)."""
        out = []
        t = self.type(tname).get_canonical()
        self._rec(t, 0, "", out, prefer)
        return out

    def _kind(self, c):
        k = c.kind
        if k == T.FLOAT: return "f"
        if k in (T.DOUBLE, T.LONGDOUBLE): return "d"
        if k in (T.INT, T.LONG): return "i"
        if k in (T.UINT, T.ULONG): return "u"
        if k == T.SHORT: return "s"
        if k == T.USHORT: return "S"
        if k in (T.CHAR_S, T.SCHAR): return "c"
        if k in (T.CHAR_U, T.UCHAR, T.BOOL): return "C"
        if k in (T.LONGLONG, T.ULONGLONG): return "q"
        if k == T.POINTER: return "p"
        if k == T.ENUM:
            et = c.get_declaration().enum_type.get_canonical().kind
            return "u" if et in (T.UINT, T.ULONG) else "i"
        return None

    def _leaf(self, t, off, path, out, prefer):
        c = t.get_canonical()
        if c.kind in (T.CONSTANTARRAY, T.INCOMPLETEARRAY):
            et = c.element_type
            n = max(c.element_count, 0)
            esz = et.get_size()
            for i in range(n):
                self._leaf(et, off + i * esz * 8, path, out, prefer)
            return
        if c.kind == T.RECORD:
            self._rec(c, off, path, out, prefer)
            return
        kd = self._kind(c)
        if kd is None:
            raise SystemExit(f"oracle: unhandled leaf type {t.spelling} ({c.kind}) at {path}")
        out.append((off // 8, c.get_size(), kd))

    def _rec(self, c, base, path, out, prefer):
        decl = c.get_declaration()
        decl = decl.get_definition() or decl
        members = []
        for ch in decl.get_children():
            if ch.kind == K.FIELD_DECL:
                members.append(("f", ch))
            elif ch.kind in (K.STRUCT_DECL, K.UNION_DECL) and ch.is_anonymous_record_decl():
                members.append(("a", ch))

        def key(m):
            nm = m[1].spelling if m[0] == "f" else ""
            p = (path + "." + nm).strip(".") if nm else path
            return 1 if nm and any(q == p or q.startswith(p + ".") for q in prefer) else 0
        members.sort(key=key)  # stable: preferred members last
        bf_bytes = set()
        for kind, ch in members:
            if kind == "a":
                off = base + self.anon_offset(decl, ch) * 8
                self._rec(ch.type.get_canonical(), off, path, out, prefer)
                continue
            if not ch.spelling and ch.type.get_canonical().kind != T.RECORD:
                pass
            boff = base + ch.get_field_offsetof()
            if ch.is_bitfield():
                w = ch.get_bitfield_width()
                if w > 0:
                    for b in range(boff // 8, (boff + w - 1) // 8 + 1):
                        bf_bytes.add(b)
                continue
            nm = ch.spelling
            self._leaf(ch.type, boff, (path + "." + nm).strip("."), out, prefer)
        for b in sorted(bf_bytes):
            out.append((b, 1, "b"))


# records passed by pointer that are shared game objects, not small argument structs: the pointer only
PS_SKIP = {"HSD_JObj", "HSD_DObj", "HSD_Joint", "Item", "ftData", "CollData", "StaticPlayer", "HSD_GObj",
           "Fighter", "HSD_RObj", "HSD_CObj", "HSD_LObj", "HSD_Archive", "HSD_AObj", "HSD_FObj"}
PS_TAGS = set()


def kind_of(t):
    """('i'|'q'|'f'|'d'|'v'|'G'|'FP'|'v3'|'v2'|'pf'|'pi'|'fn'|'p', bool-like) for a C type."""
    c = t.get_canonical()
    k = c.kind
    if k == T.VOID:
        return "v"
    if k == T.POINTER:
        p = c.get_pointee().get_canonical()
        if p.kind in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO):
            return "fn"
        if p.kind == T.RECORD:
            n = p.get_declaration().spelling
            k0 = {"HSD_GObj": "G", "Fighter": "FP", "Vec3": "v3", "Vec": "v3", "Vec2": "v2"}.get(n)
            if k0:
                return k0
            sz = p.get_size()
            if n and 0 < sz <= 160 and n not in PS_SKIP:
                return f"ps:{n}:{sz}"
            return "p"
        if p.kind == T.FLOAT:
            return "pf"
        if p.kind in (T.INT, T.UINT, T.LONG, T.ULONG):
            return "pi"
        return "p"
    if k == T.FLOAT:
        return "f"
    if k in (T.DOUBLE, T.LONGDOUBLE):
        return "d"
    if k in (T.LONGLONG, T.ULONGLONG):
        return "q"
    if k in (T.INT, T.UINT, T.SHORT, T.USHORT, T.CHAR_S, T.SCHAR, T.CHAR_U, T.UCHAR, T.LONG, T.ULONG,
             T.BOOL, T.ENUM):
        return "i"
    raise SystemExit(f"oracle: unhandled type {t.spelling} ({k})")


def ret_is_bool(t):
    c = t.get_canonical()
    if c.kind == T.BOOL or t.spelling in ("bool", "BOOL"):
        return True
    if c.kind == T.ENUM:
        n = sum(1 for _ in c.get_declaration().get_children() if _.kind == K.ENUM_CONSTANT_DECL)
        return n <= 2
    return False


# ------------------------------------------------------------------ Rust hook parsing
def parse_hooks(rs_text):
    """hook!(HOOK, name, (a: T, ...) -> R ...) / hook_or!(... , port) / port_hook!(HOOK, name, port, (..)) lines in the file's `ext` module."""
    out = {}
    for m in re.finditer(r"^\s*(?:crate::)?(?:port_)?hook(?:_or)?!\(\s*(\w+)\s*,\s*(\w+)\s*,\s*(?:[\w:]+\s*,\s*)?\(", rs_text, re.M):
        hook, name = m.group(1), m.group(2)
        i = m.end()
        depth, j = 1, i
        while depth:
            ch = rs_text[j]
            if ch in "([<{":
                depth += 1
            elif ch in ")]>}":
                depth -= 1
            j += 1
        # `->` contains '>' which would unbalance; handle by scanning params separately
        params_src = rs_text[i:j - 1]
        # the scan above counts '<' / '>' in generics and `->` in fn types; recompute robustly
        params_src = scan_balanced(rs_text, i)
        j = i + len(params_src) + 1
        rest = rs_text[j:j + 400]
        rm = re.match(r"\s*->\s*", rest)
        ret = None
        if rm:
            k = rm.end()
            d, e = 0, k
            while e < len(rest):
                ch = rest[e]
                if ch in "(<[":
                    d += 1
                elif ch in ")>]":
                    if d == 0:
                        break
                    d -= 1
                elif ch == "," and d == 0:
                    break
                e += 1
            ret = rest[k:e].strip()
        params = []
        for p in split_top(params_src):
            p = p.strip()
            if not p:
                continue
            n, t = p.split(":", 1)
            params.append((n.strip(), t.strip()))
        out[name] = dict(hook=hook, params=params, ret=ret)
    return parse_manual_hooks(rs_text, out)


def parse_manual_hooks(rs_text, out):
    """Hand-expanded hooks: `pub static mut HOOK: Option<fn(T, ..) [-> R]> = None;` followed by the
    `pub fn name(` wrapper that consults it (the macro forms are parsed above)."""
    for m in re.finditer(r"pub static mut (\w+): Option<fn\(", rs_text):
        hook = m.group(1)
        params_src = scan_balanced(rs_text, m.end())
        j = m.end() + len(params_src) + 1
        rm = re.match(r"\s*(?:->\s*([^=]+?))?>\s*=\s*None;", rs_text[j:j + 200])
        if not rm:
            continue
        fm = re.search(r"pub fn (\w+)\(", rs_text[j:j + 600])
        if not fm or fm.group(1) in out:
            continue
        params = [(f"a{i}", t.strip()) for i, t in enumerate(split_top(params_src)) if t.strip()]
        ret = rm.group(1).strip() if rm.group(1) else None
        out[fm.group(1)] = dict(hook=hook, params=params, ret=ret)
    return out


def parse_stubs(rs_text):
    """The legacy `stub!(/// doc  HOOK, name, (a: T, ..) -> R)` / `stub_or!(.., port, (..))` entries of
    src/stubs.rs (hooks of the earlier rounds' ports); `G` is `*mut HSD_GObj`."""
    out = {}
    for m in re.finditer(r"stub(_or)?!\(\s*(?:///[^\n]*\n\s*)*(\w+),\s*(\w+),\s*(?:([\w:]+),\s*)?\(", rs_text):
        hook, name = m.group(2), m.group(3)
        i = m.end()
        params_src = scan_balanced(rs_text, i)
        j = i + len(params_src) + 1
        rest = rs_text[j:j + 300]
        rm = re.match(r"\s*->\s*", rest)
        ret = None
        if rm:
            k = rm.end()
            e = k
            while e < len(rest) and rest[e] not in ",)":
                e += 1
            ret = rest[k:e].strip()
        params = []
        for p_ in split_top(params_src):
            p_ = p_.strip()
            if p_:
                n, t = p_.split(":", 1)
                t = t.strip()
                params.append((n.strip(), "*mut HSD_GObj" if t == "G" else t))
        out[name] = dict(hook=hook, params=params, ret="*mut HSD_GObj" if ret == "G" else ret)
    g = lambda t: "*mut HSD_GObj" if t == "G" else t
    for n, info in parse_manual_hooks(rs_text, {}).items():
        if n not in out:
            out[n] = dict(hook=info["hook"], params=[(a, g(t)) for a, t in info["params"]],
                          ret=g(info["ret"]) if info["ret"] else None)
    return out


def scan_balanced(s, i):
    d = 1
    j = i
    while True:
        ch = s[j]
        if ch == "(":
            d += 1
        elif ch == ")":
            d -= 1
            if d == 0:
                return s[i:j]
        j += 1


def split_top(s):
    parts, d, cur = [], 0, ""
    k = 0
    while k < len(s):
        ch = s[k]
        if ch == "-" and s[k:k + 2] == "->":
            cur += "->"
            k += 2
            continue
        if ch in "(<[":
            d += 1
        elif ch in ")>]":
            d -= 1
        if ch == "," and d == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
        k += 1
    if cur.strip():
        parts.append(cur)
    return parts


# ------------------------------------------------------------------ C stub generation
def c_param(t, i):
    sp = t.spelling
    if "(*)" in sp:
        return sp.replace("(*)", f"(*a{i})")
    return f"{sp} a{i}"


def stub_c(name, fn):
    args = list(fn.type.argument_types())
    ret = fn.result_type
    kinds = [kind_of(a) for a in args]
    params = ", ".join(c_param(a, i) for i, a in enumerate(args)) or "void"
    rk = kind_of(ret)
    L = [f"{ret.spelling} {name}({params})", "{"]
    L.append(f'    lg_s("{name}(");')
    for i, k in enumerate(kinds):
        a = f"a{i}"
        if k == "i":
            L.append(f"    lg_int((u32) {a});")
        elif k == "q":
            L.append(f"    lg_int((u32) {a}); lg_int((u32) ((u64_) {a} >> 32));")
        elif k == "f":
            L.append(f"    lg_f({a});")
        elif k == "d":
            L.append(f"    lg_d({a});")
        elif k == "v3":
            L.append(f"    lg_v3((const Vec3*) {a});")
        elif k == "v2":
            L.append(f"    lg_v2((const Vec2*) {a});")
        elif k == "pf":
            L.append(f"    lg_pf((const float*) {a});")
        elif k == "pi":
            L.append(f"    lg_pi((const int*) {a});")
        elif k.startswith("ps:"):
            _, tag, sz = k.split(":")
            PS_TAGS.add(tag)
            L.append(f"    lg_ps((const void*) {a}, {sz}, D_{tag}, (int) ND_{tag});")
        else:  # G, FP, fn, p
            L.append(f"    lg_ptr((const void*) {a});")
    L.append('    lg_s(")");')
    outs = OUTS.get(name, [])
    for i in outs:
        k = kinds[i]
        L.append({"v3": f"    lg_oV3((Vec3*) a{i});", "v2": f"    lg_oV2((Vec2*) a{i});",
                  "pf": f"    lg_oPF((float*) a{i});", "pi": f"    lg_oPI((int*) a{i});"}[k])
    if rk == "v":
        L.append("    lg_c(';');")
    elif rk == "i":
        gen = RETS[name][0] if name in RETS else ("gen_b()" if ret_is_bool(ret) else "gen_i()")
        L.append(f"    {{ {ret.spelling} r = ({ret.spelling}) {gen}; lg_retu((u32) r); lg_c(';'); return r; }}")
    elif rk == "q":
        L.append(f"    {{ {ret.spelling} r = ({ret.spelling}) iv(0); lg_retu((u32) r); lg_c(';'); return r; }}")
    elif rk == "f":
        L.append("    { float r = rf(); lg_retf(r); lg_c(';'); return r; }")
    elif rk == "d":
        L.append("    { double r = (double) rf(); lg_retd(r); lg_c(';'); return r; }")
    elif rk in ("G", "FP", "p", "v3", "v2", "pf", "pi", "fn") or rk.startswith("ps:"):  # ps: a pointer to a small record
        L.append(f"    {{ {ret.spelling} r = ({ret.spelling}) gen_p(); lg_retp((const void*) r); lg_c(';'); return r; }}")
    L.append("}")
    return "\n".join(L), kinds, rk, ret


# stubs that write through a pointer argument: name -> [arg indices]
OUTS = {}
# stubs whose integer answer is not the generic gen_i()/gen_b(): name -> (C expression, Rust expression)
RETS = {}


# ------------------------------------------------------------------ Rust hook closures
def rs_arg_log(rtype, ck, a):
    t = rtype.replace(" ", "")
    if ck == "i":
        return f"lg_int({a} as u32);"
    if ck == "q":
        return f"lg_int({a} as u32); lg_int((({a} as u64) >> 32) as u32);"
    if ck == "f":
        return f"lg_f({a});"
    if ck == "d":
        return f"lg_d({a});"
    if ck in ("v3", "v2", "pf", "pi"):
        fn = {"v3": "lg_v3", "v2": "lg_v2", "pf": "lg_pf", "pi": "lg_pi"}[ck]
        ty = {"v3": "Vec3", "v2": "Vec2", "pf": "f32", "pi": "i32"}[ck]
        return f"{fn}({a} as *const _ as *const {ty});"
    if ck.startswith("ps:"):
        tag = ck.split(":")[1]
        m = re.search(r"(?:\*\s*(?:mut|const)|&\s*mut|&)\s*([\w:]+)", rtype)
        return f"lg_ps({a} as *const u8, core::mem::size_of::<{m.group(1)}>(), D_{tag});"
    # G, FP, p, fn
    if ck == "fn" or "fn(" in t or t == "HSD_GObjEvent" or t.startswith("Option<"):
        return f"lg_ptr(fnaddr({a}));"
    if t.startswith("&"):
        return f"lg_ptr({a} as *const _ as *const u8 as usize);"
    return f"lg_ptr({a} as *const u8 as usize);"


def rs_ret(rtype, ck, ret_t, boolish, name=None):
    r = rtype.replace(" ", "")
    if ck == "i" and name in RETS:
        return f"{{ let r = ({RETS[name][1]}) as {r}; lg_retu(r as u32); lg_c(';'); r }}"
    if ck == "i":
        if r == "bool":
            return "{ let r = (gen_b() != 0); lg_retu(r as u32); lg_c(';'); r }" if boolish else \
                   "{ let r = (gen_i() != 0); lg_retu(r as u32); lg_c(';'); r }"
        gen = "gen_b()" if boolish else "gen_i()"
        return f"{{ let r = {gen} as {r}; lg_retu(r as u32); lg_c(';'); r }}"
    if ck == "q":
        return f"{{ let r = iv(false) as {r}; lg_retu(r as u32); lg_c(';'); r }}"
    if ck == "f":
        return "{ let r = rf(); lg_retf(r); lg_c(';'); r }"
    if ck == "d":
        return "{ let r = rf() as f64; lg_retd(r); lg_c(';'); r }"
    return f"{{ let r = gen_p() as {rtype}; lg_retp(r as usize); lg_c(';'); r }}"


def rs_hook_install(name, rh, kinds, rk, ret_t, modpath):
    params = rh["params"]
    if len(params) != len(kinds):
        raise SystemExit(f"oracle: hook {name}: Rust has {len(params)} params, C has {len(kinds)}")
    sig = ", ".join(f"a{i}: {t}" for i, (_, t) in enumerate(params))
    body = [f'lg_s("{name}(");']
    for i, ((_, t), ck) in enumerate(zip(params, kinds)):
        body.append(rs_arg_log(t, ck, f"a{i}"))
    body.append('lg_s(")");')
    for i in OUTS.get(name, []):
        k = kinds[i]
        fn = {"v3": "lg_ov3", "v2": "lg_ov2", "pf": "lg_opf", "pi": "lg_opi"}[k]
        body.append(f"{fn}(a{i} as *mut _);")
    rt = rh["ret"]
    if rk == "v":
        body.append("lg_c(';');")
        rets = ""
    else:
        boolish = ret_is_bool(ret_t)
        body.append(rs_ret(rt, rk if rk in ("i", "q", "f", "d") else "p", ret_t, boolish, name))
        rets = f" -> {rt}"
    target = "melee_chara::stubs" if modpath == "@stubs" else f"melee_chara::tu::{modpath}::ext"
    return f"        {target}::{rh['hook']} = Some(|{sig}|{rets} {{ {' '.join(body)} }});"


# ------------------------------------------------------------------ the generator proper
def load_spec(stem):
    p = HERE / f"spec_{stem}.py"
    s = importlib.util.spec_from_file_location(f"spec_{stem}", p)
    m = importlib.util.module_from_spec(s)
    s.loader.exec_module(m)
    return m.SPEC


def cexpr_bf(path):
    return "OF." + path


def extract_include(stem, entries):
    """The game source cut verbatim (gen/cref/run.py EXTRACTS semantics) -> an .inc the driver includes."""
    if not entries:
        return None
    sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
    import run as R
    R.BUILD = BUILD
    key = f"oracle_{stem}"
    R.EXTRACTS[key] = entries
    R.extract(key)
    return (BUILD / f"{key}_extract.inc").as_posix()


def build(stem, verbose_fn=None):
    spec = load_spec(stem)
    OUTS.clear()
    PS_TAGS.clear()
    RETS.clear()
    RETS.update(spec.get("rets", {}))
    OUTS.update(spec.get("outs", {}))
    src = spec["src"]
    srcpath = (ROOT / src).as_posix()
    includes = spec.get("includes", DEFAULT_INCLUDES)
    rs_file = CRATE / f"src/tu/{spec['rust_mod']}.rs"
    hooks = {}  # C name -> [(module, info)]
    if spec.get("legacy_stubs"):
        for n, info in parse_stubs((CRATE / "src/stubs.rs").read_text(encoding="utf-8")).items():
            hooks.setdefault(n, []).append(("@stubs", info))
    for m in [spec["rust_mod"]] + list(spec.get("ext_mods", [])):
        for n, info in parse_hooks((CRATE / f"src/tu/{m}.rs").read_text(encoding="utf-8")).items():
            hooks.setdefault(n, []).append((m, info))

    pr = Probe(srcpath, includes)
    main_fns = []
    for n, c in pr.funcs.items():
        if c.is_definition() and Path(c.location.file.name).resolve() == (ROOT / src).resolve():
            main_fns.append(n)
    # ---- function list under test
    fspecs = []
    for f in spec["funcs"]:
        f = dict(f) if isinstance(f, dict) else dict(name=f)
        if f["name"] not in pr.funcs or not pr.funcs[f["name"]].is_definition():
            raise SystemExit(f"oracle: {f['name']} is not defined in {src}")
        fspecs.append(f)
    # ---- descriptor tables
    prefer = spec.get("prefer", [])
    D = {"Fighter": pr.descs("Fighter", prefer), "ftCommonData": pr.descs("ftCommonData"), "HSD_JObj": pr.descs("HSD_JObj")}
    attr_types = spec.get("attr_types", [])
    ext_types = spec.get("ext_types", attr_types)
    alltypes = set(attr_types) | set(ext_types)
    for f in fspecs:
        alltypes |= set(f.get("attrs", [])) | set(f.get("exts", []))
    for t in sorted(alltypes):
        D[t] = pr.descs(t)

    # ---- bitfield table
    bfs = spec.get("bitfields", [])  # (c path, rust path)
    bf_index = {c: i for i, (c, r) in enumerate(bfs)}

    def cdesc(name, lst):
        return f"static const ODesc D_{name}[] = {{\n" + ",\n".join(
            f"    {{ {o}, {s}, '{k}' }}" for o, s, k in lst) + "\n};\n"

    # ---- pass 1: driver without stubs, to find the undefined callees
    drv_head = []
    drv_head += [f'#include "{i}"' for i in includes]
    drv_head.append('#include "oracle.h"')
    drv_head.append(spec.get("pre_c", ""))
    drv_head.append(f'#include "{srcpath}"')
    ext_inc = extract_include(stem, spec.get("extracts"))
    if ext_inc:
        drv_head.append(f'#include "{ext_inc}"')
    drv1 = "\n".join(drv_head) + "\nint main(void) { return 0; }\n"
    p1 = BUILD / f"{stem}_p1.c"
    p1.write_text(drv1, encoding="utf-8")
    obj1 = ship_obj(p1, f"{stem}_p1")
    nm = [l.split()[-1] for l in run([NM, "--undefined-only", obj1], "nm").stdout.splitlines() if l.strip().endswith(tuple("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_0123456789")) and " U " in " " + l.strip() + " "]
    undef = sorted({n for n in nm if not n.startswith("__imp_")})
    stubs, stub_info, missing = [], {}, []
    gk = gekko_obj()
    gk_defs = {l.split()[-1] for l in run([NM, "--defined-only", gk], "nm").stdout.splitlines() if l.strip()}
    ignore = set(spec.get("ignore_stubs", []))
    for n in undef:
        if n in gk_defs or n in ("memcpy", "memset", "memmove", "memcmp", "strlen", "strcmp", "sqrt", "sqrtf", "fabs", "fabsf"):
            continue
        if n in pr.funcs:
            txt, kinds, rk, ret = stub_c(n, pr.funcs[n])
            stubs.append(txt)
            stub_info[n] = (kinds, rk, ret)
        elif n in pr.vars:
            missing.append(n)
        else:
            print(f"oracle: note: undefined {n} (not declared in the TU: left to the CRT / unresolved)")
    # symbols the spec defines itself (data or hand-written stubs)
    own = set(spec.get("provides", [])) | {"p_ftCommonData"}
    really_missing = [n for n in missing if n not in own]
    if really_missing:
        raise SystemExit(f"oracle: {stem}: undefined and not provided: {really_missing}\n"
                         f"  (add to spec['provides'] with a definition in spec['data_c'])")
    efs = "efSync_Spawn" in undef or "efSync_Spawn" in spec.get("variadic", [])
    # hooks present on the Rust side for each stub
    nohook = [n for n in stub_info if n != "efSync_Spawn" and n not in hooks and n not in ignore]
    if nohook:
        print(f"oracle: {len(nohook)} stubbed externs have no Rust hook (data references to other TUs' functions, or"
              f" a missing hook if the C calls them: the hashes would differ): " + ", ".join(nohook) if os.environ.get("ORACLE_WARN") else
              f"oracle: {len(nohook)} stubbed externs have no Rust hook (set ORACLE_WARN=1 to list)")

    # ---- the real driver
    S = []
    S += [f'#include "{i}"' for i in includes]
    S.append('#include "oracle.h"')
    if "efSync_Spawn" in pr.funcs:
        S.append('#include "ef/efsync.h"')
        S.append("#define efSync_Spawn(g, o, ...) oracle_efsync(OPP_NARG(__VA_ARGS__), g, o, __VA_ARGS__)")
    S.append(spec.get("pre_c", ""))
    S.append(f'#include "{srcpath}"')
    if ext_inc:
        S.append(f'#include "{ext_inc}"')
    S.append(spec.get("data_c", ""))
    for tag in sorted(PS_TAGS):
        D[tag] = pr.descs(tag)
    S.append("/* ---- descriptor tables ---- */")
    for k, v in D.items():
        S.append(cdesc(k, v))
        S.append(f"#define ND_{k} (sizeof D_{k} / sizeof D_{k}[0])")
    S.append("/* ---- generated stubs ---- */")
    for n in sorted(stub_info):
        if n == "efSync_Spawn":
            continue
        S.append(next(t for t in stubs if re.search(r"\b" + n + r"\(", t.split("\n")[0])))
    S.append(C_WORLD.replace("#define NJO 48", "#define NJO %d" % spec.get("njo", 48)))
    # fn table
    fnv_names = list(spec.get("fnvals", {}).keys())
    S.append("static void world_reg_fns(void)\n{")
    for n in fnv_names:
        S.append(f"    ofn((const void*) {n});")
    S.append("}")
    # bitfields
    S.append("static int bf_get(int i)\n{\n    switch (i) {")
    for i, (cp, rp) in enumerate(bfs):
        S.append(f"    case {i}: return (int) OF.{cp};")
    S.append("    default: return 0;\n    }\n}")
    S.append("static void bf_set(int i, int v)\n{\n    switch (i) {")
    for i, (cp, rp) in enumerate(bfs):
        S.append(f"    case {i}: OF.{cp} = v; break;")
    S.append("    default: break;\n    }\n}")
    S.append(f"#define NBF {len(bfs)}")
    S.append("static unsigned bf_byte[NBF + 1];")
    # fill
    S.append("static void world_fill_types(int fi)\n{\n    switch (fi) {")
    for fi, f in enumerate(fspecs):
        at = f.get("attrs", attr_types)
        et = f.get("exts", at if "attrs" in f else ext_types)
        S.append(f"    case {fi}:")
        for t in at:
            S.append(f"        ofill(OATTR, D_{t}, (int) ND_{t});")
        for t in at:
            S.append(f"        ofill(OBACKUP, D_{t}, (int) ND_{t});")
        for t in et:
            S.append(f"        ofill(OEXT, D_{t}, (int) ND_{t});")
        S.append("        break;")
    S.append("    default: break;\n    }\n}")
    S.append("static void world_links(void)\n{")
    S.append(spec.get("links_c", ""))
    for path, target in spec.get("opt_links", []):
        S.append(f"    {{ u32 k = rnd() % 3; OF.{path} = k ? (void*) &({target}) : (void*) 0; }}")
    S.append("}")
    # motion ids
    S.append("static const int MOTIONS[] = { %s };" % ", ".join(str(m) for m in (spec.get("motions") or [0])))
    S.append("#define NMOT %d" % len(spec.get("motions") or [0]))
    # functions
    S.append("typedef struct { const char* name; int n; unsigned bfmask; } OFn;")
    S.append("static const OFn OFNS_[] = {")
    plans = []
    for f in fspecs:
        c = pr.funcs[f["name"]]
        kinds = [kind_of(a.type) for a in c.get_arguments()]
        rk = kind_of(c.result_type)
        mask = 0
        for b in f.get("bf", spec.get("default_bf", [])):
            mask |= 1 << bf_index[b]
        n = f.get("n", spec.get("n", N_CASES))
        S.append(f'    {{ "{f["name"]}", {n}, 0x{mask:X}u }},')
        plans.append((f, c, kinds, rk, n, mask))
    S.append("};")
    S.append("static u64_ call_fn(int fi, u32* retbits)\n{\n    u64_ r = 0;\n    (void) r;\n    switch (fi) {")
    for fi, (f, c, kinds, rk, n, mask) in enumerate(plans):
        argsx, ix, fx, v3x, v2x, pfx, pix = [], 0, 0, 0, 0, 0, 0
        fixed = f.get("c_args")
        for k in kinds:
            if k == "G":
                argsx.append("&OG")
            elif k == "FP":
                argsx.append("&OF" if not any(a == "&OF" for a in argsx) else "&OVF")
            elif k == "v3":
                argsx.append(f"&AV3[{v3x}]"); v3x += 1
            elif k == "v2":
                argsx.append(f"&AV2[{v2x}]"); v2x += 1
            elif k == "pf":
                argsx.append(f"&APF[{pfx}]"); pfx += 1
            elif k == "pi":
                argsx.append(f"&API[{pix}]"); pix += 1
            elif k in ("i", "q"):
                argsx.append(f"AI[{ix}]"); ix += 1
            elif k in ("f", "d"):
                argsx.append(f"AF[{fx}]"); fx += 1
            elif fixed is None:
                raise SystemExit(f"oracle: {f['name']}: unsupported arg kind {k}; give c_args/rs_args")
        if fixed is not None:
            argsx = fixed
        call = f"{f['name']}({', '.join(argsx)})"
        if rk == "v":
            S.append(f"    case {fi}: {call}; *retbits = 0; return 0;")
        elif rk == "f":
            S.append(f"    case {fi}: {{ float r = {call}; *retbits = BC_(r); return BC_(r); }}")
        elif rk == "d":
            S.append(f"    case {fi}: {{ double r = {call}; *retbits = (u32) DC_(r); return DC_(r); }}")
        elif rk in ("i", "q"):
            S.append(f"    case {fi}: {{ u32 r = (u32) {call}; *retbits = r; return r; }}")
        else:
            S.append(f"    case {fi}: {{ u64_ r = normptr((const void*) {call}); *retbits = (u32) r; return r; }}")
    S.append("    }\n    return 0;\n}")
    S.append("static void case_pre(int fi)\n{\n    switch (fi) {")
    for fi, (f, c, kinds, rk, n, mask) in enumerate(plans):
        if f.get("pre_c"):
            S.append(f"    case {fi}: {{ {f['pre_c']} }} break;")
    S.append("    default: break;\n    }\n}")
    S.append(C_MAIN)
    drv = "\n".join(S) + "\n"
    dp = BUILD / f"{stem}_oracle.c"
    dp.write_text(drv, encoding="utf-8")
    exe = link([ship_obj(dp, f"{stem}_oracle")], stem)
    out = subprocess.run([str(exe)] + (["-v", verbose_fn] if verbose_fn else []), capture_output=True, text=True, check=True, timeout=900).stdout
    dst = CRATE / f"tests/data/oracle_{stem}.txt"
    if verbose_fn:
        (BUILD / f"{stem}_verbose.txt").write_text(out, encoding="utf-8", newline="\n")
        print(f"oracle: {stem}: verbose {verbose_fn} -> {BUILD / (stem + '_verbose.txt')}")
        return
    dst.write_text(out, encoding="utf-8", newline="\n")
    print(f"oracle: {stem}: {len(plans)} functions, {len(stub_info)} stubs -> {dst}")

    # ---- the Rust test
    R = []
    R.append(f"//! @generated by rust/melee_chara/gen/oracle/oracle.py from gen/oracle/spec_{stem}.py. Do not edit.")
    R.append(f"//! The state oracle of src/tu/{spec['rust_mod']}.rs against {src} (tests/data/oracle_{stem}.txt).")
    R.append("#![allow(unused_variables, unused_imports, unused_mut, unused_unsafe, static_mut_refs, non_snake_case, non_upper_case_globals, non_camel_case_types, clippy::all)]")
    R.append("mod oracle_common;")
    R.append("use core::ffi::c_void;")
    R.append("use melee_chara::tu::%s as t;" % spec["rust_mod"])
    R.append("use melee_types::*;")
    R.append("use oracle_common::*;")
    R.append(spec.get("use_rs", ""))
    for k, v in D.items():
        R.append(f"static D_{k}: &[ODesc] = &[" + ", ".join(f"({o}, {s}, b'{kd}')" for o, s, kd in v) + "];")
    R.append(RS_WORLD.replace("const NJO: usize = 48;", "const NJO: usize = %d;" % spec.get("njo", 48)))
    R.append("unsafe fn world_reg_fns() {")
    for n, expr in spec.get("fnvals", {}).items():
        R.append(f"    ofn(({expr}) as usize);")
    R.append("}")
    R.append("unsafe fn world_fill_types(w: &mut World, fi: usize) {\n    match fi {")
    for fi, f in enumerate(fspecs):
        at = f.get("attrs", attr_types)
        et = f.get("exts", at if "attrs" in f else ext_types)
        R.append(f"        {fi} => {{")
        for t_ in at:
            R.append(f"            ofill(w.attr.as_mut_ptr(), D_{t_});")
        for t_ in at:
            R.append(f"            ofill(w.backup.as_mut_ptr(), D_{t_});")
        for t_ in et:
            R.append(f"            ofill(w.ext.as_mut_ptr(), D_{t_});")
        R.append("        }")
    R.append("        _ => {}\n    }\n}")
    R.append("unsafe fn world_links(w: &mut World) {")
    R.append("    let f: *mut Fighter = &mut *w.f;")
    R.append(spec.get("links_rs", ""))
    for path, target in spec.get("opt_links", []):
        rtarget = spec["opt_targets_rs"][target]
        rpath = spec.get("opt_paths_rs", {}).get(path, path)
        R.append(f"    {{ let k = rnd() % 3; (*f).{rpath} = if k != 0 {{ {rtarget} }} else {{ core::ptr::null_mut() }}; }}")
    R.append("}")
    R.append("unsafe fn bf_get(f: &Fighter, i: usize) -> u32 {")
    R.append("    (match i {")
    for i, (cp, rp) in enumerate(bfs):
        R.append(f"        {i} => f.{rp}(),")
    R.append("        _ => 0,\n    }) as u32\n}")
    R.append("unsafe fn bf_set(f: &mut Fighter, i: usize, v: u32) {")
    R.append("    match i {")
    for i, (cp, rp) in enumerate(bfs):
        pre, _, last = rp.rpartition(".")
        R.append(f"        {i} => f.{pre + '.' if pre else ''}set_{last}(v as _),")
    R.append("        _ => {}\n    }\n}")
    R.append(f"const NBF: usize = {len(bfs)};")
    R.append("const MOTIONS: &[i32] = &[%s];" % ", ".join(str(m) for m in (spec.get("motions") or [0])))
    R.append("struct Fn_ { name: &'static str, n: u32, bfmask: u32 }")
    R.append("const FNS: &[Fn_] = &[")
    for f, c, kinds, rk, n, mask in plans:
        R.append(f'    Fn_ {{ name: "{f["name"]}", n: {n}, bfmask: 0x{mask:X} }},')
    R.append("];")
    R.append("unsafe fn call_fn(fi: usize, w: &mut World, a: &mut Args) -> (u64, u32) {")
    R.append("    let g = w.g;")
    R.append("    match fi {")
    for fi, (f, c, kinds, rk, n, mask) in enumerate(plans):
        argsx, ix, fx, v3x, v2x, pfx, pix, nfp = [], 0, 0, 0, 0, 0, 0, 0
        for k in kinds:
            if k == "G":
                argsx.append("g")
            elif k == "FP":
                argsx.append("w.f_ptr()" if nfp == 0 else "w.vf_ptr()"); nfp += 1
            elif k == "v3":
                argsx.append(f"&mut a.v3[{v3x}]"); v3x += 1
            elif k == "v2":
                argsx.append(f"&mut a.v2[{v2x}]"); v2x += 1
            elif k == "pf":
                argsx.append(f"&mut a.pf[{pfx}]"); pfx += 1
            elif k == "pi":
                argsx.append(f"&mut a.pi[{pix}]"); pix += 1
            elif k == "i":
                argsx.append(f"a.i[{ix}] as _"); ix += 1
            elif k == "q":
                argsx.append(f"a.i[{ix}] as _"); ix += 1
            elif k in ("f", "d"):
                argsx.append(f"a.f[{fx}] as _"); fx += 1
        if "rs_args" in f:
            argsx = f["rs_args"]
        modp = f.get("rs_path", f"t::{f['name']}")
        call = f"{modp}({', '.join(argsx)})"
        if rk == "v":
            R.append(f"        {fi} => {{ {call}; (0, 0) }}")
        elif rk == "f":
            R.append(f"        {fi} => {{ let r = {call}; (bc(r) as u64, bc(r)) }}")
        elif rk == "d":
            R.append(f"        {fi} => {{ let r = {call}; (dc(r), dc(r) as u32) }}")
        elif rk in ("i", "q"):
            R.append(f"        {fi} => {{ let r = {call} as u32; (r as u64, r) }}")
        else:
            R.append(f"        {fi} => {{ let r = normptr({call} as *const u8 as usize); (r, r as u32) }}")
    R.append("        _ => unreachable!(),\n    }\n}")
    R.append("unsafe fn install_hooks() {")
    for n in sorted(stub_info):
        if n == "efSync_Spawn":
            for m, info in hooks.get(n, []):
                R.append(f"        melee_chara::tu::{m}::ext::{info['hook']} = Some(|g: i32, o: *mut HSD_GObj, args: &[*mut c_void]| oracle_efsync(g, o, args));")
            continue
        if n not in hooks:
            continue
        kinds, rk, ret = stub_info[n]
        for m, info in hooks[n]:
            R.append(rs_hook_install(n, info, kinds, rk, ret, m))
    R.append(spec.get("hooks_rs", ""))
    R.append("}")
    R.append("const PRE: &[&str] = &[%s];" % ", ".join('"%s"' % f["name"] for f in fspecs))
    R.append("unsafe fn case_pre(fi: usize, w: &mut World, a: &mut Args) {\n    match fi {")
    for fi, (f, c, kinds, rk, n, mask) in enumerate(plans):
        if f.get("pre_rs"):
            R.append(f"        {fi} => {{ {f['pre_rs']} }}")
    R.append("        _ => {}\n    }\n}")
    R.append(RS_MAIN.replace("@STEM@", stem).replace("@NBFXXX@", str(len(bfs))))
    (CRATE / f"tests/oracle_{stem}.rs").write_text("\n".join(R) + "\n", encoding="utf-8", newline="\n")
    print(f"oracle: {stem}: wrote tests/oracle_{stem}.rs")


def ship_obj(c, stem):
    ll, lo = BUILD / f"{stem}.ll", BUILD / f"{stem}.lo.ll"
    run([CLANG, *COMMON, *SHIP_IR, str(c), "-o", str(ll)], f"clang {stem}")
    run([sys.executable, str(LOWER), str(ll), str(lo)], f"lower {stem}")
    obj = BUILD / f"{stem}.obj"
    run([CLANG, "--target=x86_64-pc-windows-msvc", "-O2", "-c", "-w", str(lo), "-o", str(obj)], f"obj {stem}")
    return obj


def coverage(stem, src):
    """Source-based coverage of the C functions under test (float semantics are irrelevant here, so the
    driver is built directly, without the shipping lowering): prints per-function region / branch
    coverage of the game .c, to see which paths the generated states reach."""
    drv = BUILD / f"{stem}_oracle.c"
    exe = BUILD / f"{stem}_cov.exe"
    prof = BUILD / f"{stem}.profraw"
    pd = BUILD / f"{stem}.profdata"
    flags = [a for a in COMMON if a != "-Wno-everything"]
    run([CLANG, *flags, "-w", "-O1", "-fprofile-instr-generate", "-fcoverage-mapping",
         str(drv), str(ROOT / "pc/gameworld/gekko_fp.c"), "-o", str(exe), "-fuse-ld=lld",
         "-Wl,/FORCE:UNRESOLVED", "-Wl,/IGNORE:4006", "-Wl,/IGNORE:4088"], "coverage build")
    env = dict(os.environ, LLVM_PROFILE_FILE=str(prof))
    subprocess.run([str(exe)], capture_output=True, text=True, check=True, env=env, timeout=900)
    run([str(Path(CLANG).with_name("llvm-profdata.exe")), "merge", "-sparse", str(prof), "-o", str(pd)], "profdata")
    r = run([str(Path(CLANG).with_name("llvm-cov.exe")), "report", str(exe), f"-instr-profile={pd}",
             "--show-functions", "--show-branch-summary", str(ROOT / src)], "llvm-cov")
    print(r.stdout)
    sh = run([str(Path(CLANG).with_name("llvm-cov.exe")), "show", str(exe), f"-instr-profile={pd}",
              "--show-branches=count", "--show-line-counts-or-regions", str(ROOT / src)], "llvm-cov show")
    (BUILD / f"{stem}_cov.txt").write_text(sh.stdout, encoding="utf-8")
    print(f"oracle: annotated source -> {BUILD / (stem + '_cov.txt')}")


def gekko_obj():
    """pc/gameworld/gekko_fp.c (sinf, cosf, __frsqrte...) through the same shipping front end + gwtool
    lowering as the game code (saturating float->int, fused double fma)."""
    gk = BUILD / "gekko_fp_lowered.obj"
    if not gk.exists():
        gk = ship_obj(ROOT / "pc/gameworld/gekko_fp.c", "gekko_fp_lowered")
    return gk


def link(objs, stem):
    gk = gekko_obj()
    exe = BUILD / f"{stem}_oracle.exe"
    run([CLANG, "--target=x86_64-pc-windows-msvc", *map(str, objs), str(gk), "-o", str(exe),
         "-fuse-ld=lld", "-Wl,/FORCE:UNRESOLVED", "-Wl,/IGNORE:4006", "-Wl,/IGNORE:4088"], "link")
    return exe


# ------------------------------------------------------------------ templates
C_WORLD = r"""
/* ---- the synthetic game state (tests/oracle_common repeats every line) ---- */
#define NJO 48
static Fighter OF, OVF;
static HSD_GObj OG, OVG, OIG[4];
static HSD_JObj OJO[NJO];
static FighterBone OPARTS[NJO];
static unsigned char OATTR[4096], OBACKUP[4096], OEXT[4096];
static ftData OFTD;
static void* OITEMS[8];
static ftCommonData OCD;
static Vec3 AV3[2];
static Vec2 AV2[2];
static float APF[2];
static int API[2];
static u32 AI[4];
static float AF[2];
ftCommonData* p_ftCommonData;
static void world_reg(void)
{
    oreg(&OF, sizeof OF);
    oreg(&OG, sizeof OG);
    oreg(OPARTS, sizeof OPARTS);
    oreg(OJO, sizeof OJO);
    oreg(OATTR, sizeof OATTR);
    oreg(OBACKUP, sizeof OBACKUP);
    oreg(OEXT, sizeof OEXT);
    oreg(&OFTD, sizeof OFTD);
    oreg(&OCD, sizeof OCD);
    oreg(OIG, sizeof OIG);
    oreg(&OVG, sizeof OVG);
    oreg(&OVF, sizeof OVF);
    oreg(OITEMS, sizeof OITEMS);
    oreg(OSCR, sizeof OSCR);
}
static void zero_bytes(void* p, unsigned n)
{
    unsigned char* b = (unsigned char*) p;
    while (n--) *b++ = 0;
}
"""

C_MAIN = r"""
static void world_case_fill(int fi)
{
    int i;
    zero_bytes(&OF, sizeof OF);
    zero_bytes(&OVF, sizeof OVF);
    zero_bytes(&OG, sizeof OG);
    zero_bytes(&OVG, sizeof OVG);
    zero_bytes(OIG, sizeof OIG);
    zero_bytes(OJO, sizeof OJO);
    zero_bytes(OPARTS, sizeof OPARTS);
    zero_bytes(OATTR, sizeof OATTR);
    zero_bytes(OBACKUP, sizeof OBACKUP);
    zero_bytes(OEXT, sizeof OEXT);
    zero_bytes(&OFTD, sizeof OFTD);
    zero_bytes(OITEMS, sizeof OITEMS);
    zero_bytes(&OCD, sizeof OCD);
    ofill(&OF, D_Fighter, (int) ND_Fighter);
    ofill(&OVF, D_Fighter, (int) ND_Fighter);
    world_fill_types(fi);
    ofill(&OCD, D_ftCommonData, (int) ND_ftCommonData);
    for (i = 0; i < 4; i++) ofill(&OJO[i], D_HSD_JObj, (int) ND_HSD_JObj);
    OG.user_data = &OF;
    OG.hsd_obj = &OJO[0];
    OVG.user_data = &OVF;
    OVG.hsd_obj = &OJO[1];
    OF.parts = OPARTS;
    OVF.parts = OPARTS;
    for (i = 0; i < NJO; i++) OPARTS[i].joint = &OJO[i];
    OF.dat_attrs = OATTR;
    OF.dat_attrs_backup = OBACKUP;
    OF.ft_data = &OFTD;
    OVF.dat_attrs = OATTR;
    OVF.dat_attrs_backup = OBACKUP;
    OVF.ft_data = &OFTD;
    OFTD.ext_attr = OEXT;
    OFTD.x48_items = OITEMS;
    for (i = 0; i < 8; i++) OITEMS[i] = &OIG[i & 3];
    p_ftCommonData = &OCD;
    world_links();
}

static void bf_probe(void)
{
    int i;
    unsigned j;
    for (i = 0; i < NBF; i++) {
        unsigned char* b = (unsigned char*) &OF;
        for (j = 0; j < sizeof OF; j++) b[j] = 0;
        bf_set(i, 1);
        for (j = 0; j < sizeof OF; j++) if (b[j]) break;
        bf_byte[i] = j;
    }
}

static u64_ hfl(u64_ h, const float* p, int n)
{
    int i;
    for (i = 0; i < n; i++) {
        u32 b = BC_(p[i]);
        h = fnv(h, &b, 4);
    }
    return h;
}

static u64_ img_hash(const void* p, unsigned n, const ODesc* d, int nd, unsigned bfmask)
{
    static unsigned char tmp[16384];
    unsigned i;
    for (i = 0; i < n; i++) tmp[i] = ((const unsigned char*) p)[i];
    onorm(tmp, d, nd);
    if (bfmask)
        for (i = 0; i < (unsigned) NBF; i++)
            if (bfmask >> i & 1) tmp[bf_byte[i]] = 0;
    return fnv(FNV0, tmp, n);
}

int main(int argc, char** argv)
{
    int fi;
    int verbose = argc > 2 && !strcmp(argv[1], "-v");
    unsigned nf = sizeof OFNS_ / sizeof OFNS_[0];
    bf_probe();
    printf("hdr %u %u %u %u %u %u %u", (unsigned) sizeof(Fighter), (unsigned) sizeof(ftCommonData),
           (unsigned) __builtin_offsetof(Fighter, cmd_vars), (unsigned) __builtin_offsetof(Fighter, self_vel),
           (unsigned) __builtin_offsetof(Fighter, mv), (unsigned) __builtin_offsetof(Fighter, coll_data),
           (unsigned) __builtin_offsetof(Fighter, input));
    for (fi = 0; fi < NBF; fi++) printf(" %u", bf_byte[fi]);
    printf("\n");
    world_reg();
    world_reg_fns();
    for (fi = 0; fi < (int) nf; fi++) {
        u64_ agg = FNV0;
        int k;
        const OFn* fn = &OFNS_[fi];
        int vb = verbose && !strcmp(argv[2], fn->name);
        if (verbose && !vb) continue;
        for (k = 0; k < fn->n; k++) {
            u64_ h = FNV0, rv, hF, hV, hG, hA, hB, hE, hC, hJ, hO;
            unsigned bits = 0;
            u32 retbits = 0;
            int i;
            rs = 0x9E3779B9u ^ ((u32) fi * 7919u + (u32) k * 104729u + 1u);
            world_case_fill(fi);
            {
                u32 m = rnd() % 3;
                int mid = (int) (rnd() % 1000);
                if (m == 0) mid = MOTIONS[rnd() % NMOT];
                OF.motion_id = mid;
            }
            for (i = 0; i < NBF; i++)
                if (fn->bfmask >> i & 1) ((unsigned char*) &OF)[bf_byte[i]] = 0;
            for (i = 0; i < NBF; i++) {
                u32 v = rnd() & 1;
                if (fn->bfmask >> i & 1) bf_set(i, (int) v);
            }
            for (i = 0; i < 4; i++) AI[i] = rnd() % 4;
            for (i = 0; i < 2; i++) AF[i] = rf();
            for (i = 0; i < 2; i++) { AV3[i].x = rf(); AV3[i].y = rf(); AV3[i].z = rf(); }
            for (i = 0; i < 2; i++) { AV2[i].x = rf(); AV2[i].y = rf(); }
            for (i = 0; i < 2; i++) APF[i] = rf();
            for (i = 0; i < 2; i++) API[i] = (int) (rnd() % 8);
            case_pre(fi);
            LGN = 0;
            rv = call_fn(fi, &retbits);
            LG[LGN] = 0;
            for (i = 0; i < NBF; i++)
                if (fn->bfmask >> i & 1) bits = bits * 1000003u ^ (unsigned) bf_get(i);
            hO = fnv(FNV0, LG, LGN);
            hF = img_hash(&OF, sizeof OF, D_Fighter, (int) ND_Fighter, fn->bfmask);
            hV = img_hash(&OVF, sizeof OVF, D_Fighter, (int) ND_Fighter, 0);
            hG = (u64_) normptr(OG.user_data) * 31u + (u64_) normptr(OG.hsd_obj);
            hA = fnv(FNV0, OATTR, sizeof OATTR);
            hB = fnv(FNV0, OBACKUP, sizeof OBACKUP);
            hE = fnv(FNV0, OEXT, sizeof OEXT);
            hC = fnv(FNV0, &OCD, sizeof OCD);
            hJ = fnv(FNV0, OJO, sizeof OJO);
            h = fnv(h, &hO, 8); h = fnv(h, &rv, 8); h = fnv(h, &bits, 4);
            h = fnv(h, &hF, 8); h = fnv(h, &hV, 8); h = fnv(h, &hG, 8); h = fnv(h, &hA, 8);
            h = fnv(h, &hB, 8); h = fnv(h, &hE, 8); h = fnv(h, &hC, 8); h = fnv(h, &hJ, 8);
            h = hfl(h, (const float*) AV3, 6); h = hfl(h, (const float*) AV2, 4); h = hfl(h, APF, 2);
            h = fnv(h, API, sizeof API);
            if (vb)
                printf("%d %016llx ret=%llx bf=%x F=%016llx V=%016llx G=%016llx A=%016llx B=%016llx E=%016llx C=%016llx J=%016llx L=%016llx A3=%08x,%08x,%08x\n  %s\n",
                       k, h, rv, bits, hF, hV, hG, hA, hB, hE, hC, hJ, hO, B_(AV3[0].x), B_(AV3[0].y), B_(AV3[0].z), LG);
            agg = fnv(agg, &h, 8);
        }
        if (!verbose) printf("%s %016llx\n", fn->name, agg);
    }
    return 0;
}
"""

RS_WORLD = r"""
const NJO: usize = 48;
struct World {
    f: Box<Fighter>,
    vf: Box<Fighter>,
    g: *mut HSD_GObj,
    og: Box<HSD_GObj>,
    ovg: Box<HSD_GObj>,
    oig: Box<[HSD_GObj; 4]>,
    jo: Box<[HSD_JObj; NJO]>,
    parts: Box<[FighterBone; NJO]>,
    attr: Box<[u8; 4096]>,
    backup: Box<[u8; 4096]>,
    ext: Box<[u8; 4096]>,
    ftd: Box<ftData>,
    items: Box<[*mut c_void; 8]>,
    cd: Box<ftCommonData>,
}
impl World {
    fn f_ptr(&mut self) -> *mut Fighter { &mut *self.f }
    fn vf_ptr(&mut self) -> *mut Fighter { &mut *self.vf }
}
struct Args { i: [u32; 4], f: [f32; 2], v3: [Vec3; 2], v2: [Vec2; 2], pf: [f32; 2], pi: [i32; 2] }
unsafe fn new_world() -> World {
    let mut w = World {
        f: Box::new(Fighter::zeroed()),
        vf: Box::new(Fighter::zeroed()),
        g: core::ptr::null_mut(),
        og: Box::new(core::mem::zeroed()),
        ovg: Box::new(core::mem::zeroed()),
        oig: Box::new(core::mem::zeroed()),
        jo: Box::new(core::mem::zeroed()),
        parts: Box::new(core::mem::zeroed()),
        attr: Box::new([0; 4096]),
        backup: Box::new([0; 4096]),
        ext: Box::new([0; 4096]),
        ftd: Box::new(core::mem::zeroed()),
        items: Box::new([core::ptr::null_mut(); 8]),
        cd: Box::new(ftCommonData::zeroed()),
    };
    w.g = &mut *w.og;
    w
}
unsafe fn world_reg(w: &mut World) {
    oreg(&*w.f, core::mem::size_of::<Fighter>());
    oreg(&*w.og, core::mem::size_of::<HSD_GObj>());
    oreg(w.parts.as_ptr(), core::mem::size_of_val(&*w.parts));
    oreg(w.jo.as_ptr(), core::mem::size_of_val(&*w.jo));
    oreg(w.attr.as_ptr(), 4096);
    oreg(w.backup.as_ptr(), 4096);
    oreg(w.ext.as_ptr(), 4096);
    oreg(&*w.ftd, core::mem::size_of::<ftData>());
    oreg(&*w.cd, core::mem::size_of::<ftCommonData>());
    oreg(w.oig.as_ptr(), core::mem::size_of_val(&*w.oig));
    oreg(&*w.ovg, core::mem::size_of::<HSD_GObj>());
    oreg(&*w.vf, core::mem::size_of::<Fighter>());
    oreg(w.items.as_ptr(), 64);
    oreg(OSCR.as_ptr(), 256);
}
unsafe fn world_case_fill(w: &mut World, fi: usize) {
    *w.f = Fighter::zeroed();
    *w.vf = Fighter::zeroed();
    *w.og = core::mem::zeroed();
    *w.ovg = core::mem::zeroed();
    *w.oig = core::mem::zeroed();
    *w.jo = core::mem::zeroed();
    *w.parts = core::mem::zeroed();
    w.attr.fill(0);
    w.backup.fill(0);
    w.ext.fill(0);
    *w.ftd = core::mem::zeroed();
    *w.items = [core::ptr::null_mut(); 8];
    *w.cd = ftCommonData::zeroed();
    ofill(&mut *w.f as *mut Fighter as *mut u8, D_Fighter);
    ofill(&mut *w.vf as *mut Fighter as *mut u8, D_Fighter);
    world_fill_types(w, fi);
    ofill(&mut *w.cd as *mut ftCommonData as *mut u8, D_ftCommonData);
    for i in 0..4 {
        ofill(&mut w.jo[i] as *mut HSD_JObj as *mut u8, D_HSD_JObj);
    }
    w.og.user_data = &mut *w.f as *mut Fighter as *mut c_void;
    w.og.hsd_obj = &mut w.jo[0] as *mut HSD_JObj as *mut _;
    w.ovg.user_data = &mut *w.vf as *mut Fighter as *mut c_void;
    w.ovg.hsd_obj = &mut w.jo[1] as *mut HSD_JObj as *mut _;
    w.f.parts = w.parts.as_mut_ptr();
    w.vf.parts = w.parts.as_mut_ptr();
    for i in 0..NJO {
        w.parts[i].joint = &mut w.jo[i];
    }
    w.f.dat_attrs = w.attr.as_mut_ptr() as *mut _;
    w.f.dat_attrs_backup = w.backup.as_mut_ptr() as *mut _;
    w.f.ft_data = &mut *w.ftd;
    w.vf.dat_attrs = w.attr.as_mut_ptr() as *mut _;
    w.vf.dat_attrs_backup = w.backup.as_mut_ptr() as *mut _;
    w.vf.ft_data = &mut *w.ftd;
    w.ftd.ext_attr = w.ext.as_mut_ptr() as *mut _;
    w.ftd.x48_items = w.items.as_mut_ptr() as *mut _;
    for i in 0..8 {
        w.items[i] = &mut w.oig[i & 3] as *mut HSD_GObj as *mut c_void;
    }
    melee_fighter::hooks::P_FT_COMMON_DATA = &*w.cd;
    world_links(w);
}
unsafe fn bf_probe(f: &mut Fighter) -> [usize; NBF] {
    let mut out = [0usize; NBF];
    for i in 0..NBF {
        bytes_mut(f).fill(0);
        bf_set(f, i, 1);
        out[i] = bytes(f).iter().position(|&b| b != 0).unwrap();
    }
    out
}
unsafe fn img_hash(img: &[u8], d: &[ODesc], bfmask: u32, bf_byte: &[usize]) -> u64 {
    let mut tmp = img.to_vec();
    onorm(&mut tmp, d);
    if bfmask != 0 {
        for i in 0..NBF {
            if bfmask >> i & 1 != 0 {
                tmp[bf_byte[i]] = 0;
            }
        }
    }
    fnv(FNV0, &tmp)
}
"""

RS_MAIN = r"""
#[test]
fn state_matches_c() {
    unsafe {
        let want = include_str!("data/oracle_@STEM@.txt");
        let mut lines = want.lines();
        let verbose = std::env::var("ORACLE_VERBOSE").ok();
        let mut w = new_world();
        let bf_byte = bf_probe(&mut w.f);
        let hdr: std::vec::Vec<&str> = lines.next().unwrap().split_whitespace().skip(1).collect();
        assert_eq!(hdr[0].parse::<usize>().unwrap(), core::mem::size_of::<Fighter>(), "sizeof(Fighter)");
        assert_eq!(hdr[1].parse::<usize>().unwrap(), core::mem::size_of::<ftCommonData>(), "sizeof(ftCommonData)");
        assert_eq!(hdr[2].parse::<usize>().unwrap(), core::mem::offset_of!(Fighter, cmd_vars), "offset cmd_vars");
        assert_eq!(hdr[3].parse::<usize>().unwrap(), core::mem::offset_of!(Fighter, self_vel), "offset self_vel");
        assert_eq!(hdr[4].parse::<usize>().unwrap(), core::mem::offset_of!(Fighter, mv), "offset mv");
        assert_eq!(hdr[5].parse::<usize>().unwrap(), core::mem::offset_of!(Fighter, coll_data), "offset coll_data");
        assert_eq!(hdr[6].parse::<usize>().unwrap(), core::mem::offset_of!(Fighter, input), "offset input");
        for i in 0..NBF {
            assert_eq!(hdr[7 + i].parse::<usize>().unwrap(), bf_byte[i], "byte of bitfield {i}");
        }
        install_hooks();
        world_reg(&mut w);
        world_reg_fns();
        let mut bad = std::vec::Vec::new();
        for (fi, fnn) in FNS.iter().enumerate() {
            let mut agg = FNV0;
            let vb = verbose.as_deref() == Some(fnn.name);
            if verbose.is_some() && !vb {
                continue;
            }
            for k in 0..fnn.n {
                let mut h = FNV0;
                RS = 0x9E37_79B9u32 ^ ((fi as u32).wrapping_mul(7919).wrapping_add(k.wrapping_mul(104729)).wrapping_add(1));
                world_case_fill(&mut w, fi);
                {
                    let m = rnd() % 3;
                    let mut mid = (rnd() % 1000) as i32;
                    if m == 0 {
                        mid = MOTIONS[(rnd() % MOTIONS.len() as u32) as usize];
                    }
                    w.f.motion_id = mid as _;
                }
                for i in 0..NBF {
                    if fnn.bfmask >> i & 1 != 0 {
                        bytes_mut(&mut *w.f)[bf_byte[i]] = 0;
                    }
                }
                for i in 0..NBF {
                    let v = rnd() & 1;
                    if fnn.bfmask >> i & 1 != 0 {
                        bf_set(&mut w.f, i, v);
                    }
                }
                let mut a = core::mem::zeroed::<Args>();
                for i in 0..4 {
                    a.i[i] = rnd() % 4;
                }
                for i in 0..2 {
                    a.f[i] = rf();
                }
                for i in 0..2 {
                    a.v3[i].x = rf();
                    a.v3[i].y = rf();
                    a.v3[i].z = rf();
                }
                for i in 0..2 {
                    a.v2[i].x = rf();
                    a.v2[i].y = rf();
                }
                for i in 0..2 {
                    a.pf[i] = rf();
                }
                for i in 0..2 {
                    a.pi[i] = (rnd() % 8) as i32;
                }
                case_pre(fi, &mut w, &mut a);
                LG.clear();
                let (rv, _rb) = call_fn(fi, &mut w, &mut a);
                let mut bits = 0u32;
                for i in 0..NBF {
                    if fnn.bfmask >> i & 1 != 0 {
                        bits = bits.wrapping_mul(1000003) ^ bf_get(&w.f, i);
                    }
                }
                let ho = fnv(FNV0, LG.as_bytes());
                let hf = img_hash(bytes(&*w.f), D_Fighter, fnn.bfmask, &bf_byte);
                let hv = img_hash(bytes(&*w.vf), D_Fighter, 0, &bf_byte);
                let hg = normptr(w.og.user_data as usize).wrapping_mul(31).wrapping_add(normptr(w.og.hsd_obj as usize));
                let ha = fnv(FNV0, &w.attr[..]);
                let hb = fnv(FNV0, &w.backup[..]);
                let he = fnv(FNV0, &w.ext[..]);
                let hc = fnv(FNV0, bytes(&*w.cd));
                let hj = fnv(FNV0, bytes(&*w.jo));
                for x in [ho, rv] {
                    h = fnv(h, &x.to_le_bytes());
                }
                h = fnv(h, &bits.to_le_bytes());
                for x in [hf, hv, hg, ha, hb, he, hc, hj] {
                    h = fnv(h, &x.to_le_bytes());
                }
                h = fnv(h, &canon_f32_bytes(&[a.v3[0].x, a.v3[0].y, a.v3[0].z, a.v3[1].x, a.v3[1].y, a.v3[1].z]));
                h = fnv(h, &canon_f32_bytes(&[a.v2[0].x, a.v2[0].y, a.v2[1].x, a.v2[1].y]));
                h = fnv(h, &canon_f32_bytes(&a.pf));
                h = fnv(h, bytes(&a.pi));
                if vb {
                    println!(
                        "{} {:016x} ret={:x} bf={:x} F={:016x} V={:016x} G={:016x} A={:016x} B={:016x} E={:016x} C={:016x} J={:016x} L={:016x} A3={:08x},{:08x},{:08x}\n  {}",
                        k, h, rv, bits, hf, hv, hg, ha, hb, he, hc, hj, ho, a.v3[0].x.to_bits(), a.v3[0].y.to_bits(), a.v3[0].z.to_bits(), LG
                    );
                }
                agg = fnv(agg, &h.to_le_bytes());
            }
            if verbose.is_some() {
                continue;
            }
            let line = lines.next().expect("data file shorter than the function table");
            let (cname, chash) = line.split_once(' ').unwrap();
            assert_eq!(cname, fnn.name, "data file and function table out of step");
            if u64::from_str_radix(chash, 16).unwrap() != agg {
                bad.push(fnn.name);
            }
        }
        assert!(bad.is_empty(), "{} of {} functions differ from the C: {:?}", bad.len(), FNS.len(), bad);
    }
}
"""


def main():
    args = sys.argv[1:]
    verbose_fn = None
    if "-v" in args:
        i = args.index("-v")
        verbose_fn = args[i + 1]
        del args[i:i + 2]
    BUILD.mkdir(parents=True, exist_ok=True)
    (CRATE / "tests/data").mkdir(parents=True, exist_ok=True)
    cov = "--cov" in args
    args = [x for x in args if x != "--cov"]
    for stem in args:
        if cov:
            coverage(stem, load_spec(stem)["src"])
            continue
        build(stem, verbose_fn)


if __name__ == "__main__":
    main()
