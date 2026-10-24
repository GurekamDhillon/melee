/* TU: src/melee/it/kinds/itfoxblaster.c (written by gen/mtu/mk_itkinds.py: do not edit) */
/* C oracle for melee_items src/tu/itfoxblaster.rs: the whole TU included verbatim; every callee out of it is a
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
#include "itfoxblaster_gen.h"

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
    fill_itFoxBlaster_ItemVars(&ITEM.xDD4_itemVar.foxblaster);
    ITEM.xC4_article_data = &ART; ITEM.xCC_item_attr = &IATTR; ITEM.xBBC_dynamicBoneTable = &DBT;
    fill_Item(&NEWITEM);
    fill_itFoxBlaster_ItemVars(&NEWITEM.xDD4_itemVar.foxblaster);
    NEWITEM.xC4_article_data = &ART; NEWITEM.xCC_item_attr = &IATTR; NEWITEM.xBBC_dynamicBoneTable = &DBT;
    fill_Item(&REFITEM);
    fill_itFoxBlaster_ItemVars(&REFITEM.xDD4_itemVar.foxblaster);
    REFITEM.xC4_article_data = &ART; REFITEM.xCC_item_attr = &IATTR; REFITEM.xBBC_dynamicBoneTable = &DBT;
    ITEM.entity = &G; REFITEM.entity = &REFG; NEWITEM.entity = &NEWG;
    for (i = 0; i < 64; i++) ATTR[i] = rw();
    fill_ItemAttr(&IATTR); fill_ItemCommonData(&ICD);
    ART.x0_common_attr = &IATTR; ART.x4_specialAttributes = ATTR; ART.x10_modelDesc = &MD;
    MD.x0_joint = (rnd() % 3u == 0) ? 0 : &JNT;
    MD.x4_bone_count = fv(); MD.x8_bone_attach_id = (int) fv();
    switch (rnd() % 4u) { case 0: ITEM.owner = 0; break; case 1: ITEM.owner = &OWN; break; default: ITEM.owner = &OWN; break; }
    { static const int t[10] = { 74, 75, 138, 139, 74, 75, 138, 139, 0, 5 }; ITEM.kind = t[rnd() % 10u]; }
    { static const int t[12] = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 }; ITEM.msid = t[rnd() % 12u]; }
    ITEM.xDD4_itemVar.foxblaster.xDD4 = (0) + (int) (rnd() % 14u);
    ITEM.xDD4_itemVar.foxblaster.xDD8 = (0) + (int) (rnd() % 5u);
    { static const int t[5] = { -2, -1, 0, 1, 2 }; ITEM.xDD4_itemVar.foxblaster.xDDC = t[rnd() % 5u]; }
    { static const int t[2] = { 0, 1 }; ITEM.xDD4_itemVar.foxblaster.xDE0 = t[rnd() % 2u]; }
    { static const int t[2] = { 0, 1 }; ITEM.xDD4_itemVar.foxblaster.gfx_spawn_var = t[rnd() % 2u]; }
    { static const int t[4] = { 0, 1, 2, 5 }; ITEM.xDD4_itemVar.foxblaster.set_sfx_var2 = t[rnd() % 4u]; }
    switch (rnd() % 3u) { case 0: ITEM.xDD4_itemVar.foxblaster.owner = 0; break; case 1: ITEM.xDD4_itemVar.foxblaster.owner = &OWN; break; case 2: ITEM.xDD4_itemVar.foxblaster.owner = &OWN; break; }
    switch (rnd() % 4u) { case 0: ITEM.owner = 0; break; case 1: ITEM.owner = &OWN; break; case 2: ITEM.owner = &OWN; break; case 3: ITEM.owner = &REFG; break; }
}

static void hash_all(void)
{
    hash_HSD_GObj(&G);
    hash_HSD_GObj(&OWN);
    hash_HSD_GObj(&REFG);
    hash_HSD_GObj(&PARG);
    hash_HSD_GObj(&NEWG);
    hash_Item(&ITEM);
    hash_itFoxBlaster_ItemVars(&ITEM.xDD4_itemVar.foxblaster);
    hash_Item(&NEWITEM);
    hash_itFoxBlaster_ItemVars(&NEWITEM.xDD4_itemVar.foxblaster);
    hash_Item(&REFITEM);
    hash_itFoxBlaster_ItemVars(&REFITEM.xDD4_itemVar.foxblaster);
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
    for (op = 0; op < 31; op++) {
        for (k = 0; k < (op == 30 ? 1 : 300); k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: {
                u32 a1 = fv();
                u32 a2 = fv();
                itFoxBlaster_PlaySFX(&ITEM, a1, a2);
                break;
            }
            case 1: {
                switch (rnd() % 4u) { case 0: G.user_data = &ITEM; break; case 1: G.user_data = &ITEM; break; case 2: G.user_data = &ITEM; break; case 3: G.user_data = 0; break; }
                HSD_GObj* a0 = (rnd() % 8u == 0) ? 0 : &G;
                int a1 = (0) + (int) (rnd() % 4u);
                it_802ADDD0(a0, a1);
                break;
            }
            case 2: {
                it_802ADEF0(&G);
                break;
            }
            case 3: {
                fillvec(&VEC0);
                fillvec(&VEC1);
                itFoxBlaster_SetShotOffset(&ITEM, &VEC0, &VEC1);
                break;
            }
            case 4: {
                fillvec(&VEC0);
                fillvec(&VEC1);
                itFoxBlaster_SetShotAngle(&ITEM, &VEC0, &VEC1);
                break;
            }
            case 5: {
                it_802ADF10(&G);
                break;
            }
            case 6: {
                it_802AE1D0(&G);
                break;
            }
            case 7: {
                switch (rnd() % 4u) { case 0: G.user_data = &ITEM; break; case 1: G.user_data = &ITEM; break; case 2: G.user_data = &ITEM; break; case 3: G.user_data = 0; break; }
                it_802AE200(&G);
                break;
            }
            case 8: {
                it_802AE538(&G);
                break;
            }
            case 9: {
                it_802AE608(&G);
                break;
            }
            case 10: {
                switch (rnd() % 4u) { case 0: G.user_data = &ITEM; break; case 1: G.user_data = &ITEM; break; case 2: G.user_data = &ITEM; break; case 3: G.user_data = 0; break; }
                it_802AE63C(&G);
                break;
            }
            case 11: {
                int a1 = (int) fv();
                itFoxBlaster_SetCommandVars(&ITEM, a1);
                break;
            }
            case 12: {
                int a1 = (0) + (int) (rnd() % 6u);
                itFoxBlaster_ClearShot(&G, a1);
                break;
            }
            case 13: {
                it_802AE7B8(&G);
                break;
            }
            case 14: {
                float a0 = rw();
                HSD_GObj* a1 = (rnd() % 4u == 0) ? 0 : &REFG;
                fillvec(&VEC0);
                int a3 = (int) fv();
                int a4 = (int) fv();
                ret = PI(it_802AE8A8(a0, a1, &VEC0, a3, a4));
                break;
            }
            case 15: {
                HSD_GObj* a0 = (rnd() % 4u == 0) ? 0 : &REFG;
                int a1 = (int) fv();
                int a2 = (int) fv();
                ret = PI(it_802AE994(a0, a1, a2));
                break;
            }
            case 16: {
                HSD_GObj* a0 = (rnd() % 8u == 0) ? 0 : &G;
                it_802AEAB4(a0);
                break;
            }
            case 17: {
                itFoxBlaster_Logic96_PickedUp(&G);
                break;
            }
            case 18: {
                switch (rnd() % 4u) { case 0: G.user_data = &ITEM; break; case 1: G.user_data = &ITEM; break; case 2: G.user_data = &ITEM; break; case 3: G.user_data = 0; break; }
                clear_blaster_references(&G);
                break;
            }
            case 19: {
                clear_blaster(&G);
                break;
            }
            case 20: {
                ret = (u32) itFoxblaster_UnkMotion8_Anim(&G);
                break;
            }
            case 21: {
                itFoxblaster_UnkMotion8_Phys(&G);
                break;
            }
            case 22: {
                ret = (u32) itFoxblaster_UnkMotion8_Coll(&G);
                break;
            }
            case 23: {
                ret = (u32) itFoxblaster_UnkMotion9_Anim(&G);
                break;
            }
            case 24: {
                itFoxblaster_UnkMotion9_Phys(&G);
                break;
            }
            case 25: {
                ret = (u32) itFoxblaster_UnkMotion9_Coll(&G);
                break;
            }
            case 26: {
                switch (rnd() % 4u) { case 0: G.user_data = &ITEM; break; case 1: G.user_data = &ITEM; break; case 2: G.user_data = &ITEM; break; case 3: G.user_data = 0; break; }
                HSD_GObj* a0 = (rnd() % 8u == 0) ? 0 : &G;
                ret = (u32) itFoxblaster_UnkMotion10_Anim(a0);
                break;
            }
            case 27: {
                itFoxblaster_UnkMotion10_Phys(&G);
                break;
            }
            case 28: {
                ret = (u32) itFoxblaster_UnkMotion10_Coll(&G);
                break;
            }
            case 29: {
                itFoxBlaster_Logic96_EvtUnk(&G, &REFG);
                break;
            }
            case 30: {
                for (i = 0; i < 11; i++) { HM((u32) it_803F6CA8[i].anim_id); HM(FI((const void*) it_803F6CA8[i].animated)); HM(FI((const void*) it_803F6CA8[i].physics_updated)); HM(FI((const void*) it_803F6CA8[i].collided)); }
                for (i = 0; i < 14; i++) HM(B(it_803F6D58[i]));
                for (i = 0; i < 14; i++) HM(B(it_803F6D90[i]));
                for (i = 0; i < 14; i++) HM(B(it_803F6DC8[i]));
                for (i = 0; i < 5; i++) HM(B(it_803F6E28[i]));
                for (i = 0; i < 11; i++) HM((u32) it_803F6E3C[i]);
                for (i = 0; i < 11; i++) HM((u32) it_803F6E68[i]);
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
