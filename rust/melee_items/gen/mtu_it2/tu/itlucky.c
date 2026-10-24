/* TU: src/melee/it/kinds/itlucky.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu_it2/mk_it_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj G, G2, OWN;
static Item IT, IT2;
static Article ART;
static ItemAttr IA;
static unsigned int SA[256];
static HSD_JObj JO[4];
static DynamicBoneTable DBT;
static Vec3 V1, V2;
static ItemCommonData ICD;
ItemCommonData* it_804D6D28;


#include "itlucky_gen.h"

static void fill_jo(HSD_JObj* j)
{
    j->flags = rnd() & (JOBJ_MTX_INDEP_SRT | JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX | JOBJ_HIDDEN);
    j->translate.x = rw(); j->translate.y = rw(); j->translate.z = rw();
    j->rotate.x = rw(); j->rotate.y = rw(); j->rotate.z = rw(); j->rotate.w = rw();
    j->scale.x = rw(); j->scale.y = rw(); j->scale.z = rw();
}
static void hash_jo(const HSD_JObj* j)
{
    HM(j->flags);
    HM(B(j->translate.x)); HM(B(j->translate.y)); HM(B(j->translate.z));
    HM(B(j->rotate.x)); HM(B(j->rotate.y)); HM(B(j->rotate.z)); HM(B(j->rotate.w));
    HM(B(j->scale.x)); HM(B(j->scale.y)); HM(B(j->scale.z));
    HM(PI(j->child)); HM(PI(j->next));
}



static void setup(void)
{
    int i;
    zero(&G, sizeof G); zero(&G2, sizeof G2); zero(&OWN, sizeof OWN); zero(&IT, sizeof IT); zero(&IT2, sizeof IT2);
    zero(&ART, sizeof ART); zero(&IA, sizeof IA); zero(SA, sizeof SA); zero(JO, sizeof JO); zero(&DBT, sizeof DBT);
    zero(&ICD, sizeof ICD);
    fill_HSD_GObj(&G); fill_HSD_GObj(&G2); fill_HSD_GObj(&OWN);
    fill_Item(&IT); fill_Item(&IT2); fill_ItemAttr(&IA); fill_ItemCommonData(&ICD);
    for (i = 0; i < 4; i++) fill_jo(&JO[i]);
    JO[0].child = &JO[1]; JO[1].child = &JO[2]; JO[1].next = &JO[3]; JO[0].next = 0;
    for (i = 0; i < 256; i++) SA[i] = (rnd() & 1) ? fv() : B(rw());
    V1.x = rw(); V1.y = rw(); V1.z = rw(); V2.x = rw(); V2.y = rw(); V2.z = rw();
    G.user_data = &IT; G.hsd_obj = &JO[0]; G2.user_data = &IT2; G2.hsd_obj = &JO[3]; OWN.user_data = 0; OWN.hsd_obj = 0;
    IT.entity = &G; IT2.entity = &G2;
    IT.xC4_article_data = &ART; IT2.xC4_article_data = &ART; ART.x4_specialAttributes = SA; ART.x0_common_attr = &IA;
    IT.xCC_item_attr = &IA; IT2.xCC_item_attr = &IA;
    IT.xBBC_dynamicBoneTable = &DBT; IT2.xBBC_dynamicBoneTable = &DBT;
    for (i = 0; i < 100; i++) DBT.bones[i] = &JO[i & 3];
    IT.owner = (rnd() & 1) ? &OWN : 0; IT2.owner = 0;
    it_804D6D28 = &ICD;
    if (rnd() % 4 != 0) { u32 q = rnd() % 5; IT.xDD4_itemVar.lucky.x60 = (q == 0 ? 0 : (q == 1 ? 1 : (q == 2 ? 2 : (q == 3 ? 3 : 4)))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xDD4_itemVar.lucky.x64 = (q == 0 ? 0 : (q == 1 ? 1 : (q == 2 ? 2 : 5))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDD4_itemVar.lucky.x68 = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDAC_itcmd_var0 = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 5; IT.xD44_lifeTimer = (q == 0 ? 0.0f : (q == 1 ? 1.0f : (q == 2 ? -1.0f : (q == 3 ? 30.0f : 1.5f)))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xC9C = (q == 0 ? 0 : (q == 1 ? 1 : 100)); }

}

static void pre_hash(void)
{

}

static void hash_all(void)
{
    int i;
    hash_HSD_GObj(&G); hash_HSD_GObj(&G2); hash_HSD_GObj(&OWN);
    hash_Item(&IT); hash_Item(&IT2); hash_ItemAttr(&IA);
    for (i = 0; i < 4; i++) hash_jo(&JO[i]);
    for (i = 0; i < 256; i++) HM(SA[i]);
    HM(B(V1.x)); HM(B(V1.y)); HM(B(V1.z)); HM(B(V2.x)); HM(B(V2.y)); HM(B(V2.z));

}

int main(void)
{
    int op, k;
    preg(&G, sizeof G);
    preg(&G2, sizeof G2);
    preg(&OWN, sizeof OWN);
    preg(&IT, sizeof IT);
    preg(&IT2, sizeof IT2);
    preg(&ART, sizeof ART);
    preg(&IA, sizeof IA);
    preg(SA, sizeof SA);
    preg(JO, sizeof JO);
    preg(&DBT, sizeof DBT);
    preg(&V1, sizeof V1);
    preg(&V2, sizeof V2);
    preg(&ICD, sizeof ICD);
    freg_all();
    freg((const void*) it_802D5050);
    freg((const void*) it_802D50F0);
    freg((const void*) it_802D5124);
    freg((const void*) it_802D51C8);
    freg((const void*) it_802D52E4);
    freg((const void*) it_802D533C_inline);
    freg((const void*) it_802D533C);
    freg((const void*) it_802D53AC);
    freg((const void*) it_802D53F0);
    freg((const void*) it_802D53F4);
    freg((const void*) it_802D5420);
    freg((const void*) it_802D546C_inline);
    freg((const void*) it_802D546C);
    freg((const void*) itLucky_UnkMotion5_Anim);
    freg((const void*) itLucky_UnkMotion5_Phys);
    freg((const void*) itLucky_UnkMotion5_Coll);
    freg((const void*) itLucky_UnkMotion6_Anim);
    freg((const void*) itLucky_UnkMotion6_Phys);
    freg((const void*) it_802D5560);
    freg((const void*) it_802D55DC);
    freg((const void*) it_802D5600);
    freg((const void*) it_802D5620);
    freg((const void*) it_802D5648_inline);
    freg((const void*) it_802D5648);
    freg((const void*) it_802D56F0);
    freg((const void*) it_802D5710_inline);
    freg((const void*) it_802D5710);
    freg((const void*) itLucky_Logic44_Spawned);
    freg((const void*) it_802D582C);
    freg((const void*) it_802D5884);
    freg((const void*) it_802D58BC);
    freg((const void*) it_802D58C0);
    freg((const void*) it_802D58EC);
    freg((const void*) itLucky_UnkMotion3_Anim);
    freg((const void*) itLucky_UnkMotion3_Phys);
    freg((const void*) itLucky_UnkMotion3_Coll);
    freg((const void*) itLucky_Logic44_PickedUp);
    freg((const void*) itLucky_UnkMotion2_Anim);
    freg((const void*) itLucky_UnkMotion2_Phys);
    freg((const void*) itLucky_Logic44_Dropped);
    freg((const void*) itLucky_Logic44_EnteredAir);
    freg((const void*) it_802D5A2C);
    freg((const void*) it_802D5A64);
    freg((const void*) it_802D5A68);
    freg((const void*) itLucky_Logic44_EvtUnk);
    for (op = 0; op < 47; op++) {
        for (k = 0; k < 150; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { it_802D5050(&G); } break;
            case 1: { it_802D50F0(&G); } break;
            case 2: { ret = (u32) it_802D5124(&G); } break;
            case 3: { it_802D51C8(&G); } break;
            case 4: { it_802D52E4(&G); } break;
            case 5: { it_802D533C_inline(&G); } break;
            case 6: { it_802D533C(&G); } break;
            case 7: { ret = (u32) it_802D53AC(&G); } break;
            case 8: { it_802D53F0(&G); } break;
            case 9: { ret = (u32) it_802D53F4(&G); } break;
            case 10: { it_802D5420(&G); } break;
            case 11: { it_802D546C_inline(&G); } break;
            case 12: { it_802D546C(&G); } break;
            case 13: { ret = (u32) itLucky_UnkMotion5_Anim(&G); } break;
            case 14: { itLucky_UnkMotion5_Phys(&G); } break;
            case 15: { ret = (u32) itLucky_UnkMotion5_Coll(&G); } break;
            case 16: { ret = (u32) itLucky_UnkMotion6_Anim(&G); } break;
            case 17: { itLucky_UnkMotion6_Phys(&G); } break;
            case 18: { it_802D5560(&G); } break;
            case 19: { ret = (u32) it_802D55DC(&G); } break;
            case 20: { it_802D5600(&G); } break;
            case 21: { ret = (u32) it_802D5620(&G); } break;
            case 22: { it_802D5648_inline(&G); } break;
            case 23: { ret = (u32) it_802D5648(&G); } break;
            case 24: { it_802D56F0(&G, &G2); } break;
            case 25: { ret = PI(it_802D5710_inline(&G, &V1, &V1, rw())); } break;
            case 26: { ret = PI(it_802D5710(&G, &V1, &V1, rw())); } break;
            case 27: { itLucky_Logic44_Spawned(&G); } break;
            case 28: { it_802D582C(&G); } break;
            case 29: { ret = (u32) it_802D5884(&G); } break;
            case 30: { it_802D58BC(&G); } break;
            case 31: { ret = (u32) it_802D58C0(&G); } break;
            case 32: { it_802D58EC(&G); } break;
            case 33: { ret = (u32) itLucky_UnkMotion3_Anim(&G); } break;
            case 34: { itLucky_UnkMotion3_Phys(&G); } break;
            case 35: { ret = (u32) itLucky_UnkMotion3_Coll(&G); } break;
            case 36: { itLucky_Logic44_PickedUp(&G); } break;
            case 37: { ret = (u32) itLucky_UnkMotion2_Anim(&G); } break;
            case 38: { itLucky_UnkMotion2_Phys(&G); } break;
            case 39: { itLucky_Logic44_Dropped(&G); } break;
            case 40: { itLucky_Logic44_EnteredAir(&G); } break;
            case 41: { ret = (u32) it_802D5A2C(&G); } break;
            case 42: { it_802D5A64(&G); } break;
            case 43: { ret = (u32) it_802D5A68(&G); } break;
            case 44: { itLucky_Logic44_EvtUnk(&G, &G2); } break;
            case 45: { { int i; for (i = 0; i < 7; i++) { HM((u32) it_803F8190[i].anim_id); HM(FI((const void*) it_803F8190[i].animated)); HM(FI((const void*) it_803F8190[i].physics_updated)); HM(FI((const void*) it_803F8190[i].collided)); } } } break;
            case 46: { { int i; for (i = 0; i < 5; i++) { HM((u32) it_803F8200[i].anim_id); HM(FI((const void*) it_803F8200[i].animated)); HM(FI((const void*) it_803F8200[i].physics_updated)); HM(FI((const void*) it_803F8200[i].collided)); } } } break;
            }
            pre_hash();
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
