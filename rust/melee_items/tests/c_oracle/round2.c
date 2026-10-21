/* Round 2 (night 2) part of the melee_items C oracle; #included by oracle.c after round1.c.
 * Checks src/itlib.rs: the bodies in round2_bodies.h are verbatim copies of the decomp C.
 *
 * Each case: seed the round-2 LCG from (function id, case), fill an Item (round2_fields.h, the
 * same field list as tests/round2/fields.rs), fill the attribute/article/common data the
 * functions follow, draw an output Vec3 and the function's own arguments, call it through a
 * GObj whose user_data is the Item, then hash every filled field plus the return value and the
 * output Vec3. Prints one `[u32; N2]` of hashes per function. Built like the rest of the oracle:
 * contraction on + gwtool lowering (gen_expected.sh). */
#include <sysdolphin/baselib/gobj.h>
#undef GET_ITEM
#define GET_ITEM(gobj) ((Item*) HSD_GObjGetUserData(gobj))
#undef Item_GObj
#define Item_GObj HSD_GObj
#define it_804D6D28 it_804D6D28_o
/* The shipping build compiles with -isystem src/MSL, whose stdbool.h makes `bool` an `int`;
 * this oracle gets clang's (`_Bool`). The difference is observable in it_80274ECC (an int
 * argument stored to a u8) and it_802750E8 (returns `xDC0 & mask`), so the copies use int. */
#undef bool
#define bool int

#include "round2_bodies.h"

static u32 r2;
static u32 R(void) { r2 = r2 * 1103515245u + 12345u; return r2; }
static const u32 R2POOL[] = { 0x00000000u, 0x80000000u, 0x3F800000u, 0xBF800000u, 0x3F000000u,
    0xBF000000u, 0x40000000u, 0x00000001u, 0x7F7FFFFFu, 0x42C80000u };
static f32 RF(void)
{
    u32 b = R(), sel = R() % 8;
    if (sel == 0) return fbits(R2POOL[b % 10]);
    if (sel < 4) return (f32) ((s32) (b >> 8) - 0x7FFFFF) / 65536.0f;
    if (((b >> 23) & 0xFF) == 0xFF) b &= ~0x40000000u; /* no inf/NaN inputs */
    return fbits(b);
}
/* moderate values (|x| < 128, 16 fraction bits), where a fused multiply-add rounds differently */
static f32 RM(void) { return (f32) ((s32) (R() >> 8) - 0x7FFFFF) / 65536.0f; }
static const u32 DMG[] = { 0x00000000u, 0x80000000u, 0x3F800000u, 0x3F000000u };
static u32 H(u32 h, u32 v) { return (h ^ v) * 0x01000193u; }

static HSD_GObj gA, gB;
static HSD_GObj* PTR(u32 r) { return r % 3 == 0 ? NULL : (r % 3 == 1 ? &gA : &gB); }
static u32 PIDX(void* p) { return p == NULL ? 0 : (p == (void*) &gA ? 1 : (p == (void*) &gB ? 2 : 3)); }

static const s32 KINDS[] = { It_Kind_Heart, It_Kind_Tomato, It_Kind_Foods, It_Kind_Coin,
    It_Kind_Lucky_Egg, It_Kind_WhispyHealApple, It_Kind_L_Gun, It_Kind_S_Scope, It_Kind_StarRod,
    It_Kind_LipStick, It_Kind_F_Flower, It_Kind_Harisen, It_Kind_Freeze, It_Kind_MSBomb,
    It_Kind_Flipper, It_Kind_BombHei, It_Kind_Link_Bomb, It_Kind_Kuriboh, It_Kind_Octarock_Stone - 1,
    It_Kind_Octarock_Stone, It_Kind_Old_Kuri - 1, It_Kind_Old_Kuri, It_Kind_Arwing_Laser - 1,
    It_Kind_Arwing_Laser, It_Kind_Kyasarin_Egg, It_Kind_WhispyApple, 0, 100, 230 };
#define NKINDS ((u32) (sizeof KINDS / sizeof KINDS[0]))

