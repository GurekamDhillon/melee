/* C reference for src/tu/ftmasterhandbackdisappear.rs: the whole game TU is #included verbatim and
 * lbVector_Diff/Normalize are cut from lbvector.c (shipping contraction via run.py). Callees outside
 * the TU are stand-ins both sides share: ft_80085134 is a no-op and ftBossLib_8015C208 writes a fixed
 * target. Prints "phys iter tx ty tz cx cy cz x2C x28 x108 x10C -> vx vy" and
 * "ms372 x110x x110y -> x8 rate" lines. */
#include "ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c"
#include "tu_ftmasterhandbackdisappear_extract.inc"
#include <stdio.h>

static Vec3 TGT;
static float RATE;
void ft_80085134(HSD_GObj* g) { (void) g; }
void ftBossLib_8015C208(HSD_GObj* g, Vec3* out) { (void) g; *out = TGT; }
void ftBossLib_8015BD20(HSD_GObj* g) { (void) g; }
void ftBossLib_8015BF74(HSD_GObj* g, float f) { (void) g; (void) f; }
void ftBossLib_8015BE40(HSD_GObj* g, Vec3* a, float* b, float c, float d) { (void) g; (void) a; (void) b; (void) c; (void) d; }
void ftBossLib_8015C190(HSD_GObj* g) { (void) g; }
void Fighter_ChangeMotionState(HSD_GObj* g, int m, u32 f, float a, float b, float c, HSD_GObj* d) { (void) g; (void) m; (void) f; (void) a; (void) b; (void) c; (void) d; }
void ftAnim_8006EBA4(HSD_GObj* g) { (void) g; }
int ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return 1; }
void ftAnim_SetAnimRate(HSD_GObj* g, float r) { (void) g; RATE = r; }
int Player_GetPlayerSlotType(int slot) { return slot; }
void ftCommon_8007E2D0(Fighter* fp, s16 a, HSD_GObjEvent b, HSD_GObjEvent c, void (*d)(HSD_GObj*, HSD_GObj*)) { (void) fp; (void) a; (void) b; (void) c; (void) d; }
void ftMh_MS_375_80154C78(HSD_GObj* g) { (void) g; }
void ftMh_MS_378_80154CF8(HSD_GObj* g) { (void) g; }
void ftMh_MS_388_80155A58(HSD_GObj* g, HSD_GObj* h) { (void) g; (void) h; }
void ftMh_MS_389_80151018(HSD_GObj* g) { (void) g; }
void ftMh_CaptureWaitMasterHand_80155D1C(HSD_GObj* g) { (void) g; }
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static u32 s = 0x5C0DEu;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return (float) (rnd() % 7) * 1e-4f;
    return ((float) ((int) (rnd() % 200001) - 100000)) / 37.0f;
}
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

int main(void)
{
    static Fighter F;
    static HSD_GObj G;
    static ftData FD;
    static ftMasterHand_SpecialAttrs DA;
    int i;
    G.user_data = &F;
    F.ft_data = &FD;
    FD.ext_attr = &DA;
    for (i = 0; i < 4000; i++) {
        TGT.x = rf(); TGT.y = rf(); TGT.z = rf();
        F.cur_pos.x = rf(); F.cur_pos.y = rf(); F.cur_pos.z = rf();
        DA.x2C = (rnd() % 4 == 0) ? 0.0f : (float) (rnd() % 3000) / 3.0f;
        DA.x28 = rf() / 1000.0f;
        DA.x108 = rf(); DA.x10C = rf();
        F.self_vel.x = 12345.0f; F.self_vel.y = 12345.0f;
        printf("phys %d %08x %08x %08x %08x %08x %08x %08x %08x %08x %08x", i, B(TGT.x), B(TGT.y), B(TGT.z), B(F.cur_pos.x),
               B(F.cur_pos.y), B(F.cur_pos.z), B(DA.x2C), B(DA.x28), B(DA.x108), B(DA.x10C));
        ftMh_Wait1_1_Phys(&G);
        printf(" %08x %08x\n", B(F.self_vel.x), B(F.self_vel.y));
    }
    for (i = 0; i < 400; i++) {
        DA.x110_pos.x = rf() * 10.0f; DA.x110_pos.y = rf();
        F.cur_anim_frame = rf();
        ftMh_MS_372_801542E0(&G);
        printf("ms372 %08x %08x %08x -> %d %08x\n", B(DA.x110_pos.x), B(DA.x110_pos.y), B(F.cur_anim_frame), F.mv.mh.unk0.x8, B(RATE));
    }
    return 0;
}
