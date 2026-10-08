/* Slice 7: the registry's answer to "what must two peers agree on for this define" (geno_define_online.h).
 * Included by geno_registry.c after the define registry, so it can see the profile type. */
/* The profile's content id folded to 32 bits (0 when there is no such profile). The game half uses it to name an article item
 * kind by CONTENT in the rollback hash: the numeric kind is profile arithmetic (GENO_ART_KIND), and two installs with different
 * mods number their profiles differently. */
unsigned gw_Geno_ProfileIdWord(int p)
{
    const gn_profile* g = gn_at(p);
    return g ? (unsigned)(g->id ^ (g->id >> 32)) : 0u;
}

int gw_Geno_DefineOnlineInfo(int index, GenoDefineOnline* out)
{
    gn_registry* r = gn_reg(); int ck, p, i, n = 0;
    if (!out) return 0;
    if (!r->kinds_built) gn_build_kinds(r);
    if (index < 0 || (unsigned)index >= r->definitions.catalog.count) return 0;
    ck = r->definitions.aliases[index];
    p = gdf_profile_for_kind(&r->definitions, ck - 1);
    if (p < 0) return 0;
    memset(out, 0, sizeof *out);
    out->ck = ck;
    out->id = r->p[p].id;
    snprintf(out->key, sizeof out->key, "%s", r->p[p].define_key);
    snprintf(out->name, sizeof out->name, "%s", r->p[p].name);
    if (r->p[p].fm) {
        const struct gn_fm* fm = r->p[p].fm;
        out->none = 1;
        snprintf(out->files[n++], sizeof out->files[0], "%s", fm->anim_file);
        snprintf(out->files[n++], sizeof out->files[0], "%s", fm->plan_file);
        for (i = 0; i < fm->ncostume && n < GENO_ONLINE_MAX_FILES; ++i)
            snprintf(out->files[n++], sizeof out->files[0], "%s", fm->cos_file[i]);
    } else {
        snprintf(out->files[n++], sizeof out->files[0], "PlMr.dat");
        snprintf(out->files[n++], sizeof out->files[0], "PlMrAJ.dat");
    }
    out->nfiles = n;
    return 1;
}
