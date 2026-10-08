/* slice 4 (v9): the entry-level "fighter" block of a base "none" define:
 *   "fighter": {"plan": "plan.json", "animation": "GnCourierAJ.dat",
 *               "costumes": [{"file": "GnCourier_default.dat", "joint": "courier_joint", "matanim": "courier_matanim"}, ...]}
 * Every name is a file of the mod's files/ folder (mounted by name) or a symbol inside it. */
static int gn_fm_str(const jdoc* d, int o, const char* key, char* out, int cap, int required)
{
    int v = jd_get(d, o, key);
    if (v < 0) { if (required) return 0; out[0] = 0; return 1; }
    if (d->n[v].type != JN_STR || !d->n[v].str[0] || (int)strlen(d->n[v].str) >= cap || strpbrk(d->n[v].str, "\/:") ||
        strstr(d->n[v].str, "..")) return 0;
    snprintf(out, (size_t)cap, "%s", d->n[v].str); return 1;
}
static int gn_define_fighter(gn_profile* p, const jdoc* d, int e, const char* where)
{
    int f = jd_get(d, e, "fighter"), cs, c;
    struct gn_fm* fm;
    if (f < 0 || d->n[f].type != JN_OBJ) { gw_log("geno: %s: a base \"none\" define needs a \"fighter\" block (plan, animation, costumes)", where); return 0; }
    fm = (struct gn_fm*)calloc(1, sizeof *fm);
    if (!fm) return 0;
    if (!gn_fm_str(d, f, "plan", fm->plan_file, sizeof fm->plan_file, 1) || !gn_fm_str(d, f, "animation", fm->anim_file, sizeof fm->anim_file, 1)) {
        gw_log("geno: %s: fighter.plan and fighter.animation must be plain file names", where); free(fm); return 0;
    }
    cs = jd_get(d, f, "costumes");
    if (cs < 0 || d->n[cs].type != JN_ARR || d->n[cs].count < 1 || d->n[cs].count > 16) {
        gw_log("geno: %s: fighter.costumes must list 1..16 costumes", where); free(fm); return 0;
    }
    for (c = d->n[cs].first; c >= 0; c = d->n[c].next) {
        int i = fm->ncostume++;
        if (d->n[c].type != JN_OBJ || !gn_fm_str(d, c, "file", fm->cos_file[i], sizeof fm->cos_file[i], 1) ||
            !gn_fm_str(d, c, "joint", fm->cos_joint[i], sizeof fm->cos_joint[i], 1) ||
            !gn_fm_str(d, c, "matanim", fm->cos_matanim[i], sizeof fm->cos_matanim[i], 0)) {
            gw_log("geno: %s: fighter.costumes[%d] needs file and joint (matanim optional)", where, i); free(fm); return 0;
        }
        /* slice 6: a declared colour set. "name" is what the select shows; "team" says which team battle colour this costume is. */
        fm->cos_team[i] = -1;
        {
            int nv = jd_get(d, c, "name"), tv = jd_get(d, c, "team"), k;
            if (nv >= 0) {
                if (d->n[nv].type != JN_STR || !d->n[nv].str[0] || strlen(d->n[nv].str) >= sizeof fm->cos_name[i]) {
                    gw_log("geno: %s: fighter.costumes[%d].name must be a string of 1..23 characters", where, i); free(fm); return 0;
                }
                for (k = 0; d->n[nv].str[k]; ++k)
                    if ((unsigned char)d->n[nv].str[k] < 32 || (unsigned char)d->n[nv].str[k] > 126) {
                        gw_log("geno: %s: fighter.costumes[%d].name must be printable ASCII", where, i); free(fm); return 0;
                    }
                snprintf(fm->cos_name[i], sizeof fm->cos_name[i], "%s", d->n[nv].str);
            }
            if (tv >= 0) {
                const char* t = d->n[tv].type == JN_STR ? d->n[tv].str : "";
                if (!strcmp(t, "red")) fm->cos_team[i] = 0;
                else if (!strcmp(t, "blue")) fm->cos_team[i] = 1;
                else if (!strcmp(t, "green")) fm->cos_team[i] = 2;
                else { gw_log("geno: %s: fighter.costumes[%d].team must be \"red\", \"blue\" or \"green\"", where, i); free(fm); return 0; }
            }
        }
    }
    p->fm = fm;
    return 1;
}

/* slice 6: the optional entry-level "presentation": {"icon": "a.gxtex", "portrait": "b.gxtex" | [..16], "stock": "c.gxtex" | [..16]}.
 * Strict like the rest of a define: an unknown key, a name that is not a plain *.gxtex file name, or more than 16 entries refuses the
 * entry with the reason in the log. Whether the file exists is checked when it is first used (and offline by tools.geno.check):
 * a missing file falls back to the donor's (or no) art and logs once. Nothing here is simulation state. */
static int gn_pres_name_ok(const char* s)
{
    size_t n = strlen(s);
    if (n < 7 || n >= 64 || _stricmp(s + n - 6, ".gxtex") || strpbrk(s, "/\\:") || strstr(s, "..")) return 0;
    for (; *s; ++s) if ((unsigned char)*s < 33 || (unsigned char)*s > 126) return 0;
    return 1;
}
static int gn_define_presentation(gn_profile* p, const jdoc* d, int e, int version, const char* where)
{
    static const char* keys[GN_PRES_KINDS] = {"icon", "portrait", "stock", "emblem"};
    int pv = jd_get(d, e, "presentation"), c, k;
    struct gn_pres* pr;
    if (pv < 0) return 1;
    if (d->n[pv].type != JN_OBJ) { gw_log("geno: %s: presentation must be an object", where); return 0; }
    for (c = d->n[pv].first; c >= 0; c = d->n[c].next) {
        for (k = 0; k < GN_PRES_KINDS && strcmp(d->n[c].key, keys[k]); ++k) {}
        if (k == GN_PRES_KINDS) { gw_log("geno: %s: presentation key \"%s\" is not known (icon, portrait, stock, emblem)", where, d->n[c].key); return 0; }
    }
    pr = (struct gn_pres*)calloc(1, sizeof *pr);
    if (!pr) return 0;
    for (k = 0; k < GN_PRES_KINDS; ++k) {
        int v = jd_get(d, pv, keys[k]), q, n = 0;
        if (v < 0) continue;
        if (d->n[v].type == JN_STR) {
            if (!gn_pres_name_ok(d->n[v].str)) goto bad;
            snprintf(pr->file[k][0], sizeof pr->file[k][0], "%s", d->n[v].str);
            n = 1;
        } else if (d->n[v].type == JN_ARR && k != GN_PRES_ICON && k != GN_PRES_EMBLEM && d->n[v].count >= 1 && d->n[v].count <= 16) {
            for (q = d->n[v].first; q >= 0; q = d->n[q].next) {
                if (d->n[q].type != JN_STR || !gn_pres_name_ok(d->n[q].str)) goto bad;
                snprintf(pr->file[k][n++], sizeof pr->file[k][0], "%s", d->n[q].str);
            }
        } else goto bad;
        pr->n[k] = n;
    }
    p->pres = pr;
    return 1;
bad:
    gw_log("geno: %s: presentation.%s must be one plain *.gxtex file name (portrait and stock: or 1..16 of them; icon and emblem: one)", where, keys[k]);
    free(pr);
    return 0;
}

