/* TU: src/melee/it/kinds/itkusudama.c */
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


#include "itkusudama_gen.h"

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
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDD4_itemVar.kusudama.x0 = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xDD4_itemVar.kusudama.x4 = (q == 0 ? -1 : (q == 1 ? 0 : (q == 2 ? 1 : 5))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDB4_itcmd_var2 = (q == 0 ? 0 : (q == 1 ? 1 : 2)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDB0_itcmd_var1 = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDAC_itcmd_var0 = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.facing_dir = (q == 0 ? 1.0f : -1.0f); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xC9C = (q == 0 ? 0 : (q == 1 ? 5 : 50)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 5; IT.x378_itemColl.env_flags = (q == 0 ? 0 : (q == 1 ? 0x2000 : (q == 2 ? 0x4000 : (q == 3 ? 0x6000 : 1)))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.x40_vel.y = (q == 0 ? 0.0f : (q == 1 ? -3.0f : (q == 2 ? 3.0f : -0.5f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 6; JO[0].rotate.x = (q == 0 ? 0.0f : (q == 1 ? 1.5f : (q == 2 ? -1.5f : (q == 3 ? 3.0f : (q == 4 ? -3.0f : 0.1f))))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 6; JO[0].rotate.y = (q == 0 ? 0.0f : (q == 1 ? 1.6f : (q == 2 ? -1.6f : (q == 3 ? 4.7f : (q == 4 ? -4.7f : 2.5f))))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IA.x10_fall_speed = (q == 0 ? 0.1f : 0.5f); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IA.x14_fall_speed_max = (q == 0 ? 0.3f : 2.0f); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; SA[0] = (q == 0 ? 0 : (q == 1 ? 1 : 5)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; SA[1] = (q == 0 ? 0 : (q == 1 ? 1 : 5)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; SA[2] = (q == 0 ? 0 : (q == 1 ? 1 : 5)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; SA[3] = (q == 0 ? 0 : (q == 1 ? 1 : 5)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; SA[4] = (q == 0 ? 34 : (q == 1 ? 18 : (q == 2 ? 5 : 0))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; SA[5] = (q == 0 ? 0 : (q == 1 ? 1 : (q == 2 ? 3 : 15))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 1; SA[16] = 0; }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; SA[6] = B((q == 0 ? 0.0f : (q == 1 ? 3.0f : -3.0f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; SA[7] = B((q == 0 ? 0.0f : (q == 1 ? 30.0f : -30.0f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; SA[10] = B((q == 0 ? 0.0f : (q == 1 ? 0.1f : (q == 2 ? 0.5f : 3.0f)))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; SA[11] = B((q == 0 ? 0.0f : (q == 1 ? 0.5f : 3.0f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; SA[8] = B((q == 0 ? 0.0f : (q == 1 ? 5.0f : 50.0f))); }
    if ((int) SA[5] > 15) SA[5] &= 15;

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
    for (op = 0; op < 47; op++) {
        for (k = 0; k < 150; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { ret = PI(it_802896CC(&V1)); } break;
            case 1: { itKusudama_Logic4_Spawned(&G); } break;
            case 2: { it_802897C8(&G, rw()); } break;
            case 3: { it_80289910(&G, rw()); } break;
            case 4: { it_80289A00(&G); } break;
            case 5: { it_80289B50(&G, (s32) fv()); } break;
            case 6: { it_80289BE8(&G, (s32) (rnd() % 4), (s32) (rnd() % 4), (s32) (rnd() % 4)); } break;
            case 7: { ret = (u32) it_8028A114(&G, (s32) fv(), (s32) fv(), (s32) fv(), (s32) fv()); } break;
            case 8: { it_8028A190(&G); } break;
            case 9: { ret = (u32) itKusudama_UnkMotion0_Anim(&G); } break;
            case 10: { itKusudama_UnkMotion0_Phys(&G); } break;
            case 11: { ret = (u32) itKusudama_UnkMotion0_Coll(&G); } break;
            case 12: { ret = (u32) itKusudama_UnkMotion1_Anim(&G); } break;
            case 13: { itKusudama_UnkMotion1_Phys(&G); } break;
            case 14: { ret = (u32) itKusudama_UnkMotion1_Coll(&G); } break;
            case 15: { it_8028A3CC(&G); } break;
            case 16: { ret = (u32) itKusudama_UnkMotion2_Anim(&G); } break;
            case 17: { itKusudama_UnkMotion2_Phys(&G); } break;
            case 18: { ret = (u32) itKusudama_UnkMotion2_Coll(&G); } break;
            case 19: { it_8028A544(&G); } break;
            case 20: { ret = (u32) itKusudama_UnkMotion3_Anim(&G); } break;
            case 21: { itKusudama_UnkMotion3_Phys(&G); } break;
            case 22: { ret = (u32) itKusudama_UnkMotion3_Coll(&G); } break;
            case 23: { itKusudama_Logic4_PickedUp(&G); } break;
            case 24: { ret = (u32) itKusudama_UnkMotion4_Anim(&G); } break;
            case 25: { itKusudama_UnkMotion4_Phys(&G); } break;
            case 26: { itKusudama_Logic4_Thrown(&G); } break;
            case 27: { ret = (u32) itKusudama_UnkMotion6_Anim(&G); } break;
            case 28: { itKusudama_UnkMotion6_Phys(&G); } break;
            case 29: { ret = (u32) itKusudama_UnkMotion5_Coll(&G); } break;
            case 30: { itKusudama_Logic4_Dropped(&G); } break;
            case 31: { ret = (u32) itKusudama_UnkMotion6_Coll(&G); } break;
            case 32: { it_8028AC74(&G); } break;
            case 33: { ret = (u32) itKusudama_UnkMotion7_Anim(&G); } break;
            case 34: { itKusudama_UnkMotion7_Phys(&G); } break;
            case 35: { ret = (u32) itKusudama_UnkMotion7_Coll(&G); } break;
            case 36: { it_8028AD44(&G); } break;
            case 37: { ret = (u32) itKusudama_UnkMotion8_Anim(&G); } break;
            case 38: { itKusudama_UnkMotion8_Phys(&G); } break;
            case 39: { ret = (u32) itKusudama_UnkMotion8_Coll(&G); } break;
            case 40: { ret = (u32) itKusudama_Logic4_DmgDealt(&G); } break;
            case 41: { ret = (u32) itKusudama_Logic4_Clanked(&G); } break;
            case 42: { ret = (u32) itKusudama_Logic4_HitShield(&G); } break;
            case 43: { ret = (u32) itKusudama_Logic4_Reflected(&G); } break;
            case 44: { ret = (u32) itKusudama_Logic4_DmgReceived(&G); } break;
            case 45: { itKusudama_Logic4_EvtUnk(&G, &G2); } break;
            case 46: { { int i; for (i = 0; i < 9; i++) { HM((u32) it_803F59F8[i].anim_id); HM(FI((const void*) it_803F59F8[i].animated)); HM(FI((const void*) it_803F59F8[i].physics_updated)); HM(FI((const void*) it_803F59F8[i].collided)); } } } break;
            }
            pre_hash();
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
