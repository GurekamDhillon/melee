/* geno_combo_registry.inc - geno 11: the "combo" block of a define (docs/geno.md section 25). Included by geno_registry.c after
 * geno_lua_registry.inc (a route's Lua gate names a slot of the define's lua.state).
 *
 *   "combo": {
 *     "buffer": 5, "max_links": 5,                                     defaults for every route / the string length cap
 *     "ki": { "max": 100, "start": 0, "on_hit": 4, "on_taken": 1, "on_block": 0 },     an optional meter (fighter state)
 *     "routes": [ { "name": "L1>L2", "from": ["jab1", "geno:Light1"], "to": "geno:Light2",
 *                   "input": "light", "dir": "any", "on": "hit", "where": "ground", "window": [6, 24],
 *                   "buffer": 5, "hold": 0, "priority": 10, "ki_cost": 0, "ki_min": 0, "depth": 5, "once": false,
 *                   "when": { "slot": "armed", "min": 1 } } ],
 *     "strings": [ { "name": "light x5", "steps": ["jab1", "geno:Light2", ...] } ]       documentation + tools/geno check
 *   }
 *
 * Strict like the rest of a define: an attach entry with the key, a file older than "geno": 11, an unknown name or a value out of
 * range refuses the entry with the reason in the log. A route with a list "from" becomes one parsed route per source. Parsed routes
 * are stably sorted by priority (highest first), so the evaluation order is the file order within one priority. The block is in the
 * entry's content id (the id hashes the whole entry), so it is in the online identity too. An entry without the key is unchanged. */

typedef struct gn_combo {
    int n;
    int32_t w[GENO_COMBO_MAX_ROUTES][GENO_CR_COUNT];
    char name[GENO_COMBO_MAX_ROUTES][GENO_COMBO_MAX_NAME];
    int32_t param[GENO_CP_COUNT];
} gn_combo;

static void gn_combo_free_all(gn_registry *r)
{
    int i;
    for (i = 0; i < r->n; ++i) {
        free(r->p[i].combo);
        r->p[i].combo = NULL;
    }
}

/* Melee's common moves by name: the motion ids a "from" covers (first = what a "to" enters). */
static const struct { const char *name; int first, count; } gn_combo_moves[] = {
    {"jab1", 44, 1}, {"jab2", 45, 1}, {"jab3", 46, 1}, {"rapid_jab", 47, 3}, {"dash_attack", 50, 1},
    {"ftilt", 53, 1}, {"ftilt_any", 51, 5}, {"utilt", 56, 1}, {"dtilt", 57, 1},
    {"fsmash", 60, 1}, {"fsmash_any", 58, 5}, {"usmash", 63, 1}, {"dsmash", 64, 1},
    {"nair", 65, 1}, {"fair", 66, 1}, {"bair", 67, 1}, {"uair", 68, 1}, {"dair", 69, 1},
    {"grab", 212, 1}, {"dash_grab", 214, 1}, {"fthrow", 219, 1}, {"bthrow", 220, 1}, {"uthrow", 221, 1}, {"dthrow", 222, 1},
};

static int gn_combo_pick(const char *s, const char *const *names, int n)
{
    int i;
    for (i = 0; i < n; ++i)
        if (_stricmp(s, names[i]) == 0) return i;
    return -1;
}

/* A "from" item: fills up to 8 target words. Returns the count, 0 when invalid. */
static int gn_combo_from(const jdoc *d, int x, const gn_profile *p, uint32_t *out)
{
    int i, k;
    if (d->n[x].type == JN_STR) {
        const char *s = d->n[x].str;
        if (strcmp(s, "*") == 0) { out[0] = GENO_TARGET(GENO_TGT_MOTION, GENO_COMBO_FROM_ANY); return 1; }
        for (i = 0; i < (int) (sizeof gn_combo_moves / sizeof gn_combo_moves[0]); ++i)
            if (_stricmp(s, gn_combo_moves[i].name) == 0) {
                for (k = 0; k < gn_combo_moves[i].count; ++k) out[k] = GENO_TARGET(GENO_TGT_MOTION, gn_combo_moves[i].first + k);
                return gn_combo_moves[i].count;
            }
    }
    if (!gn_target_p(d, x, out, p)) return 0;
    if ((out[0] >> 28) != GENO_TGT_MOTION && (out[0] >> 28) != GENO_TGT_GENO) return 0; /* a source is a motion or a Geno state */
    return 1;
}

