/* Drives the shipping gw_settings.c table code (cut out by settings_oracle.sh: ST_MAX .. before the
 * widescreen function, plus gw_Settings_Summary) with a script of operations, one result line per
 * operation. tests/settings_oracle.rs runs the same script through the Rust port.
 *
 * Script lines (fields split on single spaces; hex fields are lowercase hex bytes, "-" = empty):
 *   S key valhex     SetStr          I key n        SetInt
 *   GS key cap dflthex|~   Str read  GI key dflt     Int read
 *   R prefix         RemovePrefix    SUM cap        Summary
 *   ENV name valhex  putenv (empty hex "-" = set to empty)   UNENV name
 *   RAW contenthex   write the cfg file bytes verbatim and reload
 *   RELOAD           forget the table, reread the file
 *   BADPATH / GOODPATH   point saves at an unwritable path / back
 *   FILE             dump the cfg file as hex
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

static void gw_log(const char *fmt, ...) { (void)fmt; }
static char *gw_path_separator(char *p) {
    char *a = strrchr(p, '\\'), *b = strrchr(p, '/');
    return a > b ? a : b;
}

#include "settings_cut.inc"

static int hexval(char c) { return c <= '9' ? c - '0' : c - 'a' + 10; }
static int unhex(const char *h, unsigned char *out) {
    int n = 0;
    if (strcmp(h, "-") == 0) return 0;
    for (; h[0] && h[1]; h += 2) out[n++] = (unsigned char) (hexval(h[0]) * 16 + hexval(h[1]));
    return n;
}
static void phex(const unsigned char *b, int n) {
    int i;
    if (n == 0) printf("-");
    for (i = 0; i < n; ++i) printf("%02x", b[i]);
}
static void dump_file(const char *path) {
    FILE *f = fopen(path, "rb");
    unsigned char buf[1 << 16];
    int n = 0;
    if (f) { n = (int) fread(buf, 1, sizeof buf, f); fclose(f); }
    printf("FILE ");
    phex(buf, n);
    printf("\n");
}

int main(int argc, char **argv) {
    char line[1 << 16], goodpath[MAX_PATH];
    FILE *script;
    if (argc < 3) return 2;
    script = fopen(argv[1], "r");
    snprintf(goodpath, sizeof goodpath, "%s", argv[2]);
    _putenv_s("MELEE_SETTINGS_CFG", goodpath);
    remove(goodpath);
    while (fgets(line, sizeof line, script)) {
        char *f[6] = {0};
        int nf = 0;
        char *p = line;
        unsigned char a[1 << 14], b[1 << 14];
        line[strcspn(line, "\r\n")] = 0;
        while (nf < 6) { f[nf++] = p; p = strchr(p, ' '); if (!p) break; *p++ = 0; }
        if (!strcmp(f[0], "S")) {
            int n = unhex(f[2], a); a[n] = 0;
            gw_Settings_SetStr(f[1], (char *) a);
            printf("S ok=%d\n", st_save_ok);
            dump_file(st_path());
        } else if (!strcmp(f[0], "I")) {
            gw_Settings_SetInt(f[1], atoi(f[2]));
            printf("I ok=%d\n", st_save_ok);
            dump_file(st_path());
        } else if (!strcmp(f[0], "GS")) {
            int cap = atoi(f[2]), found;
            const char *d = NULL;
            if (strcmp(f[3], "~")) { int n = unhex(f[3], b); b[n] = 0; d = (char *) b; }
            memset(a, 0xAA, sizeof a);
            found = gw_Settings_Str(f[1], (char *) a, cap, d);
            printf("GS %d ", found);
            phex(a, cap > 0 ? (int) strlen((char *) a) : 0);
            printf("\n");
        } else if (!strcmp(f[0], "GI")) {
            printf("GI %d\n", gw_Settings_Int(f[1], atoi(f[2])));
        } else if (!strcmp(f[0], "R")) {
            int r = gw_Settings_RemovePrefix(f[1]);
            printf("R %d ok=%d\n", r, st_save_ok);
            dump_file(st_path());
        } else if (!strcmp(f[0], "SUM")) {
            int cap = atoi(f[1]), k;
            memset(a, 0xAA, sizeof a);
            k = gw_Settings_Summary((char *) a, cap);
            printf("SUM %d ", k);
            phex(a, (int) strlen((char *) a));
            printf("\n");
        } else if (!strcmp(f[0], "ENV")) {
            int n = unhex(f[2], a); a[n] = 0;
            _putenv_s(f[1], (char *) a);
            printf("ENV\n");
        } else if (!strcmp(f[0], "UNENV")) {
            _putenv_s(f[1], "");
            printf("UNENV\n");
        } else if (!strcmp(f[0], "RAW")) {
            int n = unhex(f[1], a);
            FILE *o = fopen(goodpath, "wb");
            fwrite(a, 1, n, o);
            fclose(o);
            memset(&st, 0, sizeof st);
            snprintf(st.path, sizeof st.path, "%s", goodpath);
            printf("RAW %d\n", gw_Settings_Int("zzz_probe", -1));
            printf("ROWS %d\n", st.n);
        } else if (!strcmp(f[0], "RELOAD")) {
            memset(&st, 0, sizeof st);
            snprintf(st.path, sizeof st.path, "%s", goodpath);
            gw_Settings_Int("zzz_probe", -1);
            printf("RELOAD rows=%d\n", st.n);
        } else if (!strcmp(f[0], "BADPATH")) {
            snprintf(st.path, sizeof st.path, "%s", "Z:\\no\\such\\dir\\settings.cfg");
            printf("BADPATH\n");
        } else if (!strcmp(f[0], "GOODPATH")) {
            snprintf(st.path, sizeof st.path, "%s", goodpath);
            printf("GOODPATH\n");
        } else if (!strcmp(f[0], "FILE")) {
            dump_file(goodpath);
        }
    }
    return 0;
}
