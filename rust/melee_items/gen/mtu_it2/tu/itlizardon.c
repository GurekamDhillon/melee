/* TU: src/melee/it/kinds/itlizardon.c */
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


#include "itlizardon_gen.h"

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
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.msid = (q == 0 ? 0 : (q == 1 ? 1 : (q == 2 ? 2 : 3))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xDD4_itemVar.lizardon.x60 = (q == 0 ? -1 : (q == 1 ? 0 : (q == 2 ? 1 : 3))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDD4_itemVar.lizardon.x64 = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDD4_itemVar.lizardon.x6C = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xDD4_itemVar.lizardon.x68 = (q == 0 ? 0.0f : (q == 1 ? 1.0f : (q == 2 ? 2.0f : 5.0f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xDD4_itemVar.lizardon.x70 = (q == 0 ? 0 : (q == 1 ? 1 : (q == 2 ? -1 : 7))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.lizardon.x74[0] = (q == 0 ? 0 : (q == 1 ? 1 : 5)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.lizardon.x74[1] = (q == 0 ? 0 : (q == 1 ? 1 : 5)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.lizardon.x74[2] = (q == 0 ? 0 : (q == 1 ? 1 : 5)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.lizardon.x74[3] = (q == 0 ? 0 : (q == 1 ? 1 : 5)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDAC_itcmd_var0 = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDB0_itcmd_var1 = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xD44_lifeTimer = (q == 0 ? 0.0f : (q == 1 ? 1.0f : (q == 2 ? -1.0f : 30.0f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.ground_or_air = (q == 0 ? GA_Ground : GA_Air); }

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
    freg((const void*) it_802CB8AC);
    freg((const void*) it_802CB93C);
    freg((const void*) it_802CB940);
    freg((const void*) it_802CB960);
    freg((const void*) it_802CB994);
    freg((const void*) itLizardon_UnkMotion1_Anim);
    freg((const void*) itLizardon_UnkMotion1_Phys);
    freg((const void*) itLizardon_UnkMotion1_Coll);
    freg((const void*) it_802CBAA8_inline);
    freg((const void*) it_802CBAA8);
    freg((const void*) itLizardon_UnkMotion2_Anim);
    freg((const void*) itLizardon_UnkMotion2_Phys);
    freg((const void*) itLizardon_UnkMotion2_Coll);
    freg((const void*) it_802CBD24_inline);
    freg((const void*) it_802CBD24);
    freg((const void*) it_802CBFE4);
    freg((const void*) itLizardon_UnkMotion3_Anim);
    freg((const void*) itLizardon_UnkMotion3_Phys);
    freg((const void*) itLizardon_UnkMotion3_Coll);
    freg((const void*) it_802CC0EC);
    freg((const void*) it_802CC160);
    freg((const void*) it_802CC184);
    freg((const void*) it_802CC1A4);
    freg((const void*) it_802CC1CC);
    freg((const void*) itLizardon_Logic34_Spawned);
    freg((const void*) itLizardon_Logic35_Spawned);
    freg((const void*) itLizardon_Logic36_Spawned);
    freg((const void*) itLizardon_Logic37_Spawned);
    freg((const void*) itLizardon_Logic37_EvtUnk);
    freg((const void*) itLizardon_Logic37_Reflected);
    freg((const void*) itLizardon_Logic37_HitShield);
    freg((const void*) itLizardon_Logic37_Absorbed);
    freg((const void*) it_802CC5D4);
    freg((const void*) it_802CC650);
    freg((const void*) it_802CC684);
    freg((const void*) it_802CC6C4);
    for (op = 0; op < 38; op++) {
        for (k = 0; k < 150; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { it_802CB8AC(&G); } break;
            case 1: { it_802CB93C(&G); } break;
            case 2: { it_802CB940(&G, &G2); } break;
            case 3: { it_802CB960(&G); } break;
            case 4: { it_802CB994(&G); } break;
            case 5: { ret = (u32) itLizardon_UnkMotion1_Anim(&G); } break;
            case 6: { itLizardon_UnkMotion1_Phys(&G); } break;
            case 7: { ret = (u32) itLizardon_UnkMotion1_Coll(&G); } break;
            case 8: { it_802CBAA8_inline(&G); } break;
            case 9: { it_802CBAA8(&G); } break;
            case 10: { ret = (u32) itLizardon_UnkMotion2_Anim(&G); } break;
            case 11: { itLizardon_UnkMotion2_Phys(&G); } break;
            case 12: { ret = (u32) itLizardon_UnkMotion2_Coll(&G); } break;
            case 13: { it_802CBD24_inline(&G); } break;
            case 14: { it_802CBD24(&G); } break;
            case 15: { it_802CBFE4(&G); } break;
            case 16: { ret = (u32) itLizardon_UnkMotion3_Anim(&G); } break;
            case 17: { itLizardon_UnkMotion3_Phys(&G); } break;
            case 18: { ret = (u32) itLizardon_UnkMotion3_Coll(&G); } break;
            case 19: { it_802CC0EC(&G); } break;
            case 20: { ret = (u32) it_802CC160(&G); } break;
            case 21: { it_802CC184(&G); } break;
            case 22: { ret = (u32) it_802CC1A4(&G); } break;
            case 23: { it_802CC1CC(&G, (ItemKind) fv()); } break;
            case 24: { itLizardon_Logic34_Spawned(&G); } break;
            case 25: { itLizardon_Logic35_Spawned(&G); } break;
            case 26: { itLizardon_Logic36_Spawned(&G); } break;
            case 27: { itLizardon_Logic37_Spawned(&G); } break;
            case 28: { itLizardon_Logic37_EvtUnk(&G, &G2); } break;
            case 29: { ret = (u32) itLizardon_Logic37_Reflected(&G); } break;
            case 30: { ret = (u32) itLizardon_Logic37_HitShield(&G); } break;
            case 31: { ret = (u32) itLizardon_Logic37_Absorbed(&G); } break;
            case 32: { it_802CC5D4(&G); } break;
            case 33: { ret = (u32) it_802CC650(&G); } break;
            case 34: { it_802CC684(&G); } break;
            case 35: { ret = (u32) it_802CC6C4(&G); } break;
            case 36: { { int i; for (i = 0; i < 4; i++) { HM((u32) it_803F7BC0[i].anim_id); HM(FI((const void*) it_803F7BC0[i].animated)); HM(FI((const void*) it_803F7BC0[i].physics_updated)); HM(FI((const void*) it_803F7BC0[i].collided)); } } } break;
            case 37: { { int i; for (i = 0; i < 1; i++) { HM((u32) it_803F7C00[i].anim_id); HM(FI((const void*) it_803F7C00[i].animated)); HM(FI((const void*) it_803F7C00[i].physics_updated)); HM(FI((const void*) it_803F7C00[i].collided)); } } } break;
            }
            pre_hash();
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
