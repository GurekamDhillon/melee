/* TU: src/melee/it/kinds/itfreeze.c (written by gen/mtu/mk_itkinds.py: do not edit) */
/* C oracle for melee_items src/tu/itfreeze.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj G, OWN, REFG, PARG, NEWG;
static Item ITEM, NEWITEM, REFITEM;
static HSD_JObj JO[8], OJO, GJO, LJO;
static Article ART;
static ItemModelDesc MD;
static ItemAttr IATTR;
static ItemCommonData ICD;
static HSD_Joint JNT;
static DynamicBoneTable DBT;
static double ATTRD[32];
#define ATTR ((float*) ATTRD)
static Vec3 VEC0, VEC1;
#include "itfreeze_gen.h"

static void fillvec(Vec3* v)
{
    v->x = rw();
    v->y = rw();
    v->z = rw();
}

static void setup(void)
{
    int i;
    zero(&G, sizeof G); zero(&OWN, sizeof OWN); zero(&REFG, sizeof REFG); zero(&PARG, sizeof PARG); zero(&NEWG, sizeof NEWG);
    zero(&ITEM, sizeof ITEM); zero(&NEWITEM, sizeof NEWITEM); zero(&REFITEM, sizeof REFITEM);
    zero(JO, sizeof JO); zero(&OJO, sizeof OJO); zero(&GJO, sizeof GJO); zero(&LJO, sizeof LJO);
    zero(&ART, sizeof ART); zero(&MD, sizeof MD); zero(&IATTR, sizeof IATTR); zero(&ICD, sizeof ICD);
    zero(&JNT, sizeof JNT); zero(&DBT, sizeof DBT); zero(ATTRD, sizeof ATTRD); zero(&VEC0, sizeof VEC0); zero(&VEC1, sizeof VEC1);
    fill_HSD_GObj(&G);
    fill_HSD_GObj(&OWN);
    fill_HSD_GObj(&REFG);
    fill_HSD_GObj(&PARG);
    fill_HSD_GObj(&NEWG);
    G.user_data = &ITEM; G.hsd_obj = &JO[0]; OWN.hsd_obj = &OJO; REFG.user_data = &REFITEM; REFG.hsd_obj = &OJO; PARG.hsd_obj = &OJO; NEWG.user_data = &NEWITEM; NEWG.hsd_obj = &OJO;
    fill_HSD_JObj(&JO[0]); JO[0].flags &= ~(u32) JOBJ_USE_QUATERNION;
    fill_HSD_JObj(&JO[1]); JO[1].flags &= ~(u32) JOBJ_USE_QUATERNION;
    fill_HSD_JObj(&JO[2]); JO[2].flags &= ~(u32) JOBJ_USE_QUATERNION;
    fill_HSD_JObj(&JO[3]); JO[3].flags &= ~(u32) JOBJ_USE_QUATERNION;
    fill_HSD_JObj(&JO[4]); JO[4].flags &= ~(u32) JOBJ_USE_QUATERNION;
    fill_HSD_JObj(&JO[5]); JO[5].flags &= ~(u32) JOBJ_USE_QUATERNION;
    fill_HSD_JObj(&JO[6]); JO[6].flags &= ~(u32) JOBJ_USE_QUATERNION;
    fill_HSD_JObj(&JO[7]); JO[7].flags &= ~(u32) JOBJ_USE_QUATERNION;
    fill_HSD_JObj(&OJO); OJO.flags &= ~(u32) JOBJ_USE_QUATERNION;
    fill_HSD_JObj(&GJO); GJO.flags &= ~(u32) JOBJ_USE_QUATERNION;
    fill_HSD_JObj(&LJO); LJO.flags &= ~(u32) JOBJ_USE_QUATERNION;
    if (rnd() % 6u != 0) JO[0].child = &JO[1];
    if (rnd() % 6u != 0) JO[1].child = &JO[2];
    if (rnd() % 6u != 0) JO[2].child = &JO[3];
    if (rnd() % 6u != 0) JO[3].child = &JO[4];
    JO[4].next = &JO[5]; JO[5].next = &JO[6]; JO[6].next = &JO[7];
    for (i = 0; i < 100; i++) DBT.bones[i] = &JO[i % 8];
    fill_Item(&ITEM);
    fill_itFreeze_ItemVars(&ITEM.xDD4_itemVar.freeze);
    ITEM.xC4_article_data = &ART; ITEM.xCC_item_attr = &IATTR; ITEM.xBBC_dynamicBoneTable = &DBT;
    fill_Item(&NEWITEM);
    fill_itFreeze_ItemVars(&NEWITEM.xDD4_itemVar.freeze);
    NEWITEM.xC4_article_data = &ART; NEWITEM.xCC_item_attr = &IATTR; NEWITEM.xBBC_dynamicBoneTable = &DBT;
    fill_Item(&REFITEM);
    fill_itFreeze_ItemVars(&REFITEM.xDD4_itemVar.freeze);
    REFITEM.xC4_article_data = &ART; REFITEM.xCC_item_attr = &IATTR; REFITEM.xBBC_dynamicBoneTable = &DBT;
    ITEM.entity = &G; REFITEM.entity = &REFG; NEWITEM.entity = &NEWG;
    for (i = 0; i < 64; i++) ATTR[i] = rw();
    fill_itUnkAttributes((void*) ATTR);
    fill_ItemAttr(&IATTR); fill_ItemCommonData(&ICD);
    ART.x0_common_attr = &IATTR; ART.x4_specialAttributes = ATTR; ART.x10_modelDesc = &MD;
    MD.x0_joint = (rnd() % 3u == 0) ? 0 : &JNT;
    MD.x4_bone_count = fv(); MD.x8_bone_attach_id = (int) fv();
    switch (rnd() % 4u) { case 0: ITEM.owner = 0; break; case 1: ITEM.owner = &OWN; break; default: ITEM.owner = &OWN; break; }
    it_804D6D28 = &ICD;
    { static const int t[3] = { 0, 1, 1 }; ITEM.ground_or_air = t[rnd() % 3u]; }
    ITEM.xDD4_itemVar.freeze.x0 = rg();
    ITEM.xDD4_itemVar.freeze.x10 = rg();
    ITEM.xDD4_itemVar.freeze.x14 = rg();
    { static const int t[5] = { -1, 0, 1, 2, 90 }; ITEM.xDD4_itemVar.freeze.x18 = t[rnd() % 5u]; }
    switch (rnd() % 2u) { case 0: ITEM.xDD4_itemVar.freeze.unk_1C = 0; break; case 1: ITEM.xDD4_itemVar.freeze.unk_1C = &REFG; break; }
    { static const float t[3] = { -1.0f, 1.0f, 0.5f }; ITEM.facing_dir = t[rnd() % 3u]; }
}

static void hash_all(void)
{
    hash_HSD_GObj(&G);
    hash_HSD_GObj(&OWN);
    hash_HSD_GObj(&REFG);
    hash_HSD_GObj(&PARG);
    hash_HSD_GObj(&NEWG);
    hash_Item(&ITEM);
    hash_itFreeze_ItemVars(&ITEM.xDD4_itemVar.freeze);
    hash_Item(&NEWITEM);
    hash_itFreeze_ItemVars(&NEWITEM.xDD4_itemVar.freeze);
    hash_Item(&REFITEM);
    hash_itFreeze_ItemVars(&REFITEM.xDD4_itemVar.freeze);
    hash_HSD_JObj(&JO[0]);
    hash_HSD_JObj(&JO[1]);
    hash_HSD_JObj(&JO[2]);
    hash_HSD_JObj(&JO[3]);
    hash_HSD_JObj(&JO[4]);
    hash_HSD_JObj(&JO[5]);
    hash_HSD_JObj(&JO[6]);
    hash_HSD_JObj(&JO[7]);
    hash_HSD_JObj(&OJO);
    hash_HSD_JObj(&GJO);
    hash_HSD_JObj(&LJO);
    HM(RH(ATTRD, sizeof ATTRD)); hash_ItemAttr(&IATTR);
    HM(B(VEC0.x));
    HM(B(VEC0.y));
    HM(B(VEC0.z));
    HM(B(VEC1.x));
    HM(B(VEC1.y));
    HM(B(VEC1.z));
}

int main(void)
{
    int op, k, i;
    preg(&G, sizeof G); preg(&OWN, sizeof OWN); preg(&REFG, sizeof REFG); preg(&PARG, sizeof PARG); preg(&NEWG, sizeof NEWG);
    preg(&ITEM, sizeof ITEM); preg(&NEWITEM, sizeof NEWITEM); preg(&REFITEM, sizeof REFITEM);
    preg(JO, sizeof JO); preg(&OJO, sizeof OJO); preg(&GJO, sizeof GJO); preg(&LJO, sizeof LJO);
    preg(&ART, sizeof ART); preg(&MD, sizeof MD); preg(&IATTR, sizeof IATTR); preg(&ICD, sizeof ICD);
    preg(&JNT, sizeof JNT); preg(&DBT, sizeof DBT); preg(ATTRD, sizeof ATTRD); preg(&VEC0, sizeof VEC0); preg(&VEC1, sizeof VEC1);
    freg_all();
    for (op = 0; op < 43; op++) {
        for (k = 0; k < (op == 42 ? 1 : 300); k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: {
                HSD_GObj* a0 = (rnd() % 4u == 0) ? 0 : &REFG;
                fillvec(&VEC0);
                float a2 = rw();
                float a3 = rw();
                ret = PI(it_8028EB88(a0, &VEC0, a2, a3));
                break;
            }
            case 1: {
                float a1 = rw();
                it_8028EC98(&G, a1);
                break;
            }
            case 2: {
                it_8028ECE0(&G);
                break;
            }
            case 3: {
                fillvec(&VEC0);
                ret = PI(it_8028ECF0(&G, &VEC0));
                break;
            }
            case 4: {
                itFreeze_Logic17_Destroyed(&G);
                break;
            }
            case 5: {
                it_3F14_Logic17_Spawned(&G);
                break;
            }
            case 6: {
                it_8028EDBC(&G);
                break;
            }
            case 7: {
                it_8028EF34(&G);
                break;
            }
            case 8: {
                ret = (u32) itFreeze_UnkMotion0_Anim(&G);
                break;
            }
            case 9: {
                itFreeze_UnkMotion0_Phys(&G);
                break;
            }
            case 10: {
                ret = (u32) itFreeze_UnkMotion0_Coll(&G);
                break;
            }
            case 11: {
                it_8028F1D8(&G);
                break;
            }
            case 12: {
                ret = (u32) itFreeze_UnkMotion3_Anim(&G);
                break;
            }
            case 13: {
                itFreeze_UnkMotion1_Phys(&G);
                break;
            }
            case 14: {
                ret = (u32) itFreeze_UnkMotion1_Coll(&G);
                break;
            }
            case 15: {
                itFreeze_Logic17_PickedUp(&G);
                break;
            }
            case 16: {
                ret = (u32) itFreeze_UnkMotion2_Anim(&G);
                break;
            }
            case 17: {
                itFreeze_Logic17_Dropped(&G);
                break;
            }
            case 18: {
                itFreeze_Logic17_Thrown(&G);
                break;
            }
            case 19: {
                itFreeze_UnkMotion3_Phys(&G);
                break;
            }
            case 20: {
                ret = (u32) itFreeze_UnkMotion3_Coll(&G);
                break;
            }
            case 21: {
                ret = (u32) itFreeze_Logic17_DmgDealt(&G);
                break;
            }
            case 22: {
                ret = (u32) itFreeze_Logic17_Clanked(&G);
                break;
            }
            case 23: {
                ret = (u32) itFreeze_Logic17_HitShield(&G);
                break;
            }
            case 24: {
                ret = (u32) itFreeze_Logic17_Absorbed(&G);
                break;
            }
            case 25: {
                ret = (u32) itFreeze_Logic17_Reflected(&G);
                break;
            }
            case 26: {
                ret = (u32) itFreeze_Logic17_ShieldBounced(&G);
                break;
            }
            case 27: {
                ret = (u32) itFreeze_Logic17_DmgReceived(&G);
                break;
            }
            case 28: {
                float a1 = rw();
                HSD_GObj* a2 = (rnd() % 4u == 0) ? 0 : &REFG;
                it_8028F434(&G, a1, a2);
                break;
            }
            case 29: {
                ret = (u32) itFreeze_UnkMotion4_Anim(&G);
                break;
            }
            case 30: {
                itFreeze_UnkMotion4_Phys(&G);
                break;
            }
            case 31: {
                itFreeze_ClearLinkedItem(&G);
                break;
            }
            case 32: {
                itFreeze_ResetToMotion0(&G);
                break;
            }
            case 33: {
                ret = (u32) itFreeze_UnkMotion4_Coll(&G);
                break;
            }
            case 34: {
                it_8028F7C8(&G);
                break;
            }
            case 35: {
                ret = (u32) itFreeze_UnkMotion5_Anim(&G);
                break;
            }
            case 36: {
                itFreeze_UnkMotion5_Phys(&G);
                break;
            }
            case 37: {
                ret = (u32) itFreeze_UnkMotion5_Coll(&G);
                break;
            }
            case 38: {
                itFreeze_Logic17_EvtUnk(&G, &REFG);
                break;
            }
            case 39: {
                it_8028F8E4(&G);
                break;
            }
            case 40: {
                it_8028F968(&G);
                break;
            }
            case 41: {
                it_8028F9B8(&G);
                break;
            }
            case 42: {
                for (i = 0; i < 6; i++) { HM((u32) it_803F5D28[i].anim_id); HM(FI((const void*) it_803F5D28[i].animated)); HM(FI((const void*) it_803F5D28[i].physics_updated)); HM(FI((const void*) it_803F5D28[i].collided)); }
                break;
            }
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
            fflush(0);
        }
    }
    return 0;
}
