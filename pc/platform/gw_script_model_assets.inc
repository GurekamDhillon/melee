/* Included in gw_script.c: shared asset loader for stage model= and gd.model_*.
 * Slots are scene-pinned even at zero references: old snapshots may name them. */
static int gs_model_asset_serial;
/* largemap: immutable scene pins are required by old snapshots. Bound their
 * native memory separately from MEM1 instead of evicting snapshot dependencies. */
#define GS_MODEL_BYTE_BUDGET (64 * 1024 * 1024)
static int gs_model_cache_bytes(void)
{
    int i, bytes = 0;
    for (i = 0; i < gs_stage_nmodels; ++i) {
        const GsStageModel *m = &gs_stage_models[i];
        bytes += m->bytes;
        if (m->atlas_owner) bytes += m->image_bytes + m->glow_bytes;
    }
    return bytes;
}
static int gs_model_texture(const char *dir, const char *name, GXTexObj *obj,
                            unsigned char **image, int *bytes, int available)
{
    int h = gw_GxTex_OpenAt(dir, name), w, height, size;
    if (h < 0) return 0;
    w = gw_GxTex_Width(h); height = gw_GxTex_Height(h); size = gw_GxTex_ImageSize(h);
    if (gw_GxTex_Format(h) != GX_TF_RGBA8 || w < 4 || height < 4 || w > 4096 ||
        height > 4096 || (w & 3) || (height & 3) ||
        gs_stage_tex_levels(w, height, size) < 1 || size > available) { gw_GxTex_Close(h); return 0; }
    *bytes = size;
    *image = malloc((size_t)size);
    if (!*image) { gw_GxTex_Close(h); return 0; }
    gw_GxTex_CopyImage(h, *image); gw_GxTex_Close(h);
    gs_stage_tex_init(obj, *image, w, height, size);
    return 1;
}

static uint64_t gs_model_source_stamp(lua_State *L, const char *root, char path[MAX_PATH], int optional)
{
    uint64_t stamp;
#ifdef _WIN32
    char final[MAX_PATH];
    const char *error = NULL;
    HANDLE h;
    if (optional && GetFileAttributesA(path) == INVALID_FILE_ATTRIBUTES) {
        DWORD why = GetLastError();
        if (why == ERROR_FILE_NOT_FOUND || why == ERROR_PATH_NOT_FOUND) return 0;
    }
    if (root) {
        h = gs_mission_open_native(root, path, FILE_READ_ATTRIBUTES, final, &error);
        if (h == INVALID_HANDLE_VALUE) { luaL_error(L, "%s", error); return 0; }
        CloseHandle(h);
        snprintf(path, MAX_PATH, "%s", final);
    }
#else
    if (root) { luaL_error(L, "mission models require Windows"); return 0; }
#endif
    if (!gs_mission_file_stamp(path, &stamp) || (!stamp && !optional)) {
        luaL_error(L, "model source missing or unreadable: %s", path); return 0;
    }
    return stamp;
}

/* Mission bytes are read through the very handle whose final path was checked.
 * Returning NULL lets callers release allocations before raising a Lua error. */
static unsigned char *gs_model_read_source(const char *root, const char *path, size_t *size, unsigned limit)
{
#ifdef _WIN32
    if (root) {
        char final[MAX_PATH];
        const char *error = NULL;
        BY_HANDLE_FILE_INFORMATION info;
        DWORD got, extra;
        unsigned char *data, tail;
        HANDLE h = gs_mission_open_native(root, path, GENERIC_READ, final, &error);
        if (h == INVALID_HANDLE_VALUE) return NULL;
        if (!GetFileInformationByHandle(h, &info) || (info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) ||
            info.nFileSizeHigh || info.nFileSizeLow > limit) { CloseHandle(h); return NULL; }
        data = malloc((size_t)info.nFileSizeLow + 1);
        if (!data) { CloseHandle(h); return NULL; }
        if (!ReadFile(h, data, info.nFileSizeLow, &got, NULL) || got != info.nFileSizeLow ||
            !ReadFile(h, &tail, 1, &extra, NULL) || extra) {
            free(data); CloseHandle(h); return NULL;
        }
        CloseHandle(h); data[got] = 0; *size = got; return data;
    }
#else
    (void)root; (void)limit;
#endif
    return (unsigned char *)gs_read_file(path, size);
}

/* Mission textures apply GXTX v1 bounds checks to validated-handle bytes.
 * Legacy textures continue through the shared filename-based GXTX parser. */
