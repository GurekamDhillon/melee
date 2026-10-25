/* TU: src/melee/cm/camera.c */
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
    for (op = 0; op < 34; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            if (getenv_only && !(op == DBG_OP && k == DBG_K)) continue;
            tu_case(op, k);
            mtu_dbgp = (op == DBG_OP && k == DBG_K);
            setup();
            __try {
            switch (op) {
            case 0: { Camera_Init(1 + (int) (rnd() % 8)); } break;
            case 1: { Camera_Create(); } break;
            case 2: { Camera_ForgetGameCamera(); } break;
            case 3: { Camera_RefreshViewingMtx(); } break;
            case 4: { ret = (u32) Camera_ScriptCompletion(); } break;
            case 5: { ret = (u32) Camera_8002928C((&SUBJ[rnd() % 8])); } break;
            case 6: { Camera_8002A4AC(&GO[0]); } break;
            case 7: { { CameraTransformState* ts = ((rnd() & 1) ? &game_camera.transform : &game_camera.transform_copy); Camera_8002A768(ts, (int) fv()); } } break;
            case 8: { { CameraTransformState* ts = ((rnd() & 1) ? &game_camera.transform : &game_camera.transform_copy); Camera_8002AF68(&CO[rnd() % 3], ts); } } break;
            case 9: { Camera_8002B0E0(); } break;
            case 10: { { CameraTransformState* ts = ((rnd() & 1) ? &game_camera.transform : &game_camera.transform_copy); Camera_8002B1F8(ts); } } break;
            case 11: { Camera_8002B3D4(0); } break;
            case 12: { Camera_8002F3AC(); } break;
            case 13: { { CameraTransformState* ts = ((rnd() & 1) ? &game_camera.transform : &game_camera.transform_copy); Camera_ApplyQuake(&BND[0], ts); } } break;
            case 14: { if (!game_camera.gobj) game_camera.gobj = &GO[0]; ret = (u32) Camera_LogicToScreen((&SUBJ[rnd() % 8]), &S2[0]); } break;
            case 15: { ret = PI(Camera_PCMainCObj()); } break;
            case 16: { ret = (u32) Camera_ProjectToScreen(&CO[rnd() % 3], &V3[0], &S2[0]); } break;
            case 17: { ret = (u32) Camera_ProjectToScreen(&CO[rnd() % 3], &V3[0], 0); } break;
            case 18: { { u8 r = (u8) fv(); u8 g = (u8) fv(); u8 b = (u8) fv(); Camera_SetBackgroundColor(r, g, b); } } break;
            case 19: { Camera_SetQuakeScale(rw()); } break;
            case 20: { Camera_UpdateQuakes(&BND[0]); } break;
            case 21: { cm_script_update(); } break;
            case 22: { fn_8002F360(&GO[0]); } break;
            case 23: { fn_800301D0(&GO[0], (int) fv()); } break;
            case 24: { update_bounds(&BND[0], &BND[1]); } break;
            case 25: { { CameraTransformState* ts = ((rnd() & 1) ? &game_camera.transform : &game_camera.transform_copy); update_transform(&BND[0], ts); } } break;
            case 26: { { float x = rw(); float y = rw(); Camera_SetQuakeOffset(x, y); } } break;
            case 27: { if (!game_camera.gobj) game_camera.gobj = &GO[0]; ret = (u32) Camera_80030BBC(&V3[0], &S2[0]); } break;
            case 28: { if (!game_camera.gobj) game_camera.gobj = &GO[0]; ret = (u32) Camera_80030BBC(&V3[0], 0); } break;
            case 29: { { int k = (int) (rnd() % 4) + 1; Camera_RequestQuake(k, (rnd() & 1) ? &V3[0] : 0); } } break;
            case 30: { ret = (u32) Camera_ScriptDetach(); } break;
            case 31: { Camera_ScriptAttach((int) (rnd() % 6) - 1); } break;
            case 32: { Camera_ScriptCommit(); } break;
            case 33: { Camera_800293E0(); } break;
            }
            } __except (_exception_code() == 0xE0A55E87ul) { HM(0xA55E87u); ASSN[op]++; ASLINE[op] = LASTAL; }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    if (getenv("MTU_STAT")) { for (op = 0; op < 34; op++) fprintf_stub(op); }
    return 0;
}
