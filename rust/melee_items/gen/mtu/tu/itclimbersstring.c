extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/it/kinds/itclimbersstring.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */
static int LKN;
#include TU_SRC
#include "mtu_common.h"
#include "world.h"
#include "itclimbersstring_gen.h"

static void setup_world(void)
{
    int i;
    u32 r;
    (void) r;
    zero(GO, sizeof GO);
    zero(IT, sizeof IT);
    zero(FT, sizeof FT);
    zero(JO, sizeof JO);
    zero(ATTR, sizeof ATTR);
    zero(ART, sizeof ART);
    zero(LT, sizeof LT);
    zero(DBT, sizeof DBT);
    zero(CM, sizeof CM);
    zero(STT, sizeof STT);
    zero(SDS, sizeof SDS);
    zero(ICD, sizeof ICD);
    zero(CDT, sizeof CDT);
    zero(&SP, sizeof SP);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 3; i++) { EC[i].top = rw(); EC[i].bottom = rw(); EC[i].right = rw(); EC[i].left = rw(); }
    for (i = 0; i < 16; i++) PLINK[i] = 0;
    for (i = 0; i < 9; i++) fill_HSD_GObj(&GO[i]);
    for (i = 0; i < 4; i++) fill_Item(&IT[i]);
    for (i = 0; i < 3; i++) fill_Fighter(&FT[i]);
    for (i = 0; i < 8; i++) fill_HSD_JObj(&JO[i]);
    for (i = 0; i < 2; i++) fill_ItemAttr(&ATTR[i]);
    for (i = 0; i < 2; i++) fill_Article(&ART[i]);
    for (i = 0; i < 2; i++) fill_ItemLogicTable(&LT[i]);
    for (i = 0; i < 4; i++) fill_DynamicBoneTable(&DBT[i]);
    for (i = 0; i < 2; i++) fill_CmSubject(&CM[i]);
    for (i = 0; i < 4; i++) fill_ItemStateTable(&STT[i]);
    for (i = 0; i < 3; i++) fill_ItemStateDesc(&SDS[i]);
    for (i = 0; i < 1; i++) fill_ItemCommonData(&ICD[i]);
    for (i = 0; i < 2; i++) fill_CollData(&CDT[i]);
    fill_SpawnItem(&SP);
    fillg_all();
    /* gobjs: 0..3 items, 4..6 fighters, 7..8 others; one P-link chain per kind */
    for (i = 0; i < 9; i++) {
        GO[i].hsd_obj = &JO[i % 8];
        GO[i].user_data = i < 4 ? (void*) &IT[i] : i < 7 ? (void*) &FT[i - 4] : (void*) BOGUS;
        GO[i].classifier = i < 4 ? HSD_GOBJ_CLASS_ITEM : i < 7 ? HSD_GOBJ_CLASS_FIGHTER : 3;
        GO[i].next = 0;
        GO[i].prev = 0;
    }
    for (i = 0; i < 3; i++) GO[i].next = &GO[i + 1];
    GO[1].prev = &GO[0];
    for (i = 4; i < 6; i++) GO[i].next = &GO[i + 1];
    PLINK[HSD_GOBJ_PLINK_ITEM] = &GO[0];
    PLINK[HSD_GOBJ_PLINK_FIGHTER] = &GO[4];
    for (i = 0; i < 4; i++) {
        IT[i].entity = &GO[i];
        IT[i].xCC_item_attr = &ATTR[i & 1];
        IT[i].xC4_article_data = &ART[i & 1];
        IT[i].xB8_itemLogicTable = &LT[i & 1];
        IT[i].xBBC_dynamicBoneTable = &DBT[i];
        IT[i].xBC_itemStateContainer = &STT[0];
        IT[i].xD0_itemStateDesc = &SDS[i % 3];
        IT[i].xBC0 = BOGUS;
        IT[i].x378_itemColl.x0_gobj = &GO[i];
    }
    for (i = 0; i < 3; i++) FT[i].gobj = &GO[4 + i];
    for (i = 0; i < 8; i++) { JO[i].parent = i ? &JO[(i - 1) / 2] : 0; JO[i].child = 2 * i + 1 < 8 ? &JO[2 * i + 1] : 0; JO[i].next = (i & 1) && i + 1 < 8 ? &JO[i + 1] : 0; }
    for (i = 0; i < 4; i++) for (r = 0; r < 4; r++) DBT[i].bones[r] = &JO[(i + r * 3) % 8];
    for (i = 0; i < 2; i++) { ART[i].x10_modelDesc = 0; LT[i].states = &STT[0]; }
}

