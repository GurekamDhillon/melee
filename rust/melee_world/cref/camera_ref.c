/* C reference for melee_world::cm::camera. Built by gen_camera_ref.sh, which cuts the real
 * struct definitions out of src/melee/cm/types.h (cm_types.inc) and the function bodies out of
 * src/melee/cm/camera.c (camera_body.inc). Everything else here is a minimal stand-in for the
 * headers and for the stage / GObj / grlib calls the bodies make. Prints one IN line per case
 * (all inputs as hex) followed by its OUT lines; tests/camera_vs_c.rs replays the IN lines. */
#include <setjmp.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define TARGET_PC 1
typedef int8_t s8;
typedef uint8_t u8;
typedef int16_t s16;
typedef uint16_t u16;
typedef int32_t s32;
typedef uint32_t u32;
typedef int64_t s64;
typedef uint64_t u64;
typedef float f32;
typedef int bool;
typedef int enum_t;
#define true 1
#define false 0
#define UNUSED
#define PAD_STACK(n)
#define F32_MAX 3.402823466e+38F
#define MTXDegToRad(a) ((a) * 0.01745329252f)
float tanf(float);

static jmp_buf assert_jmp;
#define HSD_ASSERT(line, c) ((c) ? (void) 0 : (void) (printf("ASSERT %d\n", line), longjmp(assert_jmp, 1)))

typedef struct { f32 x, y; } Vec2;
typedef struct { f32 x, y, z; } Vec3;
typedef Vec3 Vec;
typedef struct { u8 r, g, b, a; } GXColor;
typedef struct HSD_GObj HSD_GObj;
typedef struct HSD_CObj HSD_CObj;
typedef u32 CmSubjectState;
enum { CmSubjectState_Active, CmSubjectState_Inactive, CmSubjectState_Auto };
typedef u32 CmQuakeKind;
enum { QuakeKind_None, QuakeKind_Loop, QuakeKind_Small, QuakeKind_Medium, QuakeKind_Large,
       QuakeKind_Count };
typedef u32 CameraType;
enum { CAMERA_STANDARD, CAMERA_PAUSE, CAMERA_TRAINING_MENU, CAMERA_CLEAR, CAMERA_FIXED,
       CAMERA_FREE, CAMERA_BOSS_INTRO, CAMERA_DEBUG_FOLLOW, CAMERA_DEBUG_FREE };
enum { CAM_BOUNDS_INSIDE = 0, CAM_BOUNDS_OUTSIDE_TOP = 1, CAM_BOUNDS_OUTSIDE_BOTTOM = 2,
       CAM_BOUNDS_OUTSIDE_LEFT = 4, CAM_BOUNDS_OUTSIDE_RIGHT = 8 };
typedef struct CmSubject CmSubject;
typedef struct CmSubjectExtents { Vec2 h; Vec3 v; } CmSubjectExtents;
typedef struct CameraTransformState CameraTransformState;
typedef struct CameraBounds CameraBounds;
typedef struct CameraUnkGlobals CameraUnkGlobals;
typedef struct Camera_x2D0 Camera_x2D0;
typedef struct CameraQuake CameraQuake;
#include "cm_types.inc"
struct CameraQuake {
    Vec3 epicenter;
    CmQuakeKind kind;
};

/* game_camera: only the members the cut bodies use (names as in cm/types.h). */
typedef struct Camera {
    HSD_GObj* gobj;
    CameraType mode;
    GXColor background_color;
    f32 nearz, farz;
    CameraTransformState transform;
    Vec2 translation;
    s32 quake_frames_left[5];
    HSD_GObj* quake_gobj;
    Vec2 quake_offset;
    f32 quake_scale;
    CameraQuake quakes[2][16];
    f32 x2B0;
    s16 x2B8;
    f32 x2BC;
    u8 x341_b0 : 1, x341_b1_b2 : 2, x341_b3_b4 : 2, x341_b5_b6 : 2, x341_b7 : 1;
    union { f32 f32_v; s32 s32_v; } x378, x37C;
    u8 x398_b0 : 1, x398_b1 : 1, x398_b2 : 1, x398_b3 : 1, x398_b4 : 1, x398_b5 : 1,
        x398_b6_b7 : 2;
    u8 x399_b0_b1 : 2;
} Camera;
static Camera game_camera;
static struct { CameraType last_mode; } cm_80453004;

