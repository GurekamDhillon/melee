#!/usr/bin/env python3
"""State oracle for melee_world::tu::{mplib, mpcoll, mpisland} (the mechanical ports of src/melee/mp/*.c).

    python rust/melee_world/gen/mtu/mk_mp.py gen|run|all          (from the worktree root)

The three game TUs are compiled as ONE C translation unit (their statics shared, exactly as the
ports share them through `pub static mut`). gen_tu.py (spec "mp" of specs_mp.py) writes the recording
stubs for every callee outside them and the field-by-field fill / hash of the state records; this
file writes the driver around it, both halves from the one table below so they cannot drift:
  gen/mtu/tu/mp.c            C driver (the game's C, compiled natively with the SHIPPING float contraction)
  tests/tu_mp.rs            Rust driver, draw for draw
  tests/data/mtu_mp.txt     C output (op case ret state-hash trace-hash), computed from C source only

The state a case starts from is generated: a random but well-formed map (lines grouped floor / ceiling /
walls / dynamic, chained prev/next ids, 3 joints, vertices contiguous per joint) loaded through the
game's own mpLibLoad (which also builds the islands), a CollData placed in that map, and joint
callbacks. The hash after the call covers the CollData, every map record, the joint / vertex / line /
island tables, and the statics of the three files.
"""
import os, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
os.chdir(ROOT)
CASES = 600

# ----------------------------------------------------------------------------------- the op table
# (name, C statements, Rust statements, ret kind) ; ret kinds: v void, i int, p pointer, f float bits
CD = "&CD"
OPS = []


def op(c, r, ret="v", name=None):
    OPS.append((name or c.split("(")[0].split()[-1], c, r, ret))


# CollData-only functions: (name, ret kind)
for fn, ret in [("mpCollPrev", "v"), ("mpColl_80042384", "v"), ("mpColl_80044E10_RightWall", "i"),
                ("mpColl_80045B74_LeftWall", "i"), ("mpColl_800478F4", "i"),
                ("mpColl_800488F4", "i"), ("mpColl_80048AB0_RightWall", "i"), ("mpColl_80049778_LeftWall", "i"),
                ("mpColl_8004B4B0", "i")]:
    cr = "(u32) " if ret == "i" else ""
    op(f"{'ret = (u32) ' if ret == 'i' else ''}{fn}(&CD);",
       f"{'ret = ' if ret == 'i' else ''}pc::{fn}(&raw mut CD){' as u32' if ret == 'i' else ''};", ret, fn)
op("{ if (CD.ecb_source.kind != ECBSource_Fixed) { CD.ecb_source.kind = ECBSource_Fixed; CD.ecb_source.up = 12.0f; CD.ecb_source.down = 1.0f; CD.ecb_source.front = 4.0f; CD.ecb_source.back = 3.0f; CD.ecb_source.angle = 0.5f; } mpColl_LoadECB_Fixed(&CD); }",
   "{ if CD.ecb_source.kind != ECBSource_Fixed as _ { CD.ecb_source.kind = ECBSource_Fixed as _; CD.ecb_source.anon_4.anon_0_0.up = 12.0; CD.ecb_source.anon_4.anon_0_0.down = 1.0; CD.ecb_source.anon_4.anon_0_0.front = 4.0; CD.ecb_source.anon_4.anon_0_0.back = 3.0; CD.ecb_source.anon_4.anon_0_0.angle = 0.5; } pc::mpColl_LoadECB_Fixed(&raw mut CD); }", "v", "mpColl_LoadECB_Fixed")
op("mpCollCheckBounding(&CD, fv());", "pc::mpCollCheckBounding(&raw mut CD, fv());")
op("{ int a = (int) (fv() & 1); int b = (int) (fv() & 1); mpCollEnd(&CD, a, b); }",
   "{ let a = (fv() & 1) as i32; let b = (fv() & 1) as i32; pc::mpCollEnd(&raw mut CD, a, b); }", "v", "mpCollEnd")
op("mpCollInterpolateECB(&CD, (float) (rnd() % 11) / 10.0f);",
   "pc::mpCollInterpolateECB(&raw mut CD, (rnd() % 11) as f32 / 10.0);")
op("{ u32 f = fv(); ret = (u32) mpColl_80043754(cbPipe, &CD, f); }",
   "{ let f = fv(); ret = pc::mpColl_80043754(Some(cb_pipe), &raw mut CD, f) as u32; }", "i", "mpColl_80043754")
op("{ int lr = (int) (fv() & 1); ret = (u32) mpColl_80044628_Floor(&CD, ((rnd() & 1) ? cbFloor : 0), &GO, lr); }",
   "{ let lr = (fv() & 1) as i32; ret = pc::mpColl_80044628_Floor(&raw mut CD, if rnd() & 1 != 0 { Some(cb_floor) } else { None }, &raw mut GO, lr) as u32; }", "i", "mpColl_80044628_Floor")
op("{ int lr = (int) (fv() & 1); ret = (u32) mpColl_80044AD8_Ceiling(&CD, lr); }",
   "{ let lr = (fv() & 1) as i32; ret = pc::mpColl_80044AD8_Ceiling(&raw mut CD, lr) as u32; }", "i", "mpColl_80044AD8_Ceiling")
op("{ u32 f = fv(); ret = (u32) mpColl_80046904(&CD, f); }",
   "{ let f = fv(); ret = pc::mpColl_80046904(&raw mut CD, f) as u32; }", "i", "mpColl_80046904")
op("mpColl_LoadECB_JObj(&CD, fv());", "pc::mpColl_LoadECB_JObj(&raw mut CD, fv());")
op("mpColl_SetECBSource_JObj(&CD, &GO, JP(0), JP(1), JP(2), JP(3), JP(4), JP(5), JP(6), rg());",
   "pc::mpColl_SetECBSource_JObj(&raw mut CD, &raw mut GO, jp(0), jp(1), jp(2), jp(3), jp(4), jp(5), jp(6), rg());", "v", "mpColl_SetECBSource_JObj")
op("{ float a = rw(); float b = rw(); float c = rw(); mpColl_SetLedgeSnap(&CD, a, b, c); }",
   "{ let a = rw(); let b = rw(); let c = rw(); pc::mpColl_SetLedgeSnap(&raw mut CD, a, b, c); }", "v", "mpColl_SetLedgeSnap")
for fn in ["mpColl_80044164", "mpColl_800443C4", "mpColl_80043BBC", "mpColl_80043E90"]:
    op(f"ret = (u32) {fn}(&CD, &OUTI[0]); HM((u32) OUTI[0]);", f"ret = pc::{fn}(&raw mut CD, &raw mut OUTI[0]) as u32; hm(OUTI[0] as u32);", "i", fn)
op("{ int b = (int) (fv() & 1); ret = (u32) mpColl_80044838_Floor(&CD, b); }",
   "{ let b = (fv() & 1) as i32; ret = pc::mpColl_80044838_Floor(&raw mut CD, b) as u32; }", "i", "mpColl_80044838_Floor")
op("{ int id = RLID(); ret = (u32) mpColl_8004A908_Floor(&CD, id); }",
   "{ let id = rlid(); ret = pc::mpColl_8004A908_Floor(&raw mut CD, id) as u32; }", "i", "mpColl_8004A908_Floor")
op("ret = (u32) mpColl_8004ACE4(&CD, (int) fv());", "ret = pc::mpColl_8004ACE4(&raw mut CD, fv() as i32) as u32;", "i", "mpColl_8004ACE4")
# a few whole-pipeline sequences (the collision loop of a moving fighter)
op("{ int s; u32 f = fv(); mpCollPrev(&CD); for (s = 0; s < 4; s++) { ret = ret * 3 + (u32) mpColl_80043754(cbPipe, &CD, f); "
   "CD.cur_pos.x += (float) ((int) (rnd() % 9) - 4); CD.cur_pos.y += (float) ((int) (rnd() % 9) - 4); mpCollPrev(&CD); } mpCollEnd(&CD, (int) (fv() & 1), (int) (fv() & 1)); }",
   "{ let f = fv(); pc::mpCollPrev(&raw mut CD); for _ in 0..4 { ret = ret.wrapping_mul(3).wrapping_add(pc::mpColl_80043754(Some(cb_pipe), &raw mut CD, f) as u32); "
   "CD.cur_pos.x += ((rnd() % 9) as i32 - 4) as f32; CD.cur_pos.y += ((rnd() % 9) as i32 - 4) as f32; pc::mpCollPrev(&raw mut CD); } let a = (fv() & 1) as i32; let b = (fv() & 1) as i32; pc::mpCollEnd(&raw mut CD, a, b); }", "i", "pipeline")

# mplib
op("{ float l = rw(); float b = rw(); float r = rw(); float t = rw(); mpBoundingCheck(l, b, r, t); }",
   "{ let l = rw(); let b = rw(); let r = rw(); let t = rw(); pl::mpBoundingCheck(l, b, r, t); }", "v", "mpBoundingCheck")
