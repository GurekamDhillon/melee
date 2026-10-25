#!/usr/bin/env python3
"""Writes both halves of the melee_runtime::tu state-oracle driver (lb/*.c ports) from one description:

  python rust/melee_runtime/gen/mtu/mk_lb.py <stem> [...]      (worktree root)

The C driver (gen/mtu/tu/<stem>.c) #includes the game's .c verbatim; the Rust test (tests/tu_<stem>.rs)
makes the same draws. Each function of the TU is one op; its arguments are drawn by parameter type
(overrides in lbspecs.CFG); a small world of record pools (the spec's `pools`), the TU's own data
(`own_fill` / `own_hash`) and scratch arrays is filled from a seeded generator before every case and
hashed after it, together with the return value and the call trace of the recording stubs.
  python rust/melee_runtime/gen/mtu/run_tu.py <stem>        (C reference output)
  cargo test -p melee_runtime --test tu_<stem>
"""
import os, re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
import gen_tu  # noqa: E402
from gen_tu import ci, K, T, canon, Tu  # noqa: E402

BASE_POOLS = []  # (type, name, count) every stem has


def ctype(tu, name):
    return ("struct " + name) if name in tu.tagged else name


def world_c(tu, pools):
    c = []
    for t, a, n in pools:
        c.append(f"static {ctype(tu, t)} {a}[{n}];")
    c += ["static " + ("Vec3" if "Vec3" in tu.types else "struct { float x, y, z; }") + " VA[4];", "static u8 U8A[16];", "static int IA[4];", "static u32 UA[4];", "static float FA[4];",
          "static unsigned char BUF[256];", "static void* PPA[4];", "static char STRS[4][64] = { \"a.dat\", \"sym\", \"LbAd.dat\", \"x\" };", ""]
    for t, a, n in pools:
        c.append(f"static {ctype(tu, t)}* pick_{t}(void) {{ u32 r = rnd(); return &{a}[(r >> 8) % {n}]; }}")
        c.append(f"static {ctype(tu, t)}* picko_{t}(void) {{ u32 r = rnd(); return (r % 6) == 0 ? 0 : &{a}[(r >> 8) % {n}]; }}")
    c += [
        "struct HSD_GObj; typedef struct HSD_GObj HSD_GObj;",
        "static void cbev1(HSD_GObj* g) { T(7001); T(PI(g)); }",
        "static void cbev2(HSD_GObj* g) { T(7002); T(PI(g)); }",
        "static int cbp1(HSD_GObj* g) { T(7003); T(PI(g)); return (int) (fv() & 1); }",
        "static int cbp2(HSD_GObj* g) { T(7004); T(PI(g)); return (int) (fv() & 1); }",
        "static void cbev3(HSD_GObj* g, HSD_GObj* h) { T(7005); T(PI(g)); T(PI(h)); }",
        "static void cbi1(int a) { T(7006); T((u32) a); }",
        "static void cbi2(int a) { T(7007); T((u32) a); }",
        *autocb_defs()[0],
        "static void* pick_fn(int kind)",
        "{",
        "    u32 r = rnd();",
        "    if ((r & 3) == 0) return 0;",
        "    switch (kind) {",
        "    case 1: return (r & 4) ? (void*) cbev1 : (void*) cbev2;",
        "    case 2: return (r & 4) ? (void*) cbp1 : (void*) cbp2;",
        "    case 3: return (void*) cbev3;",
        "    case 4: return (r & 4) ? (void*) cbi1 : (void*) cbi2;",
        "    }",
        "    return 0;",
        "}",
        "static void* pick_cb(int kind) { void* f = pick_fn(kind); return f ? f : (kind == 2 ? (void*) cbp1 : kind == 3 ? (void*) cbev3 : kind == 4 ? (void*) cbi1 : (void*) cbev1); }",
        "static void freg_world(void) { freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3); freg((const void*) cbi1); freg((const void*) cbi2); " + " ".join(autocb_defs()[2]) + " }",
        "@@SPLIT@@",
        "static void preg_world(void)",
        "{",
    ]
    for t, a, n in pools:
        c.append(f"    preg({a}, sizeof {a});")
    c += ["    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);", "}", ""]
    c += ["static void setup_world(void)", "{", "    int i;", "    (void) i;"]
    for t, a, n in pools:
        c.append(f"    zero({a}, sizeof {a});")
    c += ["    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }",
          "    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }",
          "    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();", "    BUF[255] = 0;", "    for (i = 0; i < 4; i++) PPA[i] = 0;"]
    for t, a, n in pools:
        c.append(f"    for (i = 0; i < {n}; i++) fill_{t}(&{a}[i]);")
    c += ["    fillg_all();", "    fillo_all();", "}", ""]
    c += ["static void hash_world(void)", "{", "    int i;", "    (void) i;"]
    for t, a, n in pools:
        c.append(f"    for (i = 0; i < {n}; i++) hash_{t}(&{a}[i]);")
    c += ["    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }",
          "    for (i = 0; i < 16; i++) HM(U8A[i]);",
          "    for (i = 0; i < 256; i++) HM(BUF[i]);", "    for (i = 0; i < 4; i++) HM(PI(PPA[i]));",
          "    hashg_all();", "    hasho_all();", "}", ""]
    return "\n".join(c)