static int gn_combo_to(const jdoc *d, int x, const gn_profile *p, uint32_t *out)
{
    static const char *const acts[] = { "special_n", "special_s", "special_hi", "special_lw", "", "", "", "",
                                        "jump", "nair", "fair", "bair", "uair", "dair", "jump_short" };
    int i;
    if (d->n[x].type == JN_STR) {
        const char *s = d->n[x].str;
        i = gn_combo_pick(s, acts, (int) (sizeof acts / sizeof acts[0]));
        if (i >= 0 && acts[i][0]) { *out = GENO_TARGET(GENO_COMBO_TGT_ACT, i); return 1; }
        for (i = 0; i < (int) (sizeof gn_combo_moves / sizeof gn_combo_moves[0]); ++i)
            if (_stricmp(s, gn_combo_moves[i].name) == 0) { *out = GENO_TARGET(GENO_TGT_MOTION, gn_combo_moves[i].first); return 1; }
    }
    if (!gn_target_p(d, x, out, p)) return 0;
    return *out != GENO_TGT_STAY;
}

static int gn_combo_int(const jdoc *d, int obj, const char *key, int def, int lo, int hi, int *out, const char *where, const char *route)
{
    int v = jd_get(d, obj, key);
    if (v < 0) { *out = def; return 1; }
    if (d->n[v].type == JN_BOOL) { *out = d->n[v].num != 0; return 1; }
    if (d->n[v].type != JN_NUM || d->n[v].num < lo || d->n[v].num > hi || d->n[v].num != (double) (int) d->n[v].num) {
        gw_log("geno: %s: combo%s%s: \"%s\" must be an integer %d..%d - refused", where, route[0] ? " route " : "", route, key, lo, hi);
        return 0;
    }
    *out = (int) d->n[v].num;
    return 1;
}

/* Ki amounts are written in meter units (0..1000, two decimals) and kept in hundredths. */
static int gn_combo_ki(const jdoc *d, int obj, const char *key, int def, int *out, const char *where, const char *route)
{
    int v = jd_get(d, obj, key);
    if (v < 0) { *out = def; return 1; }
    if (d->n[v].type != JN_NUM || !(d->n[v].num >= 0.0 && d->n[v].num <= 1000.0)) {
        gw_log("geno: %s: combo%s%s: \"%s\" must be a number 0..1000 - refused", where, route[0] ? " route " : "", route, key);
        return 0;
    }
    *out = (int) (d->n[v].num * 100.0 + 0.5);
    return 1;
}

static int gn_combo_enum(const jdoc *d, int obj, const char *key, int def, const char *const *names, int n, int *out,
                         const char *where, const char *route)
{
    int v = jd_get(d, obj, key), i;
    if (v < 0) { *out = def; return 1; }
    i = d->n[v].type == JN_STR ? gn_combo_pick(d->n[v].str, names, n) : -1;
    if (i < 0) {
        gw_log("geno: %s: combo route %s: \"%s\" is not one of the known names - refused", where, route, key);
        return 0;
    }
    *out = i;
    return 1;
}

