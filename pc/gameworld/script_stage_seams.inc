/* Explicit scripted floor seams. MapLine id0 is mpLib's unguarded traversal /
 * island fallback; id1 is the enabled/proximity-checked cross-joint link. Keep
 * both directions reciprocal, and never infer links from other room geometry. */
#define SCRIPT_STAGE_SEAM_EPS 0.05f
static int script_stage_seam_current(void)
{
    return script_stage.map && mpLib_8004D164() == script_stage.map &&
        script_stage.cap > 0 && script_stage.cap <= SCRIPT_STAGE_LINES &&
        script_stage.base_l >= 0 && script_stage.base_j >= 0 &&
        script_stage.map->lines && script_stage.map->joints &&
        script_stage.map->line_count >= script_stage.base_l + script_stage.cap &&
        script_stage.map->joint_count >= script_stage.base_j + script_stage.cap;
}

static int script_stage_seam_slot(int handle)
{
    int i;
    for (i = 0; i < script_stage.cap; ++i)
        if (handle > 0 && script_stage.line[i].active && !script_area_retiring(script_stage.line[i].area) && script_stage.line[i].handle == handle) return i;
    return -1;
}

static int script_stage_seam_touch(int left, int right)
{
    MapLine* a = &script_stage.map->lines[script_stage.base_l + left];
    MapLine* b = &script_stage.map->lines[script_stage.base_l + right];
    CollVtx* vertices = mpGetGroundCollVtx();
    float dx, dy;
    if (!vertices || a->v0_idx >= script_stage.map->vert_count || a->v1_idx >= script_stage.map->vert_count ||
        b->v0_idx >= script_stage.map->vert_count || b->v1_idx >= script_stage.map->vert_count) return 0;
    dx = vertices[a->v1_idx].pos.x - vertices[b->v0_idx].pos.x;
    dy = vertices[a->v1_idx].pos.y - vertices[b->v0_idx].pos.y;
    return vertices[a->v0_idx].pos.x < vertices[a->v1_idx].pos.x &&
        vertices[b->v0_idx].pos.x < vertices[b->v1_idx].pos.x &&
        dx * dx + dy * dy <= SCRIPT_STAGE_SEAM_EPS * SCRIPT_STAGE_SEAM_EPS;
}

static int script_stage_seam_endpoint_valid(int slot, int next)
{
    int index = script_stage.base_l + slot, other;
    MapLine* line = &script_stage.map->lines[index];
    int id0 = next ? line->next_id0 : line->prev_id0;
    int id1 = next ? line->next_id1 : line->prev_id1;
    MapLine* peer;
    if (id0 == -1 && id1 == -1) return 1;
    if (id0 != id1 || id0 < script_stage.base_l || id0 >= script_stage.base_l + script_stage.cap) return 0;
    other = id0 - script_stage.base_l;
    if (other == slot || !script_stage.line[other].active || script_stage.line[other].kind != 1 ||
        script_stage.line[slot].owner <= 0 || script_stage.line[other].owner != script_stage.line[slot].owner) return 0;
    peer = &script_stage.map->lines[id0];
    {
        CollLine* collision = mpGetGroundCollLine();
        if (collision[id0].x0 != peer || (collision[id0].flags & LINE_FLAG_KIND) != CollLine_Floor ||
            !(collision[id0].flags & LINE_FLAG_ENABLED) || collision[id0].flags & LINE_FLAG_HIDDEN) return 0;
    }
    if ((next ? peer->prev_id0 : peer->next_id0) != index ||
        (next ? peer->prev_id1 : peer->next_id1) != index) return 0;
    return next ? script_stage_seam_touch(slot, other) : script_stage_seam_touch(other, slot);
}

/* Refresh every active floor joint: its dynamic island can cache the endpoints
 * of a chain spanning many joints, including joints beyond the changed pair.
 * 8005667C delegates to mpIsland_8005B334 without changing vertex frame history. */
static void script_stage_seam_refresh_area(int area)
{
    int i;
    mpScriptBatchBegin();
    for (i = 0; i < script_stage.cap; ++i)
        if (script_stage.line[i].active && script_stage.line[i].kind == 1 &&
            (area < 0 || script_stage.line[i].area == area))
            mpLib_8005667C(script_stage.base_j + i);
    mpScriptBatchEnd();
    mpScriptInvalidateBounding();
}
static void script_stage_seam_refresh(void) { script_stage_seam_refresh_area(-1); }

/* Clear the selected endpoint and every incoming reciprocal reference, including
 * inconsistent id0/id1 pairs. Slot reuse must never resurrect an old seam. */