static int gs_model_texture_path(const char *root, const char *path, GXTexObj *obj,
                                 unsigned char **image, int *bytes, int available)
{
    char dir[MAX_PATH], *name;
    size_t len;
    if (root) {
        unsigned char *blob;
        unsigned w, h, image_size, offset, tlut_bytes, tlut_offset;
        if (available <= 0) return 0;
        blob = gs_model_read_source(root, path, &len, GS_MODEL_BYTE_BUDGET + 64u);
        if (!blob) return 0;
        if (len < 64 || gw_r32(blob) != 0x47585458u || gw_r32(blob + 4) != 1 ||
            gw_r32(blob + 8) != GX_TF_RGBA8) { free(blob); return 0; }
        w = gw_r32(blob + 12); h = gw_r32(blob + 16);
        image_size = gw_r32(blob + 28); offset = gw_r32(blob + 36);
        tlut_bytes = gw_r32(blob + 32); tlut_offset = gw_r32(blob + 40);
        if (w < 4 || h < 4 || w > 4096 || h > 4096 || (w & 3) || (h & 3) ||
            image_size > (unsigned)available || (uint64_t)offset + image_size > len ||
            (tlut_bytes && (uint64_t)tlut_offset + tlut_bytes > len) ||
            gs_stage_tex_levels((int)w, (int)h, (int)image_size) < 1) { free(blob); return 0; }
        *image = malloc(image_size);
        if (!*image) { free(blob); return 0; }
        memcpy(*image, blob + offset, image_size); free(blob); *bytes = (int)image_size;
        gs_stage_tex_init(obj, *image, (int)w, (int)h, (int)image_size);
        return 1;
    }
    snprintf(dir, sizeof dir, "%s", path);
    name = gw_path_separator(dir);
    if (!name) return 0;
    *name++ = 0; len = strlen(name);
    if (len <= 6 || strcmp(name + len - 6, ".gxtex")) return 0;
    name[len - 6] = 0;
    return gs_model_texture(dir, name, obj, image, bytes, available);
}

