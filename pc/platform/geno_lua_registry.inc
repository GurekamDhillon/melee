/* geno_lua_registry.inc - slice 5: the "lua" block of a define and the "lua" key of its states, and the scalar functions the game
 * half calls (docs/geno.md section 23). Included by geno_registry.c after gn_profile; the domain itself is geno_lua_core.h.
 *
 *   "lua": { "script": "lua/charger.lua" | "source": "return {...}",      one module (a file of the mod, or inline text for tests)
 *            "state": { "charge": "int", "ratio": "float", "armed": "bool" } }   the typed per-fighter state, at most 16 slots
 *   states[n]: { ..., "lua": { "enter": "fn", "frame": "fn" } }              functions of that module, run on entry / each frame
 *
 * Strict, like the rest of a define: a bad block, a missing function, a module the domain refuses (geno_lua_core.h), or the key on an
 * attach entry or a file older than "geno": 10 refuses the entry with the reason in the log. The module text and the layout are folded
 * into the entry's id, so two installs with different scripts never share one. Nothing here changes an entry without a lua block. */

static const gn_profile *gn_at(int p);
static char *gn_read_file(const char *path);

#define GN_LUA_MAX_SOURCE (64 * 1024)

static void gn_lua_free_all(gn_registry *r)
{
    int i;
    for (i = 0; i < r->n; ++i) {
        glua_free(r->p[i].lua);
        r->p[i].lua = NULL;
    }
}

static uint64_t gn_lua_mix_text(uint64_t h, const char *s, size_t n)
{
    size_t i;
    for (i = 0; i < n; ++i) h = (h ^ (unsigned char) s[i]) * 0x100000001B3ull;
    return h;
}

