/* slice 6: a define's own records.
 *
 * A match's per-fighter numbers go into the SAVE, one row per retail fighter (gm_CKindToSelKind); a define has no row there (every m-ex or define kind maps to one
 * retail row, so writing it would edit another fighter's record: gm_1601.c skips a define). Its numbers go into a file of their own, keyed by the define's stable key
 * (not by the resident CK, which depends on what else is installed):
 *
 *   <card folder>/../geno_records.json   (MELEE_GENO_RECORDS=<file> overrides; with the memory card off and no override nothing is written)
 *   {"format": 1, "records": {"vanilla-courier": {"name": "Vanilla Courier", "matches": 3, "wins": 2, ...}}}
 *
 * The retail save block is never read or written here, so a save with these records is byte-identical to one without. The game side adds a finished match's
 * numbers field by field (gw_Geno_RecordAdd, the gm_80161C90 set) and commits once per fighter. Nothing is simulation state: it happens on the results
 * path, after the match. Define keys not installed right now stay in the file: a record survives a package being away for a while. */
#define GN_REC_FIELDS 12
#define GN_REC_MAXFIELD 11 /* peak_damage: a maximum, the rest are saturating sums */
#define GN_REC_MAX 64
static const char* const gn_rec_names[GN_REC_FIELDS] = {"matches", "wins", "losses", "kos", "falls", "sds", "play_seconds", "damage_dealt", "damage_taken",
                                                         "attacks_hit", "attacks_total", "peak_damage"};
struct gn_rec { char key[40], name[48]; uint32_t v[GN_REC_FIELDS]; };
static struct gn_rec gn_recs[GN_REC_MAX];
static int gn_nrec, gn_rec_loaded;

static struct gn_rec* gn_rec_find(const char* key, int create)
{
    int i;
    for (i = 0; i < gn_nrec; ++i) if (!strcmp(gn_recs[i].key, key)) return &gn_recs[i];
    if (!create || gn_nrec >= GN_REC_MAX) return NULL;
    memset(&gn_recs[gn_nrec], 0, sizeof gn_recs[0]);
    snprintf(gn_recs[gn_nrec].key, sizeof gn_recs[0].key, "%s", key);
    return &gn_recs[gn_nrec++];
}
static void gn_rec_clear(void) { gn_nrec = 0; gn_rec_loaded = 0; }

