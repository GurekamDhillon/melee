/* C reference for src/tu/ftkirbyspecialpurin.rs: all of ftkirbyspecialpurin.c cut verbatim by
 * tu_shard321.py (with lbtrigf.c's trig). Callees in other TUs are logging stand-ins (the Rust test
 * installs the same ones); the driver contains no a*b+c of its own. */
#define HSD_JObjSetScale hdr_HSD_JObjSetScale
#define HSD_JObjGetScale hdr_HSD_JObjGetScale
#define HSD_JObjSetRotationY hdr_HSD_JObjSetRotationY
#include <melee/ft/forward.h>
#include <sysdolphin/baselib/forward.h>

#include <math.h>
#include <placeholder.h>
#include <stddef.h>
#include <stdarg.h>
#include <stdio.h>

#include "ft/kinds/ftKirby/forward.h"
#include "ft/kinds/ftKirby/ftkirby.h"
#include "ft/kinds/ftKirby/types.h"
#include <melee/cm/camera.h>
#include <melee/ef/efsync.h>
#include <melee/ft/fighter.h>
#include <melee/ft/ft_081B.h>
#include <melee/ft/ft_0877.h>
#include <melee/ft/ft_0881.h>
#include <melee/ft/ft_0892.h>
#include <melee/ft/ftcliffcommon.h>
#include <melee/ft/ftcolanim.h>
#include <melee/ft/ftcoll.h>
#include <melee/ft/ftcommon.h>
#include <melee/ft/ftparts.h>
#include <melee/ft/inlines.h>
#include <melee/ft/kinds/ftCommon/ftCo_Fall.h>
#include <melee/ft/kinds/ftCommon/ftCo_FallSpecial.h>
#include <melee/ft/kinds/ftCommon/ftCo_Landing.h>
#include <melee/ft/kinds/ftPurin/inlines.h>
#include <melee/ft/types.h>
#include <melee/mp/mplib.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/jobj.h>

