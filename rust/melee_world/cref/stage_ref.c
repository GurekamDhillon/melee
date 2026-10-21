/* C reference for melee_world::gr::stage. Built by gen_stage_ref.sh, which cuts the function
 * bodies out of src/melee/gr/stage.c (stage_body.inc, stage_map.inc), stage_stage_slot_retail.inc,
 * ScriptGame_ArenaBound out of pc/gameworld/script_arena.inc (arena.inc) and the sin/cos/Rotate/
 * ApplyEulerRotation bodies out of src/melee/lb/lbvector.c (lbvector.inc). Everything else is a
 * minimal stand-in. Output (all hex):
 *   G <42 inputs> | <outputs>     one per random getter case
 *   D <arg> <bool>                Stage_80224DC8
 *   M <i> <grkind>                stage_id_map after Stage_MexInitIds with the fixed m-ex config
 *   S <kind> <ret> <stkind>       Stage_StageSlotSelect
 * tests/stage_vs_c.rs replays them. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define TARGET_PC 1
typedef uint8_t u8;
typedef int32_t s32;
typedef uint32_t u32;
typedef float f32;
typedef int bool;
typedef int enum_t;
#define true 1
#define false 0
#define M_PI 3.14159265358979323846
#define M_TAU 6.283185307179586
#define ARRAY_SIZE(a) (sizeof(a) / sizeof((a)[0]))
#define OSReport(...) ((void) 0)

typedef struct { f32 x, y, z; } Vec3;
typedef struct { f32 left, right, top, bottom; } StageBlastZone;
typedef struct {
    StageBlastZone cam_bounds;
    f32 cam_x_offset, cam_y_offset, cam_vertical_tilt, cam_pan_degrees, x20, x24,
        cam_track_ratio, cam_fixed_zoom, cam_track_smooth, cam_zoom_rate, cam_max_depth, x3C,
        pausecam_zpos_min, pausecam_zpos_init, pausecam_zpos_max, cam_angle_up, cam_angle_down,
        cam_angle_left, cam_angle_right;
    Vec3 fixed_cam_pos;
    f32 fixed_cam_fov, fixed_cam_vert_angle, fixed_cam_horz_angle;
} StageCameraInfo;
static struct {
    StageCameraInfo cam_info;
    StageBlastZone blast_zone;
} stage_info;
static struct {
    int owner;
    float x, y;
    StageBlastZone camera, blast;
} script_arena;

#include "lbvector.inc"
#include "arena.inc"
#define ARENA_BOUND(field, fallback) ScriptGame_ArenaBound(field, fallback)
#include "stage_body.inc"

/* stage_id_map + Stage_MexInitIds + Stage_8022519C */
#include "grkind.inc"
typedef s32 StKind;
struct StageIdMapEntry { GrKind grkind; s32 unk1; s32 unk2; };
struct StageSelection { StKind stkind; struct StageIdMapEntry* entry; };
static struct StageSelection selected_stage = { 2, NULL };
#define ST_MEX_EXT_MAX 384
static int Mex_GrExternalCount(void) { return 300; }
static int Mex_GrKindForExt(int e) { return e % 5 == 0 ? -1 : (e * 7) % 80; }
#include "stage_map.inc"
#include "slot.inc"

static u32 rng = 0x12345678u;
static u32 next(void)
{
    rng ^= rng << 13;
    rng ^= rng >> 17;
    rng ^= rng << 5;
    return rng;
}
static f32 rf(f32 lo, f32 hi) { return lo + (hi - lo) * (f32) (next() >> 8) / 16777216.0f; }
static u32 b(f32 f)
{
    u32 u;
    memcpy(&u, &f, 4);
    return u;
}

