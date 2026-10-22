/* C reference for src/tu/ftmariospecials.rs: the whole game TU is #included verbatim; Fall and
 * CalcSelfAccel_Deaccel are cut from ftcommon.c by tu_shard345.py (shipping contraction via
 * run.py). Everything else it calls is a recording stand-in. Prints, per case, the wind puff
 * handed to lb_800119DC and the velocities after the cape's Phys callbacks. */
#include "ft/kinds/ftMario/ftmariospecials.c"
#include "tu_ftmariospecials_extract.inc"
#include <stdio.h>

float ScriptGame_ModValue(int slot, int field, float base) { (void) slot; (void) field; return base; }
f32 Geno_FallLimit(Fighter* fp, f32 terminal) { (void) fp; return terminal; }
ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static Vec3 HIP;
static int PUFF_N;
static Vec3 PUFF_POS;
static int PUFF_A;
static float PUFF_B, PUFF_C, PUFF_D;
static int REFLECT_N;
void lb_8000B1CC(HSD_JObj* j, Vec3* off, Vec3* out) { (void) j; (void) off; *out = HIP; }
void lb_800119DC(Vec3* pos, int a, float b, float c, float d)
{ PUFF_N++; PUFF_POS = *pos; PUFF_A = a; PUFF_B = b; PUFF_C = c; PUFF_D = d; }
int ftParts_GetBoneIndex(Fighter* fp, int i) { (void) fp; return i; }
void ftColl_CreateReflectHit(Fighter_GObj* g, ReflectDesc* d, HSD_GObjEvent x) { (void) g; (void) d; (void) x; REFLECT_N++; }
void ftColl_8007AEF8(HSD_GObj* g) { (void) g; }
void ft_80084F3C(HSD_GObj* g) { (void) g; }
void ftMr_Init_OnTakeDamage(HSD_GObj* g) { (void) g; }
void ftCommon_8007D5D4(Fighter* fp) { (void) fp; }
void ftCommon_8007D7FC(Fighter* fp) { (void) fp; }
void Fighter_ChangeMotionState(HSD_GObj* g, int m, u32 f, float a, float b, float c, HSD_GObj* d)
{ (void) g; (void) m; (void) f; (void) a; (void) b; (void) c; (void) d; }
void ftAnim_8006EBA4(HSD_GObj* g) { (void) g; }
int ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return 0; }
void ft_8008A2BC(HSD_GObj* g) { (void) g; }
void ftCo_Fall_Enter(HSD_GObj* g) { (void) g; }
int ft_800827A0(HSD_GObj* g) { (void) g; return 1; }
int ft_80081D0C(HSD_GObj* g) { (void) g; return 0; }
HSD_GObj* it_802B2560(HSD_GObj* g, float facing, Vec3* pos, int bone, int kind) { (void) g; (void) facing; (void) pos; (void) bone; (void) kind; return 0; }
void it_802B2674(HSD_GObj* g) { (void) g; }
void it_802B26C0(HSD_GObj* g) { (void) g; }
void it_802B26E0(HSD_GObj* g) { (void) g; }

static u32 s = 0x77C1E5u;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    switch (rnd() % 8) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    default: return (float) ((int) (rnd() % 4001) - 2000) / 97.0f;
    }
}
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

int main(void)
{
    static Fighter F;
    static HSD_GObj G;
    static ftMario_DatAttrs A;
    static FighterBone PARTS[80];
    int i;
    G.user_data = &F;
    F.parts = PARTS;
    F.dat_attrs = &A;
    for (i = 0; i < 4000; i++) {
        HIP.x = rf(); HIP.y = rf(); HIP.z = rf();
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        if (i & 1) {
            /* a position and facing where the product 3 * facing is exact but the fused sum matters
             * at the low bit of x: x's ulp is above the f32 rounding of the product anyway */
            HIP.x = (float) ((int) (rnd() % 200001) - 100000) / 3.0f;
        }
        F.cmd_vars[0] = rnd() % 3;
        PUFF_N = 0;
        ftMr_SpecialS_Phys(&G);
        printf("S %d %u %d %08x %08x %08x %d %08x %08x %08x %u\n", i, PUFF_N, PUFF_A, B(PUFF_POS.x), B(PUFF_POS.y),
               B(PUFF_POS.z), 0, B(PUFF_B), B(PUFF_C), B(PUFF_D), F.cmd_vars[0]);
        /* AirS */
        A.specials.vel.x = rf();
        A.specials.vel.y = rf();
        A.specials.grav = rf();
        A.specials.terminal_vel = rf();
        F.co_attrs.gravity = rf();
        F.co_attrs.terminal_velocity = rf();
        F.u.mr.x2238_isCapeBoost = rnd() & 1;
        F.cmd_vars[0] = rnd() % 3;
        F.self_vel.x = rf(); F.self_vel.y = rf();
        PUFF_N = 0;
        ftMr_SpecialAirS_Phys(&G);
        printf("A %d %u %08x %08x %08x %08x %08x %08x %08x %08x %d %u\n", i, PUFF_N, B(PUFF_POS.x), B(PUFF_POS.y),
               B(PUFF_POS.z), B(PUFF_B), B(PUFF_C), B(PUFF_D), B(F.self_vel.x), B(F.self_vel.y),
               (int) F.u.mr.x2238_isCapeBoost, F.cmd_vars[0]);
        /* reflect + Enter */
        F.cmd_vars[1] = rnd() % 3;
        F.mv.mr.SpecialS.reflecting = rnd() & 1;
        F.reflecting = 0;
        REFLECT_N = 0;
        F.cmd_vars[0] = 0;
        ftMr_SpecialS_Phys(&G);
        printf("R %d %d %d %d\n", i, REFLECT_N, (int) F.mv.mr.SpecialS.reflecting, (int) F.reflecting);
        F.self_vel.x = rf();
        A.specials.vel_x_decay = rf();
        ftMr_SpecialAirS_Enter(&G);
        printf("E %d %08x\n", i, B(F.self_vel.x));
    }
    return 0;
}
