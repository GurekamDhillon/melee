/* TU state-oracle runtime (C side). Included at the END of a generated wrapper that has already
 * #included the game TU, so statics and header types are visible. No libc: the driver is main.c.
 * Everything here has a line-for-line twin in rust/melee_modes/tests/common/tu_rt.rs; the value
 * streams (splitmix64 + the draw_* policies), the FNV-1a mixing, the pointer classes and the trace
 * format must stay identical on both sides. */
#ifndef TU_ORACLE_H
#define TU_ORACLE_H

typedef unsigned long long tuo_u64;
typedef long long tuo_i64;

static tuo_u64 tuo_st;     /* fill / argument stream */
static tuo_u64 tuo_mst;    /* mock-return stream */
static tuo_u64 tuo_trace;  /* hash of every mock call (id + arguments) */

static tuo_u64 tuo_next(tuo_u64* s)
{
    tuo_u64 z;
    *s += 0x9E3779B97F4A7C15ULL;
    z = *s;
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
    return z ^ (z >> 31);
}

static tuo_u64 tuo_mix(tuo_u64 h, tuo_u64 v) { return (h ^ v) * 0x100000001B3ULL; }
#define TUO_FNV0 0xCBF29CE484222325ULL

/* integer literals of the TU (and +-1): the boundary values its comparisons test */
#define TUO_MAXCONST 512
static tuo_i64 tuo_consts[TUO_MAXCONST];
static int tuo_nconsts;

/* integer draw: biased to small values, with full-width and negative tails */
static tuo_i64 tuo_draw_int(tuo_u64* s)
{
    tuo_u64 r = tuo_next(s);
    tuo_i64 v = (tuo_i64)(r >> 3);
    switch (r & 7) {
    case 0:
    case 1: return 0;
    case 2: return 1 + (v % 3);
    case 3: return v & 0xFF;
    case 4: return (v % 41) - 20;
    case 5: return (v % 2001) - 1000;
    case 6: return v & 0xFFFF;
    default:
        if (tuo_nconsts && ((v >> 40) & 1)) return tuo_consts[(v >> 8) % (tuo_u64)tuo_nconsts];
        return (tuo_i64)(int)(v ^ (v >> 32));
    }
}
static tuo_i64 tuo_draw_range(tuo_u64* s, tuo_i64 lo, tuo_i64 hi)
{
    tuo_u64 r = tuo_next(s);
    return lo + (tuo_i64)(r % (tuo_u64)(hi - lo + 1));
}
static float tuo_draw_f32(tuo_u64* s)
{
    tuo_u64 r = tuo_next(s);
    tuo_i64 v = (tuo_i64)(r >> 8);
    switch (r & 7) {
    case 0:
    case 1: return 0.0f;
    case 2: return (float)((v % 4001) - 2000) / 100.0f;
    case 3: return (float)((v % 41) - 20);
    case 4: return (float)((v % 6001) - 3000) / 2.0f;
    case 5: return (float)((v % 201) - 100) / 1000.0f;
    case 6: return (float)((v % 20001) - 10000) / 100.0f;
    default: return (float)(v % 3);
    }
}
static double tuo_draw_f64(tuo_u64* s)
{
    tuo_u64 r = tuo_next(s);
    tuo_i64 v = (tuo_i64)(r >> 8);
    switch (r & 7) {
    case 0:
    case 1: return 0.0;
    case 2: return (double)((v % 4001) - 2000) / 100.0;
    case 3: return (double)((v % 41) - 20);
    case 4: return (double)((v % 6001) - 3000) / 2.0;
    case 5: return (double)((v % 201) - 100) / 1000.0;
    case 6: return (double)((v % 20001) - 10000) / 100.0;
    default: return (double)(v % 3);
    }
}

/* ---- regions (pointer classes) ---- */
#define TUO_MAXREG 256
static tuo_u64 tuo_reg_base[TUO_MAXREG];
static tuo_u64 tuo_reg_size[TUO_MAXREG];
static int tuo_nreg;
static int tuo_nstatic;
static const char* tuo_reg_name[TUO_MAXREG];

static void tuo_reg_add(const void* p, tuo_u64 size, const char* name)
{
    tuo_reg_base[tuo_nreg] = (tuo_u64)(unsigned long long)p;
    tuo_reg_size[tuo_nreg] = size;
    tuo_reg_name[tuo_nreg] = name;
    tuo_nreg++;
}
static tuo_u64 tuo_ptr_class(const void* p)
{
    int i;
    tuo_u64 a = (tuo_u64)(unsigned long long)p;
    if (a == 0) return 0;
    for (i = 0; i < tuo_nreg; i++) {
        if (a >= tuo_reg_base[i] && a < tuo_reg_base[i] + tuo_reg_size[i]) {
            return ((tuo_u64)(i + 1) << 32) | (a - tuo_reg_base[i]);
        }
    }
    /* one-past-the-end of a region counts as that region */
    for (i = 0; i < tuo_nreg; i++) {
        if (a == tuo_reg_base[i] + tuo_reg_size[i]) {
            return ((tuo_u64)(i + 1) << 32) | (a - tuo_reg_base[i]);
        }
    }
    return 1;
}