static void hash_world(int ft)
{
    int i;
    (void) ft;
    for (i = 0; i < 9; i++) hash_HSD_GObj(&GO[i]);
    for (i = 0; i < 4; i++) hash_Item(&IT[i]);
    if (ft) for (i = 0; i < 3; i++) hash_Fighter(&FT[i]);
    for (i = 0; i < 8; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 2; i++) hash_ItemAttr(&ATTR[i]);
    for (i = 0; i < 2; i++) hash_Article(&ART[i]);
    for (i = 0; i < 2; i++) hash_ItemLogicTable(&LT[i]);
    for (i = 0; i < 4; i++) hash_DynamicBoneTable(&DBT[i]);
    for (i = 0; i < 2; i++) hash_CmSubject(&CM[i]);
    for (i = 0; i < 4; i++) hash_ItemStateTable(&STT[i]);
    for (i = 0; i < 3; i++) hash_ItemStateDesc(&SDS[i]);
    for (i = 0; i < 1; i++) hash_ItemCommonData(&ICD[i]);
    for (i = 0; i < 2; i++) hash_CollData(&CDT[i]);
    hash_SpawnItem(&SP);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
    for (i = 0; i < 3; i++) { HM(B(EC[i].top)); HM(B(EC[i].bottom)); HM(B(EC[i].right)); HM(B(EC[i].left)); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 4; i++) { HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(PI(PLINK[i]));
}

static u32 SATTR[2][14];
static FighterBone PARTS[32]; static ItemLink* OUTP[2]; static float MXA[12];

static void setup_stem(void)
{
    int i;
    (void) i;

{ int q, w; for (q = 0; q < 2; q++) { ART[q].x4_specialAttributes = SATTR[q]; for (w = 0; w < 14; w++) { if (w == 0 || w == 1 || w == 6 || w == 7 || w == 8) ((int*) SATTR[q])[w] = (int) (fv() % 6); else ((float*) SATTR[q])[w] = rw(); } } } { int q; for (q = 0; q < 8; q++) JO[q].flags &= ~JOBJ_USE_QUATERNION; } { int q, j; setup_xpools(); LKN = 0; zero(PARTS, sizeof PARTS); zero(OUTP, sizeof OUTP); zero(MXA, sizeof MXA); for (q = 0; q < 12; q++) MXA[q] = rw(); for (q = 0; q < 32; q++) PARTS[q].joint = &JO[rnd() % 8]; for (q = 0; q < 3; q++) { FT[q].parts = PARTS; FT[q].u.pp.x2240.x = (rnd() & 1) ? 0.0f : rw(); FT[q].u.pp.x2240.y = (rnd() & 1) ? 0.0f : rw(); } for (j = 0; j < 6; j++) { LK[j].next = j ? &LK[j - 1] : 0; LK[j].prev = j < 5 ? &LK[j + 1] : 0; LK[j].gobj = &GO[7 + (rnd() & 1)]; LK[j].jobj = &JO[rnd() % 8]; } LK[5].x2C_b0 = 1; for (q = 0; q < 4; q++) { u32 r = rnd(); IT[q].xDD4_itemVar.climbersstring.x0 = rw(); IT[q].xDD4_itemVar.climbersstring.x4 = &LK[5]; IT[q].xDD4_itemVar.climbersstring.x8 = &LK[0]; IT[q].xDD4_itemVar.climbersstring.xC = &GO[4 + (r >> 8) % 3]; IT[q].xDD4_itemVar.climbersstring.x14 = &JO[(r >> 12) % 8]; } }
}

static void hash_stem(void)
{
    int i;
    (void) i;
    hashg_all();

{ int q, w; for (q = 0; q < 2; q++) for (w = 0; w < 14; w++) HM(SATTR[q][w]); } hash_xpools(); { int q; for (q = 0; q < 32; q++) HM(PI(PARTS[q].joint)); HM(PI(OUTP[0])); HM(PI(OUTP[1])); for (q = 0; q < 12; q++) HM(B(MXA[q])); }
}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    freg_all();
    freg_world();
preg(SATTR, sizeof SATTR); preg_xpools(); preg(PARTS, sizeof PARTS); preg(OUTP, sizeof OUTP); preg(MXA, sizeof MXA);
    for (op = 0; op < 25; op++) {
        int n = 0, ft = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; ft = 0; break;  /* it_802C248C_setupGObj */
        case 1: n = 300; ft = 0; break;  /* it_802C248C_joint24 */
        case 2: n = 300; ft = 0; break;  /* it_802C248C_joint28 */
        case 3: n = 300; ft = 0; break;  /* it_802C248C */
        case 4: n = 300; ft = 0; break;  /* it_802C2750 */
        case 5: n = 300; ft = 0; break;  /* it_802C27D4 */
        case 6: n = 300; ft = 0; break;  /* fn_802C28B8 */
        case 7: n = 300; ft = 0; break;  /* fn_802C28DC */
        case 8: n = 300; ft = 0; break;  /* fn_802C29E8 */
        case 9: n = 300; ft = 0; break;  /* fn_802C2AF4 */
        case 10: n = 300; ft = 0; break;  /* itClimbersstring_Cleanup */
        case 11: n = 300; ft = 0; break;  /* itClimbersstring_UnkMotion3_Anim */
        case 12: n = 300; ft = 0; break;  /* it_802C2CA8 */
        case 13: n = 300; ft = 0; break;  /* it_802C2DB0 */
        case 14: n = 300; ft = 0; break;  /* it_802C2EC4 */
        case 15: n = 300; ft = 0; break;  /* it_802C2CA8_outline */
        case 16: n = 300; ft = 0; break;  /* it_802C30E8 */
        case 17: n = 300; ft = 0; break;  /* it_802C32D4 */
        case 18: n = 300; ft = 0; break;  /* it_802C33B8 */
        case 19: n = 300; ft = 0; break;  /* it_802C3520 */
        case 20: n = 300; ft = 0; break;  /* it_2725_Logic70_PickedUp */
        case 21: n = 300; ft = 0; break;  /* it_802C3810 */
        case 22: n = 300; ft = 0; break;  /* it_802C3864 */
        case 23: n = 300; ft = 0; break;  /* it_802C3950 */
        case 24: n = 300; ft = 0; break;  /* it_2725_Logic70_EvtUnk */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: { HSD_GObj * a0 = &GO[rnd() % 4]; HSD_JObj * a1 = &JO[rnd() % 8]; GObj_RenderFunc a2 = (GObj_RenderFunc) pick_cb(1); it_802C248C_setupGObj(a0, a1, a2); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 1: { Item * a0 = &IT[rnd() % 4]; ret = PI(it_802C248C_joint24(a0)); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 2: { Item * a0 = &IT[rnd() % 4]; ret = PI(it_802C248C_joint28(a0)); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 3: { Item * a0 = &IT[rnd() % 4]; HSD_JObj * a1 = &JO[rnd() % 8]; ret = PI(it_802C248C(a0, a1)); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 4: { Item_GObj * a0 = &GO[rnd() % 4]; it_802C2750(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 5: { Fighter_GObj * a0 = &GO[4 + rnd() % 3]; Vec3 * a1 = &VA[rnd() % 4]; int a2 = fv(); float a3 = rw(); ret = PI(it_802C27D4(a0, a1, a2, a3)); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 6: { Item_GObj * a0 = &GO[rnd() % 4]; fn_802C28B8(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 7: { Item_GObj * a0 = &GO[rnd() % 4]; fn_802C28DC(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 8: { Item_GObj * a0 = &GO[rnd() % 4]; fn_802C29E8(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 9: { HSD_GObj * a0 = &GO[rnd() % 4]; fn_802C2AF4(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 10: { Item_GObj * a0 = &GO[rnd() % 4]; itClimbersstring_Cleanup(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 11: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersstring_UnkMotion3_Anim(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 12: { ItemLink * a0 = &LK[rnd() % 6]; Vec3 * a1 = &VA[rnd() % 4]; itClimbersStringAttributes * a2 = (itClimbersStringAttributes*) SATTR[rnd() & 1]; f32 a3 = rw(); it_802C2CA8(a0, a1, a2, a3); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 13: { ItemLink * a0 = &LK[rnd() % 6]; Vec3 * a1 = &VA[rnd() % 4]; itClimbersStringAttributes * a2 = (itClimbersStringAttributes*) SATTR[rnd() & 1]; f32 a3 = rw(); it_802C2DB0(a0, a1, a2, a3); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 14: { ItemLink * a0 = &LK[rnd() % 6]; Vec3 * a1 = &VA[rnd() % 4]; itClimbersStringAttributes * a2 = (itClimbersStringAttributes*) SATTR[rnd() & 1]; Item * a3 = &IT[rnd() % 4]; ret = (u32) it_802C2EC4(a0, a1, a2, a3); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 15: { ItemLink * a0 = &LK[rnd() % 6]; Vec3 * a1 = &VA[rnd() % 4]; itClimbersStringAttributes * a2 = (itClimbersStringAttributes*) SATTR[rnd() & 1]; f32 a3 = rw(); it_802C2CA8_outline(a0, a1, a2, a3); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 16: { ItemLink * a0 = &LK[rnd() % 6]; Vec3 * a1 = &VA[rnd() % 4]; itClimbersStringAttributes * a2 = (itClimbersStringAttributes*) SATTR[rnd() & 1]; Item * a3 = &IT[rnd() % 4]; ret = (u32) it_802C30E8(a0, a1, a2, a3); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 17: { ItemLink * a0 = &LK[rnd() % 6]; Vec3 * a1 = &VA[rnd() % 4]; itClimbersStringAttributes * a2 = (itClimbersStringAttributes*) SATTR[rnd() & 1]; Item * a3 = &IT[rnd() % 4]; f32 a4 = rw(); ret = (u32) it_802C32D4(a0, a1, a2, a3, a4); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 18: { Item * a0 = &IT[rnd() % 4]; it_802C33B8(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 19: { Item * a0 = &IT[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_802C3520(a0, a1); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 20: { Item_GObj * a0 = &GO[rnd() % 4]; it_2725_Logic70_PickedUp(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 21: { Item_GObj * a0 = &GO[rnd() % 4]; it_802C3810(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 22: { Item_GObj * a0 = &GO[rnd() % 4]; it_802C3864(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 23: { Item_GObj * a0 = &GO[rnd() % 4]; it_802C3950(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            case 24: { Item_GObj * a0 = &GO[rnd() % 4]; Item_GObj * a1 = &GO[rnd() % 7]; it_2725_Logic70_EvtUnk(a0, a1); { int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14)); IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } } } break;
            }
            hash_world(ft);
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