/* 1 when the entry is fine (with or without a combo block); 0 refuses it. */
static int gn_combo_parse(gn_profile *p, const jdoc *d, int e, const char *where, int version)
{
    static const char *const inputs[] = { "light", "heavy", "special", "jump", "grab", "shield" };
    static const char *const dirs[] = { "any", "neutral", "forward", "back", "up", "down", "side" };
    static const char *const ons[] = { "any", "hit", "whiff", "block", "contact" };
    static const char *const wheres[] = { "any", "ground", "air" };
    int cb = jd_get(d, e, "combo"), rl, c, ki, buffer, max_links, k, i, j;
    gn_combo *g;
    if (cb < 0) return 1;
    if (!p->defined) { gw_log("geno: %s: a combo graph is for a define; an attach entry cannot carry \"combo\" - refused", where); return 0; }
    if (version < 11) { gw_log("geno: %s: \"combo\" needs \"geno\": 11 - refused", where); return 0; }
    if (d->n[cb].type != JN_OBJ) { gw_log("geno: %s: \"combo\" must be an object - refused", where); return 0; }
    g = (gn_combo *) calloc(1, sizeof *g);
    if (g == NULL) { gw_log("geno: %s: combo: out of memory - refused", where); return 0; }
    if (!gn_combo_int(d, cb, "buffer", 5, 0, 30, &buffer, where, "") || !gn_combo_int(d, cb, "max_links", 5, 1, 32, &max_links, where, ""))
        goto bad;
    g->param[GENO_CP_MAX_LINKS] = max_links;
    ki = jd_get(d, cb, "ki");
    if (ki >= 0) {
        if (d->n[ki].type != JN_OBJ) { gw_log("geno: %s: combo.ki must be an object - refused", where); goto bad; }
        if (!gn_combo_ki(d, ki, "max", 100, &g->param[GENO_CP_KI_MAX], where, "") ||
            !gn_combo_ki(d, ki, "start", 0, &g->param[GENO_CP_KI_START], where, "") ||
            !gn_combo_ki(d, ki, "on_hit", 0, &g->param[GENO_CP_KI_ON_HIT], where, "") ||
            !gn_combo_ki(d, ki, "on_taken", 0, &g->param[GENO_CP_KI_ON_TAKEN], where, "") ||
            !gn_combo_ki(d, ki, "on_block", 0, &g->param[GENO_CP_KI_ON_BLOCK], where, ""))
            goto bad;
        if (g->param[GENO_CP_KI_START] > g->param[GENO_CP_KI_MAX]) { gw_log("geno: %s: combo.ki.start is above max - refused", where); goto bad; }
    }
    rl = jd_get(d, cb, "routes");
    if (rl < 0 || d->n[rl].type != JN_ARR) { gw_log("geno: %s: combo needs a \"routes\" list - refused", where); goto bad; }
    for (c = d->n[rl].first, k = 0; c >= 0; c = d->n[c].next, ++k) {
        char rname[GENO_COMBO_MAX_NAME];
        int32_t w[GENO_CR_COUNT];
        uint32_t from[8 * 8];
        int nfrom = 0, f, x, lo, hi, v;
        int nm = jd_get(d, c, "name");
        if (d->n[c].type != JN_OBJ) { gw_log("geno: %s: combo route %d is not an object - refused", where, k); goto bad; }
        if (nm >= 0 && d->n[nm].type == JN_STR && d->n[nm].str[0]) snprintf(rname, sizeof rname, "%s", d->n[nm].str);
        else snprintf(rname, sizeof rname, "route%d", k);
        for (i = 0; i < GENO_CR_COUNT; ++i) w[i] = 0;
        x = jd_get(d, c, "from");
        if (x >= 0 && d->n[x].type == JN_ARR) {
            for (f = d->n[x].first; f >= 0; f = d->n[f].next) {
                int got = nfrom < 56 ? gn_combo_from(d, f, p, from + nfrom) : 0;
                if (got == 0) { gw_log("geno: %s: combo route %s: a \"from\" item is not a move, a state or \"*\" (or there are too many) - refused", where, rname); goto bad; }
                nfrom += got;
            }
        } else if (x >= 0) {
            nfrom = gn_combo_from(d, x, p, from);
        }
        if (nfrom == 0) { gw_log("geno: %s: combo route %s: \"from\" is missing or names no move or state - refused", where, rname); goto bad; }
        x = jd_get(d, c, "to");
        if (x < 0 || !gn_combo_to(d, x, p, (uint32_t *) &w[GENO_CR_TO])) {
            gw_log("geno: %s: combo route %s: \"to\" is missing or is not a target (geno:<state>, a move name, special_n/s/hi/lw, jump, nair..dair, auto) - refused", where, rname);
            goto bad;
        }
        v = jd_get(d, c, "input");
        if (v >= 0 && d->n[v].type == JN_STR && _stricmp(d->n[v].str, "none") == 0) w[GENO_CR_INPUT] = GENO_CI_NONE;
        else if (v < 0) { gw_log("geno: %s: combo route %s: \"input\" is required (light, heavy, special, jump, grab, shield, none) - refused", where, rname); goto bad; }
        else if (!gn_combo_enum(d, c, "input", 0, inputs, 6, &w[GENO_CR_INPUT], where, rname)) goto bad;
        if (!gn_combo_enum(d, c, "dir", GENO_CD_ANY, dirs, 7, &w[GENO_CR_DIR], where, rname) ||
            !gn_combo_enum(d, c, "on", GENO_CO_HIT, ons, 5, &w[GENO_CR_ON], where, rname) ||
            !gn_combo_enum(d, c, "where", GENO_CW_ANY, wheres, 3, &w[GENO_CR_WHERE], where, rname) ||
            !gn_combo_int(d, c, "buffer", buffer, 0, 30, &w[GENO_CR_BUFFER], where, rname) ||
            !gn_combo_int(d, c, "hold", 0, 0, 600, &w[GENO_CR_HOLD], where, rname) ||
            !gn_combo_int(d, c, "priority", 0, -1000, 1000, &w[GENO_CR_PRIORITY], where, rname) ||
            !gn_combo_ki(d, c, "ki_cost", 0, &w[GENO_CR_KI_COST], where, rname) ||
            !gn_combo_ki(d, c, "ki_min", 0, &w[GENO_CR_KI_MIN], where, rname) ||
            !gn_combo_int(d, c, "depth", max_links, 1, 32, &w[GENO_CR_DEPTH], where, rname) ||
            !gn_combo_int(d, c, "once", 0, 0, 1, &w[GENO_CR_ONCE], where, rname) ||
            !gn_combo_int(d, c, "starts", 0, 0, 1, &w[GENO_CR_STARTS], where, rname))
            goto bad;
        lo = 1; hi = 0;
        x = jd_get(d, c, "window");
        if (x >= 0) {
            int a = d->n[x].type == JN_ARR && d->n[x].count == 2 ? d->n[x].first : -1, b = a >= 0 ? d->n[a].next : -1;
            if (a < 0 || d->n[a].type != JN_NUM || d->n[b].type != JN_NUM || d->n[a].num < 1 || d->n[a].num > 9999 ||
                d->n[b].num < 0 || d->n[b].num > 9999 || (d->n[b].num != 0 && d->n[b].num < d->n[a].num)) {
                gw_log("geno: %s: combo route %s: \"window\" must be [first, last] action frames (1-based; last 0 = to the end) - refused", where, rname);
                goto bad;
            }
            lo = (int) d->n[a].num; hi = (int) d->n[b].num;
        }
        w[GENO_CR_WIN_LO] = lo; w[GENO_CR_WIN_HI] = hi;
        w[GENO_CR_SLOT] = -1;
        x = jd_get(d, c, "when");
        if (x >= 0) {
            int sl = d->n[x].type == JN_OBJ ? jd_get(d, x, "slot") : -1, idx = -1;
            if (sl >= 0 && d->n[sl].type == JN_STR && p->lua != NULL)
                for (i = 0; i < p->lua->nslot; ++i)
                    if (strcmp(p->lua->slot_name[i], d->n[sl].str) == 0) idx = i;
            if (idx < 0) { gw_log("geno: %s: combo route %s: \"when\".slot must name a slot of lua.state - refused", where, rname); goto bad; }
            if (!gn_combo_int(d, x, "min", 1, -1000000, 1000000, &w[GENO_CR_SLOT_MIN], where, rname)) goto bad;
            w[GENO_CR_SLOT] = idx;
        }
        if (w[GENO_CR_INPUT] == GENO_CI_NONE && w[GENO_CR_ON] == GENO_CO_ANY && hi == 0) {
            gw_log("geno: %s: combo route %s: an input-less route needs a condition or a closed window - refused", where, rname);
            goto bad;
        }
        for (f = 0; f < nfrom; ++f) {
            if (g->n >= GENO_COMBO_MAX_ROUTES) { gw_log("geno: %s: combo: more than %d routes (after expanding \"from\" lists) - refused", where, GENO_COMBO_MAX_ROUTES); goto bad; }
            for (i = 0; i < GENO_CR_COUNT; ++i) g->w[g->n][i] = w[i];
            g->w[g->n][GENO_CR_FROM] = (int32_t) from[f];
            snprintf(g->name[g->n], GENO_COMBO_MAX_NAME, "%s", rname);
            g->n++;
        }
    }
    /* stable sort by priority, highest first (insertion sort: n <= 128, done once at load) */
    for (i = 1; i < g->n; ++i) {
        int32_t t[GENO_CR_COUNT];
        char tn[GENO_COMBO_MAX_NAME];
        memcpy(t, g->w[i], sizeof t);
        memcpy(tn, g->name[i], sizeof tn);
        for (j = i - 1; j >= 0 && g->w[j][GENO_CR_PRIORITY] < t[GENO_CR_PRIORITY]; --j) {
            memcpy(g->w[j + 1], g->w[j], sizeof t);
            memcpy(g->name[j + 1], g->name[j], sizeof tn);
        }
        memcpy(g->w[j + 1], t, sizeof t);
        memcpy(g->name[j + 1], tn, sizeof tn);
    }
    g->param[GENO_CP_ROUTES] = g->n;
    p->combo = g;
    gw_log("geno: %s: combo graph: %d routes, max %d links, ki max %d", where, g->n, max_links, g->param[GENO_CP_KI_MAX] / 100);
    return 1;
bad:
    free(g);
    return 0;
}

