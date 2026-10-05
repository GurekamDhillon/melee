/* gd.kit.model(model, x, y, w, h [, opts]): a script model drawn into a rectangle of the 640x480 script
 * canvas, in call order with gd.fill / gd.box / gd.text / gd.kit.* (included in gw_script.c after the
 * world-model API; the maths is gw_screen_model.h).
 *
 * WHY IMMEDIATE MODE, WHY NOT GX. The interface pass is not a GX pass: every gd draw is a GwScriptDraw
 * that gw_console.cpp replays onto the ImGui foreground draw list (draw_script_list), and kit calls are
 * quads in one shared bank (gw_kit.c). A model drawn by GX would land under that whole list, so a cell's
 * pips, flags, cursor and text, and even its gd.fill background, could not be ordered against it. The
 * model is therefore transformed here on the CPU (fixed orthographic camera, fixed light) and appended to
 * the same kit bank as lit, textured triangles (gw_Kit_DrawTri), each carrying its clip rectangle; the
 * ImGui pass scissors them. Order, clipping and "always in front of the world" are then the draw list's
 * own, and nothing here touches GX, game memory or the simulation: it is local presentation, so it is
 * allowed offline and online, and a rewind or savestate has nothing of it to restore.
 *
 * COST. One call is a handle check, a handful of option reads, 3 transforms per triangle and a sort of
 * at most GSM_MAX_TRIS triangles; the mesh is decoded from its big-endian bytes once per asset and the
 * atlas once into a kit texture. */
typedef struct { int token; GsmMesh mesh; int tex_token, tex; } GsSmCache;
static GsSmCache gs_sm_cache[GS_STAGE_MODELS];
static double gs_ui_clock;   /* 60 Hz clock units: advanced once per script tick by elapsed wall time */
static double gs_ui_clock_ms;
static int gs_sm_frame_calls, gs_sm_frame_tris;

/* The spin's time base. A script tick is a render tick (60 or 120 a second), so counting ticks would
 * spin twice as fast at 120 Hz: the clock counts 1/60 s units of wall time instead, fraction kept so
 * 120 Hz motion is smooth, a hitch clamped so a stalled frame does not jump the spin. */
static void gs_ui_clock_step(void)
{
    double now = gs_now_ms(), d = now - gs_ui_clock_ms;
    if (gs_ui_clock_ms == 0 || d < 0) d = 0;
    if (d > 100.0) d = 100.0;
    gs_ui_clock_ms = now;
    gs_ui_clock += d * 0.06;
    gs_sm_frame_calls = gs_sm_frame_tris = 0;
}

static unsigned gs_sm_idx(const unsigned char *idx, unsigned i) { return gw_r16(idx + i * 2); }

/* Decode an asset's GXMS bytes into native floats, once; refuse a mesh too big for an icon. */
static GsSmCache *gs_sm_mesh(int asset, const char **why)
{
    GsSmCache *c = &gs_sm_cache[asset];
    const GsStageModel *art = &gs_stage_models[asset];
    const unsigned char *mesh = art->mesh, *v;
    unsigned nv, ni, voff, ioff, i;
    GsmMesh *m = &c->mesh;
    if (c->token == art->token && m->ntri) return c;
    gsm_free(m); c->token = 0; c->tex_token = 0; c->tex = -1;
    if (!mesh) { *why = "model has no mesh"; return NULL; }
    nv = gw_r32(mesh + 8); ni = gw_r32(mesh + 12); voff = gw_r32(mesh + 28); ioff = gw_r32(mesh + 32);
    if (ni / 3 > GSM_MAX_TRIS) { *why = "model has too many triangles for a screen draw (512 at most)"; return NULL; }
    m->nvert = (int)nv; m->ntri = (int)(ni / 3); m->has_normals = art->has_normals;
    m->pos = malloc(nv * sizeof *m->pos); m->uv = malloc(nv * sizeof *m->uv);
    m->nrm = art->has_normals ? malloc(nv * sizeof *m->nrm) : NULL;
    m->idx = malloc(ni * sizeof *m->idx);
    if (!m->pos || !m->uv || !m->idx || (art->has_normals && !m->nrm)) {
        gsm_free(m); *why = "out of memory"; return NULL;
    }
    for (i = 0; i < nv; ++i) {
        v = mesh + voff + i * art->stride;
        m->pos[i][0] = gw_rf32(v); m->pos[i][1] = gw_rf32(v + 4); m->pos[i][2] = gw_rf32(v + 8);
        m->uv[i][0] = gw_rf32(v + 12); m->uv[i][1] = gw_rf32(v + 16);
        if (m->nrm) { m->nrm[i][0] = gw_rf32(v + 20); m->nrm[i][1] = gw_rf32(v + 24); m->nrm[i][2] = gw_rf32(v + 28); }
    }
    for (i = 0; i < ni; ++i) m->idx[i] = (uint16_t)gs_sm_idx(mesh + ioff, i);
    if (!gsm_bounds(m)) { gsm_free(m); *why = "model has no extent"; return NULL; }
    c->token = art->token;
    return c;
}