/* ---- slice 6 (geno 10): AI, Kirby copy and own audio keys -------------------------------------------------------------------------------
 *   "ai": {"like": "<retail fighter>"}           the CPU plays this define as that fighter (every kind switch and the per-kind AI tables answer for it)
 *   "kirby_copy": "none" | "retail:<fighter>"      what Kirby gets from inhaling the define: nothing (the default, made explicit and guarded), or that
 *                                                  retail fighter's hat and copy ability
 *   "audio": {"announcer": "a.gnsnd", "voice": [{"sfx": 1234, "file": "b.gnsnd", "volume": 100}]}   own sound clips (tools/geno/audio.py)
 * ai and kirby_copy change what a fighter DOES (the CPU's moves, what Kirby becomes), so they need "geno": 10, the first format that carries them; audio is
 * presentation like the art keys and stays at "geno": 9 (the owner's rule: only keys that change the simulation, and a lua block, take 10). An unknown key
 * inside any of them refuses the entry. */
static int gn_retail_kind(const char* name)
{
    int i;
    for (i = 0; i < 27; ++i) {
        if (i == 11) continue; /* Nana: Popo's partner, not a fighter one picks or copies */
        if (!_stricmp(name, gn_vanilla[i].name) || (gn_vanilla[i].alias && !_stricmp(name, gn_vanilla[i].alias))) return i;
    }
    return -1;
}
static int gn_file_name_ok(const char* s, const char* ext)
{
    size_t n = strlen(s), m = strlen(ext);
    if (n < m + 1 || n >= 64 || _stricmp(s + n - m, ext) || strpbrk(s, "/\\:") || strstr(s, "..")) return 0;
    for (; *s; ++s) if ((unsigned char)*s < 33 || (unsigned char)*s > 126) return 0;
    return 1;
}
static int gn_define_s6_keys(gn_profile* p, const jdoc* d, int e, int version, const char* where)
{
    int v, c, k;
    p->ai_like = -1; p->ai_set = 0; p->kb_copy = -1;
    if (version < 10 && (jd_get(d, e, "ai") >= 0 || jd_get(d, e, "kirby_copy") >= 0)) {
        gw_log("geno: %s: ai and kirby_copy need \"geno\": 10", where); return 0;
    }
    if (version < 9 && jd_get(d, e, "audio") >= 0) { gw_log("geno: %s: audio needs \"geno\": 9", where); return 0; }
    v = jd_get(d, e, "ai");
    if (v >= 0) {
        int lk;
        if (d->n[v].type != JN_OBJ) { gw_log("geno: %s: ai must be an object {\"like\": <retail fighter>}", where); return 0; }
        for (c = d->n[v].first; c >= 0; c = d->n[c].next)
            if (strcmp(d->n[c].key, "like")) { gw_log("geno: %s: ai key \"%s\" is not known (like)", where, d->n[c].key); return 0; }
        lk = jd_get(d, v, "like");
        if (lk < 0 || d->n[lk].type != JN_STR || (p->ai_like = gn_retail_kind(d->n[lk].str)) < 0) {
            gw_log("geno: %s: ai.like must name a retail fighter (mario, fox, captain, donkey, kirby, koopa, link, seak, ness, peach, popo, pikachu, samus, yoshi, purin, mewtwo, luigi, mars, zelda, clink, drmario, falco, pichu, gamewatch, ganon, emblem)", where);
            p->ai_like = -1; return 0;
        }
        p->ai_set = 1;
    }
    v = jd_get(d, e, "kirby_copy");
    if (v >= 0) {
        const char* t = d->n[v].type == JN_STR ? d->n[v].str : "";
        if (!strcmp(t, "none")) p->kb_copy = -1;
        else if (!strncmp(t, "retail:", 7) && (p->kb_copy = gn_retail_kind(t + 7)) >= 0 && p->kb_copy != 4) {}
        else {
            gw_log("geno: %s: kirby_copy must be \"none\" or \"retail:<fighter>\" (a retail fighter other than Kirby, e.g. \"retail:mario\")", where);
            p->kb_copy = -1; return 0;
        }
    }
    v = jd_get(d, e, "audio");
    if (v >= 0) {
        struct gn_aud* a;
        int vo;
        if (d->n[v].type != JN_OBJ) { gw_log("geno: %s: audio must be an object", where); return 0; }
        for (c = d->n[v].first; c >= 0; c = d->n[c].next)
            if (strcmp(d->n[c].key, "announcer") && strcmp(d->n[c].key, "voice")) { gw_log("geno: %s: audio key \"%s\" is not known (announcer, voice)", where, d->n[c].key); return 0; }
        a = (struct gn_aud*)calloc(1, sizeof *a);
        if (!a) return 0;
        c = jd_get(d, v, "announcer");
        if (c >= 0) {
            if (d->n[c].type != JN_STR || !gn_file_name_ok(d->n[c].str, ".gnsnd")) { gw_log("geno: %s: audio.announcer must be one plain *.gnsnd file name", where); free(a); return 0; }
            snprintf(a->announcer, sizeof a->announcer, "%s", d->n[c].str);
        }
        vo = jd_get(d, v, "voice");
        if (vo >= 0) {
            if (d->n[vo].type != JN_ARR || d->n[vo].count < 1 || d->n[vo].count > GN_MAX_VOICE) { gw_log("geno: %s: audio.voice must list 1..%d rows", where, GN_MAX_VOICE); free(a); return 0; }
            for (c = d->n[vo].first; c >= 0; c = d->n[c].next) {
                int sx, fl, vl, x, n = a->nvoice;
                if (d->n[c].type != JN_OBJ) goto bad_voice;
                for (x = d->n[c].first; x >= 0; x = d->n[x].next)
                    if (strcmp(d->n[x].key, "sfx") && strcmp(d->n[x].key, "file") && strcmp(d->n[x].key, "volume")) goto bad_voice;
                sx = jd_get(d, c, "sfx"); fl = jd_get(d, c, "file"); vl = jd_get(d, c, "volume");
                if (sx < 0 || d->n[sx].type != JN_NUM || d->n[sx].num < 1 || d->n[sx].num > 999999 || d->n[sx].num != (int)d->n[sx].num) goto bad_voice;
                if (fl < 0 || d->n[fl].type != JN_STR || !gn_file_name_ok(d->n[fl].str, ".gnsnd")) goto bad_voice;
                a->voice_vol[n] = 127;
                if (vl >= 0) {
                    if (d->n[vl].type != JN_NUM || d->n[vl].num < 0 || d->n[vl].num > 127 || d->n[vl].num != (int)d->n[vl].num) goto bad_voice;
                    a->voice_vol[n] = (int)d->n[vl].num;
                }
                a->voice_sfx[n] = (int)d->n[sx].num;
                for (k = 0; k < n; ++k) if (a->voice_sfx[k] == a->voice_sfx[n]) { gw_log("geno: %s: audio.voice names sound %d twice", where, a->voice_sfx[n]); free(a); return 0; }
                snprintf(a->voice_file[n], sizeof a->voice_file[n], "%s", d->n[fl].str);
                a->nvoice = n + 1;
            }
        }
        p->aud = a;
        return 1;
    bad_voice:
        gw_log("geno: %s: audio.voice rows are {\"sfx\": 1..999999, \"file\": \"x.gnsnd\", \"volume\": 0..127 (optional)}, each sound once", where);
        free(a); return 0;
    }
    return 1;
}

