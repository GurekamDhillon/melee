/* ScriptGame_ArenaSpawn's world/local contract against the actual script_arena.inc.
 * The matrix routines (mtx_pc.c) and lb_8000B1CC are extracted at build time, never
 * hand-copied; only the joint tree and the stage tables are doubles. */
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <string.h>
typedef unsigned char u8;
typedef unsigned int u32;
typedef float f32;
typedef struct { float x, y, z; } Vec, Vec3;
typedef struct { float x, y, z, w; } Quaternion;
typedef f32 Mtx[3][4];
typedef f32 (*MtxPtr)[4];
typedef f32 Mtx44[4][4];
typedef struct HSD_JObj {
    struct HSD_JObj* parent;
    Vec3 translate;
    Mtx mtx;
    int flags, world_fixed; /* world_fixed: mtx is this joint's given world matrix */
} HSD_JObj;
typedef struct { float left, right, top, bottom; } StageBlastZone;
static struct {
    struct { float cam_x_offset, cam_y_offset; StageBlastZone cam_bounds; } cam_info;
    StageBlastZone blast_zone;
} stage_info;
#define MTXMultVec PSMTXMultVec
static HSD_JObj* HSD_JObjGetParent(HSD_JObj* j) { return j->parent; }
static void HSD_JObjGetTranslation(HSD_JObj* j, Vec3* v) { *v = j->translate; }
static void HSD_JObjGetRotation(HSD_JObj* j, Quaternion* q) { (void) j; memset(q, 0, sizeof *q); }
static void HSD_JObjGetScale(HSD_JObj* j, Vec3* v) { (void) j; v->x = v->y = v->z = 1; }
static void HSD_JObjSetupMatrix(HSD_JObj* j);
#include "arena_spawn_functions.inc"

static void HSD_JObjSetupMatrix(HSD_JObj* j)
{
    Mtx local = { { 1, 0, 0, 0 }, { 0, 1, 0, 0 }, { 0, 0, 1, 0 } };
    if (j->world_fixed) return;
    local[0][3] = j->translate.x;
    local[1][3] = j->translate.y;
    local[2][3] = j->translate.z;
    if (j->parent) {
        HSD_JObjSetupMatrix(j->parent);
        PSMTXConcat(j->parent->mtx, local, j->mtx);
    } else PSMTXCopy(local, j->mtx);
}
static void HSD_JObjSetTranslate(HSD_JObj* j, Vec3* v) { assert(j && v); j->translate = *v; }

static HSD_JObj parent_joint, spawn_joint, free_joint;
static int missing_point;
static HSD_JObj* Ground_801C2CF4(int slot)
{
    if (missing_point) return NULL;
    return slot == 4 ? &spawn_joint : slot == 5 ? &free_joint : NULL;
}
static int script_mesh_bits(float f) { union { float f; int i; } u; u.f = f; return u.i; }
static float script_mesh_float(int i) { union { float f; int i; } u; u.i = i; return u.f; }
static void Ground_801C38BC(float x, float y) { stage_info.cam_info.cam_x_offset = x; stage_info.cam_info.cam_y_offset = y; }
struct MapLineRange { int start, count; };
typedef struct { int flags; } CollLine;
typedef struct { int flags; struct { struct MapLineRange ranges[1]; }* inner; } CollJoint;
enum { MapLineGroup_Count = 1, CollJoint_Enabled = 1, LINE_FLAG_ENABLED = 1 };
static CollJoint joints[256];
static CollLine lines[1536];
static CollJoint* mpGetGroundCollJoint(void) { return joints; }
static CollLine* mpGetGroundCollLine(void) { return lines; }
static void mpJointListAdd(int id) { (void) id; }
static void mpLib_80057BC0(int id) { (void) id; }
/* Tracking diagnostics do not require the game OS shim. */
static void OSReport(const char *fmt, ...) { (void) fmt; }
#include "../gameworld/script_arena.inc"

static int near(float a, float b) { return fabsf(a - b) < 1e-3f; }
static int same(Vec3 a, Vec3 b) { return !memcmp(&a, &b, sizeof a); }
static float world(int slot, int axis) { return ScriptGame_ArenaSpawnPos(slot, axis, -9999.0f); }
static int spawn(int slot, float x, float y) { return ScriptGame_ArenaSpawn(1, slot, script_mesh_bits(x), script_mesh_bits(y)); }