typedef struct { void* callback[9]; } CameraModeCallbacks;
typedef struct { void* a; Vec3 pos; void* b; } HSD_WObjDesc;
typedef struct {
    u32 pad[2];
    struct { s16 xmin, xmax, ymin, ymax; } viewport;
    f32 aspect;
} HSD_CameraDescPerspective;
static CameraModeCallbacks cm_803BCB18;
static HSD_CameraDescPerspective cm_803BCB64 = { { 0 }, { 0, 0x280, 0, 0x1E0 }, 1.2173333f };
static CameraUnkGlobals cm_803BCCA0 = {
    83.0f,  1000.0f, -30.0f,  5.0f,  -7.0f,  17.5f,  -17.5f, 0.0f,  0.0682f,
    60.0f,  120.0f,  0.05f,   0.1f,  120.0f, 900.0f, 0.15f,  38.0f, 0.1f,
    0.1f,   0.001f,  0.1f,    1.0f,  1.0f,   0.6f,   0.6f,   0.05f, 0.1f,
    29.0f,  0.1f,    0.1f,    0.1f,  0.1f,   0.5f,   0.5f,   0.4f,  -11.0f,
    400.0f, 0.2f,    4.0f,    0.05f, 1.0f,   -7.0f,  7.0f,   0.5f,  0.5f,
    0.004f, 0.2f,    0.025f,  0.2f,  0.003f, 0.2f,   0.025f, 0.2f,  0.02f,
    1.0f,   0.14f,   1200.0f, -0.2f, 1.2f,   0.0f,
};
static float cm_803BCB9C[] = { 0.0f, 1.5f, 1.32f, 1.16f, 1.0f };
static CmSubject* cm_804D6468;
static HSD_CObj* cm_804D6464;

/* ---- stage / ground getters (gr/stage.c, gr/ground.c) as plain inputs ---- */
static f32 st[16]; /* left right top bottom track zoom maxd x20 x24 pan tcx tcy tcz ground */
static f32 Stage_GetCamBoundsLeftOffset(void) { return st[0]; }
static f32 Stage_GetCamBoundsRightOffset(void) { return st[1]; }
static f32 Stage_GetCamBoundsTopOffset(void) { return st[2]; }
static f32 Stage_GetCamBoundsBottomOffset(void) { return st[3]; }
static f32 Stage_GetCamTrackRatio(void) { return st[4]; }
static f32 Stage_GetCamZoomRate(void) { return st[5]; }
static f32 Stage_GetCamMaxDepth(void) { return st[6]; }
static f32 Stage_GetCamInfoX20(void) { return st[7]; }
static f32 Stage_GetCamInfoX24(void) { return st[8]; }
static f32 Stage_GetCamPanAngleRadians(void) { return st[9]; }
static void Stage_UnkSetVec3TCam_Offset(Vec3* v) { v->x = st[10]; v->y = st[11]; v->z = st[12]; }
static void Ground_801C4368(f32* a, f32* b) { *a = st[13]; *b = 12345.0f; }

static int fly_any, fly_subj[8], one_p;
static struct { int state, lift_bounds; } cm_script;
static CmSubject subj[8];
int GenoFly_Any(void) { return fly_any; }
int GenoFly_CameraSubject(void* s) { return fly_subj[(CmSubject*) s - subj]; }
static bool gm_IsCurrently1PMode_inline(void) { return one_p; }
static int Mex_Enabled(const char* s) { (void) s; return 0; }
static void diag_cam_translate(int a, int b, int c, int d, float e, float f, float g, float h,
                               float i, float j) {}
static int gobj_frees, grlib_calls;
static HSD_GObj* const QUAKE_TOKEN = (HSD_GObj*) 0x1000;
static void HSD_GObjFree(HSD_GObj* g) { (void) g; gobj_frees++; }
static HSD_GObj* grLib_801C9CEC(int kind) { (void) kind; grlib_calls++; return QUAKE_TOKEN; }
#define GET_COBJ(g) ((HSD_CObj*) (g))
static void HSD_CObjSetMtxDirty(HSD_CObj* c) {}
static void HSD_CObjSetupViewingMtx(HSD_CObj* c) {}

void Camera_80030DE4(f32, f32);
static void Camera_80029AAC(CameraBounds*, CameraTransformState*, f32);
#include "camera_body.inc"

