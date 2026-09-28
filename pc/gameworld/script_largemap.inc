/* largemap: included in script_game.c's existing snapshot domain.
 * Only scalar calls cross the native/game boundary. Names are synchronous input. */
#include <sysdolphin/baselib/initialize.h>
#include <dolphin/os/OSAlloc.h>
extern int Script_AreaNameByte(int index);

/* mpLibLoad calls this before StageReady appends the reserved ranges. Online and
 * scenes without a prepared script map keep the retail 256-joint allocation. */
int ScriptGame_StageJointCapacity(void)
{
    int count = script_stage.map ? script_stage.base_j + script_stage.cap : 0;
    return count > 256 ? count : 256;
}

int ScriptGame_AreaFind(int owner)
{
    int i, j;
    for (i = 0; i < SCRIPT_STAGE_AREAS; ++i) {
        if (script_stage.area[i].owner != owner) continue;
        for (j = 0; j < 48; ++j) {
            int c = Script_AreaNameByte(j);
            if ((unsigned char)script_stage.area[i].name[j] != c) break;
            if (!c) return i + 1;
        }
    }
    return 0;
}

int ScriptGame_AreaBegin(int owner)
{
    int i, j;
    if (owner <= 0 || script_stage.loading_area) return -1;
    if (ScriptGame_AreaFind(owner)) return 0; /* idempotent */
    for (i = 0; i < SCRIPT_STAGE_AREAS && script_stage.area[i].owner; ++i) {}
    if (i == SCRIPT_STAGE_AREAS) return -2;
    script_stage.area[i].owner = owner;
    for (j = 0; j < 48; ++j) script_stage.area[i].name[j] = Script_AreaNameByte(j);
    script_stage.loading_area = i + 1;
    OSReport("largemap: area load %s owner=%d slot=%d\n", script_stage.area[i].name, owner, i + 1);
    return i + 1;
}

static void script_area_remove(int area)
{
    int i;
    for (i = 0; i < SCRIPT_MESH_INSTANCES; ++i)
        if (script_stage.instance[i].handle && script_stage.instance[i].area == area)
            ScriptGame_ModelDespawn(script_stage.instance[i].handle);
    for (i = 0; i < script_stage.cap; ++i)
        if (script_stage.line[i].active && script_stage.line[i].area == area)
            ScriptGame_StageRemove(script_stage.line[i].handle);
    for (i = 0; i < SCRIPT_STAGE_TARGETS; ++i)
        if (script_stage.target[i].active && script_stage.target[i].area == area)
            ScriptGame_StageRemove(script_stage.target[i].handle);
    OSReport("largemap: area unload %s slot=%d\n", script_stage.area[area - 1].name, area);
    memset(&script_stage.area[area - 1], 0, sizeof script_stage.area[0]);
}

void ScriptGame_AreaEnd(int success)
{
    int area = script_stage.loading_area;
    script_stage.loading_area = 0;
    if (!success && area) script_area_remove(area);
}

int ScriptGame_AreaUnload(int owner)
{
    int area;
    if (script_stage.loading_area) return -1;
    area = ScriptGame_AreaFind(owner);
    if (area) script_area_remove(area);
    return area != 0;
}

void ScriptGame_AreaUnloadOwner(int owner)
{
    int i;
    for (i = 0; i < SCRIPT_STAGE_AREAS; ++i)
        if (script_stage.area[i].owner == owner) script_area_remove(i + 1);
}

int ScriptGame_StageStat(int field)
{
    int i, n = 0;
    switch (field) {
    case 0: for (i = 0; i < script_stage.cap; ++i) n += script_stage.line[i].active != 0; return n;
    case 1: for (i = 0; i < SCRIPT_MESH_INSTANCES; ++i) n += script_stage.instance[i].handle != 0; return n;
    case 2: return script_stage.target_remaining;
    case 3: for (i = 0; i < SCRIPT_STAGE_AREAS; ++i) n += script_stage.area[i].owner != 0; return n;
    case 4: return script_stage.map ? script_stage.cap : 0;
    case 5: return SCRIPT_MESH_INSTANCES;
    case 6: return SCRIPT_STAGE_TARGETS;
    case 7: return sizeof script_stage;
    case 8: return OSCheckHeap(HSD_GetHeap());
    case 9: /* scene-heap reservation, excluding mpIsland's recycled high-water pool */
        if (!script_stage.map) return 0;
        return sizeof(MapCollData) +
            (script_stage.base_v + 2 * script_stage.cap) * sizeof(Vec2) +
            (script_stage.base_l + script_stage.cap) * sizeof(MapLine) +
            (script_stage.base_j + script_stage.cap) * sizeof(MapJoint) +
            2048 * sizeof(CollVtx) + 1536 * sizeof(CollLine) +
            ScriptGame_StageJointCapacity() * sizeof(CollJoint);
    case 10:
        for (i = 0; i < SCRIPT_MESH_INSTANCES; ++i)
            n += script_stage.instance[i].handle && script_stage.instance[i].field[SM_VISIBLE];
        return n;
    default: return 0;
    }
}

/* Rectangles use world coordinates, just like Stage_Get*Offset. Preserve stage
 * offsets (some stages animate them); only replace the bounds beneath them. */
int ScriptGame_MapBound(int which, int field)
{
    StageBlastZone* b = which ? &stage_info.blast_zone : &stage_info.cam_info.cam_bounds;
    float v = field == 0 ? b->left : field == 1 ? b->right : field == 2 ? b->bottom : b->top;
    return script_mesh_bits(v + (field < 2 ? stage_info.cam_info.cam_x_offset : stage_info.cam_info.cam_y_offset));
}

void ScriptGame_MapBounds(int which, int left, int right, int bottom, int top)
{
    StageBlastZone* b;
    if (which < 0) {
        extern void Camera_LargeMapRestore(void);
        Camera_LargeMapRestore();
        if (script_stage.bounds_saved) {
            stage_info.cam_info.cam_bounds = script_stage.saved_camera;
            stage_info.blast_zone = script_stage.saved_blast;
        }
        script_stage.bounds_saved = 0;
        OSReport("largemap: stage bounds restored\n");
        return;
    }
    if (!script_stage.bounds_saved) {
        script_stage.saved_camera = stage_info.cam_info.cam_bounds;
        script_stage.saved_blast = stage_info.blast_zone;
        script_stage.bounds_saved = 1;
    }
    b = which ? &stage_info.blast_zone : &stage_info.cam_info.cam_bounds;
    b->left = script_mesh_float(left) - stage_info.cam_info.cam_x_offset;
    b->right = script_mesh_float(right) - stage_info.cam_info.cam_x_offset;
    b->bottom = script_mesh_float(bottom) - stage_info.cam_info.cam_y_offset;
    b->top = script_mesh_float(top) - stage_info.cam_info.cam_y_offset;
    OSReport("largemap: %s bounds %f %f %f %f\n", which ? "blast" : "camera", b->left, b->right, b->bottom, b->top);
}

int ScriptGame_MapWide(void) { return script_stage.bounds_saved; }