/* bytes readable from p as a string: to the end of its region, else 512 */
static tuo_u64 tuo_str_limit(const void* p)
{
    int i;
    tuo_u64 a = (tuo_u64)(unsigned long long)p;
    for (i = 0; i < tuo_nreg; i++) {
        if (a >= tuo_reg_base[i] && a < tuo_reg_base[i] + tuo_reg_size[i]) return tuo_reg_base[i] + tuo_reg_size[i] - a;
    }
    return 512;
}

/* ---- function pointers ---- */
#define TUO_MAXFN 4096
static tuo_u64 tuo_fn_addr[TUO_MAXFN];
static int tuo_nfn;
static void tuo_fn_add(const void* p)
{
    tuo_fn_addr[tuo_nfn++] = (tuo_u64)(unsigned long long)p;
}
static tuo_u64 tuo_fn_class(const void* p)
{
    int i;
    tuo_u64 a = (tuo_u64)(unsigned long long)p;
    if (a == 0) return 0;
    for (i = 0; i < tuo_nfn; i++) {
        if (tuo_fn_addr[i] == a) return 0x1000 + i;
    }
    return 1;
}

/* ---- trace ---- */
static void tuo_t_begin(int id) { tuo_trace = tuo_mix(tuo_trace, 0xC0DE0000ULL + (tuo_u64)id); }
static void tuo_t_i(tuo_i64 v) { tuo_trace = tuo_mix(tuo_trace, (tuo_u64)v); }
static void tuo_t_f32(float v)
{
    union { float f; unsigned u; } x;
    x.f = v;
    tuo_trace = tuo_mix(tuo_trace, x.u);
}
static void tuo_t_f64(double v)
{
    union { double f; tuo_u64 u; } x;
    x.f = v;
    tuo_trace = tuo_mix(tuo_trace, x.u);
}
static void tuo_t_p(const void* p) { tuo_trace = tuo_mix(tuo_trace, tuo_ptr_class(p)); }
static void tuo_t_fn(const void* p) { tuo_trace = tuo_mix(tuo_trace, tuo_fn_class(p)); }
static void tuo_t_s(const char* s)
{
    int i;
    if (s == 0) {
        tuo_trace = tuo_mix(tuo_trace, 0xFFFF);
        return;
    }
    for (i = 0; i < 512 && (tuo_u64)i < tuo_str_limit(s) && s[i]; i++) tuo_trace = tuo_mix(tuo_trace, (unsigned char)s[i]);
    tuo_trace = tuo_mix(tuo_trace, 0xEEEE);
}

/* ---- state hashing ---- */
static tuo_u64 tuo_h;
static void tuo_h_i(tuo_i64 v) { tuo_h = tuo_mix(tuo_h, (tuo_u64)v); }
static void tuo_h_u(tuo_u64 v) { tuo_h = tuo_mix(tuo_h, v); }
static void tuo_h_f32(float v)
{
    union { float f; unsigned u; } x;
    x.f = v;
    tuo_h = tuo_mix(tuo_h, x.u);
}
static void tuo_h_f64(double v)
{
    union { double f; tuo_u64 u; } x;
    x.f = v;
    tuo_h = tuo_mix(tuo_h, x.u);
}
static void tuo_h_p(const void* p) { tuo_h = tuo_mix(tuo_h, tuo_ptr_class(p)); }
static void tuo_h_fn(const void* p) { tuo_h = tuo_mix(tuo_h, tuo_fn_class(p)); }
static void tuo_h_bytes(const void* p, tuo_u64 n)
{
    tuo_u64 i;
    const unsigned char* b = (const unsigned char*)p;
    for (i = 0; i < n; i++) tuo_h = tuo_mix(tuo_h, b[i]);
}

static void tuo_h_cstr(const char* s)
{
    int i;
    if (!s) return;
    for (i = 0; i < 512 && (tuo_u64)i < tuo_str_limit(s) && s[i]; i++) tuo_h = tuo_mix(tuo_h, (unsigned char)s[i]);
    tuo_h = tuo_mix(tuo_h, 0xEEEE);
}
#define TUO_MAXDYN 64
static tuo_u64 tuo_dynh[TUO_MAXDYN];
static int tuo_ndyn;
static void tuo_dyn(tuo_u64 h) { tuo_dynh[tuo_ndyn++] = h; }

/* per-object hashes of the last tuo_state_hash (debug: the driver's --dump) */
static tuo_u64 tuo_obj_hash[TUO_MAXREG];

#endif
