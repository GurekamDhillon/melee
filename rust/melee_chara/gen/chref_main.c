/* C oracle main for the character ports (gen/chref.py builds it with the generated drivers).
 * Every listed function runs on CASES random fighters; tests/chars_ref.rs repeats the fill
 * order exactly and compares the per-function hash. Computed from C source only. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/ftcommon.h"
#include "ft/fighter.h"
#include <sysdolphin/baselib/gobj.h>

#define FT_CAPS_MAX_JUMPS(fp) ((fp)->co_attrs.max_jumps)

/* callees of the ftCommon cut, as the vanilla game behaves (script/Geno add-ons pass through) */
float ScriptGame_ModValue(int slot, int field, float base) { (void) slot; (void) field; return base; }
f32 Geno_FallLimit(Fighter* fp, f32 terminal) { (void) fp; return terminal; }
static unsigned g_sfx;
static float g_fric;
void ft_PlaySFX(Fighter* fp, enum_t id, int vol, int pan) { (void) fp; g_sfx += id + vol + pan; }
float ft_GetGroundFrictionMultiplier(Fighter* fp) { (void) fp; return g_fric; }
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
ftCommonData* p_ftCommonData;
/* script_fighter_caps.h (TARGET_PC): vanilla = no caps script, the attribute itself */
int ScriptGame_CapsMaxJumps(Fighter* fp) { return fp->co_attrs.max_jumps; }

/* Round 2: the group-3 calls the translated callbacks make (melee_chara::stubs). Each stub
 * records its call (id + every non-pointer argument; a gobj argument as 1 = the fighter's own,
 * 0 = NULL, 2 = other) into g_tr, and a value-returning one answers from g_seed. tests/chars_ref.rs
 * installs the same recorders. Group 3's real ftCommon_8007D5D4/8007D60C run (cut below); their
 * own callees Geno_GroundEdge / pl_8003FC44 are recorders too. */
static u32 g_tr, g_seed;
static HSD_GObj* g_self;
static void tr(u32 x) { g_tr = (g_tr ^ x) * 16777619u; }
static void trf(float f) { union { float f; u32 u; } c; c.f = f; tr(c.u); }
static void trg(void* p) { tr(p == 0 ? 0u : p == (void*) g_self ? 1u : 2u); }
static u32 pick(void) { g_seed = g_seed * 1103515245u + 12345u; return g_seed >> 16; }
#define STUB_G(id, name) void name(HSD_GObj* gobj) { tr(id); trg(gobj); }
STUB_G(1, ft_80084F3C)
STUB_G(2, ft_80084EEC)
STUB_G(3, ft_80085134)
STUB_G(4, ft_80084DB0)
STUB_G(5, ft_80084FA8)
STUB_G(6, ft_80083F88)
STUB_G(8, ftAnim_8006EBA4)
STUB_G(10, ft_8008A2BC)
STUB_G(11, ftCo_Fall_Enter)
bool ftAnim_IsFramesRemaining(HSD_GObj* gobj) { tr(7); trg(gobj); return pick() & 1; }
GroundOrAir ft_80081D0C(HSD_GObj* gobj) { tr(12); trg(gobj); return pick() & 1; }
GroundOrAir ft_80082708(HSD_GObj* gobj) { tr(13); trg(gobj); return pick() & 1; }
bool ft_800827A0(HSD_GObj* gobj) { tr(14); trg(gobj); return pick() & 1; }
void Fighter_ChangeMotionState(HSD_GObj* gobj, FtMotionId msid, MotionFlags flags, f32 a, f32 b, f32 c, HSD_GObj* g3)
{ tr(9); trg(gobj); tr((u32) msid); tr(flags); trf(a); trf(b); trf(c); trg(g3); }
void ftCommon_8007D7FC(Fighter* fp) { tr(15); tr(fp == g_self->user_data); }
void ftCo_80096900(HSD_GObj* gobj, int a, int b, bool c, float d, float e)
{ tr(16); trg(gobj); tr((u32) a); tr((u32) b); tr((u32) c); trf(d); trf(e); }
void Geno_GroundEdge(Fighter* fp, int landing) { tr(17); tr(fp == g_self->user_data); tr((u32) landing); }
void pl_8003FC44(int slot, int sub) { tr(18); tr((u32) slot); tr((u32) sub); }

#include "ftcommon_extract.inc"
#include "chref_table.inc"

