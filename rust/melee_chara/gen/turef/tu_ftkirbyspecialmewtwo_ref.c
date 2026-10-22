/* C reference for src/tu/ftkirbyspecialmewtwo.rs: the Shadow Ball recoil / launch / charge-sfx ladder and
 * the Loop anim, cut verbatim by tu_shard321.py. Callees in other TUs are logging stand-ins. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include <math.h>
#include <stdarg.h>
#include "ft/kinds/ftKirby/forward.h"
#include "ft/kinds/ftKirby/ftkirby.h"
#include "ft/kinds/ftKirby/types.h"
#include "ft/kinds/ftKirby/inlines.h"
#include <melee/ef/eflib.h>
#include <melee/ef/efsync.h>
#include <melee/ft/ft_0877.h>
#include <melee/ft/ftcommon.h>
#include <melee/lb/lb_00B0.h>
#include <placeholder.h>
#include <stdio.h>

ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static u32 ev[40];
static int nev;
static void E(u32 v) { if (nev < 40) ev[nev++] = v; }
static u32 b(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static void Ef(float f) { E(b(f)); }
static float lbx, lby, lbz;
static HSD_GObj BALL;

void lb_8000B1CC(HSD_JObj* j, Vec3* a, Vec3* out)
{
    (void) j; E(0x10); if (a) { Ef(a->x); Ef(a->y); Ef(a->z); }
    out->x = lbx; out->y = lby; out->z = lbz;
}
void it_802C53F0(Item_GObj* i, Vec3* p, float ang, float ch, float mx)
{
    (void) i; E(0x11); Ef(p->x); Ef(p->y); Ef(p->z); Ef(ang); Ef(ch); Ef(mx);
}
HSD_GObj* it_802C5000(Item_GObj* g, Vec3* p, s32 part, s32 kind, float dir)
{
    (void) g; E(0x12); Ef(p->x); Ef(p->y); Ef(p->z); E((u32) part); E((u32) kind); Ef(dir);
    return &BALL;
}
void ft_PlaySFX(Fighter* fp, enum_t id, u8 vol, u8 pan) { (void) fp; E(0x13); E((u32) id); E(vol); E(pan); }
void ft_80088510(Fighter* fp, int id, u8 vol, u8 pan) { (void) fp; E(0x14); E((u32) id); E(vol); E(pan); }
void efLib_DestroyAll(HSD_GObj* g) { (void) g; E(0x15); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId ms, MotionFlags fl, f32 a, f32 sp, f32 bl, Fighter_GObj* x)
{
    (void) g; (void) x; E(0x16); E((u32) ms); E(fl); Ef(a); Ef(sp); Ef(bl);
}
void ftCommon_8007EBAC(Fighter* fp, u32 a, u32 c) { (void) fp; E(0x17); E(a); E(c); }
bool ftCo_800BFFD0(Fighter* fp, enum_t a, bool c) { (void) fp; E(0x18); E((u32) a); E(c); return 0; }
void* efSync_Spawn(s32 id, HSD_GObj* g, ...)
{
    va_list ap;
    Vec3* p;
    (void) g;
    va_start(ap, g);
    p = va_arg(ap, Vec3*);
    E(0x19); E((u32) id); Ef(p->x); Ef(p->y); Ef(p->z);
    va_end(ap);
    return NULL;
}
void ftKb_Init_800EE74C(Fighter_GObj* g) { (void) g; }
void ftKb_Init_800EE7B8(Fighter_GObj* g) { (void) g; }
bool ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return 0; }
void ftCo_800BFFAC(Fighter* fp) { (void) fp; E(0x1A); }

static Vec3 const ftKb_Init_803B7570 = { 0.0f, 4.0f, 0.0f };
u32 ftKb_Init_803CB828[] = { 0x00030DB9, 0x00030DBC, 0x00030DBF, 0x00030DC2 };

#include "tu_ftkirbyspecialmewtwo_extract.inc"

static Fighter F;
static HSD_GObj G;
static FighterBone PARTS[100];
static HSD_JObj dummy_jobj;
static ftKb_DatAttrs DA;
static u32 s = 0x3E27u;
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

static void dump(const char* tag, int i)
{
    int k;
    printf("%s %d %08x %08x %d %d %08x %08x %d %d %d %d |", tag, i, b(F.self_vel.x), b(F.gr_vel), F.u.kb.x9C,
           F.mv.kb.specialhi.xC, b(F.mv.kb.specialhi.x10.f), b(F.mv.kb.specialhi.x8.f), F.mv.kb.specialhi.x4,
           F.mv.kb.specialhi.x8.i, F.cmd_vars[1], F.u.kb.x98 != NULL);
    for (k = 0; k < nev; k++) printf(" %x", ev[k]);
    printf("\n");
    nev = 0;
}

int main(void)
{
    int i, k;
    G.user_data = &F;
    F.dat_attrs = &DA;
    F.gobj = &G;
    for (k = 0; k < 100; k++) PARTS[k].joint = &dummy_jobj;
    F.parts = PARTS;
    for (i = 0; i < 1500; i++) {
        Vec3 pa, pb;
        DA.specialn_mt_charge_time = 1.0f + (float) (rnd() % 60);
        DA.specialn_mt_ground_horizontal_momentum = rf();
        DA.specialn_mt_air_horizontal_momentum = rf();
        DA.specialn_mt_loops_to_full_charge = rnd() % 4;
        lbx = rf(); lby = rf(); lbz = rf();
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.ground_or_air = rnd() & 1;
        F.motion_id = (rnd() & 1) ? ftKb_MS_MtSpecialAirNEnd : ((rnd() & 1) ? ftKb_MS_MtSpecialNEnd : 0);
        F.u.kb.x9C = rnd() % 70;
        F.self_vel.x = rf();
        F.gr_vel = rf();
        F.cur_anim_frame = rf();
        F.mv.kb.specialhi.xC = (int) (rnd() % 4) - 1;
        F.mv.kb.specialhi.x4 = (int) (rnd() % 5);
        F.mv.kb.specialhi.x8.i = rnd() & 1;
        F.mv.kb.specialhi.x10.f = (rnd() & 3) ? ((rnd() & 1) ? rf() * 0.5f : 0.0f) : 0.0001f;
        F.cmd_vars[1] = rnd() % 3;
        F.cmd_vars[2] = rnd() % 2;
        F.cmd_vars[3] = rnd() % 2;
        F.u.kb.x98 = (rnd() & 1) ? &BALL : NULL;
        F.item_gobj = NULL;
        ftKb_SpecialNMt_SetRecoil(&G);
        dump("A", i);
        ftKb_SpecialNMt_80107410(&G);
        dump("B", i);
        ftKb_SpecialNMt_801071FC(&G);
        dump("C", i);
        ftKb_MtSpecialNLoop_Anim(&G);
        dump("D", i);
        F.u.kb.x98 = NULL;
        ftKb_MtSpecialNStart_Anim(&G);
        dump("E", i);
        (void) pa; (void) pb;
    }
    return 0;
}
