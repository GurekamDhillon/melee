/* Drives the shipping gw_mods.c (the whole file, #included so its statics are reachable) plus the
 * file overlay scan of shim_dvd.c (cut out by mods_oracle.sh) over a scenario script.
 *
 *   mods_oracle <script> <root>
 * Script lines (fields split on single spaces, hex = lowercase hex bytes, "-" = empty):
 *   MD rel               mkdir -p <root>/rel
 *   MK rel hex           write <root>/rel
 *   ENV name hex         putenv (empty hex = remove)
 *   BOOT                 first use of the registry: print every mod and the mount order
 *   SET i on             gw_Mods_SetEnabled
 *   SAVE                 gw_Mods_Save and the file it wrote
 *   RESTART              gw_Mods_RestartNeeded
 *   FP                   overlay mounted (gw_DVDInit), fingerprint, Describe, per-mod digests, overlay table
 *   DIFF peerhex cap     diff_describe(mine, peer, out, cap)
 *   WIN disc             gw_Mods_WinnerOf
 *   ENGINE jsonhex       gw_Mods_JsonEngineMiss
 *   CAT rel              print <root>/rel as hex
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdarg.h>
#include <windows.h>

#include "gw.h"
#include "gw_ui_menus_json.h"

static void gw_log(const char *fmt, ...) {
    char b[4096];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(b, sizeof b, fmt, ap);
    va_end(ap);
    printf("LOG %s\n", b);
}

/* ---- stubs for what gw_mods.c reaches outside itself ---- */
int at_menus_parse(const char *json, const char *mod, AtEntry *out, int cap, char *err, int errcap) {
    (void) json; (void) mod; (void) out; (void) cap;
    if (err && errcap) err[0] = 0;
    return 0;
}
int gw_DVDAltList(int r, int k, char *name, int cap, int *tagged) { (void) r; (void) k; (void) name; (void) cap; (void) tagged; return 0; }
int gw_DVDAltSelect(int r, const char *name) { (void) r; (void) name; return 0; }
int gw_DVDSelectStage(int r, int want) { (void) r; (void) want; return 0; }
int gw_DVDStageFile(int r, int mod_index, int n, char *disc, int dcap, char *host, int hcap) { (void) r; (void) mod_index; (void) n; (void) disc; (void) dcap; (void) host; (void) hcap; return 0; }
int gw_DVDStageIsStub(int r) { (void) r; return 0; }
int gw_DVDStageProviders(int r, int *mods, int cap) { (void) r; (void) mods; (void) cap; return 0; }
int gw_DVDStageToDisc(int r) { (void) r; return 0; }
void gw_Netplay_Leave(void) {}
int gw_Settings_Str(const char *key, char *out, int cap, const char *dflt) { (void) key; if (out && cap > 0) snprintf(out, cap, "%s", dflt ? dflt : ""); return 0; }
void gw_Settings_SetStr(const char *key, const char *value) { (void) key; (void) value; }
int gw_ms_nc_queue(const int *posts, int n) { (void) posts; (void) n; return 0; }
int gw_ms_nc_state(int post, int *pct, char *why, int cap) { (void) post; (void) pct; (void) why; (void) cap; return 0; }
int gw_ms_nc_folder(int post, char *folder, int cap) { (void) post; (void) folder; (void) cap; return 0; }
void gw_ms_nc_pump(void) {}
typedef int (*gw_test_fn)(void);
void gw_test_register(const char *name, gw_test_fn fn) { (void) name; (void) fn; }
void gw_test_fail(const char *fmt, ...) { (void) fmt; }

static void overlay_mount(void);
void gw_DVDInit(void) { overlay_mount(); }

#include "gw_mods.c"

/* ---- the overlay (shim_dvd.c gw_mods_scan .. gw_mod_fingerprint_size) ---- */
static int gw_iso_lookup(const char *path) { (void) path; return -1; }
#include "overlay_cut.inc"

