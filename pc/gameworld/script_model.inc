/* Runtime GXMS instances. Included by script_game.c, so both MEM1 data and these
 * statics use its existing snapshot registration. No native asset pointers. */
int ScriptGame_ModelRef(int asset, int token, int delta)
{
    if (asset < 0 || asset >= SCRIPT_MESH_ASSETS) return 0;
    if (delta > 0) {
        if (script_stage.asset[asset].instances && script_stage.asset[asset].token != token) return 0;
        if (script_stage.asset[asset].refs == 0) script_stage.asset[asset].token = token;
        if (script_stage.asset[asset].token != token || script_stage.asset[asset].refs == 0x7FFFFFFF) return 0;
        ++script_stage.asset[asset].refs;
    } else if (script_stage.asset[asset].token != token || !script_stage.asset[asset].refs) return 0;
    else if (delta < 0) --script_stage.asset[asset].refs;
    return 1;
}

/* A load reference belongs to one script generation. Assets and atlas bytes stay
 * scene-pinned; retiring a script releases only its scalar retain counts. */
int ScriptGame_ModelRefOwned(int asset, int token, int owner, int delta)
{
    int i, free_slot = -1;
    if (owner <= 0 || asset < 0 || asset >= SCRIPT_MESH_ASSETS) return 0;
    for (i = 0; i < SCRIPT_MESH_OWNERS; ++i) {
        if (script_stage.model_owner[i].owner == owner) break;
        if (!script_stage.model_owner[i].owner && free_slot < 0) free_slot = i;
    }
    if (delta <= 0) {
        int j;
        if (i == SCRIPT_MESH_OWNERS || !script_stage.model_owner[i].refs[asset] ||
            !ScriptGame_ModelRef(asset, token, delta)) return 0;
        if (delta == 0) return 1;
        --script_stage.model_owner[i].refs[asset];
        for (j = 0; j < SCRIPT_MESH_ASSETS; ++j)
            if (script_stage.model_owner[i].refs[j]) return 1;
        script_stage.model_owner[i].owner = 0;
        return 1;
    }
    if (i == SCRIPT_MESH_OWNERS) i = free_slot;
    if (i < 0 || !ScriptGame_ModelRef(asset, token, 1)) return 0;
    script_stage.model_owner[i].owner = owner;
    ++script_stage.model_owner[i].refs[asset];
    return 1;
}

static void script_model_release_owner_refs(int owner)
{
    int i, j;
    for (i = 0; i < SCRIPT_MESH_OWNERS; ++i) {
        if (script_stage.model_owner[i].owner != owner) continue;
        for (j = 0; j < SCRIPT_MESH_ASSETS; ++j) {
            script_stage.asset[j].refs -= script_stage.model_owner[i].refs[j];
            script_stage.model_owner[i].refs[j] = 0;
        }
        script_stage.model_owner[i].owner = 0;
    }
}

int ScriptGame_ModelSlot(int handle)
{
    int i;
    for (i = 0; i < SCRIPT_MESH_INSTANCES; ++i)
        if (script_stage.instance[i].handle == handle && handle > 0) return i;
    return -1;
}
int ScriptGame_ModelField(int slot, int field)
{
    ScriptMeshInstance* m;
    if (slot < 0 || slot >= SCRIPT_MESH_INSTANCES) return 0;
    m = &script_stage.instance[slot];
    if (field == -1) return m->handle;
    if (field == -2) return m->asset;
    if (field == -3) return m->count;
    if (field == -4) return m->dynamic;
    if (field == -5) return m->owner;
    return field >= 0 && field < SM_FIELDS ? m->field[field] : 0;
}

int ScriptGame_ModelLineOwner(int handle)
{
    int i, j;
    for (i = 0; i < SCRIPT_MESH_INSTANCES; ++i) {
        ScriptMeshInstance* m = &script_stage.instance[i];
        if (!m->handle) continue;
        for (j = 0; j < m->count; ++j)
            if (m->line[j].handle == handle) return m->handle;
    }
    return 0;
}

/* Validate every transformed segment before changing any CollVtx. Kind is semantic:
 * turning a floor upside down does not silently turn it into a ceiling. */