/* GameCube-order ("canonical") values of the bitfield units, built field by field */
static u32 flag32_get(flag32* f)
{
    return (f->flags.x0 << 31) | (f->flags.x1 << 30) | (f->flags.x2 << 29) | (f->flags.x3 << 28) |
           (f->flags.x4 << 27) | (f->flags.x5 << 26) | (f->flags.x6 << 25) | (f->flags.x7 << 24) |
           (f->flags.x8 << 23) | (f->flags.x9 << 22) | (f->flags.xA << 21) | (f->flags.xB << 20) |
           (f->flags.xC << 19) | (f->flags.xD << 18) | (f->flags.xE << 17) | (f->flags.xF << 13) |
           (f->flags.x13 << 12) | (f->flags.x14 << 11) | (f->flags.x15 << 10) | (f->flags.x16 << 9) |
           (f->flags.x17 << 7) | (f->flags.x19 << 6) | (f->flags.x1A << 5) | (f->flags.x1B << 4) |
           (f->flags.x1C << 3) | (f->flags.x1D << 2) | (f->flags.x1E << 1) | f->flags.x1F;
}
static void flag32_set(flag32* f, u32 v)
{
    f->flags.x0 = v >> 31; f->flags.x1 = v >> 30; f->flags.x2 = v >> 29; f->flags.x3 = v >> 28;
    f->flags.x4 = v >> 27; f->flags.x5 = v >> 26; f->flags.x6 = v >> 25; f->flags.x7 = v >> 24;
    f->flags.x8 = v >> 23; f->flags.x9 = v >> 22; f->flags.xA = v >> 21; f->flags.xB = v >> 20;
    f->flags.xC = v >> 19; f->flags.xD = v >> 18; f->flags.xE = v >> 17; f->flags.xF = v >> 13;
    f->flags.x13 = v >> 12; f->flags.x14 = v >> 11; f->flags.x15 = v >> 10; f->flags.x16 = v >> 9;
    f->flags.x17 = v >> 7; f->flags.x19 = v >> 6; f->flags.x1A = v >> 5; f->flags.x1B = v >> 4;
    f->flags.x1C = v >> 3; f->flags.x1D = v >> 2; f->flags.x1E = v >> 1; f->flags.x1F = v;
}
static u32 ufs_get(UnkFlagStruct* f)
{
    return (f->b0 << 7) | (f->b1 << 6) | (f->b2 << 5) | (f->b3 << 4) | (f->b4 << 3) | (f->b5 << 2) |
           (f->b6 << 1) | f->b7;
}
static void ufs_set(UnkFlagStruct* f, u8 v)
{
    f->b0 = v >> 7; f->b1 = v >> 6; f->b2 = v >> 5; f->b3 = v >> 4;
    f->b4 = v >> 3; f->b5 = v >> 2; f->b6 = v >> 1; f->b7 = v;
}
static u32 flags_daa_get(Item* ip) { return ufs_get(&ip->xDAA_flag); }
static void flags_daa_set(Item* ip, u8 v) { ufs_set(&ip->xDAA_flag, v); }
static u32 dcc_get(Item* ip)
{
    return (ip->xDCC_flag.b0 << 7) | (ip->xDCC_flag.b1 << 6) | (ip->xDCC_flag.b2 << 5) |
           (ip->xDCC_flag.b3 << 4) | ip->xDCC_flag.b4567;
}
static void dcc_set(Item* ip, u8 v)
{
    ip->xDCC_flag.b0 = v >> 7; ip->xDCC_flag.b1 = v >> 6; ip->xDCC_flag.b2 = v >> 5;
    ip->xDCC_flag.b3 = v >> 4; ip->xDCC_flag.b4567 = v;
}
static u32 dcd_get(Item* ip)
{
    CameraBoxFlags* f = &ip->xDCD_flag;
    return (f->b01 << 6) | (f->b2 << 5) | (f->b3 << 4) | (f->b4 << 3) | (f->b5 << 2) | (f->b6 << 1) | f->b7;
}
static void dcd_set(Item* ip, u8 v)
{
    CameraBoxFlags* f = &ip->xDCD_flag;
    f->b01 = v >> 6; f->b2 = v >> 5; f->b3 = v >> 4; f->b4 = v >> 3; f->b5 = v >> 2; f->b6 = v >> 1; f->b7 = v;
}
static u32 hit_x40_get(HitCapsule* h)
{
    return (h->x40_b0 << 15) | (h->x40_b1 << 14) | (h->x40_b2 << 13) | (h->x40_b3 << 12) |
           (h->x40_b4 << 4) | (h->x41_b4 << 3) | (h->x41_b5 << 2) | (h->x41_b6 << 1) | h->x41_b7;
}
static void hit_x40_set(HitCapsule* h, u16 v)
{
    h->x40_b0 = v >> 15; h->x40_b1 = v >> 14; h->x40_b2 = v >> 13; h->x40_b3 = v >> 12;
    h->x40_b4 = v >> 4; h->x41_b4 = v >> 3; h->x41_b5 = v >> 2; h->x41_b6 = v >> 1; h->x41_b7 = v;
}
static u32 hurt_x24_get(HurtCapsule* h)
{
    return (h->skip_update_pos << 7) | (h->x24_b1 << 6) | (h->x24_b2 << 5) | (h->x24_b3 << 4) |
           (h->x24_b4 << 3) | (h->x24_b5 << 2) | (h->x24_b6 << 1) | h->x24_b7;
}
static void hurt_x24_set(HurtCapsule* h, u8 v)
{
    h->skip_update_pos = v >> 7; h->x24_b1 = v >> 6; h->x24_b2 = v >> 5; h->x24_b3 = v >> 4;
    h->x24_b4 = v >> 3; h->x24_b5 = v >> 2; h->x24_b6 = v >> 1; h->x24_b7 = v;
}