op("{ float l = RC(); float b = RC(); float r = RC(); float t = RC(); mpBoundingCheck(l, b, r, t); }",
   "{ let l = rc(); let b = rc(); let r = rc(); let t = rc(); pl::mpBoundingCheck(l, b, r, t); }", "v", "mpBoundingCheck_world")
op("ret = (u32) mpCheckedBounding();", "ret = pl::mpCheckedBounding() as u32;", "i", "mpCheckedBounding")
op("{ didCheckBounding = (int) (rnd() & 1); mpUncheckBounding(); }", "{ pl::didCheckBounding = (rnd() & 1) as i32; pl::mpUncheckBounding(); }", "v", "mpUncheckBounding")
op("ret = PI(mpGetGroundCollVtx());", "ret = pi(pl::mpGetGroundCollVtx());", "p", "mpGetGroundCollVtx")
op("{ int i = (int) (rnd() % 7); if (i < NLN_) LINES[i].v1_idx = LINES[i].v0_idx; mpPruneEmptyLines(&M); }",
   "{ let i = (rnd() % 7) as usize; if (i as i32) < NLN_ { LINES[i].v1_idx = LINES[i].v0_idx; } pl::mpPruneEmptyLines(&raw mut M); }", "v", "mpPruneEmptyLines")
op("{ int j = (int) (rnd() % 3); mpJointUpdateDynamics(j); }", "{ let j = (rnd() % 3) as i32; pl::mpJointUpdateDynamics(j); }", "v", "mpJointUpdateDynamics")
op("ret = PI(mpLib_8004D164());", "ret = pi(pl::mpLib_8004D164());", "p", "mpLib_8004D164")
op("mpLib_80058614_Floor();", "pl::mpLib_80058614_Floor();")
op("mpLib_800587FC(&GO);", "pl::mpLib_800587FC(&raw mut GO);")
for fn in ["mpLib_80056A54", "mpLib_80056AC4"]:
    op(f"{{ int a = (int) (rnd() % 20); stage_info.grkind = (int) (rnd() % 80); ret = (u32) {fn}(a, &OUTI[0]); HM((u32) OUTI[0]); }}",
       f"{{ let a = (rnd() % 20) as i32; pl::ext::stage_info.grkind = (rnd() % 80) as _; ret = pl::{fn}(a, &raw mut OUTI[0]) as u32; hm(OUTI[0] as u32); }}", "i", fn)
op("{ int a = (int) (rnd() % 20); int* p; stage_info.grkind = (int) (rnd() % 80); p = mpLib_80056A8C(a, &OUTI[0]); HM((u32) OUTI[0]); HM((u32) p[0]); HM((u32) p[1]); HM((u32) p[2]); HM((u32) p[3]); }",
   "{ let a = (rnd() % 20) as i32; pl::ext::stage_info.grkind = (rnd() % 80) as _; let p = pl::mpLib_80056A8C(a, &raw mut OUTI[0]); hm(OUTI[0] as u32); hm(*p as u32); hm(*p.add(1) as u32); hm(*p.add(2) as u32); hm(*p.add(3) as u32); }", "v", "mpLib_80056A8C")
op("{ float a = RI(), b = RI(), c = RI(), d = RI(), e = RI(), f = RI(), g = RI(), h = RI(); ret = (u32) mpLineIntersection(a, b, c, d, e, f, g, h, &OUTF[0], &OUTF[1]); HM(B(OUTF[0])); HM(B(OUTF[1])); }",
   "{ let (a, b, c, d, e, f, g, h) = (ri(), ri(), ri(), ri(), ri(), ri(), ri(), ri()); ret = pl::mpLineIntersection(a, b, c, d, e, f, g, h, &raw mut OUTF[0], &raw mut OUTF[1]) as u32; hm(OUTF[0].to_bits()); hm(OUTF[1].to_bits()); }", "i", "mpLineIntersection")
op("{ float a = RI(), b = RI(), c = RI(), d = RI(), e = RI(), f = RI(), g = RI(); ret = (u32) mpLineIntersectionH(&OUTF[0], &OUTF[1], a, b, c, d, e, f, g); HM(B(OUTF[0])); HM(B(OUTF[1])); }",
   "{ let (a, b, c, d, e, f, g) = (ri(), ri(), ri(), ri(), ri(), ri(), ri()); ret = pl::mpLineIntersectionH(&raw mut OUTF[0], &raw mut OUTF[1], a, b, c, d, e, f, g) as u32; hm(OUTF[0].to_bits()); hm(OUTF[1].to_bits()); }", "i", "mpLineIntersectionH")
op("{ float a = RI(), b = RI(), c = RI(), d = RI(), e = RI(), f = RI(), g = RI(); ret = (u32) mpLineIntersectionV(&OUTF[0], &OUTF[1], a, b, c, d, e, f, g); HM(B(OUTF[0])); HM(B(OUTF[1])); }",
   "{ let (a, b, c, d, e, f, g) = (ri(), ri(), ri(), ri(), ri(), ri(), ri()); ret = pl::mpLineIntersectionV(&raw mut OUTF[0], &raw mut OUTF[1], a, b, c, d, e, f, g) as u32; hm(OUTF[0].to_bits()); hm(OUTF[1].to_bits()); }", "i", "mpLineIntersectionV")
# mpisland
op("mpIsland_8005A728();", "pi_::mpIsland_8005A728();")
op("{ int s = (int) (rnd() % 30); int c = (int) (rnd() % 12); int b = (int) (fv() & 1); mpIsland_8005AE1C(&mpIsland_80458E88.next, &mpIsland_80458E88.x18, s, c, b); }",
   "{ let s = (rnd() % 30) as i32; let c = (rnd() % 12) as i32; let b = (fv() & 1) as i32; pi_::mpIsland_8005AE1C(&raw mut pi_::mpIsland_80458E88.next, &raw mut pi_::mpIsland_80458E88.x18, s, c, b); }", "v", "mpIsland_8005AE1C")
op("{ int j = (int) (rnd() % 3); int k = (int) (1 + (rnd() & 1)); int s = (int) (rnd() % 30); int c = (int) (rnd() % 12); int b = (int) (fv() & 1); "
   "mpIsland_8005B004((k == 1) ? &mpIsland_80458E88.x10 : &mpIsland_80458E88.x14, &mpIsland_80458E88.x20, j, k, s, c, b); }",
   "{ let j = (rnd() % 3) as i32; let k = (1 + (rnd() & 1)) as i32; let s = (rnd() % 30) as i32; let c = (rnd() % 12) as i32; let b = (fv() & 1) as i32; "
   "pi_::mpIsland_8005B004(if k == 1 { &raw mut pi_::mpIsland_80458E88.x10 } else { &raw mut pi_::mpIsland_80458E88.x14 }, &raw mut pi_::mpIsland_80458E88.x20, j, k, s, c, b); }", "v", "mpIsland_8005B004")
op("{ int j = (int) (rnd() % 3); int s = (int) (rnd() % 30); int c = (int) (rnd() % 12); int b = (int) (fv() & 1); mpIsland_8005B334(j, s, c, b); }",
   "{ let j = (rnd() % 3) as i32; let s = (rnd() % 30) as i32; let c = (rnd() % 12) as i32; let b = (fv() & 1) as i32; pi_::mpIsland_8005B334(j, s, c, b); }", "v", "mpIsland_8005B334")


# --- helpers of mplib the targets call (line queries): same oracle, finer localisation of a divergence
for fn in ["mpLineGetNext", "mpLineGetPrev", "mpLineNextNonFloor", "mpLinePrevNonFloor", "mpLinePrevNonCeiling", "mpLineNextNonCeiling",
           "mpLineNextNonLeftWall", "mpLinePrevNonLeftWall", "mpLinePrevNonRightWall", "mpLineNextNonRightWall",
           "mpLib_80053394_Floor", "mpLib_80053448_Floor", "mpLib_800534FC_Floor", "mpLib_800536CC_Floor",
           "mpLib_8005389C_Ceiling", "mpLib_80053950_Ceiling", "mpLib_80053A04_Ceiling", "mpLib_80053BD4_Ceiling",
           "mpLineGetKind", "mpLineGetFlags", "mpLib_80054ED8", "mpJointFromLine"]:
    op(f"{{ int id = RVID(); ret = (u32) {fn}(id); }}", f"{{ let id = rvid(); ret = pl::{fn}(id) as u32; }}", "i", fn)
for fn in ["mpLib_80053DA4_Floor", "mpLib_80053ECC_Floor", "mpFloorGetRight", "mpFloorGetLeft", "mpCeilingGetRight", "mpCeilingGetLeft",
           "mpLeftWallGetTop", "mpLeftWallGetBottom", "mpRightWallGetTop", "mpRightWallGetBottom", "mpLineGetV1Pos", "mpLineGetV0Pos"]:
    op(f"{{ int id = RVID(); {fn}(id, &V3[0]); }}", f"{{ let id = rvid(); pl::{fn}(id, &raw mut V3[0]); }}", "v", fn)
