/* TU: src/melee/it/kinds/itlinkbow.c */
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


#include "itlinkbow_gen.h"

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
    if (rnd() % 4 != 0) { u32 q = rnd() % 5; IT.kind = (q == 0 ? It_Kind_Link_Bow : (q == 1 ? It_Kind_CLink_Bow : (q == 2 ? It_Kind_Kirby_CLinkBow : (q == 3 ? It_Kind_Kirby_LinkBow : 0)))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 7; IT.msid = (q == 0 ? 0 : (q == 1 ? 1 : (q == 2 ? 2 : (q == 3 ? 3 : (q == 4 ? 4 : (q == 5 ? 5 : 6)))))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.x5CC_currentAnimFrame = (q == 0 ? 0.0f : (q == 1 ? 24.0f : (q == 2 ? 30.0f : 5.0f))); }
    { int q = rnd() % 4; IT.xDD4_itemVar.linkbow.x4 = q == 0 ? 0 : q == 1 ? IT.owner : q == 2 ? &OWN : &G2; if (rnd() & 1) IT.owner = IT.xDD4_itemVar.linkbow.x4; }

}

static void pre_hash(void)
{
    { u32 id = PI(IT.xDD4_itemVar.linkbow.x4); unsigned char* b = (unsigned char*) &IT.xDD4_itemVar.linkbow.x4; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT2.xDD4_itemVar.linkbow.x4); unsigned char* b = (unsigned char*) &IT2.xDD4_itemVar.linkbow.x4; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }

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
    freg((const void*) it_802AF1A4);
    freg((const void*) itLinkBow_Logic100_Destroyed);
    freg((const void*) it_802AF304);
    freg((const void*) it_802AF32C);
    freg((const void*) itLinkBow_Logic100_PickedUp);
    freg((const void*) itLinkbow_UnkMotion5_Anim);
    freg((const void*) itLinkbow_UnkMotion5_Phys);
    freg((const void*) itLinkbow_UnkMotion5_Coll);
    freg((const void*) itLinkbow_UnkMotion6_Anim);
    freg((const void*) itLinkbow_UnkMotion6_Phys);
    freg((const void*) itLinkbow_UnkMotion6_Coll);
    freg((const void*) itLinkBow_Logic100_EvtUnk);
    for (op = 0; op < 13; op++) {
        for (k = 0; k < 150; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { ret = PI(it_802AF1A4(rw(), &G, &V1, (Fighter_Part) fv(), (int) fv())); } break;
            case 1: { itLinkBow_Logic100_Destroyed(&G); } break;
            case 2: { it_802AF304(&G); } break;
            case 3: { it_802AF32C(&G); } break;
            case 4: { itLinkBow_Logic100_PickedUp(&G); } break;
            case 5: { ret = (u32) itLinkbow_UnkMotion5_Anim(&G); } break;
            case 6: { itLinkbow_UnkMotion5_Phys(&G); } break;
            case 7: { ret = (u32) itLinkbow_UnkMotion5_Coll(&G); } break;
            case 8: { ret = (u32) itLinkbow_UnkMotion6_Anim(&G); } break;
            case 9: { itLinkbow_UnkMotion6_Phys(&G); } break;
            case 10: { ret = (u32) itLinkbow_UnkMotion6_Coll(&G); } break;
            case 11: { itLinkBow_Logic100_EvtUnk(&G, &G2); } break;
            case 12: { { int i; for (i = 0; i < 7; i++) { HM((u32) it_803F6E98[i].anim_id); HM(FI((const void*) it_803F6E98[i].animated)); HM(FI((const void*) it_803F6E98[i].physics_updated)); HM(FI((const void*) it_803F6E98[i].collided)); } } } break;
            }
            pre_hash();
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
