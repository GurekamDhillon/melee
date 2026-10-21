/* Round 8 part of the melee_items C oracle; #included by oracle.c after round7.c and reusing
 * round 2's LCG / Item fill+snap and round 4's r4_pre / r4_h. Checks src/itfwd8.rs (501
 * generated empty / constant-return state callbacks) and src/itkinds8.rs (7 hand ports):
 * round8_bodies.h holds verbatim copies of the decomp C, extracted by gen_round8.py. Built like
 * the rest: contraction on + gwtool lowering (gen_expected.sh). */
typedef struct StarRodAttributes {
    int x0;
    Vec x4;
} StarRodAttributes;

#include "round8_bodies.h"

static void round8(void)
{
#include "round8_fwd.inc"
    /* ---- hand ports (src/itkinds8.rs) */
    T(2801, "R8H_GET_MIN_LIFE", { ret = bits(get_min_life()); });
    T(2802, "R8H_802A13EC_INLINE", { r4_pre(ip); ret = bits(it_802A13EC_inline(ip)); ret = H(ret, r4_h(ip)); });
    T(2803, "R8H_802A20E8_INLINE", { r4_pre(ip); ret = bits(it_802A20E8_inline(ip)); ret = H(ret, r4_h(ip)); });
    T(2804, "R8H_802EAAEC_INLINE", { r4_pre(ip); ret = bits(it_802EAAEC_inline(ip)); ret = H(ret, r4_h(ip)); });
    T(2805, "R8H_80292394", { r4_pre(ip); it_80292394(g, &ov); ret = H(ret, r4_h(ip)); });
    T(2806, "R8H_DOSEI_M4_ANIMSPEED", { itDoseiAttributes a; memset(&a, 0, sizeof a); a.unk0 = RF(); a.unk4 = (s32) R(); ret = bits(itDosei_UnkMotion4_GetAnimSpeed(&a)); });
    T(2807, "R8H_LINKBOOMERANG_X18", { itLinkBoomerangAttributes a; memset(&a, 0, sizeof a); a.x18 = RF(); ret = bits(itLinkboomerang_get_x18(&a)); });
}
