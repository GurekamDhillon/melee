/* TU: src/melee/it/kinds/itflipper.c */
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


#include "itflipper_gen.h"

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
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xC38 = (q == 0 ? 20 : 19); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 8; IT.xDD4_itemVar.flipper.xDD4_flightTimer = (q == 0 ? -1 : (q == 1 ? 0 : (q == 2 ? 1 : (q == 3 ? 5 : (q == 4 ? 6 : (q == 5 ? 10 : (q == 6 ? 11 : 12))))))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDD4_itemVar.flipper.xDD8_isSettled = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDD4_itemVar.flipper.xDE8_isStageFixed = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.flipper.xDDC_hitboxTimer = (q == 0 ? 0 : (q == 1 ? 1 : 2)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 8; IT.xDD4_itemVar.flipper.xDE4_spinVel = (q == 0 ? 0.0f : (q == 1 ? 0.01f : (q == 2 ? -0.01f : (q == 3 ? 0.5f : (q == 4 ? -0.5f : (q == 5 ? 3.5f : (q == 6 ? -3.5f : 7.0f))))))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xD44_lifeTimer = (q == 0 ? 0.5f : (q == 1 ? 1.0f : (q == 2 ? 2.0f : 30.0f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; ICD.x34 = (q == 0 ? 0 : (q == 1 ? 1 : (q == 2 ? 2 : 30))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; SA[8] = B((q == 0 ? 0.0f : (q == 1 ? 0.1f : (q == 2 ? 0.5f : 5.0f)))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; SA[9] = B((q == 0 ? 0.5f : (q == 1 ? 3.0f : 10.0f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; SA[7] = B((q == 0 ? 0.0f : (q == 1 ? 0.3f : 2.0f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; SA[6] = B((q == 0 ? 0.5f : (q == 1 ? 1.0f : 2.0f))); }
    IT.xDD4_itemVar.flipper.xDEC_stageAnchor = (rnd() & 1) ? &JO[1] : 0; IT2.xDD4_itemVar.flipper.xDEC_stageAnchor = 0;
    IT.xCF4_fighterGObjUnk = (rnd() & 1) ? &OWN : 0; IT.xCEC_fighterGObj = (rnd() & 1) ? &OWN : 0;

}

static void pre_hash(void)
{
    {
        Item* its[2]; int k2, i2;
        its[0] = &IT; its[1] = &IT2;
        for (k2 = 0; k2 < 2; k2++) {
            u32 id = PI(its[k2]->xDD4_itemVar.flipper.xDEC_stageAnchor);
            unsigned char* b = (unsigned char*) &its[k2]->xDD4_itemVar.flipper.xDEC_stageAnchor;
            for (i2 = 0; i2 < 8; i2++) b[i2] = 0;
            *(u32*) b = id;
        }
    }

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
    for (op = 0; op < 46; op++) {
        for (k = 0; k < 150; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { ret = PI(itFlipper_Spawn(&JO[1])); } break;
            case 1: { itFlipper_Spawned(&G); } break;
            case 2: { itFlipper_UpdateSpin(&G); } break;
            case 3: { itFlipper_AddSpinImpulse(&G, &V1, rw()); } break;
            case 4: { ret = B(itFlipper_SpinSpeedFromFighter(&G, &G2, &V1, &V1)); } break;
            case 5: { itFlipper_SpinFromFighter_inline(&G, &V1, &V1, rw()); } break;
            case 6: { itFlipper_SpinFromFighter(&G); } break;
            case 7: { itFlipper_Repel_inline(&G, (s32) fv(), &V1, &V1); } break;
            case 8: { itFlipper_Repel(&G, (s32) fv(), &V1); } break;
            case 9: { itFlipper_EnterResting(&G); } break;
            case 10: { ret = (u32) itFlipper_Resting_Anim(&G); } break;
            case 11: { itFlipper_Resting_Phys(&G); } break;
            case 12: { ret = (u32) itFlipper_Resting_Coll(&G); } break;
            case 13: { itFlipper_EnterFalling(&G); } break;
            case 14: { ret = (u32) itFlipper_Falling_Anim(&G); } break;
            case 15: { itFlipper_Falling_Phys(&G); } break;
            case 16: { ret = (u32) itFlipper_Falling_Coll(&G); } break;
            case 17: { itFlipper_PickedUp(&G); } break;
            case 18: { ret = (u32) itFlipper_Held_Anim(&G); } break;
            case 19: { itFlipper_Held_Phys(&G); } break;
            case 20: { itFlipper_Dropped(&G); } break;
            case 21: { itFlipper_Thrown(&G); } break;
            case 22: { ret = (u32) itFlipper_Inflight_Anim(&G); } break;
            case 23: { itFlipper_Inflight_Phys(&G); } break;
            case 24: { ret = (u32) itFlipper_Inflight_Coll(&G); } break;
            case 25: { itFlipper_Settle(&G); } break;
            case 26: { itFlipper_EnterActive(&G); } break;
            case 27: { itFlipper_RefreshHitboxes(&G); } break;
            case 28: { ret = (u32) itFlipper_UpdateActive(&G); } break;
            case 29: { ret = (u32) itFlipper_Active_Anim(&G); } break;
            case 30: { ret = (u32) itFlipper_Spinning_Anim(&G); } break;
            case 31: { itFlipper_Spinning_Phys(&G); } break;
            case 32: { ret = (u32) itFlipper_Spinning_Coll(&G); } break;
            case 33: { ret = (u32) itFlipper_DmgDealt(&G); } break;
            case 34: { ret = (u32) itFlipper_Clanked(&G); } break;
            case 35: { ret = (u32) itFlipper_HitShield(&G); } break;
            case 36: { ret = (u32) itFlipper_Reflected(&G); } break;
            case 37: { ret = (u32) itFlipper_ShieldBounced(&G); } break;
            case 38: { itFlipper_SpinFromFighterRecv(&G, &V1, &V1, rw()); } break;
            case 39: { ret = (u32) itFlipper_DmgReceived(&G); } break;
            case 40: { itFlipper_EnteredAir(&G); } break;
            case 41: { ret = (u32) itFlipper_Airborne_Anim(&G); } break;
            case 42: { itFlipper_Airborne_Phys(&G); } break;
            case 43: { ret = (u32) itFlipper_Airborne_Coll(&G); } break;
            case 44: { itFlipper_EvtUnk(&G, &G2); } break;
            case 45: { { int i; for (i = 0; i < 7; i++) { HM((u32) ItemStateTable_Flipper[i].anim_id); HM(FI((const void*) ItemStateTable_Flipper[i].animated)); HM(FI((const void*) ItemStateTable_Flipper[i].physics_updated)); HM(FI((const void*) ItemStateTable_Flipper[i].collided)); } } } break;
            }
            pre_hash();
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