def world_rs(pools):
    r = ["pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };",
         "pub static mut U8A: [u8; 16] = [0; 16];", "pub static mut IA: [i32; 4] = [0; 4];", "pub static mut UA: [u32; 4] = [0; 4];",
         "pub static mut FA: [f32; 4] = [0.0; 4];",
         "pub static mut BUF: [u8; 256] = [0; 256];", "pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];",
         "pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];"]
    for t, a, n in pools:
        r.append(f"pub static mut {a}: [{t}; {n}] = unsafe {{ core::mem::zeroed() }};")
    for t, a, n in pools:
        r.append(f"pub fn pick_{t}() -> *mut {t} {{ let r = rnd(); unsafe {{ &raw mut {a}[((r >> 8) % {n}) as usize] }} }}")
        r.append(f"pub fn picko_{t}() -> *mut {t} {{ let r = rnd(); unsafe {{ if r % 6 == 0 {{ core::ptr::null_mut() }} else {{ &raw mut {a}[((r >> 8) % {n}) as usize] }} }} }}")
    r += [
        "pub unsafe extern \"C\" fn cbev1(g: *mut HSD_GObj) { t(7001); t(pi(g)); }",
        "pub unsafe extern \"C\" fn cbev2(g: *mut HSD_GObj) { t(7002); t(pi(g)); }",
        "pub unsafe extern \"C\" fn cbp1(g: *mut HSD_GObj) -> i32 { t(7003); t(pi(g)); (fv() & 1) as i32 }",
        "pub unsafe extern \"C\" fn cbp2(g: *mut HSD_GObj) -> i32 { t(7004); t(pi(g)); (fv() & 1) as i32 }",
        "pub unsafe extern \"C\" fn cbev3(g: *mut HSD_GObj, h: *mut HSD_GObj) { t(7005); t(pi(g)); t(pi(h)); }",
        "pub unsafe extern \"C\" fn cbi1(a: i32) { t(7006); t(a as u32); }",
        "pub unsafe extern \"C\" fn cbi2(a: i32) { t(7007); t(a as u32); }",
        *autocb_defs()[1],
        "pub fn pick_fn(kind: i32) -> usize {",
        "    let r = rnd();",
        "    if r & 3 == 0 { return 0; }",
        "    match kind {",
        "        1 => if r & 4 != 0 { cbev1 as usize } else { cbev2 as usize },",
        "        2 => if r & 4 != 0 { cbp1 as usize } else { cbp2 as usize },",
        "        3 => cbev3 as usize,",
        "        4 => if r & 4 != 0 { cbi1 as usize } else { cbi2 as usize },",
        "        _ => 0,",
        "    }",
        "}",
        "pub fn pick_cb(kind: i32) -> usize { let f = pick_fn(kind); if f != 0 { f } else if kind == 2 { cbp1 as usize } else if kind == 3 { cbev3 as usize } else if kind == 4 { cbi1 as usize } else { cbev1 as usize } }",
        "pub fn freg_world() { freg(cbev1 as usize); freg(cbev2 as usize); freg(cbp1 as usize); freg(cbp2 as usize); freg(cbev3 as usize); freg(cbi1 as usize); freg(cbi2 as usize); " + " ".join(autocb_defs()[3]) + " }",
        "pub fn preg_world() {",
        "    unsafe {",
    ]
    for t, a, n in pools:
        r.append(f"        preg(&raw const {a}, core::mem::size_of_val(&{a}));")
    r += ["        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);",
          "        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));", "    }", "}", ""]
    r += ["pub unsafe fn setup_world() {"]
    for t, a, n in pools:
        r.append(f"    {a} = core::mem::zeroed();")
    r += ["    for (k, st) in [&b\"a.dat\"[..], &b\"sym\"[..], &b\"LbAd.dat\"[..], &b\"x\"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }",
          "    for i in 0..16 { U8A[i] = rnd() as u8; }",
          "    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }",
          "    for i in 0..256 { BUF[i] = rnd() as u8; }", "    BUF[255] = 0;", "    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }"]
    for t, a, n in pools:
        r.append(f"    for i in 0..{n} {{ g::fill_{t}(&mut {a}[i]); }}")
    r += ["    g::fillg_all();", "    g::fillo_all();", "}", ""]
    r += ["pub unsafe fn hash_world() {"]
    for t, a, n in pools:
        r.append(f"    for i in 0..{n} {{ g::hash_{t}(&{a}[i]); }}")
    r += ["    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }",
          "    for i in 0..16 { hm(U8A[i] as u32); }",
          "    for i in 0..256 { hm(BUF[i] as u32); }", "    for i in 0..4 { hm(pi(PPA[i])); }",
          "    g::hashg_all();", "    g::hasho_all();", "}", ""]
    return "\n".join(r)


