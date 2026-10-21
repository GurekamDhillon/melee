/* Round 4 (night 3) part of the melee_items C oracle; #included by oracle.c after round3.c and
 * reusing round 2's LCG, Item fill/snap and fill_ext. Checks src/itkinds.rs: the bodies in
 * round4_bodies.h are verbatim copies of the decomp C (per-kind state callbacks).
 *
 * Each case fills the Item and external data like round 2, then r4_pre() additionally draws
 * ground_or_air, the four item-command vars, anim frame/speed, xCCC, xCA8, the ItemAttr fall
 * speeds, the ItemCommonData floats the callees read, a 256-word special-attributes block
 * (article x4_specialAttributes) and every 32-bit word of the item-var union (zero, a small int,
 * or a float). After the call it hashes snap(Item) + ret + those fields + every item-var word.
 * Built like the rest: contraction on + gwtool lowering (gen_expected.sh). */
#include "melee/it/kinds/itkinoko.h"
#include "melee/it/kinds/itdosei.h"
#define it_804D6D28 it_804D6D28_o

#include "round4_bodies.h"

static u32 r4_sa[256];
static void r4_pre(Item* ip)
{
    u32* w = (u32*) &ip->xDD4_itemVar;
    u32 k, n = (u32) (sizeof ip->xDD4_itemVar / 4);
    ip->ground_or_air = (GroundOrAir) (R() % 3);
    ip->xDAC_itcmd_var0 = (R() & 1) ? 0 : R();
    ip->xDB0_itcmd_var1 = (R() & 1) ? 0 : R();
    ip->xDB4_itcmd_var2 = (R() & 1) ? 0 : R();
    ip->xDB8_itcmd_var3 = (R() & 1) ? 0 : R();
    ip->x5CC_currentAnimFrame = (R() & 3) ? RF() : 16.0f;
    ip->x5D0_animFrameSpeed = RF();
    ip->xCCC_incDamageDirection = RF();
    ip->xCA8 = (s32) R();
    r2_attr.x10_fall_speed = (R() & 3) ? RM() : RF();
    r2_attr.x14_fall_speed_max = (R() & 3) ? RM() : RF();
    r2_common.x4C_float = RF();
    r2_common.x58_float = RM(); r2_common.x5C_float = RM(); r2_common.x60_float = RM();
    for (k = 0; k < 256; k++) r4_sa[k] = (R() & 3) ? bits(RM()) : bits(RF());
    r2_art.x4_specialAttributes = r4_sa;
    for (k = 0; k < n; k++) {
        u32 s = R() % 4;
        w[k] = s == 0 ? 0 : s == 1 ? R() % 4 : s == 2 ? bits(RF()) : bits(RM());
    }
}
static u32 r4_h(Item* ip)
{
    u32* w = (u32*) &ip->xDD4_itemVar;
    u32 k, n = (u32) (sizeof ip->xDD4_itemVar / 4);
    u32 h = H(H(H(H(H(0, (u32) ip->ground_or_air), ip->xDAC_itcmd_var0), ip->xDB0_itcmd_var1),
                ip->xDB4_itcmd_var2), ip->xDB8_itcmd_var3);
    h = H(H(H(H(h, bits(ip->x5CC_currentAnimFrame)), bits(ip->x5D0_animFrameSpeed)),
            bits(ip->xCCC_incDamageDirection)), (u32) ip->xCA8);
    for (k = 0; k < n; k++) h = H(h, w[k]);
    return h;
}

