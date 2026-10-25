"""texpdag.c spec: TEV expression DAGs (an acyclic pool of TExp nodes whose args point at later nodes), dependency lists, the register allocator
and the scheduler recursion, and the simplification passes. texp.c callees (type query, ref / unref) are traced stubs; the Rust ports take
the host (the generated hooks) as their first parameter."""
from lbspecs import add

POOLS = [("HSD_TExp", "TE", 6), ("HSD_TExpDag", "DG", 6)]
ONLY = ["assign_reg", "order_dag", "CalcDistance", "SimplifySrc", "SimplifyThis", "SimplifyByMerge", "HSD_TExpSimplify"]
C_DECL = r'''
static HSD_TExp* TEP[6];
static int DIST[6];
static int ORD[8];
static int MINO[8];
static int MINV;
static u32 DEP[8];
static u32 FDEP[8];
static u8 pick_type(int q) { u8 t; if (q < 5 && (fv() & 1)) return 1; t = (u8) (fv() % 6); return (q == 5 && t == 1) ? 0 : t; }
'''
RS_EXTRA = r'''
static mut TEP: [*mut HSD_TExp; 6] = [null_mut(); 6];
static mut DIST: [i32; 6] = [0; 6];
static mut ORD: [i32; 8] = [0; 8];
static mut MINO: [i32; 8] = [0; 8];
static mut MINV: i32 = 0;
static mut DEP: [u32; 8] = [0; 8];
static mut FDEP: [u32; 8] = [0; 8];
fn pick_type(q: usize) -> u8 { if q < 5 && fv() & 1 != 0 { return 1; } let t = (fv() % 6) as u8; if q == 5 && t == 1 { 0 } else { t } }
'''
_SETUP_C = r'''{ int q, i; LG = 0;
 for (q = 0; q < 6; q++) { TEP[q] = &TE[q]; DIST[q] = (int) (fv() % 4); ORD[q] = (int) (fv() % 6); MINO[q] = 0; DEP[q] = fv() & 0x3F; FDEP[q] = fv() & 0x3F; DG[q].tev = &TE[q].tev; DG[q].idx = (u8) q; DG[q].nb_dep = (u8) (fv() % 3); DG[q].nb_ref = (u8) (fv() % 3); DG[q].dist = 0;
   for (i = 0; i < 8; i++) DG[q].depend[i] = &DG[fv() % 6];
   TE[q].tev.c_ref = (int) (fv() % 3); TE[q].tev.a_ref = (int) (fv() % 3); TE[q].tev.c_dst = (u8) (fv() % 4); TE[q].tev.a_dst = (u8) (fv() % 4); TE[q].tev.tex = 0; TE[q].tev.chan = (fv() & 1) ? 0xFF : (u8) (fv() % 3);
   TE[q].tev.tex_swap = (fv() & 1) ? 0xFF : (u8) (fv() % 2); TE[q].tev.ras_swap = (fv() & 1) ? 0xFF : (u8) (fv() % 2); TE[q].tev.kcsel = (fv() & 1) ? 0xFF : (u8) (fv() % 4); TE[q].tev.kasel = (fv() & 1) ? 0xFF : (u8) (fv() % 4);
   TE[q].tev.c_op = (fv() % 5 == 0) ? 0xFF : (u8) (fv() % 2); TE[q].tev.a_op = (fv() % 5 == 0) ? 0xFF : (u8) (fv() % 2); TE[q].tev.c_bias = (u8) (fv() % 3); TE[q].tev.c_scale = (u8) (fv() % 3); TE[q].tev.a_bias = (u8) (fv() % 3); TE[q].tev.a_scale = (u8) (fv() % 3);
   TE[q].tev.c_clamp = (u8) (fv() & 1); TE[q].tev.a_clamp = (u8) (fv() & 1);
   for (i = 0; i < 4; i++) {
     TE[q].tev.c_in[i].type = pick_type(q); TE[q].tev.c_in[i].sel = (u8) (fv() % 3); TE[q].tev.c_in[i].exp = &TE[q < 5 ? q + 1 + fv() % (5 - q) : q];
     TE[q].tev.a_in[i].type = pick_type(q); TE[q].tev.a_in[i].sel = (u8) (fv() % 3); TE[q].tev.a_in[i].exp = &TE[q < 5 ? q + 1 + fv() % (5 - q) : q]; } }
 MINV = 1000; }'''.replace("\n", " ")
