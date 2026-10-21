/* Round 9 part of the melee_items C oracle; #included by oracle.c after round8.c and reusing
 * round 2's LCG / Item fill+snap, round 4's r4_pre / r4_h and round 7's vector draws and
 * __frsqrte. Checks src/itkinds9.rs (26 hand ports) against round9_bodies.h, verbatim copies of
 * the decomp C extracted by gen_round9.py. Built like the rest: contraction on + gwtool lowering
 * (gen_expected.sh). The game's own random.c (TARGET_PC) is compiled in; each case sets the
 * logic seed from the case LCG and hashes it after the call, so draw count and order are
 * checked too. */
void Replay_RandTrace(u32 caller, u32 seed_after, s32 stream) { (void) caller; (void) seed_after; (void) stream; }
#include "sysdolphin/baselib/random.c"

static const f32 zero_init[1] = { 0.0f };        /* itdrop.c */
static Vec3 const _Toy_803B88D4 = { 0 };         /* toy.c */
#define it_804D6D28 it_804D6D28_o
#include "round9_bodies.h"
#undef it_804D6D28

/* sqrt input: any RF() value, moderate, or their absolute values */
static f32 r9_x(void)
{
    u32 s = (R() >> 16) % 4;
    f32 v = (s & 1) ? RM() : RF();
    return s >= 2 ? (v < 0 ? -v : v) : v;
}
/* an int argument: small (0..199), sometimes any s32 */
static s32 r9_i(void) { return ((R() >> 16) & 3) ? (s32) ((R() >> 16) % 200) : (s32) R(); }
static u16 r9_tab[33];
static u8 r9_kinds[33];
static itEgg_ItemVars r9_egg;
static itTaruAttributes r9_taru;
static it_2E5A_Attrs r9_2e5a;
static itPeachTurnipAttributes r9_turnip;
#define SEED() (*HSD_RandSeedPtr = R())
/* a seed whose next draw is > 0.9 (the grapple gravity branch): (s * 214013 + 2531011) >> 16 >= 0xE667 */
#define SEED_HI() do { u32 s_; do { s_ = R(); } while (((s_ * 214013u + 2531011u) >> 16) < 0xE667u); *HSD_RandSeedPtr = s_; } while (0)
/* sequenced: the MSVC-ABI target evaluates call arguments right to left */
static u32 r9_out;
#define SEED_H(x) (r9_out = (u32) (x), H(r9_out, *HSD_RandSeedPtr))

/* grapple gravity inputs where the fused `0.6f * r + |v|` differs from two f32 roundings (about
 * 0.6% of draws); the test itself uses volatile temporaries so it is not contracted */
static void r9_grav_pick(f32* v_out)
{
    for (;;) {
        u32 s_ = R();
        u32 k = (s_ * 214013u + 2531011u) >> 16;
        f32 v = RM();
        f32 w = v < 0 ? -v : v;
        f32 r = (f32) k / 65536.0f;
        volatile f32 p = 0.6f * r;
        volatile f32 q = p + w;
        volatile f64 d = (f64) 0.6f * (f64) r;
        volatile f64 e = d + (f64) w;
        if (k >= 0xE667u && (f32) e != q) { *HSD_RandSeedPtr = s_; *v_out = v; return; }
    }
}

