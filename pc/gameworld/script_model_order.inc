/* Transient per-camera order. HSD pass 0 is opaque, pass 2 is translucent
 * (HSD_GObj_80390EB8); transparent geometry must see all opaque depth first. */
/* 0: every instance; 1: only translucent parts at or beyond the player plane (drawn before
 * fighters/effects); 2: only parts nearer the camera than that plane (drawn after them). */
static int script_mesh_split;
static int script_mesh_order_bucket(Mtx view, int alpha, int background, int order[SCRIPT_MESH_INSTANCES])
{
    int count = 0, i, n;
    float depth[SCRIPT_MESH_INSTANCES];
    for (i = 0; i < SCRIPT_MESH_INSTANCES; ++i) {
        ScriptMeshInstance* m = &script_stage.instance[i];
        float scale, r[3][3], x, y, z, cx, cy, cz;
        if (!m->handle || !ScriptGame_AreaVisible(m->area) || !m->field[SM_VISIBLE] || !!SM_IS_ALPHA(m) != alpha || !!m->field[SM_BACKGROUND] != background) continue;
        scale = script_mesh_float(m->field[SM_SCALE]);
        script_model_rotation(script_mesh_float(m->field[SM_RX]),
                              script_mesh_float(m->field[SM_RY]),
                              script_mesh_float(m->field[SM_ROT]), r);
        cx = m->center[0] * scale * script_mesh_float(m->field[SM_SX]);
        cy = m->center[1] * scale * script_mesh_float(m->field[SM_SY]);
        cz = m->center[2] * scale * script_mesh_float(m->field[SM_SZ]);
        x = script_mesh_float(m->field[SM_X]) + r[0][0]*cx + r[0][1]*cy + r[0][2]*cz;
        y = script_mesh_float(m->field[SM_Y]) + r[1][0]*cx + r[1][1]*cy + r[1][2]*cz;
        z = script_mesh_float(m->field[SM_Z]) + r[2][0]*cx + r[2][1]*cy + r[2][2]*cz;
        depth[i] = view[2][0] * x + view[2][1] * y + view[2][2] * z + view[2][3];
        if (script_mesh_split && ((script_mesh_split == 2) != (depth[i] > view[2][3]))) continue; /* eye -Z is forward */
        for (n = count; n > 0; --n) {
            ScriptMeshInstance* prev = &script_stage.instance[order[n - 1]];
            /* Eye-space forward is -Z: most negative first. Layers only break
             * alpha depth ties; batching must never override depth order. */
            if (alpha && depth[order[n - 1]] < depth[i]) break;
            if (!alpha || depth[order[n - 1]] == depth[i]) {
                if (prev->field[SM_LAYER] < m->field[SM_LAYER]) break;
                if (prev->field[SM_LAYER] == m->field[SM_LAYER] &&
                    (alpha || prev->asset <= m->asset)) break;
            }
            order[n] = order[n - 1];
        }
        order[n] = i;
        ++count;
    }
    return count;
}
static int script_mesh_order(Mtx view, int alpha, int order[SCRIPT_MESH_INSTANCES])
{
    return script_mesh_order_bucket(view, alpha, 0, order);
}
