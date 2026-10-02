/* Standalone unit test of the actual game-side model implementation.
 * Collision calls are fakes: this verifies ownership/transactions/snapshot data,
 * not mpLib physics, the PPC retargeter, or the real snapshot engine. */
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <string.h>
#include "../gameworld/script_model.h"
#define SCRIPT_STAGE_LINES 200
#define SCRIPT_STAGE_MODELS 64
#define SCRIPT_STAGE_TARGETS 32
static struct {
    void *map, *draw;
    int cap;
    struct { int active, handle, model_handle, owner; float x0, y0, x1, y1; } line[200];
    struct { int active, handle, owner; } model[SCRIPT_STAGE_MODELS], target[SCRIPT_STAGE_TARGETS];
    ScriptMeshInstance instance[SCRIPT_MESH_INSTANCES];
    struct { int token, refs, instances; } asset[SCRIPT_MESH_ASSETS];
    struct { int owner, refs[SCRIPT_MESH_ASSETS]; } model_owner[SCRIPT_MESH_OWNERS];
} script_stage, snapshot;
static int input[SM_FIELDS + 3], line_input[6], line_sets, fail_after = -1;
static int current_owner, alive_owner, remove_refused;
static int Script_StageResourceOwner(void) { return current_owner; }
static int Script_StageResourceOwnerAlive(int owner) { return owner == alive_owner; }
static int script_mesh_bits(float f) { union { float f; int i; } u; u.f = f; return u.i; }
static float script_mesh_float(int i) { union { float f; int i; } u; u.i = i; return u.f; }
static int Script_ModelInput(int field, int line) { return line < 0 ? input[field] : line_input[field]; }
static int ScriptGame_StageAddLine(int x0, int y0, int x1, int y1, int kind, int flags, int handle)
{
    int i;
    (void)kind; (void)flags;
    if (fail_after == 0) return -1;
    if (fail_after > 0) --fail_after;
    for (i = 0; i < script_stage.cap; ++i) if (!script_stage.line[i].active) {
        script_stage.line[i].active = 1; script_stage.line[i].handle = handle;
        script_stage.line[i].owner = current_owner;
        script_stage.line[i].x0 = script_mesh_float(x0); script_stage.line[i].y0 = script_mesh_float(y0);
        script_stage.line[i].x1 = script_mesh_float(x1); script_stage.line[i].y1 = script_mesh_float(y1);
        return handle;
    }
    return -1;
}
static int ScriptGame_StageRemove(int handle)
{
    int i;
    if (!script_stage.map || remove_refused == handle) return 0;
    for (i = 0; i < script_stage.cap; ++i) if (script_stage.line[i].active && script_stage.line[i].handle == handle) {
        script_stage.line[i].active = 0; return 1;
    }
    return 0;
}
static int script_stage_line_set(int handle, float x0, float y0, float x1, float y1)
{
    int i;
    ++line_sets;
    for (i = 0; i < script_stage.cap; ++i) if (script_stage.line[i].handle == handle) {
        script_stage.line[i].x0 = x0; script_stage.line[i].y0 = y0;
        script_stage.line[i].x1 = x1; script_stage.line[i].y1 = y1; return 1;
    }
    return 0;
}
#define OSReport(...) ((void)0)
#include "../gameworld/script_model.inc"
#include "../gameworld/script_stage_owner.inc"

