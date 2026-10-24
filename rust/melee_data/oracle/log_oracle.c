/* Drives the shipping gw_log.c logger (gl_* cut out by log_oracle.sh, its file/stdout sink replaced by
 * gl_emit) with a script of operations. Output: one "W <hex>" per line written, and after every
 * operation an "ST" line with the noted state. tests/log_oracle.rs runs the same script through
 * the Rust port.
 *   T tick64 frame   ENV name valhex   UNENV name   SLOG hex   ISO hex   NOISO
 *   LOG hex   OSR hex   RAW prefixhex texthex|-   CRASH   GOBJ   DESC   ENVQ name */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <ctype.h>
#include <stdint.h>
#include <windows.h>

static unsigned long long g_tick64;
static unsigned g_frame;
static char g_iso[512];
static int g_has_iso;
static char g_slog[256];
#undef GetTickCount64
#undef GetTickCount
#define GetTickCount64() (g_tick64)
#define GetTickCount() ((DWORD)g_tick64)
#define gw_prof_log_frame() (g_frame)
#define gw_hang_log_time() ((void)0)
#define MAX_PATH_X 260
static char g_sum[1024];
int gw_Settings_Summary(char *out, int cap) { strncpy(out, g_sum, (size_t)cap - 1); out[cap - 1] = 0; return 1; }
static char *gw_path_separator(char *p) {
    char *a = strrchr(p, '\\'), *b = strrchr(p, '/');
    return a > b ? a : b;
}
const char *gw_iso_path(void) { return g_has_iso ? g_iso : NULL; }
int gw_Settings_Str(const char *key, char *out, int cap, const char *dflt) {
    (void)key; (void)dflt;
    strncpy(out, g_slog, (size_t)cap - 1); out[cap - 1] = 0;
    return 1;
}
#include "../../../pc/platform/gw_disc_privacy.h"
static void gl_emit(const char *s);

#include "log_cut.inc"
#include "log_cut2.inc"

static void phex(const char *b) {
    size_t n = strlen(b), i;
    if (n == 0) printf("-");
    for (i = 0; i < n; ++i) printf("%02x", (unsigned char)b[i]);
}
static void gl_emit(const char *s) { printf("W "); phex(s); printf("\n"); }
static int hexval(char c) { return c <= '9' ? c - '0' : c - 'a' + 10; }
static void unhex(const char *h, char *out, size_t cap) {
    size_t n = 0;
    if (strcmp(h, "-") != 0) for (; h[0] && h[1] && n + 1 < cap; h += 2) out[n++] = (char)(hexval(h[0]) * 16 + hexval(h[1]));
    out[n] = 0;
}
static char g_user[64], g_profile[MAX_PATH], g_stamp[16];
static void subst(char *buf, size_t cap) {
    static const char *const tok[3] = {"@USER@", "@PROFILE@", "@STAMP@"};
    char out[16384]; size_t o = 0; const char *s = buf;
    while (*s && o + 1 < sizeof out) {
        int k, hit = 0;
        for (k = 0; k < 3 && !hit; ++k) if (strncmp(s, tok[k], strlen(tok[k])) == 0) {
            const char *v = k == 0 ? g_user : k == 1 ? g_profile : g_stamp;
            while (*v && o + 1 < sizeof out) out[o++] = *v++;
            s += strlen(tok[k]); hit = 1;
        }
        if (!hit) out[o++] = *s++;
    }
    out[o] = 0;
    strncpy(buf, out, cap - 1); buf[cap - 1] = 0;
}
static uint64_t fnv(uint64_t h, const char *p, size_t n) { while (n--) { h ^= (unsigned char)*p++; h *= 1099511628211ull; } return h; }
static void write_file(const char *path, const char *data, int has) {
    if (!has) { remove(path); return; }
    { FILE *f = fopen(path, "wb"); if (f) { fwrite(data, 1, strlen(data), f); fclose(f); } }
}
static void report(const char *kind, const char *reason, const SYSTEMTIME *st) {
    FILE *f; static char rep_[70000]; size_t n, o = 0, i; uint64_t h = 1469598103934665603ull;
    gl_lock();
    gl_write_report("rep.tmp", kind, reason, st);
    gl_unlock();
    f = fopen("rep.tmp", "rb"); n = f ? fread(rep_, 1, sizeof rep_ - 1, f) : 0; if (f) fclose(f);
    rep_[n] = 0;
    for (i = 0; i < n; ) {   /* drop the machine lines: os, build id */
        char *e = strchr(rep_ + i, '\n'); size_t len = e ? (size_t)(e - (rep_ + i)) + 1 : n - i;
        if (strncmp(rep_ + i, "os:  ", 5) != 0 && strncmp(rep_ + i, "build id:", 9) != 0) { h = fnv(h, rep_ + i, len); o += len; }
        i += len;
    }
    printf("REPORT %u %016llx\n", (unsigned)o, (unsigned long long)h);
    if (getenv("ORACLE_DUMP")) { FILE *d = fopen(getenv("ORACLE_DUMP"), "ab"); if (d) { fwrite(rep_, 1, n, d); fputs("\n=====\n", d); fclose(d); } }
}
static void osr(const char *s) {
    /* gw_log_osreportv with the text already formatted */
    char tmp[4096];
    snprintf(tmp, sizeof tmp, "%s", s);
    {
        va_list ap; memset(&ap, 0, sizeof ap);
    }
    gl_lock();
    {
        char buf[2048]; const char *p;
        snprintf(buf, sizeof buf, "%s", tmp);
        for (p = buf; *p != '\0'; ++p) {
            if (*p == '\n') {
                gl.pending[gl.npending] = '\0';
                while (gl.npending > 0 && gl.pending[gl.npending - 1] == '\r') gl.pending[--gl.npending] = '\0';
                if (gl.npending > 0) gl_line(gl.pending);
                gl.npending = 0;
            } else {
                if (gl.npending >= (int)sizeof gl.pending - 1) gl_flush_pending();
                gl.pending[gl.npending++] = *p;
            }
        }
    }
    gl_unlock();
}