for fn in ["mpLib_8004DD90_Floor", "mpLib_8004E090_Ceiling", "mpLib_8004E398_LeftWall", "mpLib_8004E684_RightWall"]:
    op(f"{{ int id = RVID(); V3[0].x = RC(); V3[0].y = RC(); V3[0].z = 0.0f; ret = (u32) {fn}(id, &V3[0], &OUTF[0], &OUTU[0], &V3[1]); }}",
       f"{{ let id = rvid(); V3[0].x = rc(); V3[0].y = rc(); V3[0].z = 0.0; ret = pl::{fn}(id, &raw mut V3[0], &raw mut OUTF[0], &raw mut OUTU[0], &raw mut V3[1]) as u32; }}", "i", fn)
op("{ int a = RVID(), b = RVID(); ret = (u32) mpLinesConnected(a, b); }", "{ let a = rvid(); let b = rvid(); ret = pl::mpLinesConnected(a, b) as u32; }", "i", "mpLinesConnected")
op("{ int id = RVID(); V3[0].x = RC(); V3[0].y = RC(); V3[0].z = 0.0f; ret = (u32) mpGetSpeed(id, &V3[0], &V3[1]); }",
   "{ let id = rvid(); V3[0].x = rc(); V3[0].y = rc(); V3[0].z = 0.0; ret = pl::mpGetSpeed(id, &raw mut V3[0], &raw mut V3[1]) as u32; }", "i", "mpGetSpeed")
op("{ int id = RVID(); ret = PI(mpLineGetNormal(id, &V3[0])); }", "{ let id = rvid(); ret = pi(pl::mpLineGetNormal(id, &raw mut V3[0])); }", "p", "mpLineGetNormal")
op("{ float a = RC(), b = RC(), c = RC(), d = RC(); int s = (int) (fv() & 3); ret = (u32) mpCheckFloor(a, b, c, d, 0.0f, &V3[0], &OUTI[0], &OUTU[0], &V3[1], (s == 0) ? RLID() : -1, (int) (rnd() % 4) - 1, (int) (rnd() % 4) - 1, 0, 0); }",
   "{ let a = rc(); let b = rc(); let c = rc(); let d = rc(); let s = fv() & 3; let sk = if s == 0 { rlid() } else { -1 }; let js = (rnd() % 4) as i32 - 1; let jo = (rnd() % 4) as i32 - 1; ret = pl::mpCheckFloor(a, b, c, d, 0.0, &raw mut V3[0], &raw mut OUTI[0], &raw mut OUTU[0], &raw mut V3[1], sk, js, jo, None, null_mut()) as u32; }", "i", "mpCheckFloor")
for fn in ["mpCheckCeiling", "mpCheckLeftWall", "mpCheckRightWall"]:
    op(f"{{ float a = RC(), b = RC(), c = RC(), d = RC(); int js = (int) (rnd() % 4) - 1; int jo = (int) (rnd() % 4) - 1; ret = (u32) {fn}(a, b, c, d, &V3[0], &OUTI[0], &OUTU[0], &V3[1], js, jo); }}",
       f"{{ let a = rc(); let b = rc(); let c = rc(); let d = rc(); let js = (rnd() % 4) as i32 - 1; let jo = (rnd() % 4) as i32 - 1; ret = pl::{fn}(a, b, c, d, &raw mut V3[0], &raw mut OUTI[0], &raw mut OUTU[0], &raw mut V3[1], js, jo) as u32; }}", "i", fn)

