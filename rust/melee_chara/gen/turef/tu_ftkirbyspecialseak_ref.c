/* C reference for src/tu/ftkirbyspecialseak.rs: fn_80106DB0 (5 fused sites) and
 * ftKb_SpecialNSk_8010603C (1), with the needle-offset table and perm_randi, cut verbatim from
 * the game by tu_shard329.py. Callees outside this TU are recording stand-ins both sides share. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftKirby/forward.h"
#include "ft/kinds/ftKirby/types.h"
#include <melee/ef/efsync.h>
#include <melee/it/kinds/itseakneedleheld.h>
#include <melee/it/kinds/itseakneedlethrown.h>
#include <sysdolphin/baselib/random.h>
#include <stdarg.h>
#include <stdio.h>

ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
static u32 s = 0x5EA6u;
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
static u32 rr = 777;
s32 HSD_Randi(s32 n) { rr = rr * 1664525u + 1013904223u; return (s32) ((rr >> 8) % (u32) n); }
static HSD_GObj NEEDLE;
static u32 nl = 99;
Item_GObj* it_802AFD8C(Item_GObj* g, Vec3* pos, u32 kind, float facing)
{
    (void) g;
    nl = nl * 1103515245u + 12345u;
    printf(" throw %08x %08x %08x %u %08x", b(pos->x), b(pos->y), b(pos->z), kind, b(facing));
    return ((nl >> 16) & 3) ? &NEEDLE : NULL;
}
void it_802AFEA8(Item_GObj* gobj, Fighter_GObj* owner, u32 flag) { (void) gobj; (void) owner; printf(" arm %u", flag); }
void* efSync_Spawn(s32 gfx_id, HSD_GObj* g, ...)
{
    va_list ap;
    Vec3* p;
    (void) g;
    va_start(ap, g);
    p = va_arg(ap, Vec3*);
    printf(" fx %x %08x %08x %08x", gfx_id, b(p->x), b(p->y), b(p->z));
    va_end(ap);
    return NULL;
}
void ft_PlaySFX(Fighter* fp, enum_t id, u8 vol, u8 pan) { (void) fp; printf(" sfx %x %d %d", id, vol, pan); }

#include "tu_ftkirbyspecialseak_extract.inc"

static Fighter F;
static HSD_GObj G;
static ftKb_DatAttrs DA;

int main(void)
{
    int i;
    G.user_data = &F;
    F.dat_attrs = &DA;
    for (i = 0; i < 3000; i++) {
        DA.specialn_sk_graphic_x_offset_ground = rf();
        DA.specialn_sk_graphic_y_offset_ground = rf();
        DA.specialn_sk_graphic_x_offset_air = rf();
        DA.specialn_sk_graphic_y_offset_air = rf();
        F.x34_scale.y = rf();
        F.facing_dir = (i & 2) ? 1.0f : -1.0f;
        F.cur_pos.x = rf(); F.cur_pos.y = rf(); F.cur_pos.z = rf();
        F.ground_or_air = (rnd() & 1) ? GA_Ground : GA_Air;
        F.u.kb.xB4 = rnd() % 7;
        F.u.kb.xB8 = (rnd() & 3) ? (Item_GObj*) &NEEDLE : NULL;
        F.mv.kb.specialhi.x4 = (rnd() % 4) != 0;
        printf("%d", i);
        if (i & 1) {
            ftKb_SpecialNSk_8010603C(&G);
        } else {
            fn_80106DB0(&G);
        }
        printf(" | %d %d %d\n", F.u.kb.xB4, F.u.kb.xB8 != NULL, F.mv.kb.specialhi.x4);
    }
    return 0;
}
