/* Actual-source graph/traversal tests. stage_seam_test.py supplies verbatim
 * functions from the game TUs. Only memory, scale, and GObj operations are fixtures. */
#include <assert.h>
#include <stdbool.h>
#include <float.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
typedef uint32_t u32;
typedef uint16_t u16;
typedef int16_t s16;
typedef uint8_t u8;
typedef struct { float x, y; } Vec2;
typedef struct { float x, y, z; } Vec3;
typedef struct { u16 v0_idx, v1_idx; s16 prev_id0, next_id0, prev_id1, next_id1; u16 hi_flags, lo_flags; } MapLine;
typedef struct { MapLine* x0; u32 flags; } CollLine;
typedef struct { float x0, x4; Vec2 pos; float x10, x14; } CollVtx;
enum { MapLineGroup_Floor, MapLineGroup_Ceiling, MapLineGroup_RightWall, MapLineGroup_LeftWall, MapLineGroup_Dynamic, MapLineGroup_Count };
typedef struct MapLineRange { s16 start, count; } MapLineRange;
typedef struct { MapLineRange ranges[MapLineGroup_Count]; float left_bound, bottom_bound, right_bound, top_bound; s16 vtx_start, vtx_count; } MapJoint;
typedef struct CollJoint { struct CollJoint* next; MapJoint* inner; u32 flags; bool xE; Vec2 bounding_min, bounding_max; } CollJoint;
typedef struct { Vec2* verts; int vert_count; MapLine* lines; int line_count; MapLineRange ranges[MapLineGroup_Count]; MapJoint* joints; int joint_count; } MapCollData;
typedef struct mp_UnkStruct0 { struct mp_UnkStruct0* next; u16 x4, x6; Vec3 x8, x14; int x20; s16 x24, x26, x28; } mp_UnkStruct0;
enum { CollLine_Floor = 1, LINE_FLAG_KIND = 15, LINE_FLAG_PLATFORM = 256, LINE_FLAG_LEDGE = 512,
       LINE_FLAG_ENABLED = 1 << 16, LINE_FLAG_HIDDEN = 1 << 18,
       CollJoint_B8 = 1 << 8, CollJoint_B11 = 1 << 11, CollJoint_Enabled = 1 << 16, CollJoint_Hidden = 1 << 18 };
#define SCRIPT_STAGE_LINES 12
#define SCRIPT_STAGE_MODELS 2
#define SCRIPT_STAGE_TARGETS 2
#define SQ(x) ((x) * (x))
#define ABS(x) fabsf(x)
#define F32_MAX FLT_MAX
#define UNUSED __attribute__((unused))
#define PAD_STACK(n) ((void)0)
#define HSD_ASSERT(line, expr) assert(expr)
#define LINEID_CHECK(line, index) assert((index) >= 0 && (index) < map.line_count)
#define OSReport(...) ((void)0)
#define memzero(ptr, count) memset(ptr, 0, count)
typedef struct { Vec3 translate; } HSD_JObj;
typedef struct { HSD_JObj root; } HSD_GObj;
typedef HSD_GObj Item_GObj;
#define GET_JOBJ(object) (&(object)->root)
typedef struct { int handle, active, kind, flags, model_handle, owner, area; float x0, y0, x1, y1; } ScriptStageLine;
typedef struct { int handle, active, line_handle, owner; HSD_GObj* gobj; float x, y; } ScriptStageModel;
typedef struct { int handle, active; Item_GObj* gobj; } ScriptStageTarget;
static struct {
    MapCollData* map; int cap, base_v, base_l, base_j, target_remaining, loading_area;
    ScriptStageLine line[SCRIPT_STAGE_LINES]; ScriptStageModel model[SCRIPT_STAGE_MODELS]; ScriptStageTarget target[SCRIPT_STAGE_TARGETS];
} script_stage;
static MapLine lines[32], saved_lines[32];
static MapJoint map_joints[20];
static CollLine groundCollLine[32];
static CollVtx groundCollVtx[40];
static CollJoint groundCollJoint[20];
static Vec2 vertices[40];
static MapCollData map, *current_map;
#define mpLib_804D64B4 current_map
static CollJoint *jointListStart, *jointListEnd;
static struct { mp_UnkStruct0* next; } mpIsland_80458E88;
static mp_UnkStruct0* island_pool;
static int owner = 7, refresh_calls, bounding_checks;
static MapCollData* mpLib_8004D164(void) { return current_map; }
static CollLine* mpGetGroundCollLine(void) { return groundCollLine; }
static CollVtx* mpGetGroundCollVtx(void) { return groundCollVtx; }
static CollJoint* mpGetGroundCollJoint(void) { return groundCollJoint; }
static int Script_StageResourceOwner(void) { return owner; }
static float Ground_801C0498(void) { return 1; }
static void mpUncheckBounding(void) { ++bounding_checks; }
/* This fixture tests standalone seams (area 0); area gates/batching have their
 * own actual-source fixture. Keep host scalar hooks explicit. */
