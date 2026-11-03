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


def load_spec(stem):
    p = HERE / f"spec_{stem}.py"
    s = importlib.util.spec_from_file_location(f"gspec_{stem}", p)
    m = importlib.util.module_from_spec(s)
    s.loader.exec_module(m)
    return m.SPEC


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
    pr = GProbe(srcpath, includes)
    fspecs = []
    for f in spec["funcs"]:
        f = dict(f) if isinstance(f, dict) else dict(name=f)
        if f["name"] not in pr.funcs or not pr.funcs[f["name"]].is_definition():
            raise SystemExit(f"gro: {f['name']} is not defined in {src}")
        fspecs.append(f)

    # ---- pass 1: undefined callees
    head = [f'#include "{i}"' for i in includes] + ['#include "oracle.h"', spec.get("pre_c", ""), f'#include "{srcpath}"']
    p1 = O.BUILD / f"gro_{stem}_p1.c"
    p1.write_text("\n".join(head) + "\nint main(void) { return 0; }\n", encoding="utf-8")
    obj1 = O.ship_obj(p1, f"gro_{stem}_p1")
    nm = [l.split()[-1] for l in O.run([O.NM, "--undefined-only", str(obj1)], "nm").stdout.splitlines() if " U " in " " + l.strip() + " "]
    undef = sorted({n for n in nm if not n.startswith("__imp_")})
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
            if pr.funcs[n].type.is_function_variadic():
                first, rest = txt.split("\n", 1)
                first = first.rstrip(")") + ", ...)" if "(void)" not in first else first.replace("(void)", "(...)")
                txt = first + "\n" + rest
            pool = ret_pool(ret)
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
        TYPES[tag] = dict(c=ctype, desc=d, bf=bf)

    reg("HSD_GObj")
    reg("HSD_JObj")
    # Ground variants by the Ground.u members they use
    def uset(f):
        return tuple(sorted(f.get("u", spec.get("u", []))))
    usets = []
    for f in fspecs:
        u = uset(f)
        if u not in usets:
            usets.append(u)
    if not usets:
        usets.append(())
    for k, u in enumerate(usets):
        def keep(p, u=u):
            if p == "u" or not p.startswith("u."):
                return True
            return p.split(".")[1] in u
        reg(f"Ground_{k}", "Ground", "Ground", keep=keep)
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
            TYPES[t] = dict(c=t, desc=[], bf=[])
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
        st["fill"] = st["fill"] if "fill" in st else not st.get("ty")
        statics.append(st)

    # ---- emitters shared by C and Rust
    def cdesc(name, lst):
        return f"static const ODesc D_{name}[] = {{\n" + ",\n".join(f"    {{ {o}, {s}, '{k}' }}" for o, s, k in lst) + ("," if lst else "") + "\n    { 0, 0, 0 }\n};\n#define ND_" + name + " (sizeof D_" + name + " / sizeof D_" + name + "[0] - 1)\n"

    def cfill(name, lst):
        l2 = [x for x in lst if x[2] != 'b']
        return f"static const ODesc F_{name}[] = {{\n" + ",\n".join(f"    {{ {o}, {s}, '{k}' }}" for o, s, k in l2) + ("," if l2 else "") + "\n    { 0, 0, 0 }\n};\n#define NF_" + name + " (sizeof F_" + name + " / sizeof F_" + name + "[0] - 1)\n"

    def c_bf_funcs(tag):
        t = TYPES[tag]
        b = t["bf"]
        L = [f"static void bf_fill_{tag}({t['c']}* p)\n{{\n    (void) p;"]
        for cp, rp, nm_, w, sg in b:
            m = (1 << (w - 1 if sg else w)) - 1
            L.append(f"    p->{cp} = (rnd() & {m}u);")
        L.append("}")
        L.append(f"static u64_ bf_hash_{tag}(u64_ h, const {t['c']}* p)\n{{\n    u32 v;\n    (void) p; (void) v;")
        for cp, rp, nm_, w, sg in b:
            L.append(f"    v = (u32) p->{cp}; h = fnv(h, &v, 4);")
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
        L.append(f"unsafe fn bf_hash_{tag}(mut h: u64, p: *const {t['c']}) -> u64 {{\n    let _ = p;")
        for cp, rp, nm_, w, sg in b:
            L.append(f"    h = fnv(h, &(((*p).{rp + '.' if rp else ''}{nm_}()) as u32).to_le_bytes());")
        L.append("    h\n}")
        return "\n".join(L)

    def c_fill(expr, tag):
        return f"ofill(&{expr}, F_{tag}, (int) NF_{tag}); " + (f"bf_fill_{tag}(&{expr});" if TYPES[tag]["bf"] else "")

    def c_hash(expr, tag):
        return f"h = img_hash(h, &{expr}, sizeof {expr}, D_{tag}, (int) ND_{tag}); " + (f"h = bf_hash_{tag}(h, &{expr});" if TYPES[tag]["bf"] else "")

    def rs_fill(expr, tag):
        return f"ofill(addr_of_mut!({expr}) as *mut u8, F_{tag}); " + (f"bf_fill_{tag}(addr_of_mut!({expr}) as *mut {TYPES[tag]['c']});" if TYPES[tag]["bf"] else "")

    def rs_hash(expr, tag, size=None):
        base = f"core::slice::from_raw_parts(addr_of!({expr}) as *const u8, {size or f'core::mem::size_of_val(&*addr_of!({expr}))'})"
        return f"h = img_hash(h, {base}, D_{tag}); " + (f"h = bf_hash_{tag}(h, addr_of!({expr}) as *const {TYPES[tag]['c']});" if TYPES[tag]["bf"] else "")

    # ------------------------------------------------------------- the C driver
    S = [f'#include "{i}"' for i in includes] + ['#include "oracle.h"', spec.get("pre_c", ""), f'#include "{srcpath}"']
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
            S.append(f"    for (i = 0; i < {st.get('n', 1)}; i++) bf_fill_{st['ty']}((void*) ((unsigned char*) &{st['c']} + i * sizeof({st['ty']})));")
    S.append("""    for (i = 0; i < NGO; i++) {
        OGO[i].user_data = &OGP[i];
        OGO[i].hsd_obj = &OJO[i];
        OGP[i].gobj = &OGO[i];
        OJO[i].child = &OJO[NGO + i];
        OJO[NGO + i].next = &OJO[2 * NGO + i];
        OJO[NGO + i].parent = &OJO[i];
        OJO[2 * NGO + i].parent = &OJO[i];
    }""")
    S.append(spec.get("links_c", ""))
    S.append("}")
    S.append("typedef struct { const char* name; int n; } OFn;\nstatic const OFn OFNS_[] = {")
    plans = []
    for f in fspecs:
        c = pr.funcs[f["name"]]
        kinds = [O.kind_of(a.type) for a in c.get_arguments()]
        rk = O.kind_of(c.result_type)
        n = f.get("n", spec.get("n", N_CASES))
        S.append(f'    {{ "{f["name"]}", {n} }},')
        plans.append((f, c, kinds, rk, n))
    S.append("};")
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
    onorm(tmp, d, nd);
    for (i = 0; i < (unsigned) nd; i++)
        if (d[i].kind == 'b') for (j = 0; j < d[i].size; j++) tmp[d[i].off + j] = 0;
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
    hw = ["static u64_ world_hash(int fi)\n{\n    u64_ h = FNV0;\n    int i;\n    (void) fi;",
          "    for (i = 0; i < NGO; i++) { " + c_hash("OGO[i]", "HSD_GObj") + " }",
          "    for (i = 0; i < NGO; i++) {\n        switch (GVAR[fi]) {"]
    for k in range(len(usets)):
        hw.append(f"        case {k}: " + c_hash("OGP[i]", f"Ground_{k}") + " break;")
    hw.append("        }\n    }")
    hw.append("    for (i = 0; i < NJO; i++) { " + c_hash("OJO[i]", "HSD_JObj") + " }")
    for g in spec.get("globals", []):
        hw.append("    " + c_hash(g["c"], g["ty"]))
    for t in pools:
        hw.append("    " + c_hash(f"OP_{t}[0]", t) + " " + c_hash(f"OP_{t}[1]", t))
    for i, st in enumerate(statics):
        hw.append(f"    h = img_hash(h, &{st['c']}, sizeof {st['c']}, D_st{i}, (int) ND_st{i});")
        if st.get("ty") and TYPES[st["ty"]]["bf"]:
            hw.append(f"    for (i = 0; i < {st.get('n', 1)}; i++) h = bf_hash_{st['ty']}(h, (const void*) ((const unsigned char*) &{st['c']} + i * sizeof({st['ty']})));")
    hw.append("    h = word_hash(h, OSCR, sizeof OSCR);\n    return h;\n}")
    S.append("\n".join(hw))
    hdr_items = ["(unsigned) sizeof(Ground)", "(unsigned) sizeof(HSD_GObj)", "(unsigned) sizeof(HSD_JObj)"]
    for g in spec.get("globals", []):
        hdr_items.append(f"(unsigned) sizeof({g['c']})")
    for st in statics:
        hdr_items.append(f"(unsigned) sizeof({st['c']})")
    S.append("int main(int argc, char** argv)\n{")
    S.append("    int fi;\n    int verbose = argc > 2 && !strcmp(argv[1], \"-v\");\n    unsigned nf = sizeof OFNS_ / sizeof OFNS_[0];")
    S.append('    printf("hdr' + " %u" * len(hdr_items) + '\\n", ' + ", ".join(hdr_items) + ");")
    S.append("""    world_reg();
    world_reg_fns();
    for (fi = 0; fi < (int) nf; fi++) {
        u64_ agg = FNV0;
        int k;
        const OFn* fn = &OFNS_[fi];
        int vb = verbose && !strcmp(argv[2], fn->name);
        if (verbose && !vb) continue;
        for (k = 0; k < fn->n; k++) {
            u64_ h = FNV0, rv, hW, hO;
            u32 retbits = 0;
            int i;
            rs = 0x9E3779B9u ^ ((u32) fi * 7919u + (u32) k * 104729u + 1u);
            world_fill(fi);
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
            hO = fnv(FNV0, LG, LGN);
            hW = world_hash(fi);
            h = fnv(h, &hO, 8); h = fnv(h, &rv, 8); h = fnv(h, &hW, 8);
            h = hfl(h, (const float*) AV3, 6); h = hfl(h, (const float*) AV2, 4); h = hfl(h, APF, 2);
            h = fnv(h, API, sizeof API);
            if (vb)
                printf("%d %016llx ret=%llx W=%016llx L=%016llx\\n  %s\\n", k, h, rv, hW, hO, LG);
            agg = fnv(agg, &h, 8);
        }
        if (!verbose) printf("%s %016llx\\n", fn->name, agg);
    }
    return 0;
}""")
    drv = "\n".join(S) + "\n"
    dp = O.BUILD / f"gro_{stem}.c"
    dp.write_text(drv, encoding="utf-8")
    exe = O.link([O.ship_obj(dp, f"gro_{stem}")], f"gro_{stem}")
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
         f"use melee_world::tu::{rs_main} as t;", "use melee_types::*;", "use oracle_common::*;", spec.get("use_rs", "")]
    for tag, t in TYPES.items():
        R.append(f"static D_{tag}: &[ODesc] = &[" + ", ".join(f"({o}, {s}, b'{kd}')" for o, s, kd in t["desc"]) + "];")
        R.append(f"static F_{tag}: &[ODesc] = &[" + ", ".join(f"({o}, {s}, b'{kd}')" for o, s, kd in t["desc"] if kd != 'b') + "];")
        if t["bf"]:
            R.append(rs_bf_funcs(tag))
    for i, st in enumerate(statics):
        R.append(f"static D_st{i}: &[ODesc] = &[" + ", ".join(f"({o}, {s}, b'{kd}')" for o, s, kd in st["desc"]) + "];")
        R.append(f"static F_st{i}: &[ODesc] = &[" + ", ".join(f"({o}, {s}, b'{kd}')" for o, s, kd in st["desc"] if kd != 'b') + "];")
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
            R.append(rs_hook_install(n, info, kinds, rk, ret, m, pool))
    R.append(spec.get("hooks_rs", ""))
    R.append("}")
    R.append("unsafe fn case_pre(fi: usize, a: &mut Args) {\n    match fi {")
    for fi, (f, c, kinds, rk, n) in enumerate(plans):
        if f.get("pre_rs"):
            R.append(f"        {fi} => {{ {f['pre_rs']} }}")
    R.append("        _ => {}\n    }\n}")
    R.append("unsafe fn img_hash(h: u64, img: &[u8], d: &[ODesc]) -> u64 {\n    let mut tmp: std::vec::Vec<u8> = img.to_vec();\n    onorm(&mut tmp, d);\n    for &(off, size, kind) in d { if kind == b'b' { for j in 0..size as usize { tmp[off as usize + j] = 0; } } }\n    fnv(h, &tmp)\n}")
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
    R.append("unsafe fn world_hash(fi: usize) -> u64 {\n    let mut h = FNV0;")
    R.append("    for i in 0..NGO { " + rs_hash("OGO[i]", "HSD_GObj") + " }")
    R.append("    for i in 0..NGO {\n        match GVAR[fi] {")
    for k in range(len(usets)):
        R.append(f"            {k} => {{ " + rs_hash("OGP[i]", f"Ground_{k}") + " }")
    R.append("            _ => unreachable!(),\n        }\n    }")
    R.append("    for i in 0..NJO { " + rs_hash("OJO[i]", "HSD_JObj") + " }")
    for g in spec.get("globals", []):
        R.append("    " + rs_hash(f"G_{g['c']}", g["ty"], pr.type(g["ty"]).get_size()))
    for t in pools:
        R.append("    " + rs_hash(f"OP_{t}[0]", t) + " " + rs_hash(f"OP_{t}[1]", t))
    for i, st in enumerate(statics):
        R.append(f"    h = img_hash(h, core::slice::from_raw_parts(addr_of!({st['rs']}) as *const u8, core::mem::size_of_val(&*addr_of!({st['rs']}))), D_st{i});")
        if st.get("ty") and TYPES[st["ty"]]["bf"]:
            R.append(f"    for i in 0..{st.get('n', 1)} {{ h = bf_hash_{st['ty']}(h, (addr_of!({st['rs']}) as *const u8).add(i * core::mem::size_of::<{st['ty']}>()) as *const {st['ty']}); }}")
    R.append("    h = word_hash(h, core::slice::from_raw_parts(addr_of!(OSCR) as *const u8, core::mem::size_of_val(&*addr_of!(OSCR))));\n    h\n}")
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
        let verbose = std::env::var("ORACLE_VERBOSE").ok();
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
            if verbose.is_some() && !vb {
                continue;
            }
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
                case_pre(fi, &mut a);
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
                    println!("{} {:016x} ret={:x} W={:016x} L={:016x}\\n  {}", k, h, rv, hw, ho, LG);
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