#include "round2_fields.h"

static ItemAttr r2_attr;
static Article r2_art;
static itUnkAttributes r2_unk;
static ItemCommonData r2_common;
static void fill_ext(Item* ip)
{
    memset(&r2_attr, 0, sizeof r2_attr); memset(&r2_art, 0, sizeof r2_art);
    memset(&r2_unk, 0, sizeof r2_unk); memset(&r2_common, 0, sizeof r2_common);
    r2_attr.x0_is_heavy = R(); r2_attr.x0_78 = R(); r2_attr.x0_hold_kind = R();
    r2_attr.x1C_damage_mul = RF();
    r2_unk.x0_float = RF(); r2_unk.x4_float = RF();
    r2_art.x4_specialAttributes = &r2_unk;
    r2_art.x8_hurtbones = (R() & 1) ? (void*) &gA : NULL;
    r2_common.xB8 = RF(); r2_common.xBC = RF(); r2_common.xDC = (s32) R();
    ip->xCC_item_attr = &r2_attr; ip->xC4_article_data = &r2_art;
    it_804D6D28_o = &r2_common;
}

#define N2 48
static Item r2_it;
static HSD_GObj r2_g;
/* T(id, NAME, body): body sees ip, g (the GObj), ov (output Vec3), ret (u32, hashed) */
#define T(ID, NAME, ...)                                                                       \
    do {                                                                                       \
        int c;                                                                                 \
        printf("pub const " NAME ": [u32; %d] = [", N2);                                       \
        for (c = 0; c < N2; c++) {                                                             \
            Item* ip = &r2_it; HSD_GObj* g = &r2_g; Vec3 ov; u32 ret = 0;                      \
            r2 = 0x2545F491u ^ ((u32) (ID) * 0x9E3779B9u) ^ ((u32) c * 0x85EBCA6Bu);          \
            fill(ip); fill_ext(ip);                                                            \
            memset(g, 0, sizeof *g); g->user_data = ip;                                        \
            ov.x = RF(); ov.y = RF(); ov.z = RF();                                             \
            __VA_ARGS__;                                                                          \
            printf("%s0x%08X", c ? ", " : "",                                                  \
                   H(H(H(H(snap(ip), ret), bits(ov.x)), bits(ov.y)), bits(ov.z)));             \
        }                                                                                      \
        printf("];\n");                                                                        \
    } while (0)
#define HURT_IN(hc) (memset(&(hc), 0, sizeof (hc)), (hc).a_offset.x = RF(), (hc).a_offset.y = RF(), \
    (hc).a_offset.z = RF(), (hc).b_offset.x = RF(), (hc).b_offset.y = RF(), (hc).b_offset.z = RF(), \
    (hc).scale = RF())
#define HURT_H(hc) H(H(H(H(H(H(H(0, bits((hc).a_offset.x)), bits((hc).a_offset.y)), bits((hc).a_offset.z)), \
    bits((hc).b_offset.x)), bits((hc).b_offset.y)), bits((hc).b_offset.z)), bits((hc).scale))