AUTOCB = {}  # signature spelling -> (index, function type)


def autocb_defs():
    """C and Rust definitions of the generated callbacks: trace id 9000+N, every argument, a shared-stream return."""
    from gen_tu import arg_class, c_trace, r_trace, c_ret, r_ret, rust_ty
    cc, rr, reg_c, reg_r = [], [], [], []
    for sp, (n, ft) in AUTOCB.items():
        ats = list(ft.argument_types())
        rt = ft.get_result()
        rcls = arg_class(rt)
        cps = ", ".join(f"{a.spelling} a{i}" for i, a in enumerate(ats)) or "void"
        body = [f"T({9000 + n});"] + [c_trace(arg_class(a), f"a{i}") for i, a in enumerate(ats)]
        rps = ", ".join(f"a{i}: {rust_ty(a)}" for i, a in enumerate(ats))
        rbody = [f"t({9000 + n});"] + [r_trace(arg_class(a), f"a{i}") for i, a in enumerate(ats)]
        if rcls == "void":
            cc.append(f"static void autocb_{n}({cps}) {{ " + " ".join(body) + " }")
            rr.append(f"pub unsafe extern \"C\" fn autocb_{n}({rps}) {{ " + " ".join(rbody) + " }")
        else:
            cc.append(f"static {rt.spelling} autocb_{n}({cps}) {{ " + " ".join(body) + f" return ({rt.spelling}) ({c_ret(rcls)}); }}")
            rr.append(f"pub unsafe extern \"C\" fn autocb_{n}({rps}) -> {rust_ty(rt)} {{ " + " ".join(rbody) + f" {r_ret(rcls, rt)} }}")
        reg_c.append(f"freg((const void*) autocb_{n});")
        reg_r.append(f"freg(autocb_{n} as usize);")
    return cc, rr, reg_c, reg_r


FNK = {"void (int)": 4, "void (struct HSD_GObj *)": 1, "int (struct HSD_GObj *)": 2, "void (struct HSD_GObj *, struct HSD_GObj *)": 3}


