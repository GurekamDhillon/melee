/* arena-hooks: included in script_game.c so all state is in its snapshotted BSS.
 * Extents are origin-relative, like Ground_801C39C0/801C3BB4. Stage_Get*Offset
 * consults this override so a native scrolling-stage update cannot undo a lock.
 * No native pointers or host timers participate in simulation. */
static struct {
    int owner, frames, groups;
    float x, y, target_x, target_y, saved_x, saved_y;
    StageBlastZone camera, blast, saved_camera, saved_blast;
    int group_owner[256]; /* mpLibLoad's fixed collision-joint capacity */
    u8 group_enabled[256];
    u8 line_enabled[1536]; /* preserve individually disabled lines on unload */
} script_arena;

static void script_arena_apply(void)
{
    if (!script_arena.owner) return;
    Ground_801C38BC(script_arena.x, script_arena.y);
    stage_info.cam_info.cam_bounds = script_arena.camera;
    stage_info.blast_zone = script_arena.blast;
}

int ScriptGame_ArenaOwner(void) { return script_arena.owner; }

static void script_arena_claim(int owner)
{
    if (script_arena.owner) return;
    script_arena.owner = owner;
    script_arena.x = script_arena.saved_x = stage_info.cam_info.cam_x_offset;
    script_arena.y = script_arena.saved_y = stage_info.cam_info.cam_y_offset;
    script_arena.camera = script_arena.saved_camera = stage_info.cam_info.cam_bounds;
    script_arena.blast = script_arena.saved_blast = stage_info.blast_zone;
}

void ScriptGame_ArenaOrigin(int owner, int x, int y, int frames)
{
    script_arena_claim(owner);
    script_arena.target_x = script_mesh_float(x);
    script_arena.target_y = script_mesh_float(y);
    script_arena.frames = frames;
    if (!frames) {
        script_arena.x = script_arena.target_x;
        script_arena.y = script_arena.target_y;
    }
    script_arena_apply();
}

void ScriptGame_ArenaBounds(int owner, int blast, int l, int r, int t, int b)
{
    StageBlastZone* bounds;
    script_arena_claim(owner);
    bounds = blast ? &script_arena.blast : &script_arena.camera;
    bounds->left = script_mesh_float(l);
    bounds->right = script_mesh_float(r);
    bounds->top = script_mesh_float(t);
    bounds->bottom = script_mesh_float(b);
    script_arena_apply();
}

void ScriptGame_ArenaRestore(void)
{
    if (!script_arena.owner) return;
    Ground_801C38BC(script_arena.saved_x, script_arena.saved_y);
    stage_info.cam_info.cam_bounds = script_arena.saved_camera;
    stage_info.blast_zone = script_arena.saved_blast;
    script_arena.owner = script_arena.frames = 0;
}

/* Called even during LAB resimulation, once per completed logic frame. Mirrors
 * grKinokoRoute_80207C88's (target-current)/remaining 60-tick transition. */
static void script_arena_frame(void)
{
    if (!script_arena.owner) return;
    if (script_arena.frames > 0) {
        script_arena.x += (script_arena.target_x - script_arena.x) / script_arena.frames;
        script_arena.y += (script_arena.target_y - script_arena.y) / script_arena.frames;
        if (--script_arena.frames == 0) {
            script_arena.x = script_arena.target_x;
            script_arena.y = script_arena.target_y;
        }
    }
    script_arena_apply();
}

/* Game-to-game accessor, not the native ABI. Fields 0..7 are camera L/R/T/B,
 * blast L/R/T/B; 8/9 are origin X/Y. */
float ScriptGame_ArenaBound(int field, float fallback)
{
    StageBlastZone* b;
    float value;
    if (!script_arena.owner) return fallback;
    if (field == 8) return script_arena.x;
    if (field == 9) return script_arena.y;
    b = field < 4 ? &script_arena.camera : &script_arena.blast;
    field &= 3;
    value = field == 0 ? b->left : field == 1 ? b->right : field == 2 ? b->top : b->bottom;
    return value + (field < 2 ? script_arena.x : script_arena.y);
}

int ScriptGame_ArenaRead(int field)
{
    StageBlastZone* b;
    float value;
    if (field == 10) return script_arena.frames;
    if (field == 8 || field == 9) {
        value = field == 8 ? stage_info.cam_info.cam_x_offset : stage_info.cam_info.cam_y_offset;
        if (script_arena.owner) value = field == 8 ? script_arena.x : script_arena.y;
        return script_mesh_bits(value);
    }
    b = field < 4 ? &stage_info.cam_info.cam_bounds : &stage_info.blast_zone;
    value = (field & 3) == 0 ? b->left : (field & 3) == 1 ? b->right : (field & 3) == 2 ? b->top : b->bottom;
    value += (field & 3) < 2 ? stage_info.cam_info.cam_x_offset : stage_info.cam_info.cam_y_offset;
    return script_mesh_bits(ScriptGame_ArenaBound(field, value));
}

int ScriptGame_ArenaGroup(int id, int field)
{
    CollJoint* joints = mpGetGroundCollJoint();
    if (id < 0) return script_arena.groups;
    if (!joints || id >= script_arena.groups) return -1;
    if (field == 1) return script_arena.group_owner[id];
    return (joints[id].flags & CollJoint_Enabled) != 0;
}

void ScriptGame_ArenaGroupSet(int owner, int id, int enabled)
{
    CollJoint* joint = &mpGetGroundCollJoint()[id];
    int k, n;
    if (!script_arena.group_owner[id]) {
        script_arena.group_owner[id] = owner;
        script_arena.group_enabled[id] = (joint->flags & CollJoint_Enabled) != 0;
        for (k = 0; k < MapLineGroup_Count; ++k) {
            struct MapLineRange* range = &joint->inner->ranges[k];
            for (n = range->start; n < range->start + range->count; ++n)
                script_arena.line_enabled[n] = (mpGetGroundCollLine()[n].flags & LINE_FLAG_ENABLED) != 0;
        }
    }
    /* The same collision list primitives used by grKinokoRoute_8020836C. */
    if (enabled) mpJointListAdd(id); else mpLib_80057BC0(id);
}

void ScriptGame_ArenaRelease(int owner)
{
    int id, k, n;
    if (script_arena.owner == owner) ScriptGame_ArenaRestore();
    for (id = 0; id < script_arena.groups; ++id) {
        CollJoint* joint;
        if (script_arena.group_owner[id] != owner) continue;
        joint = &mpGetGroundCollJoint()[id];
        if (script_arena.group_enabled[id]) mpJointListAdd(id); else mpLib_80057BC0(id);
        for (k = 0; k < MapLineGroup_Count; ++k) {
            struct MapLineRange* range = &joint->inner->ranges[k];
            for (n = range->start; n < range->start + range->count; ++n) {
                CollLine* line = &mpGetGroundCollLine()[n];
                if (script_arena.line_enabled[n]) line->flags |= LINE_FLAG_ENABLED;
                else line->flags &= ~LINE_FLAG_ENABLED;
            }
        }
        script_arena.group_owner[id] = 0;
    }
}