int main(void)
{
    int i;
    script_stage.map = script_stage.draw = &script_stage; script_stage.cap = 2;
    input[SM_SCALE] = script_mesh_bits(1); input[SM_VISIBLE] = 1; input[SM_TINT] = -1;
    input[SM_SX] = input[SM_SY] = input[SM_SZ] = script_mesh_bits(1);
    line_input[0] = script_mesh_bits(-10); line_input[2] = script_mesh_bits(10);
    line_input[4] = 1; line_input[5] = 3;
    assert(!ScriptGame_ModelRef(0, 7, 0));
    assert(ScriptGame_ModelRef(0, 7, 1));
    /* A reflected slope must keep its height and reverse its endpoint order.
     * A vertical reflection becomes a ceiling; walls exchange left/right. */
    input[SM_SX] = script_mesh_bits(-2);
    input[SM_SY] = script_mesh_bits(3);
    line_input[3] = script_mesh_bits(4);
    assert(ScriptGame_ModelSpawn(0, 7, 90, 1, 190) == 90);
    assert(script_stage.line[0].x0 == -20 && script_stage.line[0].y0 == 12);
    assert(script_stage.line[0].x1 == 20 && script_stage.line[0].y1 == 0);
    assert(ScriptGame_ModelDespawn(90));
    input[SM_SX] = script_mesh_bits(1); input[SM_SY] = script_mesh_bits(-1);
    assert(ScriptGame_ModelSpawn(0, 7, 91, 1, 191) == 91);
    assert(script_stage.instance[0].line[0].kind == 2);
    assert(script_stage.line[0].x0 == 10 && script_stage.line[0].y0 == -4);
    input[SM_SY] = script_mesh_bits(1);
    assert(ScriptGame_ModelSet(91) == -4); /* changing owned collision kind needs respawn */
    assert(script_stage.instance[0].line[0].kind == 2);
    assert(ScriptGame_ModelDespawn(91));
    /* X reflection exchanges walls and retains directed endpoint convention. */
    input[SM_SX] = script_mesh_bits(-1);
    line_input[0] = line_input[2] = 0;
    line_input[1] = script_mesh_bits(4); line_input[3] = 0;
    line_input[4] = 3; line_input[5] = 0;
    assert(ScriptGame_ModelSpawn(0, 7, 92, 1, 192) == 92);
    assert(script_stage.instance[0].line[0].kind == 4);
    assert(script_stage.line[0].y0 == 0 && script_stage.line[0].y1 == 4);
    assert(ScriptGame_ModelDespawn(92));
    input[SM_SX] = script_mesh_bits(1);
    line_input[0] = script_mesh_bits(-10); line_input[2] = script_mesh_bits(10);
    line_input[1] = 0; line_input[4] = 1; line_input[5] = 3;
    line_input[3] = 0;
    fail_after = 1;
    assert(ScriptGame_ModelSpawn(0, 7, 99, 2, 200) == -3);
    assert(!script_stage.line[0].active && !script_stage.line[1].active);
    fail_after = -1;
    assert(ScriptGame_ModelSpawn(0, 7, 100, 1, 200) == 100);
    assert(script_stage.asset[0].instances == 1 && script_stage.line[0].model_handle == 100);
    assert(ScriptGame_ModelLineOwner(200) == 100);
    snapshot = script_stage;
    input[SM_ALPHA] = 1; input[SM_SZ] = script_mesh_bits(-2);
    input[SM_X] = script_mesh_bits(20);
    assert(ScriptGame_ModelSet(100) == 1 && script_stage.line[0].x0 == 10);
    assert(ScriptGame_ModelField(0, -4) == 1);
    input[SM_TINT] = 0x12345678;
    assert(ScriptGame_ModelSet(100) == 1 && line_sets == 1); /* no phantom collision move */
    input[SM_ROT] = script_mesh_bits(180);
    assert(ScriptGame_ModelSet(100) == -4 && script_stage.line[0].x0 == 10);
    input[SM_ROT] = 0;
    script_stage = snapshot;
    assert(ScriptGame_ModelField(0, SM_X) == 0 && script_stage.line[0].x0 == -10);
    assert(ScriptGame_ModelField(0, -4) == 0);
    assert(ScriptGame_ModelField(0, SM_ALPHA) == 0 &&
           ScriptGame_ModelField(0, SM_SZ) == script_mesh_bits(1));
    assert(ScriptGame_ModelField(0, -1) == 100 && ScriptGame_ModelField(0, -3) == 1);
    assert(ScriptGame_ModelDespawn(100) && !ScriptGame_ModelDespawn(100));
    assert(!script_stage.line[0].active && script_stage.asset[0].instances == 0);
    assert(!ScriptGame_ModelLineOwner(200));
    script_stage = snapshot; /* resurrection needs the original asset and owned lines */
    assert(ScriptGame_ModelSlot(100) == 0 && script_stage.asset[0].instances == 1);
    assert(ScriptGame_ModelRef(0, 7, -1));
    assert(ScriptGame_ModelSpawn(0, 7, 101, 0, 0) == -1); /* released load reference */
    assert(ScriptGame_ModelSet(100) == 1); /* instance owns its own reference */
    assert(ScriptGame_ModelRef(0, 7, 1));
    assert(ScriptGame_ModelSpawn(0, 7, 101, 2, 201) == -3); /* no partial capacity claim */
    for (i = 1; i < SCRIPT_MESH_INSTANCES; ++i)
        assert(ScriptGame_ModelSpawn(0, 7, 100 + i, 0, 0) == 100 + i);
    assert(ScriptGame_ModelSpawn(0, 7, 999, 0, 0) == -2);
    assert(ScriptGame_ModelDespawn(100));
    assert(ScriptGame_ModelSpawn(0, 7, 999, 0, 0) == 999 && ScriptGame_ModelSlot(100) == -1);
    /* Two script generations share one immutable asset, but neither owns the
     * other's load retains, instances or collision lines. */
    memset(&script_stage, 0, sizeof script_stage);
    script_stage.map = script_stage.draw = &script_stage; script_stage.cap = 8;
    current_owner = 41;
    assert(ScriptGame_ModelRefOwned(0, 17, 41, 1));
    assert(ScriptGame_ModelRefOwned(0, 17, 41, 1));
    assert(!ScriptGame_ModelRefOwned(0, 17, 42, 0));
    assert(!ScriptGame_ModelRefOwned(0, 17, 42, -1));
    assert(ScriptGame_ModelSpawn(0, 17, 400, 2, 401) == 400);
    assert(ScriptGame_ModelSpawn(0, 17, 403, 0, 0) == 403);
    assert(ScriptGame_StageAddLine(0, 0, script_mesh_bits(10), 0, 1, 0, 404) == 404);
    current_owner = 42; alive_owner = 42;
    assert(ScriptGame_ModelSpawn(0, 17, 500, 0, 0) == -1); /* foreign retain */
    assert(ScriptGame_ModelRefOwned(0, 17, 42, 1));
    assert(ScriptGame_ModelSpawn(0, 17, 500, 1, 501) == 500);
    assert(ScriptGame_StageAddLine(0, 0, script_mesh_bits(10), 0, 1, 0, 502) == 502);
    assert(ScriptGame_StageResourceAccess(400, 41) == 1);
    assert(ScriptGame_StageResourceAccess(400, 42) == -1);
    assert(ScriptGame_StageResourceAccess(9999, 42) == 0);
    snapshot = script_stage;
    /* Removal can partially succeed. Preserve the failed instance and its
     * asset count until the outstanding line is actually gone. */
    remove_refused = 402;
    assert(ScriptGame_StageReleaseOwner(41) == 2);
    assert(ScriptGame_ModelSlot(400) >= 0 && ScriptGame_ModelSlot(403) < 0);
    assert(script_stage.asset[0].instances == 2 && script_stage.asset[0].refs == 1);
    assert(ScriptGame_ModelRefOwned(0, 17, 42, 0));
    assert(!ScriptGame_ModelRefOwned(0, 17, 41, 0));
    remove_refused = 0;
    assert(ScriptGame_StageReleaseDeadOwners() == 0);
    assert(ScriptGame_ModelSlot(400) < 0 && ScriptGame_ModelSlot(500) >= 0);
    assert(script_stage.asset[0].instances == 1 && script_stage.asset[0].refs == 1);
    assert(!memcmp(&script_stage.instance[2], &snapshot.instance[2], sizeof script_stage.instance[2]));
    assert(!memcmp(&script_stage.line[3], &snapshot.line[3], sizeof script_stage.line[3]));
    assert(!memcmp(&script_stage.line[4], &snapshot.line[4], sizeof script_stage.line[4]));
    assert(ScriptGame_StageResourceAccess(501, 42) == 1 && ScriptGame_StageResourceAccess(502, 42) == 1);
    assert(ScriptGame_StageReleaseOwner(41) == 0); /* idempotent */
    /* A stale/absent map must refuse collision cleanup without dropping
     * handles. Once the map returns, the same registry supports retry. */
    script_stage = snapshot; script_stage.map = NULL;
    assert(ScriptGame_StageReleaseOwner(41) > 0 && ScriptGame_ModelSlot(400) >= 0);
    script_stage.map = &script_stage;
    assert(ScriptGame_StageReleaseDeadOwners() == 0);
    assert(ScriptGame_ModelSlot(500) >= 0 && script_stage.asset[0].refs == 1);
    /* Restoring the old snapshot resurrects owner 41, not a replacement
     * owner's generation. Reconciliation retires 41 and preserves live 42. */
    script_stage = snapshot;
    assert(ScriptGame_StageReleaseDeadOwners() == 0);
    assert(ScriptGame_ModelSlot(400) < 0 && ScriptGame_ModelSlot(500) >= 0);
    assert(ScriptGame_ModelRefOwned(0, 17, 42, -1));
    assert(ScriptGame_ModelSet(500) == 1); /* instance outlives explicit load release */
    assert(ScriptGame_StageReleaseOwner(42) == 0 && !script_stage.asset[0].instances);
    memset(&script_stage, 0, sizeof script_stage); /* genuine scene reset */
    assert(ScriptGame_StageReleaseOwner(42) == 0 && ScriptGame_StageReleaseDeadOwners() == 0);
    for (i = 1; i <= SCRIPT_MESH_OWNERS; ++i) assert(ScriptGame_ModelRefOwned(0, 33, i, 1));
    assert(!ScriptGame_ModelRefOwned(0, 33, 65, 1) && script_stage.asset[0].refs == 64);
    assert(ScriptGame_StageReleaseOwner(1) == 0 && script_stage.asset[0].refs == 63);
    assert(ScriptGame_ModelRefOwned(0, 33, 65, 1));
    alive_owner = 65;
    assert(ScriptGame_StageReleaseDeadOwners() == 0 && script_stage.asset[0].refs == 1);
    assert(ScriptGame_StageReleaseOwner(65) == 0 && !script_stage.asset[0].refs);
    puts("model instance lifecycle validation passed");
    return 0;
}