def bind(ptype, pools):
    c = canon(ptype)
    k = c.kind
    pm = {t: (f"picko_{t}()", f"picko_{t}()") for t, a, n in pools}
    pn = {t: (f"pick_{t}()", f"pick_{t}()") for t, a, n in pools}
    if k == T.CONSTANTARRAY:
        return "(void*) BUF", "(&raw mut BUF) as *mut _"
    if k == T.FLOAT:
        return "rw()", "rw()"
    if k == T.DOUBLE:
        return "(double) rw()", "rw() as f64"
    if k == T.POINTER:
        pt = canon(c.get_pointee())
        if pt.kind in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO):
            kind = FNK.get(pt.spelling)
            if kind is None:
                # a callback of any other signature: a generated recording function (autocb_N), one per signature
                if pt.spelling not in AUTOCB:
                    AUTOCB[pt.spelling] = (len(AUTOCB), pt)
                n = AUTOCB[pt.spelling][0]
                return (f"({ptype.spelling}) autocb_{n}", f"Some(autocb_{n} as _)")
            return (f"({ptype.spelling}) pick_cb({kind})", f"unsafe {{ core::mem::transmute::<usize, _>(pick_cb({kind})) }}")
        if pt.kind == T.RECORD:
            n = pt.get_declaration().spelling
            if n == "Vec":
                n = "Vec3"
            if n in pn:
                return pn[n]
            if n.startswith("_") and n[1:] in pn:
                return pn[n[1:]]
            if n == "Vec3":
                return ("&VA[rnd() % 4]", "&raw mut VA[(rnd() % 4) as usize]")
        if pt.kind == T.POINTER:
            return ("(void*) &PPA[fv() % 4]", "(&raw mut PPA[(fv() % 4) as usize]) as *mut _")
        if pt.kind == T.VOID:
            return ("(void*) BUF", "(&raw mut BUF) as *mut c_void")
        if pt.kind == T.FLOAT:
            return ("&FA[rnd() % 4]", "&raw mut FA[(rnd() % 4) as usize]")
        if pt.kind in (T.UCHAR, T.CHAR_U):
            return ("U8A", "(&raw mut U8A) as *mut u8")
        if pt.kind in (T.CHAR_S, T.SCHAR):
            return ("STRS[rnd() % 4]", "(&raw mut STRS[(rnd() % 4) as usize]) as *mut i8")
        if pt.kind == T.INT:
            return ("IA", "(&raw mut IA) as *mut i32")
        if pt.kind == T.UINT:
            return ("UA", "(&raw mut UA) as *mut u32")
        if pt.kind == T.RECORD:
            # a record no pool describes: scratch memory (the callee reads or writes it as plain data)
            print(f"mk_lb: note: pointer to {pt.spelling} bound to BUF")
            return (f"({ptype.spelling}) BUF", "(&raw mut BUF) as *mut _")
        if pt.kind == T.CONSTANTARRAY:
            # pointer to a fixed array (float (*)[3]): scratch memory, written as plain data
            return (f"({ptype.spelling}) BUF", "(&raw mut BUF) as *mut _")
        raise SystemExit(f"no binding for pointer to {pt.spelling}")
    if k == T.BOOL:
        return "fv()", "fv() != 0"
    if k in (T.LONGLONG, T.ULONGLONG):
        return "fv64()", "fv64() as _"
    if k in gen_tu.INTS or k == T.ENUM:
        return "fv()", "fv() as _"
    raise SystemExit(f"no binding for {ptype.spelling}")


def retcls(rt):
    c = canon(rt)
    if c.kind == T.VOID:
        return "void"
    if c.kind == T.FLOAT:
        return "f4"
    if c.kind == T.DOUBLE:
        return "f8"
    if c.kind == T.POINTER:
        return "ptr"
    if c.kind in (T.LONGLONG, T.ULONGLONG):
        return "i8"
    return "i4"


def pub_fns(stem):
    """Names of the `pub fn`s of src/tu/<stem>.rs: statics of the C file the port keeps private are not ops (their
    callers cover them)."""
    import re
    txt = (ROOT / "rust/melee_runtime/src/tu" / f"{stem}.rs").read_text(encoding="utf-8")
    return set(re.findall(r"(?m)^\s*pub (?:unsafe )?(?:extern \"C\" )?fn (\w+)", txt))