#undef HSD_JObjSetScale
#undef HSD_JObjGetScale
#undef HSD_JObjSetRotationY
ftCommonData* p_ftCommonData;
float MSL_TrigF_80400770[1];
float MSL_TrigF_80400774[1];
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static u32 ev[40];
static int nev;
static void E(u32 v) { if (nev < 40) ev[nev++] = v; }
static u32 b(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static void Ef(float f) { E(b(f)); }

/* controlled returns */
static int r_80082888, r_80082978, r_8008239C, r_800824A0, r_80081298, r_ft_80082708, r_ft_80081D0C;
static float r_mp;
static int zz, ab;
static float za;
static HSD_JObj dummy_jobj;

void ftPartSetRotX(Fighter* fp, int i, f32 v) { (void) fp; E(0x10); E((u32) i); Ef(v); }
void ftPartSetRotY(Fighter* fp, int i, f32 v) { (void) fp; E(0x11); E((u32) i); Ef(v); }
void HSD_JObjSetScale(HSD_JObj* j, Vec3* s) { (void) j; E(0x12); Ef(s->x); Ef(s->y); Ef(s->z); }
void HSD_JObjGetScale(HSD_JObj* j, Vec3* s) { (void) j; s->x = 1.5f; s->y = 2.5f; s->z = 3.5f; E(0x13); }
void HSD_JObjSetRotationY(HSD_JObj* j, f32 v) { (void) j; E(0x14); Ef(v); }
void ft_80089824(Fighter_GObj* g) { (void) g; E(0x15); }
void ft_800892A0(Fighter_GObj* g) { (void) g; E(0x16); }
void ftColl_8007AFF8(Fighter_GObj* g) { (void) g; E(0x17); }
void ftColl_8007ABD0(HitCapsule* h, u32 dmg, Fighter_GObj* g) { (void) h; (void) g; E(0x18); E(dmg); }
void ftCo_800BFFAC(Fighter* fp) { (void) fp; E(0x19); }
bool ftCo_800BFFD0(Fighter* fp, enum_t a, bool c) { (void) fp; E(0x1A); E((u32) a); E(c); return 0; }
void ft_80088510(Fighter* fp, int id, u8 v, u8 p) { (void) fp; E(0x1B); E((u32) id); E(v); E(p); }
void Camera_RequestQuake(CmQuakeKind k, Vec* p) { E(0x1C); E((u32) k); Ef(p->x); Ef(p->y); Ef(p->z); }
void* efSync_Spawn(s32 id, HSD_GObj* g, ...)
{
    va_list ap;
    Vec3* p;
    float* q;
    float* r;
    (void) g;
    va_start(ap, g);
    p = va_arg(ap, Vec3*);
    q = va_arg(ap, float*);
    E(0x1D); E(id); Ef(p->x); Ef(p->y); Ef(p->z); Ef(*q);
    if (id == 0x3FF) {
        r = va_arg(ap, float*);
        Ef(*r);
    }
    va_end(ap);
    return NULL;
}
bool ft_80082888(Fighter_GObj* g, ftCollisionBox* bx) { (void) g; (void) bx; E(0x1E); return r_80082888; }
bool ft_80082978(Fighter_GObj* g, ftCollisionBox* bx) { (void) g; (void) bx; E(0x1F); return r_80082978; }
bool ft_8008239C(Fighter_GObj* g, int d, ftCollisionBox* bx) { (void) g; (void) bx; E(0x20); E((u32) d); return r_8008239C; }
bool ft_800824A0(Fighter_GObj* g, ftCollisionBox* bx) { (void) g; (void) bx; E(0x21); return r_800824A0; }
bool ftCliffCommon_80081298(Fighter_GObj* g) { (void) g; return r_80081298; }
void ftCliffCommon_80081370(Fighter_GObj* g) { (void) g; E(0x22); }
float mpLib_800569EC(u32 f) { (void) f; return r_mp; }
void ftCommon_8007D5D4(Fighter* fp) { (void) fp; E(0x23); }
void ftCommon_8007D7FC(Fighter* fp) { (void) fp; E(0x24); }
void ftCommon_Fall(Fighter* fp, float a, float c) { (void) fp; E(0x25); Ef(a); Ef(c); }
void ftCommon_SetSelfMovementFromGroundedMovement(Fighter_GObj* g) { (void) g; E(0x26); }
void ftCommon_8007EBAC(Fighter* fp, u32 a, u32 c) { (void) fp; E(0x27); E(a); E(c); }
void ft_PlaySFX(Fighter* fp, enum_t id, u8 vol, u8 pan) { (void) fp; E(0x28); E((u32) id); E((u32) vol); E((u32) pan); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId ms, MotionFlags fl, f32 a, f32 sp, f32 bl, Fighter_GObj* x)
{
    (void) g; (void) x; E(0x29); E((u32) ms); E(fl); Ef(a); Ef(sp); Ef(bl);
}
void ftAnim_8006EBA4(HSD_GObj* g) { (void) g; E(0x2A); }
void ftAnim_SetAnimRate(HSD_GObj* g, f32 r) { (void) g; E(0x2B); Ef(r); }
bool ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return 0; }
void ft_8008A2BC(HSD_GObj* g) { (void) g; E(0x2C); }
void ftCo_Fall_Enter(HSD_GObj* g) { (void) g; E(0x2D); }
void ftCo_LandingFallSpecial_Enter(HSD_GObj* g, bool a, f32 l) { (void) g; E(0x2E); E(a); Ef(l); }
void ftCo_80096900(HSD_GObj* g, bool a, bool c, bool d, f32 e, f32 f) { (void) g; E(0x2F); E(a); E(c); E(d); Ef(e); Ef(f); }
void ftKb_Init_800EE74C(Fighter_GObj* g) { (void) g; }
void ftKb_Init_800EE7B8(Fighter_GObj* g) { (void) g; }
void ftCommon_CalcGroundAccel_Deaccel(Fighter* fp, float f) { (void) fp; E(0x30); Ef(f); }
void ftCommon_CalcSelfAccel_Drift(Fighter* fp) { (void) fp; E(0x31); }
GroundOrAir ft_80082708(Fighter_GObj* g) { (void) g; return r_ft_80082708; }
GroundOrAir ft_80081D0C(Fighter_GObj* g) { (void) g; return r_ft_80081D0C; }

/* The game declares the two scale tables separately but ftKb_AirScaleAnimStep reads base[frame + 4] off the
 * end of the first; on the PPC they are adjacent (3CB710 / 3CB720), a native build need not place them so. */
static float ftKb_Init_803CB710[8] = { 0.65F, 0.7F, 0.8F, 1.0F, 1.1F, 1.35F, 1.3F, 1.2F };
#define ftKb_Init_803CB720 (ftKb_Init_803CB710 + 4)
static void fn_80100E0C(Fighter_GObj* gobj);
static void fn_80100F60(Fighter_GObj* gobj);
static void fn_80105978(Fighter_GObj* gobj);

#include "tu_ftkirbyspecialpurin_extract.inc"