# ----------------------------------------------------------------------------------- C driver
C_HEAD = r'''/* TU: src/melee/mp/{mplib,mpcoll,mpisland}.c as one translation unit */
/* C oracle for the Rust ports of these TUs: the three files included verbatim; every callee out of
 * them is a recording stub. Written by gen/mtu/mk_mp.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#define __assert mtu_unused_assert
#include "mtu_common.h"
#undef __assert
extern void exit(int);
extern char* getenv(const char*);
extern void setbuf(void*, char*);
extern void* __acrt_iob_func(unsigned);
extern void RaiseException(unsigned long, unsigned long, unsigned long, const void*);
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; RaiseException(0xE0A55E87ul, 1, 0, 0); }
void* memset(void* d, int c, unsigned long n) { unsigned char* p = (unsigned char*) d; while (n--) *p++ = (unsigned char) c; return d; }
void memzero(void* d, int n) { unsigned char* p = (unsigned char*) d; while (n-- > 0) *p++ = 0; }
int __HI(double x) { union { double d; u64_t_ u; } c; c.d = x; return (int) (c.u >> 32); }
int __LO(double x) { union { double d; u64_t_ u; } c; c.d = x; return (int) (u32) c.u; }
#define NLN 24
#define NVX 64
#define NJ 3
static MapCollData M;
static Vec2 VERTS[NVX];
static MapLine LINES[NLN];
static MapJoint JOINTS[NJ];
static CollVtx VTXA[2048];
static CollLine LINEA[1536];
static CollJoint JOINTA[8];
static mp_UnkStruct0 ISL[48];
static int ISLN;
static CollData CD;
static HSD_GObj GO;
static HSD_JObj JO[8];
static int OUTI[2];
static float OUTF[2];
static u32 OUTU[2];
static Vec3 V3[3];
int mtu_dbgp;
static int ASSN[256];
extern int fprintf(void*, const char*, ...);
static void fprintf_stub(int op) { fprintf(__acrt_iob_func(2), "ASSERTS %d %d%c", op, ASSN[op], 10); }
static int getenv_only;
static int NLN_;
static int NVX_;

static float RNN(void) { static const float k[4] = { 1.0f, 13.0f, 100.0f, 1000.0f }; float a = (float) ((int) (rnd() % 41) - 20) / 20.0f; return a * k[rnd() % 4]; }
#include "mp_gen.h"

static void jcbA(void* ud, int jid, CollData* c, int x50, mpLib_GroundEnum k, float dy) { T(0xCA); T(PI(ud)); T((u32) jid); T(PI(c)); T((u32) x50); T((u32) k); T(B(dy)); c->x50 += 1.0f; }
static void jcbB(void* ud, int jid, CollData* c, int x50, mpLib_GroundEnum k, float dy) { T(0xCB); T(PI(ud)); T((u32) jid); T(PI(c)); T((u32) x50); T((u32) k); T(B(dy)); c->x50 -= 0.5f; }
static bool cbPipe(CollData* c, u32 f) { T(0xCC); T(PI(c)); T(f); c->cur_pos.x += 0.5f; return (bool) (fv() & 1); }
static bool cbFloor(Fighter_GObj* g, int i) { T(0xCD); T(PI(g)); T((u32) i); return (bool) (fv() & 1); }
static HSD_JObj* JP(int i) { return (rnd() % 5 == 0) ? 0 : &JO[i]; }
static float RI(void) { if (rnd() % 8 == 0) return rg(); return (float) ((int) (rnd() % 21) - 10); }
static float RC(void) { return (float) ((int) (rnd() % 61) - 30) + ((rnd() & 1) ? 0.5f : 0.0f); }
static int RVID(void) { return (int) (rnd() % (u32) NLN_); }
static int RLID(void) { return (rnd() % 4 == 0) ? -1 : (int) (rnd() % (u32) NLN_); }

static void mkecb(ftECB* e, float w, float h)
{
    e->top.x = 0.0f; e->top.y = h; e->bottom.x = 0.0f; e->bottom.y = 0.0f;
    e->right.x = w; e->right.y = h * 0.5f; e->left.x = -w; e->left.y = h * 0.5f;
}

static void build_world(void)
{
    int g, i, j, k, n[5], st[5], tot = 0, slot = 0, joint_of[NLN], grp_of[NLN], pos_of[NLN];
    float p0x[NLN], p0y[NLN], p1x[NLN], p1y[NLN];
    n[0] = 2 + (int) (rnd() % 4); n[1] = 1 + (int) (rnd() % 3); n[2] = 1 + (int) (rnd() % 2); n[3] = 1 + (int) (rnd() % 2); n[4] = (int) (rnd() % 3);
    for (g = 0; g < 5; g++) { st[g] = tot; tot += n[g]; }
    for (g = 0; g < 5; g++) {
        float cx = (float) ((int) (rnd() % 61) - 30), cy = (float) ((int) (rnd() % 41) - 20);
        for (k = 0; k < n[g]; k++) {
            int len = 4 + (int) (rnd() % 12), d = (int) (rnd() % 9) - 4;
            i = st[g] + k; grp_of[i] = g; pos_of[i] = k;
            p0x[i] = cx; p0y[i] = cy;
            if (g == 0) { cx += (float) len; cy += (float) d; }
            else if (g == 1) { cx -= (float) len; cy += (float) d; }
            else if (g == 2) { cy -= (float) len; cx += (float) d; }
            else if (g == 3) { cy += (float) len; cx += (float) d; }
            else { int dx = (int) (rnd() % 21) - 10, dy = (int) (rnd() % 21) - 10; if (dx == 0 && dy == 0) dx = 1; cx += (float) dx; cy += (float) dy; }
            p1x[i] = cx; p1y[i] = cy;
        }
    }
    for (g = 0; g < 5; g++) for (k = 0; k < n[g]; k++) joint_of[st[g] + k] = (k * 3) / n[g];
    for (j = 0; j < NJ; j++) {
        int first = slot, cnt = 0;
        float lo_x = 1e9f, lo_y = 1e9f, hi_x = -1e9f, hi_y = -1e9f;
        for (g = 0; g < 5; g++) {
            int s = 0, c = 0;
            for (k = 0; k < n[g]; k++) if (joint_of[st[g] + k] == j) { if (c == 0) s = st[g] + k; c++; }
            JOINTS[j].ranges[g].start = (s16) (c ? s : st[g]); JOINTS[j].ranges[g].count = (s16) c;
        }
        for (i = 0; i < tot; i++) if (joint_of[i] == j) {
            LINES[i].v0_idx = (u16) slot; LINES[i].v1_idx = (u16) (slot + 1);
            VERTS[slot].x = p0x[i]; VERTS[slot].y = p0y[i]; VERTS[slot + 1].x = p1x[i]; VERTS[slot + 1].y = p1y[i];
            if (p0x[i] < lo_x) lo_x = p0x[i]; if (p1x[i] < lo_x) lo_x = p1x[i]; if (p0x[i] > hi_x) hi_x = p0x[i]; if (p1x[i] > hi_x) hi_x = p1x[i];
            if (p0y[i] < lo_y) lo_y = p0y[i]; if (p1y[i] < lo_y) lo_y = p1y[i]; if (p0y[i] > hi_y) hi_y = p0y[i]; if (p1y[i] > hi_y) hi_y = p1y[i];
            slot += 2; cnt += 2;
        }
        JOINTS[j].left_bound = lo_x - 2.0f; JOINTS[j].bottom_bound = lo_y - 2.0f; JOINTS[j].right_bound = hi_x + 2.0f; JOINTS[j].top_bound = hi_y + 2.0f;
        JOINTS[j].vtx_start = (s16) first; JOINTS[j].vtx_count = (s16) cnt;
    }
    for (i = 0; i < tot; i++) {
        int r;
        LINES[i].prev_id0 = (s16) (pos_of[i] > 0 ? i - 1 : -1);
        LINES[i].next_id0 = (s16) (pos_of[i] < n[grp_of[i]] - 1 ? i + 1 : -1);
        r = (int) rnd(); if (r % 8 == 0) LINES[i].prev_id0 = (s16) (i > 0 ? (int) (rnd() % (u32) i) : -1);
        r = (int) rnd(); if (r % 8 == 0) LINES[i].next_id0 = (s16) (tot - 1 - i > 0 ? i + 1 + (int) (rnd() % (u32) (tot - 1 - i)) : -1);
        LINES[i].prev_id1 = (s16) ((rnd() % 3 == 0) ? (i > 0 ? (int) (rnd() % (u32) i) : -1) : -1);
        LINES[i].next_id1 = (s16) ((rnd() % 3 == 0) ? (tot - 1 - i > 0 ? i + 1 + (int) (rnd() % (u32) (tot - 1 - i)) : -1) : -1);
        LINES[i].hi_flags = (u16) (((grp_of[i] < 4) ? (1 << grp_of[i]) : (1 << (rnd() % 4))) | ((rnd() & 1) ? 0x100 : 0) | ((rnd() & 1) ? 0x200 : 0) | ((rnd() % 4 == 0) ? 0x400 : 0) | (fv() & 0xF870));
        LINES[i].lo_flags = (u16) (fv() & 0xFFFF);
        if (rnd() % 3 == 0) LINES[i].lo_flags |= 0x400;
    }
    M.verts = VERTS; M.vert_count = slot; M.lines = LINES; M.line_count = tot; M.joints = JOINTS; M.joint_count = NJ;
    for (g = 0; g < 5; g++) { M.ranges[g].start = (s16) st[g]; M.ranges[g].count = (s16) n[g]; }
    NLN_ = tot; NVX_ = slot;
}

static void setup(void)
{
    int i, j;
    float w, h;
    zero(&M, sizeof M); zero(VERTS, sizeof VERTS); zero(LINES, sizeof LINES); zero(JOINTS, sizeof JOINTS);
    zero(VTXA, sizeof VTXA); zero(LINEA, sizeof LINEA); zero(JOINTA, sizeof JOINTA); zero(ISL, sizeof ISL); ISLN = 0;
    zero(&CD, sizeof CD); zero(&GO, sizeof GO); zero(JO, sizeof JO); zero(OUTI, sizeof OUTI); zero(OUTF, sizeof OUTF); zero(OUTU, sizeof OUTU); zero(V3, sizeof V3);
    stage_info.grkind = 0;
    zero(&mpColl_80458810, sizeof mpColl_80458810); mpColl_804D6488 = 0; mpColl_804D648C = 0; mpColl_804D6490_max_x = 0.0f; mpColl_804D6494_line_id = 0; mpColl_804D6498_flags = 0;
    mpColl_IsEcbTiny = 0; mpColl_804D64A0 = 0; mpColl_804D64A4 = 0; mpColl_804D64A8 = 0; mpColl_804D64AC = 0; mpColl_LedgeLevelOK = 0;
    mpLib_804D64CC = 0; mpLib_804D64D0 = 0; mpLib_804D64D4 = 0; mpLib_804D64D8 = 0; mpLib_804D64DC = 0; mpLib_804D64E0 = 0; mpLib_804D64E4 = 0;
    zero(mpLib_80458868, sizeof mpLib_80458868); scriptBatchDepth = 0;
    build_world();
    mpLibLoad(&M);
    for (j = 0; j < NJ; j++) {
        CollJoint* cj = &JOINTA[j];
        if (rnd() % 5 == 0) cj->flags &= ~CollJoint_Enabled;
        if (rnd() % 6 == 0) cj->flags |= CollJoint_Hidden;
        cj->flags |= (fv() & 0x1F00);
        cj->xE = (rnd() & 1);
        cj->x20 = (rnd() & 1) ? &JO[j] : 0;
        cj->cb_0 = (rnd() % 3) ? ((rnd() & 1) ? jcbA : jcbB) : 0; cj->cb_data_0 = (rnd() & 1) ? (Ground*) &GO : 0;
        cj->cb_1 = (rnd() % 3) ? ((rnd() & 1) ? jcbA : jcbB) : 0; cj->cb_data_1 = (rnd() & 1) ? (Ground*) &JO[1] : 0;
    }
    fill_HSD_GObj(&GO);
    for (i = 0; i < 8; i++) fill_HSD_JObj(&JO[i]);
    fill_CollData(&CD);
    CD.x0_gobj = (rnd() & 3) ? &GO : 0;
    CD.cur_pos.x = RC(); CD.cur_pos.y = RC(); CD.cur_pos.z = 0.0f;
    if (rnd() % 4 != 0) { /* near a line end, so the sweeps meet geometry */
        int li = (int) (rnd() % (u32) NLN_), vi = (int) (rnd() & 1);
        float ox = (float) ((int) (rnd() % 9) - 4) * 0.5f, oy = (float) ((int) (rnd() % 9) - 4) * 0.5f;
        Vec2* vp = &VERTS[vi ? LINES[li].v1_idx : LINES[li].v0_idx];
        CD.cur_pos.x = vp->x + ox; CD.cur_pos.y = vp->y + oy;
    }
    CD.prev_pos.x = CD.cur_pos.x + (float) ((int) (rnd() % 7) - 3) * 0.5f; CD.prev_pos.y = CD.cur_pos.y + (float) ((int) (rnd() % 7) - 3) * 0.5f; CD.prev_pos.z = 0.0f;
    CD.last_pos.x = CD.prev_pos.x + (float) ((int) (rnd() % 5) - 2) * 0.5f; CD.last_pos.y = CD.prev_pos.y + (float) ((int) (rnd() % 5) - 2) * 0.5f; CD.last_pos.z = 0.0f;
    CD.x28_vec = CD.prev_pos;
    w = (float) (3 + (int) (rnd() % 3)); h = (float) (8 + (int) (rnd() % 8));
    mkecb(&CD.ecb, w, h); mkecb(&CD.prev_ecb, w + (float) (rnd() % 2), h); mkecb(&CD.desired_ecb, w, h + (float) (rnd() % 3)); mkecb(&CD.x64_ecb, w, h); mkecb(&CD.xE4_ecb, w, h);
    CD.floor_skip = (rnd() % 4 == 0) ? (int) (rnd() % (u32) NLN_) : -1;
    CD.ledge_id_right = (rnd() % 3 == 0) ? (int) (rnd() % (u32) NLN_) : -1;
    CD.ledge_id_left = (rnd() % 3 == 0) ? (int) (rnd() % (u32) NLN_) : -1;
    CD.joint_id_skip = (rnd() % 6 == 0) ? (int) (rnd() % NJ) : -1;
    CD.joint_id_only = (rnd() % 6 == 0) ? (int) (rnd() % NJ) : -1;
    CD.x50 = rg(); CD.lstick_x = rg();
    CD.ledge_snap_x = (float) (2 + (int) (rnd() % 4)); CD.ledge_snap_y = (float) (rnd() % 8); CD.ledge_snap_height = (float) (4 + (int) (rnd() % 8));
    CD.facing_dir = (rnd() & 1) ? 1 : -1;
    CD.env_flags = (s32) rnd(); CD.prev_env_flags = (s32) rnd(); CD.x130_flags = rnd() & 0xFF; CD.x13C = (s32) rnd();
    if (rnd() & 1) {
        CD.ecb_source.kind = ECBSource_Fixed;
        CD.ecb_source.up = (float) (8 + (int) (rnd() % 8)); CD.ecb_source.down = (float) (rnd() % 3); CD.ecb_source.front = (float) (3 + (int) (rnd() % 3)); CD.ecb_source.back = (float) (3 + (int) (rnd() % 3));
        CD.ecb_source.angle = (float) ((int) (rnd() % 13) - 6) * 0.5f;
    } else {
        CD.ecb_source.kind = ECBSource_JObj;
        CD.ecb_source.x108_joint = &JO[0];
        for (i = 0; i < 6; i++) CD.ecb_source.x10C_joint[i] = &JO[i + 1];
    }
    CD.ecb_source.x124 = (float) ((int) (rnd() % 41) - 20) / 20.0f; CD.ecb_source.x128 = (float) (rnd() % 12); CD.ecb_source.x12C = (float) (rnd() % 12);
    CD.floor.index = RLID(); CD.floor.flags = fv(); CD.ceiling.index = RLID(); CD.ceiling.flags = fv();
    CD.left_facing_wall.index = RLID(); CD.left_facing_wall.flags = fv(); CD.right_facing_wall.index = RLID(); CD.right_facing_wall.flags = fv();
    CD.contact.x = rg(); CD.contact.y = rg(); CD.contact.z = 0.0f;
    if (rnd() % 2 == 0) { int li = (int) (rnd() % (u32) NLN_); if (LINES[li].hi_flags & 1) CD.floor.index = li; else if (LINES[li].hi_flags & 2) CD.ceiling.index = li; else if (LINES[li].hi_flags & 4) CD.right_facing_wall.index = li; else CD.left_facing_wall.index = li; }
}

static void hash_all(void)
{
    int i;
    hash_CollData(&CD);
    HM((u32) CD.ecb_source.kind);
    if (CD.ecb_source.kind == ECBSource_Fixed) {
        HM(B(CD.ecb_source.up)); HM(B(CD.ecb_source.down)); HM(B(CD.ecb_source.front)); HM(B(CD.ecb_source.back)); HM(B(CD.ecb_source.angle));
    } else {
        HM(PI(CD.ecb_source.x108_joint)); for (i = 0; i < 6; i++) HM(PI(CD.ecb_source.x10C_joint[i]));
    }
    HM(B(CD.ecb_source.x124)); HM(B(CD.ecb_source.x128)); HM(B(CD.ecb_source.x12C));
    hash_MapCollData(&M);
    for (i = 0; i < NVX; i++) { HM(B(VERTS[i].x)); HM(B(VERTS[i].y)); }
    for (i = 0; i < NLN; i++) hash_MapLine(&LINES[i]);
    for (i = 0; i < NJ; i++) hash_MapJoint(&JOINTS[i]);
    for (i = 0; i < NVX; i++) hash_CollVtx(&VTXA[i]);
    for (i = 0; i < NLN; i++) hash_CollLine(&LINEA[i]);
    for (i = 0; i < 8; i++) hash_CollJoint(&JOINTA[i]);
    for (i = 0; i < 48; i++) hash_mp_UnkStruct0(&ISL[i]);
    HM((u32) ISLN);
    hash_HSD_GObj(&GO);
    HM(PI(groundCollVtx)); HM(PI(groundCollLine)); HM(PI(groundCollJoint)); HM(PI(jointListStart)); HM(PI(jointListEnd)); HM(PI(mpLib_804D64B4));
    HM((u32) didCheckBounding); HM((u32) scriptBatchDepth);
    HM((u32) mpLib_804D64CC); HM((u32) mpLib_804D64D0); HM((u32) mpLib_804D64D4); HM((u32) mpLib_804D64D8); HM((u32) mpLib_804D64DC); HM((u32) mpLib_804D64E0); HM((u32) mpLib_804D64E4);
    for (i = 0; i < 2; i++) { HM(B(mpLib_80458868[i].top)); HM(B(mpLib_80458868[i].bottom)); HM(B(mpLib_80458868[i].left)); HM(B(mpLib_80458868[i].right)); }
    for (i = 0; i < 9; i++) { HM((u32) mpColl_80458810.right[i]); HM((u32) mpColl_80458810.left[i]); }
    HM(B(mpColl_80458810.normal.x)); HM(B(mpColl_80458810.normal.y)); HM(B(mpColl_80458810.normal.z));
    HM((u32) mpColl_804D6488); HM((u32) mpColl_804D648C); HM(B(mpColl_804D6490_max_x)); HM((u32) mpColl_804D6494_line_id); HM(mpColl_804D6498_flags); HM((u32) mpColl_IsEcbTiny);
    HM(FI((const void*) mpColl_804D64A0)); HM(PI(mpColl_804D64A4)); HM(FI((const void*) mpColl_804D64A8)); HM((u32) mpColl_804D64AC); HM((u32) mpColl_LedgeLevelOK);
    HM(PI(mpIsland_80458E88.next)); HM(PI(mpIsland_80458E88.x4)); HM(PI(mpIsland_80458E88.x8)); HM(PI(mpIsland_80458E88.xC)); HM(PI(mpIsland_80458E88.x10));
    HM(PI(mpIsland_80458E88.x14)); HM(PI(mpIsland_80458E88.x18)); HM(PI(mpIsland_80458E88.x1C)); HM(PI(mpIsland_80458E88.x20));
    HM((u32) stage_info.grkind);
    for (i = 0; i < 3; i++) { HM(B(V3[i].x)); HM(B(V3[i].y)); HM(B(V3[i].z)); }
    HM(OUTU[0]); HM(OUTU[1]); HM((u32) OUTI[0]); HM((u32) OUTI[1]); HM(B(OUTF[0])); HM(B(OUTF[1]));
}

int main(void)
{
    int op, k;
    setbuf(__acrt_iob_func(1), 0);
    getenv_only = getenv("MTU_ONLY") != 0;
    preg(&M, sizeof M); preg(VERTS, sizeof VERTS); preg(LINES, sizeof LINES); preg(JOINTS, sizeof JOINTS);
    preg(VTXA, sizeof VTXA); preg(LINEA, sizeof LINEA); preg(JOINTA, sizeof JOINTA); preg(ISL, sizeof ISL);
    preg(&CD, sizeof CD); preg(&GO, sizeof GO); preg(JO, sizeof JO); preg(OUTI, sizeof OUTI); preg(V3, sizeof V3);
    freg((const void*) jcbA); freg((const void*) jcbB); freg((const void*) cbPipe); freg((const void*) cbFloor);
    freg_all();
    for (op = 0; op < @NOPS@; op++) {
        for (k = 0; k < @CASES@; k++) {
            u32 ret = 0;
            if (getenv_only && !(op == DBG_OP && k == DBG_K)) continue;
            tu_case(op, k);
            mtu_dbgp = (op == DBG_OP && k == DBG_K);
            setup();
            __try {
            switch (op) {
@COPS@
            }
            } __except (_exception_code() == 0xE0A55E87ul) { HM(0xA55E87u); ASSN[op]++; }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    if (getenv("MTU_STAT")) { for (op = 0; op < @NOPS@; op++) fprintf_stub(op); }
    return 0;
}
'''