def ops_of(tu, stem, cfg, pools):
    ops = []
    pub = pub_fns(stem) if cfg.get("pub_only") else None
    for name, cur in tu.defs.items():
        if name in cfg.get("skip", ()) or ("only" in cfg and name not in cfg["only"]):
            continue
        if pub is not None and name not in pub:
            print(f"mk_lb: {stem}: skipping private {name}")
            continue
        params = list(cur.get_arguments())
        ov = cfg.get("ov", {}).get(name, {})
        pre_c, pre_r, args = [], [], []
        for i, p in enumerate(params):
            if i in ov:
                ce, re_ = ov[i]
            else:
                ce, re_ = bind(p.type, pools)
            ct = "void*" if (canon(p.type).kind == T.CONSTANTARRAY or '(*)' in p.type.spelling) else p.type.spelling
            pre_c.append(f"{ct} a{i} = {ce};")
            pre_r.append(f"let a{i} = {re_};")
            args.append(f"a{i}")
        rc = retcls(cur.result_type)
        call_c = f"{name}({', '.join(args)})"
        call_r = f"m::{name}({', '.join(args)})"
        if name in cfg.get("call", {}):
            call_c, call_r = cfg["call"][name]
        if rc == "void":
            sc, sr = f"{call_c};", f"{call_r};"
        elif rc == "f4":
            sc, sr = f"ret = B({call_c});", f"ret = ({call_r}).to_bits();"
        elif rc == "f8":
            sc, sr = f"{{ double dd = {call_c}; HM64(dd); }}", f"{{ let dd = {call_r}; hm64(dd); }}"
        elif rc == "ptr":
            sc, sr = f"ret = PI({call_c});", f"ret = pi({call_r});"
        elif rc == "i8":
            sc, sr = f"{{ u64_t_ rr = (u64_t_) {call_c}; HM((u32) rr); ret = (u32) (rr >> 32); }}", f"{{ let rr = {call_r} as u64; hm(rr as u32); ret = (rr >> 32) as u32; }}"
        else:
            sc, sr = f"ret = (u32) {call_c};", f"ret = {call_r} as u32;"
        if name in cfg.get("stmt", {}):
            # the whole call statement, with {call} standing for the call expression (pointer results named by index)
            sc, sr = (x.replace("{call}", cc) for x, cc in zip(cfg["stmt"][name], (call_c, call_r)))
        post_c, post_r = cfg.get("post", {}).get(name, ("", ""))
        pre2_c, pre2_r = cfg.get("pre", {}).get(name, ("", ""))
        ops.append(dict(name=name, c=pre2_c + " " + " ".join(pre_c) + " " + sc + " " + post_c,
                        r=pre2_r + "\n" + "\n".join(pre_r) + "\n" + sr + "\n" + post_r,
                        n=cfg.get("n", {}).get(name, cfg.get("ncases", 300))))
    return ops


