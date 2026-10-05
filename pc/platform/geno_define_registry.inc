/* Included after the registry types/parser helpers. Native boot metadata only.
 * Definitions are process-immutable in GF1: changing one requires restarting.
 */
static int gn_define_parse(gn_profile* p, const jdoc* d, int e, int version, const char* where)
{
    static const char* keys[] = {"key", "name", "base", "common", "resources"};
    int def = jd_get(d, e, "define"), c, i, v;
    if (def < 0) return 0;
    /* v7 (slice 2) widens a define: the whole common attribute table, special_attributes, fx_bindings.
     * A v6 file keeps exactly its v6 meaning; articles stay refused until slice 3. */
    if ((version != 6 && version != 7) || jd_get(d, e, "attach") >= 0 || d->n[def].type != JN_OBJ ||
        jd_get(d, e, "articles") >= 0 ||
        (version < 7 && (jd_get(d, e, "fx_bindings") >= 0 || jd_get(d, e, "special_attributes") >= 0))) goto refused;
    for (c = d->n[def].first; c >= 0; c = d->n[c].next) {
        for (i = 0; i < 5 && strcmp(d->n[c].key, keys[i]); ++i) {}
        if (i == 5 || d->n[c].type != JN_STR) goto refused;
    }
    for (i = 0; i < 5; ++i) { v = jd_get(d, def, keys[i]); if (v < 0 || d->n[v].type != JN_STR) goto refused; }
    v = jd_get(d, def, "base"); if (strcmp(d->n[v].str, "mario")) goto refused;
    v = jd_get(d, def, "common"); if (strcmp(d->n[v].str, "melee.common.v1")) goto refused;
    v = jd_get(d, def, "resources"); if (strcmp(d->n[v].str, "retail:mario")) goto refused;
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
    gw_log("geno: %s: define refused (requires v6 or v7 native Mario/common v1/retail references; no attach or articles; v7 for special_attributes, fx_bindings and attributes past the first 40)", where);
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