# ----------------------------------------------------------------------------------- Rust driver
R_HEAD = r'''//! Written by gen/mtu/mk_mp.py from the same table as gen/mtu/tu/mp.c.
//! tu::{mplib, mpcoll, mpisland} vs the C (the three files as one TU, shipping contraction; regenerate
//! the expected file with `python rust/melee_world/gen/mtu/mk_mp.py run`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu as m;
use melee_world::tu::mpcoll as pc;
use melee_world::tu::mpisland as pi_;
use melee_world::tu::mplib as pl;
use mtu_harness::*;
#[path = "mtu/mp_gen.rs"]
mod g;

const NLN: usize = 24;
const NVX: usize = 64;
const NJ: usize = 3;
static mut M: MapCollData = unsafe { core::mem::zeroed() };
static mut VERTS: [Vec2; NVX] = unsafe { core::mem::zeroed() };
static mut LINES: [MapLine; NLN] = unsafe { core::mem::zeroed() };
static mut JOINTS: [MapJoint; NJ] = unsafe { core::mem::zeroed() };
static mut VTXA: [CollVtx; 2048] = unsafe { core::mem::zeroed() };
static mut LINEA: [CollLine; 1536] = unsafe { core::mem::zeroed() };
static mut JOINTA: [CollJoint; 8] = unsafe { core::mem::zeroed() };
static mut ISL: [mp_UnkStruct0; 48] = unsafe { core::mem::zeroed() };
static mut ISLN: i32 = 0;
static mut CD: CollData = unsafe { core::mem::zeroed() };
static mut GO: HSD_GObj = unsafe { core::mem::zeroed() };
static mut JO: [HSD_JObj; 8] = unsafe { core::mem::zeroed() };
static mut OUTI: [i32; 2] = [0; 2];
static mut OUTF: [f32; 2] = [0.0; 2];
static mut OUTU: [u32; 2] = [0; 2];
static mut V3: [Vec3; 3] = unsafe { core::mem::zeroed() };
static mut NLN_: i32 = 0;
static mut NVX_: i32 = 0;

unsafe extern "C" fn jcb_a(ud: *mut c_void, jid: i32, c: *mut CollData, x50: i32, k: u32, dy: f32) {
    t(0xCA); t(pi(ud)); t(jid as u32); t(pi(c)); t(x50 as u32); t(k); t(dy.to_bits()); (*c).x50 += 1.0;
}
unsafe extern "C" fn jcb_b(ud: *mut c_void, jid: i32, c: *mut CollData, x50: i32, k: u32, dy: f32) {
    t(0xCB); t(pi(ud)); t(jid as u32); t(pi(c)); t(x50 as u32); t(k); t(dy.to_bits()); (*c).x50 -= 0.5;
}
unsafe extern "C" fn cb_pipe(c: *mut CollData, f: u32) -> i32 {
    t(0xCC); t(pi(c)); t(f); (*c).cur_pos.x += 0.5; (fv() & 1) as i32
}
unsafe extern "C" fn cb_floor(g: *mut HSD_GObj, i: i32) -> i32 {
    t(0xCD); t(pi(g)); t(i as u32); (fv() & 1) as i32
}
unsafe fn jp(i: usize) -> *mut HSD_JObj {
    if rnd() % 5 == 0 { null_mut() } else { &raw mut JO[i] }
}
fn ri() -> f32 {
    if rnd() % 8 == 0 { return rg(); }
    ((rnd() % 21) as i32 - 10) as f32
}
fn rc() -> f32 {
    ((rnd() % 61) as i32 - 30) as f32 + if rnd() & 1 != 0 { 0.5 } else { 0.0 }
}
unsafe fn rvid() -> i32 {
    (rnd() % NLN_ as u32) as i32
}
unsafe fn rlid() -> i32 {
    if rnd() % 4 == 0 { -1 } else { (rnd() % NLN_ as u32) as i32 }
}
fn rnn() -> f32 {
    const K: [f32; 4] = [1.0, 13.0, 100.0, 1000.0];
    let a = ((rnd() % 41) as i32 - 20) as f32 / 20.0;
    a * K[(rnd() % 4) as usize]
}
unsafe fn mkecb(e: *mut ftECB, w: f32, h: f32) {
    (*e).top.x = 0.0; (*e).top.y = h; (*e).bottom.x = 0.0; (*e).bottom.y = 0.0;
    (*e).right.x = w; (*e).right.y = h * 0.5; (*e).left.x = -w; (*e).left.y = h * 0.5;
}

unsafe fn build_world() {
    let mut n = [0i32; 5];
    let mut st = [0i32; 5];
    let mut tot = 0i32;
    let mut slot = 0i32;
    let mut joint_of = [0i32; NLN];
    let mut grp_of = [0usize; NLN];
    let mut pos_of = [0i32; NLN];
    let mut p0x = [0f32; NLN];
    let mut p0y = [0f32; NLN];
    let mut p1x = [0f32; NLN];
    let mut p1y = [0f32; NLN];
    n[0] = 2 + (rnd() % 4) as i32; n[1] = 1 + (rnd() % 3) as i32; n[2] = 1 + (rnd() % 2) as i32; n[3] = 1 + (rnd() % 2) as i32; n[4] = (rnd() % 3) as i32;
    for g in 0..5 { st[g] = tot; tot += n[g]; }
    for g in 0..5 {
        let mut cx = ((rnd() % 61) as i32 - 30) as f32;
        let mut cy = ((rnd() % 41) as i32 - 20) as f32;
        for k in 0..n[g] {
            let len = 4 + (rnd() % 12) as i32;
            let d = (rnd() % 9) as i32 - 4;
            let i = (st[g] + k) as usize;
            grp_of[i] = g; pos_of[i] = k;
            p0x[i] = cx; p0y[i] = cy;
            if g == 0 { cx += len as f32; cy += d as f32; }
            else if g == 1 { cx -= len as f32; cy += d as f32; }
            else if g == 2 { cy -= len as f32; cx += d as f32; }
            else if g == 3 { cy += len as f32; cx += d as f32; }
            else { let mut dx = (rnd() % 21) as i32 - 10; let dy = (rnd() % 21) as i32 - 10; if dx == 0 && dy == 0 { dx = 1; } cx += dx as f32; cy += dy as f32; }
            p1x[i] = cx; p1y[i] = cy;
        }
    }
    for g in 0..5 { for k in 0..n[g] { joint_of[(st[g] + k) as usize] = (k * 3) / n[g]; } }
    for j in 0..NJ {
        let first = slot;
        let mut cnt = 0i32;
        let (mut lo_x, mut lo_y, mut hi_x, mut hi_y) = (1e9f32, 1e9f32, -1e9f32, -1e9f32);
        for g in 0..5 {
            let mut s = 0i32;
            let mut c = 0i32;
            for k in 0..n[g] { if joint_of[(st[g] + k) as usize] == j as i32 { if c == 0 { s = st[g] + k; } c += 1; } }
            JOINTS[j].ranges[g].start = (if c != 0 { s } else { st[g] }) as i16; JOINTS[j].ranges[g].count = c as i16;
        }
        for i in 0..tot as usize {
            if joint_of[i] == j as i32 {
                LINES[i].v0_idx = slot as u16; LINES[i].v1_idx = (slot + 1) as u16;
                let sl = slot as usize;
                VERTS[sl].x = p0x[i]; VERTS[sl].y = p0y[i]; VERTS[sl + 1].x = p1x[i]; VERTS[sl + 1].y = p1y[i];
                if p0x[i] < lo_x { lo_x = p0x[i]; } if p1x[i] < lo_x { lo_x = p1x[i]; } if p0x[i] > hi_x { hi_x = p0x[i]; } if p1x[i] > hi_x { hi_x = p1x[i]; }
                if p0y[i] < lo_y { lo_y = p0y[i]; } if p1y[i] < lo_y { lo_y = p1y[i]; } if p0y[i] > hi_y { hi_y = p0y[i]; } if p1y[i] > hi_y { hi_y = p1y[i]; }
                slot += 2; cnt += 2;
            }
        }
        JOINTS[j].left_bound = lo_x - 2.0; JOINTS[j].bottom_bound = lo_y - 2.0; JOINTS[j].right_bound = hi_x + 2.0; JOINTS[j].top_bound = hi_y + 2.0;
        JOINTS[j].vtx_start = first as i16; JOINTS[j].vtx_count = cnt as i16;
    }
    for i in 0..tot as usize {
        LINES[i].prev_id0 = (if pos_of[i] > 0 { i as i32 - 1 } else { -1 }) as i16;
        LINES[i].next_id0 = (if pos_of[i] < n[grp_of[i]] - 1 { i as i32 + 1 } else { -1 }) as i16;
        let r = rnd() as i32; if r % 8 == 0 { LINES[i].prev_id0 = (if i > 0 { (rnd() % i as u32) as i32 } else { -1 }) as i16; }
        let r = rnd() as i32; if r % 8 == 0 { LINES[i].next_id0 = (if tot - 1 - i as i32 > 0 { i as i32 + 1 + (rnd() % (tot - 1 - i as i32) as u32) as i32 } else { -1 }) as i16; }
        LINES[i].prev_id1 = (if rnd() % 3 == 0 { if i > 0 { (rnd() % i as u32) as i32 } else { -1 } } else { -1 }) as i16;
        LINES[i].next_id1 = (if rnd() % 3 == 0 { if tot - 1 - i as i32 > 0 { i as i32 + 1 + (rnd() % (tot - 1 - i as i32) as u32) as i32 } else { -1 } } else { -1 }) as i16;
        let mut hf = if grp_of[i] < 4 { 1u32 << grp_of[i] } else { 1u32 << (rnd() % 4) };
        if rnd() & 1 != 0 { hf |= 0x100; }
        if rnd() & 1 != 0 { hf |= 0x200; }
        if rnd() % 4 == 0 { hf |= 0x400; }
        hf |= fv() & 0xF870;
        LINES[i].hi_flags = hf as u16;
        LINES[i].lo_flags = (fv() & 0xFFFF) as u16;
        if rnd() % 3 == 0 { LINES[i].lo_flags |= 0x400; }
    }
    M.verts = VERTS.as_mut_ptr(); M.vert_count = slot; M.lines = LINES.as_mut_ptr(); M.line_count = tot; M.joints = JOINTS.as_mut_ptr(); M.joint_count = NJ as i32;
    for g in 0..5 { M.ranges[g].start = st[g] as i16; M.ranges[g].count = n[g] as i16; }
    NLN_ = tot; NVX_ = slot;
}

unsafe fn setup() {
    M = core::mem::zeroed(); VERTS = core::mem::zeroed(); LINES = core::mem::zeroed(); JOINTS = core::mem::zeroed();
    VTXA = core::mem::zeroed(); LINEA = core::mem::zeroed(); JOINTA = core::mem::zeroed(); ISL = core::mem::zeroed(); ISLN = 0;
    CD = core::mem::zeroed(); GO = core::mem::zeroed(); JO = core::mem::zeroed(); OUTI = [0; 2]; OUTF = [0.0; 2]; OUTU = [0; 2]; V3 = core::mem::zeroed();
    pl::ext::stage_info.grkind = 0;
    pc::ext::DbLevel = 0;
    pc::mpColl_80458810 = core::mem::zeroed(); pc::mpColl_804D6488 = 0; pc::mpColl_804D648C = 0; pc::mpColl_804D6490_max_x = 0.0; pc::mpColl_804D6494_line_id = 0; pc::mpColl_804D6498_flags = 0;
    pc::mpColl_IsEcbTiny = 0; pc::mpColl_804D64A0 = None; pc::mpColl_804D64A4 = null_mut(); pc::mpColl_804D64A8 = None; pc::mpColl_804D64AC = 0; pc::mpColl_LedgeLevelOK = 0;
    pl::mpLib_804D64CC = 0; pl::mpLib_804D64D0 = 0; pl::mpLib_804D64D4 = 0; pl::mpLib_804D64D8 = 0; pl::mpLib_804D64DC = 0; pl::mpLib_804D64E0 = 0; pl::mpLib_804D64E4 = 0;
    pl::mpLib_80458868 = core::mem::zeroed(); pl::scriptBatchDepth = 0;
    build_world();
    pl::mpLibLoad(&raw mut M);
    for j in 0..NJ {
        let cj = &raw mut JOINTA[j];
        if rnd() % 5 == 0 { (*cj).flags &= !(CollJoint_Enabled as u32); }
        if rnd() % 6 == 0 { (*cj).flags |= CollJoint_Hidden as u32; }
        (*cj).flags |= fv() & 0x1F00;
        (*cj).set_xE((rnd() & 1) as u8);
        (*cj).x20 = if rnd() & 1 != 0 { &raw mut JO[j] } else { null_mut() };
        (*cj).cb_0 = if rnd() % 3 != 0 { if rnd() & 1 != 0 { Some(jcb_a) } else { Some(jcb_b) } } else { None };
        (*cj).cb_data_0 = if rnd() & 1 != 0 { &raw mut GO as *mut _ } else { null_mut() };
        (*cj).cb_1 = if rnd() % 3 != 0 { if rnd() & 1 != 0 { Some(jcb_a) } else { Some(jcb_b) } } else { None };
        (*cj).cb_data_1 = if rnd() & 1 != 0 { &raw mut JO[1] as *mut _ } else { null_mut() };
    }
    g::fill_HSD_GObj(&mut GO);
    for i in 0..8 { g::fill_HSD_JObj(&mut JO[i]); }
    g::fill_CollData(&mut CD);
    CD.x0_gobj = if rnd() & 3 != 0 { &raw mut GO } else { null_mut() };
    CD.cur_pos.x = rc(); CD.cur_pos.y = rc(); CD.cur_pos.z = 0.0;
    if rnd() % 4 != 0 {
        let li = (rnd() % NLN_ as u32) as usize;
        let vi = rnd() & 1;
        let ox = ((rnd() % 9) as i32 - 4) as f32 * 0.5;
        let oy = ((rnd() % 9) as i32 - 4) as f32 * 0.5;
        let vp = &VERTS[(if vi != 0 { LINES[li].v1_idx } else { LINES[li].v0_idx }) as usize];
        CD.cur_pos.x = vp.x + ox; CD.cur_pos.y = vp.y + oy;
    }
    CD.prev_pos.x = CD.cur_pos.x + ((rnd() % 7) as i32 - 3) as f32 * 0.5; CD.prev_pos.y = CD.cur_pos.y + ((rnd() % 7) as i32 - 3) as f32 * 0.5; CD.prev_pos.z = 0.0;
    CD.last_pos.x = CD.prev_pos.x + ((rnd() % 5) as i32 - 2) as f32 * 0.5; CD.last_pos.y = CD.prev_pos.y + ((rnd() % 5) as i32 - 2) as f32 * 0.5; CD.last_pos.z = 0.0;
    CD.x28_vec = CD.prev_pos;
    let w = (3 + (rnd() % 3)) as f32;
    let h = (8 + (rnd() % 8)) as f32;
    mkecb(&raw mut CD.ecb, w, h);
    mkecb(&raw mut CD.prev_ecb, w + (rnd() % 2) as f32, h);
    mkecb(&raw mut CD.desired_ecb, w, h + (rnd() % 3) as f32);
    mkecb(&raw mut CD.x64_ecb, w, h);
    mkecb(&raw mut CD.xE4_ecb, w, h);
    CD.floor_skip = if rnd() % 4 == 0 { (rnd() % NLN_ as u32) as i32 } else { -1 };
    CD.ledge_id_right = if rnd() % 3 == 0 { (rnd() % NLN_ as u32) as i32 } else { -1 };
    CD.ledge_id_left = if rnd() % 3 == 0 { (rnd() % NLN_ as u32) as i32 } else { -1 };
    CD.joint_id_skip = if rnd() % 6 == 0 { (rnd() % NJ as u32) as i32 } else { -1 };
    CD.joint_id_only = if rnd() % 6 == 0 { (rnd() % NJ as u32) as i32 } else { -1 };
    CD.x50 = rg(); CD.lstick_x = rg();
    CD.ledge_snap_x = (2 + (rnd() % 4)) as f32; CD.ledge_snap_y = (rnd() % 8) as f32; CD.ledge_snap_height = (4 + (rnd() % 8)) as f32;
    CD.facing_dir = if rnd() & 1 != 0 { 1 } else { -1 };
    CD.env_flags = rnd() as i32; CD.prev_env_flags = rnd() as i32; CD.x130_flags = rnd() & 0xFF; CD.x13C = rnd() as i32;
    if rnd() & 1 != 0 {
        CD.ecb_source.kind = ECBSource_Fixed as _;
        CD.ecb_source.anon_4.anon_0_0.up = (8 + (rnd() % 8)) as f32; CD.ecb_source.anon_4.anon_0_0.down = (rnd() % 3) as f32; CD.ecb_source.anon_4.anon_0_0.front = (3 + (rnd() % 3)) as f32; CD.ecb_source.anon_4.anon_0_0.back = (3 + (rnd() % 3)) as f32;
        CD.ecb_source.anon_4.anon_0_0.angle = ((rnd() % 13) as i32 - 6) as f32 * 0.5;
    } else {
        CD.ecb_source.kind = ECBSource_JObj as _;
        CD.ecb_source.anon_4.anon_0.x108_joint = &raw mut JO[0];
        for i in 0..6 { CD.ecb_source.anon_4.anon_0.x10C_joint[i] = &raw mut JO[i + 1]; }
    }
    CD.ecb_source.x124 = ((rnd() % 41) as i32 - 20) as f32 / 20.0; CD.ecb_source.x128 = (rnd() % 12) as f32; CD.ecb_source.x12C = (rnd() % 12) as f32;
    CD.floor.index = rlid(); CD.floor.flags = fv(); CD.ceiling.index = rlid(); CD.ceiling.flags = fv();
    CD.left_facing_wall.index = rlid(); CD.left_facing_wall.flags = fv(); CD.right_facing_wall.index = rlid(); CD.right_facing_wall.flags = fv();
    CD.contact.x = rg(); CD.contact.y = rg(); CD.contact.z = 0.0;
    if rnd() % 2 == 0 {
        let li = (rnd() % NLN_ as u32) as usize;
        if LINES[li].hi_flags & 1 != 0 { CD.floor.index = li as i32; } else if LINES[li].hi_flags & 2 != 0 { CD.ceiling.index = li as i32; } else if LINES[li].hi_flags & 4 != 0 { CD.right_facing_wall.index = li as i32; } else { CD.left_facing_wall.index = li as i32; }
    }
}

unsafe fn hash_all() {
    g::hash_CollData(&CD);
    hm(CD.ecb_source.kind as u32);
    if CD.ecb_source.kind == ECBSource_Fixed as _ {
        hm(CD.ecb_source.anon_4.anon_0_0.up.to_bits()); hm(CD.ecb_source.anon_4.anon_0_0.down.to_bits()); hm(CD.ecb_source.anon_4.anon_0_0.front.to_bits());
        hm(CD.ecb_source.anon_4.anon_0_0.back.to_bits()); hm(CD.ecb_source.anon_4.anon_0_0.angle.to_bits());
    } else {
        hm(pi(CD.ecb_source.anon_4.anon_0.x108_joint)); for i in 0..6 { hm(pi(CD.ecb_source.anon_4.anon_0.x10C_joint[i])); }
    }
    hm(CD.ecb_source.x124.to_bits()); hm(CD.ecb_source.x128.to_bits()); hm(CD.ecb_source.x12C.to_bits());
    g::hash_MapCollData(&M);
    for i in 0..NVX { hm(VERTS[i].x.to_bits()); hm(VERTS[i].y.to_bits()); }
    for i in 0..NLN { g::hash_MapLine(&LINES[i]); }
    for i in 0..NJ { g::hash_MapJoint(&JOINTS[i]); }
    for i in 0..NVX { g::hash_CollVtx(&VTXA[i]); }
    for i in 0..NLN { g::hash_CollLine(&LINEA[i]); }
    for i in 0..8 { g::hash_CollJoint(&JOINTA[i]); }
    for i in 0..48 { g::hash_mp_UnkStruct0(&ISL[i]); }
    hm(ISLN as u32);
    g::hash_HSD_GObj(&GO);
    hm(pi(pl::groundCollVtx)); hm(pi(pl::groundCollLine)); hm(pi(pl::groundCollJoint)); hm(pi(pl::jointListStart)); hm(pi(pl::jointListEnd)); hm(pi(pl::mpLib_804D64B4));
    hm(pl::didCheckBounding as u32); hm(pl::scriptBatchDepth as u32);
    hm(pl::mpLib_804D64CC as u32); hm(pl::mpLib_804D64D0 as u32); hm(pl::mpLib_804D64D4 as u32); hm(pl::mpLib_804D64D8 as u32); hm(pl::mpLib_804D64DC as u32); hm(pl::mpLib_804D64E0 as u32); hm(pl::mpLib_804D64E4 as u32);
    for i in 0..2 { hm(pl::mpLib_80458868[i].top.to_bits()); hm(pl::mpLib_80458868[i].bottom.to_bits()); hm(pl::mpLib_80458868[i].left.to_bits()); hm(pl::mpLib_80458868[i].right.to_bits()); }
    for i in 0..9 { hm(pc::mpColl_80458810.right[i] as u32); hm(pc::mpColl_80458810.left[i] as u32); }
    hm(pc::mpColl_80458810.normal.x.to_bits()); hm(pc::mpColl_80458810.normal.y.to_bits()); hm(pc::mpColl_80458810.normal.z.to_bits());
    hm(pc::mpColl_804D6488 as u32); hm(pc::mpColl_804D648C as u32); hm(pc::mpColl_804D6490_max_x.to_bits()); hm(pc::mpColl_804D6494_line_id as u32); hm(pc::mpColl_804D6498_flags as u32); hm(pc::mpColl_IsEcbTiny as u32);
    hm(fi(pc::mpColl_804D64A0)); hm(pi(pc::mpColl_804D64A4)); hm(fi(pc::mpColl_804D64A8)); hm(pc::mpColl_804D64AC as u32); hm(pc::mpColl_LedgeLevelOK as u32);
    hm(pi(pi_::mpIsland_80458E88.next)); hm(pi(pi_::mpIsland_80458E88.x4)); hm(pi(pi_::mpIsland_80458E88.x8)); hm(pi(pi_::mpIsland_80458E88.xC)); hm(pi(pi_::mpIsland_80458E88.x10));
    hm(pi(pi_::mpIsland_80458E88.x14)); hm(pi(pi_::mpIsland_80458E88.x18)); hm(pi(pi_::mpIsland_80458E88.x1C)); hm(pi(pi_::mpIsland_80458E88.x20));
    hm(pl::ext::stage_info.grkind as u32);
    for i in 0..3 { hm(V3[i].x.to_bits()); hm(V3[i].y.to_bits()); hm(V3[i].z.to_bits()); }
    hm(OUTU[0]); hm(OUTU[1]); hm(OUTI[0] as u32); hm(OUTI[1] as u32); hm(OUTF[0].to_bits()); hm(OUTF[1].to_bits());
}

#[test]
fn mp_matches_c() {
    unsafe {
        std::panic::set_hook(Box::new(|i| { let m = i.to_string(); if !(m.contains(": assertion failed") || m.contains("lineID")) { eprintln!("{}", m); } }));
        g::install();
        pl::ext::hook::OSReport = Some(|_| {});
        pc::ext::hook::OSReport = Some(|_| {});
        preg(&raw const M, core::mem::size_of::<MapCollData>());
        preg(&raw const VERTS, core::mem::size_of_val(&VERTS));
        preg(&raw const LINES, core::mem::size_of_val(&LINES));
        preg(&raw const JOINTS, core::mem::size_of_val(&JOINTS));
        preg(&raw const VTXA, core::mem::size_of_val(&VTXA));
        preg(&raw const LINEA, core::mem::size_of_val(&LINEA));
        preg(&raw const JOINTA, core::mem::size_of_val(&JOINTA));
        preg(&raw const ISL, core::mem::size_of_val(&ISL));
        preg(&raw const CD, core::mem::size_of::<CollData>());
        preg(&raw const GO, core::mem::size_of::<HSD_GObj>());
        preg(&raw const JO, core::mem::size_of_val(&JO));
        preg(&raw const OUTI, core::mem::size_of_val(&OUTI));
        preg(&raw const V3, core::mem::size_of_val(&V3));
        freg(jcb_a as *const () as usize); freg(jcb_b as *const () as usize); freg(cb_pipe as *const () as usize); freg(cb_floor as *const () as usize);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_mp.txt"));
        for op in 0..@NOPS@u32 {
            for k in 0..@CASES@u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match op {
@ROPS@
                    _ => unreachable!(),
                }
                }));
                if let Err(e) = r {
                    let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    if !(msg.contains(": assertion failed") || msg.contains("lineID")) { std::panic::resume_unwind(e); }
                    hm(0xA55E87);
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), @NOPS@ * @CASES@);
    }
}
'''