#define ScriptGame_StageJointActive(joint) 1
#define ScriptGame_StageLineActive(line) 1
#define script_area_retiring(area) 0
#define mpScriptBatchBegin() ((void)0)
#define mpScriptBatchEnd() ((void)0)
#define mpScriptInvalidateBounding() mpUncheckBounding()
#define mpScriptIslandRefresh mpIsland_8005B334
static void HSD_GObjFree(HSD_GObj* object) { (void)object; assert(!"unexpected GObj fixture"); }
static void Item_8026A8EC(Item_GObj* object) { (void)object; assert(!"unexpected target fixture"); }
static void HSD_JObjSetTranslate(HSD_JObj* object, const Vec3* pos) { object->translate = *pos; }
static void PSVECNormalize(const Vec3* source, Vec3* out) {
    float norm = sqrtf(source->x * source->x + source->y * source->y + source->z * source->z);
    out->x = source->x / norm; out->y = source->y / norm; out->z = source->z / norm;
}
static void* HSD_MemAlloc(size_t bytes) {
    /* Native code allocates the 32-bit island prefix; the host fixture uses
     * named fields on either host pointer width. No ABI claim is made here. */
    assert(bytes == 0x2C); return calloc(1, sizeof(mp_UnkStruct0));
}
static void mpIsland_AssertSeg(void* segment) { assert(segment); }
#include "readers.inc"
#include "islands.inc"
static void mpIsland_8005B334(int joint, int first, int count, bool enabled) {
    ++refresh_calls;
    mpIsland_8005B004(&mpIsland_80458E88.next, &island_pool, joint, 1, first, count, enabled);
}
#include "refresh.inc"
#include "script_stage_seams.inc"
#include "lifetime.inc"
#include "stitcher.inc"

static int bits(float value) { union { int i; float f; } u; u.f = value; return u.i; }
static void release_islands(mp_UnkStruct0* list) {
    while (list) { mp_UnkStruct0* next = list->next; free(list); list = next; }
}
static void fixture(void) {
    int i;
    release_islands(mpIsland_80458E88.next); release_islands(island_pool);
    mpIsland_80458E88.next = island_pool = NULL;
    memset(&script_stage, 0, sizeof script_stage); memset(lines, 0, sizeof lines);
    memset(map_joints, 0, sizeof map_joints); memset(groundCollLine, 0, sizeof groundCollLine);
    memset(groundCollVtx, 0, sizeof groundCollVtx); memset(groundCollJoint, 0, sizeof groundCollJoint);
    map.verts = vertices; map.lines = lines; map.joints = map_joints;
    map.vert_count = 40; map.line_count = 32; map.joint_count = 20;
    script_stage.map = current_map = &map; script_stage.cap = SCRIPT_STAGE_LINES;
    script_stage.base_l = 4; script_stage.base_v = 8; script_stage.base_j = 4;
    for (i = 0; i < 32; ++i) {
        lines[i].prev_id0 = lines[i].prev_id1 = lines[i].next_id0 = lines[i].next_id1 = -1;
        groundCollLine[i].x0 = &lines[i];
    }
    for (i = 0; i < 20; ++i) groundCollJoint[i].inner = &map_joints[i];
    for (i = 0; i < SCRIPT_STAGE_LINES; ++i) {
        lines[4 + i].v0_idx = 8 + i * 2; lines[4 + i].v1_idx = 9 + i * 2;
        map_joints[4 + i].vtx_start = 8 + i * 2; map_joints[4 + i].vtx_count = 2;
    }
    jointListStart = jointListEnd = NULL; owner = 7; refresh_calls = bounding_checks = 0;
}
static int add(int handle, float x0, float y0, float x1, float y1, int kind, int flags) {
    assert(ScriptGame_StageAddLine(bits(x0), bits(y0), bits(x1), bits(y1), kind, flags, handle) == handle);
    return handle;
}
static int index(int handle) { int slot = script_stage_seam_slot(handle); assert(slot >= 0); return 4 + slot; }
static int follow(int handle, float x, float y) {
    Vec3 position = { x, y, 0 }; float difference;
    return mpLib_8004DD90_Floor(index(handle), &position, &difference, NULL, NULL);
}
static void no_seams(int line) {
    assert(lines[line].prev_id0 == -1 && lines[line].prev_id1 == -1 &&
           lines[line].next_id0 == -1 && lines[line].next_id1 == -1);
}

