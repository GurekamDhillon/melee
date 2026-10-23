/* TU: src/melee/it/kinds/itlinkhookshot.c */
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

static ItemLink LK[16];
static HSD_GObj LG[16];
static Fighter FTR;
static FighterBone FPB[160];
static ftData FTD;
static ftLk_DatAttrs LKA;
static mp_UnkStruct0 MPU;
static float MTXB[3][4];
static float MF;
static ItemLink *LTAIL, *LHEAD, *LP;
static int ACNT, GCNT;
extern f32 it_804D6D48;

#include "itlinkhookshot_gen.h"

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


static void fill_ftr(void)
{
    u32 q;
    zero(&FTR, sizeof FTR); zero(FPB, sizeof FPB); zero(&FTD, sizeof FTD); zero(&LKA, sizeof LKA);
    q = rnd() % 6; FTR.motion_id = q == 0 ? 0x168 : q == 1 ? 0x169 : q == 2 ? 0xD4 : q == 3 ? 0xD6 : q == 4 ? 0x100 : 0x168;
    FTR.kind = (rnd() & 1) ? 20 : 6;
    FTR.ground_or_air = (rnd() & 1) ? 1 : 0;
    FTR.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
    { u32 q3 = rnd() % 3; FTR.x34_scale.y = q3 == 0 ? 0.5f : q3 == 1 ? 1.0f : 2.0f; }
    FTR.cur_pos.x = rw(); FTR.cur_pos.y = rw(); FTR.cur_pos.z = rw();
    FTR.self_vel.x = rw(); FTR.self_vel.y = rw(); FTR.pos_delta.x = rw(); FTR.pos_delta.y = rw();
    FTR.input.pressed_buttons = (rnd() & 1) ? 0x100 : (rnd() & 0xFF);
    LKA.x88 = rnd() % 4; LKA.x98 = rnd() % 4; LKA.xA8 = rnd() % 4; LKA.xB4 = rw(); LKA.xB8 = rnd() % 4;
    if (rnd() & 1) { q = rnd() % 5; FTR.mv.ca.specials.grav = q == 0 ? 0.0f : q == 1 ? 1.0f : q == 2 ? 2.0f : q == 3 ? 3.0f : rw(); }
    else { q = rnd() % 4; FTR.mv.lk.specialn.x0.y = q == 0 ? 0.0f : q == 1 ? 1.0f : q == 2 ? -1.0f : 5.0f; }
    FTR.u.lk.xC = (rnd() & 1) ? &G : 0;
    FTR.accessory2_cb = (rnd() & 1) ? it_802A7AAC : 0;
    FTR.death1_cb = (rnd() & 1) ? it_802A7AF0 : 0;
    FTR.accessory3_cb = (rnd() & 1) ? it_802A7B34 : 0;
    FTR.ft_data = &FTD; FTD.ext_attr = &LKA; FTR.dat_attrs = &LKA; FTR.parts = FPB;
    { int i; for (i = 0; i < 160; i++) FPB[i].joint = &JO[i & 3]; }
}
static void hash_ftr(void)
{
    HM((u32) FTR.motion_id); HM((u32) FTR.kind); HM((u32) FTR.ground_or_air); HM(B(FTR.facing_dir)); HM(B(FTR.x34_scale.y));
    HM(B(FTR.cur_pos.x)); HM(B(FTR.cur_pos.y)); HM(B(FTR.cur_pos.z));
    HM(B(FTR.self_vel.x)); HM(B(FTR.self_vel.y)); HM(B(FTR.pos_delta.x)); HM(B(FTR.pos_delta.y));
    HM((u32) FTR.input.pressed_buttons); HM(B(FTR.mv.ca.specials.grav)); HM(B(FTR.mv.lk.specialn.x0.y));
    HM(PI(FTR.u.lk.xC)); HM(FI((const void*) FTR.accessory2_cb)); HM(FI((const void*) FTR.death1_cb)); HM(FI((const void*) FTR.accessory3_cb));
    HM(PI(FPB[139].joint));
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
    if (rnd() % 4 != 0) { u32 q = rnd() % 2; IT.xDD4_itemVar.linkhookshot.x14 = (q == 0 ? 0 : 1); }
    { itLinkHookshotAttributes* A = (itLinkHookshotAttributes*) SA;
      A->x0 = rw();
      A->x4 = rw();
      A->x8 = (float) (rnd() % 5) / 4.0f;
      A->xC = 2 + rnd() % 3;
      A->x10 = 0.5f + (float) (rnd() % 4) * 0.5f;
      A->x14 = (float) (rnd() % 8) * 0.25f;
      A->x18 = (float) (rnd() % 8) * 0.25f;
      A->x1C = (float) (rnd() % 8) * 0.25f;
      A->x20 = (float) (rnd() % 8) * 0.25f;
      A->x24 = (float) (rnd() % 8) * 0.25f;
      A->x28 = (float) (rnd() % 8) * 0.25f;
      A->x2C = 2 + rnd() % 4;
      A->x30 = 0.5f + (float) (rnd() % 4) * 0.5f;
      A->x34 = (float) (rnd() % 8) * 0.25f;
      A->x38 = (float) (rnd() % 8) * 0.25f;
      A->x3C = (float) (rnd() % 4) * 0.05f;
      A->x40 = (float) (rnd() % 6) * 0.5f;
      A->x44 = (float) (rnd() % 6) * 0.5f;
      A->x48 = (float) (rnd() % 4) * 0.1f;
      A->x4C = 1.0f + (float) (rnd() % 3) * 0.5f;
      A->x50 = 1.0f + (float) (rnd() % 3) * 0.5f;
    }
    fill_ftr();
    ACNT = 0; GCNT = 0; LP = 0;
    { int i; for (i = 0; i < 4; i++) { int r, c; for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) JO[i].mtx[r][c] = rw(); } }
    { int r, c; for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) MTXB[r][c] = rw(); }
    MF = rw();
    MPU.x8.x = rw(); MPU.x8.y = rw(); MPU.x8.z = rw(); MPU.x14.x = rw(); MPU.x14.y = rw(); MPU.x14.z = rw();
    {
        int K = 3 + rnd() % 3, i;
        for (i = 0; i < 16; i++) { zero(&LK[i], sizeof LK[i]); fill_ItemLink(&LK[i]); zero(&LG[i], sizeof LG[i]); fill_HSD_GObj(&LG[i]); }
        for (i = 0; i < 16; i++) {
            LK[i].next = (i < K && i) ? &LK[i - 1] : 0; LK[i].prev = (i < K - 1) ? &LK[i + 1] : 0;
            LK[i].gobj = &LG[i]; LK[i].jobj = &JO[(i + 1) & 3];
            LG[i].user_data = &LK[i]; LG[i].hsd_obj = &JO[i & 3];
            LK[i].x2C_b0 = (i == 0) | (i == K - 1) | (rnd() & 1); LK[i].x2C_b1 = rnd() & 1; LK[i].x2C_b2 = rnd() & 1;
        }
        LTAIL = &LK[K - 1]; LHEAD = &LK[0];
        IT.xDD4_itemVar.linkhookshot.x0 = LTAIL; IT.xDD4_itemVar.linkhookshot.x4 = LHEAD;
        IT.xDD4_itemVar.linkhookshot.x8 = &OWN; IT.xDD4_itemVar.linkhookshot.xC = &JO[1];
        IT.xDD4_itemVar.linkhookshot.x10 = (rnd() & 1) ? it_802A2EE4 : 0;
        IT2.xDD4_itemVar.linkhookshot.x0 = 0; IT2.xDD4_itemVar.linkhookshot.x4 = 0; IT2.xDD4_itemVar.linkhookshot.x8 = 0;
        IT2.xDD4_itemVar.linkhookshot.xC = 0; IT2.xDD4_itemVar.linkhookshot.x10 = 0;
        IT.owner = &OWN; IT2.owner = &OWN; OWN.user_data = &FTR; OWN.hsd_obj = &JO[0];
    }
    it_804D6D48 = (rnd() & 1) ? 0.0f : 6.0f;

}

