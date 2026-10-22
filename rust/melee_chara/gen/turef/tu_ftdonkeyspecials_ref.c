/* C reference for src/tu/ftdonkeyspecials.rs: ftDk_SpecialAirS_Enter (the x-momentum divide) and
 * ftDk_SpecialAirS_Phys (which attribute feeds which ftCommon call) cut verbatim from
 * ftdonkeyspecials.c by tu_shard281.py. The ftCommon / ftAnim callees are recording stand-ins
 * both sides share. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/ftanim.h"
#include "ft/ftcommon.h"
#include "ft/kinds/ftDonkey/forward.h"
#include "ft/kinds/ftDonkey/ftdonkeyspecials.h"
#include "ft/kinds/ftDonkey/ftdonkeyspeciallw.h"
#include "ft/kinds/ftDonkey/types.h"
#include <stdio.h>

void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static char L[512];
static u32 b(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static int Ln;
static void lc(char c) { L[Ln++] = c; L[Ln] = 0; }
static void ls(const char* t) { while (*t) lc(*t++); }
static void lu(unsigned long v, int base)
{
    char t[24];
    int n = 0;
    if (v == 0) t[n++] = '0';
    while (v) { t[n++] = "0123456789abcdef"[v % base]; v /= base; }
    while (n) lc(t[--n]);
}
/* tag letters + number (decimal or hex), then a space or comma */
static void lg_(const char* tag, long a, int base, char sep) { ls(tag); lu((unsigned long) (unsigned int) a, base); lc(sep); }
static u32 s = 0xD0D0A1Bu;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    switch (rnd() % 8) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    default: return (float) ((int) (rnd() % 4001) - 2000) / 97.0f;
    }
}

void ftAnim_8006EBA4(Fighter_GObj* g) { (void) g; ls("Z "); }
void Fighter_ChangeMotionState(Fighter_GObj* g, int ms, u32 flags, float a, float sp, float bl, Fighter_GObj* x)
{
    (void) g; (void) flags; (void) a; (void) sp; (void) bl; (void) x;
    lg_("M", ms, 10, ' ');
}
f32 Geno_FallLimit(Fighter* fp, f32 terminal) { (void) fp; return terminal; }
void ftDk_SpecialLw_8010E0CC(HSD_GObj* g) { (void) g; }
void ftDk_SpecialLw_8010E148(HSD_GObj* g) { (void) g; }

#include "tu_ftdonkeyspecials_extract.inc"

static Fighter F;
static HSD_GObj G;
static ftDonkeyAttributes DA;

int main(void)
{
    int i;
    G.user_data = &F;
    F.dat_attrs = &DA;
    for (i = 0; i < 4000; i++) {
        Ln = 0; L[0] = 0;
        DA.SpecialS.x3C_MIN_STICK_X_MOMENTUM = rf();
        DA.SpecialS.x40_MOMENTUM_TRANSITION_MODIFIER = rf();
        DA.SpecialS.x44_AERIAL_GRAVITY = rf();
        F.co_attrs.terminal_velocity = rf();
        F.cmd_vars[0] = rnd() % 2;
        F.self_vel.x = rf();
        F.self_vel.y = rf();
        F.x74_self_accel.x = rf();
        F.accessory4_cb = 0;
        if (i & 1) {
            ftDk_SpecialAirS_Enter(&G);
        } else {
            ftDk_SpecialAirS_Phys(&G);
        }
        printf("%d %08x %08x %08x %d %d | %s\n", i, b(F.self_vel.x), b(F.self_vel.y), b(F.x74_self_accel.x),
               F.cmd_vars[0], F.accessory4_cb != 0, L);
    }
    return 0;
}