static int overlay_done;
static void overlay_mount(void) {
    int n, k;
    if (overlay_done) return;
    overlay_done = 1;
    n = gw_Mods_ActiveCount();
    if (n == 0) {
        if (gw_Mods_Count() > 0) gw_log("gw: mods: none of the %d installed mods is mounting", gw_Mods_Count());
        return;
    }
    gw_mod_files = (gw_mod_file *) calloc(GW_MOD_MAX_FILES, sizeof *gw_mod_files);
    for (k = 0; k < n; ++k) {
        int idx = gw_Mods_ActiveAt(k);
        int before = gw_mod_count;
        gw_mods_scan(idx, gw_Mods_Id(idx), gw_Mods_PayloadDir(idx), "", gw_Mods_PayloadIsModDir(idx));
        gw_log("gw: mods: %s (%d new paths)", gw_Mods_Id(idx), gw_mod_count - before);
    }
    for (k = 0; k < gw_mod_count; ++k) {
        gw_mod_file *m = &gw_mod_files[k];
        gw_Mods_NoteFile(m->mod_index, m->disc, gw_mod_fingerprint_size(m));
    }
}

static int hexval(char c) { return c <= '9' ? c - '0' : c - 'a' + 10; }
static int unhex(const char *h, unsigned char *out) {
    int n = 0;
    if (strcmp(h, "-") == 0) return 0;
    for (; h[0] && h[1]; h += 2) out[n++] = (unsigned char) (hexval(h[0]) * 16 + hexval(h[1]));
    return n;
}
static void phex(const char *s) {
    if (!s || !*s) { printf("-"); return; }
    for (; *s; ++s) printf("%02x", (unsigned char) *s);
}
static void phexn(const unsigned char *b, int n) {
    int i;
    if (n <= 0) { printf("-"); return; }
    for (i = 0; i < n; ++i) printf("%02x", b[i]);
}
static char root[MAX_PATH];
/* a host path with the scenario root replaced by "<R>" (so the transcript is machine independent) */
static void phexp(const char *s) {
    size_t n = strlen(root);
    if (s && strncmp(s, root, n) == 0) {
        printf("3c523e");
        s += n;
        if (!*s) return;
    }
    if (!s || !*s) { printf("-"); return; }
    for (; *s; ++s) printf("%02x", (unsigned char) *s);
}

/* creates every directory above the last path component */
static void mkparents(const char *full) {
    char tmp[MAX_PATH];
    char *p;
    snprintf(tmp, sizeof tmp, "%s", full);
    for (p = tmp + 3; *p; ++p) {
        if (*p == '/' || *p == '\\') {
            char c = *p;
            *p = 0;
            CreateDirectoryA(tmp, NULL);
            *p = c;
        }
    }
}

