/* C reference for melee_world::cm::camera_boss (the boss-intro camera script entry points of
 * src/melee/cm/camera.c: Camera_8002E6FC .. Camera_8002F274 with Camera_8002DFE4 / Camera_8002E234).
 * The functions are cut verbatim from camera.c into camera_boss_extract.inc by gen_camera_boss_ref.py
 * (lb/lbvector.c is included whole, lb/lbtrigf.c's atan2f cut as in the fighter references); built with
 * the shipping contraction. Player_GetEntity / ftLib_GetCameraSubject are stand-ins over random subjects.
 * One camera with random fields, one op per case; each line is op, case, return, hash of the state. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include <melee/cm/types.h>
#include <melee/cm/camera.h>
#include <melee/lb/lbvector.h>
#include <sysdolphin/baselib/gobj.h>

#include <Runtime/platform.h>

int printf(const char*, ...);
int atoi(const char*);

float MSL_TrigF_80400770[1];
float MSL_TrigF_80400774[1];
float lbRefract3_804D7DD4;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }

Camera game_camera;

static HSD_GObj GOBJS[6];
static HSD_GObj* pg[6];
static CmSubject SUBJ[6];
static CmSubject* ps[6];
HSD_GObj* Player_GetEntity(s32 slot) { return slot >= 0 && slot < 6 ? pg[slot] : NULL; }
CmSubject* ftLib_GetCameraSubject(HSD_GObj* g) { return ps[g - GOBJS]; }

static Vec3 cbv[2];
static int cbret[2];
static u32 cblog;
static int cb_a(Vec3* v) { *v = cbv[0]; cblog = cblog * 31u + 1u; return cbret[0]; }
static int cb_b(Vec3* v) { *v = cbv[1]; cblog = cblog * 31u + 2u; return cbret[1]; }

/* lb/lbvector.c and lb/lbtrigf.c (atan2f) come in whole / cut */
#include "lb/lbvector.c"
#include "camera_boss_extract.inc"

static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return 1.0f;
    return (float) ((int) (rnd() % 2001) - 1000) / 37.0f;
}
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }
static Vec3 rv(void) { Vec3 v; v.x = rf(); v.y = rf(); v.z = rf(); return v; }

static int pr;
static u32 state(void)
{
    u32 h = 0;
    Camera* c = &game_camera;
#define H(v) (h = h * 31u + (u32) (v), pr ? (void) printf(" %08X", (u32) (v)) : (void) 0)
#define HV(v) (H(B((v).x)), H(B((v).y)), H(B((v).z)))
    H(c->mode);
    HV(c->transform.interest); HV(c->transform.target_interest); HV(c->transform.position); HV(c->transform.target_position);
    H(B(c->transform.fov)); H(B(c->transform.target_fov));
    H(c->x341_b1_b2); H(c->x341_b3_b4); H(c->x341_b5_b6); H(c->x341_b7);
    if (c->x341_b1_b2 == 1) H(c->x344.slot);
    else if (c->x341_b1_b2 == 2) HV(c->x344.vec);
    else if (c->x341_b1_b2 == 3) H(c->x344.cb == cb_a ? 1 : c->x344.cb == NULL ? 0 : 2);
    HV(c->x350);
    if (c->x341_b3_b4 == 2) {
        H(c->x35C.bits.b0); H(c->x35C.bits.b1); H(c->x35C.bits.b2);
        /* the axis values are only meaningful while their flag is set (the rest is stale pointer bytes) */
        if (c->x35C.bits.b0) H(c->x35C.bits.x2);
        if (c->x35C.bits.b1) H(B(c->x35C.vec.y));
        if (c->x35C.bits.b2) H(B(c->x35C.vec.z));
    } else if (c->x341_b3_b4 == 3) {
        H(c->x35C.cb == cb_b ? 1 : c->x35C.cb == NULL ? 0 : 2);
    } else {
        /* x35C.vec.x aliases the bitfield bytes (laid out differently natively than on the GC) */
        if (c->x341_b3_b4 == 1) { H(B(c->x35C.vec.y)); H(B(c->x35C.vec.z)); } /* mode 0: stale (maybe pointer) bytes */
    }
    H(*(s16*) &c->x368); H(B(c->x368.y)); H(B(c->x368.z));
    if (c->x341_b3_b4 != 2) H(B(c->x368.x));
    H(B(c->x374));
    H(c->x378.s32_v); H(c->x37C.s32_v);
    H(cblog);
#undef HV
#undef H
    return h;
}