/* Included after the registry types/parser helpers. Native boot metadata only.
 * Definitions are process-immutable in GF1: changing one requires restarting.
 */
static int gn_define_parse(gn_profile* p, const jdoc* d, int e, int version, const char* where)
{
    static const char* keys[] = {"key", "name", "base", "common", "resources"};
    int def = jd_get(d, e, "define"), c, i, v;
    if (def < 0) return 0;
    /* v7 (slice 2) widens a define: the whole common attribute table, special_attributes, fx_bindings.
     * v8 (slice 3) adds articles and the named-sound table. A v6 or v7 file keeps exactly its meaning. */
    if (version < 6 || version > 10 || jd_get(d, e, "attach") >= 0 || d->n[def].type != JN_OBJ ||
        (version < 8 && (jd_get(d, e, "articles") >= 0 || jd_get(d, e, "sounds") >= 0)) ||
        (version < 7 && (jd_get(d, e, "fx_bindings") >= 0 || jd_get(d, e, "special_attributes") >= 0))) {
        if (version < 8 && (jd_get(d, e, "articles") >= 0 || jd_get(d, e, "sounds") >= 0))
            gw_log("geno: %s: define articles and sounds need \"geno\": 8", where);
        goto refused;
    }
    for (c = d->n[def].first; c >= 0; c = d->n[c].next) {
        for (i = 0; i < 5 && strcmp(d->n[c].key, keys[i]); ++i) {}
        if (i == 5 || d->n[c].type != JN_STR) goto refused;
    }
    for (i = 0; i < 5; ++i) { v = jd_get(d, def, keys[i]); if (v < 0 || d->n[v].type != JN_STR) goto refused; }
    v = jd_get(d, def, "base");
    if (!strcmp(d->n[v].str, "none")) { if (version < 9) { gw_log("geno: %s: base \"none\" needs \"geno\": 9", where); goto refused; } p->none = 1; }
    else if (strcmp(d->n[v].str, "mario")) goto refused;
    v = jd_get(d, def, "common"); if (strcmp(d->n[v].str, "melee.common.v1")) goto refused;
    v = jd_get(d, def, "resources"); if (strcmp(d->n[v].str, p->none ? "mod:files" : "retail:mario")) goto refused;
    if (p->none && !gn_define_fighter(p, d, e, where)) goto refused;
    v = jd_get(d, def, "key");
    if (!d->n[v].str[0] || strlen(d->n[v].str) >= 40 || strstr(d->n[v].str, "..")) goto refused;
    for (i = 0; d->n[v].str[i]; ++i) {
        char ch = d->n[v].str[i];
        if (!((ch >= 'a' && ch <= 'z') || (ch >= '0' && ch <= '9') || (i && (ch == '_' || ch == '.' || ch == '-')))) goto refused;
    }
    snprintf(p->define_key, sizeof p->define_key, "%s", d->n[v].str);
    v = jd_get(d, def, "name");
    if (!d->n[v].str[0] || strlen(d->n[v].str) >= 48) goto refused;
    for (i = 0; d->n[v].str[i]; ++i) if ((unsigned char)d->n[v].str[i] < 32 || (unsigned char)d->n[v].str[i] > 126) goto refused;
    snprintf(p->name, sizeof p->name, "%s", d->n[v].str);
    v = jd_get(d, e, "attributes");
    if (v >= 0) {
        if (d->n[v].type != JN_OBJ || d->n[v].count > GENO_MAX_ATTRS) goto refused;
        for (c = d->n[v].first; c >= 0; c = d->n[c].next)
            if (d->n[c].type != JN_NUM || gw_GenoGame_AttrFind(d->n[c].key) < 0 ||
                (version < 7 && gw_GenoGame_AttrFind(d->n[c].key) >= 40)) {
                gw_log("geno: %s: define attribute \"%s\" %s", where, d->n[c].key,
                       d->n[c].type != JN_NUM ? "is not a number" :
                       gw_GenoGame_AttrFind(d->n[c].key) < 0 ? "is not a common attribute Geno knows" : "needs \"geno\": 7");
                goto refused;
            }
    }
    if (!gn_sounds_check(d, e, where)) goto refused;
    if (version < 9 && jd_get(d, e, "presentation") >= 0) { gw_log("geno: %s: presentation needs \"geno\": 9", where); goto refused; }
    if (!gn_define_presentation(p, d, e, version, where)) goto refused;
    if (!gn_define_s6_keys(p, d, e, version, where)) goto refused;
    v = jd_get(d, e, "subactions");
    if (v >= 0) {
        if (d->n[v].type != JN_ARR) goto refused;
        for (c = d->n[v].first; c >= 0; c = d->n[c].next) {
            int index = jd_get(d, c, "index");
            if (index < 0 || d->n[index].type != JN_NUM || d->n[index].num < 0 ||
                d->n[index].num >= 303 || d->n[index].num != (int)d->n[index].num) goto refused;
        }
    }
    v = jd_get(d, e, "common_states");
    if (v >= 0) {
        static const char* fields[] = {"motion", "like", "subaction", "flags", "move_id", "anim", "iasa", "phys", "coll"};
        int row, j, x, k;
        if (d->n[v].type != JN_ARR || d->n[v].count > 64) goto refused;
        for (row = d->n[v].first; row >= 0; row = d->n[row].next) {
            if (d->n[row].type != JN_OBJ) goto refused;
            for (j = 0; j < 9; ++j) p->common_row[p->ncommon][j] = -1;
            p->cs_tag[p->ncommon] = gn_move_tag(d, row);
            for (x = d->n[row].first; x >= 0; x = d->n[x].next) {
                if (!strcmp(d->n[x].key, "move_tag")) continue; /* a declaration, not a row field */
                for (j = 0; j < 9 && strcmp(d->n[x].key, fields[j]); ++j) {}
                if (j == 9) goto refused;
                if (j < 5) {
                    if (d->n[x].type != JN_NUM || d->n[x].num < 0 || d->n[x].num > (j < 2 ? 350 : j == 2 ? 302 : j == 4 ? 255 : 2147483647) ||
                        d->n[x].num != (int)d->n[x].num) goto refused;
                    k = (int)d->n[x].num;
                } else {
                    if (d->n[x].type != JN_STR) goto refused;
                    k = gw_GenoGame_CallbackFind(j-5, d->n[x].str);
                    if (k < 0 && k != GENO_CB_LIKE) goto refused;
                }
                p->common_row[p->ncommon][j] = k;
            }
            if (p->common_row[p->ncommon][0] < 0) goto refused;
            for (j = 0; j < p->ncommon; ++j)
                if (p->common_row[j][0] == p->common_row[p->ncommon][0]) goto refused;
            ++p->ncommon;
        }
    }
    for (i = 0; i < 2; ++i) {
        v = jd_get(d, e, i ? "motion_anims" : "states");
        if (v < 0) continue;
        if (d->n[v].type != JN_ARR) goto refused;
        for (c = d->n[v].first; c >= 0; c = d->n[c].next) {
            int sub = jd_get(d, c, "subaction");
            if (sub >= 0 && d->n[sub].type == JN_NUM &&
                (d->n[sub].num < 0 || d->n[sub].num >= 303 || d->n[sub].num != (int)d->n[sub].num)) goto refused;
        }
    }
    p->defined = 1;
    return 1;
refused:
    gw_log("geno: %s: define refused (requires v6, v7 or v8 native Mario/common v1/retail references; no attach; v7 for special_attributes, fx_bindings and attributes past the first 40; v8 for articles and sounds)", where);
    return -1;
}