def emit(stem):
    from lbspecs import CFG, SPECS
    cfg = CFG[stem]
    spec = SPECS[stem]
    pools = cfg.get("pools", [])
    tu = Tu(gen_tu.patched_source(stem, spec), spec.get("args", ()), (ROOT / spec["src"]).parent)
    ops = ops_of(tu, stem, cfg, pools)
    nops = len(ops)
    c = ["extern int atoi(const char*);", "extern char* getenv(const char*);",
         f"/* TU: {spec['src']} */",
         "/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a",
         " * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */",
         cfg.get("c_pre", ""), "#include TU_SRC", '#include "mtu_common.h"', "@@DECL@@", f'#include "{stem}_gen.h"', cfg.get("c_extra", ""), "",
         "@@WORLD@@", "enum { " + ", ".join(f"OP_{o['name']} = {i}" for i, o in enumerate(ops)) + " };", "",
         "int main(int argc, char** argv)", "{", "    int op, k, only = -1;",
         "    if (argc > 1) only = atoi(argv[1]);",
         "    preg_world();", "    preg_own();", "    freg_all();", "    freg_world();", cfg.get("pregs", ("", ""))[0],
         f"    for (op = 0; op < {nops}; op++) {{", "        int n = 0;", "        if (only >= 0 && op != only) continue;", "        switch (op) {"]
    for i, o in enumerate(ops):
        c.append(f"        case {i}: n = {o['n']}; break;  /* {o['name']} */")
    c += ["        }", "        for (k = 0; k < n; k++) {", "            u32 ret = 0;", "            tu_case(op, k);", "            setup_world();",
          cfg.get("setup", ("", ""))[0],
          '            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }',
          "            ARMED = 1;", "            if (MTU_SETJMP()) { ARMED = 0; printf(\"%d %d ABORT\\n\", op, k); continue; }", "            switch (op) {"]
    for i, o in enumerate(ops):
        c.append(f"            case {i}: {{ {o['c']} }} break;")
    c += ["            }", "            ARMED = 0;", "            hash_world();", cfg.get("hash", ("", ""))[0],
          '            printf("%d %d %08X %08X %08X\\n", op, k, ret, H, TR);', "        }", "    }", "    return 0;", "}", ""]
    c_txt = "\n".join(c)
    wd, wf = world_c(tu, pools).split("@@SPLIT@@")
    c_txt = c_txt.replace("@@DECL@@", wd + cfg.get("c_decl", "")).replace("@@WORLD@@", wf)
    c_txt = "typedef unsigned long long u64_t_;\n" + c_txt if "u64_t_" in c_txt and "typedef unsigned long long u64_t_" not in open(HERE / "tu/mtu_common.h").read() else c_txt
    (HERE / "tu" / f"{stem}.c").write_text(c_txt, encoding="utf-8", newline="\n")

    r = [f"//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/{stem}.c.",
         f"//! src/tu/{stem}.rs vs the C (the whole TU, shipping contraction; regenerate the expected file",
         f"//! with `python rust/melee_runtime/gen/mtu/run_tu.py {stem}`).",
         "//! C functions of the TU this test covers (the private ones through their public callers): " + ", ".join(sorted(tu.defs)) + ".",
         "#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]",
         *[f"const OP_{o['name']}: u32 = {i};" for i, o in enumerate(ops)], "mod mtu_harness;", "use core::ffi::c_void;", "use core::ptr::null_mut;", "use melee_types::*;", f"use {cfg.get('rmod', 'melee_runtime::tu::' + stem)} as m;",
         "use mtu_harness::*;", f'#[path = "mtu/{stem}_gen.rs"]', "mod g;", "", cfg.get("rs_extra", ""), "",
         world_rs(pools),
         "// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),",
         "// which a libtest main thread cannot survive at exit.",
         f"fn main() {{ {stem}_matches_c(); println!(\"tu_{stem}: ok\"); }}", "", f"fn {stem}_matches_c() {{", "    unsafe {", "        g::install();", "        preg_world();", "        g::preg_own();", "        g::freg_all();",
         "        freg_world();", cfg.get("pregs", ("", ""))[1],
         "        install_abort_hook();",
         f'        let mut c = Check::new(include_str!("data/mtu_{stem}.txt"));', "        let mut total = 0u32;",
         f"        for op in 0..{nops}u32 {{", "            let n = match op {"]
    for i, o in enumerate(ops):
        r.append(f"                {i} => {o['n']}u32,")
    r += ["                _ => unreachable!(),", "            };", "            for k in 0..n {", "                case(op, k);",
          "                setup_world();", cfg.get("setup", ("", ""))[1], "                let mut ret = 0u32;",
          "                let aborted = run_case(|| unsafe { match op {"]
    for i, o in enumerate(ops):
        r.append(f"                    {i} => {{ {o['r']} }}")
    r += ["                    _ => unreachable!(),", "                } });",
          "                if aborted { c.abort(op, k); total += 1; continue; }", "                hash_world();", cfg.get("hash", ("", ""))[1],
          "                c.line(op, k, ret);", "                total += 1;", "            }", "        }", "        assert_eq!(c.done(), total);", "    }", "}", ""]
    (ROOT / "rust/melee_runtime/tests" / f"tu_{stem}.rs").write_text("\n".join(r), encoding="utf-8", newline="\n")
    print(f"mk_lb: {stem}: {nops} ops")
    return nops


if __name__ == "__main__":
    for s in sys.argv[1:]:
        emit(s)
