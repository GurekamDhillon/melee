/* Included in gw_script.c: shared asset loader for stage model= and gd.model_*.
 * Slots are scene-pinned even at zero references: old snapshots may name them. */
static int gs_model_asset_serial;
static int gs_model_texture(const char *dir, const char *name, GXTexObj *obj,
                            unsigned char **image)
{
    int h = gw_GxTex_OpenAt(dir, name), w, height, size;
    if (h < 0) return 0;
    w = gw_GxTex_Width(h); height = gw_GxTex_Height(h); size = gw_GxTex_ImageSize(h);
    if (gw_GxTex_Format(h) != GX_TF_RGBA8 || w < 4 || height < 4 || w > 4096 ||
        height > 4096 || (w & 3) || (height & 3) ||
        gs_stage_tex_levels(w, height, size) < 1) { gw_GxTex_Close(h); return 0; }
    *image = malloc((size_t)size);
    if (!*image) { gw_GxTex_Close(h); return 0; }
    gw_GxTex_CopyImage(h, *image); gw_GxTex_Close(h);
    gs_stage_tex_init(obj, *image, w, height, size);
    return 1;
}

static int gs_model_open_at(lua_State *L, const char *dir, const char *name)
{
    char path[MAX_PATH], sidepath[MAX_PATH], atlaspath[MAX_PATH], glow[64], canonical[MAX_PATH];
    GmCollision collision;
    GsStageModel art;
    unsigned char *mesh;
    char *json;
    size_t len, json_len;
    int i, shared = -1;
    DWORD full = GetFullPathNameA(dir, sizeof canonical, canonical, NULL);
    if (!full || full >= sizeof canonical) return luaL_error(L, "model path too long");
    dir = canonical;
    if (snprintf(path, sizeof path, "%s\\%s.gxmesh", dir, name) >= sizeof path ||
        snprintf(sidepath, sizeof sidepath, "%s\\%s.coll.json", dir, name) >= sizeof sidepath)
        return luaL_error(L, "model path too long");
    for (i = 0; i < gs_stage_nmodels; ++i)
        if (!_stricmp(path, gs_stage_models[i].path)) return i + 1;
    if (gs_stage_nmodels == GS_STAGE_MODELS)
        return luaL_error(L, "model cache full (%d scene-pinned assets); start a new scene", GS_STAGE_MODELS);
    memset(&collision, 0, sizeof collision);
    json = gs_read_file(sidepath, &json_len);
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
    for (i = 0; i < gs_stage_nmodels; ++i)
        if (!_stricmp(atlaspath, gs_stage_models[i].atlas_path)) { shared = i; break; }
    mesh = (unsigned char *)gs_read_file(path, &len);
    if (!mesh || !gs_stage_mesh_valid(mesh, len)) {
        free(mesh); return luaL_error(L, "missing or invalid GXMS model: %s", path);
    }
    if (gs_model_asset_serial == INT_MAX) { free(mesh); return luaL_error(L, "model handle space exhausted"); }
    memset(&art, 0, sizeof art);
    art.mesh = mesh; art.bytes = (int)len; art.has_normals = gw_r32(mesh + 4) == 2;
    art.stride = art.has_normals ? 32 : 20; art.collision = collision;
    art.atlas_owner = shared < 0;
    if (shared >= 0) {
        GsStageModel *owner = &gs_stage_models[shared];
        art.image = owner->image; art.glow_image = owner->glow_image;
        art.texture = owner->texture; art.glow = owner->glow; art.has_glow = owner->has_glow;
    } else {
        if (!gs_model_texture(dir, collision.atlas, &art.texture, &art.image)) {
            free(mesh); return luaL_error(L, "model needs a valid RGBA8 GXTX atlas: %s", atlaspath);
        }
        snprintf(glow, sizeof glow, "%s.glow", collision.atlas);
        if (snprintf(sidepath, sizeof sidepath, "%s\\%s.gxtex", dir, glow) >= sizeof sidepath) {
            free(mesh); free(art.image); return luaL_error(L, "model glow path too long");
        }
        if (GetFileAttributesA(sidepath) != INVALID_FILE_ATTRIBUTES) {
            if (!gs_model_texture(dir, glow, &art.glow, &art.glow_image)) {
                free(mesh); free(art.image); return luaL_error(L, "invalid model glow atlas: %s", sidepath);
            }
            art.has_glow = 1;
        }
    }
    snprintf(art.path, sizeof art.path, "%s", path);
    snprintf(art.atlas_path, sizeof art.atlas_path, "%s", atlaspath);
    art.token = ++gs_model_asset_serial;
    i = gs_stage_nmodels++;
    gs_stage_models[i] = art;
    gw_log("script stage: model %s loaded (GXMS v%u, %u triangles, %s%s; shared atlas=%d, collision=%d)",
           path, gw_r32(mesh + 4), gw_r32(mesh + 12) / 3,
           art.has_normals ? "lit" : "unlit", art.has_glow ? " + glow" : "", shared >= 0, collision.count);
    return i + 1;
}

static int gs_stage_model_open(lua_State *L, const char *name)
{
    GsScript *s = gs_cur_script();
    char dir[MAX_PATH], modeldir[MAX_PATH], path[MAX_PATH];
    int up;
    if (!s || !gm_model_path(name) || strchr(name, '/') || strlen(name) > 48)
        return luaL_error(L, "model must be a basename of at most 48 letters, digits, _ or -");
    snprintf(dir, sizeof dir, "%s", s->entry);
    for (up = 0; up < 2; ++up) {
        char *slash = strrchr(dir, '\\'), *forward = strrchr(dir, '/');
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