static void pre_hash(void)
{
    { u32 id = PI(IT.xDD4_itemVar.linkhookshot.x0); unsigned char* b = (unsigned char*) &IT.xDD4_itemVar.linkhookshot.x0; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT.xDD4_itemVar.linkhookshot.x4); unsigned char* b = (unsigned char*) &IT.xDD4_itemVar.linkhookshot.x4; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT.xDD4_itemVar.linkhookshot.x8); unsigned char* b = (unsigned char*) &IT.xDD4_itemVar.linkhookshot.x8; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT.xDD4_itemVar.linkhookshot.xC); unsigned char* b = (unsigned char*) &IT.xDD4_itemVar.linkhookshot.xC; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = FI((const void*) IT.xDD4_itemVar.linkhookshot.x10); unsigned char* b = (unsigned char*) &IT.xDD4_itemVar.linkhookshot.x10; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT2.xDD4_itemVar.linkhookshot.x0); unsigned char* b = (unsigned char*) &IT2.xDD4_itemVar.linkhookshot.x0; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT2.xDD4_itemVar.linkhookshot.x4); unsigned char* b = (unsigned char*) &IT2.xDD4_itemVar.linkhookshot.x4; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT2.xDD4_itemVar.linkhookshot.x8); unsigned char* b = (unsigned char*) &IT2.xDD4_itemVar.linkhookshot.x8; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = PI(IT2.xDD4_itemVar.linkhookshot.xC); unsigned char* b = (unsigned char*) &IT2.xDD4_itemVar.linkhookshot.xC; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
    { u32 id = FI((const void*) IT2.xDD4_itemVar.linkhookshot.x10); unsigned char* b = (unsigned char*) &IT2.xDD4_itemVar.linkhookshot.x10; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }

}

