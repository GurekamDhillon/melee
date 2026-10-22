/* C side of the shard-497-504 state oracle. Built normally (not under -nostdinc).
   Reads a scenario file, builds the synthetic state, calls the game's C function (through the
   generated tu_part.c) and prints a trace: every stubbed callee call, the return value and a
   field-by-field dump of the objects the scenario names. tests/s497_rt/mod.rs does the same for
   the Rust port; the two outputs must be byte-identical. */
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "rt.h"

#define ARENA_SIZE (512 * 1024)
#define MAXRET 1024

typedef struct { const char* name; void* p; } FEnt;
typedef struct { const char* name; void* p; unsigned long long size; int fill; } GEnt;
extern const FEnt s497_ftab[];
extern const GEnt s497_gtab[];
extern const int s497_nf, s497_ng;
extern int s497_dispatch(const char* name, const unsigned long long* a, int na, unsigned long long* ret);
extern int s497_dump(const char* type, const unsigned char* base, int count);
extern int s497_bffill(const char* type, unsigned char* base, int count, unsigned long long seed);

static unsigned char* arena;
static unsigned long long rng_state;
static unsigned long long ncalls;

static void put(const char* fmt, ...)
{
    char tmp[8192];
    va_list ap;
    int n;
    va_start(ap, fmt);
    n = vsnprintf(tmp, sizeof tmp, fmt, ap);
    va_end(ap);
    fwrite(tmp, 1, (size_t) n, stdout);
}

unsigned long long rt_rng(void)
{
    unsigned long long x = rng_state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    rng_state = x;
    return x * 0x2545F4914F6CDD1DULL;
}
void* rt_arena(void) { return arena; }
unsigned long long rt_ncalls(void) { return ncalls; }

static void fill(unsigned char* p, unsigned long long n, unsigned long long seed)
{
    unsigned long long save = rng_state, i;
    rng_state = seed * 0x9E3779B97F4A7C15ULL + 1;
    if (rng_state == 0) rng_state = 1;
    for (i = 0; i + 8 <= n; i += 8) {
        unsigned long long v = rt_rng();
        memcpy(p + i, &v, 8);
    }
    if (i < n) {
        unsigned long long v = rt_rng();
        memcpy(p + i, &v, (size_t) (n - i));
    }
    rng_state = save;
}
static unsigned long long hash_name(const char* s)
{
    unsigned long long h = 0xCBF29CE484222325ULL;
    while (*s) { h ^= (unsigned char) *s++; h *= 0x100000001B3ULL; }
    return h;
}
static unsigned long long fnv(const unsigned char* p, unsigned long long n)
{
    unsigned long long h = 0xCBF29CE484222325ULL, i;
    for (i = 0; i < n; i++) { h ^= p[i]; h *= 0x100000001B3ULL; }
    return h;
}

/* ---- pointer normalisation */
static void norm(char* out, unsigned long long v)
{
    int i;
    unsigned long long base = (unsigned long long) arena;
    if (v == 0) { strcpy(out, "N"); return; }
    if (v >= base && v < base + ARENA_SIZE) { sprintf(out, "P:%llx", v - base); return; }
    for (i = 0; i < s497_nf; i++)
        if ((unsigned long long) s497_ftab[i].p == v) { sprintf(out, "F:%s", s497_ftab[i].name); return; }
    for (i = 0; i < s497_ng; i++) {
        unsigned long long g = (unsigned long long) s497_gtab[i].p;
        if (v >= g && v < g + s497_gtab[i].size) { sprintf(out, "G:%s+%llx", s497_gtab[i].name, v - g); return; }
    }
    sprintf(out, "X:%llx", v);
}

