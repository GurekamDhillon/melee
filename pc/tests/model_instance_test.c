/* Standalone unit test of the actual game-side model implementation.
 * Collision calls are fakes: this verifies ownership/transactions/snapshot data,
 * not mpLib physics, the PPC retargeter, or the real snapshot engine. */
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <string.h>
#include "../gameworld/script_model.h"
static struct {
    void *map, *draw;
    int cap;
    struct { int active, handle, model_handle; float x0, y0, x1, y1; } line[200];
    ScriptMeshInstance instance[SCRIPT_MESH_INSTANCES];
    struct { int token, refs, instances; } asset[SCRIPT_MESH_ASSETS];
} script_stage, snapshot;
static int input[SM_FIELDS], line_input[6], line_sets, fail_after = -1;
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
        script_stage.line[i].x0 = script_mesh_float(x0); script_stage.line[i].y0 = script_mesh_float(y0);
        script_stage.line[i].x1 = script_mesh_float(x1); script_stage.line[i].y1 = script_mesh_float(y1);
        return handle;
    }
    return -1;
}
static int ScriptGame_StageRemove(int handle)
{
    int i;
    for (i = 0; i < script_stage.cap; ++i) if (script_stage.line[i].handle == handle) {
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

int main(void)
{
    int i;
    script_stage.map = script_stage.draw = &script_stage; script_stage.cap = 2;
    input[SM_SCALE] = script_mesh_bits(1); input[SM_VISIBLE] = 1; input[SM_TINT] = -1;
    line_input[0] = script_mesh_bits(-10); line_input[2] = script_mesh_bits(10);
    line_input[4] = 1; line_input[5] = 3;
    assert(!ScriptGame_ModelRef(0, 7, 0));
    assert(ScriptGame_ModelRef(0, 7, 1));
    fail_after = 1;
    assert(ScriptGame_ModelSpawn(0, 7, 99, 2, 200) == -3);
    assert(!script_stage.line[0].active && !script_stage.line[1].active);
    fail_after = -1;
    assert(ScriptGame_ModelSpawn(0, 7, 100, 1, 200) == 100);
    assert(script_stage.asset[0].instances == 1 && script_stage.line[0].model_handle == 100);
    assert(ScriptGame_ModelLineOwner(200) == 100);
    snapshot = script_stage;
    input[SM_X] = script_mesh_bits(20);
    assert(ScriptGame_ModelSet(100) == 1 && script_stage.line[0].x0 == 10);
    input[SM_TINT] = 0x12345678;
    assert(ScriptGame_ModelSet(100) == 1 && line_sets == 1); /* no phantom collision move */
    input[SM_ROT] = script_mesh_bits(180);
    assert(ScriptGame_ModelSet(100) == -4 && script_stage.line[0].x0 == 10);
    input[SM_ROT] = 0;
    script_stage = snapshot;
    assert(ScriptGame_ModelField(0, SM_X) == 0 && script_stage.line[0].x0 == -10);
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
    puts("model instance lifecycle validation passed");
    return 0;
}