/* Build a separate source-independent catalogue, using P5's identity primitives.
 * Unlike GDRSTR01 discovery, no MxDt fingerprint/file/row is consulted.
 * Resident assignment is boot-stable; snapshots never depend on a mutable host map.
 */
static void gn_define_bind(gn_registry* r)
{
    unsigned char occupied[128] = {0};
    int i, slot, ok = 1;
    gdf_free(&r->definitions);
    /* Attach-only registries must not add definition-driven MxDt discovery. */
    for (i = 0; i < r->n && !r->p[i].defined; ++i) {}
    if (i == r->n) return;
    for (slot = 0; slot < GW_MEX_SLOT_COUNT; ++slot)
        if (gw_Mex_SlotInternal(slot) >= 0) occupied[34 + slot] = 1;
    for (i = 0; i < r->n; ++i) if (r->p[i].defined) {
        gn_profile* p = &r->p[i];
        if (!gdf_add(&r->definitions, p->mod, p->define_key, p->name, p->id, i)) {
            gw_log("geno: %s: native catalogue refused duplicate identity/allocation/id exhaustion", p->name);
            ok = 0; break;
        }
    }
    if (ok) ok = gdf_bind(&r->definitions, occupied);
    if (!ok) {
        gdf_free(&r->definitions);
        gw_log("geno: define admission refused: no unoccupied resident aliases or invalid catalogue; definitions inactive");
    }
    for (i = 0; i < r->n; ++i) if (r->p[i].defined) {
        int ck = gdf_alias_for_profile(&r->definitions, i);
        r->p[i].kind = ck >= 34 ? ck - 1 : -1;
    }
}

int gw_Geno_DefineCount(void)
{
    gn_registry* r = gn_reg();
    if (!r->kinds_built) gn_build_kinds(r);
    return (int)r->definitions.catalog.count;
}
int gw_Geno_DefineCKAt(int index)
{
    gn_registry* r = gn_reg();
    if (!r->kinds_built) gn_build_kinds(r);
    return index >= 0 && (unsigned)index < r->definitions.catalog.count ? r->definitions.aliases[index] : -1;
}
/* The declared move tag of a define fighter's common_states row for `motion`, 0 = none. */
int gw_Geno_CommonMoveTag(int p, int motion) {
    int i;
    if (p < 0) return 0;
    for (i = 0; i < gn_reg()->p[p].ncommon; ++i)
        if (gn_reg()->p[p].common_row[i][0] == motion) return gn_reg()->p[p].cs_tag[i];
    return 0;
}
int gw_Geno_DefineBaseKind(int kind)
{
    gn_registry* r = gn_reg();
    int p;
    if (!r->kinds_built) gn_build_kinds(r);
    p = gdf_profile_for_kind(&r->definitions, kind);
    return p >= 0 ? 0 : -1; /* native Mario only in slice 1 */
}
int gw_Geno_DefineBaseCK(int ck) { return gw_Geno_DefineBaseKind(ck-1) >= 0 ? 8 : -1; }
int gw_Geno_DefineName(int ck, char* out, int cap)
{
    gn_registry* r = gn_reg(); int p;
    if (!r->kinds_built) gn_build_kinds(r);
    p = gdf_profile_for_kind(&r->definitions, ck-1);
    if (p < 0 || !out || cap <= 0) return 0;
    snprintf(out, (size_t)cap, "%s", r->p[p].name); return 1;
}
int gw_Geno_DefineResolve(const char* token)
{
    gn_registry* r = gn_reg(); int i;
    if (!r->kinds_built) gn_build_kinds(r);
    for (i = 0; i < r->n; ++i) if (r->p[i].defined && r->p[i].kind >= 0 &&
        (!_stricmp(token, r->p[i].define_key) || !_stricmp(token, r->p[i].name))) return r->p[i].kind+1;
    return -1;
}
int gw_Geno_DefineIdentityWord(int kind, int high)
{
    gn_registry* r = gn_reg(); int p;
    if (!r->kinds_built) gn_build_kinds(r);
    p = gdf_profile_for_kind(&r->definitions, kind);
    return p >= 0 ? (int)(high ? r->p[p].id >> 32 : r->p[p].id) : 0;
}
int gw_Geno_DefineOverrideCount(int kind)
{
    gn_registry* r = gn_reg(); int p;
    if (!r->kinds_built) gn_build_kinds(r);
    p = gdf_profile_for_kind(&r->definitions, kind);
    return p >= 0 ? r->p[p].ncommon : 0;
}
int gw_Geno_DefineOverrideField(int kind, int row, int field)
{
    gn_registry* r = gn_reg(); int p;
    if (!r->kinds_built) gn_build_kinds(r);
    p = gdf_profile_for_kind(&r->definitions, kind);
    return p >= 0 && row >= 0 && row < r->p[p].ncommon && field >= 0 && field < 9 ? r->p[p].common_row[row][field] : -1;
}

/* Read only after proving the object lies inside MEM1. A foreign fighter being
   passed to guest code is conservatively a violation too (e.g. copy callbacks). */