/* ---- scalar functions for the game half (pc/geno/geno_game_combo.inc) and the LAB --------------------------------------- */

int gw_Geno_ComboParam(int p, int what)
{
    const gn_profile *x = gn_at(p);
    if (x == NULL || x->combo == NULL || what < 0 || what >= GENO_CP_COUNT) return 0;
    return x->combo->param[what];
}

int gw_Geno_ComboWord(int p, int route, int word)
{
    const gn_profile *x = gn_at(p);
    if (x == NULL || x->combo == NULL || route < 0 || route >= x->combo->n || word < 0 || word >= GENO_CR_COUNT) return 0;
    return x->combo->w[route][word];
}

const char *gw_Geno_ComboName(int p, int route)
{
    const gn_profile *x = gn_at(p);
    if (x == NULL || x->combo == NULL || route < 0 || route >= x->combo->n) return "";
    return x->combo->name[route];
}

/* The name of a Geno state of profile p (for the LAB's combo panel). */
const char *gw_Geno_ComboStateName(int p, int s)
{
    const gn_profile *x = gn_at(p);
    if (x == NULL || s < 0 || s >= x->nstate) return "";
    return x->st_name[s];
}

/* The game half's log of a taken link (diagnostics: rate-limited, never state; resimulation repeats frames). */
void gw_Geno_ComboNote(int p, int route, int from, int to, int frame, int ki_links)
{
    static int logged;
    if (logged >= 4000) return;
    logged++;
    gw_log("geno: combo link %d: route %d (%s) motion %d -> %d at action frame %d, ki %.2f", (ki_links >> 24) & 0xFF, route,
           gw_Geno_ComboName(p, route), from, to, frame, (double) (ki_links & 0xFFFFFF) / 100.0);
}