int printf(const char*, ...);
int strcmp(const char*, const char*);
int _flushall(void);

#define CASES 4000
/* round-2 callbacks (index >= N_R1) are thin wrappers checked through the stub trace: fewer cases */
#define CASES_R2 256
#define ATTR_WORDS 1024

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    return (float) ((int) (rnd() % 2001) - 1000) / 7.0f;
}
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
/* one 32-bit slot: small int, zero, or a float's bits */
static u32 slot(u32 small)
{
    u32 k = rnd() % 8;
    if (k < 3) return rnd() % small;
    if (k == 3) return 0;
    return B(rf());
}

typedef unsigned long long u64;
static u64 fnv(u64 h, const void* p, unsigned n)
{
    const unsigned char* b = p;
    while (n--) { h ^= *b++; h *= 0x100000001B3ull; }
    return h;
}

static Fighter fp;
static u32 attrs[ATTR_WORDS];
static ftCommonData cd;

/* Bitfields the ported functions (and the ftCommon helpers they call) read or write. Native C
 * packs bitfields LSB-first while melee_types keeps them in GC order (MSB-first), so the same
 * bytes hold different values: these are compared by value, and their bytes are zeroed (both
 * sides) before the byte hash. The byte of each is found by probing a zeroed Fighter. */
#define NBF 14
static int bf_get(int i)
{
    switch (i) {
    case 0: return fp.fall_fast;
    case 1: return fp.x2219_b5;
    case 2: return fp.x221F_b3;
    case 3: return fp.x2223_b4;
    case 4: return fp.throw_flags_b0;
    case 5: return fp.x2070.x2071_b6;
    case 6: return fp.mv.ca.specialhi.x2_b0;
    case 7: return fp.mv.ca.specialhi.x2_b1;
    case 8: return fp.is_sub_fighter;
    case 9: return fp.x2227_b0;
    case 10: return fp.allow_interrupt;
    case 11: return fp.invisible;
    case 12: return fp.reflecting;
    default: return fp.x2218_b4;
    }
}
static void bf_set(int i, int v)
{
    switch (i) {
    case 0: fp.fall_fast = v; break;
    case 1: fp.x2219_b5 = v; break;
    case 2: fp.x221F_b3 = v; break;
    case 3: fp.x2223_b4 = v; break;
    case 4: fp.throw_flags_b0 = v; break;
    case 5: fp.x2070.x2071_b6 = v; break;
    case 6: fp.mv.ca.specialhi.x2_b0 = v; break;
    case 7: fp.mv.ca.specialhi.x2_b1 = v; break;
    case 8: fp.is_sub_fighter = v; break;
    case 9: fp.x2227_b0 = v; break;
    case 10: fp.allow_interrupt = v; break;
    case 11: fp.invisible = v; break;
    case 12: fp.reflecting = v; break;
    default: fp.x2218_b4 = v; break;
    }
}
static unsigned bf_byte[NBF];

/* Round 3: a callback a port stores into one of Fighter's HSD_GObjEvent fields is replaced by
 * the FNV-1a of its key "<setter tu>:<C name>" before hashing (Rust: chars::cb::CALLBACKS), so
 * the stored function, not its address, is compared. Other values stay as they are. */
static unsigned long long cb_key(const void* p)
{
    int t, j;
    for (t = 0; t < N_CBTAB; t++)
        for (j = 0; CBP[t][j]; j++)
            if (CBP[t][j] == p) {
                unsigned long long h = 0xCBF29CE484222325ull;
                const char* c = CBN[t][j];
                while (*c) { h ^= (unsigned char) *c++; h *= 0x100000001B3ull; }
                return h;
            }
    return 0;
}
static void cb_normalise(void)
{
    int i;
    for (i = 0; i < N_CB_OFF; i++) {
        void** slot = (void**) ((char*) &fp + CB_OFF[i]);
        unsigned long long k = cb_key(*slot);
        if (k) __builtin_memcpy(slot, &k, 8);
    }
}
static void bf_probe(void)
{
    int i;
    unsigned j;
    for (i = 0; i < NBF; i++) {
        unsigned char* b = (unsigned char*) &fp;
        for (j = 0; j < sizeof fp; j++) b[j] = 0;
        bf_set(i, 1);
        for (j = 0; j < sizeof fp; j++) if (b[j]) break;
        bf_byte[i] = j;
    }
}

