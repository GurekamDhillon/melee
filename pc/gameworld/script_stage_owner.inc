/* Included after script_model.inc. All ownership is scalar game snapshot data;
 * immutable model/atlas bytes remain pinned in the native scene cache. */
int ScriptGame_StageResourceAccess(int handle, int owner)
{
    int i, found = -1;
    for (i = 0; i < SCRIPT_MESH_INSTANCES; ++i)
        if (script_stage.instance[i].handle == handle && handle > 0)
            found = script_stage.instance[i].owner;
    for (i = 0; i < SCRIPT_STAGE_LINES; ++i)
        if (script_stage.line[i].active && script_stage.line[i].handle == handle)
            found = script_stage.line[i].owner;
    for (i = 0; i < SCRIPT_STAGE_MODELS; ++i)
        if (script_stage.model[i].active && script_stage.model[i].handle == handle)
            found = script_stage.model[i].owner;
    for (i = 0; i < SCRIPT_STAGE_TARGETS; ++i)
        if (script_stage.target[i].active && script_stage.target[i].handle == handle)
            found = script_stage.target[i].owner;
    return found < 0 ? 0 : found == owner && owner > 0 ? 1 : -1;
}

/* Return the remaining live records, never a confirmation based on Lua tables.
 * A wrong/currently absent mpLib map makes StageRemove refuse, leaving records
 * available for another attempt or the real StageEnd lifecycle reset. */
int ScriptGame_StageReleaseOwner(int owner)
{
    int i, remaining = 0;
    if (owner <= 0) return 0;
    for (i = 0; i < SCRIPT_MESH_INSTANCES; ++i)
        if (script_stage.instance[i].handle && script_stage.instance[i].owner == owner)
            ScriptGame_ModelDespawn(script_stage.instance[i].handle);
    for (i = 0; i < SCRIPT_STAGE_MODELS; ++i)
        if (script_stage.model[i].active && script_stage.model[i].owner == owner)
            ScriptGame_StageRemove(script_stage.model[i].handle);
    for (i = 0; i < SCRIPT_STAGE_LINES; ++i)
        if (script_stage.line[i].active && script_stage.line[i].owner == owner &&
            !script_stage.line[i].model_handle)
            ScriptGame_StageRemove(script_stage.line[i].handle);
    for (i = 0; i < SCRIPT_STAGE_TARGETS; ++i)
        if (script_stage.target[i].active && script_stage.target[i].owner == owner)
            ScriptGame_StageRemove(script_stage.target[i].handle);
    script_model_release_owner_refs(owner);
    for (i = 0; i < SCRIPT_MESH_INSTANCES; ++i)
        if (script_stage.instance[i].handle && script_stage.instance[i].owner == owner) ++remaining;
    for (i = 0; i < SCRIPT_STAGE_LINES; ++i)
        if (script_stage.line[i].active && script_stage.line[i].owner == owner) ++remaining;
    for (i = 0; i < SCRIPT_STAGE_MODELS; ++i)
        if (script_stage.model[i].active && script_stage.model[i].owner == owner) ++remaining;
    for (i = 0; i < SCRIPT_STAGE_TARGETS; ++i)
        if (script_stage.target[i].active && script_stage.target[i].owner == owner) ++remaining;
    return remaining;
}

/* An old snapshot can resurrect a retired script generation. Resolve each dead
 * owner once, also serving as a bounded retry after unsafe-map cleanup refusal. */
int ScriptGame_StageReleaseDeadOwners(void)
{
    int owners[SCRIPT_MESH_INSTANCES + SCRIPT_STAGE_LINES + SCRIPT_STAGE_MODELS +
               SCRIPT_STAGE_TARGETS + SCRIPT_MESH_OWNERS];
    int i, j, n = 0, remaining = 0;
#define STAGE_DEAD_OWNER(id) do { \
    int candidate_owner = (id); \
    if (candidate_owner > 0 && !Script_StageResourceOwnerAlive(candidate_owner)) { \
        for (j = 0; j < n && owners[j] != candidate_owner; ++j) {} \
        if (j == n) owners[n++] = candidate_owner; \
    } \
} while (0)
    for (i = 0; i < SCRIPT_MESH_INSTANCES; ++i)
        if (script_stage.instance[i].handle) STAGE_DEAD_OWNER(script_stage.instance[i].owner);
    for (i = 0; i < SCRIPT_STAGE_LINES; ++i)
        if (script_stage.line[i].active) STAGE_DEAD_OWNER(script_stage.line[i].owner);
    for (i = 0; i < SCRIPT_STAGE_MODELS; ++i)
        if (script_stage.model[i].active) STAGE_DEAD_OWNER(script_stage.model[i].owner);
    for (i = 0; i < SCRIPT_STAGE_TARGETS; ++i)
        if (script_stage.target[i].active) STAGE_DEAD_OWNER(script_stage.target[i].owner);
    for (i = 0; i < SCRIPT_MESH_OWNERS; ++i) STAGE_DEAD_OWNER(script_stage.model_owner[i].owner);
#undef STAGE_DEAD_OWNER
    for (i = 0; i < n; ++i) remaining += ScriptGame_StageReleaseOwner(owners[i]);
    return remaining;
}