static void round2(void)
{
    /* it_26B1.c */
    T(1, "R2_8026B294", it_8026B294(g, &ov));
    T(2, "R2_ISHEAVY", ret = itIsHeavy(g));
    T(3, "R2_8026B2D8", ret = it_8026B2D8(g));
    T(4, "R2_GETKIND", ret = itGetKind(g));
    T(5, "R2_8026B30C", ret = it_8026B30C(g));
    T(6, "R2_GETHOLDKIND", ret = itGetHoldKind(g));
    T(7, "R2_DAMAGEMUL", ret = bits(itGetDamageMultiplier(g)));
    T(8, "R2_8026B344", { ip->facing_dir = RM(); ip->xBCC_unk.x = RM(); ip->pos.x = RM(); it_8026B344(g, &ov); });
    T(9, "R2_GRABX", ret = bits(itGetGrabRangeX(g)));
    T(10, "R2_GRABY", ret = bits(itGetGrabRangeY(g)));
    T(11, "R2_8026B390", it_8026B390(g));
    T(12, "R2_8026B3A8", it_8026B3A8(g));
    T(13, "R2_8026B424", { int d = (R() & 1) ? (s32) (R() % 1000) - 100 : (s32) R(); ret = bits(it_8026B424(d)); });
    T(14, "R2_8026B47C", ret = it_8026B47C(g));
    T(15, "R2_8026B4F0", ret = it_8026B4F0(g));
    T(16, "R2_8026B54C", ret = bits(it_8026B54C(g)));
    T(17, "R2_8026B560", ret = bits(it_8026B560(g)));
    T(18, "R2_8026B574", ret = bits(it_8026B574(g)));
    T(19, "R2_8026B588", ret = it_8026B588());
    T(20, "R2_8026B594", ret = it_8026B594(g));
    T(21, "R2_8026B6C8", ret = it_8026B6C8(g));
    T(22, "R2_8026B718", it_8026B718(g, RF()));
    T(23, "R2_8026B724", it_8026B724(g));
    T(24, "R2_8026B73C", it_8026B73C(g));
    T(25, "R2_8026B774", ret = it_8026B774(g, (u8) (R() % 32)));
    T(26, "R2_MOTIONID", ret = itGetMotionId(g));
    T(27, "R2_TEAMID", ret = itGetTeamId(g));
    T(28, "R2_8026B7BC", ret = it_8026B7BC(g));
    T(29, "R2_8026B7CC", ret = it_8026B7CC(g));
    T(30, "R2_8026B7E8", ret = it_8026B7E8(g));
    T(31, "R2_8026B894", ret = it_8026B894(g, PTR(R())));
    T(32, "R2_8026B924", ret = it_8026B924(g));
    T(33, "R2_8026B960", ret = bits(it_8026B960(g)));
    T(34, "R2_8026BB88", it_8026BB88(g, &ov));
    T(35, "R2_8026BBCC", it_8026BBCC(g, &ov));
    T(36, "R2_8026BC68", ret = it_8026BC68(g));
    T(37, "R2_ATTACKID", ret = itGetAttackId(g));
    T(38, "R2_8026BCF4", it_8026BCF4(g));
    T(39, "R2_8026BD0C", it_8026BD0C(g));
    T(40, "R2_8026BD24", it_8026BD24(g));
    T(41, "R2_8026BD3C", it_8026BD3C(g));
    T(42, "R2_8026BD54", it_8026BD54(g));
    T(43, "R2_8026BD6C", it_8026BD6C(g));
    T(44, "R2_8026BD84", it_8026BD84(g));
    T(45, "R2_8026BD9C", it_8026BD9C(g));
    T(46, "R2_8026BDCC", it_8026BDCC(g));
    T(47, "R2_8026BE28", it_8026BE28(g));
    T(48, "R2_8026C1E8", ret = it_8026C1E8(g));
    T(49, "R2_8026C334", it_8026C334(g, &ov));
    /* it_2725.c */
    T(50, "R2_80273030", ret = it_80273030(g));
    T(51, "R2_80273130", ret = it_80273130(g));
    T(52, "R2_8027346C", it_8027346C(g));
    T(53, "R2_80273484", it_80273484(g));
    T(54, "R2_8027349C", it_8027349C(g));
    T(55, "R2_802734B4", it_802734B4(g));
    T(56, "R2_80274C60", it_80274C60(g));
    T(57, "R2_80274C78", ret = it_80274C78(g));
    T(58, "R2_80274C88", it_80274C88(g));
    T(59, "R2_80274D6C", it_80274D6C(g));
    T(60, "R2_80274DAC", it_80274DAC(g));
    T(61, "R2_80274DFC", it_80274DFC(g));
    T(62, "R2_80274E44", it_80274E44(g));
    T(63, "R2_80274ECC", it_80274ECC(g, (bool) (R() % 3 == 0 ? (s32) R() : (s32) (R() & 1))));
    T(64, "R2_80274EF8", it_80274EF8(g));
    T(65, "R2_80274F10", it_80274F10(g));
    T(66, "R2_802750E8", ret = it_802750E8(g, (s32) R()));
    T(67, "R2_802751D8", ret = it_802751D8(g));
    T(68, "R2_80275210", it_80275210(g));
    T(69, "R2_80275228", it_80275228(g));
    T(70, "R2_80275240", it_80275240(g));
    T(71, "R2_80275258", it_80275258(g));
    T(72, "R2_80275270", it_80275270(g));
    T(73, "R2_802753BC", it_802753BC(g, (s16) R()));
    /* ithitbox.c */
    T(74, "R2_802753DC", it_802753DC(g));
    T(75, "R2_80275414", it_80275414(g));
    T(76, "R2_8027542C", it_8027542C(g));
    T(77, "R2_80275444", it_80275444(g));
    T(78, "R2_80275474", it_80275474(g));
    T(79, "R2_802754A4", it_802754A4(g));
    T(80, "R2_802754BC", it_802754BC(g));
    T(81, "R2_802754D4", it_802754D4(g));
    T(82, "R2_80275504", it_80275504(g));
    T(83, "R2_80275534", it_80275534(g, RF()));
    T(84, "R2_80275594", { s32 idx = (s32) (R() % 4); it_80275594(g, idx, RF()); });
    T(85, "R2_802755C0", it_802755C0(g, RF()));
    T(86, "R2_802756D0", it_802756D0(g));
    T(87, "R2_8027570C", it_8027570C(g, (s32) (R() % 4)));
    T(88, "R2_8027572C", it_8027572C(g, (s32) (R() % 4)));
    T(89, "R2_80275788", it_80275788(g));
    T(90, "R2_80275820", { Vec3 a, b; s32 idx; a.x = RF(); a.y = RF(); a.z = RF(); b.x = RF(); b.y = RF(); b.z = RF();
                           idx = (s32) (R() % 4); it_80275820(g, &a, &b, idx); });
    T(91, "R2_80275870", ret = it_80275870(g));
    T(92, "R2_802758D4", ret = bits(it_802758D4(g)));
    T(104, "R2_802758D4_ZEROS", { int k; for (k = 0; k < 4; k++) ip->x5D4_hitboxes[k].hit.damage = fbits(DMG[R() % 4]);
                                  ret = bits(it_802758D4(g)); });
    /* itmaplib.c */
    T(93, "R2_802762D8", ret = it_802762D8(g));
    T(94, "R2_80276308", ret = it_80276308(g));
    T(95, "R2_80276348", ret = it_80276348(g, &ov));
    T(96, "R2_802763B8", ret = it_802763B8(g));
    T(97, "R2_80276408", it_80276408(g, &ip->x378_itemColl, &ov));
    /* itcoll.c */
    T(98, "R2_802714C0", it_802714C0(g));
    T(99, "R2_80271508", it_80271508(g, (HurtCapsuleState) (R() % 3)));
    T(100, "R2_80271534", { HurtCapsule hc; s32 idx = (s32) (R() % 2); HURT_IN(hc); it_80271534(g, idx, &hc); });
    T(101, "R2_80271590", { HurtCapsule hc; s32 idx = (s32) (R() % 2); HURT_IN(hc); it_80271590(g, idx, &hc); ret = HURT_H(hc); });
    T(102, "R2_80272280", it_80272280(g));
    T(103, "R2_80272298", it_80272298(g));
}
#undef it_804D6D28
#undef bool
#define bool _Bool