int main(int argc, char **argv) {
    static char line[65536];
    FILE *f;
    (void)argc;
    { char *d = argv[2]; if (d) SetCurrentDirectoryA(d); }
    { DWORD n = sizeof g_user; if (!GetUserNameA(g_user, &n)) g_user[0] = 0; }
    _putenv_s("USERPROFILE", "C:\\Users\\Testy"); strcpy(g_profile, "C:\\Users\\Testy");
    { char names[64][64]; int k = 0, i; LPCH blk = GetEnvironmentStringsA(), v;
      for (v = blk; *v; v += strlen(v) + 1) if (strncmp(v, "MELEE_", 6) == 0 && k < 64) { strncpy(names[k], v, 63); names[k][63] = 0; *strchr(names[k] + 1, '=') = 0; ++k; }
      for (i = 0; i < k; ++i) _putenv_s(names[i], ""); FreeEnvironmentStringsA(blk); }
    { char dir[MAX_PATH]; unsigned long st; (void)st; gl_exe_dir(dir, sizeof dir); snprintf(g_stamp, sizeof g_stamp, "%08lx", (unsigned long)gl_link_stamp()); }
    f = fopen(argv[1], "r");
    while (fgets(line, sizeof line, f)) {
        char *t[4] = {0}; int nt = 0; char *p = line, buf[8192], buf2[8192];
        size_t L = strlen(line);
        while (L && (line[L - 1] == '\n' || line[L - 1] == '\r')) line[--L] = 0;
        while (nt < 4) { t[nt++] = p; p = strchr(p, ' '); if (!p) break; *p++ = 0; }
        if (!strcmp(t[0], "T")) { g_tick64 = strtoull(t[1], 0, 10); g_frame = (unsigned)strtoul(t[2], 0, 10); }
        else if (!strcmp(t[0], "ENV")) { unhex(t[2], buf, sizeof buf); _putenv_s(t[1], buf); }
        else if (!strcmp(t[0], "UNENV")) _putenv_s(t[1], "");
        else if (!strcmp(t[0], "SLOG")) unhex(t[1], g_slog, sizeof g_slog);
        else if (!strcmp(t[0], "ISO")) { unhex(t[1], g_iso, sizeof g_iso); subst(g_iso, sizeof g_iso); g_has_iso = 1; }
        else if (!strcmp(t[0], "SUMM")) unhex(t[1], g_sum, sizeof g_sum);
        else if (!strcmp(t[0], "DISCFILE")) { unhex(t[1], buf, sizeof buf); write_file("disc.bin", buf, strcmp(t[1], "~") != 0); }
        else if (!strcmp(t[0], "VERSION") || !strcmp(t[0], "MAP")) {
            char dir[MAX_PATH], path[MAX_PATH]; static char big[40000];
            unhex(t[1], big, sizeof big); subst(big, sizeof big);
            gl_exe_dir(dir, sizeof dir); snprintf(path, sizeof path, "%s%s", dir, t[0][0] == 'V' ? "version.txt" : "melee-pc.map");
            write_file(path, big, strcmp(t[1], "~") != 0);
        }
        else if (!strcmp(t[0], "REPORT")) {
            SYSTEMTIME st; memset(&st, 0, sizeof st);
            sscanf(t[3] ? t[3] : "", "%hu/%hu/%hu/%hu/%hu/%hu", &st.wYear, &st.wMonth, &st.wDay, &st.wHour, &st.wMinute, &st.wSecond);
            unhex(t[1], buf, sizeof buf); unhex(t[2], buf2, sizeof buf2); subst(buf2, sizeof buf2);
            report(buf, strcmp(t[2], "~") ? buf2 : NULL, &st);
        }
        else if (!strcmp(t[0], "NOISO")) g_has_iso = 0;
        else if (!strcmp(t[0], "LOG")) { unhex(t[1], buf, sizeof buf); subst(buf, sizeof buf); gw_log("%s", buf); }
        else if (!strcmp(t[0], "OSR")) { unhex(t[1], buf, sizeof buf); subst(buf, sizeof buf); osr(buf); }
        else if (!strcmp(t[0], "RAW")) { unhex(t[1], buf, sizeof buf); unhex(t[2], buf2, sizeof buf2); gw_log_raw(buf, strcmp(t[2], "~") ? buf2 : NULL); }
        else if (!strcmp(t[0], "CRASH")) gw_log_crash_begin();
        else if (!strcmp(t[0], "GOBJ")) printf("GOBJ %d\n", gw_GObjLogEnabled());
        else if (!strcmp(t[0], "DESC")) { gl_lock(); if (gl.state == 0) gl_configure(); gl_describe(buf, sizeof buf); gl_unlock(); printf("DESC "); phex(buf); printf(" "); phex(gl.spec); printf("\n"); }
        else if (!strcmp(t[0], "RESET")) memset(&gl, 0, sizeof gl);
        else if (!strcmp(t[0], "ENVQ")) { const char *v = getenv(t[1]); printf("ENVQ %s ", v ? "1" : "0"); phex(v ? v : ""); printf("\n"); }
        printf("ST "); phex(gl.disc_path); printf(" "); phex(gl.disc_id); printf(" "); phex(gl.mods);
        printf(" %d %d %u %u %u %u\n", gl.shutting_down, gl.crashing, gl.ntail, gl.nscene, gl.npad, gl.nwatch);
    }
    return 0;
}