_SETUP_R = r'''{
 for q in 0..6usize { TEP[q] = &raw mut TE[q]; DIST[q] = (fv() % 4) as i32; ORD[q] = (fv() % 6) as i32; MINO[q] = 0; DEP[q] = fv() & 0x3F; FDEP[q] = fv() & 0x3F; DG[q].tev = (&raw mut TE[q]) as *mut _HSD_TETev; DG[q].idx = q as u8; DG[q].nb_dep = (fv() % 3) as u8; DG[q].nb_ref = (fv() % 3) as u8; DG[q].dist = 0;
   for i in 0..8usize { DG[q].depend[i] = &raw mut DG[(fv() % 6) as usize]; }
   TE[q].tev.c_ref = (fv() % 3) as i32; TE[q].tev.a_ref = (fv() % 3) as i32; TE[q].tev.c_dst = (fv() % 4) as u8; TE[q].tev.a_dst = (fv() % 4) as u8; TE[q].tev.tex = null_mut(); TE[q].tev.chan = if fv() & 1 != 0 { 0xFF } else { (fv() % 3) as u8 };
   TE[q].tev.tex_swap = if fv() & 1 != 0 { 0xFF } else { (fv() % 2) as u8 }; TE[q].tev.ras_swap = if fv() & 1 != 0 { 0xFF } else { (fv() % 2) as u8 }; TE[q].tev.kcsel = if fv() & 1 != 0 { 0xFF } else { (fv() % 4) as u8 }; TE[q].tev.kasel = if fv() & 1 != 0 { 0xFF } else { (fv() % 4) as u8 };
   TE[q].tev.c_op = if fv() % 5 == 0 { 0xFF } else { (fv() % 2) as u8 }; TE[q].tev.a_op = if fv() % 5 == 0 { 0xFF } else { (fv() % 2) as u8 }; TE[q].tev.c_bias = (fv() % 3) as u8; TE[q].tev.c_scale = (fv() % 3) as u8; TE[q].tev.a_bias = (fv() % 3) as u8; TE[q].tev.a_scale = (fv() % 3) as u8;
   TE[q].tev.c_clamp = (fv() & 1) as u8; TE[q].tev.a_clamp = (fv() & 1) as u8;
   for i in 0..4usize {
     TE[q].tev.c_in[i].r#type = pick_type(q); TE[q].tev.c_in[i].sel = (fv() % 3) as u8; TE[q].tev.c_in[i].exp = { let k = if q < 5 { q + 1 + (fv() % (5 - q) as u32) as usize } else { q }; &raw mut TE[k] };
     TE[q].tev.a_in[i].r#type = pick_type(q); TE[q].tev.a_in[i].sel = (fv() % 3) as u8; TE[q].tev.a_in[i].exp = { let k = if q < 5 { q + 1 + (fv() % (5 - q) as u32) as usize } else { q }; &raw mut TE[k] }; } }
 MINV = 1000; }'''.replace("\n", " ")
