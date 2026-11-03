#!/usr/bin/env python3
"""State oracle for the native stage / mode TUs of melee_world (rust/melee_world/src/tu/<stem>.rs).

  python rust/melee_world/gen/oracle/gr_oracle.py <stem> [...] [-v FUNC]        (worktree root, Windows)

Same method as rust/melee_chara/gen/oracle/oracle.py (whose helpers it imports): the game's own .c is
#included into a C driver compiled the way the shipping build compiles game code (clang
-ffp-contract=on, gwtool's lowering redone by melee_runtime/oracle/lower.py); every callee outside the
TU is a generated stub that logs its arguments and answers from a seeded generator. What differs from
the character oracle is the synthetic world:
  * NGO gobjs OGO[] whose user_data are Grounds OGP[] and whose hsd_obj are JObj chains OJO[],
  * the TU's own statics and the extern globals the spec names (`globals`), all filled by field kind,
  * pools of records for pointer-returning stubs,
  * the `Ground.u` union filled through the members a function uses (spec `u=[..]`, per function too),
and after each call the whole world, the call log, the return value and the out-params are hashed
(pointers normalised to region id + offset, bitfield bytes masked and compared by value through the
Rust accessors, since C packs bitfields LSB-first and the Rust records MSB-first like the PPC).
The Rust test (tests/gro_<stem>.rs) repeats every line against the port; the expected hashes
(tests/data/gro_<stem>.txt) come from C source only.

Spec keys (spec_<stem>.py, SPEC = dict(...)): src, rust_mod, ext_mods, funcs (names or dicts with
name/c_args/rs_args/pre_c/pre_rs/n/u/rs_path), globals, statics, extra_pools, u, links_c/links_rs,
hooks_rs, fnvals, rets, outs, provides, data_c, pre_c, bf_skip, no_hook, n.
"""
import importlib.util, re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CRATE = HERE.parents[1]
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_chara/gen/oracle"))
import oracle as O  # noqa: E402

O.ROOT = ROOT
# The shipping game TUs are compiled `clang --target=ppc32-none-eabi`: left-to-right evaluation of call arguments.
# The MS C++ ABI of the x86_64-pc-windows-msvc target evaluates them right to left, so the oracle's C front end uses
# the MinGW (Itanium) ABI of the same LLP64 data model; the object code is still produced for the MSVC target.
assert O.COMMON[0] == "--target=x86_64-pc-windows-msvc"
O.COMMON[0] = "--target=x86_64-pc-windows-gnu"
import os as _os
if _os.environ.get("GRO_DEBUG"):
    O.SHIP_IR = O.SHIP_IR + ["-g", "-gcodeview"]
K, T = O.K, O.T
NGO = 6
NJO = 3 * NGO + 6
DEFAULT_INCLUDES = ["melee/gr/types.h", "melee/gr/forward.h", "melee/gr/ground.h",
                    "sysdolphin/baselib/gobj.h", "sysdolphin/baselib/jobj.h"]
N_CASES = 400
SPECIAL_PTR = {"HSD_GObj": "gobj", "HSD_JObj": "jobj", "Ground": "ground"}
KSIZE = {"p": 8, "f": 4, "d": 8, "i": 4, "u": 4, "s": 2, "S": 2, "c": 1, "C": 1, "q": 8}


# ------------------------------------------------------------------ the Rust record layouts (gen.rs)
def parse_gen_rs():
    txt = (ROOT / "rust/melee_types/src/gen.rs").read_text(encoding="utf-8")
    out = {}
    for m in re.finditer(r"^pub (?:struct|union) (\w+) \{\n", txt, re.M):
        end = txt.index("\n}\n", m.end() - 1)
        fields = []
        for fm in re.finditer(r"^    pub (\w+): (.+),$", txt[m.end():end + 1], re.M):
            fields.append((fm.group(1), fm.group(2)))
        out[m.group(1)] = fields
    return out


RS_TYPES = parse_gen_rs()


def arr_elem(rt):
    m = re.match(r"^\[(.+); (\w+)\]$", rt)
    return (m.group(1), int(m.group(2))) if m else None


def is_signed(t):
    c = t.get_canonical()
    return c.kind in (T.INT, T.LONG, T.SHORT, T.SCHAR, T.CHAR_S, T.LONGLONG) or (
        c.kind == T.ENUM and c.get_declaration().enum_type.get_canonical().kind in (T.INT, T.LONG))


def bf_paths(pr, tname, rs_tname=None, keep=None):
    """[(c path, rust accessor prefix, name, width, signed)] of every bitfield reachable from record `tname`.
    The Rust path is resolved by walking the generated record (anonymous C members are `anon_<GC offset>`,
    taken in declaration order)."""
    out = []
    rs_tname = rs_tname or tname

    def walk(t, rt, cpath, rpath):
        c = t.get_canonical()
        if c.kind == T.CONSTANTARRAY:
            ae = arr_elem(rt) if rt else None
            for i in range(max(c.element_count, 0)):
                walk(c.element_type, ae[0] if ae else None, f"{cpath}[{i}]", f"{rpath}[{i}]")
            return
        if c.kind != T.RECORD or rt is None:
            return
        decl = c.get_declaration()
        decl = decl.get_definition() or decl
        fields = RS_TYPES.get(rt)
        if fields is None:
            return
        fmap = dict(fields)
        anons = [f for f in fields if f[0].startswith("anon_")]
        k = 0
        for ch in decl.get_children():
            if ch.kind == K.FIELD_DECL:
                nm = ch.spelling
                if ch.is_bitfield():
                    if nm and ch.get_bitfield_width() > 0:
                        cp = (cpath + "." if cpath else "") + nm
                        if keep is None or keep(cp):
                            out.append((cp, rpath, nm, ch.get_bitfield_width(), is_signed(ch.type)))
                    continue
                if nm:
                    cp = (cpath + "." if cpath else "") + nm
                    if keep is not None and not keep(cp):
                        continue
                    walk(ch.type, fmap.get(nm), cp, (rpath + "." if rpath else "") + nm)
            elif ch.kind in (K.STRUCT_DECL, K.UNION_DECL) and ch.is_anonymous_record_decl():
                if k >= len(anons):
                    raise SystemExit(f"gro: {rt}: more anonymous C members than anon_ fields")
                an, at = anons[k]
                k += 1
                walk(ch.type, at, cpath, (rpath + "." if rpath else "") + an)

    walk(pr.type(tname), rs_tname, "", "")
    return out