int gw_Geno_DefinePPCProbe(uint32_t argument)
{
    uint32_t fp;
    int kind;
    extern void gw_GenoGame_DefinePPCViolation(int kind);
    if (!gw_mem1 || argument < 0x80000000u || (uint64_t)argument+0x30 > 0x80000000ull+gw_mem1_size) return 0;
    if (gw_r16((const void*)(uintptr_t)argument) == 4) {
        fp = gw_r32((const void*)(uintptr_t)(argument+0x2c));
    } else {
        /* Some native-style adapters pass Fighter* instead of HSD_GObj*. */
        uint32_t gobj = gw_r32((const void*)(uintptr_t)argument);
        if (gobj < 0x80000000u || (uint64_t)gobj+0x30 > 0x80000000ull+gw_mem1_size ||
            gw_r16((const void*)(uintptr_t)gobj) != 4 ||
            gw_r32((const void*)(uintptr_t)(gobj+0x2c)) != argument) return 0;
        fp = argument;
    }
    if (fp < 0x80000000u || (uint64_t)fp+8 > 0x80000000ull+gw_mem1_size) return 0;
    kind = (int)gw_r32((const void*)(uintptr_t)(fp+4));
    if (gw_Geno_DefineBaseKind(kind) < 0) return 0;
    gw_GenoGame_DefinePPCViolation(kind);
    gw_log("geno: ERROR native define kind %d attempted PowerPC execution; callback refused", kind);
    return 1;
}

/* ---- slice 4: a base "none" define's own files and plan -------------------------------------- */
static struct gn_fm* gn_fm_for_kind(int kind)
{
    gn_registry* r = gn_reg(); int p;
    if (!r->kinds_built) gn_build_kinds(r);
    p = gdf_profile_for_kind(&r->definitions, kind);
    return p >= 0 ? r->p[p].fm : NULL;
}
int gw_Geno_DefineIsNone(int kind) { return gn_fm_for_kind(kind) != NULL; }
int gw_Geno_DefineCostumeCount(int kind) { struct gn_fm* fm = gn_fm_for_kind(kind); return fm ? fm->ncostume : 0; }
/* which: 0 animation bank file, 1 plan file, 2 costume model file, 3 its joint symbol, 4 its matanim symbol (NULL = none) */
const char* gw_Geno_DefineString(int kind, int which, int costume)
{
    struct gn_fm* fm = gn_fm_for_kind(kind);
    if (!fm) return NULL;
    if (which == 0) return fm->anim_file;
    if (which == 1) return fm->plan_file;
    if (costume < 0 || costume >= fm->ncostume) return NULL;
    if (which == 2) return fm->cos_file[costume];
    if (which == 3) return fm->cos_joint[costume];
    if (which == 4) return fm->cos_matanim[costume][0] ? fm->cos_matanim[costume] : NULL;
    return NULL;
}

/* ---- slice 6: costumes as declared colour sets, and a define's own menu and HUD art ----------------------------------------------------
 * Everything is by CK (the resident id the menus and the match setup carry; the FighterKind is ck - 1). A donor-based define (base "mario")
 * keeps Mario's costume list; a base "none" define's costumes are its fighter.costumes, with the team colours it declared. */
static int gn_fm_team_costume(const struct gn_fm* fm, int team)
{
    int i;
    if (!fm || fm->ncostume < 1 || team < 0 || team > 2) return -1;
    for (i = 0; i < fm->ncostume; ++i) if (fm->cos_team[i] == team) return i;
    /* undeclared: red is the first costume, blue the second, green the third, each wrapping to the first when there are fewer */
    return team < fm->ncostume ? team : 0;
}
static const char* gn_pres_file(const gn_profile* p, int what, int costume)
{
    if (!p || !p->pres || what < 0 || what >= GN_PRES_KINDS || p->pres->n[what] < 1) return NULL;
    if (what == GN_PRES_ICON || what == GN_PRES_EMBLEM || costume < 0 || costume >= p->pres->n[what]) costume = 0;
    return p->pres->file[what][costume];
}
static const gn_profile* gn_define_profile_ck(int ck)
{
    gn_registry* r = gn_reg(); int p;
    if (!r->kinds_built) gn_build_kinds(r);
    p = gdf_profile_for_kind(&r->definitions, ck - 1);
    return p >= 0 ? &r->p[p] : NULL;
}
/* how many costumes the define declares: 0 for anything that is not a base "none" define (the caller keeps its own count) */
int gw_Geno_DefineCostumeCountCK(int ck)
{
    const gn_profile* p = gn_define_profile_ck(ck);
    return p && p->fm ? p->fm->ncostume : 0;
}
/* the costume a team battle colour (0 red, 1 blue, 2 green) uses, or -1 when the define is not a base "none" one */
int gw_Geno_DefineTeamCostume(int ck, int team)
{
    const gn_profile* p = gn_define_profile_ck(ck);
    return p ? gn_fm_team_costume(p->fm, team) : -1;
}
/* the declared name of a costume ("" when it has none; 0 when the define is not a base "none" one or the costume is out of range) */
int gw_Geno_DefineCostumeName(int ck, int costume, char* out, int cap)
{
    const gn_profile* p = gn_define_profile_ck(ck);
    if (!p || !p->fm || costume < 0 || costume >= p->fm->ncostume || !out || cap <= 0) return 0;
    snprintf(out, (size_t)cap, "%s", p->fm->cos_name[costume]);
    return out[0] != 0;
}
/* 1 when the package declares art of this kind (0 icon, 1 portrait, 2 stock, 3 emblem) */
int gw_Geno_DefineHasArt(int ck, int what, int costume) { return gn_pres_file(gn_define_profile_ck(ck), what, costume) != NULL; }
/* the file's bytes, from the mod's files/ by name: malloc'd, the caller frees; NULL when the define declares none or the file is missing
 * (logged once per file). Host only: the game side gets a gxtex handle or a kit texture, never this pointer. */
static void* gn_pres_read(const gn_profile* p, const char* name, uint32_t* len)
{
    extern void* gw_DVDReadFileAlloc(const char* path, uint32_t* out_size);
    void* raw = gw_DVDReadFileAlloc(name, len);
    if (!raw) {
        static char said[16][64]; static int nsaid; int i;
        for (i = 0; i < nsaid && strcmp(said[i], name); ++i) {}
        if (i == nsaid && nsaid < 16) { snprintf(said[nsaid++], sizeof said[0], "%s", name); gw_log("geno: %s: presentation file %s is not in the mod's files/", p->name, name); }
    }
    return raw;
}
/* A kit texture index for the define's art, decoded into the kit's pool (key "gn:<define key>:<what>:<costume entry>"), or -1: nothing declared,
 * a missing or malformed file, or a full pool. The Atlas select draws a tile with letters then. */
int gw_Geno_DefineArtTex(int ck, int what, int costume)
{
    extern int gw_Kit_TexAddGxtex(const char* key, const uint8_t* blob, size_t len);
    extern int gw_Kit_TexHsdFind(const char* key);
    const gn_profile* p = gn_define_profile_ck(ck);
    const char* file = gn_pres_file(p, what, costume);
    char key[96];
    uint32_t len = 0; void* raw; int tex;
    if (!file) return -1;
    snprintf(key, sizeof key, "gn:%s:%d:%s", p->define_key, what, file);
    tex = gw_Kit_TexHsdFind(key);
    if (tex >= 0) return tex;
    raw = gn_pres_read(p, file, &len);
    if (!raw) return -1;
    tex = gw_Kit_TexAddGxtex(key, (const uint8_t*)raw, len);
    if (tex < 0) gw_log("geno: %s: presentation file %s is not a usable .gxtex", p->name, file);
    else gw_log("geno: %s: %s from its package: %s as kit texture %d", p->name, what == 0 ? "select icon" : what == 1 ? "portrait" : what == 2 ? "stock icon" : "results emblem", file, tex);
    free(raw);
    return tex;
}
/* A gxtex handle (gw_GxTex_*: width, height, format, CopyImage, CopyTlut, Close) for the same art, for game code that builds an HSD_ImageDesc
 * of its own (the HUD stock icon, the results screen). -1 as above. The caller closes the handle. */
