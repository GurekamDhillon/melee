/* Verbatim copies of the decomp C bodies ported in src/itkinds.rs (round 4), extracted unchanged
 * from the files named below; #included by round4.c. Also the callee it_802762BC (itmaplib.c,
 * ported earlier in src/item.rs but not yet in this oracle) and the file-local itGShell_Attrs. */
/* ---- src/melee/it/itmaplib.c */
void it_802762BC(Item* item)
{
    item->ground_or_air = GA_Air;
    item->xDC8_word.flags.x1F = 0;
}

/* ---- src/melee/it/it_2725.c. round1.c's copy of it_80275158 takes the Item itself (its
 * GET_ITEM is a cast); these callers pass the GObj, so they get this verbatim GObj copy. */
#define it_80275158 it_80275158_g
void it_80275158(Item_GObj* item_gobj, f32 lifetime)
{
    Item* item;

    item = GET_ITEM(item_gobj);
    item->xD44_lifeTimer = lifetime;
    item->xD48_halfLifeTimer = lifetime * it_804D6D28->x4C_float;
}

/* ---- src/melee/it/kinds/itgshell.c (file-local type) */
typedef struct itGShell_Attrs {
    float x0;
    float x4;
    float x8;
    float xC;
    float x10;
    float x14;
    char pad18[0x1C - 0x18];
    float x1C;
    float x20;
    float x24;
    float x28;
    float x2C;
    float x30;
    Vec x34;
} itGShell_Attrs;

/* ---- src/melee/it/kinds/itbombhei.c */
void itBombhei_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itclimbersice.c */
void itClimbersice_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itdosei.c */
void itDosei_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itdosei.c */
void itDosei_UnkMotion8_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itdosei.c */
void itDosei_UnkMotion11_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itevyoshiegg.c */
void itEvyoshiegg_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itfflower.c */
void itFflower_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attr = ip->xCC_item_attr;

    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itfflower.c */
void itFflower_UnkMotion6_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attr = ip->xCC_item_attr;

    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itfflowerflame.c */
void itFflowerflame_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    ItemAttr* attrs = item->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itflipper.c */
void itFlipper_Falling_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itfreeze.c */
void itFreeze_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itfushigibana.c */
void itFushigibana_UnkMotion2_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itgshell.c */
void itGshell_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attr = ip->xCC_item_attr;
    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itgshell.c */
void itGshell_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attr = ip->xCC_item_attr;
    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itgshell.c */
void itGshell_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attr = ip->xCC_item_attr;
    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itgshell.c */
void itGshell_UnkMotion8_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attr = ip->xCC_item_attr;
    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itharisen.c */
void itHarisen_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itheart.c */
void itHeart_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    ItemAttr* attr = item->xCC_item_attr;

    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/ithinoarashi.c */
void itHinoarashi_UnkMotion2_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itkusudama.c */
void itKusudama_UnkMotion2_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itleadead.c */
void itLeadead_UnkMotion10_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itleadead.c */
void itLeadead_UnkMotion14_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itlikelike.c */
void itLikelike_UnkMotion19_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itlipstick.c */
void itLipstick_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itlucky.c */
void itLucky_UnkMotion5_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itlucky.c */
void itLucky_UnkMotion6_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itlucky.c */
void itLucky_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itmaril.c */
void itMaril_UnkMotion2_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itmarumine.c */
void itMarumine_UnkMotion2_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attr = ip->xCC_item_attr;

    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itmarumine.c */
void itMarumine_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attr = ip->xCC_item_attr;

    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itmsbomb.c */
void itMsbomb_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attr = ip->xCC_item_attr;
    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itoctarock.c */
void itOctarock_UnkMotion2_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itoctarock.c */
void itOctarock_UnkMotion6_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itoldkuri.c */
void itOldkuri_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itoldkuri.c */
void itOldkuri_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itoldkuri.c */
void itOldkuri_UnkMotion6_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itoldkuri.c */
void itOldkuri_UnkMotion8_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itoldottosea.c */
void itOldottosea_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itpatapata.c */
void itPatapata_UnkMotion6_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itrshell.c */
void itRshell_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itrshell.c */
void itRshell_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itrshell.c */
void itRshell_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itrshell.c */
void itRshell_UnkMotion6_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itsamusbomb.c */
void itSamusbomb_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itscball.c */
void itScball_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    ItemAttr* attr = item->xCC_item_attr;

    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itspycloak.c */
void itSpycloak_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    ItemAttr* attr = item->xCC_item_attr;

    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itspycloak.c */
void itSpycloak_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    ItemAttr* attr = item->xCC_item_attr;

    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itstar.c */
