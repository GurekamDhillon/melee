/* TU: src/melee/it/kinds/itkoopaflame.c */
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


#include "itkoopaflame_gen.h"

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
    IT.xC4_article_data = &ART; IT2.xC4_article_data = &ART; ART.x4_specialAttributes = (union ItemSpecialAttributes*) SA; ART.x0_common_attr = &IA;
    IT.xCC_item_attr = &IA; IT2.xCC_item_attr = &IA;
    IT.xBBC_dynamicBoneTable = &DBT; IT2.xBBC_dynamicBoneTable = &DBT;
    for (i = 0; i < 100; i++) DBT.bones[i] = &JO[i & 3];
    IT.owner = (rnd() & 1) ? &OWN : 0; IT2.owner = 0;
    it_804D6D28 = &ICD;
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.kind = (q == 0 ? It_Kind_Koopa_Flame : (q == 1 ? It_Kind_Kirby_KoopaFlame : 5)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT2.kind = (q == 0 ? It_Kind_Koopa_Flame : (q == 1 ? It_Kind_Kirby_KoopaFlame : 5)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 9; IT.x378_itemColl.env_flags = (q == 0 ? 0 : (q == 1 ? 1 : (q == 2 ? 0x40 : (q == 3 ? 0x2000 : (q == 4 ? 0x8000 : (q == 5 ? 0x8001 : (q == 6 ? 0x2041 : (q == 7 ? 0xA041 : 0x10000)))))))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.x5D4_hitboxes[0].hit.state = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.koopaflame.x34_base_scale = (q == 0 ? 0.0f : (q == 1 ? 1.0f : 2.0f)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDD4_itemVar.koopaflame.x44_spawned = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 5; IT.xDD4_itemVar.koopaflame.x40_frame_counter = (q == 0 ? 0 : (q == 1 ? 10 : (q == 2 ? 19 : (q == 3 ? 20 : 21)))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 7; IT.xDD4_itemVar.koopaflame.x24_angle = (q == 0 ? 0.0f : (q == 1 ? 1.0f : (q == 2 ? -1.0f : (q == 3 ? 3.0f : (q == 4 ? -3.0f : (q == 5 ? 4.0f : 7.0f)))))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.facing_dir = (q == 0 ? 1.0f : -1.0f); }
    for (i = 0; i < 6; i++) SA[i] = B(rg());
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; SA[1] = B((q == 0 ? 20.0f : (q == 1 ? 0.5f : 10.0f))); }
    if (!(IT.xDD4_itemVar.koopaflame.x24_angle > -50.0f && IT.xDD4_itemVar.koopaflame.x24_angle < 50.0f)) IT.xDD4_itemVar.koopaflame.x24_angle = 0.5f;

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
            case 0: { itKoopaFlame_Update_Direction(&G, (int) (rnd() % 16)); } break;
            case 1: { itKoopaFlame_Update_Angle(&G, (int) (rnd() % 16)); } break;
            case 2: { ret = PI(itKoopaFlame_Spawn(&OWN, &V1, rw(), fv(), (s32) fv(), (s32) (fv() % 9), (s32) (fv() % 9), (s32) (fv() % 3 ? 5 : 6))); } break;
            case 3: { itKoopaFlame_Setup(&G, &G2, (int) fv()); } break;
            case 4: { ret = (u32) itKoopaFlame_UnkMotion0_Anim(&G); } break;
            case 5: { itKoopaFlame_UnkMotion0_Phys(&G); } break;
            case 6: { ret = (u32) itKoopaFlame_UnkMotion0_Coll(&G); } break;
            case 7: { ret = (u32) itKoopaFlame_Logic111_DmgDealt(&G); } break;
            case 8: { ret = (u32) itKoopaFlame_Logic111_Reflected(&G); } break;
            case 9: { ret = (u32) itKoopaFlame_Logic111_Clanked(&G); } break;
            case 10: { ret = (u32) itKoopaFlame_Logic111_Absorbed(&G); } break;
            case 11: { ret = (u32) itKoopaFlame_Logic111_ShieldBounced(&G); } break;
            case 12: { ret = (u32) itKoopaFlame_Logic111_HitShield(&G); } break;
            case 13: { itKoopaFlame_Logic111_EvtUnk(&G, &G2); } break;
            case 14: { { int i; for (i = 0; i < 1; i++) { HM((u32) ItemStateTable_KoopaFlame[i].anim_id); HM(FI((const void*) ItemStateTable_KoopaFlame[i].animated)); HM(FI((const void*) ItemStateTable_KoopaFlame[i].physics_updated)); HM(FI((const void*) ItemStateTable_KoopaFlame[i].collided)); } } } break;
            }
            pre_hash();
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
