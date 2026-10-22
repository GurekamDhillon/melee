/* TU state-oracle runtime, C side (plain C, system headers). See gen_tuo.py.
 *
 * A "case" = one call of one game function on a synthetic game state. The state is built from a
 * seeded generator (splitmix64) by visitor functions that walk C types leaf by leaf; the Rust
 * test (tests/tuo_common/mod.rs) walks the same leaves in the same order, so both sides see the
 * same LOGICAL state even though bitfields are laid out differently. Every callee that is not in
 * the TU is a stub that logs its arguments and returns generated values. After the call the
 * resulting state (globals, argument objects, pool objects) is dumped leaf by leaf. Each case
 * prints  <fn> <idx> <fnv64 of the whole log>  and, with --full, the log itself.
 *
 * Config (cfg file, shared with the Rust side):
 *   dom <key> <v>...        value domain for leaves whose path ends with <key> (v: int, or f:<hexbits>)
 *   link <key> <target>     pointer leaves whose path ends with <key>: null | &<global> |
 *                           new:<Type>[:<count>] | maybe:<Type> | buf:<bytes>
 *   fn <name> <ncases> <ret> <arg>...
 *       ret: v | i | u | f32 | f64 | p ;  arg: i<bits><s|u> | f32 | f64 | p:<Type> | pn:<Type>
 *       | buf:<bytes> | null
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <stdint.h>
#include <malloc.h>

typedef unsigned long long u64;
typedef void (*VisFn)(const char* path, void* p);

/* ---- generator ---- */
static u64 rs;
u64 tuo_draw(void)
{
    u64 z = (rs += 0x9E3779B97F4A7C15ULL);
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
    return z ^ (z >> 31);
}

/* ---- log ---- */
static u64 lh;
static int full;
static char line[16384];
static void hashline(const char* s)
{
    for (; *s; s++) {
        lh ^= (unsigned char) *s;
        lh *= 0x100000001B3ULL;
    }
    lh ^= '\n';
    lh *= 0x100000001B3ULL;
    if (full) puts(line);
}
void tuo_emit(const char* fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(line, sizeof line, fmt, ap);
    va_end(ap);
    hashline(line);
}

/* ---- config ---- */
#define MAXV 512
struct Dom { char key[128]; int n; u64 v[MAXV]; };
struct Link { char key[128]; char tgt[96]; int ngate; char gate[16][96]; int nforce; char force[16][96]; };
static const char* curfn;
static struct Dom doms[512]; static int ndom;
static struct Link links[512]; static int nlink;
struct Fn { char name[96]; int n; char ret[16]; int nargs; char args[12][64]; };
static struct Fn fns[512]; static int nfn;

static u64 parse_u(const char* s)
{
    if (s[0] == 'f' && s[1] == ':') return strtoull(s + 2, 0, 16);
    if (s[0] == '-') return (u64) strtoll(s, 0, 0);
    return strtoull(s, 0, 0);
}

static void load_cfg(const char* path)
{
    FILE* f = fopen(path, "r");
    if (!f) { fprintf(stderr, "no cfg %s\n", path); exit(2); }
    static char b[262144];
    while (fgets(b, sizeof b, f)) {
        char* tok[1024]; int nt = 0;
        for (char* p = strtok(b, " \t\r\n"); p && nt < 1024; p = strtok(0, " \t\r\n"))
            tok[nt++] = p;
        if (!nt) continue;
        if (!strcmp(tok[0], "dom")) {
            struct Dom* d = &doms[ndom++];
            strcpy(d->key, tok[1]);
            d->n = nt - 2;
            for (int i = 0; i < d->n; i++) d->v[i] = parse_u(tok[2 + i]);
        } else if (!strcmp(tok[0], "link")) {
            strcpy(links[nlink].key, tok[1]);
            strcpy(links[nlink].tgt, tok[2]);
            nlink++;
        } else if (!strcmp(tok[0], "gate")) {
            for (int li = 0; li < nlink; li++)
                if (!strcmp(links[li].key, tok[1]))
                    for (int k = 2; k < nt; k++) strcpy(links[li].gate[links[li].ngate++], tok[k]);
        } else if (!strcmp(tok[0], "force")) {
            for (int li = 0; li < nlink; li++)
                if (!strcmp(links[li].key, tok[1]))
                    for (int k = 2; k < nt; k++) strcpy(links[li].force[links[li].nforce++], tok[k]);
        } else if (!strcmp(tok[0], "fn")) {
            struct Fn* g = &fns[nfn++];
            strcpy(g->name, tok[1]);
            g->n = atoi(tok[2]);
            strcpy(g->ret, tok[3]);
            g->nargs = nt - 4;
            for (int i = 0; i < g->nargs; i++) strcpy(g->args[i], tok[4 + i]);
        }
    }
    fclose(f);
}