/* ---- scripted returns */
typedef struct { char name[64]; int n, next; unsigned long long v[MAXRET]; } Ret;
static Ret rets[128];
static int nrets;
static int next_scripted(const char* name, unsigned long long* v)
{
    int i;
    for (i = 0; i < nrets; i++) {
        Ret* r = &rets[i];
        if (!strcmp(r->name, name)) {
            if (r->n == 0) return 0;
            *v = r->v[r->next];
            r->next = (r->next + 1) % r->n;
            return 1;
        }
    }
    return 0;
}
unsigned long long ret_i(const char* name, unsigned bits)
{
    unsigned long long v;
    if (next_scripted(name, &v)) return v;
    v = rt_rng();
    return bits <= 8 ? (v & 1) : (v >> 33) % 5;
}
unsigned ret_f(const char* name)
{
    unsigned long long v;
    float f;
    unsigned b;
    if (next_scripted(name, &v)) return (unsigned) v;
    f = (float) (int) ((rt_rng() >> 33) % 2000) / 16.0f - 40.0f;
    memcpy(&b, &f, 4);
    return b;
}
unsigned long long ret_d(const char* name)
{
    unsigned long long v;
    double f;
    if (next_scripted(name, &v)) return v;
    f = (double) (int) ((rt_rng() >> 33) % 2000) / 16.0 - 40.0;
    memcpy(&v, &f, 8);
    return v;
}
void* ret_p(const char* name)
{
    unsigned long long v;
    if (next_scripted(name, &v)) return (void*) v;
    return 0;
}
void rt_fill_out(void* p, unsigned long long n)
{
    unsigned long long i;
    for (i = 0; i < n; i++) ((unsigned char*) p)[i] = (unsigned char) (rt_rng() >> 24);
}

/* ---- trace */
void tr_begin(const char* name) { ncalls++; put("L %s", name); }
void tr_i(unsigned long long v) { put(" I:%llx", v); }
void tr_f(unsigned bits) { put(" F:%08x", bits); }
void tr_d(unsigned long long bits) { put(" D:%016llx", bits); }
void tr_p(const void* p) { char b[160]; norm(b, (unsigned long long) p); put(" %s", b); }
void tr_s(const char* p)
{
    unsigned long long v = (unsigned long long) p, base = (unsigned long long) arena;
    int i;
    if (v == 0) { put(" N"); return; }
    if (v >= base && v < base + ARENA_SIZE) { put(" P:%llx", v - base); return; }
    for (i = 0; i < s497_ng; i++) {
        unsigned long long g = (unsigned long long) s497_gtab[i].p;
        if (v >= g && v < g + s497_gtab[i].size) { put(" G:%s+%llx", s497_gtab[i].name, v - g); return; }
    }
    put(" S:\"");
    for (i = 0; i < 80 && p[i]; i++) {
        unsigned char c = (unsigned char) p[i];
        if (c >= 0x20 && c < 0x7f && c != '"' && c != '\\') put("%c", c); else put("\\x%02x", c);
    }
    put("\"");
}
void tr_m(const void* p, unsigned long long n) { put(" M:%llx", fnv((const unsigned char*) p, n)); }
void tr_o(void) { put(" O"); }
void tr_end(void) { put("\n"); }

/* ---- dumps */
void d_i(const char* path, unsigned long long v) { put("  %s=%llx\n", path, v); }
void d_f(const char* path, unsigned bits) { put("  %s=f%08x\n", path, bits); }
void d_d(const char* path, unsigned long long bits) { put("  %s=d%016llx\n", path, bits); }
void d_p(const char* path, unsigned long long v) { char b[160]; norm(b, v); put("  %s=%s\n", path, b); }
void d_blob(const char* path, const void* p, unsigned long long n) { put("  %s=#%llu:%llx\n", path, n, fnv((const unsigned char*) p, n)); }

