/* TU: src/melee/it/kinds/itlinkboomerang.c */
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

static ItemLogicTable LT;
static ItemStateArray ISA;
static void lt_thrown(HSD_GObj* g) { T(0xBEEF); T(PI(g)); }

#include "itlinkboomerang_gen.h"

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
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDAC_itcmd_var0 = (q == 0 ? 0 : (q == 1 ? 1 : 2)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDB0_itcmd_var1 = (q == 0 ? 0 : (q == 1 ? 1 : 2)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDB4_itcmd_var2 = (q == 0 ? 0 : (q == 1 ? 1 : 2)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDD4_itemVar.linkboomerang.xDE8 = (q == 0 ? 0 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDD4_itemVar.linkboomerang.xDEC = (q == 0 ? 6 : 1); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDD4_itemVar.linkboomerang.xDE4 = (q == 0 ? 0 : 3); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.linkboomerang.xF70 = (q == 0 ? 0.0f : (q == 1 ? 1.0f : 5.0f)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xDD4_itemVar.linkboomerang.xF8C = (q == 0 ? 0.0f : (q == 1 ? 1.0f : (q == 2 ? -2.0f : 5.0f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.linkboomerang.xF80 = (q == 0 ? 0.0f : (q == 1 ? 1.0f : 3.0f)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 3; IT.xDD4_itemVar.linkboomerang.xF84 = (q == 0 ? 0.1f : (q == 1 ? 0.5f : 1.0f)); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 7; IT.xDD4_itemVar.linkboomerang.xF74 = (q == 0 ? 0.0f : (q == 1 ? 1.0f : (q == 2 ? 3.0f : (q == 3 ? -3.0f : (q == 4 ? 7.0f : (q == 5 ? -7.0f : 0.5f)))))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 4; IT.xD44_lifeTimer = (q == 0 ? 0.0f : (q == 1 ? 1.0f : (q == 2 ? -1.0f : 30.0f))); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.facing_dir = (q == 0 ? 1.0f : -1.0f); }
    if (rnd() % 4 != 0) { u32 q = rnd() % 5; IT.x378_itemColl.env_flags = (q == 0 ? 0 : (q == 1 ? 0xFC0 : (q == 2 ? 0x3F : (q == 3 ? 0x6000 : 0x18000)))); }
    zero(&LT, sizeof LT); zero(&ISA, sizeof ISA); LT.thrown = lt_thrown; IT.xB8_itemLogicTable = &LT; IT2.xB8_itemLogicTable = &LT; ART.xC_itemStates = &ISA;
    IT.xDD4_itemVar.linkboomerang.xDDC[0] = rnd() % 16; IT.xDD4_itemVar.linkboomerang.xDDC[1] = rnd() % 16; IT.xDD4_itemVar.linkboomerang.xDD8 = rnd() % 16;
    IT2.xDD4_itemVar.linkboomerang.xDDC[0] = rnd() % 16; IT2.xDD4_itemVar.linkboomerang.xDDC[1] = rnd() % 16; IT2.xDD4_itemVar.linkboomerang.xDD8 = rnd() % 16;
    IT.xDD4_itemVar.linkboomerang.xF90[0] = (rnd() & 1) ? &JO[1] : 0; IT.xDD4_itemVar.linkboomerang.xF90[1] = (rnd() & 1) ? &JO[2] : 0;
    IT2.xDD4_itemVar.linkboomerang.xF90[0] = (rnd() & 1) ? &JO[1] : 0; IT2.xDD4_itemVar.linkboomerang.xF90[1] = (rnd() & 1) ? &JO[2] : 0;
    { int q = rnd() % 4; IT.xDD4_itemVar.linkboomerang.xF98 = q == 0 ? 0 : q == 1 ? &OWN : q == 2 ? &G2 : IT.owner; }
    IT2.xDD4_itemVar.linkboomerang.xF98 = 0;

}

static void pre_hash(void)
{
    { u32 id = PI(IT.xDD4_itemVar.linkboomerang.xF90[0]); unsigned char* b = (unsigned char*) &IT.xDD4_itemVar.linkboomerang.xF90[0]; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT.xDD4_itemVar.linkboomerang.xF90[1]); unsigned char* b = (unsigned char*) &IT.xDD4_itemVar.linkboomerang.xF90[1]; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT.xDD4_itemVar.linkboomerang.xF98); unsigned char* b = (unsigned char*) &IT.xDD4_itemVar.linkboomerang.xF98; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT2.xDD4_itemVar.linkboomerang.xF90[0]); unsigned char* b = (unsigned char*) &IT2.xDD4_itemVar.linkboomerang.xF90[0]; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT2.xDD4_itemVar.linkboomerang.xF90[1]); unsigned char* b = (unsigned char*) &IT2.xDD4_itemVar.linkboomerang.xF90[1]; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT2.xDD4_itemVar.linkboomerang.xF98); unsigned char* b = (unsigned char*) &IT2.xDD4_itemVar.linkboomerang.xF98; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }

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
    freg((const void*) norm_xF74);
    freg((const void*) norm_xF74_from_angle);
    freg((const void*) remove_boomerang);
    freg((const void*) it_8029FDBC);
    freg((const void*) it_8029FDDC);
    freg((const void*) it_8029FE64);
    freg((const void*) it_802A013C_inline);
    freg((const void*) it_802A013C_LoadAnim);
    freg((const void*) it_802A013C);
    freg((const void*) loop_lb_8000BA0C_gobj);
    freg((const void*) loop_lb_8000BA0C_gobj_1);
    freg((const void*) it_802A0534);
    freg((const void*) it_802A07B4);
    freg((const void*) it_802A0810);
    freg((const void*) itLinkBoomerang_Logic18_Destroyed);
    freg((const void*) it_802A0930);
    freg((const void*) it_8029FE64_no_inline);
    freg((const void*) it_802A0C34_sub_1);
    freg((const void*) it_802A0C34_sub_2);
    freg((const void*) it_802A0C34);
    freg((const void*) it_802A0E70);
    freg((const void*) itLinkboomerang_UnkMotion0_Anim);
    freg((const void*) it_802A0F84);
    freg((const void*) it_802A0F88);
    freg((const void*) itLinkboomerang_UnkMotion1_Anim);
    freg((const void*) itLinkboomerang_get_x18);
    freg((const void*) itLinkboomerang_UnkMotion1_Phys);
    freg((const void*) itLinkboomerang_UnkMotion1_Coll);
    freg((const void*) it_802A10E4);
    freg((const void*) itLinkboomerang_UnkMotion2_Anim);
    freg((const void*) itLinkboomerang_UnkMotion2_Phys);
    freg((const void*) clamp_tau);
    freg((const void*) clamp_pi_tau);
    freg((const void*) it_802A13EC_inline);
    freg((const void*) it_802A13EC);
    freg((const void*) it_802A15EC);
    freg((const void*) it_802A15EC_no_inline);
    freg((const void*) itLinkboomerang_UnkMotion2_Coll);
    freg((const void*) it_802A19E0_no_inline);
    freg((const void*) clamp_angle_pi);
    freg((const void*) it_802A1948);
    freg((const void*) it_802A19E0);
    freg((const void*) itLinkboomerang_UnkMotion3_Anim);
    freg((const void*) itLinkboomerang_UnkMotion3_Phys_sub);
    freg((const void*) itLinkboomerang_UnkMotion3_Phys_add_clamp);
    freg((const void*) itLinkboomerang_UnkMotion3_Phys);
    freg((const void*) it_802A1F08);
    freg((const void*) it_802A1FA8);
    freg((const void*) itLinkBoomerang_Logic18_Absorbed);
    freg((const void*) it_802A20E8_inline);
    freg((const void*) it_802A20E8);
    freg((const void*) it_802A2288);
    freg((const void*) it_802A2320);
    freg((const void*) it_802A23CC);
    for (op = 0; op < 55; op++) {
        for (k = 0; k < 150; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { norm_xF74(&IT); } break;
            case 1: { norm_xF74_from_angle(&IT, rw()); } break;
            case 2: { remove_boomerang(&G); } break;
            case 3: { ret = (u32) it_8029FDBC(&G); } break;
            case 4: { it_8029FDDC(&G, (int) fv()); } break;
            case 5: { it_8029FE64(&G, (s32) (fv() % 2)); } break;
            case 6: { it_802A013C_inline(&G); } break;
            case 7: { ret = PI(it_802A013C_LoadAnim((itLinkBoomerangAttributes*) SA, (int) (fv() % 2))); } break;
            case 8: { ret = PI(it_802A013C(rw(), &G, &V1, (Fighter_Part) fv(), (s32) fv())); } break;
            case 9: { loop_lb_8000BA0C_gobj(&G, &JO[1], (itLinkBoomerangAttributes*) SA); } break;
            case 10: { loop_lb_8000BA0C_gobj_1(&G, &JO[1]); } break;
            case 11: { it_802A0534(&G, &V1, rw()); } break;
            case 12: { it_802A07B4(&G); } break;
            case 13: { it_802A0810(&G); } break;
            case 14: { itLinkBoomerang_Logic18_Destroyed(&G); } break;
            case 15: { it_802A0930(&G); } break;
            case 16: { it_8029FE64_no_inline(&G, (s32) (fv() % 2)); } break;
            case 17: { it_802A0C34_sub_1(&G); } break;
            case 18: { it_802A0C34_sub_2(&G, &V1); } break;
            case 19: { ret = (u32) it_802A0C34(&G); } break;
            case 20: { it_802A0E70(&G); } break;
            case 21: { ret = (u32) itLinkboomerang_UnkMotion0_Anim(&G); } break;
            case 22: { it_802A0F84(&G); } break;
            case 23: { it_802A0F88(&G); } break;
            case 24: { ret = (u32) itLinkboomerang_UnkMotion1_Anim(&G); } break;
            case 25: { ret = B(itLinkboomerang_get_x18((itLinkBoomerangAttributes*) SA)); } break;
            case 26: { itLinkboomerang_UnkMotion1_Phys(&G); } break;
            case 27: { ret = (u32) itLinkboomerang_UnkMotion1_Coll(&G); } break;
            case 28: { it_802A10E4(&G); } break;
            case 29: { ret = (u32) itLinkboomerang_UnkMotion2_Anim(&G); } break;
            case 30: { itLinkboomerang_UnkMotion2_Phys(&G); } break;
            case 31: { { float a = (float) ((int) (rnd() % 2001) - 1000) / 100.0f; clamp_tau(&a); ret = B(a); } } break;
            case 32: { { float a = (float) ((int) (rnd() % 2001) - 1000) / 100.0f; clamp_pi_tau(&a); ret = B(a); } } break;
            case 33: { ret = B(it_802A13EC_inline(&IT)); } break;
            case 34: { ret = B(it_802A13EC(&G)); } break;
            case 35: { it_802A15EC(&G, &V1); } break;
            case 36: { it_802A15EC_no_inline(&G, &V1); } break;
            case 37: { ret = (u32) itLinkboomerang_UnkMotion2_Coll(&G); } break;
            case 38: { it_802A19E0_no_inline(&G); } break;
            case 39: { { float a = (float) ((int) (rnd() % 2001) - 1000) / 100.0f; clamp_angle_pi(&a); ret = B(a); } } break;
            case 40: { it_802A1948(&G, (s32) fv()); } break;
            case 41: { it_802A19E0(&G); } break;
            case 42: { ret = (u32) itLinkboomerang_UnkMotion3_Anim(&G); } break;
            case 43: { itLinkboomerang_UnkMotion3_Phys_sub(&G, rw()); } break;
            case 44: { ret = B(itLinkboomerang_UnkMotion3_Phys_add_clamp(rw(), rw(), rw())); } break;
            case 45: { itLinkboomerang_UnkMotion3_Phys(&G); } break;
            case 46: { ret = (u32) it_802A1F08(&G); } break;
            case 47: { ret = (u32) it_802A1FA8(&G); } break;
            case 48: { ret = (u32) itLinkBoomerang_Logic18_Absorbed(&G); } break;
            case 49: { ret = B(it_802A20E8_inline(&IT)); } break;
            case 50: { ret = (u32) it_802A20E8(&G); } break;
            case 51: { ret = (u32) it_802A2288(&G); } break;
            case 52: { ret = (u32) it_802A2320(&G); } break;
            case 53: { it_802A23CC(&G, &G2); } break;
            case 54: { { int i; for (i = 0; i < 4; i++) { HM((u32) it_803F6920[i].anim_id); HM(FI((const void*) it_803F6920[i].animated)); HM(FI((const void*) it_803F6920[i].physics_updated)); HM(FI((const void*) it_803F6920[i].collided)); } } } break;
            }
            pre_hash();
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