int gw_Geno_DefineArtOpen(int ck, int what, int costume)
{
    extern int gw_GxTex_OpenBlob(const unsigned char* blob, uint32_t len, const char* label);
    const gn_profile* p = gn_define_profile_ck(ck);
    const char* file = gn_pres_file(p, what, costume);
    uint32_t len = 0; void* raw; int h;
    if (!file) return -1;
    raw = gn_pres_read(p, file, &len);
    if (!raw) return -1;
    h = gw_GxTex_OpenBlob((const unsigned char*)raw, len, file); /* copies; the blob is ours to free */
    free(raw);
    return h;
}

/* ---- slice 6 (geno 10): AI, Kirby copy, own sound clips ----------------------------------------------------------------------------------- */
/* The retail FighterKind a CPU treats the define (FighterKind `kind`, = CK - 1) as, or -1: not a define, or it declared no "ai.like". */
int gw_Geno_DefineAiLike(int kind)
{
    gn_registry* r = gn_reg(); int p;
    if (kind < 0x21) return -1; /* the retail cast: no registry walk on the AI's hot path */
    if (!r->kinds_built) gn_build_kinds(r);
    p = gdf_profile_for_kind(&r->definitions, kind);
    return p >= 0 && r->p[p].ai_set ? r->p[p].ai_like : -1;
}
/* What Kirby gets from inhaling FighterKind `kind`: -2 = it is not a define (the caller keeps retail behaviour), -1 = nothing (the default and "none"),
 * otherwise the retail FighterKind whose hat and ability Kirby wears. */
int gw_Geno_DefineKirbyCopy(int kind)
{
    gn_registry* r = gn_reg(); int p;
    if (kind < 0x21) return -2;
    if (!r->kinds_built) gn_build_kinds(r);
    p = gdf_profile_for_kind(&r->definitions, kind);
    return p < 0 ? -2 : r->p[p].kb_copy;
}

/* A .gnsnd (tools/geno/audio.py): 32-byte big-endian header, then s16 big-endian samples. Decoded to native s16, or NULL (and *why) when it is not a
 * valid v1 file. Pure: the tests feed it blobs. */
static int16_t* gn_gnsnd_decode(const unsigned char* b, uint32_t len, int* frames, int* channels, const char** why)
{
    uint32_t w[8], i, n;
    int16_t* pcm;
#define GN_BE32(o) ((uint32_t)b[o] << 24 | (uint32_t)b[(o)+1] << 16 | (uint32_t)b[(o)+2] << 8 | b[(o)+3])
    if (len < 32 || memcmp(b, "GNSD", 4)) { *why = "not a .gnsnd (magic)"; return NULL; }
    for (i = 1; i < 8; ++i) w[i] = GN_BE32(i * 4);
    if (w[1] != 1) { *why = "unknown .gnsnd version"; return NULL; }
    if (w[2] != 32000) { *why = "sample rate is not 32000"; return NULL; }
    if (w[3] < 1 || w[3] > 2) { *why = "channels must be 1 or 2"; return NULL; }
    if (w[4] < 1 || w[4] > 320000) { *why = "frames must be 1..320000"; return NULL; }
    if (w[5] != 32 || w[7] != 0 || w[6] != w[4] * w[3] * 2 || len != 32 + w[6]) { *why = "header and length disagree"; return NULL; }
    n = w[4] * w[3];
    pcm = (int16_t*)malloc((size_t)n * 2);
    if (!pcm) { *why = "out of memory"; return NULL; }
    for (i = 0; i < n; ++i) pcm[i] = (int16_t)((uint16_t)b[32 + i * 2] << 8 | b[33 + i * 2]);
    *frames = (int)w[4]; *channels = (int)w[3];
#undef GN_BE32
    return pcm;
}
static const struct gn_clip* gn_clip_load(const gn_profile* p, struct gn_clip* c, const char* file)
{
    extern void* gw_DVDReadFileAlloc(const char* path, uint32_t* out_size);
    uint32_t len = 0; void* raw; const char* why = "";
    if (c->pcm) return c;
    if (c->tried) return NULL;
    c->tried = 1;
    raw = gw_DVDReadFileAlloc(file, &len);
    if (!raw) { gw_log("geno: %s: audio file %s is not in the mod's files/", p->name, file); return NULL; }
    c->pcm = gn_gnsnd_decode((const unsigned char*)raw, len, &c->frames, &c->channels, &why);
    free(raw);
    if (!c->pcm) { gw_log("geno: %s: audio file %s is not usable: %s", p->name, file, why); return NULL; }
    gw_log("geno: %s: audio %s: %d frames, %d channel(s)", p->name, file, c->frames, c->channels);
    return c;
}
/* 1 when the define has a clip for this: which 0 = its announcer call (sfx ignored), 1 = a voice row for retail sound id `sfx` */
int gw_Geno_DefineAudioHas(int ck, int which, int sfx)
{
    const gn_profile* p = gn_define_profile_ck(ck);
    int i;
    if (!p || !p->aud) return 0;
    if (which == 0) return p->aud->announcer[0] != 0;
    for (i = 0; i < p->aud->nvoice; ++i) if (p->aud->voice_sfx[i] == sfx) return 1;
    return 0;
}
/* Start the define's clip (see gw_Geno_DefineAudioHas); 1 when one was started, 0 when there is none (the caller then does what it did before).
 * Never while a rollback session is armed: a define is offline only, and a clip is not rolled back. */
int gw_Geno_DefineAudioPlay(int ck, int which, int sfx)
{
    extern int gw_Audio_PlayClip(const int16_t* pcm, int frames, int channels, int volume);
    extern int gw_rb_active(void);
    const gn_profile* p = gn_define_profile_ck(ck);
    const struct gn_clip* c = NULL;
    int i, vol = 127;
    if (!p || !p->aud) return 0;
    if (which == 0) {
        if (!p->aud->announcer[0]) return 0;
        c = gn_clip_load(p, &p->aud->announcer_clip, p->aud->announcer);
    } else {
        for (i = 0; i < p->aud->nvoice && p->aud->voice_sfx[i] != sfx; ++i) {}
        if (i == p->aud->nvoice) return 0;
        vol = p->aud->voice_vol[i];
        c = gn_clip_load(p, &p->aud->voice_clip[i], p->aud->voice_file[i]);
    }
    if (!c) return 1; /* declared but unusable: stay silent rather than fall back to the donor's voice */
    if (gw_rb_active()) return 1;
    gw_Audio_PlayClip(c->pcm, c->frames, c->channels, vol);
    gw_log("geno: %s: plays its own %s (%s)", p->name, which == 0 ? "announcer call" : "voice line", which == 0 ? p->aud->announcer : p->aud->voice_file[i]);
    return 1;
}
/* the same by FighterKind (the fighter code knows kind, not CK) */
int gw_Geno_DefineVoicePlayKind(int kind, int sfx) { return kind < 0x21 ? 0 : gw_Geno_DefineAudioPlay(kind + 1, 1, sfx); }