static int endswith(const char* s, const char* k)
{
    size_t a = strlen(s), b = strlen(k);
    return a >= b && !strcmp(s + a - b, k);
}
static struct Dom* finddom(const char* path)
{
    for (int i = 0; i < ndom; i++)
        if (endswith(path, doms[i].key)) return &doms[i];
    return 0;
}
static struct Link* findlink(const char* path)
{
    for (int i = 0; i < nlink; i++)
        if (endswith(path, links[i].key)) return &links[i];
    return 0;
}

/* ---- registry ---- */
struct Reg { char name[96]; uintptr_t base; size_t size; };
static struct Reg regs[16384]; static int nreg; static int nperm;
struct Fp { char name[96]; uintptr_t a; };
static struct Fp fps[4096]; static int nfp;

void tuo_regfn(const char* name, void* a)
{
    strcpy(fps[nfp].name, name);
    fps[nfp++].a = (uintptr_t) a;
}
static void reg(const char* name, void* p, size_t n)
{
    strcpy(regs[nreg].name, name);
    regs[nreg].base = (uintptr_t) p;
    regs[nreg].size = n ? n : 1;
    nreg++;
}

static const char* pstr(const void* p, char* out)
{
    uintptr_t a = (uintptr_t) p;
    if (!a) return "null";
    for (int i = nreg - 1; i >= 0; i--)
        if (a >= regs[i].base && a < regs[i].base + regs[i].size) {
            if (a == regs[i].base) sprintf(out, "%s", regs[i].name);
            else sprintf(out, "%s+%llx", regs[i].name, (u64) (a - regs[i].base));
            return out;
        }
    for (int i = 0; i < nfp; i++)
        if (fps[i].a == a) { sprintf(out, "fn:%s", fps[i].name); return out; }
    return "X";
}

/* ---- types, globals ---- */
struct Ty { char name[96]; size_t size, align; VisFn vis; };
static struct Ty tys[2048]; static int nty;
void tuo_regtype(const char* name, size_t size, size_t align, VisFn vis)
{
    strcpy(tys[nty].name, name);
    tys[nty].size = size; tys[nty].align = align; tys[nty].vis = vis;
    nty++;
}
static struct Ty* findty(const char* n)
{
    for (int i = 0; i < nty; i++) if (!strcmp(tys[i].name, n)) return &tys[i];
    fprintf(stderr, "no type %s\n", n); exit(3);
}
struct Gl { char name[96]; void* p; size_t size; VisFn vis; int fill; unsigned char* img; };
static struct Gl gls[1024]; static int ngl;
void tuo_regglobal(const char* name, void* p, size_t size, VisFn vis, int fill)
{
    struct Gl* g = &gls[ngl++];
    strcpy(g->name, name); g->p = p; g->size = size; g->vis = vis; g->fill = fill;
    g->img = (unsigned char*) malloc(size ? size : 1);
    memcpy(g->img, p, size);
    char nm[128]; sprintf(nm, "g:%s", name);
    reg(nm, p, size);
    nperm = nreg;
}

static u64 curargs[12]; static int curptr[12]; static int pending_arg = -1; /* the case's argument values, for links to "the argument object" */
/* mode: 1 = SET (fill), 2 = DUMP */
int tuo_mode;
static int npool;
struct Pool { char name[96]; void* p; struct Ty* t; size_t size; int cnt; int ptrs; };
static struct Pool pool[16384]; static int npl;