/* ---- harness ---- */
static u32 rng = 0x13579bdfu;
static u32 rnd(void) { rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5; return rng; }
static u32 rn(u32 n) { return rnd() % n; }
static f32 fr(f32 lo, f32 hi) { return lo + (hi - lo) * ((rnd() >> 8) * (1.0f / 16777216.0f)); }
static u32 fb(f32 f) { u32 u; memcpy(&u, &f, 4); return u; }
static void pf(f32 f) { printf(" %08x", fb(f)); }
static void pv(Vec3 v) { pf(v.x); pf(v.y); pf(v.z); }
static void pt(const CameraTransformState* t)
{
    pv(t->interest); pv(t->target_interest); pv(t->position); pv(t->target_position);
    pf(t->fov); pf(t->target_fov);
}
static void pb(const CameraBounds* b)
{
    pf(b->x_min); pf(b->y_min); pf(b->x_max); pf(b->y_max); printf(" %d", b->total_subjects);
    pf(b->z_pos);
}
static void psubj(int n)
{
    int i;
    printf("S");
    for (i = 0; i < n; i++) {
        CmSubject* s = &subj[i];
        printf(" %d %d", s->state_timer, s->was_framed);
        pf(s->ext.h.x); pf(s->ext.h.y); pf(s->ext.v.x); pf(s->ext.v.y); pf(s->ext.v.z);
    }
    printf("\n");
}
static f32 ext_f(void) { return rn(8) == 0 ? 0.0f : fr(-60.0f, 60.0f); }

static void case_frame(int k)
{
    int n, i;
    CameraBounds b, b2;
    CameraTransformState t;
    f32 speed, scale;
    st[0] = fr(-320.0f, -40.0f); st[1] = fr(40.0f, 320.0f);
    st[3] = fr(-220.0f, -10.0f); st[2] = fr(40.0f, 320.0f);
    st[4] = fr(0.3f, 1.6f); st[5] = fr(40.0f, 220.0f);
    st[6] = rn(10) == 0 ? st[5] + fr(0.0f, 0.0009f) : st[5] + fr(0.0f, 600.0f);
    st[7] = fr(0.0f, 0.35f); st[8] = fr(0.0f, 0.35f); st[9] = fr(-0.25f, 0.25f);
    st[10] = fr(-60.0f, 60.0f); st[11] = fr(-60.0f, 60.0f); st[12] = fr(-60.0f, 60.0f);
    st[13] = fr(-250.0f, 60.0f);
    fly_any = rn(10) == 0;
    cm_script.state = rn(10) == 0; cm_script.lift_bounds = rn(2);
    one_p = rn(2);
    n = rn(7);
    memset(subj, 0, sizeof subj);
    for (i = 0; i < n; i++) {
        CmSubject* s = &subj[i];
        s->state = rn(3);
        s->force_inactive = rn(8) == 0;
        s->was_framed = rn(2);
        s->state_timer = rn(3) == 0 ? (s16) rn(3) : 0;
        s->pos.x = fr(st[0] * 1.6f, st[1] * 1.6f);
        s->pos.y = fr(st[3] * 1.6f, st[2] * 1.6f);
        s->ext.h.x = ext_f(); s->ext.h.y = ext_f(); s->ext.v.x = ext_f(); s->ext.v.y = ext_f();
        s->ext.v.z = ext_f();
        s->target_ext.h.x = rn(4) ? s->ext.h.x + fr(-2.0f, 2.0f) : s->ext.h.x;
        s->target_ext.h.y = s->ext.h.y + fr(-2.0f, 2.0f);
        s->target_ext.v.x = s->ext.v.x + fr(-0.6f, 0.6f);
        s->target_ext.v.y = s->ext.v.y + fr(-2.0f, 2.0f);
        s->target_ext.v.z = s->ext.v.z + fr(-2.0f, 2.0f);
        s->prev = i + 1 < n ? &subj[i + 1] : NULL;
        fly_subj[i] = rn(2);
    }
    cm_804D6468 = n ? &subj[0] : NULL;
    t.interest.x = fr(-200, 200); t.interest.y = fr(-200, 200); t.interest.z = fr(-20, 20);
    t.target_interest.x = fr(-200, 200); t.target_interest.y = fr(-200, 200);
    t.target_interest.z = fr(-20, 20);
    t.position.x = fr(-200, 200); t.position.y = fr(-200, 200);
    t.position.z = rn(4) == 0 ? fr(-100.0f, 100.0f) : fr(-6000.0f, 6000.0f);
    t.target_position.x = fr(-200, 200); t.target_position.y = fr(-200, 200);
    t.target_position.z = fr(0, 3000);
    t.fov = fr(10.0f, 70.0f); t.target_fov = fr(10.0f, 70.0f);
    game_camera.x2BC = rn(8) == 0 ? fr(0.0f, 0.0002f) : fr(0.0f, 2.0f);
    game_camera.quake_offset.x = fr(-2, 2); game_camera.quake_offset.y = fr(-2, 2);
    game_camera.quake_scale = fr(0, 2);
    speed = fr(0.0f, 3.0f); scale = fr(0.0f, 30.0f);

    printf("IN F %d %d %d %d %d %d", k, n, fly_any, cm_script.state && cm_script.lift_bounds, one_p, 0);
    for (i = 0; i < 14; i++) pf(st[i]);
    for (i = 0; i < n; i++) {
        CmSubject* s = &subj[i];
        printf(" %u %d %d %d %d", s->state, s->force_inactive, s->was_framed, s->state_timer, fly_subj[i]);
        pf(s->pos.x); pf(s->pos.y);
        pf(s->ext.h.x); pf(s->ext.h.y); pf(s->ext.v.x); pf(s->ext.v.y); pf(s->ext.v.z);
        pf(s->target_ext.h.x); pf(s->target_ext.h.y); pf(s->target_ext.v.x);
        pf(s->target_ext.v.y); pf(s->target_ext.v.z);
    }
    pt(&t);
    pf(game_camera.x2BC); pf(game_camera.quake_offset.x); pf(game_camera.quake_offset.y);
    pf(game_camera.quake_scale); pf(speed); pf(scale);
    printf("\n");

    for (i = 0; i < n; i++) {
        Vec3 p = subj[i].pos;
        int d = (int) rn(81) - 40;
        f32 df = fr(-40.0f, 40.0f);
        printf("B %u %d %d %d", Camera_80029124(&p, d), Camera_80031154(&p),
               Camera_8003118C(&p, df), d);
        pf(df);
        printf("\n");
    }
    Camera_800293E0();
    psubj(n);
    memset(&b, 0, sizeof b);
    Camera_8002958C(&b, &t);
    psubj(n);
    printf("R"); pb(&b); printf("\n");
    Camera_80029AAC(&b, &t, speed);
    b2 = b;
    Camera_80029BC4(&b2, &t);
    printf("T"); pt(&t); pb(&b2); printf("\n");
    Camera_80029C88(&b, &t, scale);
    printf("T"); pt(&t); printf("\n");
    if (setjmp(assert_jmp) == 0) {
        Camera_80029CF8(&b2, &t);
        printf("T"); pt(&t); printf("\n");
    }
    Camera_ApplyQuake(&b2, &t);
    printf("Q"); pf(game_camera.translation.x); pf(game_camera.translation.y);
    pf(game_camera.quake_offset.x); pf(game_camera.quake_offset.y); printf("\n");
}