static int script_stage_seam_unlink_endpoint(int slot, int next)
{
    int i, changed = 0, index = script_stage.base_l + slot;
    MapLine* line = &script_stage.map->lines[index];
    if (next) {
        changed = line->next_id0 != -1 || line->next_id1 != -1;
        line->next_id0 = line->next_id1 = -1;
    } else {
        changed = line->prev_id0 != -1 || line->prev_id1 != -1;
        line->prev_id0 = line->prev_id1 = -1;
    }
    for (i = 0; i < script_stage.cap; ++i) {
        MapLine* peer = &script_stage.map->lines[script_stage.base_l + i];
        if (next) {
            if (peer->prev_id0 == index) { peer->prev_id0 = -1; changed = 1; }
            if (peer->prev_id1 == index) { peer->prev_id1 = -1; changed = 1; }
        } else {
            if (peer->next_id0 == index) { peer->next_id0 = -1; changed = 1; }
            if (peer->next_id1 == index) { peer->next_id1 = -1; changed = 1; }
        }
    }
    return changed;
}

static int script_stage_seam_clear(int slot)
{
    int changed = script_stage_seam_unlink_endpoint(slot, 0);
    return script_stage_seam_unlink_endpoint(slot, 1) || changed;
}

static void script_stage_seam_moved(int slot)
{
    MapLine* line = &script_stage.map->lines[script_stage.base_l + slot];
    int i, index = script_stage.base_l + slot;
    int linked = line->prev_id0 != -1 || line->prev_id1 != -1 ||
                 line->next_id0 != -1 || line->next_id1 != -1;
    if (!script_stage_seam_endpoint_valid(slot, 0)) script_stage_seam_unlink_endpoint(slot, 0);
    if (!script_stage_seam_endpoint_valid(slot, 1)) script_stage_seam_unlink_endpoint(slot, 1);
    /* Also repair one-sided incoming references. No unguarded id0 may keep
     * following this line after it moves away, even if its own slots were clear. */
    for (i = 0; i < script_stage.cap; ++i) {
        MapLine* peer = &script_stage.map->lines[script_stage.base_l + i];
        if (peer->next_id0 == index || peer->next_id1 == index) {
            linked = 1;
            if (!script_stage.line[i].active || !script_stage_seam_endpoint_valid(i, 1))
                script_stage_seam_unlink_endpoint(i, 1);
        }
        if (peer->prev_id0 == index || peer->prev_id1 == index) {
            linked = 1;
            if (!script_stage.line[i].active || !script_stage_seam_endpoint_valid(i, 0))
                script_stage_seam_unlink_endpoint(i, 0);
        }
    }
    if (linked) script_stage_seam_refresh_area(script_stage.line[slot].area);
    else mpLib_8005667C(script_stage.base_j + slot);
}

/* 1 success; negative refusal codes are translated by gd.stage_link. */
int ScriptGame_StageLink(int handle_a, int handle_b, int owner)
{
    int a, b, left, right, forward, reverse, i, walk;
    MapLine* ml;
    MapLine* mr;
    CollLine* collision;
    if (!script_stage_seam_current()) return -1;
    a = script_stage_seam_slot(handle_a); b = script_stage_seam_slot(handle_b);
    if (a < 0 || b < 0 || a == b) return -2;
    if (owner <= 0 || script_stage.line[a].owner != owner || script_stage.line[b].owner != owner) return -3;
    if (script_stage.line[a].kind != 1 || script_stage.line[b].kind != 1) return -4;
    /* Independently gated rooms cannot share a cached traversable island. */
    if (script_stage.line[a].area != script_stage.line[b].area) return -8;
    collision = mpGetGroundCollLine();
    for (i = 0; i < 2; ++i) {
        int slot = i ? b : a, index = script_stage.base_l + slot;
        if (collision[index].x0 != &script_stage.map->lines[index] ||
            (collision[index].flags & LINE_FLAG_KIND) != CollLine_Floor ||
            !(collision[index].flags & LINE_FLAG_ENABLED) || collision[index].flags & LINE_FLAG_HIDDEN)
            return -8;
        if (!script_stage_seam_endpoint_valid(slot, 0) || !script_stage_seam_endpoint_valid(slot, 1)) return -8;
    }
    forward = script_stage_seam_touch(a, b); reverse = script_stage_seam_touch(b, a);
    if (forward && reverse) return -6;
    if (!forward && !reverse) return -5;
    left = forward ? a : b; right = forward ? b : a;
    ml = &script_stage.map->lines[script_stage.base_l + left];
    mr = &script_stage.map->lines[script_stage.base_l + right];
    if (ml->next_id0 != -1 || ml->next_id1 != -1 || mr->prev_id0 != -1 || mr->prev_id1 != -1) return -7;
    /* A small tolerance must not permit a cyclic chain of very short floors. */
    walk = right;
    for (i = 0; i < script_stage.cap; ++i) {
        int id;
        if (walk == left) return -6;
        if (!script_stage_seam_endpoint_valid(walk, 1)) return -8;
        id = script_stage.map->lines[script_stage.base_l + walk].next_id0;
        if (id == -1) break;
        walk = id - script_stage.base_l;
    }
    if (i == script_stage.cap) return -6;
    ml->next_id0 = ml->next_id1 = script_stage.base_l + right;
    mr->prev_id0 = mr->prev_id1 = script_stage.base_l + left;
    script_stage_seam_refresh_area(script_stage.line[a].area);
    return 1;
}