static int gn_lua_name_ok(const char *s, size_t cap)
{
    size_t i, n = strlen(s);
    if (n == 0 || n >= cap) return 0;
    for (i = 0; i < n; ++i) {
        char c = s[i];
        if (!((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_' || (i && c >= '0' && c <= '9'))) return 0;
    }
    return 1;
}

/* 1 when the entry is fine (with or without Lua); 0 refuses it. */
static int gn_lua_parse(gn_profile *p, const jdoc *d, int e, const char *mod, const char *where, int version)
{
    int lb = jd_get(d, e, "lua"), st = jd_get(d, e, "states"), c, k, any = 0;
    char names[GENO_LUA_STATE_SLOTS][GLUA_SLOT_NAME];
    int types[GENO_LUA_STATE_SLOTS];
    char tnames[GENO_MAX_STATES][GLUA_NAME];
    char *file_text = NULL;
    const char *src = NULL;
    size_t srclen = 0;
    int nslot = 0, ok = 0;
    gn_lua *g;
    if (st >= 0 && d->n[st].type == JN_ARR)
        for (c = d->n[st].first; c >= 0; c = d->n[c].next)
            if (jd_get(d, c, "lua") >= 0) any = 1;
    if (lb < 0 && !any) return 1;
    if (!p->defined) { gw_log("geno: %s: fighter Lua is for a define; an attach entry cannot carry a \"lua\" block - refused", where); return 0; }
    if (version < 10) { gw_log("geno: %s: \"lua\" needs \"geno\": 10 - refused", where); return 0; }
    if (lb < 0 || d->n[lb].type != JN_OBJ) { gw_log("geno: %s: a state names Lua functions but the entry has no \"lua\" block - refused", where); return 0; }
    {
        int sc = jd_get(d, lb, "script"), so = jd_get(d, lb, "source"), sl = jd_get(d, lb, "state");
        if ((sc < 0) == (so < 0) || (sc >= 0 && d->n[sc].type != JN_STR) || (so >= 0 && d->n[so].type != JN_STR)) {
            gw_log("geno: %s: lua needs exactly one of \"script\" (a file of the mod) and \"source\" (inline text) - refused", where);
            return 0;
        }
        if (sc >= 0) {
            const char *rel = d->n[sc].str;
            char path[MAX_PATH];
            const char *dir = gw_Mods_Dir();
            if (!rel[0] || strstr(rel, "..") || strchr(rel, ':') || rel[0] == '/' || rel[0] == '\\' || dir == NULL || !dir[0]) {
                gw_log("geno: %s: lua.script \"%s\" must be a relative path inside the mod - refused", where, rel);
                return 0;
            }
            snprintf(path, sizeof path, "%s\\%s\\%s", dir, mod, rel);
            file_text = gn_read_file(path);
            if (file_text == NULL) { gw_log("geno: %s: cannot read lua.script %s - refused", where, path); return 0; }
            src = file_text;
        } else {
            src = d->n[so].str;
        }
        srclen = strlen(src);
        if (srclen == 0 || srclen > GN_LUA_MAX_SOURCE) {
            gw_log("geno: %s: the Lua module must be 1..%d bytes - refused", where, GN_LUA_MAX_SOURCE);
            free(file_text);
            return 0;
        }
        if (sl >= 0) {
            if (d->n[sl].type != JN_OBJ) { gw_log("geno: %s: lua.state must be an object of name: type - refused", where); free(file_text); return 0; }
            for (c = d->n[sl].first; c >= 0; c = d->n[c].next) {
                const char *ty = d->n[c].type == JN_STR ? d->n[c].str : "";
                int t = !strcmp(ty, "int") ? GLUA_INT : !strcmp(ty, "float") ? GLUA_FLOAT : !strcmp(ty, "bool") ? GLUA_BOOL : -1;
                int j;
                if (nslot >= GENO_LUA_STATE_SLOTS) { gw_log("geno: %s: lua.state has more than %d slots - refused", where, GENO_LUA_STATE_SLOTS); free(file_text); return 0; }
                if (t < 0 || !gn_lua_name_ok(d->n[c].key, GLUA_SLOT_NAME)) {
                    gw_log("geno: %s: lua.state \"%s\": a name of 1..%d letters, digits and underscores, and a type of int / float / bool - refused",
                           where, d->n[c].key, GLUA_SLOT_NAME - 1);
                    free(file_text);
                    return 0;
                }
                for (j = 0; j < nslot; ++j)
                    if (!strcmp(names[j], d->n[c].key)) { gw_log("geno: %s: lua.state \"%s\" is declared twice - refused", where, d->n[c].key); free(file_text); return 0; }
                snprintf(names[nslot], sizeof names[0], "%s", d->n[c].key);
                types[nslot++] = t;
            }
        }
    }
    g = glua_new();
    if (g == NULL) { gw_log("geno: %s: Lua could not allocate - refused", where); free(file_text); return 0; }
    for (k = 0; k < p->nstate && k < GENO_MAX_STATES; ++k) snprintf(tnames[k], sizeof tnames[0], "%s", p->st_name[k]);
    glua_set_layout(g, nslot, names, types);
    glua_set_targets(g, p->nstate, tnames);
    {
        char chunk[96];
        snprintf(chunk, sizeof chunk, "=%s", jd_get(d, lb, "script") >= 0 ? d->n[jd_get(d, lb, "script")].str : "lua.source");
        if (!glua_load(g, src, srclen, chunk)) {
            gw_log("geno: %s: the Lua module was refused: %s", where, g->err);
            glua_free(g);
            free(file_text);
            return 0;
        }
    }
    for (k = 0; k < GENO_MAX_STATES; ++k) p->st_lua[k][0] = p->st_lua[k][1] = -1;
    if (st >= 0 && d->n[st].type == JN_ARR) {
        static const char *const phases[GENO_LUA_PHASES] = { "enter", "frame" };
        for (c = d->n[st].first, k = 0; c >= 0 && k < p->nstate; c = d->n[c].next, ++k) {
            int lk = jd_get(d, c, "lua"), ph;
            if (lk < 0) continue;
            if (d->n[lk].type != JN_OBJ) { gw_log("geno: %s: state %s: \"lua\" must be an object {enter, frame} - refused", where, p->st_name[k]); goto done; }
            for (ph = 0; ph < GENO_LUA_PHASES; ++ph) {
                int v = jd_get(d, lk, phases[ph]);
                if (v < 0) continue;
                if (d->n[v].type != JN_STR || (p->st_lua[k][ph] = glua_find(g, d->n[v].str)) < 0) {
                    gw_log("geno: %s: state %s: lua.%s \"%s\" is not a function of the module - refused", where, p->st_name[k], phases[ph],
                           d->n[v].type == JN_STR ? d->n[v].str : "?");
                    goto done;
                }
            }
            if (p->st_lua[k][GENO_LUA_PHASE_FRAME] >= 0 && p->st_cb[k][GENO_CB_ANIM] < 0) {
                p->st_cb[k][GENO_CB_ANIM] = gw_GenoGame_CallbackFind(GENO_CB_ANIM, "lua");
                if (p->st_cb[k][GENO_CB_ANIM] < 0) { gw_log("geno: %s: this build has no \"lua\" anim callback - refused", where); goto done; }
            }
        }
    }
    p->lua_hash = gn_lua_mix_text(0x4C55412D56310000ull, src, srclen); /* "LUA-V1" */
    for (k = 0; k < nslot; ++k) p->lua_hash = gn_lua_mix_text(p->lua_hash, names[k], strlen(names[k]) + 1) ^ (uint64_t) types[k];
    ok = 1;
done:
    if (ok) p->lua = g; else glua_free(g);
    free(file_text);
    return ok;
}

/* ---- scalar functions for the game half (pc/geno/geno_game_lua.inc) and the LAB ----------------------------------------- */

int gw_Geno_LuaFn(int p, int s, int phase)
{
    const gn_profile *x = gn_at(p);
    return x != NULL && x->lua != NULL && s >= 0 && s < GENO_MAX_STATES && phase >= 0 && phase < GENO_LUA_PHASES && x->st_lua[s][phase] >= 0;
}

int gw_Geno_LuaSlots(int p)
{
    const gn_profile *x = gn_at(p);
    return x != NULL && x->lua != NULL ? x->lua->nslot : 0;
}

int gw_Geno_LuaSlotType(int p, int i)
{
    const gn_profile *x = gn_at(p);
    return x != NULL && x->lua != NULL && i >= 0 && i < x->lua->nslot ? x->lua->slot_type[i] : -1;
}

const char *gw_Geno_LuaSlotName(int p, int i)
{
    const gn_profile *x = gn_at(p);
    return x != NULL && x->lua != NULL && i >= 0 && i < x->lua->nslot ? x->lua->slot_name[i] : "";
}

/* Runs state s's function for the phase over the game's io block. GENO_LUA_NONE when there is none; 0 ok; a GENO_LUA_FAULT_* otherwise. */
int gw_Geno_LuaCall(int p, int s, int phase, void *io)
{
    const gn_profile *x = gn_at(p);
    gn_lua *g;
    int rc;
    if (!gw_Geno_LuaFn(p, s, phase) || io == NULL) return GENO_LUA_NONE;
    g = x->lua;
    rc = glua_call(g, x->st_lua[s][phase], (uint32_t *) io);
    if (rc != 0 && g->faults <= 8) {
        gw_log("geno: lua fault %d in profile %d state %s (%s): %s%s", rc, p, x->st_name[s], phase == GENO_LUA_PHASE_ENTER ? "enter" : "frame",
               g->err, g->faults == 8 ? " (further faults of this profile are not logged)" : "");
    }
    return rc;
}