/* The asset's atlas as a kit texture (-1: none, the triangles then draw flat). */
static int gs_sm_texture(int asset, GsSmCache *c)
{
    const GsStageModel *art = &gs_stage_models[asset];
    char key[MAX_PATH + 40];
    if (c->tex_token == art->token) return c->tex;
    c->tex_token = art->token; c->tex = -1;
    if (!art->image || art->tex_w < 4) return -1;
    snprintf(key, sizeof key, "%s|%llx|%llx", art->path, (unsigned long long)art->content_stamp[2],
             (unsigned long long)art->content_stamp[3]);
    c->tex = gw_Kit_TexAddGX(key, art->image, (size_t)art->tex_w * art->tex_h * 4,
                             art->tex_w, art->tex_h, art->has_glow ? art->glow_image : NULL,
                             (size_t)art->glow_w * art->glow_h * 4, art->glow_w, art->glow_h, art->collision.alpha);
    return c->tex;
}

/* Draw asset (an index into gs_stage_models) into the rectangle p. Returns the triangles added; -1 and
 * *why when the asset cannot be drawn this way. */
static int gs_sm_emit(int asset, const GsmParams *p, int clip, const char **why)
{
    static GsmTri tris[GSM_MAX_TRIS];
    GsSmCache *c = gs_sm_mesh(asset, why);
    float rect[4];
    int n, k, tex, added = 0;
    if (!c) return -1;
    n = gsm_build(&c->mesh, p, tris, GSM_MAX_TRIS);
    if (n > gw_Kit_QuadRoom()) { *why = "the kit's quad list is full this frame"; return -1; }
    tex = gs_sm_texture(asset, c);
    rect[0] = p->x; rect[1] = p->y; rect[2] = p->x + p->w; rect[3] = p->y + p->h;
    for (k = 0; k < n; ++k)
        added += gw_Kit_DrawTri(tex, tris[k].x, tris[k].y, tris[k].u, tris[k].v, tris[k].c, clip ? rect : NULL);
    return added;
}

static float gs_sm_num(lua_State *L, int t, const char *k, double def, double lo, double hi)
{
    double v = def;
    if (lua_istable(L, t)) {
        lua_getfield(L, t, k);
        if (!lua_isnil(L, -1)) {
            if (!lua_isnumber(L, -1)) luaL_error(L, "gd.kit.model: %s must be a number", k);
            v = lua_tonumber(L, -1);
            if (!(v == v) || v < lo || v > hi) {
                char range[64]; /* lua_pushfstring has no %g */
                snprintf(range, sizeof range, "%g..%g", lo, hi);
                luaL_error(L, "gd.kit.model: %s must be %s", k, range);
            }
        }
        lua_pop(L, 1);
    }
    return (float)v;
}

/* gd.kit.model(model, x, y, w, h [, opts]) -> triangles drawn
 *   model  a gd.model_load handle
 *   x, y, w, h  the rectangle, script canvas units (the same space as gd.fill / gd.kit.image)
 *   opts   yaw, pitch (degrees, -360..360), spin (degrees per second, +-3600), t (the spin's clock in
 *          1/60 s units; default: the engine's UI clock), margin (0..0.45 of w and h, default 0.08),
 *          tint (colour, default white), dim (0..1), alpha (0..1), cull (bool), clip (bool, default true) */
static int l_kit_model(lua_State *L)
{
    GsmParams p;
    const char *why = "";
    int asset, n, clip;
    double x = luaL_checknumber(L, 2), y = luaL_checknumber(L, 3), w = luaL_checknumber(L, 4), h = luaL_checknumber(L, 5);
    lua_settop(L, 6);
    if (!lua_isnil(L, 6)) luaL_checktype(L, 6, LUA_TTABLE);
    if (!(x == x) || !(y == y) || fabs(x) > 100000 || fabs(y) > 100000)
        return luaL_error(L, "gd.kit.model: x and y must be finite and within +/-100000");
    if (!(w == w) || !(h == h) || w <= 0 || h <= 0 || w > GSM_MAX_SIDE || h > GSM_MAX_SIDE)
        return luaL_error(L, "gd.kit.model: w and h must be above 0 and at most 4096");
    p.x = (float)x; p.y = (float)y; p.w = (float)w; p.h = (float)h;
    p.yaw = gs_sm_num(L, 6, "yaw", 0, -360, 360);
    p.pitch = gs_sm_num(L, 6, "pitch", 0, -360, 360);
    p.spin = gs_sm_num(L, 6, "spin", 0, -3600, 3600);
    p.clock = gs_sm_num(L, 6, "t", gs_ui_clock, -1e7, 1e7);
    p.margin = gs_sm_num(L, 6, "margin", 0.08, 0, 0.45);
    p.dim = gs_sm_num(L, 6, "dim", 1, 0, 1);
    p.alpha = gs_sm_num(L, 6, "alpha", 1, 0, 1);
    p.tint = gs_kit_optcolour(L, 6, "tint", 0xFFFFFFFFu);
    p.cull = gs_kit_optbool(L, 6, "cull", 0);
    clip = gs_kit_optbool(L, 6, "clip", 1);
    asset = gs_model_asset(L, 1); /* after the options: a bad call names its first fault, a stale handle last */
    if (p.alpha <= 0.0f) { lua_pushinteger(L, 0); return 1; }
    {
        int first = gw_Kit_QuadCount();
        n = gs_sm_emit(asset, &p, clip, &why);
        if (n < 0) return luaL_error(L, "gd.kit.model: %s", why);
        gs_kit_record(first, n);
    }
    ++gs_sm_frame_calls; gs_sm_frame_tris += n;
    lua_pushinteger(L, n);
    return 1;
}