add("texpdag", "src/sysdolphin/baselib/texpdag.c", dict(
    pub_only=True, pools=POOLS, only=ONLY, setup=(_SETUP_C, _SETUP_R), ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA,
    pregs=("preg(TEP, sizeof TEP); preg(DIST, sizeof DIST); preg(ORD, sizeof ORD); preg(MINO, sizeof MINO); preg(&MINV, sizeof MINV); preg(DEP, sizeof DEP); preg(FDEP, sizeof FDEP);",
           "preg(&raw const TEP, 48); preg(&raw const DIST, 24); preg(&raw const ORD, 32); preg(&raw const MINO, 32); preg(&raw const MINV, 4); preg(&raw const DEP, 32); preg(&raw const FDEP, 32); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    hash=("{ int i; for (i = 0; i < 6; i++) { HM((u32) DIST[i]); } for (i = 0; i < 8; i++) { HM((u32) ORD[i]); HM((u32) MINO[i]); } HM((u32) MINV); }",
          "{ for i in 0..6usize { hm(DIST[i] as u32); } for i in 0..8usize { hm(ORD[i] as u32); hm(MINO[i] as u32); } hm(MINV as u32); }"),
    ov={"assign_reg": {0: ("(int) (1 + fv() % 6)", "(1 + fv() % 6) as i32"), 1: ("(u32*) DEP", "(&raw mut DEP) as *mut u32"), 2: ("DG", "(&raw mut DG) as *mut HSD_TExpDag"), 3: ("ORD", "(&raw mut ORD) as *mut i32")},
        "order_dag": {0: ("(int) (1 + fv() % 5)", "(1 + fv() % 5) as i32"), 1: ("DEP", "(&raw mut DEP) as *mut u32"), 2: ("FDEP", "(&raw mut FDEP) as *mut u32"), 3: ("DG", "(&raw mut DG) as *mut HSD_TExpDag"),
                      4: ("(int) 0", "0"), 5: ("(int) (fv() % 5)", "(fv() % 5) as i32"), 8: ("ORD", "(&raw mut ORD) as *mut i32"), 9: ("&MINV", "&raw mut MINV"), 10: ("MINO", "(&raw mut MINO) as *mut i32")},
        "CalcDistance": {0: ("TEP", "(&raw mut TEP) as *mut *mut HSD_TExp"), 1: ("DIST", "(&raw mut DIST) as *mut i32"), 2: ("&TE[fv() % 6]", "&raw mut TE[(fv() % 6) as usize]"), 3: ("(int) (1 + fv() % 6)", "(1 + fv() % 6) as i32"), 4: ("(int) (fv() % 3)", "(fv() % 3) as i32")},
        "SimplifySrc": {0: ("&TE[fv() % 5]", "&raw mut TE[(fv() % 5) as usize]")}, "SimplifyThis": {0: ("&TE[fv() % 5]", "&raw mut TE[(fv() % 5) as usize]")},
        "SimplifyByMerge": {0: ("&TE[fv() % 5]", "&raw mut TE[(fv() % 5) as usize]")}, "HSD_TExpSimplify": {0: ("&TE[fv() % 5]", "&raw mut TE[(fv() % 5) as usize]")}},
    call={"assign_reg": ("assign_reg(a0, a1, a2, a3)", "m::assign_reg(&mut m::Ext, a0, a1, a2, a3)"),
          "order_dag": ("order_dag(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10)", "m::order_dag(&mut m::Ext, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10)"),
          "CalcDistance": ("CalcDistance(a0, a1, a2, a3, a4)", "m::CalcDistance(a0, a1, a2, a3, a4)"),
          "SimplifySrc": ("SimplifySrc(a0)", "m::SimplifySrc(&mut m::Ext, a0)"), "SimplifyThis": ("SimplifyThis(a0)", "m::SimplifyThis(&mut m::Ext, a0)"),
          "SimplifyByMerge": ("SimplifyByMerge(a0)", "m::SimplifyByMerge(&mut m::Ext, a0)"), "HSD_TExpSimplify": ("HSD_TExpSimplify(a0)", "m::HSD_TExpSimplify(&mut m::Ext, a0)")}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=[], own_hash=[], own_only=ONLY,
    skip_stubs=["memcpy", "memset"],
    patch=[('#include "texpdag.h"\n', '#include "texpdag.h"\nextern int printf(const char*, ...);\nstatic int LG; void __assert(const char*, unsigned int, const char*);\nstatic int LGUARD(int w) { (void) w; if (++LG > 100) { __assert("texpdag.c", 0, "simplify loop does not converge"); } return 0; }\n')],
    sub=[(r"\} while \(changed != 0\);", "} while (changed != 0 && !LGUARD(1));"), (r"\} while \(merged != 0\);", "} while (merged != 0 && !LGUARD(2));")],
    ret={"HSD_TExpGetType": ("(u32) ((fv() % 3 == 0) ? 1 : (fv() % 4))", "if fv() % 3 == 0 { 1 } else { fv() % 4 }")})