static int script_mesh_points(ScriptMeshInstance* m, float points[SCRIPT_MESH_LINES][4])
{
    float rot = script_mesh_float(m->field[SM_ROT]) * 0.017453292519943295f;
    float scale = script_mesh_float(m->field[SM_SCALE]);
    float sx = scale * script_mesh_float(m->field[SM_SX]);
    float sy = scale * script_mesh_float(m->field[SM_SY]);
    float c = cosf(rot), s = sinf(rot);
    float x = script_mesh_float(m->field[SM_X]), y = script_mesh_float(m->field[SM_Y]);
    int i, j;
    for (i = 0; i < m->count; ++i) {
        float* p = points[i];
        int kind = m->line[i].kind;
        p[0] = x + c * sx * m->line[i].x0 - s * sy * m->line[i].y0;
        p[1] = y + s * sx * m->line[i].x0 + c * sy * m->line[i].y0;
        p[2] = x + c * sx * m->line[i].x1 - s * sy * m->line[i].y1;
        p[3] = y + s * sx * m->line[i].x1 + c * sy * m->line[i].y1;
        /* mpLib's directed segments keep the solid side on the same side of
         * the edge. A reflection reverses orientation (rotation does not). */
        if (sx * sy < 0) {
            float t = p[0]; p[0] = p[2]; p[2] = t;
            t = p[1]; p[1] = p[3]; p[3] = t;
        }
        for (j = 0; j < 4; ++j) if (!(p[j] >= -100000 && p[j] <= 100000)) return 0;
        if ((kind == 1 && p[0] >= p[2]) || (kind == 2 && p[0] <= p[2]) ||
            (kind == 3 && p[1] <= p[3]) || (kind == 4 && p[1] >= p[3])) return 0;
    }
    return 1;
}

int ScriptGame_ModelSpawn(int asset, int token, int handle, int count, int first_line)
{
    ScriptMeshInstance candidate;
    float points[SCRIPT_MESH_LINES][4];
    int i, slot, free_lines = 0, owner = Script_StageResourceOwner();
    if (!ScriptGame_ModelRef(asset, token, 0) || count < 0 || count > SCRIPT_MESH_LINES ||
        script_stage.draw == NULL) return -1;
    if (owner > 0 && !ScriptGame_ModelRefOwned(asset, token, owner, 0)) return -1;
    for (slot = 0; slot < SCRIPT_MESH_INSTANCES && script_stage.instance[slot].handle; ++slot) {}
    if (slot == SCRIPT_MESH_INSTANCES) return -2;
    for (i = 0; i < script_stage.cap; ++i) if (!script_stage.line[i].active) ++free_lines;
    if (count > free_lines || (count && script_stage.map == NULL)) return -3;
    memset(&candidate, 0, sizeof candidate);
    candidate.handle = handle; candidate.asset = asset; candidate.count = count;
    candidate.owner = owner;
    for (i = 0; i < SM_FIELDS; ++i) candidate.field[i] = Script_ModelInput(i, -1);
    for (i = 0; i < 3; ++i) candidate.center[i] = script_mesh_float(Script_ModelInput(SM_FIELDS + i, -1));
    for (i = 0; i < count; ++i) {
        candidate.line[i].x0 = script_mesh_float(Script_ModelInput(0, i));
        candidate.line[i].y0 = script_mesh_float(Script_ModelInput(1, i));
        candidate.line[i].x1 = script_mesh_float(Script_ModelInput(2, i));
        candidate.line[i].y1 = script_mesh_float(Script_ModelInput(3, i));
        candidate.line[i].kind = Script_ModelInput(4, i);
        candidate.line[i].flags = Script_ModelInput(5, i);
        {
            int kind = candidate.line[i].kind;
            float scale = script_mesh_float(candidate.field[SM_SCALE]);
            if (kind <= 2 && scale * script_mesh_float(candidate.field[SM_SY]) < 0)
                kind = 3 - kind;
            if (kind >= 3 && scale * script_mesh_float(candidate.field[SM_SX]) < 0)
                kind = 7 - kind;
            candidate.line[i].kind = kind;
            if (kind != 1) candidate.line[i].flags = 0;
        }
        candidate.line[i].handle = first_line + i;
    }
    if (!script_mesh_points(&candidate, points)) return -4;
    for (i = 0; i < count; ++i) {
        int k;
        float* p = points[i];
        if (ScriptGame_StageAddLine(script_mesh_bits(p[0]), script_mesh_bits(p[1]),
              script_mesh_bits(p[2]), script_mesh_bits(p[3]), candidate.line[i].kind,
              candidate.line[i].flags, first_line + i) < 0) {
            while (i > 0) ScriptGame_StageRemove(first_line + --i);
            return -3;
        }
        for (k = 0; k < script_stage.cap; ++k)
            if (script_stage.line[k].active && script_stage.line[k].handle == first_line + i)
                script_stage.line[k].model_handle = handle; /* hide the debug slab */
    }
    script_stage.instance[slot] = candidate;
    ++script_stage.asset[asset].instances;
    OSReport("script model: spawn instance=%d asset=%d lines=%d\n", handle, asset, count);
    return handle;
}