#define NOPS 11
int main(int argc, char** argv)
{
    int op, k, i;
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < 500; k++) {
            if (argc > 2 && !(op == atoi(argv[1]) && k == atoi(argv[2]))) continue;
            pr = argc > 2;
            u32 ret = 0;
            Camera* c = &game_camera;
            zero(c, sizeof *c);
            rs = 0xCA3u + (u32) op * 7919u + (u32) k * 104729u;
            cblog = 0;
            c->mode = (rnd() % 3 == 0) ? CAMERA_BOSS_INTRO : (CameraType) (rnd() % 9);
            c->transform.interest = rv(); c->transform.target_interest = rv();
            c->transform.position = rv(); c->transform.target_position = rv();
            c->transform.fov = rf(); c->transform.target_fov = rf();
            c->x341_b1_b2 = rnd() % 4; c->x341_b3_b4 = rnd() % 4; c->x341_b5_b6 = rnd() % 4; c->x341_b7 = rnd() & 1;
            if (c->x341_b1_b2 == 3) { c->x344.cb = cb_a; }
            else if (c->x341_b1_b2 == 1) { c->x344.slot = (s32) (rnd() % 6); }
            else { c->x344.vec = rv(); }
            c->x350 = rv();
            if (c->x341_b3_b4 == 2) {
                c->x35C.bits.b0 = rnd() & 1; c->x35C.bits.b1 = rnd() & 1; c->x35C.bits.b2 = rnd() & 1;
                c->x35C.bits.x2 = (s16) rnd(); c->x35C.vec.y = rf(); c->x35C.vec.z = rf();
            } else if (c->x341_b3_b4 == 3) { c->x35C.cb = cb_b; }
            else { c->x35C.vec = rv(); }
            c->x368 = rv();
            c->x374 = rf();
            if (c->x341_b5_b6 == 1) { c->x378.s32_v = (s32) (rnd() % 20); c->x37C.s32_v = (rnd() % 4 == 0) ? c->x378.s32_v : (s32) (rnd() % 20); }
            else { c->x378.f32_v = rf(); c->x37C.f32_v = rf(); }
            for (i = 0; i < 6; i++) {
                u32 m = rnd() % 3;
                pg[i] = m ? &GOBJS[i] : NULL;
                ps[i] = (m == 2) ? &SUBJ[i] : NULL;
                SUBJ[i].bone_pos = rv();
            }
            cbv[0] = rv(); cbv[1] = rv(); cbret[0] = (int) (rnd() & 1); cbret[1] = (int) (rnd() & 1);
            if (pr) printf("FILL mode=%d b1=%d b3=%d b5=%d b7=%d y=%08X z=%08X x=%08X\n", c->mode, c->x341_b1_b2, c->x341_b3_b4, c->x341_b5_b6, c->x341_b7, B(c->x35C.vec.y), B(c->x35C.vec.z), B(c->x35C.vec.x));
            switch (op) {
            case 0: Camera_8002E6FC((int) (rnd() % 6)); break;
            case 1: Camera_8002E948((bool (*)(Vec*)) cb_a); break;
            case 2: Camera_8002EB5C(rf()); break;
            case 3: Camera_8002EC7C(rf()); break;
            case 4: Camera_8002ED9C(rf()); break;
            case 5: Camera_8002EEC8(rf()); break;
            case 6: Camera_8002EF14(); break;
            case 7: Camera_8002F0E4((s32) (rnd() % 30)); break;
            case 8: ret = (u32) Camera_8002F260(); break;
            case 9: Camera_8002F274(); break;
            case 10: ret = (u32) Camera_8002E234(); break;
            }
            if (pr) printf("DUMP");
            { u32 st = state(); if (pr) printf("\n"); printf("%d %d %08X %08X\n", op, k, ret, st); }
        }
    }
    return 0;
}