int main(void)
{
    int i;
    for (i = 0; i < 20000; i++) {
        f32* ci = (f32*) &stage_info.cam_info;
        int k;
        u32 in[42];
        int n = 0;
        for (k = 0; k < (int) (sizeof(StageCameraInfo) / 4); k++) {
            ci[k] = rf(-500.0f, 500.0f);
        }
        stage_info.blast_zone.left = rf(-500, 500);
        stage_info.blast_zone.right = rf(-500, 500);
        stage_info.blast_zone.top = rf(-500, 500);
        stage_info.blast_zone.bottom = rf(-500, 500);
        if (i % 3 == 0) {
            stage_info.cam_info.fixed_cam_vert_angle = rf(-60, 60);
            stage_info.cam_info.fixed_cam_horz_angle = rf(-60, 60);
        }
        script_arena.owner = (next() & 3) == 0 ? (int) (next() % 4 + 1) : 0;
        script_arena.x = rf(-300, 300);
        script_arena.y = rf(-300, 300);
        script_arena.camera.left = rf(-300, 300);
        script_arena.camera.right = rf(-300, 300);
        script_arena.camera.top = rf(-300, 300);
        script_arena.camera.bottom = rf(-300, 300);
        script_arena.blast.left = rf(-300, 300);
        script_arena.blast.right = rf(-300, 300);
        script_arena.blast.top = rf(-300, 300);
        script_arena.blast.bottom = rf(-300, 300);

        printf("G");
        for (k = 0; k < (int) (sizeof(StageCameraInfo) / 4); k++) printf(" %08x", b(ci[k]));
        printf(" %08x %08x %08x %08x", b(stage_info.blast_zone.left), b(stage_info.blast_zone.right),
               b(stage_info.blast_zone.top), b(stage_info.blast_zone.bottom));
        printf(" %x %08x %08x", script_arena.owner, b(script_arena.x), b(script_arena.y));
        printf(" %08x %08x %08x %08x", b(script_arena.camera.left), b(script_arena.camera.right),
               b(script_arena.camera.top), b(script_arena.camera.bottom));
        printf(" %08x %08x %08x %08x |", b(script_arena.blast.left), b(script_arena.blast.right),
               b(script_arena.blast.top), b(script_arena.blast.bottom));
        {
            Vec3 v;
            f32 o[] = {
                Stage_GetCamBoundsLeftOffset(), Stage_GetCamBoundsRightOffset(),
                Stage_GetCamBoundsTopOffset(), Stage_GetCamBoundsBottomOffset(),
                Stage_GetCamPanAngleRadians(), Stage_GetCamMaxDepth(), Stage_GetCamZoomRate(),
                Stage_GetCamInfoX20(), Stage_GetCamInfoX24(), Stage_GetCamFixedZoom(),
                Stage_GetCamTrackRatio(), Stage_GetCamTrackSmooth(),
                Stage_GetBlastZoneRightOffset(), Stage_GetBlastZoneLeftOffset(),
                Stage_GetBlastZoneTopOffset(), Stage_GetBlastZoneBottomOffset(),
                Stage_CalcUnkCamY(), Stage_CalcUnkCamYBounds(), Stage_GetPauseCamZPosMin(),
                Stage_GetPauseCamZPosInit(), Stage_GetPauseCamZPosMax(),
                Stage_GetCamAngleRadiansUp(), Stage_GetCamAngleRadiansDown(),
                Stage_GetCamAngleRadiansLeft(), Stage_GetCamAngleRadiansRight(),
                Stage_GetCamFixedFov(),
            };
            for (k = 0; k < (int) (sizeof(o) / 4); k++) printf(" %08x", b(o[k]));
            Stage_UnkSetVec3TCam_Offset(&v);
            printf(" %08x %08x %08x", b(v.x), b(v.y), b(v.z));
            Stage_SetVecToFixedCamPos(&v);
            printf(" %08x %08x %08x", b(v.x), b(v.y), b(v.z));
            Stage_80224CAC(&v);
            printf(" %08x %08x %08x\n", b(v.x), b(v.y), b(v.z));
        }
    }
    for (i = -20; i < 120; i++) printf("D %x %x\n", (u32) i, Stage_80224DC8(i));
    Stage_8022519C(0);
    for (i = 0; i < ST_MEX_EXT_MAX; i++) printf("M %x %x\n", i, stage_id_map[i].grkind);
    for (i = -2; i < 90; i++) {
        int r = Stage_StageSlotSelect(i);
        printf("S %x %x %x\n", (u32) i, (u32) r, (u32) selected_stage.stkind);
    }
    return 0;
}