int main(int argc, char** argv)
{
    int verbose = argc > 1 && !strcmp(argv[1], "-v");
    unsigned f, n = sizeof FUNCS / sizeof FUNCS[0];
    HSD_GObj g;
    bf_probe();
    printf("sizes %u %u", (unsigned) sizeof(Fighter), (unsigned) sizeof(ftCommonData));
    {
        int i;
        for (i = 0; i < NBF; i++) printf(" %u", bf_byte[i]);
    }
    printf("\n");
    for (f = 0; f < n; f++) {
        u64 h = 0xCBF29CE484222325ull;
        int k;
        /* mv.ca.specialhi.x2_b0/b1 share their byte with every other character's motion vars
         * (the mv union): only the Captain function treats that byte as bitfields */
        unsigned bmask = strcmp(FUNCS[f].name, "ftCa_SpecialLw_800E49FC") ? ((1u << NBF) - 1) & ~0xC0u : (1u << NBF) - 1;
        if (verbose) _flushall();
        for (k = 0; k < (f < N_R1 ? CASES : CASES_R2); k++) {
            unsigned i, ret = 0;
            float a;
            int ia, nul, o1, o2;
            Vec3 v3;
            Vec2 v2;
            u32* w = (u32*) &fp;
            rs = 0x9E3779B9u ^ (f * 7919u + (u32) k * 104729u + 1u);
            for (i = 0; i < sizeof fp / 4; i++) w[i] = slot(400);
            fp.motion_id = (rnd() % 4) ? 330 + rnd() % 300 : rnd() % 1000;
            for (i = 0; i < NBF; i++) if (bmask >> i & 1) ((unsigned char*) &fp)[bf_byte[i]] = 0;
            for (i = 0; i < NBF; i++) { int v = rnd() & 1; if (bmask >> i & 1) bf_set(i, v); }
            for (i = 0; i < ATTR_WORDS; i++) attrs[i] = slot(8);
            w = (u32*) &cd;
            for (i = 0; i < sizeof cd / 4; i++) w[i] = slot(8);
            fp.dat_attrs = attrs;
            p_ftCommonData = &cd;
            g_sfx = 0;
            g_fric = rf();
            a = rf();
            ia = (int) (rnd() % 4);
            if (rnd() % 2) ia = (int) (rnd() % 20);
            nul = rnd() % 10 == 0;
            o1 = (int) rnd(); o2 = (int) rnd();
            v3.x = rf(); v3.y = rf(); v3.z = rf();
            v2.x = rf(); v2.y = rf();
            g_seed = rnd();
            g_tr = 0;
            g_self = &g;
            g.user_data = &fp;
            FUNCS[f].drv((int) f, &g, &fp, a, ia, nul, &v3, &v2, &o1, &o2, &ret);
            fp.dat_attrs = 0;
            {
                unsigned bits = 0;
                for (i = 0; i < NBF; i++) if (bmask >> i & 1) bits |= (unsigned) bf_get(i) << i;
                for (i = 0; i < NBF; i++) if (bmask >> i & 1) ((unsigned char*) &fp)[bf_byte[i]] = 0;
                h = fnv(h, &bits, 4);
                cb_normalise();
                if (verbose) printf("b%02X ", bits);
            }
            if (verbose) {
                u64 hf = fnv(0xCBF29CE484222325ull, &fp, sizeof fp);
                printf("%u %d %08X %08X %08X %08X %08X %08X %d %d %08X %016llX %016llX %016llX %08X\n", f, k,
                       ret, B(v3.x), B(v3.y), B(v3.z), B(v2.x), B(v2.y), o1, o2, g_sfx, hf,
                       fnv(0xCBF29CE484222325ull, attrs, sizeof attrs), fnv(0xCBF29CE484222325ull, &cd, sizeof cd), g_tr);
            }
            h = fnv(h, &fp, sizeof fp);
            h = fnv(h, attrs, sizeof attrs);
            h = fnv(h, &cd, sizeof cd);
            h = fnv(h, &ret, 4);
            h = fnv(h, &v3, sizeof v3);
            h = fnv(h, &v2, sizeof v2);
            h = fnv(h, &o1, 4);
            h = fnv(h, &o2, 4);
            h = fnv(h, &g_sfx, 4);
            h = fnv(h, &g_tr, 4);
        }
        if (!verbose) printf("%s %016llX\n", FUNCS[f].name, h);
    }
    return 0;
}
