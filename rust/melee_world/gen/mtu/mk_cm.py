#!/usr/bin/env python3
"""State oracle for melee_world::tu::camera (the mechanical port of src/melee/cm/camera.c).

    python rust/melee_world/gen/mtu/mk_cm.py gen|run|all          (from the worktree root)

camera.c is compiled natively with the SHIPPING float contraction as one C TU; gen_tu.py (spec "cm" of
specs_cm.py) writes the recording stubs for every callee outside it and the field-by-field fill / hash
of the state records; this file writes the driver around it, both halves from the one table below:
  gen/mtu/tu/cm.c            C driver
  tests/tu_cm.rs            Rust driver, draw for draw
  tests/data/mtu_cm.txt     C output (op case ret state-hash trace-hash), computed from C source only

A case starts from the game's own Camera_Init (a pool of 8 subjects), a few subjects taken from it
with Camera_80029044, a randomised game_camera / cm_script / debug mode, CObjs and GObjs wired to
each other, and stub return values from the shared stream. The math the port takes from
melee_runtime (lbVector_*, atan2f, tanf, sinf ...) runs as the real C on the oracle side.
"""
import os, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
os.chdir(ROOT)
CASES = 500

OPS = []


def op(c, r, ret="v", name=None):
    OPS.append((name or c.split("(")[0].split()[-1], c, r, ret))


TS = "((rnd() & 1) ? &game_camera.transform : &game_camera.transform_copy)"
TS_R = "(if rnd() & 1 != 0 { &raw mut cm::game_camera.transform } else { &raw mut cm::game_camera.transform_copy })"
SUBJI = "(&SUBJ[rnd() % 8])"
SUBJI_R = "(&raw mut SUBJ[(rnd() % 8) as usize])"

# --- the targets, then helpers around them
op("Camera_Init(1 + (int) (rnd() % 8));", "cm::Camera_Init(1 + (rnd() % 8) as i32);")
op("Camera_Create();", "cm::Camera_Create();")
op("Camera_ForgetGameCamera();", "cm::Camera_ForgetGameCamera();")
op("Camera_RefreshViewingMtx();", "cm::Camera_RefreshViewingMtx();")
op("ret = (u32) Camera_ScriptCompletion();", "ret = cm::Camera_ScriptCompletion() as u32;", "i", "Camera_ScriptCompletion")
op("ret = (u32) Camera_8002928C(" + SUBJI + ");", "ret = cm::Camera_8002928C(" + SUBJI_R + ") as u32;", "i", "Camera_8002928C")
op("Camera_8002A4AC(&GO[0]);", "cm::Camera_8002A4AC(&raw mut GO[0]);")
op("{ CameraTransformState* ts = " + TS + "; Camera_8002A768(ts, (int) fv()); }",
   "{ let ts = " + TS_R + "; cm::Camera_8002A768(ts, fv() as i32); }", "v", "Camera_8002A768")
op("{ CameraTransformState* ts = " + TS + "; Camera_8002AF68(&CO[rnd() % 3], ts); }",
   "{ let ts = " + TS_R + "; cm::Camera_8002AF68(&raw mut CO[(rnd() % 3) as usize], ts); }", "v", "Camera_8002AF68")
op("Camera_8002B0E0();", "cm::Camera_8002B0E0();")
op("{ CameraTransformState* ts = " + TS + "; Camera_8002B1F8(ts); }", "{ let ts = " + TS_R + "; cm::Camera_8002B1F8(ts); }", "v", "Camera_8002B1F8")
op("Camera_8002B3D4(0);", "cm::Camera_8002B3D4(null_mut());")
op("Camera_8002F3AC();", "cm::Camera_8002F3AC();")
op("{ CameraTransformState* ts = " + TS + "; Camera_ApplyQuake(&BND[0], ts); }", "{ let ts = " + TS_R + "; cm::Camera_ApplyQuake(&raw mut BND[0], ts); }", "v", "Camera_ApplyQuake")
op("if (!game_camera.gobj) game_camera.gobj = &GO[0]; ret = (u32) Camera_LogicToScreen(" + SUBJI + ", &S2[0]);", "if cm::game_camera.gobj.is_null() { cm::game_camera.gobj = &raw mut GO[0]; } ret = cm::Camera_LogicToScreen(" + SUBJI_R + ", &raw mut S2[0]) as u32;", "i", "Camera_LogicToScreen")
op("ret = PI(Camera_PCMainCObj());", "ret = pi(cm::Camera_PCMainCObj());", "p", "Camera_PCMainCObj")
op("ret = (u32) Camera_ProjectToScreen(&CO[rnd() % 3], &V3[0], &S2[0]);", "ret = cm::Camera_ProjectToScreen(&raw mut CO[(rnd() % 3) as usize], &raw mut V3[0], &raw mut S2[0]) as u32;", "i", "Camera_ProjectToScreen")
op("ret = (u32) Camera_ProjectToScreen(&CO[rnd() % 3], &V3[0], 0);", "ret = cm::Camera_ProjectToScreen(&raw mut CO[(rnd() % 3) as usize], &raw mut V3[0], null_mut()) as u32;", "i", "Camera_ProjectToScreen_null")
op("{ u8 r = (u8) fv(); u8 g = (u8) fv(); u8 b = (u8) fv(); Camera_SetBackgroundColor(r, g, b); }",
   "{ let r = fv() as u8; let g = fv() as u8; let b = fv() as u8; cm::Camera_SetBackgroundColor(r, g, b); }", "v", "Camera_SetBackgroundColor")
