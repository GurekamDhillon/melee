/* TU: src/melee/ft/kinds/ftCommon/ftCo_Damage.c */
/* C oracle for src/tu/ftCo_Damage.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. The Fighter and ftCommonData are filled word for word from the xorshift
 * (mirrored by tests/tu_ftCo_Damage.rs), the few fields that steer control flow or are pointers
 * are then overridden. The state hash covers the whole of both Fighters (pointer fields zeroed,
 * every NaN canonical). Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

#define NOPS 36

static u32 BN(float f) { return f != f ? 0x7FC00000u : B(f); }
static u32 chk(int id) { return (SRV[10 + id % 6] >> (id % 7)) & 1; }
static u32 GOK(Fighter_GObj* g) { return (g == &GO) | ((g == &GO2) << 1) | ((g == &IGO) << 2); }

struct Fighter_804D6524_t* Fighter_804D6524;
static struct Fighter_804D6524_t D6524;
static FighterBone PARTS[8];
static ftData FTD;
static FtSFX SFXD;
static u8 JTB[4];

#define DS_V(name, id) void name(Fighter_GObj* g) { T(id); T(GOK(g)); }
#define DS_CHK(name, id) bool name(Fighter_GObj* g) { T(id); T(GOK(g)); return chk(id); }
#define DS_FV(name, id) void name(Fighter* f) { T(id); T(f == &F); }

void ScriptGame_ArmorResetReaction(Fighter* f) { T(1); T(f == &F); }
float ScriptGame_ArmorSubtract(Fighter* f, float kb) { T(2); T(f == &F); T(BN(kb)); return (rnd() & 1) ? rg() : kb; }
int ScriptGame_ArmorReact(Fighter* f) { T(3); T(f == &F); return chk(3); }
int ScriptGame_CapsNoFlinch(Fighter* f) { T(4); T(f == &F); return chk(4); }
int ScriptGame_ShockTake(Fighter* f, int base) { T(5); T(f == &F); T(base); return (int) (rnd() % 5) - 2; }
int Geno_TakeStunBonus(Fighter* f) { T(6); T(f == &F); return (int) (rnd() % 7) - 3; }
void diag_geno_stun(int port, int base, int bonus) { T(7); T(port); T(base); T(bonus); }
int Mex_Enabled(const char* n) { (void) n; T(8); return chk(8); }
int Replay_UcfVersion(int port) { T(9); T(port); return (SRV[3] & 1) ? 84 : 83; }
int ftUcf_PadRaw(int port, int which_y, int back) { T(10); T(port); T(which_y); T(back); return (int) (rnd() % 256) - 128; }
bool ftColl_8007AC68(u32 a) { T(11); T(a); return chk(11); }
float ftCo_CalcYScaledKnockback(float a, float s, float b) { T(12); T(BN(a)); T(BN(s)); T(BN(b)); return rg(); }
bool ftCo_800BFFD0(Fighter* f, enum_t a, bool b) { T(13); T(f == &F); T(a); T(b); return chk(13); }
void lbBgFlash_80021C48(u32 a, u32 b) { T(14); T(a); T(b); }
void efAsync_Spawn(HSD_GObj* g, void* q, u32 k, u32 gfx, HSD_JObj* j, ...)
{
    __builtin_va_list ap;
    float* a;
    __builtin_va_start(ap, j);
    a = __builtin_va_arg(ap, float*);
    __builtin_va_end(ap);
    T(15); T(GOK(g)); T(q == (void*) &F.x60C); T(k); T(gfx); T(j == (HSD_JObj*) JTB); T(BN(*a));
}
void ftPartSetRotX(Fighter* f, int idx, f32 v) { T(16); T(f == &F); T(idx); T(BN(v)); }
/* GetBoneIndex and atan2f are the two calls in one argument list (inlineA1): the shipping PPC build
 * evaluates left to right, this x86-64 MSVC-target oracle right to left. Commutative trace, no draws. */
