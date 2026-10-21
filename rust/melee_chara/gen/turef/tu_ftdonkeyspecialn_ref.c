/* C reference for src/tu/ftdonkeyspecialn.rs: the Giant Punch Enter / Anim / IASA functions cut
 * verbatim from ftdonkeyspecialn.c by tu_shard281.py. Everything they call outside the TU is a
 * recording stand-in both sides share (the Rust test installs the same-format hooks). */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/ftanim.h"
#include "ft/ftcoll.h"
#include "ft/ftcolanim.h"
#include "ft/ftcommon.h"
#include "ft/ft_0892.h"
#include "ft/kinds/ftCommon/ftCo_Escape.h"
#include "ft/kinds/ftCommon/ftCo_Fall.h"
#include "ft/kinds/ftCommon/ftCo_FallSpecial.h"
#include "ft/kinds/ftDonkey/forward.h"
#include "ft/kinds/ftDonkey/ftdonkey.h"
#include "ft/kinds/ftDonkey/ftdonkeyspecialn.h"
#include "ft/kinds/ftDonkey/types.h"
#include "ef/eflib.h"
#include "ef/efsync.h"
#include <stdio.h>

void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }

static char L[512];
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
static u32 s = 0xD0DE5A1u;
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
static u32 b(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

static int frames_remaining;
void ftDk_Init_8010D774(HSD_GObj* g) { (void) g; }
void efLib_PauseAll(HSD_GObj* g) { (void) g; }
void efLib_ResumeAll(HSD_GObj* g) { (void) g; }
void ftDk_SpecialN_DestroyAllEffects(HSD_GObj* g) { (void) g; }
void ftDk_SpecialN_DestroyAllEffectsPlus(HSD_GObj* g) { (void) g; }
void efLib_DestroyAll(HSD_GObj* g) { (void) g; }
void* efSync_Spawn(s32 id, HSD_GObj* g, ...) { (void) g; lg_("E", id, 10, ' '); return 0; }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { (void) g; return frames_remaining; }
void ftAnim_8006EBA4(Fighter_GObj* g) { (void) g; ls("Z "); }
void Fighter_ChangeMotionState(Fighter_GObj* g, int ms, u32 flags, float a, float sp, float bl, Fighter_GObj* x)
{
    (void) g; (void) flags; (void) a; (void) sp; (void) bl; (void) x;
    lg_("M", ms, 10, ' ');
}
void ft_8008A2BC(Fighter_GObj* g) { (void) g; ls("W "); }
void ftCo_Fall_Enter(Fighter_GObj* g) { (void) g; ls("F "); }
void ftCo_80096900(Fighter_GObj* g, int a, int bb, int c, float d, float lag)
{
    (void) g; (void) a; (void) bb; (void) c; (void) d;
    lg_("L", (long) b(lag), 16, ' ');
}
void ftColl_8007ABD0(HitCapsule* h, u32 dmg, HSD_GObj* g) { (void) h; (void) g; lg_("D", (long) dmg, 10, ' '); }
bool ftCo_800BFFD0(Fighter* fp, enum_t a, bool bb) { (void) fp; lg_("C", (long) a, 10, ' '); return 1; }
void ftCommon_8007D7FC(Fighter* fp) { (void) fp; ls("G "); }
static int escape_ret;
bool ftCo_8009917C(Fighter_GObj* g) { (void) g; return escape_ret; }

#include "tu_ftdonkeyspecialn_extract.inc"

static void dummy_cb(HSD_GObj* g) { (void) g; }
static Fighter F;
static HSD_GObj G;
static ftDonkeyAttributes DA;
static Fighter_Part PARTS[2];

int main(void)
{
    int i;
    G.user_data = &F;
    F.dat_attrs = &DA;
    F.parts = PARTS;
    for (i = 0; i < 6000; i++) {
        int k = i % 11;
        Ln = 0; L[0] = 0;
        DA.SpecialN.x2C_MAX_ARM_SWINGS = 1 + rnd() % 5;
        DA.SpecialN.x30_DAMAGE_PER_SWING = rnd() % 5;
        DA.SpecialN.x34_PUNCH_HORIZONTAL_VEL = rf();
        DA.SpecialN.x38_LANDING_LAG = (rnd() & 1) ? 0.0f : rf();
        F.u.dk.x222C = rnd() % 6;
        F.mv.dk.specialn.x0 = rnd() % 2;
        F.mv.dk.specialn.x4 = rnd() % 3;
        F.mv.dk.specialn.x8 = rnd() % 3;
        F.mv.dk.specialn.xC = rnd() % 6;
        F.mv.dk.specialn.x10 = (rnd() & 1) ? -1 : (int) (rnd() % 30);
        F.mv.dk.specialn.x14 = (rnd() & 1) ? -1 : (int) (rnd() % 30);
        F.cmd_vars[0] = rnd() % 2;
        F.ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
        F.x914[0].state = rnd() % 2;
        F.x914[0].damage = rf();
        F.x914[1].damage = rf();
        F.cur_anim_frame = (rnd() & 1) ? 0.0f : 1.0f;
        F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
        F.gr_vel = 0;
        F.self_vel.y = 5.0f;
        F.input.pressed_buttons = rnd() & 0x80000200u;
        F.take_dmg_cb = 0;
        F.death2_cb = 0;
        F.take_dmg_2_cb = (rnd() & 1) ? dummy_cb : 0;
        F.pre_hitlag_cb = (rnd() & 1) ? dummy_cb : 0;
        F.post_hitlag_cb = (rnd() & 1) ? dummy_cb : 0;
        frames_remaining = rnd() % 2;
        escape_ret = rnd() % 2;
        switch (k) {
        case 0: ftDk_SpecialN_Enter(&G); break;
        case 1: ftDk_SpecialAirN_Enter(&G); break;
        case 2: ftDk_SpecialNLoop_Anim(&G); break;
        case 3: ftDk_SpecialN_Anim(&G); break;
        case 4: ftDk_SpecialNFull_Anim(&G); break;
        case 5: ftDk_SpecialAirNLoop_Anim(&G); break;
        case 6: ftDk_SpecialAirN_Anim(&G); break;
        case 7: ftDk_SpecialAirNFull_Anim(&G); break;
        case 8: ftDk_SpecialNLoop_IASA(&G); break;
        case 9: ftDk_SpecialN_Anim(&G); ftDk_SpecialN_Anim(&G); break;
        default: ftDk_SpecialAirN_Enter(&G); ftDk_SpecialN_Enter(&G); break;
        }
        printf("%d %d %08x %d %d %d %d %d %d %d %d%d%d%d | %s\n", i, k, b(F.gr_vel), F.mv.dk.specialn.x0,
               F.mv.dk.specialn.x4, F.mv.dk.specialn.x8, F.mv.dk.specialn.xC, F.mv.dk.specialn.x10,
               F.mv.dk.specialn.x14, F.u.dk.x222C, F.take_dmg_cb != 0, F.death2_cb != 0,
               F.take_dmg_2_cb != 0, F.pre_hitlag_cb != 0 || F.post_hitlag_cb != 0, L);
    }
    return 0;
}