static void round4(void)
{
    printf("pub const R4_ITEMVAR_BYTES: usize = %u;\n", (unsigned) sizeof(((Item*) 0)->xDD4_itemVar));
    T(301, "R4_ITBOMBHEI_UNKMOTION1_PHYS", { r4_pre(ip);  itBombhei_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(302, "R4_ITCLIMBERSICE_UNKMOTION1_PHYS", { r4_pre(ip);  itClimbersice_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(303, "R4_ITDOSEI_UNKMOTION3_PHYS", { r4_pre(ip);  itDosei_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(304, "R4_ITDOSEI_UNKMOTION8_PHYS", { r4_pre(ip);  itDosei_UnkMotion8_Phys(g); ret = H(ret, r4_h(ip)); });
    T(305, "R4_ITDOSEI_UNKMOTION11_PHYS", { r4_pre(ip);  itDosei_UnkMotion11_Phys(g); ret = H(ret, r4_h(ip)); });
    T(306, "R4_ITEVYOSHIEGG_UNKMOTION1_PHYS", { r4_pre(ip);  itEvyoshiegg_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(307, "R4_ITFFLOWER_UNKMOTION1_PHYS", { r4_pre(ip);  itFflower_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(308, "R4_ITFFLOWER_UNKMOTION6_PHYS", { r4_pre(ip);  itFflower_UnkMotion6_Phys(g); ret = H(ret, r4_h(ip)); });
    T(309, "R4_ITFFLOWERFLAME_UNKMOTION3_PHYS", { r4_pre(ip);  itFflowerflame_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(310, "R4_ITFLIPPER_FALLING_PHYS", { r4_pre(ip);  itFlipper_Falling_Phys(g); ret = H(ret, r4_h(ip)); });
    T(311, "R4_ITFREEZE_UNKMOTION1_PHYS", { r4_pre(ip);  itFreeze_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(312, "R4_ITFUSHIGIBANA_UNKMOTION2_PHYS", { r4_pre(ip);  itFushigibana_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(313, "R4_ITGSHELL_UNKMOTION1_PHYS", { r4_pre(ip);  itGshell_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(314, "R4_ITGSHELL_UNKMOTION3_PHYS", { r4_pre(ip);  itGshell_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(315, "R4_ITGSHELL_UNKMOTION4_PHYS", { r4_pre(ip);  itGshell_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(316, "R4_ITGSHELL_UNKMOTION8_PHYS", { r4_pre(ip);  itGshell_UnkMotion8_Phys(g); ret = H(ret, r4_h(ip)); });
    T(317, "R4_ITHARISEN_UNKMOTION1_PHYS", { r4_pre(ip);  itHarisen_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(318, "R4_ITHEART_UNKMOTION3_PHYS", { r4_pre(ip);  itHeart_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(319, "R4_ITHINOARASHI_UNKMOTION2_PHYS", { r4_pre(ip);  itHinoarashi_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(320, "R4_ITKUSUDAMA_UNKMOTION2_PHYS", { r4_pre(ip);  itKusudama_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(321, "R4_ITLEADEAD_UNKMOTION10_PHYS", { r4_pre(ip);  itLeadead_UnkMotion10_Phys(g); ret = H(ret, r4_h(ip)); });
    T(322, "R4_ITLEADEAD_UNKMOTION14_PHYS", { r4_pre(ip);  itLeadead_UnkMotion14_Phys(g); ret = H(ret, r4_h(ip)); });
    T(323, "R4_ITLIKELIKE_UNKMOTION19_PHYS", { r4_pre(ip);  itLikelike_UnkMotion19_Phys(g); ret = H(ret, r4_h(ip)); });
    T(324, "R4_ITLIPSTICK_UNKMOTION1_PHYS", { r4_pre(ip);  itLipstick_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(325, "R4_ITLUCKY_UNKMOTION5_PHYS", { r4_pre(ip);  itLucky_UnkMotion5_Phys(g); ret = H(ret, r4_h(ip)); });
    T(326, "R4_ITLUCKY_UNKMOTION6_PHYS", { r4_pre(ip);  itLucky_UnkMotion6_Phys(g); ret = H(ret, r4_h(ip)); });
    T(327, "R4_ITLUCKY_UNKMOTION3_PHYS", { r4_pre(ip);  itLucky_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(328, "R4_ITMARIL_UNKMOTION2_PHYS", { r4_pre(ip);  itMaril_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(329, "R4_ITMARUMINE_UNKMOTION2_PHYS", { r4_pre(ip);  itMarumine_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(330, "R4_ITMARUMINE_UNKMOTION4_PHYS", { r4_pre(ip);  itMarumine_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(331, "R4_ITMSBOMB_UNKMOTION1_PHYS", { r4_pre(ip);  itMsbomb_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(332, "R4_ITOCTAROCK_UNKMOTION2_PHYS", { r4_pre(ip);  itOctarock_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(333, "R4_ITOCTAROCK_UNKMOTION6_PHYS", { r4_pre(ip);  itOctarock_UnkMotion6_Phys(g); ret = H(ret, r4_h(ip)); });
    T(334, "R4_ITOLDKURI_UNKMOTION3_PHYS", { r4_pre(ip);  itOldkuri_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(335, "R4_ITOLDKURI_UNKMOTION4_PHYS", { r4_pre(ip);  itOldkuri_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(336, "R4_ITOLDKURI_UNKMOTION6_PHYS", { r4_pre(ip);  itOldkuri_UnkMotion6_Phys(g); ret = H(ret, r4_h(ip)); });
    T(337, "R4_ITOLDKURI_UNKMOTION8_PHYS", { r4_pre(ip);  itOldkuri_UnkMotion8_Phys(g); ret = H(ret, r4_h(ip)); });
    T(338, "R4_ITOLDOTTOSEA_UNKMOTION4_PHYS", { r4_pre(ip);  itOldottosea_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(339, "R4_ITPATAPATA_UNKMOTION6_PHYS", { r4_pre(ip);  itPatapata_UnkMotion6_Phys(g); ret = H(ret, r4_h(ip)); });
    T(340, "R4_ITRSHELL_UNKMOTION1_PHYS", { r4_pre(ip);  itRshell_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(341, "R4_ITRSHELL_UNKMOTION3_PHYS", { r4_pre(ip);  itRshell_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(342, "R4_ITRSHELL_UNKMOTION4_PHYS", { r4_pre(ip);  itRshell_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(343, "R4_ITRSHELL_UNKMOTION6_PHYS", { r4_pre(ip);  itRshell_UnkMotion6_Phys(g); ret = H(ret, r4_h(ip)); });
    T(344, "R4_ITSAMUSBOMB_UNKMOTION1_PHYS", { r4_pre(ip);  itSamusbomb_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(345, "R4_ITSCBALL_UNKMOTION1_PHYS", { r4_pre(ip);  itScball_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(346, "R4_ITSPYCLOAK_UNKMOTION1_PHYS", { r4_pre(ip);  itSpycloak_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(347, "R4_ITSPYCLOAK_UNKMOTION3_PHYS", { r4_pre(ip);  itSpycloak_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(348, "R4_ITSTAR_UNKMOTION0_PHYS", { r4_pre(ip);  itStar_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(349, "R4_ITTARU_UNKMOTION1_PHYS", { r4_pre(ip);  itTaru_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(350, "R4_ITTARUCANN_UNKMOTION1_PHYS", { r4_pre(ip);  itTarucann_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(351, "R4_ITTINCLE_UNKMOTION8_PHYS", { r4_pre(ip);  itTincle_UnkMotion8_Phys(g); ret = H(ret, r4_h(ip)); });
    T(352, "R4_ITTOMATO_UNKMOTION4_PHYS", { r4_pre(ip);  itTomato_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(353, "R4_ITTOSAKINTO_UNKMOTION3_PHYS", { r4_pre(ip);  itTosakinto_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(354, "R4_ITWHISPYAPPLE_UNKMOTION5_PHYS", { r4_pre(ip);  itWhispyapple_UnkMotion5_Phys(g); ret = H(ret, r4_h(ip)); });
    T(355, "R4_ITOLDOTTOSEA_UNKMOTION9_PHYS", { r4_pre(ip);  itOldottosea_UnkMotion9_Phys(g); ret = H(ret, r4_h(ip)); });
    T(356, "R4_ITWHITEBEA_UNKMOTION3_PHYS", { r4_pre(ip);  itWhitebea_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(357, "R4_ITWHITEBEA_UNKMOTION6_PHYS", { r4_pre(ip);  itWhitebea_UnkMotion6_Phys(g); ret = H(ret, r4_h(ip)); });
    T(358, "R4_ITWHITEBEA_UNKMOTION9_PHYS", { r4_pre(ip);  itWhitebea_UnkMotion9_Phys(g); ret = H(ret, r4_h(ip)); });
    T(359, "R4_ITWSTAR_UNKMOTION1_PHYS", { r4_pre(ip);  itWstar_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(360, "R4_ITYOSHIEGGTHROW_UNKMOTION1_PHYS", { r4_pre(ip);  itYoshieggthrow_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(361, "R4_ITZRSHELL_UNKMOTION1_PHYS", { r4_pre(ip);  itZrshell_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(362, "R4_ITZRSHELL_UNKMOTION3_PHYS", { r4_pre(ip);  itZrshell_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(363, "R4_ITZRSHELL_UNKMOTION4_PHYS", { r4_pre(ip);  itZrshell_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(364, "R4_ITZRSHELL_UNKMOTION8_PHYS", { r4_pre(ip);  itZrshell_UnkMotion8_Phys(g); ret = H(ret, r4_h(ip)); });
    T(365, "R4_ITBAT_LOGIC11_CLANKED", { r4_pre(ip);  ret = (u32) itBat_Logic11_Clanked(g); ret = H(ret, r4_h(ip)); });
    T(366, "R4_ITBAT_LOGIC11_HITSHIELD", { r4_pre(ip);  ret = (u32) itBat_Logic11_HitShield(g); ret = H(ret, r4_h(ip)); });
    T(367, "R4_ITDOSEI_LOGIC7_CLANKED", { r4_pre(ip);  ret = (u32) itDosei_Logic7_Clanked(g); ret = H(ret, r4_h(ip)); });
    T(368, "R4_ITDOSEI_LOGIC7_HITSHIELD", { r4_pre(ip);  ret = (u32) itDosei_Logic7_HitShield(g); ret = H(ret, r4_h(ip)); });
    T(369, "R4_ITFFLOWER_LOGIC25_DMGDEALT", { r4_pre(ip);  ret = (u32) itFFlower_Logic25_DmgDealt(g); ret = H(ret, r4_h(ip)); });
    T(370, "R4_ITFFLOWER_LOGIC25_CLANKED", { r4_pre(ip);  ret = (u32) itFFlower_Logic25_Clanked(g); ret = H(ret, r4_h(ip)); });
    T(371, "R4_ITFFLOWER_LOGIC25_HITSHIELD", { r4_pre(ip);  ret = (u32) itFFlower_Logic25_HitShield(g); ret = H(ret, r4_h(ip)); });
    T(372, "R4_ITLGUN_LOGIC16_DMGDEALT", { r4_pre(ip);  ret = (u32) itLGun_Logic16_DmgDealt(g); ret = H(ret, r4_h(ip)); });
    T(373, "R4_ITLGUN_LOGIC16_CLANKED", { r4_pre(ip);  ret = (u32) itLGun_Logic16_Clanked(g); ret = H(ret, r4_h(ip)); });
    T(374, "R4_ITLGUN_LOGIC16_HITSHIELD", { r4_pre(ip);  ret = (u32) itLGun_Logic16_HitShield(g); ret = H(ret, r4_h(ip)); });
    T(375, "R4_ITPARASOL_LOGIC13_DMGDEALT", { r4_pre(ip);  ret = (u32) itParasol_Logic13_DmgDealt(g); ret = H(ret, r4_h(ip)); });
    T(376, "R4_ITPARASOL_LOGIC13_CLANKED", { r4_pre(ip);  ret = (u32) itParasol_Logic13_Clanked(g); ret = H(ret, r4_h(ip)); });
    T(377, "R4_ITPARASOL_LOGIC13_HITSHIELD", { r4_pre(ip);  ret = (u32) itParasol_Logic13_HitShield(g); ret = H(ret, r4_h(ip)); });
    T(378, "R4_ITPEACHTURNIP_LOGIC56_CLANKED", { r4_pre(ip);  ret = (u32) itPeachTurnip_Logic56_Clanked(g); ret = H(ret, r4_h(ip)); });
    T(379, "R4_ITPEACHTURNIP_LOGIC56_HITSHIELD", { r4_pre(ip);  ret = (u32) itPeachTurnip_Logic56_HitShield(g); ret = H(ret, r4_h(ip)); });
    T(380, "R4_ITRSHELL_LOGIC15_CLANKED", { r4_pre(ip);  ret = (u32) itRShell_Logic15_Clanked(g); ret = H(ret, r4_h(ip)); });
    T(381, "R4_ITSCBALL_LOGIC30_HITSHIELD", { r4_pre(ip);  ret = (u32) itScball_Logic30_HitShield(g); ret = H(ret, r4_h(ip)); });
    T(382, "R4_ITSSCOPE_LOGIC21_DMGDEALT", { r4_pre(ip);  ret = (u32) itSScope_Logic21_DmgDealt(g); ret = H(ret, r4_h(ip)); });
    T(383, "R4_ITSSCOPE_LOGIC21_CLANKED", { r4_pre(ip);  ret = (u32) itSScope_Logic21_Clanked(g); ret = H(ret, r4_h(ip)); });
    T(384, "R4_ITSSCOPE_LOGIC21_HITSHIELD", { r4_pre(ip);  ret = (u32) itSScope_Logic21_HitShield(g); ret = H(ret, r4_h(ip)); });
    T(385, "R4_ITSWORD_LOGIC12_CLANKED", { r4_pre(ip);  ret = (u32) itSword_Logic12_Clanked(g); ret = H(ret, r4_h(ip)); });
    T(386, "R4_ITSWORD_LOGIC12_HITSHIELD", { r4_pre(ip);  ret = (u32) itSword_Logic12_HitShield(g); ret = H(ret, r4_h(ip)); });
    T(387, "R4_ITKYASARIN_UNKMOTION9_PHYS", { r4_pre(ip);  itKyasarin_UnkMotion9_Phys(g); ret = H(ret, r4_h(ip)); });
    T(388, "R4_ITLEADEAD_UNKMOTION16_PHYS", { r4_pre(ip);  itLeadead_UnkMotion16_Phys(g); ret = H(ret, r4_h(ip)); });
    T(389, "R4_ITLEADEAD_UNKMOTION17_PHYS", { r4_pre(ip);  itLeadead_UnkMotion17_Phys(g); ret = H(ret, r4_h(ip)); });
    T(390, "R4_ITLIKELIKE_UNKMOTION21_PHYS", { r4_pre(ip);  itLikelike_UnkMotion21_Phys(g); ret = H(ret, r4_h(ip)); });
    T(391, "R4_ITOCTAROCK_UNKMOTION8_PHYS", { r4_pre(ip);  itOctarock_UnkMotion8_Phys(g); ret = H(ret, r4_h(ip)); });
    T(392, "R4_ITOCTAROCKSTONE_UNKMOTION1_PHYS", { r4_pre(ip);  itOctarockstone_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(393, "R4_ITOLDKURI_UNKMOTION11_PHYS", { r4_pre(ip);  itOldkuri_UnkMotion11_Phys(g); ret = H(ret, r4_h(ip)); });
    T(394, "R4_ITOLDOTTOSEA_UNKMOTION11_PHYS", { r4_pre(ip);  itOldottosea_UnkMotion11_Phys(g); ret = H(ret, r4_h(ip)); });
    T(395, "R4_ITWHITEBEA_UNKMOTION11_PHYS", { r4_pre(ip);  itWhitebea_UnkMotion11_Phys(g); ret = H(ret, r4_h(ip)); });
    T(396, "R4_ITZRSHELL_UNKMOTION10_PHYS", { r4_pre(ip);  itZrshell_UnkMotion10_Phys(g); ret = H(ret, r4_h(ip)); });
    T(397, "R4_ITENTEI_UNKMOTION0_PHYS", { r4_pre(ip);  itEntei_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(398, "R4_IT_802CA618", { r4_pre(ip);  it_802CA618(g); ret = H(ret, r4_h(ip)); });
    T(399, "R4_ITKAMEX_UNKMOTION2_PHYS", { r4_pre(ip);  itKamex_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(400, "R4_ITPIPPI_UNKMOTION5_PHYS", { r4_pre(ip);  itPippi_UnkMotion5_Phys(g); ret = H(ret, r4_h(ip)); });
    T(401, "R4_ITRAIKOU_UNKMOTION0_PHYS", { r4_pre(ip);  itRaikou_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(402, "R4_ITSUIKUN_UNKMOTION0_PHYS", { r4_pre(ip);  itSuikun_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(403, "R4_ITLEADEAD_UNKMOTION12_PHYS", { r4_pre(ip);  itLeadead_UnkMotion12_Phys(g); ret = H(ret, r4_h(ip)); });
    T(404, "R4_ITNOKONOKO_UNKMOTION5_PHYS", { r4_pre(ip);  itNokonoko_UnkMotion5_Phys(g); ret = H(ret, r4_h(ip)); });
    T(405, "R4_ITOCTAROCK_UNKMOTION3_PHYS", { r4_pre(ip);  itOctarock_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(406, "R4_ITOLDKURI_UNKMOTION9_PHYS", { r4_pre(ip);  itOldkuri_UnkMotion9_Phys(g); ret = H(ret, r4_h(ip)); });
    T(407, "R4_ITOLDOTTOSEA_UNKMOTION5_PHYS", { r4_pre(ip);  itOldottosea_UnkMotion5_Phys(g); ret = H(ret, r4_h(ip)); });
    T(408, "R4_ITWHITEBEA_UNKMOTION4_PHYS", { r4_pre(ip);  itWhitebea_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(409, "R4_IT_2ADA_UNKMOTION0_PHYS", { r4_pre(ip);  it_2ADA_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(410, "R4_IT_2E5A_UNKMOTION1_PHYS", { r4_pre(ip);  it_2E5A_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(411, "R4_ITCOIN_UNKMOTION4_PHYS", { r4_pre(ip);  itCoin_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(412, "R4_ITEGG_UNKMOTION1_PHYS", { r4_pre(ip);  itEgg_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(413, "R4_ITLGUN_UNKMOTION1_PHYS", { r4_pre(ip);  itLgun_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(414, "R4_ITFOODS_UNKMOTION3_PHYS", { r4_pre(ip);  itFoods_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(415, "R4_ITHAMMER_UNKMOTION1_PHYS", { r4_pre(ip);  itHammer_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(416, "R4_ITHAMMER_UNKMOTION4_PHYS", { r4_pre(ip);  itHammer_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(417, "R4_ITHAMMERHEAD_LOGIC40_REFLECTED", { r4_pre(ip);  ret = (u32) itHammerHead_Logic40_Reflected(g); ret = H(ret, r4_h(ip)); });
    T(418, "R4_ITOCTAROCKSTONE_LOGIC4_REFLECTED", { r4_pre(ip);  ret = (u32) itOctarockstone_Logic4_Reflected(g); ret = H(ret, r4_h(ip)); });
    T(419, "R4_ITPEACHTURNIP_LOGIC56_REFLECTED", { r4_pre(ip);  ret = (u32) itPeachTurnip_Logic56_Reflected(g); ret = H(ret, r4_h(ip)); });
    T(420, "R4_ITSTARROD_LOGIC22_DMGDEALT", { r4_pre(ip);  ret = (u32) itStarRod_Logic22_DmgDealt(g); ret = H(ret, r4_h(ip)); });
    T(421, "R4_ITSTARROD_LOGIC22_CLANKED", { r4_pre(ip);  ret = (u32) itStarRod_Logic22_Clanked(g); ret = H(ret, r4_h(ip)); });
    T(422, "R4_ITSTARROD_LOGIC22_HITSHIELD", { r4_pre(ip);  ret = (u32) itStarRod_Logic22_HitShield(g); ret = H(ret, r4_h(ip)); });
    T(423, "R4_ITDKINOKO_UNKMOTION0_PHYS", { r4_pre(ip);  itDkinoko_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(424, "R4_ITKINOKO_UNKMOTION0_PHYS", { r4_pre(ip);  itKinoko_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(425, "R4_ITDKINOKO_UNKMOTION1_PHYS", { r4_pre(ip);  itDkinoko_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(426, "R4_ITKINOKO_UNKMOTION1_PHYS", { r4_pre(ip);  itKinoko_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(427, "R4_ITDOSEI_UNKMOTION6_ANIM", { r4_pre(ip);  ret = (u32) itDosei_UnkMotion6_Anim(g); ret = H(ret, r4_h(ip)); });
    T(428, "R4_ITDOSEI_UNKMOTION8_ANIM", { r4_pre(ip);  ret = (u32) itDosei_UnkMotion8_Anim(g); ret = H(ret, r4_h(ip)); });
    T(429, "R4_IT_802CAE60", { r4_pre(ip);  ret = (u32) it_802CAE60(g); ret = H(ret, r4_h(ip)); });
    T(430, "R4_IT_802CF120", { r4_pre(ip);  ret = (u32) it_802CF120(g); ret = H(ret, r4_h(ip)); });
    T(431, "R4_ITLINKBOMB_UNKMOTION1_PHYS", { r4_pre(ip);  itLinkbomb_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(432, "R4_ITPEACHTURNIP_UNKMOTION1_PHYS", { r4_pre(ip);  itPeachturnip_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(433, "R4_IT_802D5884", { r4_pre(ip);  ret = (u32) it_802D5884(g); ret = H(ret, r4_h(ip)); });
    T(434, "R4_ITMARIL_UNKMOTION2_ANIM_INLINE", { r4_pre(ip);  ret = (u32) itMaril_UnkMotion2_Anim_inline(g); ret = H(ret, r4_h(ip)); });
    T(435, "R4_ITLUIGIFIREBALL_UNKMOTION0_ANIM", { r4_pre(ip);  ret = (u32) itLuigifireball_UnkMotion0_Anim(g); ret = H(ret, r4_h(ip)); });
    T(436, "R4_ITMARIOFIREBALL_UNKMOTION0_ANIM", { r4_pre(ip);  ret = (u32) itMariofireball_UnkMotion0_Anim(g); ret = H(ret, r4_h(ip)); });
    T(437, "R4_ITMBALL_MOTION1_PHYS", { r4_pre(ip);  itMball_Motion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(438, "R4_ITMBALL_MOTION6_PHYS", { r4_pre(ip);  itMball_Motion6_Phys(g); ret = H(ret, r4_h(ip)); });
    T(439, "R4_ITMETALB_UNKMOTION1_PHYS", { r4_pre(ip);  itMetalb_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(440, "R4_ITMETALB_UNKMOTION3_PHYS", { r4_pre(ip);  itMetalb_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(441, "R4_ITNESSPKFIRE_UNKMOTION0_ANIM", { r4_pre(ip);  ret = (u32) itNesspkfire_UnkMotion0_Anim(g); ret = H(ret, r4_h(ip)); });
    T(442, "R4_ITPIKACHUTHUNDER_UNKMOTION1_ANIM", { r4_pre(ip);  ret = (u32) itPikachuthunder_UnkMotion1_Anim(g); ret = H(ret, r4_h(ip)); });
    T(443, "R4_IT_802AA7E4", { r4_pre(ip);  ret = PIDX(it_802AA7E4(g)); ret = H(ret, r4_h(ip)); });
    T(444, "R4_ITZELDADINFIRE_GETOWNER", { r4_pre(ip);  ret = PIDX(itZeldaDinFire_GetOwner(g)); ret = H(ret, r4_h(ip)); });
    T(445, "R4_ITNOKONOKO_UNKMOTION4_PHYS", { r4_pre(ip);  itNokonoko_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(446, "R4_ITNOKONOKO_UNKMOTION10_PHYS", { r4_pre(ip);  itNokonoko_UnkMotion10_Phys(g); ret = H(ret, r4_h(ip)); });
    T(447, "R4_ITPIPPI_UNKMOTION1_PHYS", { r4_pre(ip);  itPippi_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(448, "R4_ITTOGEPY_UNKMOTION1_PHYS", { r4_pre(ip);  itTogepy_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(449, "R4_ITRABBITC_UNKMOTION1_PHYS", { r4_pre(ip);  itRabbitc_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(450, "R4_ITRABBITC_UNKMOTION3_PHYS", { r4_pre(ip);  itRabbitc_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(451, "R4_ITRSHELL_UNKMOTION3_ANIM", { r4_pre(ip);  ret = (u32) itRshell_UnkMotion3_Anim(g); ret = H(ret, r4_h(ip)); });
    T(452, "R4_ITRSHELL_UNKMOTION4_ANIM", { r4_pre(ip);  ret = (u32) itRshell_UnkMotion4_Anim(g); ret = H(ret, r4_h(ip)); });
    T(453, "R4_ITSEAKNEEDLETHROWN_UNKMOTION1_PHYS", { r4_pre(ip);  itSeakneedlethrown_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(454, "R4_ITSEAKNEEDLETHROWN_UNKMOTION4_PHYS", { r4_pre(ip);  itSeakneedlethrown_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(455, "R4_ITCAPSULE_UNKMOTION4_PHYS", { r4_pre(ip);  itCapsule_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(456, "R4_ITFFLOWERFLAME_UNKMOTION1_PHYS", { r4_pre(ip);  itFflowerflame_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(457, "R4_ITFREEZE_UNKMOTION3_PHYS", { r4_pre(ip);  itFreeze_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(458, "R4_ITSSCOPE_UNKMOTION1_PHYS", { r4_pre(ip);  itSscope_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(459, "R4_ITOCTAROCKSTONE_UNKMOTION0_ANIM", { r4_pre(ip);  ret = (u32) itOctarockstone_UnkMotion0_Anim(g); ret = H(ret, r4_h(ip)); });
    T(460, "R4_ITLIZARDON_UNKMOTION1_PHYS", { r4_pre(ip);  itLizardon_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(461, "R4_ITMARUMINE_UNKMOTION5_PHYS", { r4_pre(ip);  itMarumine_UnkMotion5_Phys(g); ret = H(ret, r4_h(ip)); });
    T(462, "R4_ITKABIGON_UNKMOTION0_PHYS", { r4_pre(ip);  itKabigon_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(463, "R4_ITNESSPKFIREPILLAR_UNKMOTION0_PHYS", { r4_pre(ip);  itNesspkfirepillar_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(464, "R4_ITBAT_LOGIC11_DMGDEALT", { r4_pre(ip);  ret = (u32) itBat_Logic11_DmgDealt(g); ret = H(ret, r4_h(ip)); });
    T(465, "R4_ITHARISEN_LOGIC24_DMGDEALT", { r4_pre(ip);  ret = (u32) itHarisen_Logic24_DmgDealt(g); ret = H(ret, r4_h(ip)); });
    T(466, "R4_ITLIPSTICK_LOGIC23_DMGDEALT", { r4_pre(ip);  ret = (u32) itLipstick_Logic23_DmgDealt(g); ret = H(ret, r4_h(ip)); });
    T(467, "R4_ITLINKBOMB_LOGIC16_HITSHIELD", { r4_pre(ip);  ret = (u32) itLinkBomb_Logic16_HitShield(g); ret = H(ret, r4_h(ip)); });
    T(468, "R4_ITNOKONOKO_UNKMOTION7_PHYS", { r4_pre(ip);  itNokonoko_UnkMotion7_Phys(g); ret = H(ret, r4_h(ip)); });
    T(469, "R4_ITLIKELIKE_UNKMOTION3_PHYS", { r4_pre(ip);  itLikelike_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(470, "R4_ITMEWTWODISABLE_LOGIC67_REFLECTED", { r4_pre(ip);  ret = (u32) itMewtwoDisable_Logic67_Reflected(g); ret = H(ret, r4_h(ip)); });
    T(471, "R4_ITDRMARIOPILL_REFLECTED", { r4_pre(ip);  ret = (u32) itDrMarioPill_Reflected(g); ret = H(ret, r4_h(ip)); });
    T(472, "R4_ITMBALL_REFLECTED", { r4_pre(ip);  ret = (u32) itMball_Reflected(g); ret = H(ret, r4_h(ip)); });
    T(473, "R4_ITGSHELL_LOGIC14_REFLECTED", { r4_pre(ip);  ret = (u32) itGShell_Logic14_Reflected(g); ret = H(ret, r4_h(ip)); });
    T(474, "R4_ITRSHELL_LOGIC15_REFLECTED", { r4_pre(ip);  ret = (u32) itRShell_Logic15_Reflected(g); ret = H(ret, r4_h(ip)); });
    T(475, "R4_ITPEACHTOADSPORE_LOGIC68_REFLECTED", { r4_pre(ip);  ret = (u32) itPeachToadSpore_Logic68_Reflected(g); ret = H(ret, r4_h(ip)); });
    T(476, "R4_IT_2725_LOGIC50_REFLECTED", { r4_pre(ip);  ret = (u32) it_2725_Logic50_Reflected(g); ret = H(ret, r4_h(ip)); });
    T(477, "R4_IT_2ADA_UNKMOTION0_ANIM", { r4_pre(ip);  ret = (u32) it_2ADA_UnkMotion0_Anim(g); ret = H(ret, r4_h(ip)); });
    T(478, "R4_IT_2F28_UNKMOTION0_ANIM", { r4_pre(ip);  ret = (u32) it_2F28_UnkMotion0_Anim(g); ret = H(ret, r4_h(ip)); });
    T(479, "R4_ITCHICORITALEAF_UNKMOTION0_ANIM", { r4_pre(ip);  ret = (u32) itChicoritaleaf_UnkMotion0_Anim(g); ret = H(ret, r4_h(ip)); });
    T(480, "R4_ITFUSHIGIBANA_UNKMOTION1_ANIM_INLINE2", { r4_pre(ip);  ret = (u32) itFushigibana_UnkMotion1_Anim_inline2(g); ret = H(ret, r4_h(ip)); });
    T(481, "R4_ITDRMARIOPILL_UNKMOTION0_ANIM", { r4_pre(ip);  ret = (u32) itDrMarioPill_UnkMotion0_Anim(g); ret = H(ret, r4_h(ip)); });
    T(482, "R4_ITPEACHTOADSPORE_UNKMOTION0_ANIM", { r4_pre(ip);  ret = (u32) itPeachtoadspore_UnkMotion0_Anim(g); ret = H(ret, r4_h(ip)); });
    T(483, "R4_ITSONANS_UNKMOTION1_ANIM", { r4_pre(ip);  ret = (u32) itSonans_UnkMotion1_Anim(g); ret = H(ret, r4_h(ip)); });
    T(484, "R4_IT_802D4FFC", { r4_pre(ip);  ret = (u32) it_802D4FFC(g); ret = H(ret, r4_h(ip)); });
    T(485, "R4_ITBOX_UNKMOTION7_ANIM", { r4_pre(ip);  ret = (u32) itBox_UnkMotion7_Anim(g); ret = H(ret, r4_h(ip)); });
    T(486, "R4_ITEGG_UNKMOTION6_ANIM", { r4_pre(ip);  ret = (u32) itEgg_UnkMotion6_Anim(g); ret = H(ret, r4_h(ip)); });
    T(487, "R4_ITFOXLASER_UNKMOTION1_PHYS", { r4_pre(ip);  itFoxlaser_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(488, "R4_ITLGUNRAY_UNKMOTION0_PHYS", { r4_pre(ip);  itLgunray_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(489, "R4_ITCEREBI_UNKMOTION2_PHYS", { r4_pre(ip);  itCerebi_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(490, "R4_ITFIRE_UNKMOTION2_PHYS", { r4_pre(ip);  itFire_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(491, "R4_ITMEW_UNKMOTION2_PHYS", { r4_pre(ip);  itMew_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(492, "R4_ITCLIMBERSICE_SUB_X4", { r4_pre(ip);  itClimbersIce_sub_x4(g); ret = H(ret, r4_h(ip)); });
    T(493, "R4_ITCOIN_UNKMOTION0_PHYS", { r4_pre(ip);  itCoin_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(494, "R4_ITCOIN_UNKMOTION3_PHYS", { r4_pre(ip);  itCoin_UnkMotion3_Phys(g); ret = H(ret, r4_h(ip)); });
    T(495, "R4_ITDOSEI_UNKMOTION1_PHYS", { r4_pre(ip);  itDosei_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(496, "R4_ITDRMARIOPILL_MOTION2_PHYS", { r4_pre(ip);  itDrMarioPill_Motion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(497, "R4_ITDRMARIOPILL_MOTION2_COLL", { r4_pre(ip);  ret = (u32) itDrMariopill_Motion2_Coll(g); ret = H(ret, r4_h(ip)); });
    T(498, "R4_ITDRMARIOPILL_MOTION6_COLL", { r4_pre(ip);  ret = (u32) itDrMarioPill_Motion6_Coll(g); ret = H(ret, r4_h(ip)); });
    T(499, "R4_ITFOXILLUSION_LOGIC14_DMGDEALT", { r4_pre(ip);  ret = (u32) itFoxIllusion_Logic14_DmgDealt(g); ret = H(ret, r4_h(ip)); });
    T(500, "R4_IT_802D4EF4", { r4_pre(ip);  ret = (u32) it_802D4EF4(g); ret = H(ret, r4_h(ip)); });
    T(501, "R4_ITLIPSTICK_UNKMOTION2_ANIM", { r4_pre(ip);  ret = (u32) itLipstick_UnkMotion2_Anim(g); ret = H(ret, r4_h(ip)); });
    T(502, "R4_ITSONANS_LOGIC9_DMGDEALT", { r4_pre(ip);  ret = (u32) itSonans_Logic9_DmgDealt(g); ret = H(ret, r4_h(ip)); });
    T(503, "R4_ITHITODEMAN_UNKMOTION1_PHYS", { r4_pre(ip);  itHitodeman_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(504, "R4_ITLEADEAD_UNKMOTION3_ANIM_INLINE", { r4_pre(ip);  itLeadead_UnkMotion3_Anim_inline(g); ret = H(ret, r4_h(ip)); });
    T(505, "R4_ITLEADEAD_UNKMOTION4_PHYS", { r4_pre(ip);  itLeadead_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(506, "R4_IT_802CC684", { r4_pre(ip);  it_802CC684(g); ret = H(ret, r4_h(ip)); });
    T(507, "R4_ITPEACHTOADSPORE_UNKMOTION0_PHYS", { r4_pre(ip);  itPeachtoadspore_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(508, "R4_ITPARASOL_UNKMOTION1_PHYS", { r4_pre(ip);  itParasol_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(509, "R4_ITPARASOL_UNKMOTION2_PHYS", { r4_pre(ip);  itParasol_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(510, "R4_ITSEAKNEEDLETHROWN_UNKMOTION0_ANIM", { r4_pre(ip);  ret = (u32) itSeakneedlethrown_UnkMotion0_Anim(g); ret = H(ret, r4_h(ip)); });
    T(511, "R4_ITPIKACHUTJOLTGROUND_UNKMOTION0_ANIM", { r4_pre(ip);  ret = (u32) itPikachutjoltground_UnkMotion0_Anim(g); ret = H(ret, r4_h(ip)); });
    T(512, "R4_ITARWINGLASER_UNKMOTION3_ANIM", { r4_pre(ip);  itArwinglaser_UnkMotion3_Anim(g); ret = H(ret, r4_h(ip)); });
    T(513, "R4_ITNESSPKFLASHEXPLODE_UNKMOTION0_PHYS", { r4_pre(ip);  itNessPKFlashExplode_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(514, "R4_ITLIPSTICKSPORE_UNKMOTION0_PHYS", { r4_pre(ip);  itLipstickspore_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(515, "R4_ITOCTAROCK_UNKMOTION1_PHYS", { r4_pre(ip);  itOctarock_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(516, "R4_ITOCTAROCK_UNKMOTION4_PHYS", { r4_pre(ip);  itOctarock_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(517, "R4_IT_802E5944", { r4_pre(ip);  it_802E5944(g); ret = H(ret, r4_h(ip)); });
    T(518, "R4_ITNOKONOKO_UNKMOTION8_PHYS", { r4_pre(ip);  itNokonoko_UnkMotion8_Phys(g); ret = H(ret, r4_h(ip)); });
    T(519, "R4_ITPATAPATA_UNKMOTION4_COLL", { r4_pre(ip);  ret = (u32) itPatapata_UnkMotion4_Coll(g); ret = H(ret, r4_h(ip)); });
    T(520, "R4_IT_802BDC38", { r4_pre(ip);  ret = (u32) it_802BDC38(g); ret = H(ret, r4_h(ip)); });
    T(521, "R4_IT_8029FDBC", { r4_pre(ip);  ret = (u32) it_8029FDBC(g); ret = H(ret, r4_h(ip)); });
    T(522, "R4_IT_802AA7F0", { r4_pre(ip);  ret = (u32) it_802AA7F0(g); ret = H(ret, r4_h(ip)); });
    T(523, "R4_IT_8028D390", { r4_pre(ip);  it_8028D390(g); ret = H(ret, r4_h(ip)); });
    T(524, "R4_ITGAMEWATCHCHEF_UNKMOTION0_PHYS", { r4_pre(ip); ip->xDD4_itemVar.gamewatchchef.x4 = (s32) (R() % 4); itGamewatchchef_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(525, "R4_ITHAMMERHEAD_LOGIC40_SPAWNED", { r4_pre(ip);  itHammerHead_Logic40_Spawned(g); ret = H(ret, r4_h(ip)); });
    T(526, "R4_ITHASSAM_UNKMOTION0_PHYS", { r4_pre(ip);  itHassam_UnkMotion0_Phys(g); ret = H(ret, r4_h(ip)); });
    T(527, "R4_ITHASSAM_UNKMOTION2_PHYS", { r4_pre(ip);  itHassam_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(528, "R4_ITMARIL_UNKMOTION1_PHYS", { r4_pre(ip);  itMaril_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(529, "R4_ITLINKARROW_UNKMOTION1_PHYS", { r4_pre(ip);  itLinkarrow_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(530, "R4_IT_80283BD4", { r4_pre(ip);  it_80283BD4(g); ret = H(ret, r4_h(ip)); });
    T(531, "R4_IT_802D98AC", { r4_pre(ip);  it_802D98AC(g); ret = H(ret, r4_h(ip)); });
    T(532, "R4_IT_802F2020", { r4_pre(ip);  ret = (u32) it_802F2020(g); ret = H(ret, r4_h(ip)); });
    T(533, "R4_ITFREEZE_UNKMOTION4_PHYS", { r4_pre(ip);  itFreeze_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(534, "R4_ITFREEZER_UNKMOTION2_PHYS", { r4_pre(ip);  itFreezer_UnkMotion2_Phys(g); ret = H(ret, r4_h(ip)); });
    T(535, "R4_ITHOUOU_UNKMOTION4_PHYS", { r4_pre(ip);  itHouou_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(536, "R4_ITKIREIHANA_UNKMOTION4_PHYS", { r4_pre(ip);  itKireihana_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(537, "R4_ITKAMEX_UNKMOTION1_PHYS", { r4_pre(ip);  itKamex_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(538, "R4_ITLINKBOMB_UNKMOTION4_PHYS", { r4_pre(ip);  itLinkbomb_UnkMotion4_Phys(g); ret = H(ret, r4_h(ip)); });
    T(539, "R4_ITSAMUSMISSILE_UNKMOTION1_PHYS", { r4_pre(ip);  itSamusmissile_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
    T(540, "R4_IT_802CD7D4", { r4_pre(ip);  ret = (u32) it_802CD7D4(g); ret = H(ret, r4_h(ip)); });
    T(541, "R4_IT_8028D4E4", { r4_pre(ip);  it_8028D4E4(g); ret = H(ret, r4_h(ip)); });
    T(542, "R4_ITGSHELL_UNKMOTION3_ANIM", { r4_pre(ip);  ret = (u32) itGshell_UnkMotion3_Anim(g); ret = H(ret, r4_h(ip)); });
    T(543, "R4_IT_8028B988", { r4_pre(ip);  it_8028B988(g); ret = H(ret, r4_h(ip)); });
    T(544, "R4_IT_802D24A0", { r4_pre(ip); ip->kind = (R() % 4 == 0) ? (s32) (R() % 240) : 193 + (s32) (R() % 11); it_802D24A0(g); ret = H(ret, r4_h(ip)); });
    T(545, "R4_IT_802CB844", { r4_pre(ip); ip->kind = (R() % 4 == 0) ? (s32) (R() % 240) : 193 + (s32) (R() % 11); it_802CB844(g); ret = H(ret, r4_h(ip)); });
    T(546, "R4_ITFLIPPER_ADDSPINIMPULSE", { r4_pre(ip);  { Vec3 p; f32 v; p.x = RF(); p.y = RF(); p.z = RF(); v = RF(); itFlipper_AddSpinImpulse(g, &p, v); } ret = H(ret, r4_h(ip)); });
    T(547, "R4_ITCHICORITA_PHYS", { r4_pre(ip);  itChicorita_Phys(g); ret = H(ret, r4_h(ip)); });
    T(548, "R4_IT_802E657C", { r4_pre(ip);  ret = (u32) it_802E657C(g); ret = H(ret, r4_h(ip)); });
    /* targeted: itSamusmissile_UnkMotion1_Phys's fused site with moderate inputs and no clamp
     * (x34 = 1e30); the random table above clamps or swamps every case where fusing matters */
    T(600, "R4_SAMUSMISSILE_FMA", { r4_pre(ip); ip->xDB0_itcmd_var1 = 1; ip->facing_dir = RM();
                                    ip->x40_vel.x = RM(); r4_sa[13] = bits(1e30f);
                                    itSamusmissile_UnkMotion1_Phys(g); ret = H(ret, r4_h(ip)); });
}
#undef it_804D6D28
#undef it_80275158
