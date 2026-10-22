/* C reference for src/tu/ftkirbyspecialpikachu.rs: ftKb_PkSpecialN_Anim / ftKb_PkSpecialAirN_Anim cut
 * verbatim by tu_shard321.py (the jolt spawn position is the fused part). Callees are logging stand-ins. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/kinds/ftKirby/forward.h"
#include "ft/kinds/ftKirby/types.h"
#include "ft/kinds/ftPikachu/forward.h"
#include "ft/kinds/ftPikachu/types.h"
#include <placeholder.h>
#include <stdio.h>

ftCommonData* p_ftCommonData;
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static u32 ev[16];
static int nev;
static float px, py, pz, pdir;
static int frames_remaining;
static void E(u32 v) { if (nev < 16) ev[nev++] = v; }

Item_GObj* itPikachuThunderJolt_Spawn(Item_GObj* o, Vec3* p, float dir, ItemKind kind)
{
    (void) o; px = p->x; py = p->y; pz = p->z; pdir = dir; E(0x10); E((u32) kind);
    return NULL;
}
void ft_PlaySFX(Fighter* fp, enum_t id, int vol, int pan) { (void) fp; E(0x20); E((u32) id); E((u32) vol); E((u32) pan); }
bool ftAnim_IsFramesRemaining(HSD_GObj* g) { (void) g; return frames_remaining; }
void ft_8008A2BC(HSD_GObj* g) { (void) g; E(0x30); }
void ftCo_Fall_Enter(HSD_GObj* g) { (void) g; E(0x31); }
void ftCo_80096900(HSD_GObj* g, bool a, bool b, bool c, f32 d, f32 e)
{
    (void) g; E(0x32); E(a); E(b); E(c); E(*(u32*) &d); E(*(u32*) &e);
}

#include "tu_ftkirbyspecialpikachu_extract.inc"

static Fighter F;
static HSD_GObj G;
static ftKb_DatAttrs DA;
static u32 s = 0x91CAu;
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
    printf("%s %d %08x %08x %08x %08x %u %u |", tag, i, b(px), b(py), b(pz), b(pdir), F.cmd_vars[0], F.cmd_vars[1]);
    for (k = 0; k < nev; k++) printf(" %x", ev[k]);
    printf("\n");
    nev = 0;
    px = py = pz = pdir = 0;
}

int main(void)
{
    int i;
    G.user_data = &F;
    F.dat_attrs = &DA;
    for (i = 0; i < 2000; i++) {
        int k;
        DA.specialn_pk_spawn_offset.x = rf();
        DA.specialn_pk_spawn_offset.y = rf();
        DA.specialn_pc_spawn_offset.x = rf();
        DA.specialn_pc_spawn_offset.y = rf();
        DA.specialairn_pk_spawn_offset.x = rf();
        DA.specialairn_pk_spawn_offset.y = rf();
        DA.specialairn_pc_spawn_offset.x = rf();
        DA.specialairn_pc_spawn_offset.y = rf();
        DA.specialn_pk_itkind = rnd() % 7;
        DA.specialn_pc_itkind = rnd() % 7;
        DA.specialairn_pk_landing_lag = (rnd() % 3) ? rf() : 0.0f;
        DA.specialairn_pc_landing_lag = (rnd() % 3) ? rf() : 0.0f;
        F.x34_scale.y = rf();
        F.cur_pos.x = rf();
        F.cur_pos.y = rf();
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        k = rnd() % 3;
        F.u.kb.hat.kind = (k == 0) ? Ft_Kind_Pikachu : (k == 1) ? Ft_Kind_Pichu : Ft_Kind_Kirby;
        frames_remaining = rnd() & 1;
        F.cmd_vars[0] = rnd() % 3;
        F.cmd_vars[1] = rnd() % 3;
        ftKb_PkSpecialN_Anim(&G);
        flush("G", i);
        F.cmd_vars[0] = rnd() % 3;
        F.cmd_vars[1] = rnd() % 3;
        ftKb_PkSpecialAirN_Anim(&G);
        flush("H", i);
    }
    return 0;
}