static Fighter F;
static FighterBone PARTS[100];
static HSD_GObj G;
static ftKb_DatAttrs DA;
static u32 s = 0x9071Fu;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    switch (rnd() % 8) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    default: return (float) ((int) (rnd() % 4001) - 2000) / 997.0f;
    }
}
static float rg(void) { return rf() * 3.0f; }
static float fa(float x) { return x < 0.0f ? -x : x; }
static float sgn(void) { return (rnd() & 1) ? 1.0f : -1.0f; }

static void randomize(void)
{
    struct ftPurin_SpecialNVars* sp = &F.mv.pr.specialn;
    DA.specialn_pr_duration = rnd() % 30;
    DA.specialn_pr_unk = rnd() % 5;
    DA.specialn_pr_air_height_offset_at_start = rf();
    DA.specialn_pr_bounciness = rf();
    DA.specialn_pr_unk1 = rf();
    DA.specialn_pr_base_speed = rg();
    DA.specialn_pr_max_speed = rg();
    DA.specialn_pr_unk2 = rf();
    DA.specialn_pr_air_x_axis_momentum = rf();
    DA.specialn_pr_air_y_axis_momentum = rf();
    DA.specialn_pr_spinning_speed = rf();
    DA.specialn_pr_spinning_speed_turn = rg();
    DA.specialn_pr_delay_per_smoke = 1 + rnd() % 4;
    DA.specialn_pr_unk3 = rf();
    DA.specialn_pr_bounce1 = rf();
    DA.specialn_pr_bounce2 = rf();
    DA.specialn_pr_base_damage = rg();
    DA.specialn_pr_damage_multiplier = rg();
    DA.specialn_pr_horizontal_bounce_momentum_on_hit = rf();
    DA.specialn_pr_vertical_bounce_momentum_on_hit = rf();
    DA.specialn_pr_forward_momentum_from_stick = rf();
    DA.specialn_pr_unk4 = rf();
    DA.specialn_pr_unk5 = rg();
    DA.specialn_pr_unk6 = 1 + rnd() % 3;
    DA.specialn_pr_charge_rate1 = rg();
    DA.specialn_pr_charge_time = fa(rg()) * 4.0f;
    DA.specialn_pr_charge_rate2 = rg();
    DA.specialn_pr_charge_spin_animation = rg() * 10.0f;
    DA.specialn_pr_unk8 = rf();
    DA.specialn_pr_some_speed_var = rf();
    DA.specialn_pr_spin_anim_speed_after_collision = rf();
    DA.specialn_pr_air_speed = rf();
    DA.specialn_pr_turn_rate_related = rf();
    DA.specialn_pr_unk9 = rf();
    DA.specialn_pr_unk10 = rf();
    DA.specialn_pr_unk11 = rf();
    DA.specialn_pr_unk12 = rf();
    DA.specialn_pr_freefall_toggle = (rnd() & 3) ? rf() : 0.0f;
    F.cur_pos.x = rf();
    F.cur_pos.y = rf();
    F.cur_pos.z = rf();
    F.facing_dir = sgn();
    F.gr_vel = rg();
    F.self_vel.x = rg();
    F.self_vel.y = rg();
    F.self_vel.z = rf();
    F.xE4_ground_accel_1 = rf();
    F.x74_self_accel.x = rf();
    F.x74_self_accel.y = rf();
    F.cur_anim_frame = rf();
    F.ground_or_air = rnd() & 1;
    F.motion_id = 0x1DB + rnd() % 17;
    F.u.kb.x8C.x = rf();
    F.u.kb.x8C.y = rf();
    F.u.kb.x8C.z = rf();
    F.coll_data.floor.normal.x = rf();
    F.coll_data.floor.normal.y = rf();
    F.coll_data.floor.flags = rnd();
    F.coll_data.left_facing_wall.normal.x = rf();
    F.coll_data.left_facing_wall.normal.y = rf();
    F.coll_data.ecb.top.y = rf();
    F.coll_data.ecb.bottom.y = rf();
    F.coll_data.ecb.left.x = rf();
    F.coll_data.ecb.right.x = rf();
    F.coll_data.env_flags = (rnd() & 1) ? (int) (rnd() & 0xFFF) : 0;
    F.input.lstick[0].x = rf();
    F.input.held_buttons[0] = (rnd() & 1) ? HSD_PAD_B : 0;
    F.x914[0].state = rnd() % 3;
    F.x914[0].x4 = rnd() % 4;
    sp->x0 = (int) (rnd() % 30) - 5;
    sp->x4 = (int) (rnd() % 5) - 1;
    sp->x8 = (int) (rnd() % 7) - 2;
    sp->xC = (int) (rnd() % 4);
    sp->x10 = rg();
    sp->x14 = rg() * 4.0f;
    sp->x18 = rg();
    sp->x1C = rf();
    sp->facing_dir = (rnd() & 3) ? 0.0f : sgn();
    sp->x24 = (int) (rnd() % 20);
    sp->x2C = (int) (rnd() % 400) - 20;
    sp->x30 = rnd() & 1;
    sp->x34.x = sgn();
    sp->x34.y = (rnd() & 1) ? 0.0f : rg();
    r_80082888 = rnd() & 1;
    r_80082978 = rnd() & 1;
    r_8008239C = rnd() & 1;
    r_800824A0 = rnd() & 1;
    r_80081298 = rnd() & 1;
    r_ft_80082708 = rnd() & 1;
    r_ft_80081D0C = rnd() & 1;
    r_mp = rf();
    zz = rnd() & 1;
    za = rf();
    ab = rnd() & 1;
}