int main(void) {
    int a, b, c, d, copies[4], i, before;
    (void)script_stage_seam_refresh; /* full-slot rebuild used by stage slots */
    fixture();
    a = add(101, -52, 0, -26, 13, 1, 1); b = add(102, -26, 13, 0, 13, 1, 0);
    c = add(103, 0, 13, 26, 26, 1, 1); d = add(104, 26, 26, 52, 26, 1, 0);
    assert(follow(a, -20, 13) == -1); /* actual floor-follow dead end before linking */
    assert(ScriptGame_StageLink(a, b, owner) == 1);
    assert(ScriptGame_StageLink(c, b, owner) == 1); /* reversed argument order */
    assert(ScriptGame_StageLink(c, d, owner) == 1);
    assert(lines[index(a)].next_id0 == index(b) && lines[index(a)].next_id1 == index(b));
    assert(lines[index(b)].prev_id0 == index(a) && lines[index(b)].prev_id1 == index(a));
    assert(follow(a, 40, 26) == index(d) && follow(d, -40, 6) == index(a));
    for (i = 0; i < 4; ++i) {
        mp_UnkStruct0* island = mpIsland_8005AB54(4 + i);
        assert(island && island->x24 == index(a) && island->x26 == index(d));
    }
    /* Every per-joint island caches the whole chain; removing its middle must
     * refresh distant joints and erase reciprocal id0 as well as id1. */
    assert(ScriptGame_StageRemove(b));
    assert(mpLineGetNext(index(a)) == -1 && mpLineGetPrev(index(c)) == -1);
    no_seams(5);
    assert(mpIsland_8005AB54(index(a))->x26 == index(a));
    assert(mpIsland_8005AB54(index(d))->x24 == index(c));
    assert(follow(a, 40, 26) == -1 && follow(c, 40, 26) == index(d));
    add(105, -26, 13, 0, 13, 1, 0); no_seams(index(105)); /* slot reuse stays unlinked */
    assert(ScriptGame_StageLink(a, 105, owner) == 1 && ScriptGame_StageLink(105, c, owner) == 1);
    /* Moving a joined line apart clears both fallback directions. Movement
     * within tolerance retains the reciprocal graph; moving back does not infer it. */
    assert(script_stage_line_set(c, 0.01f, 13, 26.01f, 26));
    assert(mpLineGetPrev(index(c)) == index(105) && mpLineGetNext(index(c)) == index(d));
    assert(ScriptGame_StageMove(c, bits(18), bits(19.5f)));
    no_seams(index(c)); assert(mpLineGetNext(index(105)) == -1 && mpLineGetPrev(index(d)) == -1);
    assert(script_stage_line_set(c, 0, 13, 26, 26)); no_seams(index(c));
    assert(ScriptGame_StageLink(105, c, owner) == 1 && ScriptGame_StageLink(c, d, owner) == 1);
    /* Source/destination copies have one mod owner. Explicit handles link only
     * their own graph even though every endpoint has duplicate geometry. */
    copies[0] = add(201, -52, 0, -26, 13, 1, 1); copies[1] = add(202, -26, 13, 0, 13, 1, 0);
    copies[2] = add(203, 0, 13, 26, 26, 1, 1); copies[3] = add(204, 26, 26, 52, 26, 1, 0);
    for (i = 0; i < 3; ++i) assert(ScriptGame_StageLink(copies[i], copies[i + 1], owner) == 1);
    assert(mpLineGetNext(index(a)) == index(105) && mpLineGetNext(index(copies[0])) == index(copies[1]));
    assert(ScriptGame_StageRemove(a) && ScriptGame_StageRemove(105) && ScriptGame_StageRemove(c) && ScriptGame_StageRemove(d));
    assert(follow(copies[0], 40, 26) == index(copies[3]));
    /* Vanilla nearby-joint stitching is excluded for scripted joints. */
    memcpy(saved_lines, lines, sizeof lines);
    mpLib_800581DC(8, 9); assert(!memcmp(saved_lines, lines, sizeof lines));

    fixture(); a = add(301, -10, 0, 0, 0, 1, 0); b = add(302, 0, 0, 10, 0, 1, 0);
    memcpy(saved_lines, lines, sizeof lines); before = refresh_calls;
    assert(ScriptGame_StageLink(a, a, owner) == -2);
    assert(ScriptGame_StageLink(a, 9999, owner) == -2);
    assert(ScriptGame_StageLink(a, b, owner + 1) == -3);
    current_map = NULL; assert(ScriptGame_StageLink(a, b, owner) == -1);
    assert(!ScriptGame_StageMove(b, bits(50), bits(0)) && !ScriptGame_StageRemove(b)); current_map = &map;
    assert(!memcmp(saved_lines, lines, sizeof lines) && refresh_calls == before);
    groundCollVtx[lines[index(b)].v0_idx].pos.x = 0.1f;
    assert(ScriptGame_StageLink(a, b, owner) == -5); /* actual physics vertices, not stale metadata */
    groundCollVtx[lines[index(b)].v0_idx].pos.x = 0;
    owner = 8; c = add(303, 0, 0, 10, 0, 1, 0); owner = 7;
    assert(ScriptGame_StageLink(a, c, owner) == -3);
    d = add(304, 0, 0, 0, 10, 4, 0); assert(ScriptGame_StageLink(a, d, owner) == -4);
    c = add(305, 0.06f, 0, 10, 0, 1, 0); assert(ScriptGame_StageLink(a, c, owner) == -5);
    c = add(306, -5, 0, 5, 0, 1, 0); assert(ScriptGame_StageLink(a, c, owner) == -5); /* T/interior */
    assert(ScriptGame_StageLink(a, b, owner) == 1);
    assert(ScriptGame_StageLink(a, b, owner) == -7); /* occupied, including same pair */
    c = add(307, 0, 0, 15, 0, 1, 0); assert(ScriptGame_StageLink(a, c, owner) == -7);
    lines[index(a)].next_id1 = -1; assert(ScriptGame_StageLink(a, c, owner) == -8);
    assert(ScriptGame_StageRemove(a)); assert(mpLineGetPrev(index(b)) == -1); /* clear inconsistent reciprocal pair */
    fixture(); a = add(351, -10, 0, 0, 0, 1, 0); b = add(352, 0, 0, 10, 0, 1, 0);
    lines[index(a)].next_id0 = index(b); /* one-sided stale fallback */
    assert(ScriptGame_StageMove(b, bits(50), bits(0)));
    assert(mpLineGetNext(index(a)) == -1); no_seams(index(b));
    /* Unsafe-map refusal retains the live handle and graph for a real retry;
     * pointer equality alone is insufficient after reserved counts retire. */
    fixture(); a = add(371, -10, 0, 0, 0, 1, 0); b = add(372, 0, 0, 10, 0, 1, 0);
    assert(ScriptGame_StageLink(a, b, owner) == 1);
    memcpy(saved_lines, lines, sizeof lines); before = refresh_calls;
    current_map = NULL;
    assert(!ScriptGame_StageRemove(a));
    assert(script_stage_seam_slot(a) >= 0);
    current_map = &map; map.line_count = script_stage.base_l;
    assert(!ScriptGame_StageRemove(a));
    assert(script_stage_seam_slot(a) >= 0);
    assert(!memcmp(saved_lines, lines, sizeof lines) && refresh_calls == before);
    map.line_count = 32;
    assert(ScriptGame_StageRemove(a));
    assert(script_stage_seam_slot(a) == -1 && mpLineGetPrev(index(b)) == -1);
    no_seams(4);
    fixture(); a = add(401, 0, 0, 0.01f, 0, 1, 0); b = add(402, 0.01f, 0, 0.02f, 0, 1, 0);
    assert(ScriptGame_StageLink(a, b, owner) == -6); /* both orientations within EPS */
    fixture(); a = add(501, -10, 0, 0, 0, 1, 0); b = add(502, 0.03f, 0.04f, 10, 0, 1, 0);
    assert(ScriptGame_StageLink(a, b, owner) == 1); /* Euclidean EPS boundary */
    groundCollLine[index(b)].flags |= LINE_FLAG_HIDDEN;
    assert(ScriptGame_StageLink(a, b, owner) == -8);
    fixture();
    /* The scripted-joint exclusion leaves vanilla builtin stitching intact. */
    lines[0].v0_idx = 0; lines[0].v1_idx = 1; lines[1].v0_idx = 2; lines[1].v1_idx = 3;
    groundCollVtx[0].pos.x = -10; groundCollVtx[1].pos.x = 0;
    groundCollVtx[2].pos.x = 0; groundCollVtx[3].pos.x = 10;
    map_joints[0].vtx_start = 0; map_joints[0].vtx_count = 2;
    map_joints[1].vtx_start = 2; map_joints[1].vtx_count = 2;
    map_joints[0].ranges[MapLineGroup_Floor].start = 0; map_joints[0].ranges[MapLineGroup_Floor].count = 1;
    map_joints[1].ranges[MapLineGroup_Floor].start = 1; map_joints[1].ranges[MapLineGroup_Floor].count = 1;
    mpLib_800581DC(0, 1);
    assert(lines[0].next_id1 == 1 && lines[1].prev_id1 == 0);
    assert(lines[0].next_id0 == -1 && lines[1].prev_id0 == -1);
    add(601, 0, 0, 10, 0, 1, 0);
    memcpy(saved_lines, lines, sizeof lines);
    mpLib_800581DC(0, 4); assert(!memcmp(saved_lines, lines, sizeof lines));
    fixture();
    puts("actual-source stage seam graph, floor-follow, dynamic-island and lifecycle tests passed");
    return 0;
}