static int gs_model_open_at_root(lua_State *L, const char *dir, const char *name, const char *root)
{
    char path[MAX_PATH], sidepath[MAX_PATH], atlaspath[MAX_PATH], glowpath[MAX_PATH], glow[64], canonical[MAX_PATH];
    GmCollision collision;
    GsStageModel art;
    unsigned char *mesh;
    char *json;
    size_t len, json_len;
    int i, shared = -1, available;
    uint64_t stamp[4];
    WIN32_FILE_ATTRIBUTE_DATA info;
    DWORD full = GetFullPathNameA(dir, sizeof canonical, canonical, NULL);
    if (!full || full >= sizeof canonical) return luaL_error(L, "model path too long");
    dir = canonical;
    if (snprintf(path, sizeof path, "%s\\%s.gxmesh", dir, name) >= sizeof path ||
        snprintf(sidepath, sizeof sidepath, "%s\\%s.coll.json", dir, name) >= sizeof sidepath)
        return luaL_error(L, "model path too long");
    if (!root && gs_stage_nmodels == GS_STAGE_MODELS) {
        for (i = 0; i < gs_stage_nmodels; ++i)
            if (!_stricmp(path, gs_stage_models[i].path)) break;
        if (i == gs_stage_nmodels)
            return luaL_error(L, "model cache full (%d scene-pinned assets); restart the match", GS_STAGE_MODELS);
    }
    stamp[0] = gs_model_source_stamp(L, root, path, 0);
    stamp[1] = gs_model_source_stamp(L, root, sidepath, 1);
    memset(&collision, 0, sizeof collision);
    json = (char *)gs_model_read_source(root, sidepath, &json_len, 16384);
    if (json) {
        int valid = json_len <= 16384 && strlen(json) == json_len && gm_collision_parse(json, &collision);
        free(json);
        if (!valid) return luaL_error(L, "invalid model collision sidecar: %s", sidepath);
    } else if (GetFileAttributesA(sidepath) != INVALID_FILE_ATTRIBUTES)
        return luaL_error(L, "cannot read collision sidecar: %s", sidepath);
    if (!collision.atlas[0]) {
        if (strlen(name) > 48) return luaL_error(L, "model basename exceeds 48 bytes");
        strcpy(collision.atlas, name);
    }
    if (snprintf(atlaspath, sizeof atlaspath, "%s\\%s.gxtex", dir, collision.atlas) >= sizeof atlaspath)
        return luaL_error(L, "model atlas path too long");
    snprintf(glow, sizeof glow, "%s.glow", collision.atlas);
    if (snprintf(glowpath, sizeof glowpath, "%s\\%s.gxtex", dir, glow) >= sizeof glowpath)
        return luaL_error(L, "model glow path too long");
    stamp[2] = gs_model_source_stamp(L, root, atlaspath, 0);
    stamp[3] = gs_model_source_stamp(L, root, glowpath, 1);
    for (i = 0; i < gs_stage_nmodels; ++i)
        if (!_stricmp(path, gs_stage_models[i].path) &&
            !memcmp(stamp, gs_stage_models[i].content_stamp, sizeof stamp)) return i + 1;
    if (gs_stage_nmodels == GS_STAGE_MODELS)
        return luaL_error(L, "model cache full (%d scene-pinned assets); restart the match", GS_STAGE_MODELS);
    for (i = 0; i < gs_stage_nmodels; ++i)
        if (!_stricmp(atlaspath, gs_stage_models[i].atlas_path) &&
            !memcmp(stamp + 2, gs_stage_models[i].atlas_stamp, sizeof gs_stage_models[i].atlas_stamp)) { shared = i; break; }
    available = GS_MODEL_BYTE_BUDGET - gs_model_cache_bytes();
    if (!GetFileAttributesExA(path, GetFileExInfoStandard, &info) ||
        info.nFileSizeHigh || info.nFileSizeLow > (DWORD)available)
        return luaL_error(L, "model file missing or scene asset budget exceeded (64 MiB)");
    mesh = gs_model_read_source(root, path, &len, 4u * 1024 * 1024);
    if (!mesh || len > (size_t)available || !gs_stage_mesh_valid(mesh, len)) {
        free(mesh); return luaL_error(L, "missing or invalid GXMS model: %s", path);
    }
    if (gs_model_asset_serial == INT_MAX) { free(mesh); return luaL_error(L, "model handle space exhausted"); }
    memset(&art, 0, sizeof art);
    art.mesh = mesh; art.bytes = (int)len; art.has_normals = gw_r32(mesh + 4) == 2;
    art.stride = art.has_normals ? 32 : 20; art.collision = collision;
    {
        unsigned v, n = gw_r32(mesh + 8), offset = gw_r32(mesh + 28);
        int axis;
        for (axis = 0; axis < 3; ++axis) {
            float lo = gw_rf32(mesh + offset + axis * 4), hi = lo;
            for (v = 1; v < n; ++v) {
                float f = gw_rf32(mesh + offset + v * art.stride + axis * 4);
                if (f < lo) lo = f;
                if (f > hi) hi = f;
            }
            art.center[axis] = lo * 0.5f + hi * 0.5f;
        }
    }
    art.atlas_owner = shared < 0;
    available -= (int)len;
    if (shared >= 0) {
        GsStageModel *owner = &gs_stage_models[shared];
        art.image = owner->image; art.glow_image = owner->glow_image;
        art.texture = owner->texture; art.glow = owner->glow; art.has_glow = owner->has_glow;
        art.tex_w = owner->tex_w; art.tex_h = owner->tex_h; art.glow_w = owner->glow_w; art.glow_h = owner->glow_h;
    } else {
        if (!gs_model_texture_path(root, atlaspath, &art.texture, &art.image, &art.image_bytes, available)) {
            free(mesh); return luaL_error(L, "model needs a valid RGBA8 GXTX atlas: %s", atlaspath);
        }
        available -= art.image_bytes;
        art.tex_w = gs_tex_last_w; art.tex_h = gs_tex_last_h;
        if (stamp[3]) {
            if (!gs_model_texture_path(root, glowpath, &art.glow, &art.glow_image, &art.glow_bytes, available)) {
                free(mesh); free(art.image); return luaL_error(L, "invalid model glow atlas: %s", glowpath);
            }
            art.has_glow = 1; art.glow_w = gs_tex_last_w; art.glow_h = gs_tex_last_h;
        }
    }
    snprintf(art.path, sizeof art.path, "%s", path);
    snprintf(art.atlas_path, sizeof art.atlas_path, "%s", atlaspath);
    memcpy(art.content_stamp, stamp, sizeof stamp);
    memcpy(art.atlas_stamp, stamp + 2, sizeof art.atlas_stamp);
    {
        const char *sources[4] = { path, sidepath, atlaspath, glowpath };
        uint64_t after;
        for (i = 0; i < 4; ++i) {
            if (!gs_mission_file_stamp(sources[i], &after) || after != stamp[i]) {
                free(art.mesh);
                if (art.atlas_owner) { free(art.image); free(art.glow_image); }
                return luaL_error(L, "model sources changed while loading; retry the export");
            }
        }
    }
    {
        char material_error[2048] = "";
        art.custom_material = gw_Shader_ModelLoad(root, path, atlaspath, art.has_glow ? glowpath : NULL,
                                                  material_error, sizeof material_error);
    }
    art.token = ++gs_model_asset_serial;
    i = gs_stage_nmodels++;
    gs_stage_models[i] = art;
    gw_log("script stage: model %s loaded (GXMS v%u, %u triangles, %s%s; shared atlas=%d, collision=%d, alpha=%d)",
           path, gw_r32(mesh + 4), gw_r32(mesh + 12) / 3,
           art.has_normals ? "lit" : "unlit", art.has_glow ? " + glow" : "", shared >= 0, collision.count, collision.alpha);
    return i + 1;
}