def rs_hook_install(name, rh, kinds, rk, ret_t, modpath, pool):
    params = rh["params"]
    # a C variadic (`...`) is a trailing `&[usize]` slice on the Rust side; the stub logs only the named arguments
    va = len(params) == len(kinds) + 1 and params[-1][1].replace(" ", "").startswith("&[")
    if len(params) != len(kinds) and not va:
        raise SystemExit(f"gro: hook {name}: Rust has {len(params)} params, C has {len(kinds)}")
    sig = ", ".join(f"a{i}: {t}" for i, (_, t) in enumerate(params))
    body = [f'lg_s("{name}(");']
    for i, ((_, t), ck) in enumerate(zip(params, kinds)):
        body.append(O.rs_arg_log(t, ck, f"a{i}"))
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
        if rk not in ("i", "q", "f", "d", "v") and name in O.RETS:
            txt = txt.replace("gen_p()", O.RETS[name][1])
        elif pool and name not in O.RETS:
            gen = {"gobj": "gen_gobj()", "jobj": "gen_jobj()", "ground": "gen_ground()"}.get(pool[0]) or f"gen_rec_{pool[1]}()"
            txt = txt.replace("gen_p()", gen)
        body.append(txt)
        rets = f" -> {rt}"
    return f"        melee_world::tu::{modpath}::ext::{rh['hook']} = Some(|{sig}|{rets} {{ {' '.join(body)} }});"


def main():
    args = sys.argv[1:]
    verbose_fn = None
    if "-v" in args:
        i = args.index("-v")
        verbose_fn = args[i + 1]
        del args[i:i + 2]
    O.BUILD.mkdir(parents=True, exist_ok=True)
    (CRATE / "tests/data").mkdir(parents=True, exist_ok=True)
    for stem in args:
        build(stem, verbose_fn)


if __name__ == "__main__":
    main()
