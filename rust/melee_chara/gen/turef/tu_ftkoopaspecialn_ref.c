/* C reference for src/tu/ftkoopaspecialn.rs: ftKp_SpecialLw_80134ACC (the flame spawn, 2 fused
 * sites) with its direction table and ftKp_SpecialN_80134ACC_inline, cut verbatim from the game
 * by tu_shard329.py. The callees outside this TU are recording stand-ins both sides share. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftKoopa/types.h"
#include <melee/it/kinds/itkoopaflame.h>
#include <stdio.h>

ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
static u32 s = 0x329A11u;
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
static u32 rr = 12345;
int HSD_Randi(int n) { rr = rr * 1664525u + 1013904223u; return (int) ((rr >> 8) % (u32) n); }
static Vec3 LBV;
void lb_8000B1CC(HSD_JObj* j, Vec3* a, Vec3* out) { (void) j; (void) a; *out = LBV; }
static u32 counter;
u32 Item_8026AE60(void) { return counter++; }
void ft_80089824(Fighter_GObj* g) { (void) g; printf(" A"); }
void ft_800892A0(Fighter_GObj* g) { (void) g; printf(" B"); }
void ft_PlaySFX(Fighter* fp, enum_t id, u8 vol, u8 pan) { (void) fp; printf(" sfx %x %d %d", id, vol, pan); }
Item_GObj* itKoopaFlame_Spawn(Fighter_GObj* parent, Vec3* pos, f32 facing_dir, u32 unk, s32 gfx, s32 base_speed, s32 scale, s32 kind)
{
    (void) parent;
    printf(" spawn %08x %08x %08x %u %d %d %d %d", b(pos->x), b(pos->y), b(facing_dir), unk, gfx, base_speed, scale, kind);
    return 0;
}

#include "tu_ftkoopaspecialn_extract.inc"

static Fighter F;
static HSD_GObj G;
static ftKoopaAttributes DA;

int main(void)
{
    int i;
    G.user_data = &F;
    F.dat_attrs = &DA;
    F.parts = (FighterBone*) &F + 0; /* parts[48].joint only passes through to the stub */
    for (i = 0; i < 3000; i++) {
        static FighterBone parts[64];
        F.parts = parts;
        LBV.x = rf(); LBV.y = rf(); LBV.z = rf();
        F.x34_scale.y = rf();
        F.facing_dir = (i & 1) ? 1.0f : -1.0f;
        if (i % 7 == 0) F.facing_dir = rf();
        DA.x24 = rf(); DA.x28 = rf(); DA.x1C = rf() * 3.0f; DA.x18 = rf() * 5.0f + 7.0f;
        F.u.kp.x222C = rf() * 40.0f; F.u.kp.x2230 = rf() * 6.0f + 3.0f;
        F.mv.kp.specialn.facing_dir = rnd() % 4;
        F.mv.kp.specialn.x14 = rnd() % 12;
        F.mv.kp.specialn.x4 = rnd() % 5;
        F.kind = (rnd() & 1) ? Ft_Kind_Koopa : Ft_Kind_Mario;
        printf("%d", i);
        ftKp_SpecialLw_80134ACC(&G);
        printf(" | %d %u\n", F.mv.kp.specialn.x14, F.mv.kp.specialn.x4);
    }
    return 0;
}