int main(void)
{
    /* Parent: scale 2, rotated 90 degrees about Z, translated (100, 50, 7). */
    const Mtx turned = { { 0, -2, 0, 100 }, { 2, 0, 0, 50 }, { 0, 0, 2, 7 } };
    const Vec3 local0 = { 2, 3, 0.5f };
    Vec3 moved;
    stage_info.cam_info.cam_bounds = (StageBlastZone) { -100, 100, 200, 0 };
    stage_info.blast_zone = (StageBlastZone) { -120, 120, 220, -20 };
    memcpy(parent_joint.mtx, turned, sizeof turned);
    parent_joint.world_fixed = 1;
    spawn_joint.parent = &parent_joint;
    spawn_joint.translate = local0;

    /* world = P * local: (100 - 2*3, 50 + 2*2, 7 + 2*.5) */
    assert(near(world(4, 0), 94) && near(world(4, 1), 54) && near(world(4, 2), 8));

    /* A requested world position is what reads back, through rotation and scale. */
    assert(spawn(4, 10, 20) == 1);
    assert(ScriptGame_ArenaOwner() == 1);
    printf("parented: requested world (10,20) => world (%g,%g,%g) local (%g,%g,%g)\n", world(4, 0), world(4, 1),
           world(4, 2), spawn_joint.translate.x, spawn_joint.translate.y, spawn_joint.translate.z);
    assert(near(world(4, 0), 10) && near(world(4, 1), 20) && near(world(4, 2), 8));
    /* A second move keeps the first saved transform, not the moved one. */
    assert(spawn(4, -30, 44) == 1);
    assert(near(world(4, 0), -30) && near(world(4, 1), 44));
    moved = spawn_joint.translate;
    assert(!same(moved, local0));

    /* Restore puts back the exact local transform, even though the parent moved meanwhile. */
    parent_joint.mtx[0][3] = 300;
    ScriptGame_ArenaRestore();
    assert(same(spawn_joint.translate, local0));
    assert(!ScriptGame_ArenaOwner());
    assert(near(world(4, 0), 294) && near(world(4, 1), 54) && near(world(4, 2), 8));
    parent_joint.mtx[0][3] = 100;
    printf("restore: local (%g,%g,%g) bit-exact, world (%g,%g,%g)\n", spawn_joint.translate.x,
           spawn_joint.translate.y, spawn_joint.translate.z, world(4, 0), world(4, 1), world(4, 2));
    /* Nothing left saved: a second restore is inert. */
    spawn_joint.translate = moved;
    ScriptGame_ArenaRestore();
    assert(same(spawn_joint.translate, moved));
    spawn_joint.translate = local0;

    /* An unparented point's local space is world space. */
    free_joint.translate = (Vec3) { 5, 6, 1 };
    assert(spawn(5, 70, 80) == 1);
    assert(free_joint.translate.x == 70 && free_joint.translate.y == 80 && free_joint.translate.z == 1);
    assert(world(5, 0) == 70 && world(5, 1) == 80);
    ScriptGame_ArenaRestore();
    assert(free_joint.translate.x == 5 && free_joint.translate.y == 6 && free_joint.translate.z == 1);

    /* Absent point and out-of-range slots: failure, no ownership, no change. */
    missing_point = 1;
    assert(spawn(4, 30, 40) == 0 && !ScriptGame_ArenaOwner());
    missing_point = 0;
    assert(spawn(-1, 30, 40) == 0 && spawn(261, 30, 40) == 0 && spawn(9, 30, 40) == 0);
    assert(!ScriptGame_ArenaOwner() && same(spawn_joint.translate, local0));

    /* Singular parent: failure before ownership, the point untouched, nothing to restore. */
    memset(parent_joint.mtx, 0, sizeof parent_joint.mtx);
    assert(spawn(4, 30, 40) == 0);
    assert(!ScriptGame_ArenaOwner() && same(spawn_joint.translate, local0));
    assert(!script_arena.spawn_saved_ok[4]);
    puts("arena spawn world/local contract OK");
    return 0;
}