static int gn_plan_num(const jdoc* d, int o, const char* key, int lo, int hi)
{
    int v = jd_get(d, o, key);
    if (v < 0 || d->n[v].type != JN_NUM || d->n[v].num < lo || d->n[v].num > hi || d->n[v].num != (int)d->n[v].num) return -1;
    return (int)d->n[v].num;
}
static int gn_plan_parse(GenoPlan* pl, const jdoc* d, const char* who)
{
    int root = 0, parts, a, c, i, bank, clips, rows;
    memset(pl, 0, sizeof *pl);
    pl->joints = gn_plan_num(d, root, "joint_count", 1, GPL_MAX_JOINTS);
    parts = jd_get(d, root, "parts");
    if (pl->joints < 0 || parts < 0) { gw_log("geno: plan %s: no joint_count or parts", who); return 0; }
    memset(pl->joint_to_part, GPL_NONE, sizeof pl->joint_to_part);
    memset(pl->part_to_joint, GPL_NONE, sizeof pl->part_to_joint);
    a = jd_get(d, parts, "joint_to_part");
    if (a < 0 || d->n[a].type != JN_ARR || d->n[a].count != pl->joints) { gw_log("geno: plan %s: joint_to_part is not %d long", who, pl->joints); return 0; }
    for (c = d->n[a].first, i = 0; c >= 0; c = d->n[c].next, ++i) pl->joint_to_part[i] = (unsigned char)d->n[c].num;
    a = jd_get(d, parts, "part_to_joint");
    if (a < 0 || d->n[a].type != JN_ARR || d->n[a].count < 1 || d->n[a].count > GPL_MAX_PARTS) { gw_log("geno: plan %s: part_to_joint size", who); return 0; }
    pl->nparts = d->n[a].count;
    for (c = d->n[a].first, i = 0; c >= 0; c = d->n[c].next, ++i) pl->part_to_joint[i] = (unsigned char)d->n[c].num;
    bank = jd_get(d, root, "bank"); clips = bank >= 0 ? jd_get(d, bank, "clips") : -1;
    if (clips < 0 || d->n[clips].type != JN_OBJ || d->n[clips].count > GPL_MAX_CLIPS) { gw_log("geno: plan %s: no bank.clips (run the converter again)", who); return 0; }
    for (c = d->n[clips].first; c >= 0; c = d->n[c].next) {
        int sym = jd_get(d, c, "symbol"), off = gn_plan_num(d, c, "offset", 0, 0x7fffffff), by = gn_plan_num(d, c, "bytes", 1, 0x7fffffff);
        if (sym < 0 || d->n[sym].type != JN_STR || off < 0 || by < 0 || strlen(d->n[c].key) >= 32 || strlen(d->n[sym].str) >= 72) { gw_log("geno: plan %s: bad clip %s", who, d->n[c].key); return 0; }
        snprintf(pl->clip_name[pl->nclips], 32, "%s", d->n[c].key);
        snprintf(pl->clip_symbol[pl->nclips], 72, "%s", d->n[sym].str);
        pl->clip_offset[pl->nclips] = (unsigned)off; pl->clip_bytes[pl->nclips] = (unsigned)by;
        pl->clip_frames[pl->nclips] = gn_plan_num(d, c, "frames", 1, 100000);
        ++pl->nclips;
    }
    rows = jd_get(d, root, "motion_rows");
    if (rows < 0 || d->n[rows].type != JN_ARR || d->n[rows].count > GPL_MAX_MOTIONS) { gw_log("geno: plan %s: no motion_rows", who); return 0; }
    for (i = 0; i < GPL_MAX_MOTIONS; ++i) pl->motion_clip[i] = -1;
    for (i = 0; i < GPL_MAX_ROWS; ++i) pl->row_clip[i] = -1;
    {   /* row_clips: the clip for each animation row by its ftCo_SM_ name (rows no state names but the engine plays by number) */
        int rc = jd_get(d, root, "row_clips"), e, ri = 0;
        if (rc >= 0 && d->n[rc].type == JN_ARR) {
            for (e = d->n[rc].first; e >= 0 && ri < GPL_MAX_ROWS; e = d->n[e].next, ++ri) {
                if (d->n[e].type == JN_STR) {
                    for (i = 0; i < pl->nclips && strcmp(pl->clip_name[i], d->n[e].str); ++i) {}
                    if (i < pl->nclips) pl->row_clip[ri] = (short)i;
                }
            }
        }
    }
    for (c = d->n[rows].first; c >= 0; c = d->n[c].next) {
        int m = gn_plan_num(d, c, "motion", 0, GPL_MAX_MOTIONS - 1), cl = jd_get(d, c, "clip");
        if (m < 0) { gw_log("geno: plan %s: bad motion row", who); return 0; }
        if (cl >= 0 && d->n[cl].type == JN_STR) {
            for (i = 0; i < pl->nclips && strcmp(pl->clip_name[i], d->n[cl].str); ++i) {}
            if (i == pl->nclips) { gw_log("geno: plan %s: motion %d names clip %s, not in the bank", who, m, d->n[cl].str); return 0; }
            pl->motion_clip[m] = (short)i;
        }
        if (m + 1 > pl->nmotions) pl->nmotions = m + 1;
    }
    {
        static const struct { const char* key; int n; int off; } lists[] = {
            {"x8_part_bytes", 5, 0}, {"x34_centre", 1, 1}, {"x38_coin", 2, 2}, {"x44_ecb", 6, 3}, {"x54_gfx", 5, 4}, {"x58_ik", 10, 5}, {"x30_hurtbox_bones", 10, 6}};
        int fd = jd_get(d, root, "ftdata"), li, hb;
        int* dst[7] = {pl->ft_x8, pl->ft_x34, pl->ft_x38, pl->ft_x44, pl->ft_x54, pl->ft_x58, NULL};
        for (li = 0; li < 6; ++li) {
            int arr = fd >= 0 ? jd_get(d, fd, lists[li].key) : -1, e, k = 0;
            for (k = 0; k < lists[li].n; ++k) dst[li][k] = -1;
            k = 0;
            if (arr < 0 || d->n[arr].type != JN_ARR) { gw_log("geno: plan %s: ftdata.%s missing", who, lists[li].key); return 0; }
            for (e = d->n[arr].first; e >= 0 && k < lists[li].n; e = d->n[e].next) {
                if (d->n[e].type == JN_ARR) { int q; for (q = d->n[e].first; q >= 0 && k < lists[li].n; q = d->n[q].next) dst[li][k++] = gn_plan_num(d, q, "joint", 0, GPL_MAX_JOINTS - 1); }
                else dst[li][k++] = gn_plan_num(d, e, "joint", 0, GPL_MAX_JOINTS - 1);
            }
            for (k = 0; k < lists[li].n; ++k) if (dst[li][k] < 0) { gw_log("geno: plan %s: ftdata.%s entry %d has no joint (unresolved role)", who, lists[li].key, k); return 0; }
        }
        hb = jd_get(d, root, "hurtboxes");
        if (hb < 0 || d->n[hb].type != JN_ARR || d->n[hb].count < 1 || d->n[hb].count > 15) { gw_log("geno: plan %s: hurtboxes must hold 1..15 capsules", who); return 0; }
        for (a = d->n[hb].first; a >= 0; a = d->n[a].next) {
            int n = pl->nhurt++, hv = jd_get(d, a, "height"), gv = jd_get(d, a, "grabbable"), av = jd_get(d, a, "a"), bv = jd_get(d, a, "b"), rv = jd_get(d, a, "radius"), q, k;
            pl->hurt_joint[n] = gn_plan_num(d, a, "joint", 0, GPL_MAX_JOINTS - 1);
            pl->hurt_height[n] = hv >= 0 && !strcmp(d->n[hv].str, "high") ? 2 : hv >= 0 && !strcmp(d->n[hv].str, "mid") ? 1 : 0;
            pl->hurt_grab[n] = gv >= 0 && d->n[gv].type == JN_BOOL ? (int)d->n[gv].num : 1;
            if (pl->hurt_joint[n] < 0 || av < 0 || bv < 0 || rv < 0 || d->n[av].count != 3 || d->n[bv].count != 3) { gw_log("geno: plan %s: hurtbox %d is malformed", who, n); return 0; }
            for (q = d->n[av].first, k = 0; q >= 0 && k < 3; q = d->n[q].next) pl->hurt_a[n][k++] = (float)d->n[q].num;
            for (q = d->n[bv].first, k = 0; q >= 0 && k < 3; q = d->n[q].next) pl->hurt_b[n][k++] = (float)d->n[q].num;
            pl->hurt_radius[n] = (float)d->n[rv].num;
        }
    }
    pl->ok = 1;
    return 1;
}
/* The parsed plan of a base "none" define, NULL when the kind is not one or the plan is unusable (logged). */
static const GenoPlan* gn_define_plan(int kind)
{
    extern void* gw_DVDReadFileAlloc(const char* path, uint32_t* out_size);
    struct gn_fm* fm = gn_fm_for_kind(kind);
    if (!fm) return NULL;
    if (!fm->plan_tried) {
        uint32_t len = 0; char* raw; jdoc* d; GenoPlan* pl;
        fm->plan_tried = 1;
        raw = (char*)gw_DVDReadFileAlloc(fm->plan_file, &len);
        if (!raw) { gw_log("geno: plan file %s not found in the mod's files/", fm->plan_file); return NULL; }
        d = (jdoc*)malloc(sizeof *d); pl = (GenoPlan*)calloc(1, sizeof *pl);
        {
            char* text = (char*)malloc((size_t)len + 1);
            int ok = 0;
            if (d && pl && text) {
                memcpy(text, raw, len); text[len] = 0;
                ok = jd_parse(d, text) >= 0 ? gn_plan_parse(pl, d, fm->plan_file) : (gw_log("geno: plan %s: %s", fm->plan_file, d->err ? d->err : "bad JSON"), 0);
            }
            free(text);
            if (ok) { fm->plan = pl; gw_log("geno: plan %s: %d joints, %d parts, %d clips, %d motion rows", fm->plan_file, pl->joints, pl->nparts, pl->nclips, pl->nmotions); }
            else free(pl);
        }
        free(d); free(raw);
    }
    return fm->plan;
}