op("Camera_SetQuakeScale(rw());", "cm::Camera_SetQuakeScale(rw());")
op("Camera_UpdateQuakes(&BND[0]);", "cm::Camera_UpdateQuakes(&raw mut BND[0]);")
op("cm_script_update();", "cm::cm_script_update();")
op("fn_8002F360(&GO[0]);", "cm::fn_8002F360(&raw mut GO[0]);")
op("fn_800301D0(&GO[0], (int) fv());", "cm::fn_800301D0(&raw mut GO[0], fv() as i32);")
op("update_bounds(&BND[0], &BND[1]);", "cm::update_bounds(&raw mut BND[0], &raw mut BND[1]);")
op("{ CameraTransformState* ts = " + TS + "; update_transform(&BND[0], ts); }", "{ let ts = " + TS_R + "; cm::update_transform(&raw mut BND[0], ts); }", "v", "update_transform")
op("{ float x = rw(); float y = rw(); Camera_SetQuakeOffset(x, y); }", "{ let x = rw(); let y = rw(); cm::Camera_SetQuakeOffset(x, y); }", "v", "Camera_SetQuakeOffset")
op("if (!game_camera.gobj) game_camera.gobj = &GO[0]; ret = (u32) Camera_80030BBC(&V3[0], &S2[0]);", "if cm::game_camera.gobj.is_null() { cm::game_camera.gobj = &raw mut GO[0]; } ret = cm::Camera_80030BBC(&raw mut V3[0], &raw mut S2[0]) as u32;", "i", "Camera_80030BBC")
op("if (!game_camera.gobj) game_camera.gobj = &GO[0]; ret = (u32) Camera_80030BBC(&V3[0], 0);", "if cm::game_camera.gobj.is_null() { cm::game_camera.gobj = &raw mut GO[0]; } ret = cm::Camera_80030BBC(&raw mut V3[0], null_mut()) as u32;", "i", "Camera_80030BBC_null")
# neighbours worth the same state: quake requests, script detach / attach, the pause and follow cameras
op("{ int k = (int) (rnd() % 4) + 1; Camera_RequestQuake(k, (rnd() & 1) ? &V3[0] : 0); }",
   "{ let k = (rnd() % 4) as u32 + 1; cm::Camera_RequestQuake(k, if rnd() & 1 != 0 { &raw mut V3[0] } else { null_mut() }); }", "v", "Camera_RequestQuake")
op("ret = (u32) Camera_ScriptDetach();", "ret = cm::Camera_ScriptDetach() as u32;", "i", "Camera_ScriptDetach")
op("Camera_ScriptAttach((int) (rnd() % 6) - 1);", "cm::Camera_ScriptAttach((rnd() % 6) as i32 - 1);", "v", "Camera_ScriptAttach")
op("Camera_ScriptCommit();", "cm::Camera_ScriptCommit();")
op("Camera_800293E0();", "cm::Camera_800293E0();")