int main(int argc, char **argv) {
    FILE *sf;
    static char line[1 << 17];
    static unsigned char buf[1 << 16];
    if (argc < 3) return 2;
    sf = fopen(argv[1], "r");
    snprintf(root, sizeof root, "%s", argv[2]);
    {
        char mods[MAX_PATH];
        snprintf(mods, sizeof mods, "%s/mods", root);
        _putenv_s("MELEE_MODS_DIR", mods);
        _putenv_s("MELEE_MODS", "");
    }
    while (sf && fgets(line, sizeof line, sf)) {
        char *f[4] = { 0, 0, 0, 0 };
        int nf = 0, i;
        char *p = line;
        line[strcspn(line, "\r\n")] = 0;
        while (nf < 4) { f[nf++] = p; p = strchr(p, ' '); if (!p) break; *p++ = 0; }
        if (!strcmp(f[0], "MD")) {
            char full[MAX_PATH];
            snprintf(full, sizeof full, "%s/%s/x", root, f[1]);
            mkparents(full);
        } else if (!strcmp(f[0], "MK")) {
            char full[MAX_PATH];
            FILE *o;
            int n = unhex(f[2], buf);
            snprintf(full, sizeof full, "%s/%s", root, f[1]);
            mkparents(full);
            o = fopen(full, "wb");
            fwrite(buf, 1, n, o);
            fclose(o);
        } else if (!strcmp(f[0], "ENV")) {
            int n = unhex(f[2], buf);
            buf[n] = 0;
            _putenv_s(f[1], (char *) buf);
        } else if (!strcmp(f[0], "BOOT")) {
            int n = gw_Mods_Count(), k;
            printf("BOOT n=%d dir=", n);
            phexp(gw_Mods_Dir());
            printf("\n");
            for (i = 0; i < n; ++i) {
                printf("M %d id=", i); phex(gw_Mods_Id(i));
                printf(" name="); phex(gw_Mods_Name(i));
                printf(" ver="); phex(gw_Mods_Version(i));
                printf(" kind="); phex(gw_Mods_Kind(i));
                printf(" pack="); phex(gw_Mods_Pack(i));
                printf(" desc="); phex(gw_Mods_Description(i));
                printf(" auto="); phex(gw_Mods_Autostart(i));
                printf(" req="); phex(gw_Mods_Requires(i));
                printf(" con="); phex(gw_Mods_Conflicts(i));
                printf(" src="); phex(gw_Mods_Source(i));
                printf(" nuc=%d status=%d", gw_Mods_NucleusPost(i), gw_Mods_Status(i));
                printf(" text="); phex(gw_Mods_StatusText(i));
                printf(" active=%d enabled=%d moddir=%d find=%d payload=", gw_Mods_IsActive(i), gw_Mods_IsEnabled(i), gw_Mods_PayloadIsModDir(i), gw_Mods_Find(gw_Mods_Id(i)));
                phexp(gw_Mods_PayloadDir(i));
                printf("\n");
            }
            printf("ORDER %d:", gw_Mods_ActiveCount());
            for (k = 0; k < gw_Mods_ActiveCount(); ++k) printf(" %d", gw_Mods_ActiveAt(k));
            printf("\n");
            printf("FINDMISS %d\n", gw_Mods_Find("no-such-mod"));
        } else if (!strcmp(f[0], "SET")) {
            int r = gw_Mods_SetEnabled(atoi(f[1]), atoi(f[2]));
            printf("SET %d restart=%d\n", r, gw_Mods_RestartNeeded());
            for (i = 0; i < gw_Mods_Count(); ++i) printf("%d", gw_Mods_IsEnabled(i));
            printf("\n");
        } else if (!strcmp(f[0], "SAVE")) {
            char path[MAX_PATH];
            FILE *o;
            int r = gw_Mods_Save(), n = 0;
            snprintf(path, sizeof path, "%s/mods/enabled.txt", root);
            o = fopen(path, "rb");
            if (o) { n = (int) fread(buf, 1, sizeof buf, o); fclose(o); }
            printf("SAVE %d ", r);
            phexn(buf, n);
            printf("\n");
        } else if (!strcmp(f[0], "RESTART")) {
            printf("RESTART %d\n", gw_Mods_RestartNeeded());
        } else if (!strcmp(f[0], "FP")) {
            uint64_t fp = gw_Mods_Fingerprint();
            const char *d = gw_Mods_Describe();
            printf("FP %016llx\nDESC ", (unsigned long long) fp);
            phex(d);
            printf("\n");
            for (i = 0; i < gw_Mods_Count(); ++i) printf("DIG %d %016llx\n", i, (unsigned long long) gw_Mods_Digest(i));
            for (i = 0; i < gw_mod_count; ++i) {
                const gw_mod_file *m = &gw_mod_files[i];
                int k;
                printf("F ");
                phex(m->disc);
                printf(" mod=%s idx=%d size=%u nprov=%d host=", m->mod, m->mod_index, m->size, m->nprov);
                phexp(m->host);
                for (k = 0; k < m->nprov; ++k) printf(" %s/%u", m->prov[k].mod, m->prov[k].size);
                printf("\n");
            }
        } else if (!strcmp(f[0], "DIFF")) {
            char out[2048];
            int cap = atoi(f[2]);
            int unh;
            memset(buf, 0, sizeof buf);
            unh = unhex(f[1], buf);
            buf[unh] = 0;
            memset(out, 0x55, sizeof out);
            i = gw_Mods_DiffDescribe((char *) buf, out, cap);
            printf("DIFF %d ", i);
            phex(out);
            printf("\n");
        } else if (!strcmp(f[0], "WIN")) {
            char out[128];
            int r = gw_Mods_WinnerOf(f[1], out, sizeof out);
            printf("WIN %d ", r);
            phex(out);
            printf("\n");
        } else if (!strcmp(f[0], "ENGINE")) {
            char out[256];
            int n = unhex(f[1], buf), r;
            buf[n] = 0;
            r = gw_Mods_JsonEngineMiss((char *) buf, out, sizeof out);
            printf("ENGINE %d ", r);
            phex(out);
            printf("\n");
        } else if (!strcmp(f[0], "CAT")) {
            char path[MAX_PATH];
            FILE *o;
            int n = 0;
            snprintf(path, sizeof path, "%s/%s", root, f[1]);
            o = fopen(path, "rb");
            if (o) { n = (int) fread(buf, 1, sizeof buf, o); fclose(o); }
            printf("CAT ");
            phexn(buf, n);
            printf("\n");
        }
    }
    return 0;
}