static void round9(void)
{
    T(2901, "R9_SQRTF_STORE", { f32 y = 7.0f; f32 v = r9_x(); f32 r = sqrtf_store(v, &y); ret = H(bits(r), bits(y)); });
    T(2902, "R9_ITZAKO_SQRTF", { ret = bits(itzako_sqrtf(r9_x())); });
    T(2903, "R9_TYDISPLAY_SQRT_STORE", { f32 y = 7.0f; f32 v = r9_x(); f32 r = _tyDisplay_80318CB4_sqrt_store(v, &y); ret = H(bits(r), bits(y)); });
    T(2904, "R9_SQRTF_ACCURATE_STORE", { f32 y = 7.0f; f32 v = r9_x(); f32 r = sqrtf_accurate_store(v, &y); ret = H(bits(r), bits(y)); });
    T(2905, "R9_SQRTF_ACCURATE_LOCAL", { ret = bits(sqrtf_accurate_local(r9_x())); });
    T(2906, "R9_SQRTF_ACCURATE_SP18", { ret = bits(sqrtf_accurate_sp18(r9_x())); });
    T(2907, "R9_MY_SQRTF", { ret = bits(my_sqrtf(r9_x())); });
    T(2908, "R9_MY_SQRTF_ACCURATE", { ret = bits(my_sqrtf_accurate(r9_x())); });
    T(2909, "R9_TOY_80309338", { Vec3 a, b; u32 s; r7_vin(&a); r7_vin(&b); s = (R() >> 16) % 8; ret = bits(_Toy_80309338(s == 0 ? NULL : &a, s == 1 ? NULL : &b)); });
    T(2910, "R9_RANDI_PERM", { f32 f = RM() * 256.0f; SEED(); ret = SEED_H(randi_perm(f)); });
    T(2911, "R9_RANDI_PERM_INT", { s32 i = r9_i(); SEED(); ret = SEED_H(randi_perm_int(i)); });
    T(2912, "R9_KYASARIN_RANDI", { s32 i = r9_i(); SEED(); ret = SEED_H(itKyasarin_Randi(i)); });
    T(2913, "R9_ATTRRAND", { memset(&r9_egg, 0, sizeof r9_egg); r9_egg.rand_max = r9_i(); SEED(); ret = SEED_H(attrRand(&r9_egg)); });
    T(2914, "R9_8026C65C", { int n = 1 + (int) ((R() >> 16) % 32); int k, acc = 0; ItemPickTable t; for (k = 0; k <= n; k++) { r9_tab[k] = (u16) acc; r9_kinds[k] = (u8) (R() >> 16); acc += 1 + (int) ((R() >> 16) % 50); } memset(&t, 0, sizeof t); t.size = (u8) n; t.x4 = r9_kinds; t.x8 = (u16) r9_tab[n]; t.xC = r9_tab; SEED(); ret = SEED_H(it_8026C65C(&t)); });
    T(2915, "R9_TARU_RANDCHECK", { memset(&r9_taru, 0, sizeof r9_taru); r9_taru.x0 = r9_i(); r9_taru.x4 = r9_i(); SEED(); ret = SEED_H(itTaru_RandCheck(&r9_taru)); });
    T(2916, "R9_80278800_RAND_VEC", { Vec3 v; r7_vin(&v); SEED(); it_80278800_rand_vec(&ov, &v); ret = SEED_H(0); });
    T(2917, "R9_802CBAA8_INLINE", { r4_pre(ip); SEED(); it_802CBAA8_inline(g); ret = SEED_H(r4_h(ip)); });
    T(2918, "R9_802CBD24_INLINE", { r4_pre(ip); SEED(); it_802CBD24_inline(g); ret = SEED_H(r4_h(ip)); });
    T(2919, "R9_SAMUS_GRAPPLE_CALC_GRAV", { f32 v = r7_c(); SEED(); ret = SEED_H(bits(samus_grapple_calc_grav(v))); });
    T(2920, "R9_802B9328_GRAV", { f32 v = r7_c(); SEED(); ret = SEED_H(bits(it_802B9328_grav(v))); });
    T(2927, "R9_SAMUS_GRAPPLE_CALC_GRAV_HI", { f32 v = RM(); SEED_HI(); ret = SEED_H(bits(samus_grapple_calc_grav(v))); });
    T(2928, "R9_802B9328_GRAV_HI", { f32 v = RM(); SEED_HI(); ret = SEED_H(bits(it_802B9328_grav(v))); });
    T(2929, "R9_SAMUS_GRAPPLE_CALC_GRAV_FUSED", { f32 v; r9_grav_pick(&v); ret = SEED_H(bits(samus_grapple_calc_grav(v))); });
    T(2921, "R9_8026F53C", { Vec3 v; bool chk; r7_vin(&v); chk = (R() >> 16) & 1; r2_common.x54_float = r7_c(); r2_attr.x18 = RF(); SEED(); it_8026F53C(g, &v, chk); ret = SEED_H(r7_v(&v)); });
    T(2922, "R9_80286340", { s32 a = r9_i(), b = r9_i(), c = r9_i(), d = r9_i(); SEED(); ret = SEED_H(it_80286340(g, a, b, c, d)); });
    T(2923, "R9_8028A114", { s32 a = r9_i(), b = r9_i(), c = r9_i(), d = r9_i(); SEED(); ret = SEED_H(it_8028A114(g, a, b, c, d)); });
    T(2924, "R9_80289BE8_INLINE", { Vec3 p, v; f32 s = r7_c(); SEED(); it_80289BE8_inline(g, s, &p, &v); ret = SEED_H(H(r7_v(&p), r7_v(&v))); });
    T(2925, "R9_802E61C4", { s32 a1, a2; int k; memset(&r9_2e5a, 0, sizeof r9_2e5a); r9_2e5a.x24 = RM(); for (k = 0; k < 4; k++) r9_2e5a.x2C[k] = RM(); r2_art.x4_specialAttributes = &r9_2e5a; a1 = (s32) ((R() >> 16) % 2000) - 1000; a2 = (s32) ((R() >> 16) % 4); SEED(); ret = SEED_H(it_802E61C4(g, a1, a2)); });
    T(2926, "R9_802BD32C", { int k; memset(&r9_turnip, 0, sizeof r9_turnip); r9_turnip.x4_length = (s32) ((R() >> 16) % 8); for (k = 0; k < 8; k++) r9_turnip.x8[k].x0_odds = (s32) ((R() >> 16) % 100); r2_art.x4_specialAttributes = &r9_turnip; SEED(); ret = SEED_H(it_802BD32C(g)); });
}
