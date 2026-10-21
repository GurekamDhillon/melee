/* Game-side TU for the camera accessor oracle (gen_camera_acc_ref.sh). Compiled with the game's
 * flags (contraction on, then lower.py) together with lbvector.c / mtx_pc.c. camera_acc_body.inc
 * holds the bodies cut verbatim from src/melee/cm/camera.c; game_camera, cm_803BCCA0 and
 * cm_80453004 are stand-ins with only the members those bodies use. */
#include <Runtime/platform.h>
#include <dolphin/mtx.h>
#include "lb/lbvector.h"
#include "camera_acc.h"

typedef enum { CAMERA_STANDARD, CAMERA_PAUSE, CAMERA_TRAINING_MENU, CAMERA_CLEAR, CAMERA_FIXED,
               CAMERA_FREE, CAMERA_BOSS_INTRO, CAMERA_DEBUG_FOLLOW, CAMERA_DEBUG_FREE } CameraTypeE;
typedef u32 CameraType;
typedef s32 enum_t;
typedef struct { Vec3 interest, target_interest, position, target_position; f32 fov, target_fov; }
    CameraTransformState;
typedef struct { u8 r, g, b, a; } AccColor;
typedef struct { f32 x, y; } AccVec2;
typedef struct Camera { CAM_ACC_MEMBERS(Vec3, CameraTransformState, AccColor, AccVec2) } Camera;
typedef AccColor GXColor_acc;
#define GXColor GXColor_acc
Camera game_camera;
struct { f32 x[16]; f32 x40; } cm_803BCCA0;
struct { CameraType last_mode; } cm_80453004;

#include "camera_acc_body.inc"

int acc_op(int op, int iarg, const float* fin, float* fout)
{
    Vec3 a, b, c;
    switch (op) {
    case 0: return Camera_8002F260();
    case 1: Camera_SetModeToStandard(); return 0;
    case 2: Camera_SetModeToFixed(); return 0;
    case 3: Camera_800300F0(); return 0;
    case 4: return Camera_8003010C();
    case 5: return Camera_80030130();
    case 6: return Camera_80030154();
    case 7: return Camera_80030178();
    case 8: Camera_80030730(fin[0]); return 0;
    case 9: Camera_SetBackgroundColor(iarg & 0xFF, (iarg >> 8) & 0xFF, (iarg >> 16) & 0xFF); return 0;
    case 10: { GXColor k = Camera_GetBackgroundColor(); return k.r | (k.g << 8) | (k.b << 16) | (k.a << 24); }
    case 11: Camera_GetTransformPosition(&a); fout[0] = a.x; fout[1] = a.y; fout[2] = a.z; return 0;
    case 12: Camera_GetTransformInterest(&a); fout[0] = a.x; fout[1] = a.y; fout[2] = a.z; return 0;
    case 13: Camera_80030DE4(fin[0], fin[1]); return 0;
    case 14: Camera_80030DF8(); return 0;
    case 15: fout[0] = Camera_80030E10(); return 0;
    case 16: return Camera_80031060();
    case 17: Camera_80031074((u8) iarg); return 0;
    case 18: return Camera_8003108C();
    case 19: Camera_800310A0((u8) iarg); return 0;
    case 20: Camera_800310E8(); return 0;
    case 21: fout[0] = Camera_80031144(); return 0;
    case 22: Camera_800311CC(fin[0]); return 0;
    case 23: Camera_800311DC(fin[0]); return 0;
    case 24: Camera_StageSlotClipGet(&fout[0], &fout[1]); return 0;
    case 25: Camera_80030A60(iarg); return 0;
    case 26: return Camera_80030A78();
    case 27: Camera_80030A8C(iarg); return 0;
    case 28: Camera_SetStageVisible(iarg); return 0;
    case 29: return Camera_80030AC4();
    case 30: Camera_80030AE0(iarg); return 0;
    case 31: return Camera_80030AF8();
    case 32: Camera_80030B0C(iarg); return 0;
    case 33: return Camera_80030B24();
    case 34: Camera_80030B38(iarg); return 0;
    case 35: return Camera_80030B50();
    case 36: Camera_80030B64(iarg); return 0;
    case 37: return Camera_80030B7C();
    case 38: Camera_80030B90(iarg); return 0;
    case 39: return Camera_80030BA8();
    case 40: Camera_8002FE38(); return 0;
    case 41: Camera_8002EEC8(fin[0]); return 0;
    case 42: {
        s32 r;
        a.x = fin[0]; a.y = fin[1]; a.z = fin[2];
        b.x = fin[3]; b.y = fin[4]; b.z = fin[5];
        c.x = c.y = c.z = 0.0f;
        r = Camera_8002BC78(&a, &b, &c);
        fout[0] = a.x; fout[1] = a.y; fout[2] = a.z; fout[3] = b.x; fout[4] = b.y; fout[5] = b.z;
        fout[6] = c.x; fout[7] = c.y; fout[8] = c.z;
        return r;
    }
    case 43: fout[0] = cm_script_float(iarg); return 0;
    case 44: return cm_script_bits(fin[0]);
    case 45: fout[0] = cm_script_lerp(fin[0], fin[1], fin[2]); return 0;
    case 46: fout[0] = cm_script_ease(fin[0], iarg); return 0;
    case 47: {
        CmScriptPose p, q, r;
        p.eye.x = fin[0]; p.eye.y = fin[1]; p.eye.z = fin[2];
        p.interest.x = fin[3]; p.interest.y = fin[4]; p.interest.z = fin[5];
        p.fov = fin[6]; p.roll = fin[7];
        q.eye.x = fin[8]; q.eye.y = fin[9]; q.eye.z = fin[10];
        q.interest.x = fin[11]; q.interest.y = fin[12]; q.interest.z = fin[13];
        q.fov = fin[14]; q.roll = fin[15];
        r = cm_script_blend(p, q, fin[16]);
        fout[0] = r.eye.x; fout[1] = r.eye.y; fout[2] = r.eye.z;
        fout[3] = r.interest.x; fout[4] = r.interest.y; fout[5] = r.interest.z;
        fout[6] = r.fov; fout[7] = r.roll;
        return 0;
    }
    }
    return -999;
}
