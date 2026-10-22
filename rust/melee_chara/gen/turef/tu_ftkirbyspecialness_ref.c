/* C reference for src/tu/ftkirbyspecialness.rs: ftKb_NsSpecialNStart_Anim_inline cut verbatim by
 * tu_shard321.py (the flash height `y += scale.y * 3.0f` is its fused site). */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftKirby/forward.h"
#include "ft/kinds/ftKirby/ftkirby.h"
#include "ft/kinds/ftKirby/types.h"
#include <melee/lb/lb_00B0.h>
#include <placeholder.h>
#include <stdio.h>

ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static float lbx, lby, lbz;
static float fpx, fpy, fpz, fdir;
static int spawn_ok;
static HSD_GObj SPAWNED;

void lb_8000B1CC(HSD_JObj* j, Vec3* a, Vec3* out) { (void) j; (void) a; out->x = lbx; out->y = lby; out->z = lbz; }
HSD_GObj* it_802AA8C0(Item_GObj* g, Vec3* p, ItemKind k, f32 dir)
{
    (void) g; (void) k;
    fpx = p->x; fpy = p->y; fpz = p->z; fdir = dir;
    return spawn_ok ? &SPAWNED : NULL;
}
void ftKb_Init_800EE74C(Fighter_GObj* g) { (void) g; }
void ftKb_Init_800EE7B8(Fighter_GObj* g) { (void) g; }

#include "tu_ftkirbyspecialness_extract.inc"

static Fighter F;
static HSD_GObj G;
static FighterBone PARTS[100];
static HSD_JObj dummy_jobj;
static u32 s = 0x2E55u;
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

int main(void)
{
    int i, k;
    G.user_data = &F;
    for (k = 0; k < 100; k++) PARTS[k].joint = &dummy_jobj;
    F.parts = PARTS;
    for (i = 0; i < 3000; i++) {
        Vec3 pos;
        lbx = rf();
        lby = rf();
        lbz = rf();
        F.x34_scale.y = rf();
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        spawn_ok = rnd() & 1;
        F.u.kb.ns_flash_gobj = (rnd() & 1) ? &SPAWNED : NULL;
        F.death2_cb = NULL;
        F.take_dmg_cb = NULL;
        fpx = fpy = fpz = fdir = 0;
        pos.x = pos.y = pos.z = 9.0f;
        ftKb_NsSpecialNStart_Anim_inline(&G, &pos);
        printf("%d %08x %08x %08x %08x %08x %08x %08x %d %d\n", i, b(pos.x), b(pos.y), b(pos.z), b(fpx), b(fpy), b(fpz),
               b(fdir), F.u.kb.ns_flash_gobj == &SPAWNED, F.death2_cb != NULL && F.take_dmg_cb != NULL);
    }
    return 0;
}
