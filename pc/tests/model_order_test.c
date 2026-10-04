/* Host-only test of the actual camera/pass ordering; no GX or game required. */
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include "../gameworld/script_model.h"
typedef float Mtx[3][4];
static struct { ScriptMeshInstance instance[SCRIPT_MESH_INSTANCES]; } script_stage;
static float script_mesh_float(int i) { union { int i; float f; } v; v.i = i; return v.f; }
static int bits(float f) { union { int i; float f; } v; v.f = f; return v.i; }
static int live_area[64];
static int ScriptGame_AreaVisible(int area) { return !area || live_area[area - 1]; }
#include "../gameworld/script_model_order.inc"
int main(void)
{
    Mtx view = {{1,0,0,0}, {0,1,0,0}, {0,0,1,-100}};
    int order[SCRIPT_MESH_INSTANCES], i;
    for (i = 0; i < 5; ++i) {
        ScriptMeshInstance* m = &script_stage.instance[i];
        m->handle = i + 1; m->field[SM_VISIBLE] = 1; m->field[SM_TINT] = -1;
        m->field[SM_SCALE] = m->field[SM_SX] = m->field[SM_SY] = m->field[SM_SZ] = bits(1);
        m->asset = 5 - i;
    }
    script_stage.instance[0].field[SM_ALPHA] = 1;
    script_stage.instance[1].field[SM_TINT] = 0xFFFFFF80;
    script_stage.instance[0].field[SM_Z] = bits(20);
    script_stage.instance[1].field[SM_Z] = bits(-20);
    script_stage.instance[1].field[SM_LAYER] = 8; /* layer cannot trump distance */
    script_stage.instance[4].field[SM_VISIBLE] = 0;
    assert(script_mesh_order(view, 0, order) == 2 && order[0] == 3 && order[1] == 2);
    assert(script_mesh_order(view, 1, order) == 2 && order[0] == 1 && order[1] == 0);
    view[2][2] = -1; /* camera on the other side reverses the transparent order */
    assert(script_mesh_order(view, 1, order) == 2 && order[0] == 0 && order[1] == 1);
    script_stage.instance[1].center[2] = 100;
    assert(script_mesh_order(view, 1, order) == 2 && order[0] == 1 && order[1] == 0);
    script_stage.instance[1].center[2] = 0;
    script_stage.instance[0].field[SM_Z] = bits(-20);
    script_stage.instance[1].field[SM_LAYER] = 0;
    assert(script_mesh_order(view, 1, order) == 2 && order[0] == 0 && order[1] == 1);
    script_stage.instance[0].field[SM_BACKGROUND] = 1;
    script_stage.instance[2].field[SM_BACKGROUND] = 1;
    assert(script_mesh_order(view, 0, order) == 1 && order[0] == 3);
    assert(script_mesh_order(view, 1, order) == 1 && order[0] == 1);
    assert(script_mesh_order_bucket(view, 0, 1, order) == 1 && order[0] == 2);
    assert(script_mesh_order_bucket(view, 1, 1, order) == 1 && order[0] == 0);
    script_stage.instance[0].field[SM_BACKGROUND] = 0;
    script_stage.instance[0].field[SM_Z] = 0;
    script_stage.instance[0].center[0] = 50;
    script_stage.instance[0].field[SM_RY] = bits(90);
    view[2][2] = 1;
    assert(script_mesh_order(view, 1, order) == 2 && order[0] == 0);
    script_stage.instance[0].field[SM_RY] = bits(-90);
    assert(script_mesh_order(view, 1, order) == 2 && order[0] == 1);
    script_stage.instance[0].area = 1;
    assert(script_mesh_order(view, 1, order) == 1 && order[0] == 1);
    live_area[0] = 1;
    assert(script_mesh_order(view, 1, order) == 2 && order[0] == 1);
    live_area[0] = 0;
    assert(script_mesh_order(view, 1, order) == 1 && order[0] == 1);
    puts("model pass/depth/stable ordering passed");
    return 0;
}