/* ---- scenario interpreter */
static unsigned long long hex(const char* s) { return strtoull(s, 0, 16); }
static const GEnt* gfind(const char* name)
{
    int i;
    for (i = 0; i < s497_ng; i++) if (!strcmp(s497_gtab[i].name, name)) return &s497_gtab[i];
    fprintf(stderr, "no global %s\n", name);
    exit(2);
}
static void* ffind(const char* name)
{
    int i;
    for (i = 0; i < s497_nf; i++) if (!strcmp(s497_ftab[i].name, name)) return s497_ftab[i].p;
    fprintf(stderr, "no function %s\n", name);
    exit(2);
}
static void store(void* where, int n, unsigned long long v) { memcpy(where, &v, (size_t) n); }
static unsigned long long arena_ptr(unsigned long long off) { return off == 0xFFFF ? 0 : (unsigned long long) arena + off; }
/* tok: s:hex | a:off | n | f:name | g:name[+off] */
static unsigned long long tok_val(const char* t)
{
    if (t[0] == 's' && t[1] == ':') return hex(t + 2);
    if (t[0] == 'a' && t[1] == ':') return (unsigned long long) arena + hex(t + 2);
    if (t[0] == 'n' && !t[1]) return 0;
    if (t[0] == 'f' && t[1] == ':') return (unsigned long long) ffind(t + 2);
    if (t[0] == 'g' && t[1] == ':') {
        char nm[128];
        const char* plus = strchr(t + 2, '+');
        unsigned long long off = 0;
        if (plus) { memcpy(nm, t + 2, (size_t) (plus - t - 2)); nm[plus - t - 2] = 0; off = hex(plus + 1); }
        else strcpy(nm, t + 2);
        return (unsigned long long) gfind(nm)->p + off;
    }
    fprintf(stderr, "bad token %s\n", t);
    exit(2);
}