static void fillbytes(void* p, size_t n)
{
    for (size_t i = 0; i < n; i++) ((unsigned char*) p)[i] = (unsigned char) tuo_draw();
}
void* tuo_new(const char* label, const char* ty, int count)
{
    struct Ty* t = findty(ty);
    size_t sz = t->size * (count ? count : 1);
    void* p = _aligned_malloc(sz + 16, t->align > 16 ? t->align : 16);
    memset(p, 0, sz + 16);
    char nm[96];
    if (label) strcpy(nm, label); else sprintf(nm, "p%d", npool++);
    reg(nm, p, sz);
    if (pending_arg >= 0) { curargs[pending_arg] = (u64) (uintptr_t) p; curptr[pending_arg] = 1; pending_arg = -1; }
    int pi = npl++;
    strcpy(pool[pi].name, nm); pool[pi].p = p; pool[pi].t = t; pool[pi].size = sz; pool[pi].cnt = count; pool[pi].ptrs = 0;
    int m = tuo_mode; tuo_mode = 1;
    char path[128];
    for (int i = 0; i < (count ? count : 1); i++) {
        if (count) sprintf(path, "%s[%d]", nm, i); else strcpy(path, nm);
        if (t->vis) t->vis(path, (char*) p + i * t->size);
    }
    tuo_mode = m;
    return p;
}
void* tuo_newbuf(const char* label, size_t n)
{
    void* p = _aligned_malloc(n + 16, 16);
    memset(p, 0, n + 16);
    char nm[96];
    if (label) strcpy(nm, label); else sprintf(nm, "p%d", npool++);
    reg(nm, p, n);
    int pi = npl++;
    strcpy(pool[pi].name, nm); pool[pi].p = p; pool[pi].t = 0; pool[pi].size = n; pool[pi].ptrs = 0;
    fillbytes(p, n);
    return p;
}
static void pool_reset(void)
{
    for (int i = 0; i < npl; i++) _aligned_free(pool[i].p);
    npl = 0; npool = 0; nreg = nperm;
}

/* ---- leaf API used by the generated visitors ---- */
char* tuo_cat(char* b, const char* a, const char* s)
{
    sprintf(b, "%s%s", a, s);
    return b;
}
char* tuo_cati(char* b, const char* a, const char* s, int i)
{
    sprintf(b, "%s%s[%d]", a, s, i);
    return b;
}
u64 tuo_leaf_int(const char* path, int bits, int sgn)
{
    u64 r = tuo_draw();
    struct Dom* d = finddom(path);
    if (d) return d->v[r % d->n];
    if (bits < 64) {
        u64 m = (1ULL << bits) - 1;
        r &= m;
        if (sgn && (r >> (bits - 1))) r |= ~m;
    }
    return r;
}
static unsigned f32bits(float f) { unsigned u; memcpy(&u, &f, 4); return u; }
static u64 f64bits(double f) { u64 u; memcpy(&u, &f, 8); return u; }
unsigned tuo_leaf_f32(const char* path)
{
    u64 r = tuo_draw();
    struct Dom* d = finddom(path);
    if (d) return (unsigned) d->v[r % d->n];
    switch (r & 7) {
    case 0: return f32bits(0.0f);
    case 1: return f32bits(1.0f);
    case 2: return f32bits(-1.0f);
    case 3: return f32bits(0.5f);
    case 4: return f32bits(-0.5f);
    default: return f32bits((float) (((r >> 8) % 2000001) / 1000.0 - 1000.0));
    }
}
u64 tuo_leaf_f64(const char* path)
{
    u64 r = tuo_draw();
    struct Dom* d = finddom(path);
    if (d) return d->v[r % d->n];
    switch (r & 7) {
    case 0: return f64bits(0.0);
    case 1: return f64bits(1.0);
    case 2: return f64bits(-1.0);
    case 3: return f64bits(0.5);
    case 4: return f64bits(-0.5);
    default: return f64bits(((r >> 8) % 2000001) / 1000.0 - 1000.0);
    }
}
static void* tuo_leaf_ptr_tgt(const char* path, const char* tgt);

