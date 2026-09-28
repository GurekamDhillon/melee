/* Actual batcher with a recording draw sink. Compile as host C; no GX/game.
 * A tiny capacity exercises triangle-aligned flushes without large fixtures. */
#include <assert.h>
#include <math.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "../gameworld/script_model.h"
#include "../platform/gw.h"
typedef struct {
    unsigned char *mesh, *image, *glow_image;
    int has_normals, has_glow, stride;
} GsStageModel;
static GsStageModel gs_stage_models[2], *gs_model_bound;
static int gs_stage_nmodels = 2;
#define GS_MODEL_BATCH_VERTS 6
static float gs_model_batch[GS_MODEL_BATCH_VERTS][8];
static int gs_model_batch_count;
static int fields[3][SM_FIELDS], draw_count, vertex_count, alpha_draws;
static int dynamic[3], matrix_draws;
static float captured[64][8];
static float gs_camera_float(int i) { union { int i; float f; } v; v.i=i; return v.f; }
static int bits(float f) { union { int i; float f; } v; v.f=f; return v.i; }
static int gw_ScriptGame_ModelField(int slot, int field)
{
    if (field == -1) return slot + 1;
    if (field == -2) return slot == 1 ? 1 : 0;
    if (field == -4) return dynamic[slot];
    return fields[slot][field];
}
void gw_log(const char *fmt, ...) { (void)fmt; }
static void gs_model_draw(int model, const void *view, float local[3][4], unsigned tint, int alpha)
{
    (void)model; (void)view; (void)tint;
    if (!gs_model_batch_count) {
        assert(local[0][0] == -2 && local[1][1] == 3 && local[0][3] == 10);
        ++draw_count; ++matrix_draws;
        return;
    }
    assert(local[0][0] == 1 && local[1][1] == 1 && local[2][2] == 1);
    assert(gs_model_batch_count > 0 && gs_model_batch_count % 3 == 0);
    memcpy(captured + vertex_count, gs_model_batch, gs_model_batch_count * sizeof captured[0]);
    vertex_count += gs_model_batch_count; ++draw_count; alpha_draws += alpha;
}
#include "../platform/gw_script_model_draw.inc"
int main(void)
{
    unsigned char mesh[36 + 96 + 6] = {0}, image = 0;
    int i;
    gw_w32(mesh + 12, 3); gw_w32(mesh + 28, 36); gw_w32(mesh + 32, 132);
    for (i = 0; i < 3; ++i) {
        gw_w16(mesh + 132 + i * 2, (uint16_t)i);
        gw_w32(mesh + 36 + i * 32, bits((float)i));
        gw_w32(mesh + 36 + i * 32 + 20, bits(1));
        gw_w32(mesh + 36 + i * 32 + 24, bits(1));
        fields[i][SM_SCALE] = fields[i][SM_SX] = fields[i][SM_SY] = fields[i][SM_SZ] = bits(1);
        fields[i][SM_TINT] = -1;
    }
    for (i = 0; i < 2; ++i) {
        gs_stage_models[i].mesh = mesh; gs_stage_models[i].image = &image;
        gs_stage_models[i].has_normals = 1; gs_stage_models[i].stride = 32;
    }
    fields[0][SM_SX] = bits(-2); fields[0][SM_SY] = bits(3); fields[0][SM_X] = bits(10);
    gw_Script_ModelDraw(-1, NULL);
    gw_Script_ModelDraw(0, NULL); gw_Script_ModelDraw(1, NULL); gw_Script_ModelDraw(2, NULL);
    gw_Script_ModelDraw(-2, NULL);
    assert(draw_count == 2 && vertex_count == 9); /* cross-model shared-atlas batch */
    assert(captured[0][0] == 10 && captured[1][0] == 6 && captured[2][0] == 8); /* winding */
    assert(captured[0][5] < 0 && captured[0][6] > 0);
    assert(fabsf(captured[0][5] / captured[0][6] + 1.5f) < 0.0001f);
    assert(fabsf(captured[0][5]*captured[0][5] + captured[0][6]*captured[0][6] - 1) < 0.0001f);
    draw_count = vertex_count = 0;
    fields[1][SM_TINT] = 0xFFFFFF80; fields[2][SM_ALPHA] = 1;
    gw_Script_ModelDraw(-1, NULL);
    for (i = 0; i < 3; ++i) gw_Script_ModelDraw(i, NULL);
    gw_Script_ModelDraw(-2, NULL);
    assert(draw_count == 3 && alpha_draws == 2); /* tint/alpha split without reordering */
    dynamic[0] = 1;
    gw_Script_ModelDraw(-1, NULL); gw_Script_ModelDraw(0, NULL); gw_Script_ModelDraw(-2, NULL);
    assert(matrix_draws == 1 && draw_count == 4);
    dynamic[0] = 0;
    gw_Script_ModelDraw(-1, NULL); gw_Script_ModelDraw(0, NULL);
    gs_model_batch_reset(); /* scene end discards queued work before freeing assets */
    assert(gs_model_batch_count == 0 && gs_model_bound == NULL);
    puts("model world-space transform and atlas batching passed");
    return 0;
}