/* The game side cannot read a host struct (its loads are big-endian): it gets scalars and byte arrays only.
 * what: 0 ok, 1 joints, 2 nparts, 3 nclips, 4 nmotions; indexed: 10 clip_offset, 11 clip_bytes, 12 clip_frames, 13 motion_clip. */
int gw_Geno_DefinePlanInt(int kind, int what, int i)
{
    const GenoPlan* pl = gn_define_plan(kind);
    if (!pl) return what == 0 ? 0 : -1;
    switch (what) {
    case 0: return pl->ok;
    case 1: return pl->joints;
    case 2: return pl->nparts;
    case 3: return pl->nclips;
    case 4: return pl->nmotions;
    case 10: return i >= 0 && i < pl->nclips ? (int)pl->clip_offset[i] : -1;
    case 11: return i >= 0 && i < pl->nclips ? (int)pl->clip_bytes[i] : -1;
    case 12: return i >= 0 && i < pl->nclips ? pl->clip_frames[i] : -1;
    case 13: return i >= 0 && i < GPL_MAX_MOTIONS ? (int)pl->motion_clip[i] : -1;
    case 14: return i >= 0 && i < GPL_MAX_ROWS ? (int)pl->row_clip[i] : -1;
    case 5: return pl->nhurt;
    case 20: return i >= 0 && i < 5 ? pl->ft_x8[i] : -1;
    case 21: return i == 0 ? pl->ft_x34[0] : -1;
    case 22: return i >= 0 && i < 2 ? pl->ft_x38[i] : -1;
    case 23: return i >= 0 && i < 6 ? pl->ft_x44[i] : -1;
    case 24: return i >= 0 && i < 5 ? pl->ft_x54[i] : -1;
    case 25: return i >= 0 && i < 10 ? pl->ft_x58[i] : -1;
    case 30: return i >= 0 && i < pl->nhurt ? pl->hurt_joint[i] : -1;
    case 31: return i >= 0 && i < pl->nhurt ? pl->hurt_height[i] : -1;
    case 32: return i >= 0 && i < pl->nhurt ? pl->hurt_grab[i] : -1;
    case 40: { /* float bits: hurtbox i/8, component i%8 (0-2 a, 3-5 b, 6 radius) */
        int h = i / 8, c = i % 8; float v; int bits;
        if (i < 0 || h >= pl->nhurt || c > 6) return 0;
        v = c < 3 ? pl->hurt_a[h][c] : c < 6 ? pl->hurt_b[h][c-3] : pl->hurt_radius[h];
        memcpy(&bits, &v, 4); return bits;
    }
    }
    return -1;
}
/* which: 0 clip name, 1 clip symbol (i = clip), 2 joint_to_part bytes, 3 part_to_joint bytes (i unused) */
const char* gw_Geno_DefinePlanText(int kind, int which, int i)
{
    const GenoPlan* pl = gn_define_plan(kind);
    if (!pl) return NULL;
    if (which == 0) return i >= 0 && i < pl->nclips ? pl->clip_name[i] : NULL;
    if (which == 1) return i >= 0 && i < pl->nclips ? pl->clip_symbol[i] : NULL;
    if (which == 2) return (const char*)pl->joint_to_part;
    if (which == 3) return (const char*)pl->part_to_joint;
    return NULL;
}