/* the file's records into the table (a missing file is an empty table; a malformed one is moved aside as <file>.bad so nothing is lost silently) */
static int gn_rec_load_file(const char* path)
{
    FILE* f = fopen(path, "rb");
    long n; char* text; jdoc* d; int ok = 0, recs, c, k;
    gn_nrec = 0;
    if (!f) return 1;
    fseek(f, 0, SEEK_END); n = ftell(f); fseek(f, 0, SEEK_SET);
    if (n < 0 || n > (1 << 20)) { fclose(f); gw_log("geno: records file %s is too large - ignored", path); return 0; }
    text = (char*)malloc((size_t)n + 1); d = (jdoc*)malloc(sizeof *d);
    if (text && d && fread(text, 1, (size_t)n, f) == (size_t)n) {
        text[n] = 0;
        if (jd_parse(d, text) >= 0 && (recs = jd_get(d, 0, "records")) >= 0 && d->n[recs].type == JN_OBJ) {
            for (c = d->n[recs].first; c >= 0; c = d->n[c].next) {
                struct gn_rec* r;
                int nm;
                if (d->n[c].type != JN_OBJ || !(r = gn_rec_find(d->n[c].key, 1))) continue;
                nm = jd_get(d, c, "name");
                if (nm >= 0 && d->n[nm].type == JN_STR) snprintf(r->name, sizeof r->name, "%s", d->n[nm].str);
                for (k = 0; k < GN_REC_FIELDS; ++k) {
                    int v = jd_get(d, c, gn_rec_names[k]);
                    if (v >= 0 && d->n[v].type == JN_NUM && d->n[v].num >= 0 && d->n[v].num <= 4294967295.0) r->v[k] = (uint32_t)d->n[v].num;
                }
            }
            ok = 1;
        }
    }
    fclose(f); free(text); free(d);
    if (!ok) {
        char bad[1100];
        gw_log("geno: records file %s is not a records file - kept as .bad, starting empty", path);
        snprintf(bad, sizeof bad, "%s.bad", path);
        remove(bad); rename(path, bad);
        gn_nrec = 0;
    }
    return ok;
}
static int gn_rec_save_file(const char* path)
{
    char tmp[1100]; FILE* f; int i, k;
    snprintf(tmp, sizeof tmp, "%s.tmp", path);
    f = fopen(tmp, "wb");
    if (!f) { gw_log("geno: cannot write the records file %s", path); return 0; }
    fprintf(f, "{\"format\": 1, \"records\": {\n");
    for (i = 0; i < gn_nrec; ++i) {
        fprintf(f, "  \"%s\": {\"name\": \"%s\"", gn_recs[i].key, gn_recs[i].name);
        for (k = 0; k < GN_REC_FIELDS; ++k) fprintf(f, ", \"%s\": %u", gn_rec_names[k], (unsigned)gn_recs[i].v[k]);
        fprintf(f, "}%s\n", i + 1 < gn_nrec ? "," : "");
    }
    fprintf(f, "}}\n");
    if (fclose(f) != 0) { remove(tmp); return 0; }
    remove(path);
    if (rename(tmp, path) != 0) { gw_log("geno: cannot replace the records file %s", path); return 0; }
    return 1;
}
static void gn_rec_add(struct gn_rec* r, int field, uint32_t value)
{
    if (field < 0 || field >= GN_REC_FIELDS) return;
    if (field == GN_REC_MAXFIELD) { if (value > r->v[field]) r->v[field] = value; return; }
    r->v[field] = (uint64_t)r->v[field] + value > 0xFFFFFFFFull ? 0xFFFFFFFFu : r->v[field] + value;
}
/* where the file lives: MELEE_GENO_RECORDS, else beside the card folder; "" when neither exists (card off) */
static void gn_rec_path(char* out, size_t cap)
{
    extern const char* gw_CardBasePath(void);
    const char* env = getenv("MELEE_GENO_RECORDS");
    const char* base;
    out[0] = 0;
    if (env && env[0]) { snprintf(out, cap, "%s", env); return; }
    base = gw_CardBasePath();
    if (base && base[0]) {
        char tmp[1024]; size_t n; char* s; char* b;
        snprintf(tmp, sizeof tmp, "%s", base);
        n = strlen(tmp);
        while (n > 0 && (tmp[n-1] == '/' || tmp[n-1] == '\\')) tmp[--n] = 0;
        s = strrchr(tmp, '/'); b = strrchr(tmp, '\\');
        if (b && (!s || b > s)) s = b;
        if (s) { s[1] = 0; snprintf(out, cap, "%sgeno_records.json", tmp); }
        else snprintf(out, cap, "geno_records.json");
    }
}
static struct gn_rec* gn_rec_for_ck(int ck, char* path, size_t cap)
{
    const gn_profile* p = gn_define_profile_ck(ck);
    struct gn_rec* r;
    if (!p) return NULL;
    gn_rec_path(path, cap);
    if (!path[0]) return NULL;
    if (!gn_rec_loaded) { gn_rec_load_file(path); gn_rec_loaded = 1; }
    r = gn_rec_find(p->define_key, 1);
    if (r) snprintf(r->name, sizeof r->name, "%s", p->name);
    return r;
}
/* Add one number of a finished match to the define's record (field: see gn_rec_names; value is an unsigned 32-bit count). 1 when it was taken. */
int gw_Geno_RecordAdd(int ck, int field, int value)
{
    char path[1024];
    struct gn_rec* r = gn_rec_for_ck(ck, path, sizeof path);
    if (!r || field < 0 || field >= GN_REC_FIELDS) return 0;
    gn_rec_add(r, field, (uint32_t)value);
    return 1;
}
/* Write the file (once per fighter at the end of a match). 1 when it was written. */
int gw_Geno_RecordCommit(int ck)
{
    char path[1024];
    struct gn_rec* r = gn_rec_for_ck(ck, path, sizeof path);
    int ok;
    if (!r) return 0;
    ok = gn_rec_save_file(path);
    if (ok) gw_log("geno: %s: record written to %s (%u match(es), %u win(s), %u KO(s))", r->name, path, (unsigned)r->v[0], (unsigned)r->v[1], (unsigned)r->v[3]);
    return ok;
}
/* a number of the define's record (-1: not a define, no such field, or no place to keep records) */
int gw_Geno_RecordGet(int ck, int field)
{
    char path[1024]; struct gn_rec* r;
    if (field < 0 || field >= GN_REC_FIELDS) return -1;
    r = gn_rec_for_ck(ck, path, sizeof path);
    return r ? (int)r->v[field] : -1;
}
