/* C reference for src/tu/ft_0d4d.rs: the whole of src/melee/ft/ft_0D4D.c #included verbatim
 * (shipping contraction via tu_run.py). Tested: fn_800D54A4, fn_800D55B4, ftCo_Rebirth_Phys,
 * ftCo_RebirthWait_Phys, ftCo_800D4E50 (the float / coin math). ftCommon_800804EC and
 * ftCommon_8007F8B4 are cut verbatim from ftcommon.c; player / stage callees are stubs over
 * globals that tests/tu_ft_0d4d.rs mirrors. One line per case, hex bit patterns. */
#include "ft/ft_0D4D.c"

int printf(const char*, ...);
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
ftCommonData* p_ftCommonData;

void ftCommon_8007F8B4(Fighter* fp, Vec3* v)
{
    v->x = fp->cur_pos.x + fp->xD4_unk_vel.x;
    v->y = fp->cur_pos.y + fp->xD4_unk_vel.y;
    v->z = fp->cur_pos.z + fp->xD4_unk_vel.z;
}
float ftCommon_800804EC(Fighter* fp) { return fp->x40 * fp->x34_scale.x; }

static Vec3 g_stage, g_some;
static HSD_GObj* g_ent[2];
static int g_coins, g_9c, g_gm;
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
void Stage_80224E38(Vec3* v, s32 i) { (void) i; *v = g_stage; }
void Player_GetSomePos(s32 slot, Vec3* v) { (void) slot; *v = g_some; }
HSD_GObj* Player_GetEntityAtIndex(int slot, int i) { (void) slot; return g_ent[i]; }
int Player_GetCoins(int slot) { (void) slot; return g_coins; }
void Player_SetCoins(int slot, int v) { (void) slot; g_coins = v; }
s32 Player_GetUnk9C(s32 slot) { (void) slot; return g_9c; }
void Player_SetUnk9C(s32 slot, s32 v) { (void) slot; g_9c = v; }
bool gm_8016B0B4(void) { return g_gm; }
s32 it_802E5F8C(Item_GObj* g, Vec3* v, s32 a, s32 b, f32 c, f32 d)
{
    (void) g;
    printf(" IT %08x %08x %08x %d %d %08x %08x", B(v->x), B(v->y), B(v->z), a, b, B(c), B(d));
    return 0;
}

static u32 s;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return (float) (rnd() % 7) * 1e-4f;
    return ((float) ((int) (rnd() % 200001) - 100000)) / 37.0f;
}
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static Fighter F, O;
static HSD_GObj G, GO;
static HSD_JObj J;
static ftCommonData CD;

int main(void)
{
    int op, k;
    G.user_data = &F;
    GO.user_data = &O;
    p_ftCommonData = &CD;
    for (op = 0; op < 5; op++) {
        for (k = 0; k < 2000; k++) {
            s = 0xD4D0u + (u32) op * 7919u + (u32) k * 104729u;
            zero(&F, sizeof F);
            zero(&O, sizeof O);
            zero(&J, sizeof J);
            zero(&CD, sizeof CD);
            F.facing_dir = rf(); F.x40 = rf(); F.x34_scale.x = rf();
            F.cur_pos.x = rf(); F.cur_pos.y = rf(); F.cur_pos.z = rf();
            F.xD4_unk_vel.x = rf(); F.xD4_unk_vel.y = rf();
            F.mv.co.common.x4.x = rf(); F.mv.co.common.x4.y = rf();
            F.mv.co.common.x0 = (int) (rnd() % 21) - 10;
            F.smash_attrs.x2135 = (rnd() & 1) ? -1 : (s8) (rnd() % 4);
            F.is_sub_fighter = rnd() & 1;
            O.cur_pos.y = rf(); O.self_vel.x = rf(); O.self_vel.y = rf();
            O.x221F_b3 = rnd() & 1;
            g_stage.x = rf(); g_stage.y = rf(); g_stage.z = rf();
            g_some.x = rf(); g_some.y = rf(); g_some.z = rf();
            g_ent[0] = &GO;
            g_ent[1] = (rnd() & 1) ? &GO : NULL;
            J.flags = JOBJ_MTX_INDEP_SRT;
            F.x20A0_accessory = &J;
            g_coins = (int) (rnd() % 400) - 50;
            g_9c = (int) (rnd() % 100);
            g_gm = rnd() & 1;
            CD.x5C8 = (int) (rnd() % 200);
            CD.x5CC = rf();
            printf("%d %d", op, k);
            switch (op) {
            case 0: fn_800D54A4(&G); break;
            case 1: fn_800D55B4(&G); break;
            case 2: if (F.mv.co.common.x0 == 0) F.mv.co.common.x0 = 3; ftCo_Rebirth_Phys(&G); break;
            case 3: if (F.mv.co.common.x0 == 0) F.mv.co.common.x0 = 3; ftCo_RebirthWait_Phys(&G); break;
            case 4: { Vec3 v; float a = rf(); int b = rnd() & 1; v.x = rf(); v.y = rf(); v.z = rf();
                      ftCo_800D4E50(&F, &v, b, a); break; }
            }
            printf(" %08x %08x %08x %08x %08x %08x %08x %08x %08x %d %d\n", B(F.cur_pos.x), B(F.cur_pos.y),
                   B(F.self_vel.x), B(F.self_vel.y), B(F.mv.co.common.x4.x), B(F.mv.co.common.x4.y),
                   B(F.mv.co.common.x4.z), B(J.translate.x), B(J.translate.y), g_coins, g_9c);
        }
    }
    return 0;
}
