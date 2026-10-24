/* TU: src/melee/it/kinds/itkirby_2F23.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu_it/mk_it_driver.py. Each line: op case ret state-hash trace-hash. */
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


#include "itkirby_2F23_gen.h"

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
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.kirby2f23.x1D0 = (q == 0 ? 0.5f : (q == 1 ? 5.0f : 20.0f)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.kirby2f23.x1D8 = (q == 0 ? 0.5f : (q == 1 ? 5.0f : 100.0f)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.kirby2f23.x1DC = (q == 0 ? 0.5f : (q == 1 ? 5.0f : 100.0f)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 5; IT.xDD4_itemVar.kirby2f23.x1C8 = (q == 0 ? 0.0f : (q == 1 ? 3.0f : (q == 2 ? -3.0f : (q == 3 ? 12.0f : -12.0f)))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 5; IT.xDD4_itemVar.kirby2f23.x1CC = (q == 0 ? 0.0f : (q == 1 ? 3.0f : (q == 2 ? -3.0f : (q == 3 ? 12.0f : -12.0f)))); }

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
    for (op = 0; op < 15; op++) {
        for (k = 0; k < 150; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { { double d = it_802F23AC(&G, &V1); HM64(d); } } break;
            case 1: { it_802F23EC_inline(&G); } break;
            case 2: { it_802F23EC(&G, &OWN, rw()); } break;
            case 3: { ret = (u32) itKirby_2F23_UnkMotion0_Anim(&G); } break;
            case 4: { itKirby_2F23_UnkMotion0_Phys(&G); } break;
            case 5: { ret = (u32) itKirby_2F23_UnkMotion0_Coll(&G); } break;
            case 6: { it_802F258C_scale(&G, &V1); } break;
            case 7: { it_802F258C_update(&G, &V1); } break;
            case 8: { it_802F258C(&G); } break;
            case 9: { it_802F2810(&G); } break;
            case 10: { ret = (u32) itKirby_2F23_UnkMotion1_Anim(&G); } break;
            case 11: { itKirby_2F23_UnkMotion1_Phys(&G); } break;
            case 12: { ret = (u32) itKirby_2F23_UnkMotion1_Coll(&G); } break;
            case 13: { it_802F289C(&G); } break;
            case 14: { { int i; for (i = 0; i < 2; i++) { HM((u32) it_803F9450[i].anim_id); HM(FI((const void*) it_803F9450[i].animated)); HM(FI((const void*) it_803F9450[i].physics_updated)); HM(FI((const void*) it_803F9450[i].collided)); } } } break;
            }
            pre_hash();
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