static void* mk_target(const char* path, const char* tgt)
{
    if (!strcmp(tgt, "null")) return 0;
    if (!strncmp(tgt, "fn:", 3)) {
        for (int i = 0; i < nfp; i++) if (!strcmp(fps[i].name, tgt + 3)) return (void*) fps[i].a;
        fprintf(stderr, "no function %s\n", tgt + 3); exit(4);
    }
    if (!strncmp(tgt, "maybebuf:", 9)) return (tuo_draw() & 1) ? 0 : tuo_newbuf(0, (size_t) atoi(tgt + 9));
    if (!strncmp(tgt, "mayarg:", 7)) {
        /* mayarg:N:Type = the case's pointer argument N, else (N is not a pointer argument) a new Type */
        int n = atoi(tgt + 7);
        if (curptr[n]) return (tuo_draw() & 1) ? (void*) (uintptr_t) curargs[n] : 0;
        char t2[128]; sprintf(t2, "maybe:%s", strchr(tgt + 7, ':') + 1);
        if (!strcmp(strchr(tgt + 7, ':') + 1, "null")) return 0;
        return tuo_leaf_ptr_tgt(path, t2);
    }
    if (tgt[0] == '&') {
        char nm[128]; sprintf(nm, "g:%s", tgt + 1);
        for (int i = 0; i < ngl; i++) if (!strcmp(gls[i].name, tgt + 1)) return gls[i].p;
        fprintf(stderr, "no global %s\n", tgt + 1); exit(4);
    }
    if (!strncmp(tgt, "buf:", 4)) return tuo_newbuf(0, (size_t) atoi(tgt + 4));
    if (!strncmp(tgt, "cell:", 5)) {
        int bits = atoi(tgt + 5);
        void* cp = tuo_newbuf(0, 8);
        u64 cv = tuo_leaf_int(path, bits, 1);
        memcpy(cp, &cv, bits / 8);
        return cp;
    }
    if (!strncmp(tgt, "ptrs:", 5)) {
        char ty[96]; strcpy(ty, tgt + 5);
        char* c = strchr(ty, ':'); *c = 0;
        int n = atoi(c + 1);
        void** arr = (void**) _aligned_malloc(n * 8 + 16, 16);
        memset(arr, 0, n * 8 + 16);
        char nm[96]; sprintf(nm, "p%d", npool++);
        reg(nm, arr, n * 8);
        int pi = npl++;
        strcpy(pool[pi].name, nm); pool[pi].p = arr; pool[pi].t = 0; pool[pi].size = n * 8; pool[pi].cnt = 0; pool[pi].ptrs = n;
        for (int i = 0; i < n; i++) arr[i] = tuo_new(0, ty, 0);
        return arr;
    }
    if (!strncmp(tgt, "new:", 4)) {
        char ty[96]; strcpy(ty, tgt + 4);
        int cnt = 0;
        char* c = strchr(ty, ':');
        if (c) { *c = 0; cnt = atoi(c + 1); }
        return tuo_new(0, ty, cnt);
    }
    fprintf(stderr, "bad link target %s\n", tgt); exit(5);
}
static void* tuo_leaf_ptr_tgt(const char* path, const char* tgt)
{
    if (!strncmp(tgt, "maybe:", 6)) {
        if (tuo_draw() & 1) return 0;
        char t[128]; sprintf(t, "new:%s", tgt + 6);
        return mk_target(path, t);
    }
    return mk_target(path, tgt);
}
void* tuo_leaf_ptr(const char* path)
{
    struct Link* l = findlink(path);
    if (!l) return 0;
    if (l->ngate) {
        int ok = 0;
        for (int k = 0; k < l->ngate; k++) if (curfn && !strcmp(curfn, l->gate[k])) ok = 1;
        if (!ok) return 0;
    }
    if (!strncmp(l->tgt, "maybe:", 6)) {
        int forced = 0;
        for (int k = 0; k < l->nforce; k++) if (curfn && !strcmp(curfn, l->force[k])) forced = 1;
        if (!forced && (tuo_draw() & 1)) return 0;
        char t[128]; sprintf(t, "new:%s", l->tgt + 6);
        return mk_target(path, t);
    }
    return mk_target(path, l->tgt);
}
void tuo_dump_i(const char* path, long long v) { tuo_emit("%s=%lld", path, v); }
void tuo_dump_u(const char* path, u64 v) { tuo_emit("%s=%llu", path, v); }
void tuo_dump_f32(const char* path, unsigned v) { tuo_emit("%s=f%08x", path, v); }
void tuo_dump_f64(const char* path, u64 v) { tuo_emit("%s=f%016llx", path, v); }
void tuo_dump_p(const char* path, const void* p)
{
    char b[160];
    tuo_emit("%s=@%s", path, pstr(p, b));
}
static void dump_bytes(const char* path, const unsigned char* p, size_t n)
{
    static char big[1 << 16];
    size_t k = 0;
    for (size_t i = 0; i < n && k + 3 < sizeof big; i++) k += sprintf(big + k, "%02x", p[i]);
    big[k] = 0;
    tuo_emit("%s=x%s", path, big);
}
void tuo_bytes(const char* path, void* p, size_t n)
{
    if (tuo_mode == 1) fillbytes(p, n);
    else dump_bytes(path, (const unsigned char*) p, n);
}