int ScriptGame_ModelSet(int handle)
{
    int slot = ScriptGame_ModelSlot(handle), i;
    ScriptMeshInstance candidate;
    float points[SCRIPT_MESH_LINES][4];
    if (slot < 0) return 0;
    candidate = script_stage.instance[slot];
    for (i = 0; i < SM_FIELDS; ++i) candidate.field[i] = Script_ModelInput(i, -1);
    /* Reclassifying an owned line would also change its mpLib joint list.
     * Keep updates transactional: signs may change only on visual instances. */
    if (candidate.count) {
        for (i = SM_SX; i <= SM_SY; ++i) {
            ScriptMeshInstance* old = &script_stage.instance[slot];
            if ((script_mesh_float(candidate.field[SM_SCALE]) * script_mesh_float(candidate.field[i]) < 0) !=
                (script_mesh_float(old->field[SM_SCALE]) * script_mesh_float(old->field[i]) < 0)) return -4;
        }
    }
    if (!script_mesh_points(&candidate, points)) return -4;
    /* Once moved, retain a matrix draw for Aurora's uncapped frame replay.
     * This policy belongs to the instance snapshot, not a native history cache. */
    for (i = 0; i < SM_FIELDS; ++i)
        if (SM_FLOAT_FIELD(i) && candidate.field[i] != script_stage.instance[slot].field[i])
            candidate.dynamic = 1;
    if (candidate.field[SM_X] != script_stage.instance[slot].field[SM_X] ||
        candidate.field[SM_Y] != script_stage.instance[slot].field[SM_Y] ||
        candidate.field[SM_ROT] != script_stage.instance[slot].field[SM_ROT] ||
        candidate.field[SM_SCALE] != script_stage.instance[slot].field[SM_SCALE] ||
        candidate.field[SM_SX] != script_stage.instance[slot].field[SM_SX] ||
        candidate.field[SM_SY] != script_stage.instance[slot].field[SM_SY])
        for (i = 0; i < candidate.count; ++i)
            script_stage_line_set(candidate.line[i].handle, points[i][0], points[i][1], points[i][2], points[i][3]);
    script_stage.instance[slot] = candidate;
    return 1; /* movement is per-frame; only lifetime operations log */
}

int ScriptGame_ModelDespawn(int handle)
{
    int slot = ScriptGame_ModelSlot(handle), i;
    ScriptMeshInstance* m;
    if (slot < 0) return 0;
    m = &script_stage.instance[slot];
    /* Keep the instance and its reference if even one live collision segment
     * cannot be retired. A later retry may finish partially removed segments. */
    for (i = 0; i < m->count; ++i) {
        int j;
        for (j = 0; j < script_stage.cap; ++j)
            if (script_stage.line[j].active && script_stage.line[j].handle == m->line[i].handle) break;
        if (j == script_stage.cap) continue;
        if (!ScriptGame_StageRemove(m->line[i].handle)) return 0;
    }
    --script_stage.asset[m->asset].instances;
    memset(m, 0, sizeof *m);
    OSReport("script model: despawn instance=%d\n", handle);
    return 1;
}
