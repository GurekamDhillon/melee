/* Read-only stage geometry query. Use loaded stage_info and mpLib world vertices,
 * never stage-ID constants or the mutable scripted collision extension. */
static int script_stage_floor_bounds(float out[5])
{
    MapCollData* map = mpLib_8004D164();
    CollLine* lines = mpGetGroundCollLine();
    CollVtx* verts = mpGetGroundCollVtx();
    unsigned char visited[1536]; /* mpLibLoad's line capacity */
    int queue[1536], count, i, found = 0;
    float widest = -1;
    if (!map || !lines || !verts) return 0;
    count = script_stage.map ? script_stage.base_l : map->line_count;
    if (count < 0 || count > 1536) return 0;
    memset(visited, 0, sizeof visited);
    out[4] = -100000;
    for (i = 0; i < count; ++i) {
        MapLine* l = lines[i].x0;
        int head = 0, tail = 0;
        float left = 100000, right = -100000, bottom = 100000, top = -100000;
        if (!l || !(lines[i].flags & CollLine_Floor) ||
            !(lines[i].flags & LINE_FLAG_ENABLED) ||
            (lines[i].flags & (LINE_FLAG_HIDDEN | LINE_FLAG_EMPTY))) continue;
        if (verts[l->v0_idx].pos.y > out[4]) out[4] = verts[l->v0_idx].pos.y;
        if (verts[l->v1_idx].pos.y > out[4]) out[4] = verts[l->v1_idx].pos.y;
        if (visited[i] || ((l->lo_flags | lines[i].flags) & LINE_FLAG_PLATFORM)) continue;
        visited[i] = 1;
        queue[tail++] = i;
        /* Connected solid floor chains may contain slopes; disconnected islands
         * are not silently combined into a floor across empty space. */
        while (head < tail) {
            int k, neighbors[2];
            l = lines[queue[head++]].x0;
            for (k = 0; k < 2; ++k) {
                Vec2 p = verts[k ? l->v1_idx : l->v0_idx].pos;
                if (p.x < left) left = p.x;
                if (p.x > right) right = p.x;
                if (p.y < bottom) bottom = p.y;
                if (p.y > top) top = p.y;
            }
            neighbors[0] = l->prev_id0; neighbors[1] = l->next_id0;
            for (k = 0; k < 2; ++k) {
                int n = neighbors[k];
                if (n < 0 || n >= count || visited[n] || !lines[n].x0 ||
                    !(lines[n].flags & CollLine_Floor) || !(lines[n].flags & LINE_FLAG_ENABLED) ||
                    (lines[n].flags & (LINE_FLAG_HIDDEN | LINE_FLAG_EMPTY)) ||
                    ((lines[n].x0->lo_flags | lines[n].flags) & LINE_FLAG_PLATFORM)) continue;
                visited[n] = 1;
                queue[tail++] = n;
            }
        }
        if (right - left > widest) {
            widest = right - left; found = 1;
            out[0] = left; out[1] = right; out[2] = top; out[3] = bottom;
        }
    }
    return found;
}

int ScriptGame_StageBounds(int field)
{
    float floor[5];
    StageBlastZone* box;
    if (field >= 8) {
        int found = script_stage_floor_bounds(floor);
        if (field == 12) return found;
        if (!found) return 0;
        if (field == 13) return script_mesh_bits(floor[4]);
        return field <= 11 ? script_mesh_bits(floor[field - 8]) : 0;
    }
    box = field < 4 ? &stage_info.blast_zone : &stage_info.cam_info.cam_bounds;
    switch (field & 3) {
    case 0: return script_mesh_bits(box->left);
    case 1: return script_mesh_bits(box->right);
    case 2: return script_mesh_bits(box->top);
    default: return script_mesh_bits(box->bottom);
    }
}
