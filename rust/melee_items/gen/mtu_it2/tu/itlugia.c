/* TU: src/melee/it/kinds/itlugia.c */
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


#include "itlugia_gen.h"

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
    if (rnd() % 4 != 0) { u32 q = rnd() % 6; IT.msid = (q == 0 ? 0 : (q == 1 ? 1 : (q == 2 ? 2 : (q == 3 ? 3 : (q == 4 ? 4 : 5))))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xDD4_itemVar.lugia.x60 = (q == 0 ? -1 : (q == 1 ? 0 : (q == 2 ? 1 : 3))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xDD4_itemVar.lugia.xE50.y = (q == 0 ? 0.0f : (q == 1 ? 0.5f : (q == 2 ? 1.0f : 1.5f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xDD4_itemVar.lugia.xE50.z = (q == 0 ? 0.0f : (q == 1 ? 0.5f : (q == 2 ? 1.0f : 1.5f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xDD4_itemVar.lugia.x88 = (q == 0 ? 0.0f : (q == 1 ? 0.5f : (q == 2 ? 1.0f : 1.5f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.lugia.xA4 = (q == 0 ? 0 : (q == 1 ? 90 : -45)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDAC_itcmd_var0 = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xD44_lifeTimer = (q == 0 ? 0.0f : (q == 1 ? 1.0f : (q == 2 ? -1.0f : 30.0f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.kind = (q == 0 ? It_Kind_Lugia_Aeroblast : (q == 1 ? It_Kind_Lugia_Aeroblast2 : (q == 2 ? It_Kind_Lugia_Aeroblast3 : 0))); }
    { itLugiaAttributes* A = (itLugiaAttributes*) SA; A->x34 = (rnd() & 1) ? 40.0f : 90.0f; A->x38 = (rnd() & 1) ? 10.0f : 30.0f; A->x3C = (rnd() & 1) ? 0.5f : 1.0f; { u32 q = rnd() % 3; A->x40 = q == 0 ? 1.0f : q == 1 ? 2.0f : 1e9f; }
      if (rnd() & 1) { IT.xDD4_itemVar.lugia.x64.x = 1.0f; IT.xDD4_itemVar.lugia.x64.y = 2.0f; IT.xDD4_itemVar.lugia.x64.z = 0.0f; } IT.xDD4_itemVar.lugia.x8C = IT.xDD4_itemVar.lugia.x64; }

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
    freg((const void*) my_sqrtf);
    freg((const void*) my_sqrtf_accurate);
    freg((const void*) it_2725_Logic17_Spawned);
    freg((const void*) it_802D14D0);
    freg((const void*) itLugia_Logic17_EvtUnk);
    freg((const void*) itLugia_UnkMotion1_Anim);
    freg((const void*) itLugia_UnkMotion1_Phys);
    freg((const void*) itLugia_UnkMotion1_Coll);
    freg((const void*) it_802D1580);
    freg((const void*) itLugia_UnkMotion2_Anim);
    freg((const void*) itLugia_UnkMotion2_Phys);
    freg((const void*) itLugia_UnkMotion2_Coll);
    freg((const void*) it_802D16D4);
    freg((const void*) itLugia_UnkMotion3_Anim);
    freg((const void*) itLugia_UnkMotion3_Phys);
    freg((const void*) itLugia_UnkMotion3_Coll);
    freg((const void*) it_802D1830);
    freg((const void*) itLugia_UnkMotion4_Anim);
    freg((const void*) itLugia_UnkMotion4_Phys);
    freg((const void*) itLugia_UnkMotion4_Coll);
    freg((const void*) it_802D1A44);
    freg((const void*) itLugia_UnkMotion5_Anim);
    freg((const void*) itLugia_UnkMotion5_Phys);
    freg((const void*) itLugia_UnkMotion5_Coll);
    freg((const void*) it_802D1BBC);
    freg((const void*) it_802D1D40);
    freg((const void*) it_802D1DB4);
    freg((const void*) it_802D1DD8);
    freg((const void*) it_802D1E64);
    freg((const void*) it_802D1E8C);
    freg((const void*) it_802D208C_inline);
    freg((const void*) it_802D1F64);
    freg((const void*) it_802D208C);
    freg((const void*) itLugia_Logic39_Spawned);
    freg((const void*) itLugia_Logic40_Spawned);
    freg((const void*) itLugia_Logic41_Spawned);
    freg((const void*) it_802D23D4);
    freg((const void*) it_802D23F4);
    freg((const void*) it_802D246C);
    freg((const void*) it_802D24A0);
    freg((const void*) it_802D24FC);
    for (op = 0; op < 43; op++) {
        for (k = 0; k < 150; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { ret = B(my_sqrtf(rw())); } break;
            case 1: { ret = B(my_sqrtf_accurate(rw())); } break;
            case 2: { it_2725_Logic17_Spawned(&G); } break;
            case 3: { it_802D14D0(); } break;
            case 4: { itLugia_Logic17_EvtUnk(&G, &G2); } break;
            case 5: { ret = (u32) itLugia_UnkMotion1_Anim(&G); } break;
            case 6: { itLugia_UnkMotion1_Phys(&G); } break;
            case 7: { ret = (u32) itLugia_UnkMotion1_Coll(&G); } break;
            case 8: { it_802D1580(&G); } break;
            case 9: { ret = (u32) itLugia_UnkMotion2_Anim(&G); } break;
            case 10: { itLugia_UnkMotion2_Phys(&G); } break;
            case 11: { ret = (u32) itLugia_UnkMotion2_Coll(&G); } break;
            case 12: { it_802D16D4(&G); } break;
            case 13: { ret = (u32) itLugia_UnkMotion3_Anim(&G); } break;
            case 14: { itLugia_UnkMotion3_Phys(&G); } break;
            case 15: { ret = (u32) itLugia_UnkMotion3_Coll(&G); } break;
            case 16: { it_802D1830(&G); } break;
            case 17: { ret = (u32) itLugia_UnkMotion4_Anim(&G); } break;
            case 18: { itLugia_UnkMotion4_Phys(&G); } break;
            case 19: { ret = (u32) itLugia_UnkMotion4_Coll(&G); } break;
            case 20: { it_802D1A44(&G); } break;
            case 21: { ret = (u32) itLugia_UnkMotion5_Anim(&G); } break;
            case 22: { itLugia_UnkMotion5_Phys(&G); } break;
            case 23: { ret = (u32) itLugia_UnkMotion5_Coll(&G); } break;
            case 24: { it_802D1BBC(&G); } break;
            case 25: { it_802D1D40(&G); } break;
            case 26: { ret = (u32) it_802D1DB4(&G); } break;
            case 27: { it_802D1DD8(&G); } break;
            case 28: { ret = (u32) it_802D1E64(&G); } break;
            case 29: { ret = PI(it_802D1E8C(&G, (ItemKind) fv(), rw())); } break;
            case 30: { ret = B(it_802D208C_inline(rw(), rw(), rw())); } break;
            case 31: { { Vec3 v = it_802D1F64(&G, rw()); ret = B(v.x); HM(B(v.y)); HM(B(v.z)); } } break;
            case 32: { it_802D208C(&G); } break;
            case 33: { itLugia_Logic39_Spawned(&G); } break;
            case 34: { itLugia_Logic40_Spawned(&G); } break;
            case 35: { itLugia_Logic41_Spawned(&G); } break;
            case 36: { it_802D23D4(&G, &G2); } break;
            case 37: { it_802D23F4(&G); } break;
            case 38: { ret = (u32) it_802D246C(&G); } break;
            case 39: { it_802D24A0(&G); } break;
            case 40: { ret = (u32) it_802D24FC(&G); } break;
            case 41: { { int i; for (i = 0; i < 6; i++) { HM((u32) it_803F7EE8[i].anim_id); HM(FI((const void*) it_803F7EE8[i].animated)); HM(FI((const void*) it_803F7EE8[i].physics_updated)); HM(FI((const void*) it_803F7EE8[i].collided)); } } } break;
            case 42: { { int i; for (i = 0; i < 1; i++) { HM((u32) it_803F7F48[i].anim_id); HM(FI((const void*) it_803F7F48[i].animated)); HM(FI((const void*) it_803F7F48[i].physics_updated)); HM(FI((const void*) it_803F7F48[i].collided)); } } } break;
            }
            pre_hash();
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
