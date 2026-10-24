/* TU: src/melee/it/kinds/itfflowerflame.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_it_driver.py. Each line: op case ret state-hash trace-hash. */
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


#include "itfflowerflame_gen.h"

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
    for (op = 0; op < 28; op++) {
        for (k = 0; k < 150; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { ret = PI(it_8029A748(&G, &V1, (Fighter_Part) fv(), rw())); } break;
            case 1: { it_8029A89C(&G, rw()); } break;
            case 2: { it_8029A8F4(&G, &V1); } break;
            case 3: { itFFlowerFlame_Logic41_Spawned(&G); } break;
            case 4: { it_8029AA1C(&G); } break;
            case 5: { ret = (u32) itFflowerflame_UnkMotion0_Anim(&G); } break;
            case 6: { itFflowerflame_UnkMotion0_Phys(&G); } break;
            case 7: { ret = (u32) itFflowerflame_UnkMotion0_Coll(&G); } break;
            case 8: { it_8029AB90(&G); } break;
            case 9: { ret = (u32) itFflowerflame_UnkMotion1_Anim(&G); } break;
            case 10: { itFflowerflame_UnkMotion1_Phys(&G); } break;
            case 11: { ret = (u32) itFflowerflame_UnkMotion1_Coll(&G); } break;
            case 12: { itFFlowerFlame_Logic41_PickedUp(&G); } break;
            case 13: { ret = (u32) itFflowerflame_UnkMotion2_Anim(&G); } break;
            case 14: { itFFlowerFlame_Logic41_Dropped(&G); } break;
            case 15: { ret = (u32) itFflowerflame_UnkMotion3_Anim(&G); } break;
            case 16: { itFflowerflame_UnkMotion3_Phys(&G); } break;
            case 17: { ret = (u32) itFflowerflame_UnkMotion3_Coll(&G); } break;
            case 18: { itFFlowerFlame_Logic41_EnteredAir(&G); } break;
            case 19: { ret = (u32) itFflowerflame_UnkMotion4_Anim(&G); } break;
            case 20: { itFflowerflame_UnkMotion4_Phys(&G); } break;
            case 21: { ret = (u32) itFflowerflame_UnkMotion4_Coll(&G); } break;
            case 22: { itFFlowerFlame_Logic41_EvtUnk(&G, &G2); } break;
            case 23: { it_8029A89C(0, rw()); } break;
            case 24: { it_8029A8F4(0, &V1); } break;
            case 25: { G.user_data = 0; it_8029A89C(&G, rw()); } break;
            case 26: { G.user_data = 0; it_8029A8F4(&G, &V1); } break;
            case 27: { { int i; for (i = 0; i < 5; i++) { HM((u32) it_803F66C8[i].anim_id); HM(FI((const void*) it_803F66C8[i].animated)); HM(FI((const void*) it_803F66C8[i].physics_updated)); HM(FI((const void*) it_803F66C8[i].collided)); } } } break;
            }
            pre_hash();
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