def gen():
    from specs_mp import SPECS
    import gen_tu
    gen_tu.build("mp", SPECS["mp"], False)
    cops = "\n".join(f"            case {i}: {{ {c} }} break;" for i, (n, c, r, k) in enumerate(OPS))
    rops = "\n".join(f"                    {i} => {{ {r} }}" for i, (n, c, r, k) in enumerate(OPS))
    sub = {"@NOPS@": str(len(OPS)), "@CASES@": str(CASES)}
    c = C_HEAD.replace("@COPS@", cops)
    r = R_HEAD.replace("@ROPS@", rops)
    for a, b in sub.items():
        c = c.replace(a, b)
        r = r.replace(a, b)
    (HERE / "tu" / "mp.c").write_text(c, encoding="utf-8", newline="\n")
    (ROOT / "rust/melee_world/tests/tu_mp.rs").write_text(r, encoding="utf-8", newline="\n")
    print(f"mk_mp: {len(OPS)} ops x {CASES} cases")


LINK = ["pc/gameworld/gekko_fp.c", "src/melee/lb/lbvector.c", "pc/gameworld/mtx_pc.c"]


def run():
    import run_tu
    BUILD = run_tu.BUILD
    BUILD.mkdir(parents=True, exist_ok=True)
    from specs_mp import SPECS
    import gen_tu
    spec = SPECS["mp"]
    src = gen_tu.patched_source("mp", spec)
    drv = HERE / "tu" / "mp.c"
    exe = BUILD / "mtu_mp.exe"
    lls = []
    dbg = []
    if os.environ.get("MTU_DBG"):
        dop, dk = os.environ["MTU_DBG"].split(":")
        dbg = [f"-DDBG_OP={dop}", f"-DDBG_K={dk}"]
    for s in [drv] + [ROOT / x for x in LINK]:
        ll = BUILD / f"mtu_mp__{s.stem}.ll"
        lo = BUILD / f"mtu_mp__{s.stem}.lo.ll"
        run_tu.run([run_tu.CLANG, *run_tu.FLAGS, "-fms-extensions", "-iquote", "src/melee/mp", "-I", str((ROOT / spec["src"]).parent), "-I", str(HERE / "tu"),
                    '-DTU_SRC="' + str(ROOT / src).replace("\\", "/") + '"', *dbg, str(s), "-o", str(ll)], "mp")
        run_tu.run([sys.executable, str(run_tu.LOWER), str(ll), str(lo)], "mp")
        lls.append(str(lo))
    run_tu.run([run_tu.CLANG, "-O0", "-w", *lls, "-o", str(exe), "-Wl,/OPT:NOICF", "-Wl,/FORCE:UNRESOLVED", "-fuse-ld=lld"], "mp")
    r_ = subprocess.run([str(exe)], capture_output=True, text=True); out = r_.stdout
    if os.environ.get("MTU_STAT"): print(r_.stderr)
    if dbg:
        tr = [l for l in out.splitlines() if l.startswith(("T ", "H ", "B ", "ENT", "DBG"))]
        (BUILD / "dbg_mp.txt").write_text(chr(10).join(tr) + chr(10), encoding="utf-8")
        print(f"mk_mp: {len(tr)} trace words -> {BUILD / 'dbg_mp.txt'}")
        return
    dd = ROOT / "rust/melee_world/tests/data"
    (dd / "mtu_mp.txt").write_text(out, encoding="utf-8", newline="\n")
    print(f"mk_mp: {len(out.splitlines())} lines")


if __name__ == "__main__":
    mode = sys.argv[1]
    if mode in ("gen", "all"):
        gen()
    if mode in ("run", "all"):
        run()