# ----------------------------------------------------------------------------------- C driver
C_HEAD = r'''/* TU: src/melee/cm/camera.c */
/* C oracle for the Rust port of this TU: the game's file included verbatim; every callee out of it is a
 * recording stub (the lb / math helpers the port takes from melee_runtime are the real C). Written by
 * gen/mtu/mk_cm.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#define __assert mtu_unused_assert
#include "mtu_common.h"
#undef __assert
extern void setbuf(void*, char*);
extern void* __acrt_iob_func(unsigned);
extern int fprintf(void*, const char*, ...);
extern char* getenv(const char*);
extern void RaiseException(unsigned long, unsigned long, unsigned long, const void*);
static int ASLINE[256], LASTAL;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) cond; LASTAL = (int) line; RaiseException(0xE0A55E87ul, 1, 0, 0); }
void* memset(void* d, int c, unsigned long n) { unsigned char* p = (unsigned char*) d; while (n--) *p++ = (unsigned char) c; return d; }
void memzero(void* d, int n) { unsigned char* p = (unsigned char*) d; while (n-- > 0) *p++ = 0; }
int __HI(double x) { union { double d; u64_t_ u; } c; c.d = x; return (int) (c.u >> 32); }
int __LO(double x) { union { double d; u64_t_ u; } c; c.d = x; return (int) (u32) c.u; }
static CmSubject SUBJ[8];
static HSD_CObj CO[3];
static HSD_WObj WO[4];
static HSD_GObj GO[3];
static float MTXA[4][4];
static CameraBounds BND[2];
static S32Vec2 S2[2];
static Vec3 V3[3];
static int getenv_only;
int mtu_dbgp;
static int ASSN[256];
static void fprintf_stub(int op) { fprintf(__acrt_iob_func(2), "ASSERTS %d %d line %d%c", op, ASSN[op], ASLINE[op], 10); }
static float RNN(void) { static const float k[4] = { 1.0f, 13.0f, 100.0f, 1000.0f }; float a = (float) ((int) (rnd() % 41) - 20) / 20.0f; return a * k[rnd() % 4]; }
/* position-like draw: a few tens either way */
static float RNG(int lo, int hi, int div) { return (float) (lo + (int) (rnd() % (u32) (hi - lo + 1))) / (float) div; }
static float RPOS(void) { return (float) ((int) (rnd() % 401) - 200) / 4.0f; }
static float RPOSZ(void) { return (float) (rnd() % 600); }

/* camera.c redirects these four through its debug-fly wrappers after defining them; the stubs are the originals */
#undef Stage_GetCamBoundsLeftOffset
#undef Stage_GetCamBoundsRightOffset
#undef Stage_GetCamBoundsTopOffset
#undef Stage_GetCamBoundsBottomOffset
#include "cm_gen.h"

static void fill_vec(Vec3* v) { v->x = RPOS(); v->y = RPOS(); v->z = RPOS(); }
static void fill_ts(CameraTransformState* t)
{
    fill_vec(&t->interest); fill_vec(&t->target_interest); fill_vec(&t->position); fill_vec(&t->target_position);
    t->position.z = RPOSZ() + 50.0f; t->target_position.z = RPOSZ() + 50.0f;
    t->fov = (float) (15 + (int) (rnd() % 60)); t->target_fov = (float) (15 + (int) (rnd() % 60));
}

static void setup(void)
{
    int i, n;
    zero(SUBJ, sizeof SUBJ); zero(CO, sizeof CO); zero(WO, sizeof WO); zero(GO, sizeof GO); zero(MTXA, sizeof MTXA);
    zero(BND, sizeof BND); zero(S2, sizeof S2); zero(V3, sizeof V3);
    stage_info.grkind = (int) (rnd() % 50);
    cm_804D6458 = 0; cm_804D645C = 0; cm_804D6460 = 0; cm_804D6464 = 0; cm_804D6468 = 0;
    zero(&cm_largemap, sizeof cm_largemap);
    fill_HSD_GObj(&GO[0]); fill_HSD_GObj(&GO[1]); fill_HSD_GObj(&GO[2]);
    for (i = 0; i < 3; i++) { fill_HSD_CObj(&CO[i]); CO[i].eyepos = (rnd() & 1) ? &WO[i] : 0; CO[i].interest = (rnd() & 1) ? &WO[i + 1] : 0; CO[i].proj_mtx = 0; CO[i].flags = fv() & 1; }
    GO[0].hsd_obj = &CO[0]; GO[1].hsd_obj = &CO[1]; GO[2].hsd_obj = &CO[2];
    for (i = 0; i < 4; i++) { WO[i].pos.x = RPOS(); WO[i].pos.y = RPOS(); WO[i].pos.z = RPOS(); }
    for (i = 0; i < 16; i++) ((float*) MTXA)[i] = RNN();
    Camera_Init(8);
    n = (int) (rnd() % 6);
    for (i = 0; i < n; i++) {
        CmSubject* s = Camera_80029044((int) (rnd() % 3));
        s->pos.x = RPOS(); s->pos.y = RPOS(); s->pos.z = 0.0f; s->bone_pos.x = RPOS(); s->bone_pos.y = RPOS(); s->bone_pos.z = RPOS();
        s->facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        s->state_timer = (rnd() % 3 == 0) ? (s16) (rnd() % 700) : 0;
        s->was_framed = rnd() & 1; s->force_inactive = (rnd() % 8 == 0); s->on_ledge = rnd() & 1;
        s->ext.h.x = RPOS() * 0.1f; s->ext.h.y = RPOS() * 0.1f; s->ext.v.x = RPOS() * 0.1f; s->ext.v.y = RPOS() * 0.1f; s->ext.v.z = RPOS() * 0.1f;
        s->target_ext.h.x = RPOS() * 0.1f; s->target_ext.h.y = RPOS() * 0.1f; s->target_ext.v.x = RPOS() * 0.1f; s->target_ext.v.y = RPOS() * 0.1f; s->target_ext.v.z = RPOS() * 0.1f;
    }
    game_camera.mode = (int) (rnd() % 9);
    fill_ts(&game_camera.transform); fill_ts(&game_camera.transform_copy);
    game_camera.gobj = (rnd() % 4) ? &GO[0] : 0;
    game_camera.quake_gobj = (rnd() % 3 == 0) ? &GO[2] : 0;
    game_camera.translation.x = RPOS() * 0.1f; game_camera.translation.y = RPOS() * 0.1f;
    game_camera.quake_offset.x = RNN(); game_camera.quake_offset.y = RNN();
    game_camera.quake_scale = (float) (rnd() % 5) * 0.5f;
    for (i = 0; i < 5; i++) game_camera.quake_frames_left[i] = (rnd() % 3 == 0) ? (int) (rnd() % 30) : 0;
    for (i = 0; i < 16; i++) { game_camera.quakes[0][i].kind = (rnd() % 3 == 0) ? (int) (rnd() % 5) : 0; game_camera.quakes[1][i].kind = (int) (rnd() % 5); fill_vec(&game_camera.quakes[0][i].epicenter); fill_vec(&game_camera.quakes[1][i].epicenter); }
    game_camera.x2B0 = RPOS(); game_camera.x2B4 = RPOS() * 100.0f; game_camera.x2B8 = (s16) ((rnd() % 5 == 0) ? 1000 + (int) (rnd() % 5) : (int) (rnd() % 20));
    game_camera.x2BA = (s16) (rnd() % 5); game_camera.x2BC = (rnd() % 3 == 0) ? 1.0f : (float) (rnd() % 7) * 0.5f; game_camera.x2C0 = (float) (rnd() % 400) - 50.0f;
    game_camera.x398_b0 = rnd() & 1; game_camera.x398_b1 = rnd() & 1; game_camera.x398_b2 = rnd() & 1; game_camera.x398_b3 = rnd() & 1; game_camera.x398_b4 = rnd() & 1; game_camera.x398_b5 = rnd() & 1;
    game_camera.x399_b0_b1 = rnd() & 3; game_camera.x399_b2 = rnd() & 1; game_camera.x399_b3 = rnd() & 1; game_camera.x399_b4 = rnd() & 1; game_camera.x399_b5 = rnd() & 1; game_camera.x399_b6 = rnd() & 1; game_camera.x399_b7 = rnd() & 1;
    game_camera.x39A_b0 = rnd() & 1; game_camera.x39A_b1 = rnd() & 1; game_camera.x39A_b2 = rnd() & 1;
    game_camera.nearz = (float) (rnd() % 10) * 0.1f; game_camera.farz = (float) (rnd() % 20000);
    game_camera.background_color.r = (u8) fv(); game_camera.background_color.g = (u8) fv(); game_camera.background_color.b = (u8) fv();
    cm_804D6464 = (rnd() & 1) ? &CO[2] : 0;
    cm_80453004.last_mode = (int) (rnd() % 9); cm_80453004.ply_slot = (int) (rnd() % 4);
    fill_vec(&cm_80453004.follow_int_pos); fill_vec(&cm_80453004.follow_eye_pos); fill_vec(&cm_80453004.free_int_pos); fill_vec(&cm_80453004.free_eye_pos);
    cm_80453004.follow_fov = RPOS(); cm_80453004.free_fov = RPOS();
    /* the TARGET_PC script camera */
    zero(&cm_script, sizeof cm_script);
    if (rnd() % 3 == 0) {
        cm_script.state = (int) (rnd() % 3); cm_script.lift_bounds = (int) (rnd() & 1); cm_script.elapsed = (int) (rnd() % 20); cm_script.duration = (int) (rnd() % 30);
        cm_script.ease = (int) (rnd() % 4); cm_script.path_count = (int) (rnd() % 5); cm_script.path_frame = (int) (rnd() % 12);
        cm_script.follow_kind = (rnd() % 3 == 0) ? 1 + (int) (rnd() % 3) : 0; cm_script.follow_id = (int) (rnd() % 8);
        cm_script.shake_left = (int) (rnd() % 8); cm_script.shake_total = 1 + (int) (rnd() % 8); cm_script.shake_tick = (int) (rnd() % 100);
        cm_script.shake_intensity = RNN(); cm_script.saved_roll = RNN(); cm_script.saved_far = (float) (rnd() % 20000); cm_script.saved_up_vector = (int) (rnd() & 1);
        fill_vec(&cm_script.saved_up); fill_vec(&cm_script.follow_point); fill_vec(&cm_script.follow_offset); fill_vec(&cm_script.follow_eye_delta);
        fill_ts(&cm_script.normal);
        fill_vec(&cm_script.pose.eye); fill_vec(&cm_script.pose.interest); cm_script.pose.fov = RPOS(); cm_script.pose.roll = RNN();
        fill_vec(&cm_script.from.eye); fill_vec(&cm_script.from.interest); cm_script.from.fov = RPOS(); cm_script.from.roll = RNN();
        fill_vec(&cm_script.to.eye); fill_vec(&cm_script.to.interest); cm_script.to.fov = RPOS(); cm_script.to.roll = RNN();
        for (i = 0; i < 5; i++) { fill_vec(&cm_script.path[i].pose.eye); fill_vec(&cm_script.path[i].pose.interest); cm_script.path[i].pose.fov = RPOS(); cm_script.path[i].pose.roll = RNN(); cm_script.path[i].frame = i * 3 + (int) (rnd() % 3); }
        cm_script.completion = (int) (rnd() % 4);
    }
    for (i = 0; i < 2; i++) {
        BND[i].x_min = RPOS(); BND[i].y_min = RPOS(); BND[i].x_max = BND[i].x_min + (float) (rnd() % 200); BND[i].y_max = BND[i].y_min + (float) (rnd() % 150);
        BND[i].total_subjects = (int) (rnd() % 5); BND[i].z_pos = RPOSZ();
    }
    gm_CStickSmashTargetTest = rnd() & 1;
    for (i = 0; i < 4; i++) fill_HSD_PadStatus(&HSD_PadCopyStatus[i]);
    HSD_GObj_CameraKind = (u8) fv();
    fill_vec(&V3[0]);
}

static void hash_all(void)
{
    int i;
    hash_Camera(&game_camera);
    hash_CameraDebugMode(&cm_80453004);
    for (i = 0; i < 8; i++) hash_CmSubject(&SUBJ[i]);
    for (i = 0; i < 3; i++) { hash_HSD_CObj(&CO[i]); hash_HSD_GObj(&GO[i]); }
    for (i = 0; i < 4; i++) { HM(B(WO[i].pos.x)); HM(B(WO[i].pos.y)); HM(B(WO[i].pos.z)); }
    for (i = 0; i < 2; i++) hash_CameraBounds(&BND[i]);
    for (i = 0; i < 2; i++) { HM((u32) S2[i].x); HM((u32) S2[i].y); }
    for (i = 0; i < 3; i++) { HM(B(V3[i].x)); HM(B(V3[i].y)); HM(B(V3[i].z)); }
    HM(RH(&cm_script, sizeof cm_script));
    HM(PI(cm_largemap.primary)); HM(PI(cm_largemap.secondary)); HM(B(cm_largemap.primary_far)); HM(B(cm_largemap.secondary_far));
    HM(PI(cm_804D6458)); HM(PI(cm_804D645C)); HM(PI(cm_804D6460)); HM(PI(cm_804D6464)); HM(PI(cm_804D6468));
    HM(B(cm_803BCCA0.x40)); HM((u32) stage_info.grkind);
}

int main(void)
{
    int op, k;
    setbuf(__acrt_iob_func(1), 0);
    getenv_only = getenv("MTU_ONLY") != 0;
    preg(&game_camera, sizeof game_camera); preg(SUBJ, sizeof SUBJ); preg(CO, sizeof CO); preg(WO, sizeof WO); preg(GO, sizeof GO);
    preg(MTXA, sizeof MTXA); preg(BND, sizeof BND); preg(S2, sizeof S2); preg(V3, sizeof V3); preg(&cm_script, sizeof cm_script); preg(&cm_80453004, sizeof cm_80453004);
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
            } __except (_exception_code() == 0xE0A55E87ul) { HM(0xA55E87u); ASSN[op]++; ASLINE[op] = LASTAL; }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    if (getenv("MTU_STAT")) { for (op = 0; op < @NOPS@; op++) fprintf_stub(op); }
    return 0;
}
'''

