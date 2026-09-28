/* Transient per-camera order. HSD pass 0 is opaque, pass 2 is translucent
 * (HSD_GObj_80390EB8); transparent geometry must see all opaque depth first. */
static int script_mesh_order(Mtx view, int alpha, int order[SCRIPT_MESH_INSTANCES])
{
    int count = 0, i, n;
    float depth[SCRIPT_MESH_INSTANCES];
    for (i = 0; i < SCRIPT_MESH_INSTANCES; ++i) {
        ScriptMeshInstance* m = &script_stage.instance[i];
        float scale, rot, c, s, x, y, z, cx, cy;
        if (!m->handle || !m->field[SM_VISIBLE] || !!SM_IS_ALPHA(m) != alpha) continue;
        scale = script_mesh_float(m->field[SM_SCALE]);
        rot = script_mesh_float(m->field[SM_ROT]) * 0.017453292519943295f;
        c = cosf(rot); s = sinf(rot);
        cx = m->center[0] * scale * script_mesh_float(m->field[SM_SX]);
        cy = m->center[1] * scale * script_mesh_float(m->field[SM_SY]);
        x = script_mesh_float(m->field[SM_X]) + c * cx - s * cy;
        y = script_mesh_float(m->field[SM_Y]) + s * cx + c * cy;
        z = script_mesh_float(m->field[SM_Z]) + m->center[2] * scale * script_mesh_float(m->field[SM_SZ]);
        depth[i] = view[2][0] * x + view[2][1] * y + view[2][2] * z + view[2][3];
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