static void dump(const char* tag, int i)
{
    struct ftPurin_SpecialNVars* sp = &F.mv.pr.specialn;
    int k;
    printf("%s %d %08x %08x %08x %08x %08x %08x %08x %d %d %d %d %d %d %08x %08x %08x %08x %u %u |", tag, i,
           b(F.gr_vel), b(F.self_vel.x), b(F.self_vel.y), b(F.facing_dir), b(sp->x14), b(sp->x10), b(sp->x18),
           sp->x0, sp->x2C, sp->x30, sp->x8, sp->x24, sp->xC, b(sp->x34.x), b(sp->x34.y), b(sp->x1C),
           b(sp->facing_dir), F.u.kb.hat.kind, F.x914[0].state);
    for (k = 0; k < nev; k++) printf(" %x", ev[k]);
    printf("\n");
    nev = 0;
}

#define CASE(tag, call) \
    do { randomize(); G.hsd_obj = &dummy_jobj; call; dump(tag, i); } while (0)

int main(void)
{
    int i, k;
    G.user_data = &F;
    F.dat_attrs = &DA;
    F.gobj = &G;
    F.parts = PARTS;
    for (k = 0; k < 100; k++) PARTS[k].joint = &dummy_jobj;
    for (i = 0; i < 200; i++) {
        CASE("a", ftKb_PrSpecialNLoop_Anim(&G));
        CASE("b", ftKb_PrSpecialNFull_Anim(&G));
        CASE("c", ftKb_PrSpecialAirNLoop_Anim(&G));
        CASE("d", ftKb_PrSpecialAirNFull_Anim(&G));
        CASE("e", ftKb_PrSpecialN1_Anim(&G));
        CASE("f", ftKb_PrSpecialAirN_Anim(&G));
        CASE("g", ftKb_PrSpecialNTurn_Anim(&G));
        CASE("h", ftKb_PrSpecialN_Anim(&G));
        CASE("i", ftKb_PrSpecialNHit_Anim(&G));
        CASE("j", ftKb_PrSpecialNStart_Anim(&G));
        CASE("k", ftKb_PrSpecialNEnd_Anim(&G));
        CASE("l", ftKb_PrSpecialAirNEndR_Anim(&G));
        CASE("m", ftKb_PrSpecialNLoop_IASA(&G));
        CASE("n", ftKb_PrSpecialAirNFull_IASA(&G));
        CASE("o", ftKb_PrSpecialN1_IASA(&G));
        CASE("p", ftKb_PrSpecialN1_Phys(&G));
        CASE("q", ftKb_PrSpecialNTurn_Phys(&G));
        CASE("r", ftKb_PrSpecialAirN_Phys(&G));
        CASE("s", ftKb_PrSpecialN_Phys(&G));
        CASE("t", ftKb_PrSpecialN1_Coll(&G));
        CASE("u", ftKb_PrSpecialAirN_Coll(&G));
        CASE("v", ftKb_PrSpecialNTurn_Coll(&G));
        CASE("w", ftKb_PrSpecialNHit_Coll(&G));
        CASE("x", fn_80100E0C(&G));
        CASE("y", ftKb_SpecialNPr_80100F94(&G));
        CASE("z", ftKb_SpecialNPr_801010D4(&G, zz, 0x40012, za));
        CASE("A", ftKb_SpecialNPr_8010140C(&G, ab));
        CASE("B", ftKb_SpecialNPr_80101560(&G));
        CASE("C", ftKb_SpecialNPr_80101618(&G));
        CASE("D", fn_80105978(&G));
        CASE("E", ftKb_PrSpecialNStart_Coll(&G));
        CASE("F", ftKb_PrSpecialNLoop_Coll(&G));
        CASE("G", ftKb_PrSpecialNEnd_Coll(&G));
        CASE("H", ftKb_PrSpecialAirNStart_Coll(&G));
    }
    return 0;
}