R_HEAD = r'''//! Written by gen/mtu/mk_cm.py from the same table as gen/mtu/tu/cm.c.
//! tu::camera vs the C (camera.c, shipping contraction; regenerate the expected file with
//! `python rust/melee_world/gen/mtu/mk_cm.py run`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu as m;
use melee_world::tu::camera as cm;
use mtu_harness::*;
#[path = "mtu/cm_gen.rs"]
mod g;

static mut SUBJ: [CmSubject; 8] = unsafe { core::mem::zeroed() };
static mut CO: [HSD_CObj; 3] = unsafe { core::mem::zeroed() };
static mut WO: [HSD_WObj; 4] = unsafe { core::mem::zeroed() };
static mut GO: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };
static mut MTXA: [[f32; 4]; 4] = [[0.0; 4]; 4];
static mut BND: [CameraBounds; 2] = unsafe { core::mem::zeroed() };
static mut S2: [S32Vec2; 2] = unsafe { core::mem::zeroed() };
static mut V3: [Vec3; 3] = unsafe { core::mem::zeroed() };

fn rnn() -> f32 {
    const K: [f32; 4] = [1.0, 13.0, 100.0, 1000.0];
    let a = ((rnd() % 41) as i32 - 20) as f32 / 20.0;
    a * K[(rnd() % 4) as usize]
}
fn rng(lo: i32, hi: i32, div: i32) -> f32 {
    (lo + (rnd() % (hi - lo + 1) as u32) as i32) as f32 / div as f32
}
fn rpos() -> f32 {
    ((rnd() % 401) as i32 - 200) as f32 / 4.0
}
fn rposz() -> f32 {
    (rnd() % 600) as f32
}
unsafe fn fill_vec(v: *mut Vec3) {
    (*v).x = rpos(); (*v).y = rpos(); (*v).z = rpos();
}
unsafe fn fill_ts(t: *mut CameraTransformState) {
    fill_vec(&raw mut (*t).interest); fill_vec(&raw mut (*t).target_interest); fill_vec(&raw mut (*t).position); fill_vec(&raw mut (*t).target_position);
    (*t).position.z = rposz() + 50.0; (*t).target_position.z = rposz() + 50.0;
    (*t).fov = (15 + (rnd() % 60)) as f32; (*t).target_fov = (15 + (rnd() % 60)) as f32;
}

unsafe fn setup() {
    SUBJ = core::mem::zeroed(); CO = core::mem::zeroed(); WO = core::mem::zeroed(); GO = core::mem::zeroed(); MTXA = [[0.0; 4]; 4];
    BND = core::mem::zeroed(); S2 = core::mem::zeroed(); V3 = core::mem::zeroed();
    cm::ext::stage_info.grkind = (rnd() % 50) as _;
    cm::cm_804D6458 = null_mut(); cm::cm_804D645C = null_mut(); cm::cm_804D6460 = null_mut(); cm::cm_804D6464 = null_mut(); cm::cm_804D6468 = null_mut();
    cm::cm_largemap = core::mem::zeroed();
    g::fill_HSD_GObj(&mut GO[0]); g::fill_HSD_GObj(&mut GO[1]); g::fill_HSD_GObj(&mut GO[2]);
    for i in 0..3 {
        g::fill_HSD_CObj(&mut CO[i]);
        CO[i].eyepos = if rnd() & 1 != 0 { &raw mut WO[i] } else { null_mut() };
        CO[i].interest = if rnd() & 1 != 0 { &raw mut WO[i + 1] } else { null_mut() };
        CO[i].proj_mtx = null_mut();
        CO[i].flags = (fv() & 1) as _;
    }
    GO[0].hsd_obj = &raw mut CO[0] as *mut _; GO[1].hsd_obj = &raw mut CO[1] as *mut _; GO[2].hsd_obj = &raw mut CO[2] as *mut _;
    for i in 0..4 { WO[i].pos.x = rpos(); WO[i].pos.y = rpos(); WO[i].pos.z = rpos(); }
    for i in 0..16 { MTXA[i / 4][i % 4] = rnn(); }
    cm::Camera_Init(8);
    let n = rnd() % 6;
    for _ in 0..n {
        let s = cm::Camera_80029044((rnd() % 3) as i32);
        (*s).pos.x = rpos(); (*s).pos.y = rpos(); (*s).pos.z = 0.0; (*s).bone_pos.x = rpos(); (*s).bone_pos.y = rpos(); (*s).bone_pos.z = rpos();
        (*s).facing_dir = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
        (*s).state_timer = if rnd() % 3 == 0 { (rnd() % 700) as i16 } else { 0 };
        (*s).set_was_framed((rnd() & 1) as _); (*s).set_force_inactive((rnd() % 8 == 0) as _); (*s).set_on_ledge((rnd() & 1) as _);
        (*s).ext.h.x = rpos() * 0.1; (*s).ext.h.y = rpos() * 0.1; (*s).ext.v.x = rpos() * 0.1; (*s).ext.v.y = rpos() * 0.1; (*s).ext.v.z = rpos() * 0.1;
        (*s).target_ext.h.x = rpos() * 0.1; (*s).target_ext.h.y = rpos() * 0.1; (*s).target_ext.v.x = rpos() * 0.1; (*s).target_ext.v.y = rpos() * 0.1; (*s).target_ext.v.z = rpos() * 0.1;
    }
    let gc = &raw mut cm::game_camera;
    (*gc).mode = (rnd() % 9) as _;
    fill_ts(&raw mut (*gc).transform); fill_ts(&raw mut (*gc).transform_copy);
    (*gc).gobj = if rnd() % 4 != 0 { &raw mut GO[0] } else { null_mut() };
    (*gc).quake_gobj = if rnd() % 3 == 0 { &raw mut GO[2] } else { null_mut() };
    (*gc).translation.x = rpos() * 0.1; (*gc).translation.y = rpos() * 0.1;
    (*gc).quake_offset.x = rnn(); (*gc).quake_offset.y = rnn();
    (*gc).quake_scale = (rnd() % 5) as f32 * 0.5;
    for i in 0..5 { (*gc).quake_frames_left[i] = if rnd() % 3 == 0 { (rnd() % 30) as i32 } else { 0 }; }
    for i in 0..16 {
        (*gc).quakes[0][i].kind = if rnd() % 3 == 0 { (rnd() % 5) as _ } else { 0 };
        (*gc).quakes[1][i].kind = (rnd() % 5) as _;
        fill_vec(&raw mut (*gc).quakes[0][i].epicenter); fill_vec(&raw mut (*gc).quakes[1][i].epicenter);
    }
    (*gc).x2B0 = rpos(); (*gc).x2B4 = rpos() * 100.0; (*gc).x2B8 = if rnd() % 5 == 0 { 1000 + (rnd() % 5) as i16 } else { (rnd() % 20) as i16 };
    (*gc).x2BA = (rnd() % 5) as _; (*gc).x2BC = if rnd() % 3 == 0 { 1.0 } else { (rnd() % 7) as f32 * 0.5 }; (*gc).x2C0 = (rnd() % 400) as f32 - 50.0;
    (*gc).set_x398_b0((rnd() & 1) as _); (*gc).set_x398_b1((rnd() & 1) as _); (*gc).set_x398_b2((rnd() & 1) as _); (*gc).set_x398_b3((rnd() & 1) as _); (*gc).set_x398_b4((rnd() & 1) as _); (*gc).set_x398_b5((rnd() & 1) as _);
    (*gc).set_x399_b0_b1((rnd() & 3) as _); (*gc).set_x399_b2((rnd() & 1) as _); (*gc).set_x399_b3((rnd() & 1) as _); (*gc).set_x399_b4((rnd() & 1) as _); (*gc).set_x399_b5((rnd() & 1) as _); (*gc).set_x399_b6((rnd() & 1) as _); (*gc).set_x399_b7((rnd() & 1) as _);
    (*gc).set_x39A_b0((rnd() & 1) as _); (*gc).set_x39A_b1((rnd() & 1) as _); (*gc).set_x39A_b2((rnd() & 1) as _);
    (*gc).nearz = (rnd() % 10) as f32 * 0.1; (*gc).farz = (rnd() % 20000) as f32;
    (*gc).background_color.r = fv() as u8; (*gc).background_color.g = fv() as u8; (*gc).background_color.b = fv() as u8;
    cm::cm_804D6464 = if rnd() & 1 != 0 { &raw mut CO[2] } else { null_mut() };
    cm::cm_80453004.last_mode = (rnd() % 9) as _; cm::cm_80453004.ply_slot = (rnd() % 4) as _;
    fill_vec(&raw mut cm::cm_80453004.follow_int_pos); fill_vec(&raw mut cm::cm_80453004.follow_eye_pos); fill_vec(&raw mut cm::cm_80453004.free_int_pos); fill_vec(&raw mut cm::cm_80453004.free_eye_pos);
    cm::cm_80453004.follow_fov = rpos(); cm::cm_80453004.free_fov = rpos();
    cm::cm_script = core::mem::zeroed();
    let cs = &raw mut cm::cm_script;
    if rnd() % 3 == 0 {
        (*cs).state = (rnd() % 3) as _; (*cs).lift_bounds = (rnd() & 1) as _; (*cs).elapsed = (rnd() % 20) as _; (*cs).duration = (rnd() % 30) as _;
        (*cs).ease = (rnd() % 4) as _; (*cs).path_count = (rnd() % 5) as _; (*cs).path_frame = (rnd() % 12) as _;
        (*cs).follow_kind = if rnd() % 3 == 0 { 1 + (rnd() % 3) as i32 } else { 0 }; (*cs).follow_id = (rnd() % 8) as _;
        (*cs).shake_left = (rnd() % 8) as _; (*cs).shake_total = 1 + (rnd() % 8) as i32; (*cs).shake_tick = (rnd() % 100) as _;
        (*cs).shake_intensity = rnn(); (*cs).saved_roll = rnn(); (*cs).saved_far = (rnd() % 20000) as f32; (*cs).saved_up_vector = (rnd() & 1) as _;
        fill_vec(&raw mut (*cs).saved_up); fill_vec(&raw mut (*cs).follow_point); fill_vec(&raw mut (*cs).follow_offset); fill_vec(&raw mut (*cs).follow_eye_delta);
        fill_ts(&raw mut (*cs).normal);
        fill_vec(&raw mut (*cs).pose.eye); fill_vec(&raw mut (*cs).pose.interest); (*cs).pose.fov = rpos(); (*cs).pose.roll = rnn();
        fill_vec(&raw mut (*cs).from.eye); fill_vec(&raw mut (*cs).from.interest); (*cs).from.fov = rpos(); (*cs).from.roll = rnn();
        fill_vec(&raw mut (*cs).to.eye); fill_vec(&raw mut (*cs).to.interest); (*cs).to.fov = rpos(); (*cs).to.roll = rnn();
        for i in 0..5 {
            fill_vec(&raw mut (*cs).path[i].pose.eye); fill_vec(&raw mut (*cs).path[i].pose.interest); (*cs).path[i].pose.fov = rpos(); (*cs).path[i].pose.roll = rnn();
            (*cs).path[i].frame = (i as u32 * 3 + rnd() % 3) as i32;
        }
        (*cs).completion = (rnd() % 4) as _;
    }
    for i in 0..2 {
        BND[i].x_min = rpos(); BND[i].y_min = rpos(); BND[i].x_max = BND[i].x_min + (rnd() % 200) as f32; BND[i].y_max = BND[i].y_min + (rnd() % 150) as f32;
        BND[i].total_subjects = (rnd() % 5) as _; BND[i].z_pos = rposz();
    }
    cm::ext::gm_CStickSmashTargetTest = (rnd() & 1) as _;
    for i in 0..4 { g::fill_HSD_PadStatus(&mut cm::ext::HSD_PadCopyStatus[i]); }
    cm::ext::HSD_GObj_CameraKind = fv() as u8;
    fill_vec(&raw mut V3[0]);
}

unsafe fn hash_all() {
    g::hash_Camera(&cm::game_camera);
    g::hash_CameraDebugMode(&cm::cm_80453004);
    for i in 0..8 { g::hash_CmSubject(&SUBJ[i]); }
    for i in 0..3 { g::hash_HSD_CObj(&CO[i]); g::hash_HSD_GObj(&GO[i]); }
    for i in 0..4 { hm(WO[i].pos.x.to_bits()); hm(WO[i].pos.y.to_bits()); hm(WO[i].pos.z.to_bits()); }
    for i in 0..2 { g::hash_CameraBounds(&BND[i]); }
    for i in 0..2 { hm(S2[i].x as u32); hm(S2[i].y as u32); }
    for i in 0..3 { hm(V3[i].x.to_bits()); hm(V3[i].y.to_bits()); hm(V3[i].z.to_bits()); }
    hm(rh(&cm::cm_script));
    hm(pi(cm::cm_largemap.primary)); hm(pi(cm::cm_largemap.secondary)); hm(cm::cm_largemap.primary_far.to_bits()); hm(cm::cm_largemap.secondary_far.to_bits());
    hm(pi(cm::cm_804D6458)); hm(pi(cm::cm_804D645C)); hm(pi(cm::cm_804D6460)); hm(pi(cm::cm_804D6464)); hm(pi(cm::cm_804D6468));
    hm(cm::cm_803BCCA0.x40.to_bits()); hm(cm::ext::stage_info.grkind as u32);
}

#[test]
fn cm_matches_c() {
    unsafe {
        std::panic::set_hook(Box::new(|i| { let m = i.to_string(); if !(m.contains(": assertion failed") || m.contains("lineID")) { eprintln!("{}", m); } }));
        g::install();
        cm::ext::hook::OSReport = Some(|_| {});
        preg(&raw const cm::game_camera, core::mem::size_of::<Camera>());
        preg(&raw const SUBJ, core::mem::size_of_val(&SUBJ));
        preg(&raw const CO, core::mem::size_of_val(&CO));
        preg(&raw const WO, core::mem::size_of_val(&WO));
        preg(&raw const GO, core::mem::size_of_val(&GO));
        preg(&raw const MTXA, core::mem::size_of_val(&MTXA));
        preg(&raw const BND, core::mem::size_of_val(&BND));
        preg(&raw const S2, core::mem::size_of_val(&S2));
        preg(&raw const V3, core::mem::size_of_val(&V3));
        preg(&raw const cm::cm_script, core::mem::size_of_val(&cm::cm_script));
        preg(&raw const cm::cm_80453004, core::mem::size_of_val(&cm::cm_80453004));
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_cm.txt"));
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
                    if !(msg.contains(": assertion failed") || msg.contains("lineID") || msg.contains("__assert")) { std::panic::resume_unwind(e); }
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
    from specs_cm import SPECS
    import gen_tu
    gen_tu.build("cm", SPECS["cm"], False)
    cops = "\n".join(f"            case {i}: {{ {c} }} break;" for i, (n, c, r, k) in enumerate(OPS))
    rops = "\n".join(f"                    {i} => {{ {r} }}" for i, (n, c, r, k) in enumerate(OPS))
    c = C_HEAD.replace("@COPS@", cops)
    r = R_HEAD.replace("@ROPS@", rops)
    for a, b in {"@NOPS@": str(len(OPS)), "@CASES@": str(CASES)}.items():
        c = c.replace(a, b)
        r = r.replace(a, b)
    (HERE / "tu" / "cm.c").write_text(c, encoding="utf-8", newline="\n")
    (ROOT / "rust/melee_world/tests/tu_cm.rs").write_text(r, encoding="utf-8", newline="\n")
    print(f"mk_cm: {len(OPS)} ops x {CASES} cases")


LINK = ["pc/gameworld/gekko_fp.c", "src/melee/lb/lbtrigf.c", "src/melee/lb/lbvector.c", "pc/gameworld/mtx_pc.c"]


def run():
    import run_tu
    BUILD = run_tu.BUILD
    BUILD.mkdir(parents=True, exist_ok=True)
    from specs_cm import SPECS
    import gen_tu
    spec = SPECS["cm"]
    src = gen_tu.patched_source("cm", spec)
    drv = HERE / "tu" / "cm.c"
    exe = BUILD / "mtu_cm.exe"
    lls = []
    dbg = []
    if os.environ.get("MTU_DBG"):
        dop, dk = os.environ["MTU_DBG"].split(":")
        dbg = [f"-DDBG_OP={dop}", f"-DDBG_K={dk}"]
    else:
        dbg = ["-DDBG_OP=-1", "-DDBG_K=-1"]
    for s in [drv] + [ROOT / x for x in LINK]:
        ll = BUILD / f"mtu_cm__{s.stem}.ll"
        lo = BUILD / f"mtu_cm__{s.stem}.lo.ll"
        run_tu.run([run_tu.CLANG, *run_tu.FLAGS, "-fms-extensions", "-I", str((ROOT / spec["src"]).parent), "-I", str(HERE / "tu"),
                    '-DTU_SRC="' + str(ROOT / src).replace("\\", "/") + '"', *dbg, *(["-DlbVector_WorldToScreen=lbVector_WorldToScreen_unused"] if s.stem == "lbvector" else []), str(s), "-o", str(ll)], "cm")
        run_tu.run([sys.executable, str(run_tu.LOWER), str(ll), str(lo)], "cm")
        lls.append(str(lo))
    run_tu.run([run_tu.CLANG, "-O0", "-w", *lls, "-o", str(exe), "-Wl,/OPT:NOICF", "-Wl,/FORCE:UNRESOLVED", "-fuse-ld=lld"], "cm")
    r_ = subprocess.run([str(exe)], capture_output=True, text=True)
    out = r_.stdout
    if os.environ.get("MTU_STAT"):
        print(r_.stderr)
    if os.environ.get("MTU_DBG"):
        tr = [l for l in out.splitlines() if l.startswith(("T ", "H ", "B ", "ENT", "DBG"))]
        (BUILD / "dbg_cm.txt").write_text(chr(10).join(tr) + chr(10), encoding="utf-8")
        print(f"mk_cm: {len(tr)} trace words -> {BUILD / 'dbg_cm.txt'}")
        return
    if r_.returncode:
        print(f"mk_cm: exe exit code {r_.returncode} after {len(out.splitlines())} lines")
    dd = ROOT / "rust/melee_world/tests/data"
    (dd / "mtu_cm.txt").write_text(out, encoding="utf-8", newline="\n")
    print(f"mk_cm: {len(out.splitlines())} lines")


if __name__ == "__main__":
    mode = sys.argv[1]
    if mode in ("gen", "all"):
        gen()
    if mode in ("run", "all"):
        run()