static void TC(u32 v) { TR += v * 0x9E3779B1u; }
Fighter_Part ftParts_GetBoneIndex(Fighter* f, Fighter_Part part) { TC(17u + (u32) (f == &F) * 3u + (u32) part * 131u); return ((u32) part * 7u + 3u) % 20u; }
void Fighter_8006CDA4(Fighter* f, s32 v) { T(18); T(f == &F); T(v); }
void pl_80040270(int a, int b, float c) { T(19); T(a); T(b); T(BN(c)); }
void pl_800401F0(int a, int b, float c, float d) { T(20); T(a); T(b); T(BN(c)); T(BN(d)); }
float lbVector_Angle(Vec* a, Vec* b)
{
    T(21); T(a == &F.coll_data.floor.normal); T(BN(a->x)); T(BN(a->y)); T(BN(b->x)); T(BN(b->y)); T(BN(b->z));
    return rg() * 2.0f;
}
void ftCommon_8007D5D4(Fighter* f) { T(22); T(f == &F); }
float HSD_Randf(void) { T(23); return (float) (rnd() % 1000) / 1000.0f; }
float un_803222EC(float a, float b) { T(24); T(BN(a)); T(BN(b)); return rg(); }
void un_8032233C(u32 a, u32 b) { T(25); T(a); T(b); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(26); T(GOK(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
DS_V(ftAnim_8006EBA4, 27)
bool gm_8016B014(void) { T(28); return chk(28); }
DS_V(ftColl_8007B6A0, 29)
void ftCommon_8007EFC0(Fighter* f, u32 v) { T(30); T(f == &F); T(v); }
void Camera_RequestQuake(CmQuakeKind k, Vec* p) { T(31); T(k); T(BN(p->x)); T(BN(p->y)); T(BN(p->z)); }
DS_V(ftCommon_8007F824, 32)
DS_V(ftCo_DamageIce_Init, 33)
bool ftCo_800DF608(Fighter* f) { T(34); T(f == &F); return chk(34); }
void ftColl_8007B7A4(Fighter_GObj* g, int v) { T(35); T(GOK(g)); T(v); }
void ftCo_800C318C(Fighter_GObj* g, bool v) { T(36); T(GOK(g)); T(v); }
DS_V(ftCo_800C8D00, 37)
DS_V(ftCo_800DC284, 38)
DS_V(ftCo_800DC3A4, 39)
DS_V(ftCo_800C0408, 40)
DS_FV(ftCommon_800804FC, 41)
DS_CHK(ftCo_800C3538, 42)
DS_CHK(ftCo_800C44CC, 43)
DS_CHK(ftCo_800D2FA4, 44)
DS_V(ftCo_800DE854, 45)
void ftCo_800DCE34(Fighter_GObj* a, Fighter_GObj* b) { T(46); T(GOK(a)); T(GOK(b)); }
DS_V(ftCommon_8007DB58, 47)
float ftCommon_CalcHitlag(int dmg, FtMotionId m, float mul) { T(48); T(dmg); T(m); T(BN(mul)); return rg() * 10.0f; }
DS_V(ftCo_800DE2F0, 49)
DS_V(ftCo_800DCFD4, 50)
bool it_8026B2D8(Item_GObj* g) { T(51); T(g == (Item_GObj*) &IGO); return chk(51); }
DS_V(ftDk_MS_349_800E06D8, 52)
DS_CHK(ftCo_800C0CB8, 53)
DS_V(ftCo_DamageIce_HitWhileFrozen, 54)
DS_CHK(ftCo_800C74F4, 55)
DS_V(ftCo_800C3598, 56)
DS_V(ftCommon_8007F86C, 57)
DS_CHK(ftCo_800C5240, 58)
DS_FV(ftCo_800C554C, 59)
DS_CHK(ftCo_800CB870, 60)
DS_CHK(ftAnim_IsFramesRemaining, 61)
DS_V(ftCo_80090780, 62)
DS_V(ftCo_Fall_Enter, 63)
DS_V(ft_8008A2BC, 64)
DS_CHK(ftCo_800D69C4, 65)
DS_CHK(ftCo_800CB8E0, 66)
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(67); T(f == &F); T(a); T(b); }
ftCo_JumpInput ftCo_Jump_GetInput(Fighter_GObj* g) { T(68); T(GOK(g)); return chk(68) ? (ftCo_JumpInput) (1 + rnd() % 3) : (ftCo_JumpInput) 0; }
DS_V(ftCo_HammerFall_IASA, 69)
DS_V(ftCo_HammerWait_IASA, 70)
DS_CHK(ftCo_Fall_IASA_Inner, 71)
DS_V(ftCo_Wait_IASA, 72)
DS_V(ft_80084DB0, 73)
DS_V(ft_80084EEC, 74)
DS_V(ft_80084F3C, 75)
void ft_800848DC(Fighter_GObj* g, HSD_GObjEvent cb) { T(76); T(GOK(g)); T(cb == ftCo_8008FC94); }
DS_CHK(ft_80081DD4, 77)
DS_V(ftCo_80097D40, 78)
DS_V(ftCo_Landing_Enter_Basic, 79)
DS_FV(ftCommon_8007D7FC, 80)
DS_V(ftColl_8007AFF8, 81)
DS_CHK(ftCo_800C1D38, 82)
DS_CHK(ftCo_800C23A0, 83)
DS_CHK(ftCo_800C17CC, 84)
DS_CHK(ftCo_80098928, 85)
DS_CHK(ftCo_8009872C, 86)
DS_CHK(ftCo_8009F0F0, 87)
DS_V(ftCo_DamageFall_IASA, 88)
void PSVECCrossProduct(Vec* a, Vec* b, Vec* out)
{
    T(89); T(BN(a->x)); T(BN(a->y)); T(BN(b->x)); T(BN(b->y));
    out->x = 0; out->y = 0; out->z = rg();
}
float atan2f(float y, float x)
{
    u32 h = BN(y) * 0x45D9F3Bu ^ BN(x) * 0x27D4EB2Fu;
    TC(91u + h);
    return (float) ((int) ((h >> 8) % 41u) - 20) / 20.0f;
}
static void pre_cb(HSD_GObj* g) { T(90); T(GOK(g)); }

/* ---- fill / hash (mirrored by tests/tu_ftCo_Damage.rs) ---- */
static float gf(void)
{
    union { float f; u32 u; } c;
    switch (rnd() % 32) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: c.u = 0x7FC00000u; return c.f;
    case 3: return 1.0f;
    case 4: return -1.0f;
    default: return (float) ((int) (rnd() % 41) - 20) / 20.0f;
    }
}
static u32 gw(void)
{
    switch (rnd() % 8) {
    case 0: case 1: case 2: return B(gf());
    case 3: return B(rw());
    case 4: return rnd() % 64;
    case 5: return 0;
    case 6: return rnd();
    default: return B(gf() * 3.0f);
    }
}
static void fill_words(void* p, unsigned bytes)
{
    u32* w = p;
    unsigned i;
    for (i = 0; i < bytes / 4; i++) w[i] = gw();
}
static u32 hw(u32 w)
{
    if ((w & 0x7F800000u) == 0x7F800000u && (w & 0x7FFFFFu)) return 0x7FC00000u;
    return w;
}
static const int MSL[] = { ftCo_MS_Squat, ftCo_MS_SquatWait, ftCo_MS_DamageIce, ftCo_MS_JumpF, ftCo_MS_JumpB,
    ftCo_MS_JumpAerialF, ftCo_MS_JumpAerialB, ftCo_MS_Fall, ftCo_MS_FallF, ftCo_MS_FallB, ftCo_MS_FallAerial,
    ftCo_MS_FallAerialF, ftCo_MS_FallAerialB, ftCo_MS_FallSpecial, ftCo_MS_FallSpecialF, ftCo_MS_FallSpecialB,
    ftCo_MS_DamageFall, ftCo_MS_EscapeAir, 0xe0, 0xe1, 0xe3, 0xe4, 0x145, ftCo_MS_DamageFlyRoll, ftCo_MS_DamageFlyRoll };
#define NMSL ((int) (sizeof MSL / sizeof MSL[0]))

static unsigned BF;
static void bf_probe(void)
{
    static Fighter z;
    unsigned i;
    zero(&z, sizeof z);
    z.x2218_b1 = 1;
    for (i = 0; i < sizeof z; i++)
        if (((unsigned char*) &z)[i]) BF = i;
}

static void fill_fighter(Fighter* p, HSD_GObj* own)
{
    fill_words(p, sizeof(Fighter));
    p->gobj = own;
    p->parts = PARTS;
    p->ft_data = &FTD;
    p->victim_gobj = NULL;
    p->item_gobj = NULL;
    p->hitlag_cb = NULL;
    p->pre_hitlag_cb = NULL;
    p->post_hitlag_cb = NULL;
    p->x1064_thrownHitbox.owner = NULL;
    p->dmg.x190C = NULL;
    p->motion_id = (rnd() % 4 == 0) ? (int) (rnd() % 400) : MSL[rnd() % NMSL];
    p->ground_or_air = rnd() & 1;
    p->dmg.x1860_element = rnd() % 17;
    p->dmg.x184c_damaged_hurtbox = rnd() % 3;
    p->dmg.x1848_kb_angle = (rnd() & 1) ? 361 : (int) (rnd() % 400) - 20;
    p->dmg.kb_applied = (rnd() % 5 == 0) ? 0.0f : rw();
    p->dmg.x1838_percentTemp = (rnd() % 4 == 0) ? 0.0f : rw();
    p->dmg.x18ac_time_since_hit = rnd() % 12;
    p->dmg.facing_dir_1 = (rnd() & 1) ? 1.0f : -1.0f;
    p->facing_dir = (rnd() % 4 == 0) ? rg() : ((rnd() & 1) ? 1.0f : -1.0f);
    p->x618_player_id = rnd() % 4;
    p->player_id = rnd() % 4;
    p->active_timer.lstick.x = rnd() % 8; p->active_timer.lstick.y = rnd() % 8;
    p->active_sticky.lstick.x = rnd() % 4; p->active_sticky.lstick.y = rnd() % 4;
    p->x680 = rnd() % 256; p->x684 = rnd() % 256;
    p->smash_attrs.state = rnd() % 4;
    p->x34_scale.y = (rnd() & 1) ? 1.0f : rg();
    p->input.held_buttons[0] = (rnd() & 1) ? HSD_PAD_LR : 0;
    p->input.pressed_buttons = (rnd() & 1) ? rnd() : 0;
    p->x1828 = 0;
    p->allow_sdi = (rnd() & 1);
    p->is_metal = (rnd() & 1);
    p->is_sandbag = (rnd() & 1);
    p->is_sub_fighter = (rnd() & 1);
    p->x2219_b5 = (rnd() & 1);
    p->x221A_b3 = (rnd() & 1);
    p->x221B_b5 = (rnd() & 1);
    p->x221C_b0 = (rnd() & 1);
    p->x221C_b6 = (rnd() & 1);
    p->x221D_b7 = (rnd() & 1);
    p->x2220_b3 = (rnd() % 4 == 0);
    p->x2220_b4 = (rnd() % 4 == 0);
    p->x2221_b1 = (rnd() & 1);
    p->x2222_b0 = (rnd() & 1);
    p->x2224_b2 = (rnd() & 1);
    p->x2227_b4 = (rnd() & 1);
}

static void hash_fighter(Fighter* p)
{
    static Fighter t;
    u32* w = (u32*) &t;
    u32* s = (u32*) p;
    unsigned i;
    for (i = 0; i < sizeof(Fighter) / 4; i++) w[i] = s[i];
    HM((p->hitlag_cb == ftCo_Damage_OnEveryHitlag) ? 1u : (p->hitlag_cb == pre_cb) ? 2u : p->hitlag_cb ? 3u : 0u);
    HM((p->post_hitlag_cb == ftCo_Damage_OnExitHitlag) ? 1u : (p->post_hitlag_cb == pre_cb) ? 2u : p->post_hitlag_cb ? 3u : 0u);
    HM((p->pre_hitlag_cb == pre_cb) ? 2u : p->pre_hitlag_cb ? 3u : 0u);
    HM((u32) (unsigned long long) p->dmg.x190C);
    t.gobj = NULL; t.parts = NULL; t.ft_data = NULL; t.victim_gobj = NULL; t.item_gobj = NULL;
    t.hitlag_cb = NULL; t.pre_hitlag_cb = NULL; t.post_hitlag_cb = NULL;
    t.x1064_thrownHitbox.owner = NULL;
    t.dmg.x190C = NULL;
    for (i = BF; i < BF + 19; i++) ((unsigned char*) &t)[i] = 0;
    HM(p->allow_sdi);
    HM(p->is_metal);
    HM(p->is_sandbag);
    HM(p->is_sub_fighter);
    HM(p->x2219_b5);
    HM(p->x221A_b3);
    HM(p->x221B_b5);
    HM(p->x221C_b0);
    HM(p->x221C_b6);
    HM(p->x221D_b7);
    HM(p->x2220_b3);
    HM(p->x2220_b4);
    HM(p->x2221_b1);
    HM(p->x2222_b0);
    HM(p->x2224_b2);
    HM(p->x2227_b4);
    for (i = 0; i < sizeof(Fighter) / 4; i++) HM(hw(w[i]));
}

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    hash_fighter(&F);
    hash_fighter(&F2);
    return H;
}