static void hash_all(void)
{
    int i;
    hash_HSD_GObj(&G); hash_HSD_GObj(&G2); hash_HSD_GObj(&OWN);
    hash_Item(&IT); hash_Item(&IT2); hash_ItemAttr(&IA);
    for (i = 0; i < 4; i++) hash_jo(&JO[i]);
    for (i = 0; i < 256; i++) HM(SA[i]);
    HM(B(V1.x)); HM(B(V1.y)); HM(B(V1.z)); HM(B(V2.x)); HM(B(V2.y)); HM(B(V2.z));
    hash_ftr();
    { int i; for (i = 0; i < 16; i++) { hash_ItemLink(&LK[i]); hash_HSD_GObj(&LG[i]); } }
    { int i, r, c; for (i = 0; i < 4; i++) for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) HM(B(JO[i].mtx[r][c])); for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) HM(B(MTXB[r][c])); }
    HM(B(MF)); HM(B(it_804D6D48)); HM(PI(LP)); HM(ACNT); HM(GCNT);

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
    preg(LK, sizeof LK);
    preg(LG, sizeof LG);
    preg(&FTR, sizeof FTR);
    preg(FPB, sizeof FPB);
    preg(&MPU, sizeof MPU);
    freg_all();
    freg((const void*) it_802A2418);
    freg((const void*) it_802A2428);
    freg((const void*) it_802A2474);
    freg((const void*) it_802A24A0);
    freg((const void*) it_802A24D0);
    freg((const void*) link_fighter_compare);
    freg((const void*) it_802A2568_inline);
    freg((const void*) it_link_get_joint);
    freg((const void*) it_link_get_joint_c);
    freg((const void*) it_link_lerp);
    freg((const void*) it_link_attr_math);
    freg((const void*) it_802A2568);
    freg((const void*) it_802A2B10);
    freg((const void*) it_802A2BA4);
    freg((const void*) itLinkhookshot_UnkMotion8_Anim);
    freg((const void*) fn_802A2E4C);
    freg((const void*) itLinkhookshot_UnkMotion0_Phys);
    freg((const void*) it_802A2EE4_inline);
    freg((const void*) it_802A2EE4_inline_alt);
    freg((const void*) it_802A2EE4_inline_alt_mtx_first);
    freg((const void*) it_802A2EE4_inline_alt_pad);
    freg((const void*) it_802A2EE4);
    freg((const void*) itLinkhookshot_UnkMotion1_Phys);
    freg((const void*) mtx_reset);
    freg((const void*) vec3_eq_mtx);
    freg((const void*) fn_802A3110_inline);
    freg((const void*) fn_802A3110);
    freg((const void*) itLinkhookshot_UnkMotion2_Phys);
    freg((const void*) it_802A3254);
    freg((const void*) itLinkhookshot_UnkMotion3_Phys);
    freg((const void*) fn_802A33A0_GetFighter);
    freg((const void*) fn_802A33A0);
    freg((const void*) itLinkhookshot_UnkMotion4_Phys);
    freg((const void*) it_802A3500);
    freg((const void*) itLinkhookshot_UnkMotion5_Phys);
    freg((const void*) it_802A3630);
    freg((const void*) itLinkhookshot_UnkMotion6_Phys);
    freg((const void*) it_802A3828);
    freg((const void*) itLinkhookshot_UnkMotion7_Phys);
    freg((const void*) it_802A39FC);
    freg((const void*) itLinkhookshot_UnkMotion8_Phys);
    freg((const void*) it_802A3C98);
    freg((const void*) it_802A3D90);
    freg((const void*) it_802A3E50);
    freg((const void*) it_802A40D0);
    freg((const void*) it_802A42F4);
    freg((const void*) it_802A43B8);
    freg((const void*) it_802A43EC);
    freg((const void*) it_802A4420);
    freg((const void*) it_802A4454);
    freg((const void*) it_802A44CC);
    freg((const void*) test_comp);
    freg((const void*) it_802A4758_permuterslop);
    freg((const void*) it_802A4758);
    freg((const void*) it_802A49B0);
    freg((const void*) it_802A6A78_normalize_diff);
    freg((const void*) it_802A4BFC_sqrtf_offset);
    freg((const void*) it_802A4BFC_normalize_diff);
    freg((const void*) it_802A4BFC);
    freg((const void*) it_802A5320);
    freg((const void*) it_802A5770_inline);
    freg((const void*) it_802A5770);
    freg((const void*) it_802A5AE0);
    freg((const void*) it_802A5E28);
    freg((const void*) it_802A5FE0);
    freg((const void*) it_802A4758_no_inline);
    freg((const void*) it_802A6474);
    freg((const void*) it_802A678C);
    freg((const void*) it_802A6944);
    freg((const void*) it_802A6A78_get_next);
    freg((const void*) it_802A6A78_get_pos);
    freg((const void*) it_802A6A78_normalize_diff_rev);
    freg((const void*) it_802A6A78);
    freg((const void*) it_802A6DC8);
    freg((const void*) it_802A6F80);
    freg((const void*) it_802A7168);
    freg((const void*) it_802A7384);
    freg((const void*) itLinkHookshot_Logic20_PickedUp_inline);
    freg((const void*) itLinkHookshot_Logic20_PickedUp);
    freg((const void*) it_802A76EC);
    freg((const void*) it_802A7764);
    freg((const void*) it_802A77DC);
    freg((const void*) it_802A7840);
    freg((const void*) it_802A78B8);
    freg((const void*) it_802A793C);
    freg((const void*) it_802A79A0);
    freg((const void*) it_802A7A04);
    freg((const void*) it_802A7AAC);
    freg((const void*) it_802A7AF0);
    freg((const void*) it_802A7B34_6944_inline);
    freg((const void*) it_802A7B34);
    freg((const void*) it_802A7D40);
    for (op = 0; op < 93; op++) {
        for (k = 0; k < 150; k++) {
            u32 ret = 0;
            tu_case(op, k);

            setup();
            switch (op) {
            case 0: { it_802A2418(&G); } break;
            case 1: { it_802A2428(&G); } break;
            case 2: { it_802A2474(&IT); } break;
            case 3: { it_802A24A0(&LG[0], (int) fv()); } break;
            case 4: { it_802A24D0(LTAIL, rw()); } break;
            case 5: { ret = (u32) link_fighter_compare(&FTR); } break;
            case 6: { it_802A2568_inline(LTAIL, &JO[1], &G, &V1); } break;
            case 7: { ret = PI(it_link_get_joint(&IT, (s32) fv())); } break;
            case 8: { ret = PI(it_link_get_joint_c(&IT)); } break;
            case 9: { ret = B(it_link_lerp(rw(), rw(), rw())); } break;
            case 10: { it_link_attr_math((itLinkHookshotAttributes*) SA, (s32) (fv() % 2), (rnd() & 1) ? ((rnd() & 1) ? 0.5f : 1.0f) : 2.0f); } break;
            case 11: { ret = PI(it_802A2568(&IT, &JO[1], (s32) (fv() % 2), (rnd() & 1) ? ((rnd() & 1) ? 0.5f : 1.0f) : 2.0f)); } break;
            case 12: { it_802A2B10(&G); } break;
            case 13: { ret = PI(it_802A2BA4(&OWN, &V1, rw(), (s32) fv())); } break;
            case 14: { ret = (u32) itLinkhookshot_UnkMotion8_Anim(&G); } break;
            case 15: { fn_802A2E4C(&G); } break;
            case 16: { itLinkhookshot_UnkMotion0_Phys(&G); } break;
            case 17: { it_802A2EE4_inline(MTXB, LTAIL, &V1); } break;
            case 18: { it_802A2EE4_inline_alt(LTAIL, &V1); } break;
            case 19: { it_802A2EE4_inline_alt_mtx_first(LTAIL, &V1); } break;
            case 20: { it_802A2EE4_inline_alt_pad(LTAIL, &V1); } break;
            case 21: { it_802A2EE4(&G); } break;
            case 22: { itLinkhookshot_UnkMotion1_Phys(&G); } break;
            case 23: { mtx_reset(MTXB); } break;
            case 24: { vec3_eq_mtx(&V1, MTXB); } break;
            case 25: { fn_802A3110_inline(LTAIL, &V1); } break;
            case 26: { fn_802A3110(&G); } break;
            case 27: { itLinkhookshot_UnkMotion2_Phys(&G); } break;
            case 28: { it_802A3254(&G); } break;
            case 29: { itLinkhookshot_UnkMotion3_Phys(&G); } break;
            case 30: { ret = PI(fn_802A33A0_GetFighter(&IT)); } break;
            case 31: { fn_802A33A0(&G); } break;
            case 32: { itLinkhookshot_UnkMotion4_Phys(&G); } break;
            case 33: { it_802A3500(&G); } break;
            case 34: { itLinkhookshot_UnkMotion5_Phys(&G); } break;
            case 35: { it_802A3630(&G); } break;
            case 36: { itLinkhookshot_UnkMotion6_Phys(&G); } break;
            case 37: { it_802A3828(&G); } break;
            case 38: { itLinkhookshot_UnkMotion7_Phys(&G); } break;
            case 39: { it_802A39FC(&G); } break;
            case 40: { itLinkhookshot_UnkMotion8_Phys(&G); } break;
            case 41: { ret = B(it_802A3C98(&V1, &V2, &IT.pos)); } break;
            case 42: { ret = (u32) it_802A3D90(LTAIL); } break;
            case 43: { ret = (u32) it_802A3E50(LTAIL, (rnd() & 1) ? 20 : 6, rw()); } break;
            case 44: { ret = (u32) it_802A40D0(LTAIL, rw()); } break;
            case 45: { ret = (u32) it_802A42F4(LTAIL, rw()); } break;
            case 46: { it_802A43B8(LTAIL); } break;
            case 47: { it_802A43EC(LTAIL); } break;
            case 48: { it_802A4420(LTAIL); } break;
            case 49: { it_802A4454(LTAIL); } break;
            case 50: { it_802A44CC(LTAIL, &V1, (itLinkHookshotAttributes*) SA, rw()); } break;
            case 51: { test_comp(&IT.pos, &V1, &V2, &MF); } break;
            case 52: { ret = PI(it_802A4758_permuterslop(LTAIL)); } break;
            case 53: { it_802A4758(LTAIL, &V1, (itLinkHookshotAttributes*) SA, rw()); } break;
            case 54: { it_802A49B0(LTAIL, &V1, (itLinkHookshotAttributes*) SA, rw()); } break;
            case 55: { { double d = it_802A6A78_normalize_diff(&V1, &V2, &IT.pos); HM64(d); } } break;
            case 56: { ret = B(it_802A4BFC_sqrtf_offset(rw())); } break;
            case 57: { { double d = it_802A4BFC_normalize_diff(&V1, &V2, &IT.pos); HM64(d); } } break;
            case 58: { ret = (u32) it_802A4BFC(LTAIL, &V1, (itLinkHookshotAttributes*) SA, &FTR); } break;
            case 59: { ret = (u32) it_802A5320(LTAIL, &V1, (itLinkHookshotAttributes*) SA, &FTR); } break;
            case 60: { it_802A5770_inline(LTAIL, (itLinkHookshotAttributes*) SA, &FTR, (s32) fv()); } break;
            case 61: { it_802A5770(LTAIL, &V1, (itLinkHookshotAttributes*) SA, &FTR); } break;
            case 62: { ret = (u32) it_802A5AE0(LTAIL, &V1, (itLinkHookshotAttributes*) SA); } break;
            case 63: { ret = (u32) it_802A5E28(LTAIL, &V1, (itLinkHookshotAttributes*) SA, rw()); } break;
            case 64: { ret = (u32) it_802A5FE0(LTAIL, LHEAD, &V1, (itLinkHookshotAttributes*) SA, rw()); } break;
            case 65: { it_802A4758_no_inline(LTAIL, &V1, (itLinkHookshotAttributes*) SA, rw()); } break;
            case 66: { it_802A6474(LTAIL, LHEAD, &V1, (itLinkHookshotAttributes*) SA); } break;
            case 67: { ret = (u32) it_802A678C(LTAIL, &V1, (itLinkHookshotAttributes*) SA, rw()); } break;
            case 68: { it_802A6944(&IT, rw()); } break;
            case 69: { it_802A6A78_get_next(LTAIL, &LP); } break;
            case 70: { ret = PI(it_802A6A78_get_pos(LTAIL)); } break;
            case 71: { { double d = it_802A6A78_normalize_diff_rev(&V1, &V2, &IT.pos); HM64(d); } } break;
            case 72: { ret = (u32) it_802A6A78(LTAIL, &V1, (itLinkHookshotAttributes*) SA, &FTR); } break;
            case 73: { it_802A6DC8(&JO[1], &V1, &V2); } break;
            case 74: { it_802A6F80(&JO[1], &V1, &V2, rw()); } break;
            case 75: { it_802A7168(&IT, &V1, rw()); } break;
            case 76: { it_802A7384(&IT, &V1, rw()); } break;
            case 77: { itLinkHookshot_Logic20_PickedUp_inline(&G); } break;
            case 78: { itLinkHookshot_Logic20_PickedUp(&G); } break;
            case 79: { it_802A76EC(&G); } break;
            case 80: { it_802A7764(&G); } break;
            case 81: { it_802A77DC(&G); } break;
            case 82: { it_802A7840(&G); } break;
            case 83: { it_802A78B8(&G, &V1); } break;
            case 84: { it_802A793C(&G); } break;
            case 85: { it_802A79A0(&G); } break;
            case 86: { it_802A7A04(&G); } break;
            case 87: { it_802A7AAC(&OWN); } break;
            case 88: { it_802A7AF0(&OWN); } break;
            case 89: { it_802A7B34_6944_inline(&IT); } break;
            case 90: { it_802A7B34(&OWN); } break;
            case 91: { it_802A7D40(&G, &G2); } break;
            case 92: { { int i; for (i = 0; i < 9; i++) { HM((u32) it_803F6998[i].anim_id); HM(FI((const void*) it_803F6998[i].animated)); HM(FI((const void*) it_803F6998[i].physics_updated)); HM(FI((const void*) it_803F6998[i].collided)); } } } break;
            }
            pre_hash();
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