static int gs_model_open_at(lua_State *L, const char *dir, const char *name)
{
    return gs_model_open_at_root(L, dir, name, NULL);
}

static int gs_mission_model_open(lua_State *L, const char *path)
{
    char root[MAX_PATH], dir[MAX_PATH], rel[MAX_PATH], *last, *p;
    size_t len = strlen(path);
    if (!gs_mission_path_valid(path, len) || !gs_mission_script_root(root))
        return luaL_error(L, "mission model requires a contained missions/ path in the calling mod");
    snprintf(rel, sizeof rel, "%s", path);
    if (len > 7 && !strcmp(rel + len - 7, ".gxmesh")) rel[len - 7] = 0;
    last = strrchr(rel, '/');
    if (!last || !last[1]) return luaL_error(L, "mission model basename missing");
    *last++ = 0;
    if (!gm_model_path(last) || strchr(last, '/') || strlen(last) > 48)
        return luaL_error(L, "model basename must use at most 48 letters, digits, _ or -");
    if (snprintf(dir, sizeof dir, "%s\\%s", root, rel) >= sizeof dir) return luaL_error(L, "model path too long");
    for (p = dir; *p; ++p) if (*p == '/') *p = '\\';
    return gs_model_open_at_root(L, dir, last, root);
}

static int gs_stage_model_open(lua_State *L, const char *name)
{
    GsScript *s = gs_cur_script();
    char dir[MAX_PATH], modeldir[MAX_PATH], path[MAX_PATH];
    int up;
    if (!strncmp(name, "missions/", 9)) return gs_mission_model_open(L, name);
    if (!s || !gm_model_path(name) || strchr(name, '/') || strlen(name) > 48)
        return luaL_error(L, "model must be a basename of at most 48 letters, digits, _ or -");
    snprintf(dir, sizeof dir, "%s", s->entry);
    for (up = 0; up < 2; ++up) {
        char *slash = gw_path_separator(dir), *forward = strrchr(dir, '/');
        if (forward && (!slash || forward > slash)) slash = forward;
        if (!slash) break;
        *slash = 0;
        snprintf(modeldir, sizeof modeldir, "%s\\models", dir);
        snprintf(path, sizeof path, "%s\\%s.gxmesh", modeldir, name);
        if (GetFileAttributesA(path) != INVALID_FILE_ATTRIBUTES) return gs_model_open_at(L, modeldir, name);
    }
    return luaL_error(L, "model %s not found in this mod's models/ directory", name);
}

/* mod-id/path/to/model (extension omitted), rooted at a mounted payload directory.
 * A plain basename retains the stage model= lookup beside the current script. */
static int gs_model_open(lua_State *L, const char *path)
{
    char id[181], dir[MAX_PATH], rel[181];
    const char *slash = strchr(path, '/');
    char *last;
    int mod;
    if (!strncmp(path, "missions/", 9)) return gs_mission_model_open(L, path);
    if (!gm_model_path(path)) return luaL_error(L, "model path must contain only letters, digits, _, -, and single / separators");
    if (!slash) return gs_stage_model_open(L, path);
    memcpy(id, path, slash - path); id[slash - path] = 0;
    mod = gw_Mods_Find(id);
    if (mod < 0 || !gw_Mods_IsActive(mod)) return luaL_error(L, "model mod is not mounted: %s", id);
    snprintf(rel, sizeof rel, "%s", slash + 1);
    last = strrchr(rel, '/');
    if (last) { *last++ = 0;
        if (snprintf(dir, sizeof dir, "%s\\%s", gw_Mods_PayloadDir(mod), rel) >= sizeof dir)
            return luaL_error(L, "model path too long");
    } else { last = rel; snprintf(dir, sizeof dir, "%s", gw_Mods_PayloadDir(mod)); }
    if (strlen(last) > 48) return luaL_error(L, "model basename exceeds 48 bytes");
    /* Normalize separators so path caching doesn't depend on caller spelling. */
    { char *p; for (p = dir; *p; ++p) if (*p == '/') *p = '\\'; }
    return gs_model_open_at(L, dir, last);
}