static u32 run(int op)
{
    switch (op) {
    case 0: return BN(ftCo_Damage_CalcAngle(&F, rw()));
    case 1: return BN(ftCo_ScaleBy154(rw()));
    case 2: return ftCo_8008D8E8(rw());
    case 3: ftCo_Damage_CalcKnockback(&F); break;
    case 4: { u32 el = rnd() % 17; u32 a = rnd() % 4; return ftCo_8008DA4C(&GO, el, a); }
    case 5: { u32 el = rnd() % 17; float v = rw(); ftCo_8008DB10(&GO, el, v); break; }
    case 6: { float x = rw(); float y = rw(); ftCo_Damage_CalcVel(&F, x, y); break; }
    case 7: return BN(calcAngle(rw()));
    case 8: { int a1 = (rnd() & 1) ? -1 : (int) (rnd() % 0x150); float fd = (rnd() & 1) ? 0.0f : rg(); ftCo_8008DCE0(&GO, a1, fd); break; }
    case 9: return ftCo_Damage_CheckAirMotion(&F);
    case 10: ftCo_Damage_OnEveryHitlag(&GO); break;
    case 11: ftCo_8008E5A4(&F); break;
    case 12: ftCo_Damage_OnExitHitlag(&GO); break;
    case 13: { float fd = rg(); ftCo_8008E908(&GO, fd); break; }
    case 14: return ftCo_8008E984(&F);
    case 15: ftCo_8008E9D0(&GO); break;
    case 16: ftCo_8008EB58(&GO); break;
    case 17: ftCo_8008EC90(&GO); break;
    case 18: ftCo_8008F744(&GO); break;
    case 19: ftCo_Damage_Anim(&GO); break;
    case 20: return doIasa(&GO);
    case 21: ftCo_Damage_IASA(&GO); break;
    case 22: ftCo_Damage_Phys(&GO); break;
    case 23: ftCo_Damage_Coll(&GO); break;
    case 24: ftCo_8008FC94(&GO); break;
    case 25: ftCo_Damage_SetMv8FromKbThreshold(&F); break;
    case 26: ftCo_DamageFly_Anim(&GO); break;
    case 27: ftCo_DamageFly_IASA(&GO); break;
    case 28: ftCo_DamageFly_Phys(&GO); break;
    case 29: ftCo_DamageFly_Coll(&GO); break;
    case 30: ftCo_80090184(&GO); break;
    case 31: ftCo_DamageFlyRoll_Anim(&GO); break;
    case 32: ftCo_DamageFlyRoll_IASA(&GO); break;
    case 33: ftCo_DamageFlyRoll_Phys(&GO); break;
    case 34: ftCo_DamageFlyRoll_Coll(&GO); break;
    case 35: return ftCo_Ucf084Sdi(&F);
    }
    return 0;
}

int main(void)
{
    int op, k;
    tu_init();
    bf_probe();
    Fighter_804D6524 = &D6524;
    PARTS[0].joint = (HSD_JObj*) JTB;
    FTD.x4C_sfx = &SFXD;
    for (op = 0; op < NOPS; op++) {
        int n = (op == 17) ? 6000 : (op == 8) ? 4000 : 800;
        for (k = 0; k < n; k++) {
            u32 ret;
            tu_case(op, k);
            fill_words(&CD, sizeof CD);
            fill_words(&D6524, sizeof D6524);
            fill_words(&SFXD, sizeof SFXD);
            fill_fighter(&F, &GO);
            fill_fighter(&F2, &GO2);
            F.gobj = &GO; F2.gobj = &GO2;
            if (rnd() & 1) F.victim_gobj = &GO2;
            if (rnd() % 3 == 0) F.item_gobj = (Item_GObj*) &IGO;
            if (rnd() & 1) F.x1064_thrownHitbox.owner = &GO2;
            if (rnd() & 1) F2.pre_hitlag_cb = pre_cb;
            if (op != 5 && op != 8 && F.dmg.x1838_percentTemp == 0.0f) F.dmg.x1838_percentTemp = 1.0f;
            ret = run(op);
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
