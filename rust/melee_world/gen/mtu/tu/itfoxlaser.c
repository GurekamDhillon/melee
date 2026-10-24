/* TU: src/melee/it/kinds/itfoxlaser.c (written by gen/mtu/mk_itkinds.py: do not edit) */
/* C oracle for melee_items src/tu/itfoxlaser.rs: the whole TU included verbatim; every callee out of it is a
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
#include "itfoxlaser_gen.h"

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
    fill_itFoxLaser_ItemVars(&ITEM.xDD4_itemVar.foxlaser);
    ITEM.xC4_article_data = &ART; ITEM.xCC_item_attr = &IATTR; ITEM.xBBC_dynamicBoneTable = &DBT;
    fill_Item(&NEWITEM);
    fill_itFoxLaser_ItemVars(&NEWITEM.xDD4_itemVar.foxlaser);
    NEWITEM.xC4_article_data = &ART; NEWITEM.xCC_item_attr = &IATTR; NEWITEM.xBBC_dynamicBoneTable = &DBT;
    fill_Item(&REFITEM);
    fill_itFoxLaser_ItemVars(&REFITEM.xDD4_itemVar.foxlaser);
    REFITEM.xC4_article_data = &ART; REFITEM.xCC_item_attr = &IATTR; REFITEM.xBBC_dynamicBoneTable = &DBT;
    ITEM.entity = &G; REFITEM.entity = &REFG; NEWITEM.entity = &NEWG;
    for (i = 0; i < 64; i++) ATTR[i] = rw();
    fill_FoxLaserAttr((void*) ATTR);
    fill_ItemAttr(&IATTR); fill_ItemCommonData(&ICD);
    ART.x0_common_attr = &IATTR; ART.x4_specialAttributes = ATTR; ART.x10_modelDesc = &MD;
    MD.x0_joint = (rnd() % 3u == 0) ? 0 : &JNT;
    MD.x4_bone_count = fv(); MD.x8_bone_attach_id = (int) fv();
    switch (rnd() % 4u) { case 0: ITEM.owner = 0; break; case 1: ITEM.owner = &OWN; break; default: ITEM.owner = &OWN; break; }
}

static void hash_all(void)
{
    hash_HSD_GObj(&G);
    hash_HSD_GObj(&OWN);
    hash_HSD_GObj(&REFG);
    hash_HSD_GObj(&PARG);
    hash_HSD_GObj(&NEWG);
    hash_Item(&ITEM);
    hash_itFoxLaser_ItemVars(&ITEM.xDD4_itemVar.foxlaser);
    hash_Item(&NEWITEM);
    hash_itFoxLaser_ItemVars(&NEWITEM.xDD4_itemVar.foxlaser);
    hash_Item(&REFITEM);
    hash_itFoxLaser_ItemVars(&REFITEM.xDD4_itemVar.foxlaser);
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
    for (op = 0; op < 14; op++) {
        for (k = 0; k < (op == 13 ? 1 : 300); k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: {
                ret = (u32) it_8029C4D4(&G, 0);
                break;
            }
            case 1: {
                fillvec(&VEC0);
                int a2 = (0) + (int) (rnd() % 4u);
                int a3 = (int) fv();
                float a4 = rw();
                float a5 = rw();
                it_8029C504(&PARG, &VEC0, a2, a3, a4, a5);
                break;
            }
            case 2: {
                float a0 = rw();
                float a1 = rw();
                fillvec(&VEC0);
                int a4 = (int) fv();
                it_8029C6A4(a0, a1, &PARG, &VEC0, a4);
                break;
            }
            case 3: {
                float a0 = rw();
                float a1 = rw();
                fillvec(&VEC0);
                int a4 = (int) fv();
                it_8029C6CC(a0, a1, &PARG, &VEC0, a4);
                break;
            }
            case 4: {
                ret = (u32) itFoxlaser_UnkMotion1_Anim(&G);
                break;
            }
            case 5: {
                itFoxlaser_UnkMotion1_Phys(&G);
                break;
            }
            case 6: {
                ret = (u32) itFoxlaser_UnkMotion1_Coll(&G);
                break;
            }
            case 7: {
                ret = (u32) itFoxLaser_Logic94_Clanked(&G);
                break;
            }
            case 8: {
                { u32 r = rnd(); if (r % 2u == 0) ITEM.xC68 = ITEM.facing_dir; else ITEM.xC68 = rw(); }
                ret = (u32) itFoxLaser_Logic94_Reflected(&G);
                break;
            }
            case 9: {
                ret = (u32) itFoxLaser_Logic94_Absorbed(&G);
                break;
            }
            case 10: {
                ret = (u32) itFoxLaser_Logic94_ShieldBounced(&G);
                break;
            }
            case 11: {
                ret = (u32) itFoxLaser_Logic94_HitShield(&G);
                break;
            }
            case 12: {
                itFoxLaser_Logic94_EvtUnk(&G, &REFG);
                break;
            }
            case 13: {
                for (i = 0; i < 2; i++) { HM((u32) it_803F67D0[i].anim_id); HM(FI((const void*) it_803F67D0[i].animated)); HM(FI((const void*) it_803F67D0[i].physics_updated)); HM(FI((const void*) it_803F67D0[i].collided)); }
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
