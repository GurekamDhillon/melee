/* C reference for src/tu/ftkirbyspecialsamus.rs: the Charge Shot launch, recoil, charge level sfx and the Start /
 * Hold anims, cut verbatim by tu_shard321.py. Callees in other TUs are logging stand-ins. */
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
static HSD_GObj SHOT;
static int frames_remaining;

void lb_8000B1CC(HSD_JObj* j, Vec3* a, Vec3* out)
{
    (void) j; E(0x10); if (a) { Ef(a->x); Ef(a->y); Ef(a->z); }
    out->x = lbx; out->y = lby; out->z = lbz;
}
void it_802B56E4(Item_GObj* i, Vec3* p, float ang, float ch, float mx)
{
    (void) i; E(0x11); Ef(p->x); Ef(p->y); Ef(p->z); Ef(ang); Ef(ch); Ef(mx);
}
HSD_GObj* it_802B55C8(Item_GObj* g, Vec3* p, Fighter_Part part, s32 kind, float dir)
{
    (void) g; E(0x12); Ef(p->x); Ef(p->y); Ef(p->z); E((u32) part); E((u32) kind); Ef(dir);
    return &SHOT;
}
void it_802B5974(void* item) { (void) item; E(0x13); }
void ft_80088510(Fighter* fp, int id, u8 vol, u8 pan) { (void) fp; E(0x14); E((u32) id); E(vol); E(pan); }
void efLib_DestroyAll(HSD_GObj* g) { (void) g; E(0x15); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId ms, MotionFlags fl, f32 a, f32 sp, f32 bl, Fighter_GObj* x)
{
    (void) g; (void) x; E(0x16); E((u32) ms); E(fl); Ef(a); Ef(sp); Ef(bl);
}
bool ftCo_800BFFD0(Fighter* fp, enum_t a, bool c) { (void) fp; E(0x18); E((u32) a); E(c); return 0; }
void* efSync_Spawn(s32 id, HSD_GObj* g, ...)
{
    va_list ap;
    Vec3* p;
    float* q;
    (void) g;
    va_start(ap, g);
    p = va_arg(ap, Vec3*);
    q = va_arg(ap, float*);
    E(0x19); E((u32) id); Ef(p->x); Ef(p->y); Ef(p->z); Ef(*q);
    va_end(ap);
    return NULL;
}
void ftKb_Init_800EE74C(Fighter_GObj* g) { (void) g; }
void ftKb_Init_800EE7B8(Fighter_GObj* g) { (void) g; }
bool ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return frames_remaining; }

u32 ftKb_Init_803CB6C0[] = { 260006, 260009, 260012, 260015, 260018, 0 };

#include "tu_ftkirbyspecialsamus_extract.inc"

static Fighter F;
static HSD_GObj G;
static FighterBone PARTS[100];
static HSD_JObj dummy_jobj;
static ftKb_DatAttrs DA;
static u32 s = 0x55A5u;
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
    printf("%s %d %08x %d %d %d %d %d %d %d %d %d |", tag, i, b(F.self_vel.x), F.u.kb.xA8, F.mv.kb.specialhi.x4,
           F.mv.kb.specialhi.x0, F.cmd_vars[0], F.cmd_vars[1], F.cmd_vars[2], F.u.kb.xA4 != NULL, F.u.kb.xAC != NULL,
           F.item_gobj != NULL);
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
    for (i = 0; i < 2000; i++) {
        DA.specialn_ss_charge_time = 1.0f + (float) (rnd() % 100);
        DA.specialn_ss_aerial_shot_recoil = rf();
        DA.specialn_ss_frames_per_charge_level = rnd() % 4;
        lbx = rf(); lby = rf(); lbz = rf();
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.ground_or_air = rnd() & 1;
        F.motion_id = (rnd() & 1) ? ftKb_MS_SsSpecialAirN : 0;
        F.u.kb.xA8 = rnd() % ((int) DA.specialn_ss_charge_time + 1);
        F.self_vel.x = rf();
        F.mv.kb.specialhi.x4 = rnd() % 5;
        F.mv.kb.specialhi.x0 = rnd() & 1;
        F.cmd_vars[0] = rnd() % 3;
        F.cmd_vars[1] = rnd() % 3;
        F.cmd_vars[2] = rnd() % 2;
        F.u.kb.xA4 = (rnd() & 1) ? &SHOT : NULL;
        F.u.kb.xAC = (rnd() & 1) ? &SHOT : NULL;
        F.item_gobj = NULL;
        frames_remaining = rnd() & 1;
        ftKb_SpecialNSs_800FCDE0(&G);
        dump("A", i);
        ftKb_SsSpecialNStart_Anim(&G);
        dump("B", i);
        ftKb_SsSpecialNHold_Anim(&G);
        dump("C", i);
        ftKb_SsSpecialAirNStart_Anim(&G);
        dump("D", i);
    }
    return 0;
}