void itStar_UnkMotion0_Phys(Item_GObj* item_gobj)
{
    Item* item = GET_ITEM(item_gobj);
    ItemAttr* attr = item->xCC_item_attr;
    it_80272860(item_gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/ittaru.c */
void itTaru_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/ittarucann.c */
void itTarucann_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/ittincle.c */
void itTincle_UnkMotion8_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/ittomato.c */
void itTomato_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attr = ip->xCC_item_attr;
    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/ittosakinto.c */
void itTosakinto_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itwhispyapple.c */
void itWhispyapple_UnkMotion5_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itwhitebea.c */
void itOldottosea_UnkMotion9_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itwhitebea.c */
void itWhitebea_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itwhitebea.c */
void itWhitebea_UnkMotion6_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itwhitebea.c */
void itWhitebea_UnkMotion9_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itwstar.c */
void itWstar_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/ityoshieggthrow.c */
void itYoshieggthrow_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itzgshell.c */
void itZrshell_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itzgshell.c */
void itZrshell_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itzgshell.c */
void itZrshell_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itzgshell.c */
void itZrshell_UnkMotion8_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itbat.c */
bool itBat_Logic11_Clanked(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itbat.c */
bool itBat_Logic11_HitShield(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itdosei.c */
bool itDosei_Logic7_Clanked(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itdosei.c */
bool itDosei_Logic7_HitShield(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itfflower.c */
bool itFFlower_Logic25_DmgDealt(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itfflower.c */
bool itFFlower_Logic25_Clanked(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itfflower.c */
bool itFFlower_Logic25_HitShield(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itlgun.c */
bool itLGun_Logic16_DmgDealt(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itlgun.c */
bool itLGun_Logic16_Clanked(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itlgun.c */
bool itLGun_Logic16_HitShield(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itparasol.c */
bool itParasol_Logic13_DmgDealt(Item_GObj* item_gobj)
{
    itColl_BounceOffVictim(item_gobj);
    return false;
}

/* ---- src/melee/it/kinds/itparasol.c */
bool itParasol_Logic13_Clanked(Item_GObj* item_gobj)
{
    itColl_BounceOffVictim(item_gobj);
    return false;
}

/* ---- src/melee/it/kinds/itparasol.c */
bool itParasol_Logic13_HitShield(Item_GObj* item_gobj)
{
    itColl_BounceOffVictim(item_gobj);
    return false;
}

/* ---- src/melee/it/kinds/itpeachturnip.c */
bool itPeachTurnip_Logic56_Clanked(Item_GObj* item_gobj)
{
    itColl_BounceOffVictim(item_gobj);
    return false;
}

/* ---- src/melee/it/kinds/itpeachturnip.c */
bool itPeachTurnip_Logic56_HitShield(Item_GObj* item_gobj)
{
    itColl_BounceOffVictim(item_gobj);
    return false;
}

/* ---- src/melee/it/kinds/itrshell.c */
bool itRShell_Logic15_Clanked(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itscball.c */
bool itScball_Logic30_HitShield(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itsscope.c */
bool itSScope_Logic21_DmgDealt(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itsscope.c */
bool itSScope_Logic21_Clanked(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itsscope.c */
bool itSScope_Logic21_HitShield(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itsword.c */
bool itSword_Logic12_Clanked(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itsword.c */
bool itSword_Logic12_HitShield(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itkyasarin.c */
void itKyasarin_UnkMotion9_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
}

/* ---- src/melee/it/kinds/itleadead.c */
void itLeadead_UnkMotion16_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
}

/* ---- src/melee/it/kinds/itleadead.c */
void itLeadead_UnkMotion17_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
}

/* ---- src/melee/it/kinds/itlikelike.c */
void itLikelike_UnkMotion21_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
}

/* ---- src/melee/it/kinds/itoctarock.c */
void itOctarock_UnkMotion8_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
}

/* ---- src/melee/it/kinds/itoctarockstone.c */
void itOctarockstone_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
}

/* ---- src/melee/it/kinds/itoldkuri.c */
void itOldkuri_UnkMotion11_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
}

/* ---- src/melee/it/kinds/itwhitebea.c */
void itOldottosea_UnkMotion11_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
}

/* ---- src/melee/it/kinds/itwhitebea.c */
void itWhitebea_UnkMotion11_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
}

/* ---- src/melee/it/kinds/itzgshell.c */
void itZrshell_UnkMotion10_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
}

/* ---- src/melee/it/kinds/itentei.c */
void itEntei_UnkMotion0_Phys(Item_GObj* gobj)
{
    Item* it = GET_ITEM(gobj);
    if (it->ground_or_air == GA_Air) {
        it_80272860(gobj, it->xCC_item_attr->x10_fall_speed,
                    it->xCC_item_attr->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/itkamex.c */
void it_802CA618(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->ground_or_air == GA_Air) {
        it_80272860(gobj, ip->xCC_item_attr->x10_fall_speed,
                    ip->xCC_item_attr->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/itkamex.c */
void itKamex_UnkMotion2_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->ground_or_air == GA_Air) {
        it_80272860(gobj, ip->xCC_item_attr->x10_fall_speed,
                    ip->xCC_item_attr->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/itpippi.c */
void itPippi_UnkMotion5_Phys(Item_GObj* gobj)
{
    Item* it = GET_ITEM(gobj);
    if (it->ground_or_air == GA_Air) {
        it_80272860(gobj, it->xCC_item_attr->x10_fall_speed,
                    it->xCC_item_attr->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/itraikou.c */
void itRaikou_UnkMotion0_Phys(Item_GObj* gobj)
{
    Item* it = GET_ITEM(gobj);
    if (it->ground_or_air == GA_Air) {
        it_80272860(gobj, it->xCC_item_attr->x10_fall_speed,
                    it->xCC_item_attr->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/itsuikun.c */
void itSuikun_UnkMotion0_Phys(Item_GObj* gobj)
{
    Item* it = GET_ITEM(gobj);
    if (it->ground_or_air == GA_Air) {
        it_80272860(gobj, it->xCC_item_attr->x10_fall_speed,
                    it->xCC_item_attr->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/itleadead.c */
void itLeadead_UnkMotion12_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->ground_or_air == GA_Air) {
        ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
    }
}

/* ---- src/melee/it/kinds/itnokonoko.c */
void itNokonoko_UnkMotion5_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->ground_or_air == GA_Air) {
        ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
    }
}

/* ---- src/melee/it/kinds/itoctarock.c */
void itOctarock_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->ground_or_air == GA_Air) {
        ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
    }
}

/* ---- src/melee/it/kinds/itoldkuri.c */
void itOldkuri_UnkMotion9_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->ground_or_air == GA_Air) {
        ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
    }
}

/* ---- src/melee/it/kinds/itoldottosea.c */
void itOldottosea_UnkMotion5_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->ground_or_air == GA_Air) {
        ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
    }
}

/* ---- src/melee/it/kinds/itwhitebea.c */
void itWhitebea_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->ground_or_air == GA_Air) {
        ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
    }
}

/* ---- src/melee/it/kinds/it_2ADA.c */
void it_2ADA_UnkMotion0_Phys(Item_GObj* gobj)
{
    ItemAttr* attrs = GET_ITEM(gobj)->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/it_2E5A.c */
void it_2E5A_UnkMotion1_Phys(HSD_GObj* item_gobj)
{
    ItemAttr* comm_attr = GET_ITEM(item_gobj)->xCC_item_attr;
    it_80272860(item_gobj, comm_attr->x10_fall_speed,
                comm_attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itcoin.c */
void itCoin_UnkMotion4_Phys(Item_GObj* gobj)
{
    ItemAttr* attr = GET_ITEM(gobj)->xCC_item_attr;

    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itegg.c */
void itEgg_UnkMotion1_Phys(Item_GObj* gobj)
{
    ItemAttr* attrs = GET_ITEM(gobj)->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itlgun.c */
void itLgun_UnkMotion1_Phys(Item_GObj* gobj)
{
    ItemAttr* item_comm_attr = GET_ITEM(gobj)->xCC_item_attr;
    it_80272860(gobj, item_comm_attr->x10_fall_speed,
                item_comm_attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itfoods.c */
void itFoods_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    ItemAttr* temp_r4 = item->xCC_item_attr;
    it_80272860(gobj, temp_r4->x10_fall_speed, temp_r4->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/ithammer.c */
void itHammer_UnkMotion1_Phys(HSD_GObj* gobj)
{
    Item* it = GET_ITEM(gobj);
    ItemAttr* temp_r4 = it->xCC_item_attr;
    it_80272860(gobj, temp_r4->x10_fall_speed, temp_r4->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/ithammer.c */
void itHammer_UnkMotion4_Phys(HSD_GObj* gobj)
{
    Item* it = GET_ITEM(gobj);
    ItemAttr* temp_r4 = it->xCC_item_attr;
    it_80272860(gobj, temp_r4->x10_fall_speed, temp_r4->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/ithammerhead.c */
bool itHammerHead_Logic40_Reflected(Item_GObj* gobj)
{
    it_80273030(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itoctarockstone.c */
bool itOctarockstone_Logic4_Reflected(Item_GObj* gobj)
{
    it_80273030(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itpeachturnip.c */
bool itPeachTurnip_Logic56_Reflected(Item_GObj* item_gobj)
{
    it_80273030(item_gobj);
    return false;
}

/* ---- src/melee/it/kinds/itstarrod.c */
bool itStarRod_Logic22_DmgDealt(Item_GObj* gobj)
{
    /// @todo Use #itGetMotionId if it can be made to inline here.
    Item* ip = GET_ITEM(gobj);
    if (ip->msid == 3 || ip->msid == 4) {
        itColl_BounceOffVictim(gobj);
    }
    return false;
}

/* ---- src/melee/it/kinds/itstarrod.c */
bool itStarRod_Logic22_Clanked(Item_GObj* gobj)
{
    /// @todo Use #itGetMotionId if it can be made to inline here.
    Item* ip = GET_ITEM(gobj);
    if (ip->msid == 3 || ip->msid == 4) {
        itColl_BounceOffVictim(gobj);
    }
    return false;
}

/* ---- src/melee/it/kinds/itstarrod.c */
bool itStarRod_Logic22_HitShield(Item_GObj* gobj)
{
    /// @todo Use #itGetMotionId if it can be made to inline here.
    Item* ip = GET_ITEM(gobj);
    if (ip->msid == 3 || ip->msid == 4) {
        itColl_BounceOffVictim(gobj);
    }
    return false;
}

/* ---- src/melee/it/kinds/itdkinoko.c */
void itDkinoko_UnkMotion0_Phys(Item_GObj* gobj)
{
    Item* it = GET_ITEM(gobj);
    KinokoAttrs* attrs = it->xC4_article_data->x4_specialAttributes;
    if (it->ground_or_air != GA_Air) {
        it->xDD4_itemVar.kinoko.x0 = attrs->x0 * it->facing_dir;
        it->x40_vel.x =
            it->xDD4_itemVar.kinoko.x0 * it->xDD4_itemVar.kinoko.x4.y;
    }
}

/* ---- src/melee/it/kinds/itkinoko.c */
void itKinoko_UnkMotion0_Phys(Item_GObj* gobj)
{
    Item* it = GET_ITEM(gobj);
    KinokoAttrs* attrs = it->xC4_article_data->x4_specialAttributes;
    if (it->ground_or_air != GA_Air) {
        it->xDD4_itemVar.kinoko.x0 = attrs->x0 * it->facing_dir;
        it->x40_vel.x =
            it->xDD4_itemVar.kinoko.x0 * it->xDD4_itemVar.kinoko.x4.y;
    }
}

/* ---- src/melee/it/kinds/itdkinoko.c */
void itDkinoko_UnkMotion1_Phys(Item_GObj* gobj)
{
    f32 vel;
    Item* it = GET_ITEM(gobj);
    ItemAttr* attr = it->xCC_item_attr;
    KinokoAttrs* sa = it->xC4_article_data->x4_specialAttributes;

    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);

    if (it->x40_vel.x < 0.0f) {
        vel = -it->x40_vel.x;
    } else {
        vel = it->x40_vel.x;
    }

    if (vel < sa->x4) {
        it->x40_vel.x = 0.0f;
        return;
    } else if (it->x40_vel.x > 0.0f) {
        it->x40_vel.x -= sa->x4;
    }
    if (it->x40_vel.x < 0.0f) {
        it->x40_vel.x += sa->x4;
    }
}

/* ---- src/melee/it/kinds/itkinoko.c */
void itKinoko_UnkMotion1_Phys(Item_GObj* gobj)
{
    f32 vel;
    Item* it = GET_ITEM(gobj);
    ItemAttr* attr = it->xCC_item_attr;
    KinokoAttrs* sa = it->xC4_article_data->x4_specialAttributes;

    it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);

    if (it->x40_vel.x < 0.0f) {
        vel = -it->x40_vel.x;
    } else {
        vel = it->x40_vel.x;
    }

    if (vel < sa->x4) {
        it->x40_vel.x = 0.0f;
        return;
    } else if (it->x40_vel.x > 0.0f) {
        it->x40_vel.x -= sa->x4;
    }
    if (it->x40_vel.x < 0.0f) {
        it->x40_vel.x += sa->x4;
    }
}

/* ---- src/melee/it/kinds/itdosei.c */
bool itDosei_UnkMotion6_Anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDD4_itemVar.dosei.xDE4 = ip->pos;
    return false;
}

/* ---- src/melee/it/kinds/itdosei.c */
bool itDosei_UnkMotion8_Anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDD4_itemVar.dosei.xDE4 = ip->pos;
    return false;
}

/* ---- src/melee/it/kinds/itkamex.c */
bool it_802CAE60(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (--ip->xD44_lifeTimer <= 0.0f) {
        return true;
    }
    return false;
}

/* ---- src/melee/it/kinds/itunknown.c */
bool it_802CF120(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (--ip->xD44_lifeTimer <= 0.0f) {
        return true;
    }
    return false;
}

/* ---- src/melee/it/kinds/itlinkbomb.c */
void itLinkbomb_UnkMotion1_Phys(HSD_GObj* gobj)
{
    ItemAttr* temp_r4;

    temp_r4 = GET_ITEM(gobj)->xCC_item_attr;
    it_80272860(gobj, temp_r4->x10_fall_speed, temp_r4->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itpeachturnip.c */
void itPeachturnip_UnkMotion1_Phys(Item_GObj* item_gobj)
{
    ItemAttr* temp_r4;

    temp_r4 = GET_ITEM(item_gobj)->xCC_item_attr;
    it_80272860(item_gobj, temp_r4->x10_fall_speed,
                temp_r4->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itlucky.c */
bool it_802D5884(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xD44_lifeTimer -= 1.0f;
    if (ip->xD44_lifeTimer <= 0.0f) {
        return true;
    }
    return false;
}

/* ---- src/melee/it/kinds/itmaril.c */
static bool itMaril_UnkMotion2_Anim_inline(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xD44_lifeTimer -= 1.0f;
    if (ip->xD44_lifeTimer <= 0.0f) {
        return true;
    }
    return false;
}

/* ---- src/melee/it/kinds/itluigifireball.c */
bool itLuigifireball_UnkMotion0_Anim(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    --item->xD44_lifeTimer;
    return item->xD44_lifeTimer <= 0 ? true : false;
}

/* ---- src/melee/it/kinds/itmariofireball.c */
bool itMariofireball_UnkMotion0_Anim(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    --item->xD44_lifeTimer;
    return item->xD44_lifeTimer <= 0 ? true : false;
}

/* ---- src/melee/it/kinds/itmball.c */
void itMball_Motion1_Phys(Item_GObj* arg0)
{
    Item* item = GET_ITEM(arg0);
    it_80272860(arg0, item->xCC_item_attr->x10_fall_speed,
                item->xCC_item_attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itmball.c */
void itMball_Motion6_Phys(Item_GObj* arg0)
{
    Item* item = GET_ITEM(arg0);

    it_80272860(arg0, item->xCC_item_attr->x10_fall_speed,
                item->xCC_item_attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itmetalb.c */
void itMetalb_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* it = gobj->user_data;
    it_80272860(gobj, it->xCC_item_attr->x10_fall_speed,
                it->xCC_item_attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itmetalb.c */
void itMetalb_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* it = gobj->user_data;
    it_80272860(gobj, it->xCC_item_attr->x10_fall_speed,
                it->xCC_item_attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itnesspkfire.c */
bool itNesspkfire_UnkMotion0_Anim(Item_GObj* gobj)
{
    Item* ip = gobj->user_data;
    ip->xD44_lifeTimer--;
    if (ip->xD44_lifeTimer <= 0.0f) {
        return true;
    }
    return false;
}

/* ---- src/melee/it/kinds/itpikachuthunder.c */
bool itPikachuthunder_UnkMotion1_Anim(Item_GObj* gobj)
{
    Item* ip = gobj->user_data;
    ip->xD44_lifeTimer--;
    if (ip->xD44_lifeTimer <= 0.0f) {
        return true;
    }
    return false;
}

/* ---- src/melee/it/kinds/itnesspkflash.c */
Fighter_GObj* it_802AA7E4(Item_GObj* gobj)
{
    /// @todo Use #itGetOwner if it can be made to inline here.
    return GET_ITEM(gobj)->owner;
}

/* ---- src/melee/it/kinds/itzeldadinfire.c */
Fighter_GObj* itZeldaDinFire_GetOwner(Item_GObj* gobj)
{
    /// @todo Use #itGetOwner if it can be made to inline here.
    return GET_ITEM(gobj)->owner;
}

/* ---- src/melee/it/kinds/itnokonoko.c */
void itNokonoko_UnkMotion4_Phys(Item_GObj* gobj)
{
    ItemAttr* temp_r4 = GET_ITEM(gobj)->xCC_item_attr;
    it_80272860(gobj, temp_r4->x10_fall_speed, temp_r4->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itnokonoko.c */
void itNokonoko_UnkMotion10_Phys(Item_GObj* gobj)
{
    ItemAttr* temp_r4 = GET_ITEM(gobj)->xCC_item_attr;
    it_80272860(gobj, temp_r4->x10_fall_speed, temp_r4->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itpippi.c */
void itPippi_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* it = gobj->user_data;
    if (it->ground_or_air == GA_Air) {
        it_80272860(gobj, it->xCC_item_attr->x10_fall_speed,
                    it->xCC_item_attr->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/ittogepy.c */
void itTogepy_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* it = gobj->user_data;
    if (it->ground_or_air == GA_Air) {
        it_80272860(gobj, it->xCC_item_attr->x10_fall_speed,
                    it->xCC_item_attr->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/itrabbitc.c */
void itRabbitc_UnkMotion1_Phys(Item_GObj* item_gobj)
{
    ItemAttr* attr;

    attr = ((Item*) item_gobj->user_data)->xCC_item_attr;
    it_80272860(item_gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itrabbitc.c */
void itRabbitc_UnkMotion3_Phys(Item_GObj* item_gobj)
{
    ItemAttr* attr;

    attr = ((Item*) item_gobj->user_data)->xCC_item_attr;
    it_80272860(item_gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itrshell.c */
bool itRshell_UnkMotion3_Anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->xDD4_itemVar.rshell.xDD8 <= 0.0f) {
        if (!ip->xDCD_flag.b5) {
            it_80275444(gobj);
        }
    } else {
        ip->xDD4_itemVar.rshell.xDD8 -= 1.0f;
    }
    return false;
}

/* ---- src/melee/it/kinds/itrshell.c */
bool itRshell_UnkMotion4_Anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->xDD4_itemVar.rshell.xDD8 <= 0.0f) {
        if (!ip->xDCD_flag.b5) {
            it_80275444(gobj);
        }
    } else {
        ip->xDD4_itemVar.rshell.xDD8 -= 1.0f;
    }
    return false;
}

/* ---- src/melee/it/kinds/itseakneedlethrown.c */
void itSeakneedlethrown_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.x = ip->xDD4_itemVar.seakneedlethrown.xDD8;
    ip->x40_vel.y += ip->xDD4_itemVar.seakneedlethrown.xDE0;
    if (ip->x40_vel.y < ip->xDD4_itemVar.seakneedlethrown.xDDC) {
        ip->x40_vel.y = ip->xDD4_itemVar.seakneedlethrown.xDDC;
    }
}

/* ---- src/melee/it/kinds/itseakneedlethrown.c */
void itSeakneedlethrown_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.x = ip->xDD4_itemVar.seakneedlethrown.xDD8;
    ip->x40_vel.y += ip->xDD4_itemVar.seakneedlethrown.xDE0;
    if (ip->x40_vel.y < ip->xDD4_itemVar.seakneedlethrown.xDDC) {
        ip->x40_vel.y = ip->xDD4_itemVar.seakneedlethrown.xDDC;
    }
}

/* ---- src/melee/it/kinds/itcapsule.c */
void itCapsule_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* it = GET_ITEM(gobj);
    it_80272860(gobj, it->xCC_item_attr->x10_fall_speed,
                it->xCC_item_attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itfflowerflame.c */
void itFflowerflame_UnkMotion1_Phys(Item_GObj* gobj)
{
    ItemAttr* temp_r4;

    Item* item = GET_ITEM(gobj);
    temp_r4 = item->xCC_item_attr;
    it_80272860(gobj, temp_r4->x10_fall_speed, temp_r4->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itfreeze.c */
void itFreeze_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* ip;
    ItemAttr* it_attr;

    ip = gobj->user_data;
    it_attr = ip->xCC_item_attr;
    it_80272860(gobj, it_attr->x10_fall_speed, it_attr->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itsscope.c */
void itSscope_UnkMotion1_Phys(Item_GObj* gobj)
{
    ItemAttr* attrs = (GET_ITEM(gobj))->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
}

/* ---- src/melee/it/kinds/itoctarockstone.c */
bool itOctarockstone_UnkMotion0_Anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ItemAttr* attrs = ip->xCC_item_attr;
    it_80272860(gobj, attrs->x10_fall_speed, attrs->x14_fall_speed_max);
    return false;
}

/* ---- src/melee/it/kinds/itlizardon.c */
void itLizardon_UnkMotion1_Phys(Item_GObj* gobj)
{
    ItemAttr* ip_attr;
    Item* ip = GET_ITEM(gobj);

    if (ip->ground_or_air == GA_Air) {
        ip_attr = ip->xCC_item_attr;
        it_80272860(gobj, ip_attr->x10_fall_speed,
                    ip_attr->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/itmarumine.c */
void itMarumine_UnkMotion5_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);

    if (ip->ground_or_air == GA_Air) {
        ItemAttr* attr = ip->xCC_item_attr;
        it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/itkabigon.c */
void itKabigon_UnkMotion0_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);

    if (ip->xDB0_itcmd_var1 == 0) {
        if (ip->ground_or_air == 1) {
            ItemAttr* attrs = ip->xCC_item_attr;
            it_80272860(gobj, attrs->x10_fall_speed,
                        attrs->x14_fall_speed_max);
        }
    }
}

/* ---- src/melee/it/kinds/itnesspkfirepillar.c */
void itNesspkfirepillar_UnkMotion0_Phys(Item_GObj* item_gobj)
{
    Item* item = GET_ITEM(item_gobj);
    if ((item->ground_or_air = GA_Air)) {
        ItemAttr* attr = item->xCC_item_attr;
        it_80272860(item_gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/itbat.c */
bool itBat_Logic11_DmgDealt(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);

    if (item->msid == 3) {
        itColl_BounceOffVictim(gobj);
    }

    return false;
}

/* ---- src/melee/it/kinds/itharisen.c */
bool itHarisen_Logic24_DmgDealt(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->msid == 7 || ip->msid == 8) {
        itColl_BounceOffVictim(gobj);
    }
    return false;
}

/* ---- src/melee/it/kinds/itlipstick.c */
bool itLipstick_Logic23_DmgDealt(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->msid == 3 || ip->msid == 4) {
        itColl_BounceOffVictim(gobj);
    }
    return false;
}

/* ---- src/melee/it/kinds/itlinkbomb.c */
bool itLinkBomb_Logic16_HitShield(Item_GObj* gobj)
{
    itColl_BounceOffVictim(gobj);
    return 0;
}

/* ---- src/melee/it/kinds/itnokonoko.c */
void itNokonoko_UnkMotion7_Phys(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->ground_or_air == 1) {
        ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
    }
}

/* ---- src/melee/it/kinds/itlikelike.c */
void itLikelike_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.y += ip->xCC_item_attr->x10_fall_speed;
}

/* ---- src/melee/it/kinds/itmewtwodisable.c */
bool itMewtwoDisable_Logic67_Reflected(Item_GObj* item_gobj)
{
    // process being reflected
    return it_80273030(item_gobj);
}

/* ---- src/melee/it/kinds/itdrmariopill.c */
bool itDrMarioPill_Reflected(Item_GObj* gobj)
{
    it_8026B3A8(gobj);
    return it_80273030(gobj);
}

/* ---- src/melee/it/kinds/itmball.c */
bool itMball_Reflected(Item_GObj* arg0)
{
    it_8026B3A8(arg0);
    return it_80273030(arg0);
}

/* ---- src/melee/it/kinds/itgshell.c */
bool itGShell_Logic14_Reflected(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    it_80273030(gobj);
    ip->xDD4_itemVar.gshell.xDD4 = ip->xD48_halfLifeTimer;
    return false;
}

/* ---- src/melee/it/kinds/itrshell.c */
bool itRShell_Logic15_Reflected(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    it_80273030(gobj);
    ip->xDD4_itemVar.rshell.xDD4 = ip->xD48_halfLifeTimer;
    return false;
}

/* ---- src/melee/it/kinds/itpeachtoadspore.c */
bool itPeachToadSpore_Logic68_Reflected(Item_GObj* item_gobj)
{
    Item* item;
    f32 lifeTimer;

    item = GET_ITEM(item_gobj);
    lifeTimer = item->xD44_lifeTimer;
    it_80273030(item_gobj);
    item->xD44_lifeTimer = lifeTimer;
    return false;
}

/* ---- src/melee/it/kinds/itsamusbomb.c */
bool it_2725_Logic50_Reflected(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    f32 yvel;

    it_80273030(gobj);
    yvel = ip->x40_vel.y;
    if (yvel < 0.0f) {
        ip->x40_vel.y = -yvel * ip->xC70;
    }
    ip->xDD4_itemVar.samusbomb.owner = NULL;
    return false;
}

/* ---- src/melee/it/kinds/it_2ADA.c */
bool it_2ADA_UnkMotion0_Anim(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    item->xD44_lifeTimer -= 1;
    if (item->xD44_lifeTimer <= 0) {
        return true;
    }
    return false;
}

/* ---- src/melee/it/kinds/it_2F28.c */
bool it_2F28_UnkMotion0_Anim(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xD44_lifeTimer = (item->xD44_lifeTimer - 1.0f);
    if (item->xD44_lifeTimer <= 0.0f) {
        return true;
    }
    return false;
}

/* ---- src/melee/it/kinds/itchicorita.c */
bool itChicoritaleaf_UnkMotion0_Anim(HSD_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    if (item->xD44_lifeTimer <= 0.0f) {
        return true;
    }
    item->xD44_lifeTimer--;
    return false;
}

/* ---- src/melee/it/kinds/itfushigibana.c */
static bool itFushigibana_UnkMotion1_Anim_inline2(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    item->xD44_lifeTimer -= 1.0f;
    if (item->xD44_lifeTimer <= 0) {
        return true;
    } else {
        return false;
    }
}

/* ---- src/melee/it/kinds/itdrmariopill.c */
static bool itDrMarioPill_UnkMotion0_Anim(Item_GObj* gobj)
{
    Item* ip;
    PAD_STACK(8);

    ip = gobj->user_data;
    it_8026B3A8(gobj);
    ip->xD44_lifeTimer -= 1.0f;
    if (ip->xD44_lifeTimer <= 0.0f) {
        return true;
    }
    return false;
}

/* ---- src/melee/it/kinds/itpeachtoadspore.c */
bool itPeachtoadspore_UnkMotion0_Anim(Item_GObj* item_gobj)
{
    Item* item;

    item = GET_ITEM(item_gobj);
    if (item->xDAC_itcmd_var0 != 0) {
        return true;
    }

    item->xD44_lifeTimer--;
    if (item->xD44_lifeTimer <= 0.0F) {
        item->xD44_lifeTimer = 0.0F;
        return true;
    }
    return false;
}

/* ---- src/melee/it/kinds/itsonans.c */
bool itSonans_UnkMotion1_Anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    int zero = 0;

    f32 lifetime = ip->xD44_lifeTimer - 1.0f;
    ip->xD44_lifeTimer = lifetime;

    return (lifetime == zero) ? true : false;
}

/* ---- src/melee/it/kinds/ithitodeman.c */
bool it_802D4FFC(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    f32 timer;

    timer = ip->xD44_lifeTimer - 1.0f;
    ip->xD44_lifeTimer = timer;
    if (timer <= 0.0f) {
        return true;
    }
    if (ip->xDB4_itcmd_var2 != 0) {
        return true;
    }
    return false;
}

/* ---- src/melee/it/kinds/itbox.c */
bool itBox_UnkMotion7_Anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDD4_itemVar.box.despawn_timer -= 1;
    if (ip->xDD4_itemVar.box.despawn_timer > 0) {
        return false;
    }
    return true;
}

/* ---- src/melee/it/kinds/itegg.c */
bool itEgg_UnkMotion6_Anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    --ip->xDD4_itemVar.egg.rand_max;
    return ip->xDD4_itemVar.egg.rand_max > 0 ? false : true;
}

/* ---- src/melee/it/kinds/itfoxlaser.c */
void itFoxlaser_UnkMotion1_Phys(Item_GObj* item_gobj)
{
    Item* item = GET_ITEM(item_gobj);
    item->xDD4_itemVar.foxlaser.pos = item->pos;
}

/* ---- src/melee/it/kinds/itlgunray.c */
void itLgunray_UnkMotion0_Phys(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDD4_itemVar.lgunray.pos = ip->pos;
}

/* ---- src/melee/it/kinds/itcerebi.c */
void itCerebi_UnkMotion2_Phys(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    item->x40_vel.y +=
        ((float*) item->xC4_article_data->x4_specialAttributes)[3];
}

/* ---- src/melee/it/kinds/itfire.c */
void itFire_UnkMotion2_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itFireAttributes* attrs = ip->xC4_article_data->x4_specialAttributes;
    ip->x40_vel.y += attrs->xC;
}

/* ---- src/melee/it/kinds/itmew.c */
void itMew_UnkMotion2_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    MewVars* attr = ip->xC4_article_data->x4_specialAttributes;
    ip->x40_vel.y += attr->xC;
}

/* ---- src/melee/it/kinds/itclimbersice.c */
static inline void itClimbersIce_sub_x4(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itClimbersIceAttributes* attr = ip->xC4_article_data->x4_specialAttributes;
    ip->xD44_lifeTimer -= attr->x4;
}

/* ---- src/melee/it/kinds/itcoin.c */
void itCoin_UnkMotion0_Phys(Item_GObj* gobj)
{
    ItemAttr* attr;
    Item* ip = GET_ITEM(gobj);

    if (ip->xDD4_itemVar.coin.x4 == 0) {
        attr = ip->xCC_item_attr;
        it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
        return;
    }
    it_80272860(gobj, ip->xDD4_itemVar.coin.x8, ip->xDD4_itemVar.coin.xC);
}

/* ---- src/melee/it/kinds/itcoin.c */
void itCoin_UnkMotion3_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itCoinAttributes* attr = ip->xC4_article_data->x4_specialAttributes;

    ip->x40_vel.x *= attr->x34;
    it_80272860(gobj, ip->xDD4_itemVar.coin.x8, ip->xDD4_itemVar.coin.xC);
}

/* ---- src/melee/it/kinds/itdosei.c */
void itDosei_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itDoseiAttributes* attr = ip->xC4_article_data->x4_specialAttributes;
    ip->x40_vel.x = ip->facing_dir * ABS(attr->unk8 * ip->x5D0_animFrameSpeed);
}

/* ---- src/melee/it/kinds/itdrmariopill.c */
static void itDrMarioPill_Motion2_Phys(Item_GObj* gobj)
{
    it_802756D0(gobj);
    it_8026B3A8(gobj);
}

/* ---- src/melee/it/kinds/itdrmariopill.c */
static bool itDrMariopill_Motion2_Coll(Item_GObj* gobj)
{
    it_802756D0(gobj);
    it_8026B3A8(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itdrmariopill.c */
static bool itDrMarioPill_Motion6_Coll(Item_GObj* gobj)
{
    it_8026B3A8(gobj);
    return false;
}

/* ---- src/melee/it/kinds/itfoxillusion.c */
bool itFoxIllusion_Logic14_DmgDealt(Item_GObj* item_gobj)
{
    Item* item = GET_ITEM(item_gobj);
    item->xCA8 = 0;
    return false;
}

/* ---- src/melee/it/kinds/ithitodeman.c */
bool it_802D4EF4(Item_GObj* gobj)
{
    GET_ITEM(gobj)->xDB4_itcmd_var2 = 1;
    return false;
}

/* ---- src/melee/it/kinds/itlipstick.c */
bool itLipstick_UnkMotion2_Anim(Item_GObj* gobj)
{
    GET_ITEM(gobj)->xDAC_itcmd_var0 = 1;
    return false;
}

/* ---- src/melee/it/kinds/itsonans.c */
bool itSonans_Logic9_DmgDealt(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDD4_itemVar.sonans.x60 *= -1.0f;
    return false;
}

/* ---- src/melee/it/kinds/ithitodeman.c */
void itHitodeman_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itHitodemanAttributes* attrs = ip->xC4_article_data->x4_specialAttributes;
    if (ip->xDB0_itcmd_var1 != 0) {
        ip->x40_vel.x = -((-ip->facing_dir * attrs->x4C) - ip->x40_vel.x);
    }
}

/* ---- src/melee/it/kinds/itleadead.c */
static inline void itLeadead_UnkMotion3_Anim_inline(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itLeadeadAttributes* attr = ip->xC4_article_data->x4_specialAttributes;
    ip->xDD4_itemVar.leadead.x48 = attr->x18 * 3;
}

/* ---- src/melee/it/kinds/itleadead.c */
void itLeadead_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itLeadeadAttributes* attr = ip->xC4_article_data->x4_specialAttributes;
    if (ip->xDD4_itemVar.leadead.x48 > 0x24) {
        ip->x40_vel.x = attr->x14 * ip->facing_dir;
    }
    if (ip->xDD4_itemVar.leadead.x48 > 0x71) {
        ip->x40_vel.x = 0.0f;
    }
    ip->xDD4_itemVar.leadead.x48++;
}

/* ---- src/melee/it/kinds/itlizardon.c */
void it_802CC684(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itLizardonAttributes* attrs = ip->xC4_article_data->x4_specialAttributes;

    ip->x40_vel.x *= attrs->x4;
    ip->x40_vel.y *= attrs->x4;
    ip->x40_vel.z *= attrs->x4;
}

/* ---- src/melee/it/kinds/itpeachtoadspore.c */
void itPeachtoadspore_UnkMotion0_Phys(Item_GObj* item_gobj)
{
    itPeachToadSporeAttributes* attr;
    Item* item;

    item = GET_ITEM(item_gobj);
    attr = item->xC4_article_data->x4_specialAttributes;
    item->x40_vel.x *= attr->x8_speed_decay_rate;
    item->x40_vel.y *= attr->x8_speed_decay_rate;
}

/* ---- src/melee/it/kinds/itparasol.c */
void itParasol_UnkMotion1_Phys(Item_GObj* item_gobj)
{
    Item* item = GET_ITEM(item_gobj);
    ItemAttr* attr = item->xCC_item_attr;
    f32* attrs = item->xC4_article_data->x4_specialAttributes;
    it_80272860(item_gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
    decelerateItemX(item, attrs[0]);
}

/* ---- src/melee/it/kinds/itparasol.c */
void itParasol_UnkMotion2_Phys(Item_GObj* item_gobj)
{
    Item* item = GET_ITEM(item_gobj);
    ItemAttr* attr = item->xCC_item_attr;
    f32* attrs = item->xC4_article_data->x4_specialAttributes;
    it_80272860(item_gobj, attr->x10_fall_speed, attr->x14_fall_speed_max);
    decelerateItemX(item, attrs[1]);
}

/* ---- src/melee/it/kinds/itseakneedlethrown.c */
bool itSeakneedlethrown_UnkMotion0_Anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDD4_itemVar.seakneedlethrown.xDE4 = ip->pos;
    return it_80273130(gobj);
}

/* ---- src/melee/it/kinds/itpikachutjoltground.c */
bool itPikachutjoltground_UnkMotion0_Anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDD4_itemVar.pikachujoltground.xDE8 = ip->pos;
    return it_80273130(gobj);
}

/* ---- src/melee/it/kinds/itarwinglaser.c */
static void itArwinglaser_UnkMotion3_Anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);

    ip->xDD4_itemVar.arwinglaser.xE24 = ip->xDD4_itemVar.arwinglaser.xE18;
    ip->xDD4_itemVar.arwinglaser.xE30 = 1;
    it_80273130(gobj);
}

/* ---- src/melee/it/kinds/itnesspkflashexplode.c */
void itNessPKFlashExplode_UnkMotion0_Phys(Item_GObj* gobj)
{
    Item* it = GET_ITEM(gobj);
    it->x40_vel.z = it->x40_vel.y = it->x40_vel.z = 0;
}

/* ---- src/melee/it/kinds/itlipstickspore.c */
void itLipstickspore_UnkMotion0_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);

    if (ip->x40_vel.x == 0.0f) {
        return;
    }

    if (ip->xDD4_itemVar.lipstickspore.xDD8 == 0.0f) {
        return;
    }

    if (ip->x40_vel.x > 0.0f) {
        ip->x40_vel.x -= ip->xDD4_itemVar.lipstickspore.xDD8;
    } else {
        ip->x40_vel.x += ip->xDD4_itemVar.lipstickspore.xDD8;
    }

    {
        f32 abs_vel = ip->x40_vel.x;
        if (abs_vel < 0.0f) {
            abs_vel = -abs_vel;
        }
        if (abs_vel < ip->xDD4_itemVar.lipstickspore.xDD8) {
            ip->x40_vel.x = 0.0f;
        }
    }
}

/* ---- src/melee/it/kinds/itoctarock.c */
void itOctarock_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->ground_or_air == GA_Ground && ip->x40_vel.y > 0.0f) {
        it_802762BC(ip);
    }
}

/* ---- src/melee/it/kinds/itoctarock.c */
void itOctarock_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->xDD4_itemVar.octarock.x2C == 0 && ip->x40_vel.y > 0.0f) {
        it_802762BC(ip);
        ip->xDD4_itemVar.octarock.x2C = 1;
    }
}

/* ---- src/melee/it/kinds/itoctarock.c */
void it_802E5944(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDD4_itemVar.octarock.x20.x = 0.0f;
    ip->xDD4_itemVar.octarock.x20.y = 0.0f;
    ip->xDD4_itemVar.octarock.x20.z = 0.0f;
}

/* ---- src/melee/it/kinds/itnokonoko.c */
void itNokonoko_UnkMotion8_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.y -= ip->xCC_item_attr->x10_fall_speed;
    if (ip->xDD4_itemVar.nokonoko.x28 == 0) {
        it_802756E0(gobj);
        return;
    }
    ip->xDD4_itemVar.nokonoko.x28 -= 1;
}

/* ---- src/melee/it/kinds/itpatapata.c */
bool itPatapata_UnkMotion4_Coll(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    if (ip->xDD4_itemVar.patapata.x20 == 0) {
        ip->xDCC_flag.b3 = true;
        return true;
    }
    ip->xDD4_itemVar.patapata.x20 -= 1;
    return false;
}

/* ---- src/melee/it/kinds/itpeachparasol.c */
bool it_802BDC38(Item_GObj* item_gobj)
{
    Item* item = GET_ITEM(item_gobj);
    if (item->msid != 2) {
        return false;
    }
    if (item->x5CC_currentAnimFrame >= 16.0F) {
        return true;
    }
    return false;
}

/* ---- src/melee/it/kinds/itlinkboomerang.c */
bool it_8029FDBC(Item_GObj* gobj)
{
    /// @todo Use #itGetMotionId if it can be made to inline here.
    return (GET_ITEM(gobj)->msid != 0) ? false : true;
}

/* ---- src/melee/it/kinds/itnesspkflash.c */
bool it_802AA7F0(Item_GObj* gobj)
{
    /// @todo Use #itGetMotionId if it can be made to inline here.
    Item* it = GET_ITEM(gobj);
    if (it->msid == 0x2) {
        return 1;
    }
    return 0;
}

/* ---- src/melee/it/kinds/itrshell.c */
void it_8028D390(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDD4_itemVar.rshell.xDEC--;
    if (ip->xDD4_itemVar.rshell.xDEC == 0) {
        ip->xDD4_itemVar.rshell.xDF0 = 1;
    }
}

/* ---- src/melee/it/kinds/itgamewatchchef.c */
void itGamewatchchef_UnkMotion0_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itGamewatchchefAttributes* attrs =
        ip->xC4_article_data->x4_specialAttributes;
    s32 index = ip->xDD4_itemVar.gamewatchchef.x4;
    it_80272860(gobj, attrs->entries[index].x8, attrs->entries[index].xC);
}

/* ---- src/melee/it/kinds/ithammerhead.c */
void itHammerHead_Logic40_Spawned(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itHammerheadAttributes* attrs = ip->xC4_article_data->x4_specialAttributes;
    it_80275158(gobj, attrs->x4);
}

/* ---- src/melee/it/kinds/ithassam.c */
void itHassam_UnkMotion0_Phys(Item_GObj* gobj)
{
    ItemAttr* temp_r4;
    Item* ip = GET_ITEM(gobj);

    it_8027A344(gobj);
    if (ip->ground_or_air == GA_Air) {
        temp_r4 = ip->xCC_item_attr;
        it_80272860(gobj, temp_r4->x10_fall_speed,
                    temp_r4->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/ithassam.c */
void itHassam_UnkMotion2_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);

    it_8027A344(gobj);
    ip->pos.x += ip->xDD4_itemVar.hassam.x5C.x;
    ip->pos.y += ip->xDD4_itemVar.hassam.x5C.y;
    ip->pos.z += ip->xDD4_itemVar.hassam.x5C.z;
}

/* ---- src/melee/it/kinds/itmaril.c */
void itMaril_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    it_8027A344(gobj);
    ip->x40_vel.x = ip->xDD4_itemVar.maril.x64;
}

/* ---- src/melee/it/kinds/itlinkarrow.c */
void itLinkarrow_UnkMotion1_Phys(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itLinkArrowAttributes* attr = ip->xC4_article_data->x4_specialAttributes;
    ip->xDD4_itemVar.linkarrow.x18 = ip->pos;
    ip->x40_vel.y -= ABS(attr->x1C);
}

/* ---- src/melee/it/kinds/itheart.c */
void it_80283BD4(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    HeartContainerAttr* vars = ip->xC4_article_data->x4_specialAttributes;

    ip->xDD4_itemVar.heart.xDD4_heal = vars->x4.flags;
}

/* ---- src/melee/it/kinds/itheiho.c */
void it_802D98AC(Item_GObj* arg0)
{
    Item* ip = arg0->user_data;
    ip->xDD4_itemVar.heiho.x3C.x = 0.0F;
    ip->xDD4_itemVar.heiho.x3C.y = 0.0F;
    ip->xDD4_itemVar.heiho.x3C.z = 0.0F;
}

/* ---- src/melee/it/kinds/itcoin.c */
int it_802F2020(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);

    return ip->xDD4_itemVar.coin.x0;
}

/* ---- src/melee/it/kinds/itfreeze.c */
void itFreeze_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    f32 decel;
    f32 abs_decel;
    f32 vel;
    f32 abs_vel;

    {
        f32 x10 = ip->xDD4_itemVar.freeze.x10;
        if (x10) {
            ip->x40_vel.x = x10;
            ip->xDD4_itemVar.freeze.x10 = 0.0f * 0.0f;
        }
    }

    decel = ip->xDD4_itemVar.freeze.x14;
    abs_decel = ABS(decel);

    vel = ip->x40_vel.x;
    abs_vel = ABS(vel);

    if (abs_vel < abs_decel) {
        ip->x40_vel.x = 0.0f;
        return;
    }

    if (vel > 0.0f) {
        if (ip->xDD4_itemVar.freeze.x14 < 0.0f) {
            decel = -decel;
        }
        ip->x40_vel.x -= decel;
    } else {
        if (decel < 0.0f) {
            decel = -decel;
        }
        ip->x40_vel.x += decel;
    }
}

/* ---- src/melee/it/kinds/itfreezer.c */
void itFreezer_UnkMotion2_Phys(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    if (item->xDAC_itcmd_var0 != false) {
        itFreezerAttributes* attrs =
            item->xC4_article_data->x4_specialAttributes;
        item->x40_vel.y = attrs->x4;
        item->xDAC_itcmd_var0 = 0;
        item->xDB0_itcmd_var1 = 1;
    }
    if (item->xDB0_itcmd_var1 != false) {
        itFreezerAttributes* attrs =
            item->xC4_article_data->x4_specialAttributes;
        item->x40_vel.y += attrs->x8;
    }
}

/* ---- src/melee/it/kinds/ithouou.c */
void itHouou_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itHououAttr* attr = ip->xC4_article_data->x4_specialAttributes;

    it_8027A344(gobj);

    ip->xDD4_itemVar.houou.vel_accum += attr->x1C;
    ip->x40_vel.y += ip->xDD4_itemVar.houou.vel_accum;

    if (ip->pos.y < ip->xDD4_itemVar.houou.start_pos.y - 10.0f) {
        ip->x40_vel.y = 0.0f;
    }
}

/* ---- src/melee/it/kinds/itkireihana.c */
void itKireihana_UnkMotion4_Phys(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    it_8027A344(gobj);
    item->xDD4_itemVar.kireihana.fall_speed +=
        item->xCC_item_attr->x10_fall_speed;
    if (item->xDD4_itemVar.kireihana.fall_speed >
        item->xCC_item_attr->x14_fall_speed_max)
    {
        item->xDD4_itemVar.kireihana.fall_speed =
            item->xCC_item_attr->x14_fall_speed_max;
    }
    item->x40_vel.y = -item->xDD4_itemVar.kireihana.fall_speed;
}

/* ---- src/melee/it/kinds/itkamex.c */
void itKamex_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = gobj->user_data;

    if (ip->x40_vel.x) {
        if ((ip->x40_vel.x > 0.0f && ip->facing_dir <= 0.0f) ||
            (ip->x40_vel.x <= 0.0f && ip->facing_dir > 0.0f))
        {
            ip->x40_vel.x += ip->xDD4_itemVar.pokemon.x6C * ip->facing_dir;
        }
    }
    if (ip->ground_or_air == GA_Air) {
        it_80272860(gobj, ip->xCC_item_attr->x10_fall_speed,
                    ip->xCC_item_attr->x14_fall_speed_max);
    }
}

/* ---- src/melee/it/kinds/itlinkbomb.c */
void itLinkbomb_UnkMotion4_Phys(HSD_GObj* gobj)
{
    f32 temp_f2;
    Item* item;
    itLinkBombAttributes* sa;
    f32 zero = 0.0F;

    item = GET_ITEM(gobj);
    sa = item->xC4_article_data->x4_specialAttributes;
    if (item->xDD4_itemVar.linkbomb.x0.b1) {
        temp_f2 = item->x40_vel.x;
        if (temp_f2 != zero) {
            item->x40_vel.x =
                (sa->x2C * item->xDD4_itemVar.linkbomb.x4) + temp_f2;
        }
        if (ABS(item->x40_vel.x) < sa->x30) {
            item->x40_vel.x = zero;
        }
    }
}

/* ---- src/melee/it/kinds/itsamusmissile.c */
void itSamusmissile_UnkMotion1_Phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itSamusMissileAttributes* attrs =
        ip->xC4_article_data->x4_specialAttributes;

    if (ip->xDB0_itcmd_var1 != 0) {
        ip->x40_vel.x = attrs->x30 * ip->facing_dir + ip->x40_vel.x;
        {
            float xvel = ABS(ip->x40_vel.x);
            float maxSpeed = attrs->x34;
            if (xvel > maxSpeed) {
                if (ip->facing_dir == +1.0f) {
                    ip->x40_vel.x = +maxSpeed;
                } else {
                    ip->x40_vel.x = -maxSpeed;
                }
            }
        }
    }
}

/* ---- src/melee/it/kinds/itsonans.c */
bool it_802CD7D4(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    itsonansAttributes* attrs = item->xC4_article_data->x4_specialAttributes;
    if (item->xDAC_itcmd_var0 == 0) {
        item->xDD4_itemVar.sonans.x60 =
            attrs->x8 * (item->xCA0 * item->xCCC_incDamageDirection);
        item->xDAC_itcmd_var0 = 1;
    } else {
        if (ABS(item->xDD4_itemVar.sonans.x60) <
            ABS(attrs->x8 * (item->xCA0 * item->xCCC_incDamageDirection)))
        {
            item->xDD4_itemVar.sonans.x60 =
                (attrs->x8 * (item->xCA0 * item->xCCC_incDamageDirection));
        }
    }
    if (item->xCA0 * attrs->x4 > attrs->xC) {
        item->xDD4_itemVar.sonans.x68 = attrs->xC;
    } else {
        item->xDD4_itemVar.sonans.x68 = item->xCA0 * attrs->x4;
    }
    return false;
}

/* ---- src/melee/it/kinds/itrshell.c */
void it_8028D4E4(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    f32 vel;
    itRShell_Attrs* attrs;
    f32 abs_vel;
    abs_vel = ip->x40_vel.x;
    vel = abs_vel;
    attrs = ip->xC4_article_data->x4_specialAttributes;

    if (vel < 0.0f) {
        abs_vel = -vel;
    } else {
        abs_vel = vel;
    }

    if (abs_vel > attrs->xC) {
        s32 sign;
        if (vel < 0.0f) {
            sign = -1;
        } else {
            sign = 1;
        }
        ip->x40_vel.x = (f32) sign;
        ip->x40_vel.x *= attrs->xC;
    }
}

/* ---- src/melee/it/kinds/itgshell.c */
bool itGshell_UnkMotion3_Anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itGShell_Attrs* attrs = ip->xC4_article_data->x4_specialAttributes;
    if (ip->xDD4_itemVar.gshell.xDEC_b1) {
        ip->xDD4_itemVar.gshell.xDE0 -= 1.0f;
        if (ip->xDD4_itemVar.gshell.xDE0 <= 0.0f) {
            ip->xDD4_itemVar.gshell.xDEC_b1 = 0;
            ip->xDD4_itemVar.gshell.xDE0 = attrs->x28;
            it_8027572C(gobj, 0);
            if (ip->xDD4_itemVar.gshell.xDEC_b2) {
                it_802756E0(gobj);
            }
        }
    }
    return false;
}

/* ---- src/melee/it/kinds/itgshell.c */
void it_8028B988(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    itGShell_Attrs* attrs = ip->xC4_article_data->x4_specialAttributes;
    if (ip->xDD4_itemVar.gshell.xDEC_b1) {
        ip->xDD4_itemVar.gshell.xDE0 -= 1.0f;
        if (ip->xDD4_itemVar.gshell.xDE0 <= 0.0f) {
            ip->xDD4_itemVar.gshell.xDEC_b1 = 0;
            ip->xDD4_itemVar.gshell.xDE0 = attrs->x28;
            it_8027572C(gobj, 0);
            if (ip->xDD4_itemVar.gshell.xDEC_b2) {
                it_802756E0(gobj);
            }
        }
    }
}

/* ---- src/melee/it/kinds/itlugia.c */
void it_802D24A0(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    f32* attrs = ip->xC4_article_data->x4_specialAttributes;
    f32 multiplier = 0.0f;

    switch (ip->kind) {
    case It_Kind_Lugia_Aeroblast:
        multiplier = attrs[1];
        break;
    case It_Kind_Lugia_Aeroblast2:
        multiplier = attrs[2];
        break;
    case It_Kind_Lugia_Aeroblast3:
        multiplier = attrs[3];
        break;
    default:
        break;
    }

    ip->x40_vel.z *= multiplier;
}

/* ---- src/melee/it/kinds/itmatadogas.c */
void it_802CB844(Item_GObj* gobj)
{
    Item* ip = gobj->user_data;
    f32 scale = 0.0f;
    ItemKind kind = ip->kind;
    itMatadogasAttributes* attrs = ip->xC4_article_data->x4_specialAttributes;

    switch (kind) {
    case It_Kind_Matadogas_Gas1:
        scale = attrs->x4;
        break;
    case It_Kind_Matadogas_Gas2:
        scale = attrs->x8;
        break;
    default:
        break;
    }

    ip->x40_vel.x *= scale;
    ip->x40_vel.y *= scale;
    ip->x40_vel.z *= scale;
}

/* ---- src/melee/it/kinds/itflipper.c */
void itFlipper_AddSpinImpulse(Item_GObj* gobj, Vec3* pos, f32 vel)
{
    Item* ip = GET_ITEM(gobj);
    itFlipper_DatAttrs* attrs = ip->xC4_article_data->x4_specialAttributes;
    f32 speed = vel + attrs->x1C_baseSpinOnHit;

    if (pos->x > ip->pos.x) {
        if (pos->y > ip->pos.y) {
            ip->xDD4_itemVar.flipper.xDE4_spinVel += speed;
        } else {
            ip->xDD4_itemVar.flipper.xDE4_spinVel -= speed;
        }
    } else {
        if (pos->y > ip->pos.y) {
            ip->xDD4_itemVar.flipper.xDE4_spinVel -= speed;
        } else {
            ip->xDD4_itemVar.flipper.xDE4_spinVel += speed;
        }
    }

    if (ip->xDD4_itemVar.flipper.xDE4_spinVel > attrs->x24_maxSpinVel) {
        ip->xDD4_itemVar.flipper.xDE4_spinVel = attrs->x24_maxSpinVel;
    }
    if (ip->xDD4_itemVar.flipper.xDE4_spinVel < -attrs->x24_maxSpinVel) {
        ip->xDD4_itemVar.flipper.xDE4_spinVel = -attrs->x24_maxSpinVel;
    }
}

/* ---- src/melee/it/kinds/itchicorita.c */
static inline void itChicorita_Phys(HSD_GObj* item_gobj)
{
    Item* item;
    f32 chicorita_fall_speed;
    s32 fall_speed_dir;
    s32 item_vel_y_dir;

    item = item_gobj->user_data;
    it_8027A344(item_gobj);
    if (item->ground_or_air == GA_Air) {
        fall_speed_dir = item->xCC_item_attr->x10_fall_speed < 0.0f ? -1 : 1;
        chicorita_fall_speed = item->xDD4_itemVar.chicorita.x64;
        item_vel_y_dir = chicorita_fall_speed < 0.0f ? -1 : 1;
        if (item_vel_y_dir != fall_speed_dir) {
            if (chicorita_fall_speed < 0.0f) {
                chicorita_fall_speed = -chicorita_fall_speed;
            }
            if (chicorita_fall_speed < item->xCC_item_attr->x14_fall_speed_max)
            {
            block_11:
                item->xDD4_itemVar.chicorita.x64 -=
                    item->xCC_item_attr->x10_fall_speed;
            }
        } else {
            goto block_11;
        }
        item->x40_vel.y = item->xDD4_itemVar.chicorita.x64;
        return;
    }
    item->xDD4_itemVar.chicorita.x64 = 0.0f;
}

/* ---- src/melee/it/kinds/it_2E5A.c */
bool it_802E657C(Item_GObj* item_gobj)
{
    Item* item;
    f32 timer;
    f32 temp_f0_2;
    f32 temp_f1;
    u8 temp_r4;

    item = GET_ITEM(item_gobj);
    timer = item->xD44_lifeTimer;
    if (timer <= 0.0f) {
        return true;
    }
    item->xD44_lifeTimer = timer - 1.0f;
    if (item->xDD4_itemVar.it_2E5A.x8 == 1) {
        temp_f0_2 = item->xDD4_itemVar.it_2E5A.x10;
        if (temp_f0_2 <= 0.0f) {
            temp_r4 = item->xDD4_itemVar.it_2E5A.x18.b0;
            if (!temp_r4) {
                item->xDD4_itemVar.it_2E5A.x18.b0 = 1;
            }
        } else {
            item->xDD4_itemVar.it_2E5A.x10 = temp_f0_2 - 1.0f;
        }
        temp_f1 = item->xDD4_itemVar.it_2E5A.x14;
        if (temp_f1 <= 0.0f) {
            if (item->xDCD_flag.b5 != 1) {
                it_80275444(item_gobj);
                item->xDD4_itemVar.it_2E5A.x18.b1 = 1;
            }
        } else {
            item->xDD4_itemVar.it_2E5A.x14 = temp_f1 - 1.0f;
        }
    }
    return false;
}
