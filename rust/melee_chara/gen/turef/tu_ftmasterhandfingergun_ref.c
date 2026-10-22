/* C reference for src/tu/ftmasterhandfingergun.rs: the whole game TU is #included verbatim and
 * lbVector_Diff/Normalize are cut from lbvector.c (shipping contraction via run.py). Callees
 * outside the TU are stand-ins both sides share. */
#include "ft/kinds/ftMasterHand/ftmasterhandfingergun.c"
#include "tu_ftmasterhandfingergun_extract.inc"
#include <stdio.h>

static Vec3 TGT;
static int RANDI_RET, FRAMES_LEFT, PERCENT, RATE_SET;
static float RATE;
void ft_80085134(HSD_GObj* g) { (void) g; }
void ftBossLib_8015C208(HSD_GObj* g, Vec3* out) { (void) g; *out = TGT; }
void ftBossLib_8015BD20(HSD_GObj* g) { (void) g; }
void ftMh_MS_389_80151018(HSD_GObj* g) { (void) g; }
void Fighter_ChangeMotionState(HSD_GObj* g, int m, u32 f, float a, float b, float c, HSD_GObj* d) { (void) g; (void) m; (void) f; (void) a; (void) b; (void) c; (void) d; }
void ftAnim_8006EBA4(HSD_GObj* g) { (void) g; }
void ftAnim_SetAnimRate(HSD_GObj* g, float r) { (void) g; RATE = r; RATE_SET++; }
int HSD_Randi(int n) { (void) n; return RANDI_RET; }
int ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return FRAMES_LEFT; }
s32 ftLib_80087120(HSD_GObj* g) { (void) g; return PERCENT; }
void lb_8000B1CC(HSD_JObj* j, Vec3* p0, Vec3* p1) { (void) j; (void) p0; p1->x = 1.5f; p1->y = -2.25f; p1->z = 0.5f; }
static float BUL[8];
void it_802F0AE0(Item_GObj* g, Vec3* prev, Vec3* pos, ItemKind kind, s32 msid, f32 facing, f32 vx, f32 vy)
{
    (void) g; (void) kind;
    BUL[0] = prev->x; BUL[1] = prev->y; BUL[2] = pos->x; BUL[3] = pos->y;
    BUL[4] = (float) msid; BUL[5] = facing; BUL[6] = vx; BUL[7] = vy;
}
int Player_GetPlayerSlotType(int slot) { return slot; }
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static u32 s = 0xF1A6Eu;
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
    static FighterBone PARTS[64];
    int i, j;
    F.parts = PARTS;
    G.user_data = &F;
    F.ft_data = &FD;
    FD.ext_attr = &DA;
    for (i = 0; i < 4000; i++) {
        TGT.x = rf(); TGT.y = rf(); TGT.z = rf();
        F.cur_pos.x = rf(); F.cur_pos.y = rf(); F.cur_pos.z = rf();
        DA.x2C = (rnd() % 4 == 0) ? 0.0f : (float) (rnd() % 3000) / 3.0f;
        DA.x28 = rf() / 1000.0f;
        DA.xDC = rf(); DA.xE0 = rf();
        F.cmd_vars[2] = 1;
        F.self_vel.x = 12345.0f; F.self_vel.y = 12345.0f;
        printf("%d %08x %08x %08x %08x %08x %08x %08x %08x %08x", i, B(TGT.x), B(TGT.y), B(F.cur_pos.x),
               B(F.cur_pos.y), B(DA.x2C), B(DA.x28), B(DA.xDC), B(DA.xE0), B(F.cur_pos.z));
        ftMh_FingerGun1_Phys(&G);
        printf(" %08x %08x\n", B(F.self_vel.x), B(F.self_vel.y));
    }
    /* Init (float difference -> int Randi arg, f32 + int) */
    for (i = 0; i < 400; i++) {
        DA.xE4 = (float) ((int) (rnd() % 400) - 50) / 3.0f;
        DA.xE8 = DA.xE4 + (float) (rnd() % 300) / 7.0f;
        RANDI_RET = (int) (rnd() % 400);
        F.cmd_vars[0] = F.cmd_vars[1] = F.cmd_vars[2] = 9;
        ftMh_MS_363_801530A4(&G);
        printf("init %08x %08x %d %08x %u %u %u\n", B(DA.xE4), B(DA.xE8), RANDI_RET, B(F.mv.mh.unk0.x50),
               F.cmd_vars[0], F.cmd_vars[1], F.cmd_vars[2]);
    }
    /* FingerGun1_Anim: float countdown, then the percent-dependent loop count */
    for (i = 0; i < 400; i++) {
        FRAMES_LEFT = 0;
        DA.xEC = (int) (rnd() % 100);
        DA.xF0 = (int) (rnd() % 10) + 1;
        DA.xF4 = rf();
        PERCENT = (int) (rnd() % 120);
        F.mv.mh.unk0.x50 = (float) ((int) (rnd() % 5) - 1) + (rnd() % 2 ? 0.5f : 0.0f);
        F.mv.mh.unk0.x54 = 77;
        F.cmd_vars[2] = 5;
        F.self_vel.x = 3.0f; F.self_vel.y = 4.0f;
        RATE_SET = 0; RATE = -1.0f;
        printf("a1 %08x %d %d %d %08x", B(F.mv.mh.unk0.x50), DA.xEC, DA.xF0, PERCENT, B(DA.xF4));
        ftMh_FingerGun1_Anim(&G);
        printf(" %08x %d %u %d %08x %08x %08x\n", B(F.mv.mh.unk0.x50), F.mv.mh.unk0.x54, F.cmd_vars[2], RATE_SET,
               B(RATE), B(F.self_vel.x), B(F.self_vel.y));
    }
    /* FingerGun2_Anim countdown */
    for (i = 0; i < 100; i++) {
        FRAMES_LEFT = 0;
        DA.xEC = (int) (rnd() % 100);
        DA.xF4 = rf();
        PERCENT = (int) (rnd() % 120);
        F.mv.mh.unk0.x54 = (int) (rnd() % 4) + 1;
        RATE_SET = 0; RATE = -1.0f;
        printf("a2 %d %d %d %08x", F.mv.mh.unk0.x54, DA.xEC, PERCENT, B(DA.xF4));
        ftMh_FingerGun2_Anim(&G);
        printf(" %d %d %08x\n", F.mv.mh.unk0.x54, RATE_SET, B(RATE));
    }
    /* the bullet spawn: arg2/arg3 offsets, bool b = percent > xEC */
    for (j = 0; j < 100; j++) {
        DA.xEC = (int) (rnd() % 100);
        PERCENT = (int) (rnd() % 120);
        DA.xF8 = rf(); DA.xFC = rf(); DA.x100 = rf(); DA.x104 = rf(); DA.xD4 = rf(); DA.xD8 = rf();
        F.facing_dir = (rnd() % 2) ? 1.0f : -1.0f;
        F.cmd_vars[0] = 1; F.cmd_vars[1] = 0;
        ftMh_MS_365_801535B0(&G);
        printf("bl %d %d %08x %08x %08x %08x %08x %08x %08x %08x %08x %08x %08x %u\n", DA.xEC, PERCENT,
               B(DA.xF8), B(DA.xFC), B(DA.xD4), B(DA.xD8), B(F.facing_dir), B(BUL[0]), B(BUL[1]), B(BUL[2]),
               B(BUL[3]), B(BUL[4]), B(BUL[5]), F.cmd_vars[0]);
    }
    return 0;
}
