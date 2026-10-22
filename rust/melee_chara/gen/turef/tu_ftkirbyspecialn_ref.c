/* C reference for src/tu/ftkirbyspecialn.rs: the pure / fused parts of ftkirbyspecialn.c cut verbatim
 * (with lbtrigf.c's sinf/cosf) by tu_shard321.py. Callees in other TUs are stand-ins that log what they
 * were given; the Rust test installs the same logging hooks. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftKirby/forward.h"
#include "ft/kinds/ftKirby/types.h"
#include <placeholder.h>
#include <stdio.h>

ftCommonData* p_ftCommonData;
float MSL_TrigF_80400770[1];
float MSL_TrigF_80400774[1];
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static u32 ev[16];
static int nev;
static float posx, posy, posz;
static double item_ret;
static float fighter_ret;
static void E(u32 v) { if (nev < 16) ev[nev++] = v; }

f64 it_802F23AC(Item_GObj* o, Vec3* p) { (void) o; posx = p->x; posy = p->y; posz = p->z; return item_ret; }
void it_802F2810(HSD_GObj* g) { (void) g; E(1); }
float ftCo_800BD19C(Fighter_GObj* g, Vec3* p) { (void) g; posx = p->x; posy = p->y; posz = p->z; return fighter_ret; }
void ftCo_800BD620(Fighter_GObj* g) { (void) g; E(2); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId ms, MotionFlags fl, f32 a, f32 b, f32 c, Fighter_GObj* x)
{
    (void) g; (void) a; (void) b; (void) c; (void) x;
    E(0x100); E((u32) ms); E((u32) fl);
}
void ftCommon_8007E2F4(Fighter* fp, s16 v) { fp->x1A6A = v; }
void ftKb_SpecialN_800F9070(HSD_GObj* g) { (void) g; }
void ftKb_SpecialN_800F63EC(Fighter_GObj* g)
{
    Fighter_ChangeMotionState(g, ftKb_MS_EatAir, 0x10, 0.0F, 1.0F, 0.0F, NULL);
    ftKb_SpecialN_800F9070(g);
    ftCommon_8007E2F4(GET_FIGHTER(g), 0x1FF);
}
void ftKb_SpecialN_800F6388(Fighter_GObj* g)
{
    Fighter_ChangeMotionState(g, ftKb_MS_Eat, 0x10, 0.0F, 1.0F, 0.0F, NULL);
    ftKb_SpecialN_800F9070(g);
    ftCommon_8007E2F4(GET_FIGHTER(g), 0x1FF);
}

#include "tu_ftkirbyspecialn_extract.inc"

static Fighter F;
static HSD_GObj G, T, V;
static ftKb_DatAttrs DA;
static u32 s = 0x5EED5u;
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
static u32 b(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static void flush(const char* tag, int i)
{
    int k;
    printf("%s %d %08x %08x %08x %x |", tag, i, b(posx), b(posy), b(posz), (unsigned) F.x1A6A);
    for (k = 0; k < nev; k++) printf(" %x", ev[k]);
    printf("\n");
    nev = 0;
    F.x1A6A = 0;
}

int main(void)
{
    int i;
    G.user_data = &F;
    F.dat_attrs = &DA;
    F.gobj = &G;
    F.target_item_gobj = &T;
    F.victim_gobj = &V;
    for (i = 0; i < 1500; i++) {
        Vec3 v, o;
        float r, d;
        DA.specialn_x_offset_inhaled = rf();
        DA.specialn_y_offset_inhaled = rf();
        DA.specialn_inhale_velocity = rf() * 4.0f;
        DA.specialn_spit_out_release_angle = rf() * 3.0f;
        DA.specialn_swallow_star_vertical_velocity = rf();
        DA.specialn_swallow_star_gravity = rf();
        F.cur_pos.x = rf();
        F.cur_pos.y = rf();
        F.cur_pos.z = rf();
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.ground_or_air = rnd() & 1;
        d = (rnd() & 1) ? 1.0f : -1.0f;
        v.x = v.y = v.z = 7.0f;
        r = ftKb_SpecialN_800F58D8(&G, &v, d);
        printf("A %d %08x %08x %08x %08x\n", i, b(r), b(v.x), b(v.y), b(v.z));
        o.x = o.y = o.z = 7.0f;
        ftKb_SpecialN_800F5B5C(&G, &o);
        printf("B %d %08x %08x %08x\n", i, b(o.x), b(o.y), b(o.z));
        item_ret = (double) rf() * 3.0;
        fighter_ret = rf() * 3.0f;
        ftKb_SpecialN_800F5DE8(&G);
        flush("C", i);
        ftKb_SpecialN_800F5EA8(&G);
        flush("D", i);
        ftKb_SpecialNCapture0_Anim(&G);
        flush("E", i);
        ftKb_SpecialNCapture_Anim(&G);
        flush("F", i);
    }
    return 0;
}