static void run_block(char** lines, int nl, const char* fn, unsigned long long seed)
{
    int i, k;
    nrets = 0;
    ncalls = 0;
    memset(arena, 0, ARENA_SIZE);
    fill(arena, ARENA_SIZE, seed);
    for (i = 0; i < s497_ng; i++)
        if (s497_gtab[i].fill)
            fill((unsigned char*) s497_gtab[i].p, s497_gtab[i].size, seed ^ hash_name(s497_gtab[i].name));
    rng_state = seed ^ 0xD1B54A32D192ED03ULL;
    if (!rng_state) rng_state = 1;
    put("== %s %016llx\n", fn, seed);
    for (k = 0; k < nl; k++) {
        char buf[4096], *tk[96];
        int n = 0;
        char* p;
        strncpy(buf, lines[k], sizeof buf - 1);
        buf[sizeof buf - 1] = 0;
        for (p = strtok(buf, " \t\r\n"); p && n < 96; p = strtok(0, " \t\r\n")) tk[n++] = p;
        if (!n) continue;
        if (!strcmp(tk[0], "W")) store(arena + hex(tk[1]), (int) hex(tk[2]), hex(tk[3]));
        else if (!strcmp(tk[0], "P")) store(arena + hex(tk[1]), 8, arena_ptr(hex(tk[2])));
        else if (!strcmp(tk[0], "PF")) store(arena + hex(tk[1]), 8, (unsigned long long) ffind(tk[2]));
        else if (!strcmp(tk[0], "GW")) store((unsigned char*) gfind(tk[1])->p + hex(tk[2]), (int) hex(tk[3]), hex(tk[4]));
        else if (!strcmp(tk[0], "GP")) store((unsigned char*) gfind(tk[1])->p + hex(tk[2]), 8, arena_ptr(hex(tk[3])));
        else if (!strcmp(tk[0], "GPF")) store((unsigned char*) gfind(tk[1])->p + hex(tk[2]), 8, (unsigned long long) ffind(tk[3]));
        else if (!strcmp(tk[0], "GSET")) store(gfind(tk[1])->p, 8, arena_ptr(hex(tk[2])));
        else if (!strcmp(tk[0], "RET")) {
            Ret* r = &rets[nrets++];
            int j;
            memset(r, 0, sizeof *r);
            strncpy(r->name, tk[1], 63);
            for (j = 2; j < n && r->n < MAXRET; j++) r->v[r->n++] = tok_val(tk[j]);
        } else if (!strcmp(tk[0], "call") || !strcmp(tk[0], "callf")) {
            unsigned long long a[32], r = 0;
            int j, rc, first = !strcmp(tk[0], "callf") ? 2 : 1;
            const char* target = first == 2 ? tk[1] : fn;
            for (j = first; j < n; j++) a[j - first] = tok_val(tk[j]);
            if (first == 2) put("C %s\n", target);
            rc = s497_dispatch(target, a, n - first, &r);
            if (rc < 0) { fprintf(stderr, "dispatch: unknown function %s\n", fn); exit(2); }
            if (rc == 0) put("ret -\n");
            else if (rc == 2) { char b[160]; norm(b, r); put("ret %s\n", b); }
            else put("ret I:%llx\n", r);
        } else if (!strcmp(tk[0], "dump")) {
            unsigned long long base;
            int cnt = (int) hex(tk[3]);
            put("D %s %s %s\n", tk[1], tk[2], tk[3]);
            base = tok_val(tk[1]);
            if (s497_dump(tk[2], (const unsigned char*) base, cnt) < 0) { fprintf(stderr, "no dumper %s\n", tk[2]); exit(2); }
        } else if (!strcmp(tk[0], "bf")) {
            if (s497_bffill(tk[2], (unsigned char*) tok_val(tk[1]), (int) hex(tk[3]), hex(tk[4])) < 0) { fprintf(stderr, "no bffill %s\n", tk[2]); exit(2); }
        } else if (!strcmp(tk[0], "PW")) {
            /* PW off stride count target: store an arena pointer count times */
            unsigned long long o = hex(tk[1]), st = hex(tk[2]), cnt = hex(tk[3]), j;
            for (j = 0; j < cnt; j++) store(arena + o + j * st, 8, arena_ptr(hex(tk[4])));
        } else if (!strcmp(tk[0], "hashn")) {
            /* hash of arena bytes with arena / function pointers normalised */
            unsigned long long o = hex(tk[1]), len = hex(tk[2]), j, h = 0xCBF29CE484222325ULL;
            unsigned long long base = (unsigned long long) arena;
            for (j = 0; j + 8 <= len; j += 8) {
                unsigned long long wv, q;
                int fi;
                memcpy(&wv, arena + o + j, 8);
                if (wv >= base && wv < base + ARENA_SIZE) wv = 0x5EED000000000000ULL | (wv - base);
                else for (fi = 0; fi < s497_nf; fi++) if ((unsigned long long) s497_ftab[fi].p == wv) { wv = 0xF00D000000000000ULL | (unsigned long long) fi; break; }
                for (q = 0; q < 8; q++) { h ^= (wv >> (8 * q)) & 0xFF; h *= 0x100000001B3ULL; }
            }
            put("N %s %s %llx\n", tk[1], tk[2], h);
        } else if (!strcmp(tk[0], "hash")) {
            put("H %s %s %llx\n", tk[1], tk[2], fnv(arena + hex(tk[1]), hex(tk[2])));
        } else if (!strcmp(tk[0], "pdump")) {
            const GEnt* g = gfind(tk[1]);
            char b[160];
            norm(b, *(unsigned long long*) g->p);
            put("P %s %s\n", tk[1], b);
        } else if (!strcmp(tk[0], "padump")) {
            const GEnt* g = gfind(tk[1]);
            size_t pi;
            for (pi = 0; pi < g->size / 8; pi++) {
                char b[160];
                norm(b, ((unsigned long long*) g->p)[pi]);
                put("P %s[%d] %s\n", tk[1], (int) pi, b);
            }
        } else if (!strcmp(tk[0], "gdump")) {
            const GEnt* g = gfind(tk[1]);
            put("G %s %llx\n", tk[1], fnv((const unsigned char*) g->p, g->size));
        } else { fprintf(stderr, "bad op %s\n", tk[0]); exit(2); }
    }
}

int main(int argc, char** argv)
{
    FILE* f;
    static char line[4096];
    static char* lines[8192];
    int nl = 0, in = 0;
    char fn[128] = "";
    unsigned long long seed = 0;
    if (argc < 2) { fprintf(stderr, "usage: oracle scenario.txt\n"); return 2; }
    arena = (unsigned char*) malloc(ARENA_SIZE);
    f = fopen(argv[1], "r");
    if (!f) { perror("scenario"); return 2; }
    while (fgets(line, sizeof line, f)) {
        if (!strncmp(line, "begin ", 6)) {
            sscanf(line + 6, "%127s %llx", fn, &seed);
            in = 1; nl = 0;
        } else if (!strncmp(line, "end", 3) && in) {
            run_block(lines, nl, fn, seed);
            while (nl > 0) free(lines[--nl]);
            in = 0;
        } else if (in) {
            lines[nl++] = strdup(line);
        }
    }
    fclose(f);
    return 0;
}