static void case_lerp(void)
{
    int mode = rn(4);
    f32 a = fr(-500, 500), b = fr(-500, 500);
    f32 out = 0;
    bool r;
    game_camera.x341_b5_b6 = mode;
    if (mode == 1) {
        game_camera.x378.s32_v = (s32) rn(40) - 5;
        game_camera.x37C.s32_v = rn(3) == 0 ? game_camera.x378.s32_v : (s32) rn(40) - 5;
    } else {
        game_camera.x378.f32_v = fr(-0.5f, 1.5f);
        game_camera.x37C.s32_v = (s32) rnd();
    }
    printf("IN E %d %08x %08x", mode, (u32) game_camera.x378.s32_v, (u32) game_camera.x37C.s32_v);
    pf(a); pf(b); printf("\n");
    r = Camera_8002E158(&out, a, b);
    printf("E %d", r); pf(out); printf("\n");
}

static void case_quake(void)
{
    int i, j, ops = 1 + rn(20);
    memset(&game_camera.quake_frames_left, 0, sizeof game_camera.quake_frames_left);
    memset(&game_camera.quakes, 0, sizeof game_camera.quakes);
    game_camera.quake_gobj = NULL;
    gobj_frees = grlib_calls = 0;
    printf("IN U %d\n", ops);
    for (i = 0; i < ops; i++) {
        int op = rn(4);
        if (op == 0) {
            Camera_UpdateQuakes(NULL);
            printf("O U\n");
        } else if (op == 1) {
            int k = rn(5);
            Camera_StopQuake(k);
            printf("O S %d\n", k);
        } else {
            int k = rn(5);
            if (rn(2)) {
                Vec3 p;
                p.x = fr(-100, 100); p.y = fr(-100, 100); p.z = fr(-100, 100);
                printf("O R %d 1", k); pv(p); printf("\n");
                Camera_RequestQuake(k, &p);
            } else {
                printf("O R %d 0\n", k);
                Camera_RequestQuake(k, NULL);
            }
        }
        printf("Z %d %d %d", gobj_frees, grlib_calls, game_camera.quake_gobj != NULL);
        for (j = 0; j < 5; j++) printf(" %d", game_camera.quake_frames_left[j]);
        for (j = 0; j < 32; j++) {
            CameraQuake* q = &game_camera.quakes[j / 16][j % 16];
            printf(" %u", q->kind); pv(q->epicenter);
        }
        printf("\n");
    }
}

int main(void)
{
    int k;
    for (k = 0; k < 3000; k++) case_frame(k);
    for (k = 0; k < 2000; k++) case_lerp();
    for (k = 0; k < 300; k++) case_quake();
    return 0;
}