/* ---- stub call logging ---- */
static char cl[4096]; static int cln;
void tuo_call_begin(const char* name) { cln = sprintf(cl, "call %s(", name); }
static void sep(void) { if (cl[cln - 1] != '(') cln += sprintf(cl + cln, ","); }
void tuo_arg_i(long long v) { sep(); cln += sprintf(cl + cln, "%lld", v); }
void tuo_arg_u(u64 v) { sep(); cln += sprintf(cl + cln, "%llu", v); }
void tuo_arg_f32(unsigned b) { sep(); cln += sprintf(cl + cln, "f%08x", b); }
void tuo_arg_f64(u64 b) { sep(); cln += sprintf(cl + cln, "f%016llx", b); }
void tuo_arg_p(const void* p) { char b[160]; sep(); cln += sprintf(cl + cln, "@%s", pstr(p, b)); }
void tuo_arg_s(const char* s)
{
    sep();
    if (!s) { cln += sprintf(cl + cln, "\"\"null"); return; }
    cln += sprintf(cl + cln, "\"%.200s\"", s);
}
void tuo_arg_bytes(const void* p, size_t n)
{
    sep();
    cln += sprintf(cl + cln, "x");
    for (size_t i = 0; i < n; i++) cln += sprintf(cl + cln, "%02x", ((const unsigned char*) p)[i]);
}
void tuo_call_end(void) { cl[cln++] = ')'; cl[cln] = 0; tuo_emit("%s", cl); }
/* generated return values: key is "ret:<name>" */
u64 tuo_ret_int(const char* name, int bits, int sgn)
{
    char k[128]; sprintf(k, "ret:%s", name);
    u64 r = tuo_leaf_int(k, bits, sgn);
    if (getenv("TUO_TRACE")) fprintf(stderr, "ret %s = %lld\n", name, (long long) r);
    return r;
}
unsigned tuo_ret_f32(const char* name) { char k[128]; sprintf(k, "ret:%s", name); return tuo_leaf_f32(k); }
u64 tuo_ret_f64(const char* name) { char k[128]; sprintf(k, "ret:%s", name); return tuo_leaf_f64(k); }
void* tuo_ret_ptr(const char* name) { char k[128]; sprintf(k, "ret:%s", name); return tuo_leaf_ptr(k); }

int tuo_streq(const char* a, const char* b) { return !strcmp(a, b); }

/* ---- driver ---- */
extern void tuo_unit_init(void);
extern u64 tuo_unit_call(const char* fn, u64* a);
extern u64 tuo_unit_expr(const char* n);

static u64 fnv(const char* s)
{
    u64 h = 0xcbf29ce484222325ULL;
    for (; *s; s++) { h ^= (unsigned char) *s; h *= 0x100000001B3ULL; }
    return h;
}

static void dump_all(void)
{
    char path[160];
    for (int i = 0; i < ngl; i++) {
        sprintf(path, "g:%s", gls[i].name);
        if (gls[i].vis) gls[i].vis(path, gls[i].p); else dump_bytes(path, gls[i].p, gls[i].size);
    }
    for (int i = 0; i < npl; i++) {
        if (pool[i].t) {
            int cnt = pool[i].cnt;
            for (int k = 0; k < (cnt ? cnt : 1); k++) {
                if (cnt) sprintf(path, "%s[%d]", pool[i].name, k); else strcpy(path, pool[i].name);
                if (pool[i].t->vis) pool[i].t->vis(path, (char*) pool[i].p + k * pool[i].t->size);
            }
        } else if (pool[i].ptrs) {
            for (int k = 0; k < pool[i].ptrs; k++) {
                char bb[160];
                tuo_emit("%s[%d]=@%s", pool[i].name, k, pstr(((void**) pool[i].p)[k], bb));
            }
        } else dump_bytes(pool[i].name, (unsigned char*) pool[i].p, pool[i].size);
    }
}