class GProbe(O.Probe):
    """Probe whose descriptor walk skips members for which keep(path) is false.
    Also records function-local declarations (`extern T f(..);` inside a body, e.g. grdatfiles'
    ScriptGame_StageSlotArchive) so the callee gets a stub like a file-scope extern."""
    keep = None

    def __init__(self, src_c, includes):
        super().__init__(src_c, includes)
        for fn in list(self.funcs.values()):
            if not fn.is_definition():
                continue
            for c in fn.walk_preorder():
                if c.kind == K.FUNCTION_DECL and c != fn and c.spelling not in self.funcs:
                    self.funcs[c.spelling] = c

    def _rec(self, c, base, path, out, prefer):
        decl = c.get_declaration()
        decl = decl.get_definition() or decl
        bf_bytes = set()
        for ch in decl.get_children():
            if ch.kind in (K.STRUCT_DECL, K.UNION_DECL) and ch.is_anonymous_record_decl():
                off = base + self.anon_offset(decl, ch) * 8
                self._rec(ch.type.get_canonical(), off, path, out, prefer)
            elif ch.kind == K.FIELD_DECL:
                boff = base + ch.get_field_offsetof()
                if ch.is_bitfield():
                    w = ch.get_bitfield_width()
                    if w > 0:
                        for b in range(boff // 8, (boff + w - 1) // 8 + 1):
                            bf_bytes.add(b)
                    continue
                p = (path + "." + ch.spelling).strip(".")
                if self.keep is not None and not self.keep(p):
                    continue
                self._leaf(ch.type, boff, p, out, prefer)
        for b in sorted(bf_bytes):
            out.append((b, 1, "b"))


# ------------------------------------------------------------------ the TU's hooks (all three forms)
def parse_tu_hooks(rs_text):
    """name -> dict(hook, params, ret) over the hook forms the stage TUs use:
      * `hook!(HOOK, name, (..) -> R)` / `hook_or!(..)` / `port_hook!(..)` (O.parse_hooks);
      * the expanded form gen_link writes: `pub static mut HOOK: Option<fn(..)> = None;` followed by `pub fn name(a: T, ..) -> R {`;
      * `ext_fns! { fn name(a: T, ..) -> R [= default]; }` (the static is `hook::name`).
    Each is rewritten as a `hook!` line and handed to O.parse_hooks, so the parameter / return scan is the same."""
    out = O.parse_hooks(rs_text)
    t = rs_text
    fn_re = re.compile(r'\bpub (?:unsafe )?(?:extern "C" )?fn (\w+)\(')
    for m in re.finditer(r'^\s*pub static mut (\w+): Option<(?:unsafe )?(?:extern "C" )?fn\(', t, re.M):
        fm = fn_re.search(t, m.end())
        if not fm or t.count("\n", m.end(), fm.start()) > 2:
            continue
        name = fm.group(1)
        if name in out:
            continue
        params = O.scan_balanced(t, fm.end())
        rest = t[fm.end() + len(params) + 1:fm.end() + len(params) + 400]
        rm = re.match(r"\s*(->\s*[^{]+?)\s*\{", rest)
        out.update(O.parse_hooks(f"hook!({m.group(1)}, {name}, ({params}) {rm.group(1) if rm else ''});"))
    for im in re.finditer(r"\bext_fns!\s*\{", t):
        start = im.end()
        depth, j = 1, start
        while depth and j < len(t):
            depth += {"{": 1, "}": -1}.get(t[j], 0)
            j += 1
        for em in re.finditer(r"^\s*fn (\w+)\(", t[start:j], re.M):
            name = em.group(1)
            if name in out:
                continue
            k = start + em.end()
            params = O.scan_balanced(t, k)
            rest = t[k + len(params) + 1:k + len(params) + 400]
            rm = re.match(r"\s*(->\s*.+?)\s*(?:=|;)", rest, re.S)
            info = O.parse_hooks(f"hook!(HOOK_, {name}, ({params}) {rm.group(1) if rm else ''});").get(name)
            if info:
                info["hook"] = f"hook::{name}"
                out[name] = info
    return out


def ret_pool(ret):
    """('gobj'|'jobj'|'ground', None, None) / ('rec', tag, size) / None for the pointer a stub returns."""
    c = ret.get_canonical()
    if c.kind != T.POINTER:
        return None
    p = c.get_pointee().get_canonical()
    if p.kind != T.RECORD:
        return None
    n = p.get_declaration().spelling
    if n in SPECIAL_PTR:
        return (SPECIAL_PTR[n], None, None)
    if n in ("Vec3", "Vec", "Vec2"):
        return None
    sz = p.get_size()
    if n and 0 < sz <= 8192:
        return ("rec", n, sz)
    return None


LG_STR_C = r"""
static void gr_lg_ps(const void* p, u32 n, const ODesc* d, int nd)
{
    lg_c('@');
    lg_hmin(normptr(p));
    if (p) {
        static unsigned char tmp[1024];
        u32 i;
        int k;
        for (i = 0; i < n; i++) tmp[i] = ((const unsigned char*) p)[i];
        onorm(tmp, d, nd);
        for (k = 0; k < nd; k++)
            if (d[k].kind == 'b') for (i = 0; i < d[k].size; i++) tmp[d[k].off + i] = 0;
        lg_c('{');
        lg_hx(fnv(FNV0, tmp, n), 16);
        lg_c('}');
    }
    lg_c(',');
}
static void lg_str(const char* s)
{
    lg_c('"');
    if (s) { int i; for (i = 0; i < 80 && s[i]; i++) lg_c(s[i]); } else lg_s("null");
    lg_c('"');
    lg_c(',');
}
static void lg_vaw(u64_ w, char c)
{
    if (c == 'i') lg_int((u32) w);
    else if (c == 's') lg_str((const char*) w);
    else lg_ptr((const void*) w);
}
"""

RS_LG_STR = r"""
pub fn gr_lg_ps(p: *const u8, n: usize, d: &[ODesc]) {
    lg_c('@');
    lg_hmin(normptr(p as usize));
    if !p.is_null() {
        let mut tmp = unsafe { core::slice::from_raw_parts(p, n) }.to_vec();
        unsafe { onorm(&mut tmp, d) };
        for &(off, size, kind) in d {
            if kind == b'b' { for j in 0..size as usize { tmp[off as usize + j] = 0; } }
        }
        lg_c('{');
        lg_hx(fnv(FNV0, &tmp), 16);
        lg_c('}');
    }
    lg_c(',');
}
pub fn lg_str(s: *const i8) {
    lg_c('"');
    if !s.is_null() {
        let mut i = 0;
        loop {
            let c = unsafe { *s.add(i) };
            if i >= 80 || c == 0 { break; }
            lg_c(c as u8 as char);
            i += 1;
        }
    } else {
        lg_s("null");
    }
    lg_c('"');
    lg_c(',');
}
pub fn lg_vaw(w: u64, c: u8) {
    if c == b'i' { lg_int(w as u32); }
    else if c == b's' { lg_str(w as usize as *const i8); }
    else { lg_ptr(w as usize); }
}
"""


def link_low(objs, stem):
    """O.link, but the image is not large-address-aware and is based low: every pointer fits a u32, like the
    game's u32 fields that hold pointers (`(u32) gobj`)."""
    gk = O.gekko_obj()
    exe = O.BUILD / f"{stem}_oracle.exe"
    O.run([O.CLANG, "--target=x86_64-pc-windows-msvc", *map(str, objs), str(gk), "-o", str(exe),
           "-fuse-ld=lld", "-Wl,/FORCE:UNRESOLVED", "-Wl,/IGNORE:4006", "-Wl,/IGNORE:4088", "-Wl,/LARGEADDRESSAWARE:NO",
           "-Wl,/DYNAMICBASE:NO", "-Wl,/BASE:0x10000000", "-Wl,/HIGHENTROPYVA:NO"] + (["-Wl,/DEBUG"] if _os.environ.get("GRO_DEBUG") else []), "link")
    return exe


def load_spec(stem):
    p = HERE / f"spec_{stem}.py"
    s = importlib.util.spec_from_file_location(f"gspec_{stem}", p)
    m = importlib.util.module_from_spec(s)
    s.loader.exec_module(m)
    return m.SPEC


def _find_field(t, name):
    c = t.get_canonical()
    decl = c.get_declaration()
    decl = decl.get_definition() or decl
    for ch in decl.get_children():
        if ch.kind == K.FIELD_DECL and ch.spelling == name:
            return ch.type.get_canonical()
        if ch.kind in (K.STRUCT_DECL, K.UNION_DECL) and ch.is_anonymous_record_decl():
            r = _find_field(ch.type, name)
            if r is not None:
                return r
    return None


def field_offset(pr, tname, path):
    """Byte offset of a (dotted) member path inside a record, from libclang."""
    t = pr.type(tname).get_canonical()
    total = 0
    for comp in path.split("."):
        idx = None
        if comp.endswith("]"):
            comp, ix = comp[:-1].split("[")
            idx = int(ix)
        off = t.get_offset(comp)
        if off < 0:
            raise SystemExit(f"gro: no member {comp} in {t.spelling} ({off})")
        total += off // 8
        ft = _find_field(t, comp)
        if ft is None:
            raise SystemExit(f"gro: cannot find the type of {comp} in {t.spelling}")
        if idx is not None:
            total += idx * ft.element_type.get_size()
            ft = ft.element_type.get_canonical()
        t = ft
    return total


def graph_code(graph):
    """The `graph` DSL of a spec: random acyclic pointer structure over the typed static arrays, emitted for the
    C driver and the Rust test with the same rnd() consumption order. Entries:
      list    arr n field [z]          arr[i].field -> arr[i+1] (NULL 1 in z; the last is NULL)
      pickfwd arr n field [z]          arr[i].field -> arr[j], j > i (NULL 1 in z)
      pick    arr n field src m [z]    arr[i].field -> src[rnd % m] (NULL 1 in z; z = 0: never NULL)
      set     arr n field c rs         arr[i].field = <c> / <rs> (i is the index)"""
    C, R = [], []
    for g in graph:
        k, arr, n, fld = g["kind"], g["arr"], g["n"], g["field"]
        z = g.get("z", 4)
        nz_c = f"(rnd() % {z})" if z else "1"
        nz_r = f"(rnd() % {z} != 0)" if z else "true"
        if k == "list":
            C.append(f"    for (i = 0; i < {n - 1}; i++) {arr}[i].{fld} = {nz_c} ? &{arr}[i + 1] : 0; {arr}[{n - 1}].{fld} = 0;")
            R.append(f"    for i in 0..{n - 1} {{ {arr}[i].{fld} = if {nz_r} {{ addr_of_mut!({arr}[i + 1]) }} else {{ core::ptr::null_mut() }}; }} {arr}[{n - 1}].{fld} = core::ptr::null_mut();")
        elif k == "pickfwd":
            C.append(f"    for (i = 0; i < {n}; i++) {arr}[i].{fld} = (i < {n - 1} && {nz_c}) ? &{arr}[i + 1 + rnd() % ({n - 1} - i)] : 0;")
            R.append(f"    for i in 0..{n} {{ {arr}[i].{fld} = if i < {n - 1} && {nz_r} {{ addr_of_mut!({arr}[i + 1 + (rnd() % ({n - 1} - i) as u32) as usize]) }} else {{ core::ptr::null_mut() }}; }}")
        elif k == "pick":
            src, m = g["src"], g["m"]
            C.append(f"    for (i = 0; i < {n}; i++) {arr}[i].{fld} = {nz_c} ? (void*) &{src}[rnd() % {m}] : (void*) 0;")
            R.append(f"    for i in 0..{n} {{ {arr}[i].{fld} = if {nz_r} {{ addr_of_mut!({src}[(rnd() % {m}) as usize]) as *mut _ }} else {{ core::ptr::null_mut() }}; }}")
        elif k == "set":
            C.append(f"    for (i = 0; i < {n}; i++) {arr}[i].{fld} = {g['c']};")
            R.append(f"    for i in 0..{n} {{ {arr}[i].{fld} = {g['rs']}; }}")
        else:
            raise SystemExit(f"gro: unknown graph kind {k}")
    return "\n".join(C), "\n".join(R)


def kd_desc(kind, n, esz):
    return [(i * esz, esz, kind) for i in range(n)]


def build(stem, verbose_fn=None):
    spec = load_spec(stem)
    O.OUTS.clear(); O.PS_TAGS.clear(); O.RETS.clear()
    O.RETS.update(spec.get("rets", {}))
    O.OUTS.update(spec.get("outs", {}))
    src = spec["src"]
    srcpath = (ROOT / src).as_posix()
    includes = spec.get("includes", DEFAULT_INCLUDES)
    rs_main = spec["rust_mod"]
    hooks = {}
    for m in [rs_main] + list(spec.get("ext_mods", [])):
        for n, info in parse_tu_hooks((CRATE / f"src/tu/{m}.rs").read_text(encoding="utf-8")).items():
            hooks.setdefault(n, []).append((m, info))
    # the .c's own #include lines (so that its headers are in before any macro below is defined)
    own_inc = []
    for line in (ROOT / src).read_text(encoding="utf-8", errors="ignore").splitlines():
        t = line.strip()
        if t.startswith("#include"):
            own_inc.append(t)
        elif t and not t.startswith(("#", "//", "/*", "*")) and not t.endswith("*/"):
            break
    srcdir = (ROOT / src).parent.as_posix()
    if ("-I", srcdir) not in list(zip(O.COMMON, O.COMMON[1:])):
        O.COMMON.extend(["-I", srcdir])
    pr = GProbe(srcpath, includes)
    variadic = {}
    csrc = (ROOT / src).read_text(encoding="utf-8", errors="ignore")
    for n, d in pr.funcs.items():
        if n not in spec.get("provides", []) and d.type.kind == T.FUNCTIONPROTO and d.type.is_function_variadic() and not d.is_definition() and re.search(r"(?<!\w)" + n + r"(?!\w)", csrc):
            variadic[n] = dict(nfixed=len(list(d.get_arguments())), va=spec.get("variadic", {}).get(n, "i"))
    pre_h = O.BUILD / f"gro_{stem}_pre.h"
    O.BUILD.mkdir(parents=True, exist_ok=True)
    base_includes = list(includes)

    def make_pre():
        pre_lines = ["#include <stdarg.h>"] + own_inc
        pre_lines.append("#define GNARG_(_1,_2,_3,_4,_5,_6,_7,_8,_9,_10,_11,_12,N,...) N")
        pre_lines.append("#define GNARG(...) GNARG_(__VA_ARGS__, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0)")
        for n, v in variadic.items():
            d = pr.funcs[n]
            ps = ", ".join(O.c_param(a.type, i) if "(*)" in a.type.spelling else a.type.spelling for i, a in enumerate(d.get_arguments()))
            pre_lines.append(f"static {d.result_type.spelling} {n}__va(int{', ' + ps if ps else ''}, ...);")
            pre_lines.append(f"#define {n}(...) {n}__va(GNARG(__VA_ARGS__) - {v['nfixed']}, __VA_ARGS__)")
        pre_lines.append(spec.get("pre_c", ""))
        pre_h.write_text("\n".join(pre_lines) + "\n", encoding="utf-8")

    make_pre()
    includes = base_includes + [pre_h.as_posix()]
    pr = GProbe(srcpath, includes)
    fspecs = []
    for f in spec["funcs"]:
        f = dict(f) if isinstance(f, dict) else dict(name=f)
        if f["name"] not in pr.funcs or not pr.funcs[f["name"]].is_definition():
            raise SystemExit(f"gro: {f['name']} is not defined in {src}")
        fspecs.append(f)

    drv_src = srcpath
    if spec.get("c_patches") or spec.get("c_fn_regex") or spec.get("c_regex"):
        txt = (ROOT / src).read_text(encoding="utf-8", errors="ignore")
        for pat in spec.get("c_patches", []):
            old, new = pat[0], pat[1]
            cnt = pat[2] if len(pat) > 2 else 1
            assert txt.count(old) == cnt, f"gro: c_patch found {txt.count(old)} times, expected {cnt}: {old[:60]}"
            txt = txt.replace(old, new)
        # c_regex: (regex, replacement, expected count) over the whole file
        for rx, rep, cnt in spec.get("c_regex", []):
            txt, n_ = re.subn(rx, rep, txt)
            assert n_ == cnt, f"gro: c_regex {n_} matches (expected {cnt}) for {rx[:60]}"
        # c_fn_regex: (function name, regex, replacement[, expected count]) applied inside that function's body only
        for pat in spec.get("c_fn_regex", []):
            fn, rx, rep = pat[0], pat[1], pat[2]
            cnt = pat[3] if len(pat) > 3 else None
            mfn = re.search(r"(?m)^[A-Za-z_][^;{}\n]*?\b" + re.escape(fn) + r"\([^;{]*?\)\s*\{", txt)
            assert mfn, f"gro: c_fn_regex: no definition of {fn}"
            i0 = mfn.end() - 1
            depth, i1 = 0, i0
            while True:
                ch = txt[i1]
                if ch == "{":
                    depth += 1
                elif ch == "}":
                    depth -= 1
                    if depth == 0:
                        break
                i1 += 1
            body = txt[i0:i1 + 1]
            body2, n_ = re.subn(rx, rep, body)
            assert n_ > 0 and (cnt is None or n_ == cnt), f"gro: c_fn_regex {fn}: {n_} matches (expected {cnt}) for {rx[:60]}"
            txt = txt[:i0] + body2 + txt[i1 + 1:]
        pc = O.BUILD / f"gro_{stem}_src.c"
        pc.write_text(txt, encoding="utf-8", newline="\n")
        drv_src = pc.as_posix()
    # ---- pass 1: undefined callees (a variadic callee reached only through a macro, e.g. OSReport in
    # HSD_ASSERTREPORT, is found here and the pass is repeated with its stub macro)
    for _round in range(3):
        head = [f'#include "{i}"' for i in includes] + ['#include "oracle.h"', f'#include "{drv_src}"']
        p1 = O.BUILD / f"gro_{stem}_p1.c"
        p1.write_text("\n".join(head) + "\nint main(void) { return 0; }\n", encoding="utf-8")
        obj1 = O.ship_obj(p1, f"gro_{stem}_p1")
        nm = [l.split()[-1] for l in O.run([O.NM, "--undefined-only", str(obj1)], "nm").stdout.splitlines() if " U " in " " + l.strip() + " "]
        extra = [n for n in nm if n in pr.funcs and not n.endswith('__va') and n not in variadic and n not in spec.get("provides", []) and pr.funcs[n].type.kind == T.FUNCTIONPROTO
                 and pr.funcs[n].type.is_function_variadic() and not pr.funcs[n].is_definition()]
        if not extra:
            break
        for n in extra:
            variadic[n] = dict(nfixed=len(list(pr.funcs[n].get_arguments())), va=spec.get("variadic", {}).get(n, "i"))
        make_pre()
    undef = sorted({(n[:-4] if n.endswith("__va") and n[:-4] in variadic else n) for n in nm if not n.startswith("__imp_")})
    gk = O.gekko_obj()
    gk_defs = {l.split()[-1] for l in O.run([O.NM, "--defined-only", str(gk)], "nm").stdout.splitlines() if l.strip()}
    own = set(spec.get("provides", []))
    stubs, stub_info, missing = {}, {}, []
    for n in undef:
        if n == "__assert" and "__assert" not in own:
            continue
        if n in gk_defs or n in own or n in ("memcpy", "memset", "memmove", "memcmp", "strlen", "strcmp", "sqrt", "sqrtf", "fabs", "fabsf"):
            continue
        if n in pr.funcs:
            txt, kinds, rk, ret = O.stub_c(n, pr.funcs[n])
            txt = txt.replace("lg_ps(", "gr_lg_ps(")
            # a function-like macro of the same name (HSD_JObjSetMtxDirty) must not expand in the definition
            first_, rest_ = txt.split("\n", 1)
            mm = re.match(r"^(.*?) " + re.escape(n) + r"\((.*)\)$", first_)
            if mm:
                txt = f"{mm.group(1)} ({n})({mm.group(2)})\n" + rest_
            if rk.startswith("ps:"):
                txt = txt[:txt.rindex("}")] + f"    {{ {ret.spelling} r = ({ret.spelling}) gen_p(); lg_retp((const void*) r); lg_c(';'); return r; }}\n}}"
                rk = "p"
            # const char* parameters are logged by content
            for i, a in enumerate(pr.funcs[n].get_arguments()):
                ct = a.type.get_canonical()
                if ct.kind == T.POINTER and "const" in a.type.spelling and ct.get_pointee().get_canonical().kind in (T.CHAR_S, T.SCHAR, T.CHAR_U, T.UCHAR):
                    txt = txt.replace(f"lg_ptr((const void*) a{i});", f"lg_str((const char*) a{i});")
                    kinds[i] = "str"
            if n in variadic:
                nf = variadic[n]["nfixed"]
                first, rest = txt.split("\n", 1)
                m = re.match(r"^(.*?) \(?" + n + r"\)?\((.*)\)$", first)
                params = m.group(2)
                params = "" if params == "void" else params
                first = f"static {m.group(1)} {n}__va(int nva{', ' + params if params else ''}, ...)"
                last = f"a{nf - 1}" if nf else "nva"
                pat = variadic[n]["va"]
                extra = (f'    {{ va_list ap; int k; static const char PAT[] = "{pat}"; va_start(ap, {last});\n'
                         f'      for (k = 0; k < nva; k++) {{ u64_ w = va_arg(ap, u64_); lg_vaw(w, PAT[k < {len(pat)} ? k : {len(pat) - 1}]); }}\n'
                         f'      va_end(ap); }}\n')
                rest = rest.replace('    lg_s(")");', extra + '    lg_s(")");', 1)
                txt = first + "\n" + rest
            pool = ret_pool(ret)
            if rk == "f" and n in O.RETS:
                txt = txt.replace("float r = rf();", f"float r = {O.RETS[n][0]};")
            if rk not in ("i", "q", "f", "d", "v") and n in O.RETS:
                txt = txt.replace("gen_p()", O.RETS[n][0])
            elif pool and rk != "v" and n not in O.RETS:
                gen = {"gobj": "gen_gobj()", "jobj": "gen_jobj()", "ground": "gen_ground()"}.get(pool[0]) or f"gen_rec_{pool[1]}()"
                txt = txt.replace("gen_p()", gen)
            stubs[n] = txt
            stub_info[n] = (kinds, rk, ret, pool)
        elif n in pr.vars:
            missing.append(n)
        else:
            print(f"gro: note: undefined {n} (not declared in the TU)")
    bad = [n for n in missing if n not in own and n not in {g["c"] for g in spec.get("globals", [])}]
    if bad:
        raise SystemExit(f"gro: {stem}: undefined variables not provided: {bad} (spec globals= or provides=)")
    nohook = [n for n in stub_info if n not in hooks and n not in spec.get("no_hook", [])]
    if nohook:
        print(f"gro: {len(nohook)} stubbed externs with no Rust hook: " + ", ".join(nohook))

    # ---- type registry: tag -> dict(c=C type, rs=Rust type, desc, bf)
    TYPES = {}
    bf_skip = set(spec.get("bf_skip", []))

    def reg(tag, ctype=None, rs=None, desc=None, keep=None):
        ctype = ctype or tag
        pr.keep = keep
        d = desc if desc is not None else pr.descs(ctype)
        pr.keep = None
        bf = [] if tag in bf_skip or ctype in bf_skip else bf_paths(pr, ctype, rs or ctype, keep)
        sp = pr.type(ctype).spelling
        TYPES[tag] = dict(c=ctype, cs=(ctype if '(unnamed' in sp or '(anonymous' in sp else sp), desc=d, bf=bf)

    reg("HSD_GObj")
    reg("HSD_JObj")
    # Ground variants by the Ground.u members they use
    def uset(f):
        return (tuple(sorted(f.get("u", spec.get("u", [])))), tuple(f.get("ov", spec.get("ov", []))), tuple(f.get("bfs", spec.get("bfs", []))), tuple(f.get("bfx", spec.get("bfx", []))))
    usets = []
    for f in fspecs:
        u = uset(f)
        if u not in usets:
            usets.append(u)
    if not usets:
        usets.append(((), (), (), ()))
    overlays_spec = spec.get("overlays", {})
    for k, (u, ov, bfs, bfx) in enumerate(usets):
        def keep(p, u=u):
            if p == "u" or not p.startswith("u."):
                return True
            return p.split(".")[1] in u
        reg(f"Ground_{k}", "Ground", "Ground", keep=keep)
        if bfx:
            # bitfield paths whose flags the function writes as a whole byte (`.value = n`): the C packs LSB-first and the Rust
            # MSB-first, so neither the byte nor the flag views are comparable; dropped from the by-value list, byte stays masked
            TYPES[f"Ground_{k}"]["bf"] = [b for b in TYPES[f"Ground_{k}"]["bf"] if not any(b[0].startswith(x) for x in bfx)]
        if bfs:
            # bitfield paths whose storage the function writes through an integer member of another union member
            # (the C packs bitfields LSB-first, the Rust records MSB-first: the bytes alias differently by design)
            TYPES[f"Ground_{k}"]["bf"] = [b for b in TYPES[f"Ground_{k}"]["bf"] if not any(b[0].startswith(x) for x in bfs)]
            for x in bfs:
                boff = field_offset(pr, "Ground", x)
                TYPES[f"Ground_{k}"]["desc"] = [e for e in TYPES[f"Ground_{k}"]["desc"] if not (e[0] == boff and e[2] == "b")]
        TYPES[f"Ground_{k}"]["ov"] = []
        for oid in ov:
            o = overlays_spec[oid]
            off = field_offset(pr, "Ground", o["path"])
            size = pr.type(o["ty"]).get_size()
            if o["ty"] not in TYPES:
                reg(o["ty"])
            d = [(a, b, c) for (a, b, c) in TYPES[f"Ground_{k}"]["desc"] if not (a < off + size and off < a + b)]
            d.append((off, size, "b"))
            TYPES[f"Ground_{k}"]["desc"] = d
            TYPES[f"Ground_{k}"]["ov"].append((off, o["ty"]))
        for path in spec.get("u32ptr", []):
            if path.startswith("u.") and path.split(".")[1] not in u:
                continue
            off = field_offset(pr, "Ground", path)
            TYPES[f"Ground_{k}"]["desc"] = [(a, b, ("P" if (a == off and b == 4) else c)) for (a, b, c) in TYPES[f"Ground_{k}"]["desc"]]
    gv_of = [usets.index(uset(f)) for f in fspecs]
    for g in spec.get("globals", []):
        reg(g["ty"])
    for st in spec.get("statics", []):
        if st.get("ty") and st["ty"] not in TYPES:
            reg(st["ty"])
    pools = {}
    for n, (kinds, rk, ret, pool) in stub_info.items():
        if pool and pool[0] == "rec" and n not in O.RETS:
            pools[pool[1]] = pool[2]
    for t in spec.get("extra_pools", []):
        pools[t] = pr.type(t).get_size()
    for t in pools:
        try:
            reg(t)
        except SystemExit:
            TYPES[t] = dict(c=t, cs=t, desc=[], bf=[])
    for tag in sorted(O.PS_TAGS):
        if tag not in TYPES:
            reg(tag)
    # statics: descriptor lists over the whole static
    statics = []
    for st in spec.get("statics", []):
        n = st.get("n", 1)
        if st.get("ty"):
            esz = pr.type(st["ty"]).get_size()
            desc = []
            for i in range(n):
                desc += [(o + i * esz, s, k) for o, s, k in TYPES[st["ty"]]["desc"]]
        else:
            desc = kd_desc(st["kind"], n, KSIZE[st["kind"]])
        st = dict(st)
        st["desc"] = desc
        st["const"] = st["const"] if "const" in st else (bool(st.get("ty")) and not st.get("fill", False))
        st["fill"] = not st["const"]
        statics.append(st)

    # ---- emitters shared by C and Rust
    def cdesc(name, lst):
        return f"static const ODesc D_{name}[] = {{\n" + ",\n".join(f"    {{ {o}, {s}, '{k}' }}" for o, s, k in lst) + ("," if lst else "") + "\n    { 0, 0, 0 }\n};\n#define ND_" + name + " (sizeof D_" + name + " / sizeof D_" + name + "[0] - 1)\n"

    def cfill(name, lst):
        l2 = [x for x in lst if x[2] not in ('b', 'P')]
        return f"static const ODesc F_{name}[] = {{\n" + ",\n".join(f"    {{ {o}, {s}, '{k}' }}" for o, s, k in l2) + ("," if l2 else "") + "\n    { 0, 0, 0 }\n};\n#define NF_" + name + " (sizeof F_" + name + " / sizeof F_" + name + "[0] - 1)\n"

    def c_bf_funcs(tag):
        t = TYPES[tag]
        b = t["bf"]
        L = [f"static void bf_fill_{tag}({t['cs']}* p)\n{{\n    (void) p;"]
        for cp, rp, nm_, w, sg in b:
            m = (1 << (w - 1 if sg else w)) - 1
            L.append(f"    p->{cp} = (rnd() & {m}u);")
        L.append("}")
        L.append(f"static unsigned DUMP_N;\nstatic u64_ bf_hash_{tag}(u64_ h, const {t['cs']}* p)\n{{\n    u32 v;\n    (void) p; (void) v;\n    if (DUMP_N) printf(\"%c  BFV:\", 10);")
        for cp, rp, nm_, w, sg in b:
            L.append(f"    v = (u32) p->{cp}; h = fnv(h, &v, 4); if (DUMP_N) printf(\" %x\", v);")
        L.append("    return h;\n}")
        return "\n".join(L)

    def rs_bf_funcs(tag):
        t = TYPES[tag]
        b = t["bf"]
        L = [f"unsafe fn bf_fill_{tag}(p: *mut {t['c']}) {{\n    let _ = p;"]
        for cp, rp, nm_, w, sg in b:
            m = (1 << (w - 1 if sg else w)) - 1
            L.append(f"    (*p).{rp + '.' if rp else ''}set_{nm_}((rnd() & {m}) as _);")
        L.append("}")
        L.append(f"unsafe fn bf_hash_{tag}(mut h: u64, p: *const {t['c']}) -> u64 {{\n    let _ = p;\n    if DUMP_N != 0 {{ print!(\"\n  BFV:\"); }}")
        for cp, rp, nm_, w, sg in b:
            L.append(f"    h = fnv(h, &(((*p).{rp + '.' if rp else ''}{nm_}()) as u32).to_le_bytes()); if DUMP_N != 0 {{ print!(\" {{:x}}\", ((*p).{rp + '.' if rp else ''}{nm_}()) as u32); }}")
        L.append("    h\n}")
        return "\n".join(L)

    def c_fill(expr, tag):
        s_ = f"ofill(&{expr}, F_{tag}, (int) NF_{tag}); " + (f"bf_fill_{tag}((void*) &{expr});" if TYPES[tag]["bf"] else "")
        for off, oty in TYPES[tag].get("ov", []):
            s_ += f" bf_fill_{oty}((void*) ((unsigned char*) &{expr} + {off}));"
        return s_

    def c_hash(expr, tag):
        s_ = f"h = img_hash(h, &{expr}, sizeof {expr}, D_{tag}, (int) ND_{tag}); " + (f"h = bf_hash_{tag}(h, (const void*) &{expr});" if TYPES[tag]["bf"] else "")
        for off, oty in TYPES[tag].get("ov", []):
            s_ += f" h = bf_hash_{oty}(h, (const void*) ((const unsigned char*) &{expr} + {off}));"
        return s_

    def rs_fill(expr, tag):
        s_ = f"ofill(addr_of_mut!({expr}) as *mut u8, F_{tag}); " + (f"bf_fill_{tag}(addr_of_mut!({expr}) as *mut {TYPES[tag]['c']});" if TYPES[tag]["bf"] else "")
        for off, oty in TYPES[tag].get("ov", []):
            s_ += f" bf_fill_{oty}((addr_of_mut!({expr}) as *mut u8).add({off}) as *mut {oty});"
        return s_

    def rs_hash(expr, tag, size=None):
        base = f"core::slice::from_raw_parts(addr_of!({expr}) as *const u8, {size or f'core::mem::size_of_val(&*addr_of!({expr}))'})"
        s_ = f"h = img_hash(h, {base}, D_{tag}); " + (f"h = bf_hash_{tag}(h, addr_of!({expr}) as *const {TYPES[tag]['c']});" if TYPES[tag]["bf"] else "")
        for off, oty in TYPES[tag].get("ov", []):
            s_ += f" h = bf_hash_{oty}(h, (addr_of!({expr}) as *const u8).add({off}) as *const {oty});"
        return s_

    # ------------------------------------------------------------- the C driver
    S = [f'#include "{i}"' for i in includes] + ['#include "oracle.h"', LG_STR_C, f'#include "{drv_src}"']
    S.append(spec.get("data_c", ""))
    if "__assert" in undef and "__assert" not in own:
        S.append('void __assert(const char* f, u32 l, const char* c) { printf("ASSERT %s:%u %s\\n", f, (unsigned) l, c); _flushall(); for (;;) { *(volatile int*) 0 = 1; } }')
    for g in spec.get("globals", []):
        if g["c"] in pr.vars and g["c"] not in own and not g.get("defined"):
            S.append(f'{g["ty"]} {g["c"]};')
    S.append("/* ---- descriptor tables ---- */")
    for tag, t in TYPES.items():
        S.append(cdesc(tag, t["desc"]))
        S.append(cfill(tag, t["desc"]))
        if t["bf"]:
            S.append(c_bf_funcs(tag))
    for i, st in enumerate(statics):
        S.append(cdesc(f"st{i}", st["desc"]))
        S.append(cfill(f"st{i}", st["desc"]))
    S.append(f"#define NGO {NGO}\n#define NJO {NJO}")
    S.append("static HSD_GObj OGO[NGO];\nstatic Ground OGP[NGO];\nstatic HSD_JObj OJO[NJO];")
    for t, sz in pools.items():
        S.append(f"static unsigned char OP_{t}[2][{(sz + 7) // 8 * 8}] __attribute__((aligned(16)));")
    S.append("static Vec3 AV3[2];\nstatic Vec2 AV2[2];\nstatic float APF[2];\nstatic int API[2];\nstatic u32 AI[4];\nstatic float AF[2];")
    S.append("""
static void* gen_gobj(void) { u32 k = rnd() % (NGO + 1); return k ? (void*) &OGO[k - 1] : (void*) 0; }
static void* gen_jobj(void) { u32 k = rnd() % (NJO + 1); return k ? (void*) &OJO[k - 1] : (void*) 0; }
static void* gen_ground(void) { u32 k = rnd() % (NGO + 1); return k ? (void*) &OGP[k - 1] : (void*) 0; }
""")
    for t in pools:
        S.append(f"static void* gen_rec_{t}(void) {{ u32 k = rnd() % 3; return k ? (void*) OP_{t}[k - 1] : (void*) 0; }}")
    S.append("/* ---- generated stubs ---- */")
    for n in sorted(stubs):
        S.append(stubs[n])
    S.append("static void world_reg(void)\n{")
    S.append("    oreg(OGO, sizeof OGO);\n    oreg(OGP, sizeof OGP);\n    oreg(OJO, sizeof OJO);")
    for g in spec.get("globals", []):
        S.append(f"    oreg(&{g['c']}, sizeof {g['c']});")
    for t in pools:
        S.append(f"    oreg(OP_{t}[0], sizeof OP_{t}[0]);\n    oreg(OP_{t}[1], sizeof OP_{t}[1]);")
    for st in statics:
        S.append(f"    oreg(&{st['c']}, sizeof {st['c']});")
    S.append("    oreg(OSCR, sizeof OSCR);\n}")
    S.append("static void world_reg_fns(void)\n{")
    for n in spec.get("fnvals", {}):
        S.append(f"    ofn((const void*) {n});")
    S.append("}")
    S.append("static const int GVAR[] = { " + ", ".join(str(x) for x in gv_of) + " };")
    S.append("static void zero_bytes(void* p, unsigned n) { unsigned char* b = (unsigned char*) p; while (n--) *b++ = 0; }")
    S.append("static void world_fill(int fi)\n{\n    int i;\n    (void) fi;")
    S.append("    zero_bytes(OGO, sizeof OGO); zero_bytes(OGP, sizeof OGP); zero_bytes(OJO, sizeof OJO);")
    for g in spec.get("globals", []):
        S.append(f"    zero_bytes(&{g['c']}, sizeof {g['c']});")
    for t in pools:
        S.append(f"    zero_bytes(OP_{t}, sizeof OP_{t});")
    for st in statics:
        if not st["const"]:
            S.append(f"    zero_bytes(&{st['c']}, sizeof {st['c']});")
    S.append("    zero_bytes(OSCR, sizeof OSCR);")
    S.append("    for (i = 0; i < NGO; i++) { " + c_fill("OGO[i]", "HSD_GObj") + " }")
    S.append("    for (i = 0; i < NGO; i++) {\n        switch (GVAR[fi]) {")
    for k in range(len(usets)):
        S.append(f"        case {k}: " + c_fill("OGP[i]", f"Ground_{k}") + " break;")
    S.append("        }\n    }")
    S.append("    for (i = 0; i < NJO; i++) { " + c_fill("OJO[i]", "HSD_JObj") + " }")
    for g in spec.get("globals", []):
        S.append("    " + c_fill(g["c"], g["ty"]))
    for t in pools:
        S.append("    " + c_fill(f"OP_{t}[0]", t) + " " + c_fill(f"OP_{t}[1]", t))
    for i, st in enumerate(statics):
        if st["fill"]:
            S.append(f"    ofill(&{st['c']}, F_st{i}, (int) NF_st{i});")
        if st.get("ty") and TYPES[st["ty"]]["bf"] and st["fill"]:
            S.append(f"    for (i = 0; i < {st.get('n', 1)}; i++) bf_fill_{st['ty']}((void*) ((unsigned char*) &{st['c']} + i * sizeof({TYPES[st['ty']]['cs']})));")
    S.append("""    for (i = 0; i < NGO; i++) {
        OGO[i].user_data = &OGP[i];
        OGO[i].hsd_obj = &OJO[i];
        OGP[i].gobj = &OGO[i];
        OJO[i].child = &OJO[NGO + i];
        OJO[NGO + i].next = &OJO[2 * NGO + i];
        OJO[NGO + i].parent = &OJO[i];
        OJO[2 * NGO + i].parent = &OJO[i];
    }""")
    gc_, gr_ = graph_code(spec.get("graph", []))
    S.append(gc_)
    S.append(spec.get("links_c", ""))
    S.append("}")
    S.append("typedef struct { const char* name; int n; } OFn;\nstatic const OFn OFNS_[] = {")
    plans = []
    for f in fspecs:
        c = pr.funcs[f["name"]]
        kinds = [O.kind_of(a.type) for a in c.get_arguments()]
        rk = O.kind_of(c.result_type)
        rk = "p" if rk.startswith("ps:") else rk
        if f.get("voidret"):
            rk = "v"  # the C returns garbage (a fake return type); the port returns nothing
        n = f.get("n", spec.get("n", N_CASES))
        S.append(f'    {{ "{f["name"]}", {n} }},')
        plans.append((f, c, kinds, rk, n))
    S.append("};")
    S.append("#define STDUMP() do { int q_; " + " ".join(f'printf("  ST{i}:"); for (q_ = 0; q_ < (int) sizeof({st["c"]}); q_ += 4) printf(" %08x", *(u32*) ((char*) &{st["c"]} + q_)); printf("\\n");' for i, st in enumerate(statics)) + " for (q_ = 0; q_ < NOFNS; q_++) printf(\"  FN%d: %llx\\n\", q_, (u64_) OFNS[q_]); } while (0)")
    S.append("static u64_ call_fn(int fi, u32* retbits)\n{\n    u64_ r = 0;\n    (void) r;\n    switch (fi) {")
    for fi, (f, c, kinds, rk, n) in enumerate(plans):
        argsx, ix, fx, v3x, v2x, pfx, pix = [], 0, 0, 0, 0, 0, 0
        fixed = f.get("c_args")
        for k in kinds:
            if k == "G":
                argsx.append("&OGO[0]")
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
                raise SystemExit(f"gro: {f['name']}: unsupported arg kind {k}; give c_args/rs_args")
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
    for fi, (f, c, kinds, rk, n) in enumerate(plans):
        if f.get("pre_c"):
            S.append(f"    case {fi}: {{ {f['pre_c']} }} break;")
    S.append("    default: break;\n    }\n}")
    S.append("""
static u64_ hfl(u64_ h, const float* p, int n)
{
    int i;
    for (i = 0; i < n; i++) { u32 b = BC_(p[i]); h = fnv(h, &b, 4); }
    return h;
}

static u64_ img_hash(u64_ h, const void* p, unsigned n, const ODesc* d, int nd)
{
    static unsigned char tmp[65536];
    unsigned i, j;
    for (i = 0; i < n; i++) tmp[i] = ((const unsigned char*) p)[i];
    if (DUMP_N && n == DUMP_N) { printf("  RAWG:"); for (i = 0; i + 4 <= n; i += 4) printf(" %08x", *(u32*) (tmp + i)); printf("%c", 10); }
    onorm(tmp, d, nd);
    for (i = 0; i < (unsigned) nd; i++) {
        if (d[i].kind == 'b') for (j = 0; j < d[i].size; j++) tmp[d[i].off + j] = 0;
        else if (d[i].kind == 'P') {
            u32 v = (u32) tmp[d[i].off] | (u32) tmp[d[i].off + 1] << 8 | (u32) tmp[d[i].off + 2] << 16 | (u32) tmp[d[i].off + 3] << 24;
            if (v) { u64_ r = normptr((const void*) (unsigned long long) v); v = (u32) (r >> 32) * 0x10000u + (u32) r; }
            for (j = 0; j < 4; j++) tmp[d[i].off + j] = (unsigned char) (v >> (8 * j));
        }
    }
    if (DUMP_N && n == sizeof(HSD_JObj)) { printf("  IMGJ:"); for (i = 0; i + 4 <= n; i += 4) printf(" %08x", *(u32*) (tmp + i)); printf("%c", 10); }
    if (DUMP_N && n == DUMP_N) { printf("  IMGG:"); for (i = 0; i + 4 <= n; i += 4) printf(" %08x", *(u32*) (tmp + i)); printf("%c", 10); }
    return fnv(h, tmp, n);
}
/* a pointer-sized word that names a registered region is replaced by its name; other words stay */
static u64_ word_hash(u64_ h, const void* p, unsigned n)
{
    unsigned i;
    for (i = 0; i + 8 <= n; i += 8) {
        u64_ w = *(const u64_*) ((const unsigned char*) p + i);
        int j;
        for (j = 0; j < NOREGS; j++)
            if ((const unsigned char*) w >= OREGS[j].b && (const unsigned char*) w < OREGS[j].b + OREGS[j].n && w > 0xFFFFFFFFull) {
                w = normptr((const void*) w);
                break;
            }
        h = fnv(h, &w, 8);
    }
    return h;
}
""")
    hw = ["static u64_ WH[64];\nstatic int NWH;\nstatic u64_ world_hash(int fi)\n{\n    u64_ h = FNV0;\n    int i;\n    (void) fi;\n    NWH = 0;",
          "    for (i = 0; i < NGO; i++) { " + c_hash("OGO[i]", "HSD_GObj") + " }\n    WH[NWH++] = h;",
          "    for (i = 0; i < NGO; i++) {\n        switch (GVAR[fi]) {"]
    for k in range(len(usets)):
        hw.append(f"        case {k}: " + c_hash("OGP[i]", f"Ground_{k}") + " break;")
    hw.append("        }\n    }\n    WH[NWH++] = h;")
    hw.append("    for (i = 0; i < NJO; i++) { " + c_hash("OJO[i]", "HSD_JObj") + " }\n    WH[NWH++] = h;")
    for g in spec.get("globals", []):
        hw.append("    " + c_hash(g["c"], g["ty"]) + " WH[NWH++] = h;")
    for t in pools:
        hw.append("    " + c_hash(f"OP_{t}[0]", t) + " " + c_hash(f"OP_{t}[1]", t) + " WH[NWH++] = h;")
    for i, st in enumerate(statics):
        hw.append(f"    h = img_hash(h, &{st['c']}, sizeof {st['c']}, D_st{i}, (int) ND_st{i});")
        if st.get("ty") and TYPES[st["ty"]]["bf"]:
            hw.append(f"    for (i = 0; i < {st.get('n', 1)}; i++) h = bf_hash_{st['ty']}(h, (const void*) ((const unsigned char*) &{st['c']} + i * sizeof({TYPES[st['ty']]['cs']})));")
        hw.append("    WH[NWH++] = h;")
    hw.append("    h = word_hash(h, OSCR, sizeof OSCR);\n    WH[NWH++] = h;\n    return h;\n}")
    S.append("\n".join(hw))
    hdr_items = ["(unsigned) sizeof(Ground)", "(unsigned) sizeof(HSD_GObj)", "(unsigned) sizeof(HSD_JObj)"]
    for g in spec.get("globals", []):
        hdr_items.append(f"(unsigned) sizeof({g['c']})")
    for st in statics:
        hdr_items.append(f"(unsigned) sizeof({st['c']})")
    S.append("static const char* CUR_FN; static int CUR_K; static const char* CUR_PH;\n__declspec(dllimport) void* __stdcall SetUnhandledExceptionFilter(long (__stdcall *)(void*));\nstatic long __stdcall gro_filter(void* ep) { (void) ep; printf(\"CRASH in %s case %d phase %s, log tail: %s\\n\", CUR_FN, CUR_K, CUR_PH, LGN > 1200 ? LG + LGN - 1200 : LG); _flushall(); return 1; }")
    S.append("int main(int argc, char** argv)\n{")
    S.append("    SetUnhandledExceptionFilter(gro_filter);")
    S.append("    int fi;\n    int verbose = argc > 2 && !strcmp(argv[1], \"-v\"); if (verbose) DUMP_N = (unsigned) sizeof(Ground);\n    unsigned nf = sizeof OFNS_ / sizeof OFNS_[0];")
    S.append('    printf("hdr' + " %u" * len(hdr_items) + '\\n", ' + ", ".join(hdr_items) + ");")
    S.append("""    world_reg();
    world_reg_fns();
    for (fi = 0; fi < (int) nf; fi++) {
        u64_ agg = FNV0;
        int k;
        const OFn* fn = &OFNS_[fi];
        int vb = verbose && !strcmp(argv[2], fn->name);
        DUMP_N = vb ? (unsigned) sizeof(Ground) : 0;
        for (k = 0; k < fn->n; k++) {
            u64_ h = FNV0, rv, hW, hO;
            u32 retbits = 0;
            int i;
            rs = 0x9E3779B9u ^ ((u32) fi * 7919u + (u32) k * 104729u + 1u);
            CUR_FN = fn->name; CUR_K = k;
            CUR_PH = "fill";
            world_fill(fi);
            CUR_PH = "pre";
            for (i = 0; i < 4; i++) AI[i] = rnd() % 4;
            for (i = 0; i < 2; i++) AF[i] = rf();
            for (i = 0; i < 2; i++) { AV3[i].x = rf(); AV3[i].y = rf(); AV3[i].z = rf(); }
            for (i = 0; i < 2; i++) { AV2[i].x = rf(); AV2[i].y = rf(); }
            for (i = 0; i < 2; i++) APF[i] = rf();
            for (i = 0; i < 2; i++) API[i] = (int) (rnd() % 8);
            case_pre(fi);
            if (vb) { printf("PRE%c", 10); world_hash(fi); printf("ENDPRE%c", 10); }
            LGN = 0;
            CUR_PH = "call";
            rv = call_fn(fi, &retbits);
            CUR_PH = "hash";
            LG[LGN] = 0;
            hO = fnv(FNV0, LG, LGN);
            hW = world_hash(fi);
            h = fnv(h, &hO, 8); h = fnv(h, &rv, 8); h = fnv(h, &hW, 8);
            h = hfl(h, (const float*) AV3, 6); h = hfl(h, (const float*) AV2, 4); h = hfl(h, APF, 2);
            h = fnv(h, API, sizeof API);
            if (vb)
                { int q; printf("%d %016llx ret=%llx W=%016llx L=%016llx H=", k, h, rv, hW, hO); for (q = 0; q < NWH; q++) printf("%llx,", WH[q] & 0xFFFFFFull); printf("\\n  %s\\n", LG); printf("  ARGS:"); for (q = 0; q < 6; q++) printf(" %08x", ((const u32*) AV3)[q]); for (q = 0; q < 4; q++) printf(" %08x", ((const u32*) AV2)[q]); for (q = 0; q < 2; q++) printf(" %08x", ((const u32*) APF)[q]); printf("\\n"); for (q = 0; q < NGO; q++) { int w; printf("  OGP%d:", q); for (w = 0; w < (int) sizeof(Ground); w += 4) printf(" %08x", *(u32*) ((char*) &OGP[q] + w)); printf("\\n"); } STDUMP(); }
            agg = fnv(agg, &h, 8);
        }
        if (!verbose) printf("%s %016llx\\n", fn->name, agg);
    }
    return 0;
}""")
    drv = "\n".join(S) + "\n"
    dp = O.BUILD / f"gro_{stem}.c"
    dp.write_text(drv, encoding="utf-8")
    exe = link_low([O.ship_obj(dp, f"gro_{stem}")], f"gro_{stem}")
    res = subprocess.run([str(exe)] + (["-v", verbose_fn] if verbose_fn else []), capture_output=True, text=True, timeout=900)
    if res.returncode != 0 or "ASSERT" in res.stdout:
        sys.stderr.write(res.stdout[-3000:] + res.stderr[-3000:])
        raise SystemExit(f"gro: {stem}: C driver exited {res.returncode} (an assert or a crash: see output above)")
    out = res.stdout
    dst = CRATE / f"tests/data/gro_{stem}.txt"
    if verbose_fn:
        (O.BUILD / f"gro_{stem}_verbose.txt").write_text(out, encoding="utf-8", newline="\n")
        print(f"gro: {stem}: verbose {verbose_fn} -> {O.BUILD / ('gro_' + stem + '_verbose.txt')}")
        return
    dst.write_text(out, encoding="utf-8", newline="\n")
    print(f"gro: {stem}: {len(plans)} functions, {len(stub_info)} stubs -> {dst}")

    # ------------------------------------------------------------- the Rust test
    R = [f"//! @generated by rust/melee_world/gen/oracle/gr_oracle.py from gen/oracle/spec_{stem}.py. Do not edit.",
         f"//! State oracle of src/tu/{rs_main}.rs against {src} (tests/data/gro_{stem}.txt).",
         "#![allow(unused_variables, unused_imports, unused_mut, unused_unsafe, static_mut_refs, non_snake_case, non_upper_case_globals, non_camel_case_types, clippy::all)]",
         "mod oracle_common;", "use core::ffi::c_void;", "use core::ptr::{addr_of, addr_of_mut};",
         f"use melee_world::tu::{rs_main} as t;", "use melee_types::*;", "use oracle_common::*;", spec.get("use_rs", ""), RS_LG_STR]
    for tag, t in TYPES.items():
        R.append(f"static D_{tag}: &[ODesc] = &[" + ", ".join(f"({o}, {s}, b'{kd}')" for o, s, kd in t["desc"]) + "];")
        R.append(f"static F_{tag}: &[ODesc] = &[" + ", ".join(f"({o}, {s}, b'{kd}')" for o, s, kd in t["desc"] if kd not in ('b', 'P')) + "];")
        if t["bf"]:
            R.append(rs_bf_funcs(tag))
    for i, st in enumerate(statics):
        R.append(f"static D_st{i}: &[ODesc] = &[" + ", ".join(f"({o}, {s}, b'{kd}')" for o, s, kd in st["desc"]) + "];")
        R.append(f"static F_st{i}: &[ODesc] = &[" + ", ".join(f"({o}, {s}, b'{kd}')" for o, s, kd in st["desc"] if kd not in ('b', 'P')) + "];")
    R.append(f"const NGO: usize = {NGO};\nconst NJO: usize = {NJO};")
    R.append("static GVAR: &[usize] = &[" + ", ".join(str(x) for x in gv_of) + "];")
    R.append("static mut OGO: [HSD_GObj; NGO] = unsafe { core::mem::zeroed() };\nstatic mut OGP: [Ground; NGO] = unsafe { core::mem::zeroed() };\nstatic mut OJO: [HSD_JObj; NJO] = unsafe { core::mem::zeroed() };")
    for g in spec.get("globals", []):
        n8 = (pr.type(g['ty']).get_size() + 7) // 8
        R.append(f"static mut G_{g['c']}: [u64; {n8}] = [0; {n8}];")
    for t, sz in pools.items():
        R.append(f"static mut OP_{t}: [[u64; {(sz + 7) // 8}]; 2] = [[0; {(sz + 7) // 8}]; 2];")
    R.append("static mut AV3: [Vec3; 2] = [Vec3 { x: 0.0, y: 0.0, z: 0.0 }; 2];\nstatic mut AV2: [Vec2; 2] = [Vec2 { x: 0.0, y: 0.0 }; 2];\nstatic mut APF: [f32; 2] = [0.0; 2];\nstatic mut API: [i32; 2] = [0; 2];\nstatic mut AI: [u32; 4] = [0; 4];\nstatic mut AF: [f32; 2] = [0.0; 2];")
    R.append("""unsafe fn gen_gobj() -> *mut c_void { let k = rnd() % (NGO as u32 + 1); if k != 0 { addr_of_mut!(OGO[(k - 1) as usize]) as *mut c_void } else { core::ptr::null_mut() } }
unsafe fn gen_jobj() -> *mut c_void { let k = rnd() % (NJO as u32 + 1); if k != 0 { addr_of_mut!(OJO[(k - 1) as usize]) as *mut c_void } else { core::ptr::null_mut() } }
unsafe fn gen_ground() -> *mut c_void { let k = rnd() % (NGO as u32 + 1); if k != 0 { addr_of_mut!(OGP[(k - 1) as usize]) as *mut c_void } else { core::ptr::null_mut() } }""")
    for t in pools:
        R.append(f"unsafe fn gen_rec_{t}() -> *mut c_void {{ let k = rnd() % 3; if k != 0 {{ addr_of_mut!(OP_{t}[(k - 1) as usize]) as *mut c_void }} else {{ core::ptr::null_mut() }} }}")
    R.append("unsafe fn world_reg() {")
    R.append("    oreg(addr_of!(OGO), core::mem::size_of_val(&*addr_of!(OGO)));\n    oreg(addr_of!(OGP), core::mem::size_of_val(&*addr_of!(OGP)));\n    oreg(addr_of!(OJO), core::mem::size_of_val(&*addr_of!(OJO)));")
    for g in spec.get("globals", []):
        R.append(f"    oreg(addr_of!(G_{g['c']}), core::mem::size_of_val(&*addr_of!(G_{g['c']})));")
    for t in pools:
        R.append(f"    oreg(addr_of!(OP_{t}[0]), core::mem::size_of_val(&*addr_of!(OP_{t}[0])));\n    oreg(addr_of!(OP_{t}[1]), core::mem::size_of_val(&*addr_of!(OP_{t}[1])));")
    for st in statics:
        R.append(f"    oreg(addr_of!({st['rs']}), core::mem::size_of_val(&*addr_of!({st['rs']})));")
    R.append("    oreg(addr_of!(OSCR), core::mem::size_of_val(&*addr_of!(OSCR)));\n}")
    R.append("unsafe fn world_reg_fns() {")
    for n, expr in spec.get("fnvals", {}).items():
        R.append(f"    ofn(({expr}) as *const () as usize);")
    R.append("}")
    R.append("unsafe fn world_fill(fi: usize) {")
    R.append("    OGO = core::mem::zeroed(); OGP = core::mem::zeroed(); OJO = core::mem::zeroed();")
    for g in spec.get("globals", []):
        R.append(f"    G_{g['c']}.fill(0);")
    for t in pools:
        R.append(f"    OP_{t} = [[0; {(pools[t] + 7) // 8}]; 2];")
    for st in statics:
        if not st["const"]:
            R.append(f"    core::ptr::write_bytes(addr_of_mut!({st['rs']}) as *mut u8, 0, core::mem::size_of_val(&*addr_of!({st['rs']})));")
    R.append("    OSCR = [[0; 64]; 4];")
    R.append("    for i in 0..NGO { " + rs_fill("OGO[i]", "HSD_GObj") + " }")
    R.append("    for i in 0..NGO {\n        match GVAR[fi] {")
    for k in range(len(usets)):
        R.append(f"            {k} => {{ " + rs_fill("OGP[i]", f"Ground_{k}") + " }")
    R.append("            _ => unreachable!(),\n        }\n    }")
    R.append("    for i in 0..NJO { " + rs_fill("OJO[i]", "HSD_JObj") + " }")
    for g in spec.get("globals", []):
        R.append("    " + rs_fill(f"G_{g['c']}", g["ty"]))
    for t in pools:
        R.append("    " + rs_fill(f"OP_{t}[0]", t) + " " + rs_fill(f"OP_{t}[1]", t))
    for i, st in enumerate(statics):
        if st["fill"]:
            R.append(f"    ofill(addr_of_mut!({st['rs']}) as *mut u8, F_st{i});")
        if st.get("ty") and TYPES[st["ty"]]["bf"] and st["fill"]:
            R.append(f"    for i in 0..{st.get('n', 1)} {{ bf_fill_{st['ty']}((addr_of_mut!({st['rs']}) as *mut u8).add(i * core::mem::size_of::<{st['ty']}>()) as *mut {st['ty']}); }}")
    R.append("""    for i in 0..NGO {
        OGO[i].user_data = addr_of_mut!(OGP[i]) as *mut c_void;
        OGO[i].hsd_obj = addr_of_mut!(OJO[i]) as *mut _;
        OGP[i].gobj = addr_of_mut!(OGO[i]);
        OJO[i].child = addr_of_mut!(OJO[NGO + i]);
        OJO[NGO + i].next = addr_of_mut!(OJO[2 * NGO + i]);
        OJO[NGO + i].parent = addr_of_mut!(OJO[i]);
        OJO[2 * NGO + i].parent = addr_of_mut!(OJO[i]);
    }""")
    R.append(gr_)
    R.append(spec.get("links_rs", ""))
    R.append("}")
    R.append("struct Fn_ { name: &'static str, n: u32 }\nconst FNS: &[Fn_] = &[")
    for f, c, kinds, rk, n in plans:
        R.append(f'    Fn_ {{ name: "{f["name"]}", n: {n} }},')
    R.append("];")
    R.append("struct Args { i: [u32; 4], f: [f32; 2], v3: [Vec3; 2], v2: [Vec2; 2], pf: [f32; 2], pi: [i32; 2] }")
    R.append("unsafe fn call_fn(fi: usize, a: &mut Args) -> (u64, u32) {\n    let g: *mut HSD_GObj = addr_of_mut!(OGO[0]);\n    match fi {")
    for fi, (f, c, kinds, rk, n) in enumerate(plans):
        argsx, ix, fx, v3x, v2x, pfx, pix = [], 0, 0, 0, 0, 0, 0
        for k in kinds:
            if k == "G":
                argsx.append("g")
            elif k == "v3":
                argsx.append(f"&mut a.v3[{v3x}]"); v3x += 1
            elif k == "v2":
                argsx.append(f"&mut a.v2[{v2x}]"); v2x += 1
            elif k == "pf":
                argsx.append(f"&mut a.pf[{pfx}]"); pfx += 1
            elif k == "pi":
                argsx.append(f"&mut a.pi[{pix}]"); pix += 1
            elif k in ("i", "q"):
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
        if n not in hooks:
            continue
        kinds, rk, ret, pool = stub_info[n]
        for m, info in hooks[n]:
            R.append(rs_hook_install(n, info, kinds, rk, ret, m, pool, variadic.get(n)))
    R.append(spec.get("hooks_rs", ""))
    R.append("}")
    R.append("unsafe fn case_pre(fi: usize, a: &mut Args) {\n    match fi {")
    for fi, (f, c, kinds, rk, n) in enumerate(plans):
        if f.get("pre_rs"):
            R.append(f"        {fi} => {{ {f['pre_rs']} }}")
    R.append("        _ => {}\n    }\n}")
    R.append("static mut DUMP_N: usize = 0;")
    R.append("unsafe fn img_hash(h: u64, img: &[u8], d: &[ODesc]) -> u64 {\n    let mut tmp: std::vec::Vec<u8> = img.to_vec();\n    if DUMP_N != 0 && tmp.len() == DUMP_N { let mut line = std::string::String::from(\"  RAWG:\"); for c in tmp.chunks_exact(4) { line.push_str(&format!(\" {:08x}\", u32::from_le_bytes(c.try_into().unwrap()))); } println!(\"{}\", line); }\n    onorm(&mut tmp, d);\n    for &(off, size, kind) in d {\n        if kind == b'b' { for j in 0..size as usize { tmp[off as usize + j] = 0; } }\n        else if kind == b'P' {\n            let o = off as usize;\n            let mut v = u32::from_le_bytes(tmp[o..o + 4].try_into().unwrap());\n            if v != 0 { let r = normptr(v as usize); v = ((r >> 32) as u32).wrapping_mul(0x10000).wrapping_add(r as u32); }\n            tmp[o..o + 4].copy_from_slice(&v.to_le_bytes());\n        }\n    }\n    if DUMP_N != 0 && tmp.len() == core::mem::size_of::<HSD_JObj>() { let mut line = std::string::String::from(\"  IMGJ:\"); for c in tmp.chunks_exact(4) { line.push_str(&format!(\" {:08x}\", u32::from_le_bytes(c.try_into().unwrap()))); } println!(\"{}\", line); }\n    if DUMP_N != 0 && tmp.len() == DUMP_N { let mut line = std::string::String::from(\"  IMGG:\"); for c in tmp.chunks_exact(4) { line.push_str(&format!(\" {:08x}\", u32::from_le_bytes(c.try_into().unwrap()))); } println!(\"{}\", line); }\n    fnv(h, &tmp)\n}")
    R.append("""unsafe fn word_hash(mut h: u64, img: &[u8]) -> u64 {
    let mut i = 0;
    while i + 8 <= img.len() {
        let mut w = u64::from_le_bytes(img[i..i + 8].try_into().unwrap());
        for &(b, n) in OREGS.iter() {
            if w > 0xFFFF_FFFF && (w as usize) >= b && (w as usize) < b + n {
                w = normptr(w as usize);
                break;
            }
        }
        h = fnv(h, &w.to_le_bytes());
        i += 8;
    }
    h
}""")
    R.append("static mut WH: std::vec::Vec<u64> = std::vec::Vec::new();")
    R.append("unsafe fn stdump() {\n" + "\n".join(f'    {{ let mut line = String::from("  ST{i}:"); let base = addr_of!({st["rs"]}) as *const u8; for w in (0..core::mem::size_of_val(&*addr_of!({st["rs"]}))).step_by(4) {{ line.push_str(&format!(" {{:08x}}", core::ptr::read_unaligned(base.add(w) as *const u32))); }} println!("{{}}", line); }}' for i, st in enumerate(statics)) + "\n    for (i, f) in OFNS.iter().enumerate() { println!(\"  FN{}: {:x}\", i, f); }\n}")
    R.append("unsafe fn world_hash(fi: usize) -> u64 {\n    let mut h = FNV0;\n    WH.clear();")
    R.append("    for i in 0..NGO { " + rs_hash("OGO[i]", "HSD_GObj") + " }\n    WH.push(h);")
    R.append("    for i in 0..NGO {\n        match GVAR[fi] {")
    for k in range(len(usets)):
        R.append(f"            {k} => {{ " + rs_hash("OGP[i]", f"Ground_{k}") + " }")
    R.append("            _ => unreachable!(),\n        }\n    }\n    WH.push(h);")
    R.append("    for i in 0..NJO { " + rs_hash("OJO[i]", "HSD_JObj") + " }\n    WH.push(h);")
    for g in spec.get("globals", []):
        R.append("    " + rs_hash(f"G_{g['c']}", g["ty"], pr.type(g["ty"]).get_size()) + " WH.push(h);")
    for t in pools:
        R.append("    " + rs_hash(f"OP_{t}[0]", t) + " " + rs_hash(f"OP_{t}[1]", t) + " WH.push(h);")
    for i, st in enumerate(statics):
        R.append(f"    h = img_hash(h, core::slice::from_raw_parts(addr_of!({st['rs']}) as *const u8, core::mem::size_of_val(&*addr_of!({st['rs']}))), D_st{i});")
        if st.get("ty") and TYPES[st["ty"]]["bf"]:
            R.append(f"    for i in 0..{st.get('n', 1)} {{ h = bf_hash_{st['ty']}(h, (addr_of!({st['rs']}) as *const u8).add(i * core::mem::size_of::<{st['ty']}>()) as *const {st['ty']}); }}")
        R.append("    WH.push(h);")
    R.append("    h = word_hash(h, core::slice::from_raw_parts(addr_of!(OSCR) as *const u8, core::mem::size_of_val(&*addr_of!(OSCR))));\n    WH.push(h);\n    h\n}")
    sizes = ["core::mem::size_of::<Ground>()", "core::mem::size_of::<HSD_GObj>()", "core::mem::size_of::<HSD_JObj>()"]
    for g in spec.get("globals", []):
        sizes.append(f"core::mem::size_of::<{g['ty']}>()")
    for st in statics:
        sizes.append(f"core::mem::size_of_val(&*addr_of!({st['rs']}))")
    R.append("""#[test]
fn state_matches_c() {
    unsafe {
        let want = include_str!("data/gro_@STEM@.txt");
        let mut lines = want.lines();
        let verbose = std::env::var("ORACLE_VERBOSE").ok(); if verbose.is_some() { DUMP_N = core::mem::size_of::<Ground>(); }
        let hdr: std::vec::Vec<usize> = lines.next().unwrap().split_whitespace().skip(1).map(|s| s.parse().unwrap()).collect();
        let sizes: std::vec::Vec<usize> = std::vec![@SIZES@];
        assert_eq!(hdr, sizes, "struct sizes (C native layout vs the Rust port)");
        install_hooks();
        world_reg();
        world_reg_fns();
        let mut bad = std::vec::Vec::new();
        for (fi, fnn) in FNS.iter().enumerate() {
            let mut agg = FNV0;
            let vb = verbose.as_deref() == Some(fnn.name);
            DUMP_N = if vb { core::mem::size_of::<Ground>() } else { 0 };


            for k in 0..fnn.n {
                let mut h = FNV0;
                RS = 0x9E37_79B9u32 ^ ((fi as u32).wrapping_mul(7919).wrapping_add(k.wrapping_mul(104729)).wrapping_add(1));
                world_fill(fi);
                let mut a = core::mem::zeroed::<Args>();
                for i in 0..4 { a.i[i] = rnd() % 4; }
                for i in 0..2 { a.f[i] = rf(); }
                for i in 0..2 { a.v3[i].x = rf(); a.v3[i].y = rf(); a.v3[i].z = rf(); }
                for i in 0..2 { a.v2[i].x = rf(); a.v2[i].y = rf(); }
                for i in 0..2 { a.pf[i] = rf(); }
                for i in 0..2 { a.pi[i] = (rnd() % 8) as i32; }
                case_pre(fi, &mut a); if vb { println!("PRE"); world_hash(fi); println!("ENDPRE"); }
                LG.clear();
                let (rv, _rb) = call_fn(fi, &mut a);
                let ho = fnv(FNV0, LG.as_bytes());
                let hw = world_hash(fi);
                for x in [ho, rv, hw] {
                    h = fnv(h, &x.to_le_bytes());
                }
                h = fnv(h, &canon_f32_bytes(&[a.v3[0].x, a.v3[0].y, a.v3[0].z, a.v3[1].x, a.v3[1].y, a.v3[1].z]));
                h = fnv(h, &canon_f32_bytes(&[a.v2[0].x, a.v2[0].y, a.v2[1].x, a.v2[1].y]));
                h = fnv(h, &canon_f32_bytes(&a.pf));
                h = fnv(h, bytes(&a.pi));
                if vb {
                    let hs: std::vec::Vec<std::string::String> = WH.iter().map(|x| format!("{:x},", x & 0xFF_FFFF)).collect();
                    println!("{} {:016x} ret={:x} W={:016x} L={:016x} H={}\n  {}", k, h, rv, hw, ho, hs.concat(), LG); println!("  ARGS: {}", a.v3.iter().flat_map(|v| [v.x, v.y, v.z]).chain(a.v2.iter().flat_map(|v| [v.x, v.y])).chain(a.pf.iter().copied()).map(|f| format!("{:08x}", f.to_bits())).collect::<std::vec::Vec<_>>().join(" "));
                    for q in 0..NGO { let mut line = format!("  OGP{}:", q); let base = addr_of!(OGP[q]) as *const u8; for w in (0..core::mem::size_of::<Ground>()).step_by(4) { line.push_str(&format!(" {:08x}", core::ptr::read_unaligned(base.add(w) as *const u32))); } println!("{}", line); } stdump();
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
}""".replace("@STEM@", stem).replace("@SIZES@", ", ".join(sizes)))
    (CRATE / f"tests/gro_{stem}.rs").write_text("\n".join(R) + "\n", encoding="utf-8", newline="\n")
    print(f"gro: {stem}: wrote tests/gro_{stem}.rs")


def rs_hook_install(name, rh, kinds, rk, ret_t, modpath, pool, va=None):
    params = rh["params"]
    nfix = len(params) - (1 if va else 0)
    if nfix != len(kinds):
        raise SystemExit(f"gro: hook {name}: Rust has {nfix} fixed params, C has {len(kinds)}")
    sig = ", ".join(f"a{i}: {t}" for i, (_, t) in enumerate(params))
    body = [f'lg_s("{name}(");']
    for i, ((_, t), ck) in enumerate(zip(params[:nfix], kinds)):
        body.append(f"lg_str(a{i});" if ck == "str" else O.rs_arg_log(t, ck, f"a{i}").replace("lg_ps(", "gr_lg_ps("))
    if va:
        pat = va["va"]
        body.append(f'for (k, &w) in a{nfix}.iter().enumerate() {{ lg_vaw(w as u64, b"{pat}"[k.min({len(pat) - 1})]); }}')
    body.append('lg_s(")");')
    for i in O.OUTS.get(name, []):
        k = kinds[i]
        fn = {"v3": "lg_ov3", "v2": "lg_ov2", "pf": "lg_opf", "pi": "lg_opi"}[k]
        body.append(f"{fn}(a{i} as *mut _);")
    rt = rh["ret"]
    if rk == "v":
        body.append("lg_c(';');")
        rets = ""
    else:
        boolish = O.ret_is_bool(ret_t)
        txt = O.rs_ret(rt, rk if rk in ("i", "q", "f", "d") else "p", ret_t, boolish, name)
        if rk == "f" and name in O.RETS:
            txt = txt.replace("let r = rf();", f"let r = {O.RETS[name][1]};")
        if rk not in ("i", "q", "f", "d", "v") and name in O.RETS:
            txt = txt.replace("gen_p()", O.RETS[name][1])
        elif pool and name not in O.RETS:
            gen = {"gobj": "gen_gobj()", "jobj": "gen_jobj()", "ground": "gen_ground()"}.get(pool[0]) or f"gen_rec_{pool[1]}()"
            txt = txt.replace("gen_p()", gen)
        body.append(txt)
        rets = f" -> {rt}"
    return f"        melee_world::tu::{modpath}::ext::{rh['hook']} = Some(|{sig}|{rets} {{ {' '.join(body)} }});"


def coverage(stem, src):
    """llvm-cov report of the game .c under the generated driver (the driver built directly, without the shipping
    lowering: float semantics are irrelevant to which lines run)."""
    import os
    drv = O.BUILD / f"gro_{stem}.c"
    obj = O.BUILD / f"gro_{stem}_cov.obj"
    exe = O.BUILD / f"gro_{stem}_cov.exe"
    prof = O.BUILD / f"gro_{stem}.profraw"
    pd = O.BUILD / f"gro_{stem}.profdata"
    O.COMMON.extend(["-I", (ROOT / src).parent.as_posix()])
    patched = O.BUILD / f"gro_{stem}_src.c"
    cov_src = str(patched) if patched.exists() and load_spec(stem).get("c_patches") else str(ROOT / src)
    flags = [a for a in O.COMMON if a != "-Wno-everything"]
    ll = O.BUILD / f"gro_{stem}_cov.ll"
    O.run([O.CLANG, *flags, "-w", "-O0", "-fprofile-instr-generate", "-fcoverage-mapping", "-S", "-emit-llvm", str(drv), "-o", str(ll)], "coverage ir")
    O.run([O.CLANG, "--target=x86_64-pc-windows-msvc", "-O0", "-c", "-w", str(ll), "-o", str(obj)], "coverage compile")
    O.run([O.CLANG, "--target=x86_64-pc-windows-msvc", "-fprofile-instr-generate", str(obj), str(O.gekko_obj()), "-o", str(exe),
           "-fuse-ld=lld", "-Wl,/FORCE:UNRESOLVED", "-Wl,/IGNORE:4006", "-Wl,/IGNORE:4088"], "coverage link")
    env = dict(os.environ, LLVM_PROFILE_FILE=str(prof))
    subprocess.run([str(exe)], capture_output=True, text=True, check=True, env=env, timeout=900)
    O.run([str(Path(O.CLANG).with_name("llvm-profdata.exe")), "merge", "-sparse", str(prof), "-o", str(pd)], "profdata")
    r = O.run([str(Path(O.CLANG).with_name("llvm-cov.exe")), "report", str(exe), f"-instr-profile={pd}", "--show-functions", "--show-branch-summary",
               cov_src], "llvm-cov")
    print(r.stdout)
    sh = O.run([str(Path(O.CLANG).with_name("llvm-cov.exe")), "show", str(exe), f"-instr-profile={pd}", "--show-branches=count",
                "--show-line-counts-or-regions", cov_src], "llvm-cov show")
    (O.BUILD / f"gro_{stem}_cov.txt").write_text(sh.stdout, encoding="utf-8")
    print(f"gro: annotated source -> {O.BUILD / ('gro_' + stem + '_cov.txt')}")


def main():
    args = sys.argv[1:]
    verbose_fn = None
    if "-v" in args:
        i = args.index("-v")
        verbose_fn = args[i + 1]
        del args[i:i + 2]
    O.BUILD.mkdir(parents=True, exist_ok=True)
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
