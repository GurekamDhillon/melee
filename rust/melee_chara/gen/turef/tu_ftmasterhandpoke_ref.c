/* C reference for src/tu/ftmasterhandpoke.rs: the whole game TU is #included verbatim and
 * lbVector_Diff/Normalize are cut from lbvector.c (shipping contraction via run.py). Callees
 * outside the TU are stand-ins both sides share: ft_80085134 is a no-op and ftBossLib_8015C208
 * writes a fixed target. */
#include "ft/kinds/ftMasterHand/ftmasterhandpoke.c"
#include "tu_ftmasterhandpoke_extract.inc"
#include <stdio.h>

static Vec3 TGT;
void ft_80085134(HSD_GObj* g) { (void) g; }
void ftBossLib_8015C208(HSD_GObj* g, Vec3* out) { (void) g; *out = TGT; }
void ftBossLib_8015BD20(HSD_GObj* g) { (void) g; }
void ftMh_MS_389_80151018(HSD_GObj* g) { (void) g; }
static int RANDI_RET, FRAMES_LEFT;
void Fighter_ChangeMotionState(HSD_GObj* g, int m, u32 f, float a, float b, float c, HSD_GObj* d) { (void) g; (void) m; (void) f; (void) a; (void) b; (void) c; (void) d; }
void ftAnim_8006EBA4(HSD_GObj* g) { (void) g; }
int HSD_Randi(int n) { (void) n; return RANDI_RET; }
int ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return FRAMES_LEFT; }
void ft_PlaySFX(Fighter* fp, enum_t id, u8 vol, u8 pan) { (void) fp; (void) id; (void) vol; (void) pan; }
int Player_GetPlayerSlotType(int slot) { return slot; }
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static u32 s = 0x70CE5u;
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
        DA.x98 = rf(); DA.x9C = rf();
        F.mv.mh.unk13.x4 = 1.0f;
        F.self_vel.x = 12345.0f; F.self_vel.y = 12345.0f;
        printf("%d %08x %08x %08x %08x %08x %08x %08x %08x %08x", i, B(TGT.x), B(TGT.y), B(F.cur_pos.x),
               B(F.cur_pos.y), B(DA.x2C), B(DA.x28), B(DA.x98), B(DA.x9C), B(F.cur_pos.z));
        ftMh_Poke1_Phys(&G);
        printf(" %08x %08x\n", B(F.self_vel.x), B(F.self_vel.y));
    }
    /* Init (int Randi -> float countdown) and Poke1_Anim (float countdown, vel clear) */
    for (i = 0; i < 400; i++) {
        DA.x90 = (int) (rnd() % 500) - 100;
        DA.x94 = (int) (rnd() % 300) - 50;
        RANDI_RET = (int) (rnd() % 400);
        ftMh_MS_358_80152880(&G);
        printf("init %d %d %d %08x %08x", DA.x90, DA.x94, RANDI_RET, B(F.mv.mh.unk13.x0), B(F.mv.mh.unk13.x4));
        FRAMES_LEFT = 0;
        F.self_vel.x = 7.0f; F.self_vel.y = 8.0f;
        F.mv.mh.unk13.x0 = (float) ((int) (rnd() % 5) - 1);
        printf(" in %08x", B(F.mv.mh.unk13.x0));
        ftMh_Poke1_Anim(&G);
        printf(" anim %08x %08x %08x %08x\n", B(F.mv.mh.unk13.x0), B(F.mv.mh.unk13.x4), B(F.self_vel.x), B(F.self_vel.y));
    }
    return 0;
}