static void run_case(struct Fn* f, int idx)
{
    curfn = f->name;
    rs = fnv(f->name) ^ ((u64) (idx + 1) * 0x9E3779B97F4A7C15ULL);
    tuo_draw();
    lh = 0xcbf29ce484222325ULL;
    pool_reset();
    for (int i = 0; i < ngl; i++) memcpy(gls[i].p, gls[i].img, gls[i].size);
    tuo_mode = 1;
    char path[160];
    u64* a = curargs;
    memset(curargs, 0, sizeof curargs); memset(curptr, 0, sizeof curptr);
    int argfirst = 0;
    for (int i = 0; i < nlink; i++) if (!strncmp(links[i].tgt, "mayarg:", 7)) argfirst = 1;
    if (argfirst) /* pointer arguments exist before the globals are filled, so links can name them */
        for (int i = 0; i < f->nargs; i++)
            if (!strncmp(f->args[i], "p:", 2)) { char lab[16]; sprintf(lab, "a%d", i); pending_arg = i; a[i] = (u64) (uintptr_t) tuo_new(lab, f->args[i] + 2, 0); curptr[i] = 1; }
    for (int i = 0; i < ngl; i++) {
        if (!gls[i].fill) continue;
        sprintf(path, "g:%s", gls[i].name);
        if (gls[i].vis) gls[i].vis(path, gls[i].p); else fillbytes(gls[i].p, gls[i].size);
    }
    for (int i = 0; i < f->nargs; i++) {
        const char* t = f->args[i];
        char key[160]; sprintf(key, "arg:%s#%d", f->name, i);
        char lab[16]; sprintf(lab, "a%d", i);
        if (argfirst && !strncmp(t, "p:", 2)) continue;
        if (t[0] == 'i') {
            int bits = atoi(t + 1); a[i] = tuo_leaf_int(key, bits, t[strlen(t) - 1] == 's');
        } else if (!strcmp(t, "f32")) a[i] = tuo_leaf_f32(key);
        else if (!strcmp(t, "f64")) a[i] = tuo_leaf_f64(key);
        else if (!strcmp(t, "null")) a[i] = 0;
        else if (!strncmp(t, "e:", 2)) a[i] = tuo_unit_expr(t + 2);
        else if (!strncmp(t, "fn:", 3)) a[i] = (u64) (uintptr_t) mk_target(key, t);
        else if (!strncmp(t, "pn:", 3)) a[i] = (tuo_draw() & 3) == 0 ? 0 : (u64) (uintptr_t) tuo_new(lab, t + 3, 0);
        else if (!strncmp(t, "p:", 2)) a[i] = (u64) (uintptr_t) tuo_new(lab, t + 2, 0);
        else if (!strncmp(t, "cell:", 5)) {
            int bits = atoi(t + 5);
            void* cp = tuo_newbuf(lab, 8);
            u64 cv = tuo_leaf_int(key, bits, 1);
            memcpy(cp, &cv, bits / 8);
            a[i] = (u64) (uintptr_t) cp;
        }
        else if (!strncmp(t, "buf:", 4)) a[i] = (u64) (uintptr_t) tuo_newbuf(lab, (size_t) atoi(t + 4));
        else { fprintf(stderr, "bad arg %s\n", t); exit(6); }
    }
    tuo_emit("case %s %d", f->name, idx);
    if (getenv("TUO_PRE")) { tuo_mode = 2; dump_all(); tuo_mode = 1; } /* debug: the state before the call */
    u64 r = tuo_unit_call(f->name, a);
    tuo_mode = 2;
    char b[160];
    if (!strcmp(f->ret, "v")) tuo_emit("ret=void");
    else if (!strcmp(f->ret, "i")) tuo_emit("ret=%lld", (long long) r);
    else if (!strcmp(f->ret, "u")) tuo_emit("ret=%llu", r);
    else if (!strcmp(f->ret, "f32")) tuo_emit("ret=f%08x", (unsigned) r);
    else if (!strcmp(f->ret, "f64")) tuo_emit("ret=f%016llx", r);
    else tuo_emit("ret=@%s", pstr((void*) (uintptr_t) r, b));
    dump_all();
    printf("%s %d %016llx\n", f->name, idx, lh);
}

int main(int argc, char** argv)
{
    if (argc < 2) { fprintf(stderr, "usage: oracle cfg [--full] [fn [idx]]\n"); return 1; }
    load_cfg(argv[1]);
    const char* only = 0; int oidx = -1;
    for (int i = 2; i < argc; i++) {
        if (!strcmp(argv[i], "--full")) full = 1;
        else if (!only) only = argv[i];
        else oidx = atoi(argv[i]);
    }
    tuo_unit_init();
    for (int i = 0; i < nfn; i++) {
        if (only && strcmp(only, fns[i].name)) continue;
        for (int k = 0; k < fns[i].n; k++) {
            if (oidx >= 0 && k != oidx) continue;
            run_case(&fns[i], k);
        }
    }
    return 0;
}
