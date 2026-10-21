/* C reference for src/tu/ftCo_CapturePulled.rs and ftCo_CaptureKirby.rs: the arithmetic
 * functions cut verbatim (tu_shard121.py). Callees are stubs whose outputs come from the same
 * generator the Rust tests run; every result is printed as bits. */
#include <Runtime/platform.h>
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/ftparts.h"
#include "ft/inlines.h"
#include "ft/kinds/ftCommon/ftCo_CapturePulled.h"
#include "ft/kinds/ftCommon/ftCo_CaptureKirby.h"
#include "ft/kinds/ftKirby/ftkirby.h"
#include "lb/lb_00B0.h"
#include "lb/lbvector.h"
#include "pl/player.h"
#include <sysdolphin/baselib/jobj.h>
#include <stdio.h>

void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

static ftCommonData CD;
ftCommonData* p_ftCommonData = &CD;
static Fighter F, V;
static HSD_GObj G, VG;
static HSD_JObj J, HOLD, XROT;
static FighterBone PB[4];
static int SLOT, HC, LBN, DIRTY;
static Vec3 LBV[2], KBV;

s32 Player_80033BB8(int slot) { (void) slot; return SLOT; }
int Player_GetHandicap(int slot) { (void) slot; return HC; }
Fighter_Part ftParts_GetBoneIndex(Fighter* fp, Fighter_Part part) { (void) fp; (void) part; return 1; }
void lb_8000B1CC(HSD_JObj* j, Vec3* off, Vec3* out) { (void) j; (void) off; *out = LBV[LBN++ & 1]; }
void ftKb_SpecialN_800F5B5C(Fighter_GObj* g, Vec3* out) { (void) g; *out = KBV; }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { (void) j; DIRTY++; }

#include "tu_shard121_extract.inc"

static u32 rs = 0x121C0DE5u;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    switch (rnd() % 10) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    default: return (float) ((int) (rnd() % 20001) - 10000) / 1013.0f;
    }
}

int main(void)
{
    int i;
    G.user_data = &F;
    G.hsd_obj = &J;
    VG.user_data = &V;
    F.victim_gobj = &VG;
    F.parts = PB;
    PB[1].joint = &XROT;
    for (i = 0; i < 6000; i++) {
        u32 op = rnd() % 7;
        float r = 0;
        LBN = 0;
        DIRTY = 0;
        CD.x354 = rf(); CD.x358 = rf(); CD.x35C = rf(); CD.x360 = rf(); CD.x364 = rf(); CD.x368 = rf();
        CD.x3C4 = rf();
        SLOT = (int) (rnd() % 6) - 1;
        HC = (int) (rnd() % 10);
        F.dmg.x1830_percent = rf();
        F.facing_dir = rf();
        F.x1A70.y = rf(); F.x1A70.z = rf();
        F.x34_scale.y = rf();
        F.cur_pos.x = rf(); F.cur_pos.y = rf(); F.cur_pos.z = rf();
        F.ground_or_air = (rnd() & 1) ? GA_Ground : GA_Air;
        F.x2170 = rf();
        V.cur_pos.y = rf();
        V.mv.co.capturedamage.x18 = &HOLD;
        LBV[0].x = rf(); LBV[0].y = rf(); LBV[0].z = rf();
        LBV[1].x = rf(); LBV[1].y = rf(); LBV[1].z = rf();
        KBV.x = rf(); KBV.y = rf(); KBV.z = rf();
        F.mv.co.capturekirby.pos_offset.x = rf();
        F.mv.co.capturekirby.pos_offset.y = rf();
        F.mv.co.capturekirby.x8.x = rf();
        F.mv.co.capturekirby.x8.y = rf();
        F.mv.co.capturekirby.x10.x = rf();
        F.mv.co.capturekirby.x10.y = rf();
        F.mv.co.capturekirby.scale.x = rf();
        F.mv.co.capturekirby.scale.y = rf();
        F.mv.co.capturekirby.scale.z = rf();
        J.flags = rnd() & (JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX | JOBJ_MTX_INDEP_SRT);
        J.scale.x = J.scale.y = J.scale.z = 7.0f;
        switch (op) {
        case 0: r = ftCo_800DA824(&F); break;
        case 1: fn_800DAA40(&G, &VG); break;
        case 2: r = (float) fn_800DAD18(&G); break;
        case 3: ftCo_CapturePulledHi_Phys(&G); break;
        case 4: ftCo_800DB464(&G); break;
        case 5: { Vec3 p; p.x = rf(); p.y = rf(); p.z = rf(); r = ftCo_800BD19C(&G, &p); break; }
        default: ftCo_800BD39C(&G); break;
        }
        printf("%d %u %08X %08X %08X %08X %08X %08X %08X %08X %08X %08X %08X %d\n", i, op, B(r),
               B(F.cur_pos.x), B(F.cur_pos.y), B(F.cur_pos.z), B(V.x2170),
               B(F.mv.co.capturekirby.pos_offset.x), B(F.mv.co.capturekirby.pos_offset.y),
               B(J.scale.x), B(J.scale.y), B(J.scale.z), (u32) J.flags, DIRTY);
    }
    return 0;
}
