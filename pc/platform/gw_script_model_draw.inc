/* Transient atlas batcher. Included by gw_script.c and the mocked draw test. */
extern void gw_GXBackgroundFogBegin(void);
extern void gw_GXBackgroundFogEnd(void);
static int gs_batch_model, gs_batch_alpha, gs_batch_layer, gs_batch_background;
static unsigned gs_batch_tint;
static int gs_batch_instances, gs_batch_draws;
static int gs_batch_last_instances[2] = {-1, -1}, gs_batch_last_draws[2] = {-1, -1};
static void gs_model_batch_reset(void)
{
    gs_model_batch_count = gs_batch_instances = gs_batch_draws = 0;
    gs_model_bound = NULL;
    gs_batch_last_instances[0] = gs_batch_last_instances[1] = -1;
    gs_batch_last_draws[0] = gs_batch_last_draws[1] = -1;
}
static void gs_model_batch_flush(const void *view)
{
    float identity[3][4] = {{1,0,0,0}, {0,1,0,0}, {0,0,1,0}};
    float viewport[6];
    if (!gs_model_batch_count) return;
    gs_model_bound = NULL; /* vertex mode and alpha state belong to this batch */
    if (gs_batch_background) {
        gw_GXBackgroundFogBegin();
        GXGetViewportv(viewport);
        /* GX far depth maps to near-zero under Aurora reversed Z. Keep world
         * projection unchanged and retain opaque kit self-occlusion. */
        GXSetViewport(viewport[0], viewport[1], viewport[2], viewport[3],
                      viewport[5] - (viewport[5] - viewport[4]) * .01f, viewport[5]);
    }
    gs_model_draw(gs_batch_model + 1, view, identity, gs_batch_tint, gs_batch_alpha);
    if (gs_batch_background) {
        GXSetViewport(viewport[0], viewport[1], viewport[2], viewport[3], viewport[4], viewport[5]);
        gw_GXBackgroundFogEnd();
    }
    gs_model_batch_count = 0;
    ++gs_batch_draws;
}
void gw_Script_ModelDraw(int slot, const void *view)
{
    float scale, sx, sy, sz, rotation[3][3], x, y, z;
    int asset, alpha, layer, mirrored, background;
    unsigned tint, count, k, voff, ioff;
    const GsStageModel *art, *prev;
    if (slot == -1) {
        gs_model_batch_count = gs_batch_instances = gs_batch_draws = 0;
        gs_model_bound = NULL;
        return;
    }
    if (slot == -2) {
        gs_model_batch_flush(view);
        if (gs_batch_instances && (gs_batch_last_instances[gs_batch_alpha] != gs_batch_instances ||
                                  gs_batch_last_draws[gs_batch_alpha] != gs_batch_draws)) {
            gw_log("script model: %s pass instances=%d draws=%d (per camera)",
                   gs_batch_alpha ? "alpha" : "opaque", gs_batch_instances, gs_batch_draws);
            gs_batch_last_instances[gs_batch_alpha] = gs_batch_instances;
            gs_batch_last_draws[gs_batch_alpha] = gs_batch_draws;
        }
        return;
    }
    if (slot < 0 || !gw_ScriptGame_ModelField(slot, -1)) return;
    asset = gw_ScriptGame_ModelField(slot, -2);
    if (asset < 0 || asset >= gs_stage_nmodels) return;
    art = &gs_stage_models[asset];
    tint = (unsigned)gw_ScriptGame_ModelField(slot, SM_TINT);
    alpha = gw_ScriptGame_ModelField(slot, SM_ALPHA) || (tint & 255) != 255;
    layer = gw_ScriptGame_ModelField(slot, SM_LAYER);
    background = gw_ScriptGame_ModelField(slot, SM_BACKGROUND);
    prev = gs_model_batch_count ? &gs_stage_models[gs_batch_model] : NULL;
    /* Different parts may share an atlas. No shader/material boundary survives
     * export except alpha; glow remains a TEV stage in the same submission. */
    if (prev && (prev->image != art->image || prev->glow_image != art->glow_image ||
                 prev->custom_material != art->custom_material || /* a batch draws with one material */
                 prev->has_normals != art->has_normals || prev->has_glow != art->has_glow ||
                 gs_batch_tint != tint || gs_batch_alpha != alpha || gs_batch_layer != layer ||
                 gs_batch_background != background))
        gs_model_batch_flush(view);
    gs_batch_model = asset; gs_batch_tint = tint; gs_batch_alpha = alpha; gs_batch_layer = layer;
    gs_batch_background = background;
    ++gs_batch_instances;
    scale = gs_camera_float(gw_ScriptGame_ModelField(slot, SM_SCALE));
    sx = scale * gs_camera_float(gw_ScriptGame_ModelField(slot, SM_SX));
    sy = scale * gs_camera_float(gw_ScriptGame_ModelField(slot, SM_SY));
    sz = scale * gs_camera_float(gw_ScriptGame_ModelField(slot, SM_SZ));
    script_model_rotation(gs_camera_float(gw_ScriptGame_ModelField(slot, SM_RX)),
                          gs_camera_float(gw_ScriptGame_ModelField(slot, SM_RY)),
                          gs_camera_float(gw_ScriptGame_ModelField(slot, SM_ROT)), rotation);

    x = gs_camera_float(gw_ScriptGame_ModelField(slot, SM_X));
    y = gs_camera_float(gw_ScriptGame_ModelField(slot, SM_Y));
    z = gs_camera_float(gw_ScriptGame_ModelField(slot, SM_Z));
    if (gw_ScriptGame_ModelField(slot, -4)) {
        /* Static room parts batch; moved instances keep stable local geometry
         * and matrix loads, which Aurora's frame replay can interpolate. */
        float local[3][4] = {{0}};
        float viewport[6];
        gs_model_batch_flush(view);
        for (k = 0; k < 3; ++k) {
            local[k][0] = rotation[k][0]*sx;
            local[k][1] = rotation[k][1]*sy;
            local[k][2] = rotation[k][2]*sz;
        }
        local[0][3] = x; local[1][3] = y; local[2][3] = z;
        gs_model_bound = NULL;
        if (background) {
            gw_GXBackgroundFogBegin();
            GXGetViewportv(viewport);
            GXSetViewport(viewport[0], viewport[1], viewport[2], viewport[3],
                          viewport[5] - (viewport[5] - viewport[4]) * .01f, viewport[5]);
        }
        gs_model_draw(asset + 1, view, local, tint, alpha);
        if (background) {
            GXSetViewport(viewport[0], viewport[1], viewport[2], viewport[3], viewport[4], viewport[5]);
            gw_GXBackgroundFogEnd();
        }
        ++gs_batch_draws;
        return;
    }
    mirrored = sx * sy * sz < 0;
    count = gw_r32(art->mesh + 12); voff = gw_r32(art->mesh + 28); ioff = gw_r32(art->mesh + 32);
    for (k = 0; k < count; ++k) {
        unsigned index = k;
        const unsigned char *v;
        float *out, px, py, pz;
        if (gs_model_batch_count == GS_MODEL_BATCH_VERTS) gs_model_batch_flush(view);
        /* Preserve front faces under odd reflections, even with two-sided glass. */
        if (mirrored && k % 3) index = k - k % 3 + (3 - k % 3);
        v = art->mesh + voff + gw_r16(art->mesh + ioff + index * 2) * art->stride;
        out = gs_model_batch[gs_model_batch_count++];
        px = gw_rf32(v) * sx; py = gw_rf32(v + 4) * sy;
        pz = gw_rf32(v + 8)*sz;
        out[0] = x + rotation[0][0]*px + rotation[0][1]*py + rotation[0][2]*pz;
        out[1] = y + rotation[1][0]*px + rotation[1][1]*py + rotation[1][2]*pz;
        out[2] = z + rotation[2][0]*px + rotation[2][1]*py + rotation[2][2]*pz;
        out[3] = gw_rf32(v + 12); out[4] = gw_rf32(v + 16);
        if (art->has_normals) {
            /* Inverse transpose, then normalize: nonuniform scale must not
             * change brightness. Do not multiply normals by determinant sign. */
            float nx = gw_rf32(v + 20) / sx, ny = gw_rf32(v + 24) / sy;
            float nz = gw_rf32(v + 28) / sz, length = sqrtf(nx*nx + ny*ny + nz*nz);
            if (length == 0) length = 1;
            for (index = 0; index < 3; ++index)
                out[5 + index] = (rotation[index][0]*nx + rotation[index][1]*ny + rotation[index][2]*nz)/length;
        }
    }
}

