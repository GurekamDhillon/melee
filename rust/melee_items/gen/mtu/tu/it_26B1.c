extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/it/it_26B1.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
#include "world.h"
#include "it_26B1_gen.h"

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

static Article* ARR[256]; static Article* ARR2[256]; static u32 SPA[2][8]; static const int KINDS[28] = { It_Kind_Heart, It_Kind_Tomato, It_Kind_Foods, It_Kind_Coin, It_Kind_Lucky_Egg, It_Kind_WhispyHealApple, It_Kind_L_Gun, It_Kind_S_Scope, It_Kind_StarRod, It_Kind_LipStick, It_Kind_F_Flower, It_Kind_Harisen, It_Kind_Freeze, It_Kind_MSBomb, It_Kind_Flipper, It_Kind_BombHei, It_Kind_Link_Bomb, It_Kind_CLink_Bomb, It_Kind_Kyasarin_Egg, It_Kind_WhispyApple, It_Kind_Kuriboh, It_Kind_Octarock_Stone, It_Kind_Old_Kuri, It_Kind_Arwing_Laser, It_Kind_Box, It_Kind_Kusudama, It_Kind_Nokonoko, It_PKind_Marumine };

static void setup_stem(void)
{
    int i;
    (void) i;
    HSD_GObjPLinkHead = PLINK;
    it_804D6D28 = &ICD[0];
{ int q, k; setup_xpools(); for (q = 0; q < 256; q++) { ARR[q] = &ART[q & 1]; ARR2[q] = (rnd() & 3) ? 0 : &ART[q & 1]; } it_804D6D24 = ARR; it_804D6D38 = ARR2; for (q = 0; q < 2; q++) { for (k = 0; k < 8; k++) SPA[q][k] = (k & 1) ? fv() : B(rw()); ART[q].x4_specialAttributes = (void*) SPA[q]; ART[q].xC_itemStates = &STA[q]; } for (q = 0; q < 2; q++) { static const int BL[11] = { 1, 20, 8, 9, 4, 12, 21, 25, 31, 11, 6 }; BOR[q].x14 = (rnd() % 4 == 0) ? (int) fv() : BL[rnd() % 11]; } for (q = 0; q < 4; q++) { if (rnd() % 4) IT[q].kind = KINDS[rnd() % 28]; if (rnd() % 3) IT[q].hold_kind = 4 + rnd() % 4; IT[q].anim_id = (rnd() % 4 == 0) ? -1 : (int) (rnd() % 8); } for (q = 0; q < 8; q++) JO[q].flags &= ~JOBJ_USE_QUATERNION; }
}

static void hash_stem(void)
{
    int i;
    (void) i;
    hashg_all();
    HM(PI(HSD_GObjPLinkHead));
    HM(PI(it_804D6D28));
hash_xpools(); { int q; for (q = 0; q < 256; q++) { HM(PI(ARR2[q])); } HM(RH(SPA, sizeof SPA)); }
}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    freg_all();
    freg_world();
preg_xpools(); preg(ARR, sizeof ARR); preg(ARR2, sizeof ARR2); preg(SPA, sizeof SPA);
    for (op = 0; op < 82; op++) {
        int n = 0, ft = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; ft = 0; break;  /* it_8026B1D4 */
        case 1: n = 300; ft = 0; break;  /* it_8026B294 */
        case 2: n = 300; ft = 0; break;  /* itIsHeavy */
        case 3: n = 300; ft = 0; break;  /* it_8026B2D8 */
        case 4: n = 300; ft = 0; break;  /* itGetKind */
        case 5: n = 300; ft = 0; break;  /* it_8026B30C */
        case 6: n = 300; ft = 0; break;  /* itGetHoldKind */
        case 7: n = 300; ft = 0; break;  /* itGetDamageMultiplier */
        case 8: n = 300; ft = 0; break;  /* it_8026B344 */
        case 9: n = 300; ft = 0; break;  /* itGetGrabRangeX */
        case 10: n = 300; ft = 0; break;  /* itGetGrabRangeY */
        case 11: n = 300; ft = 0; break;  /* it_8026B390 */
        case 12: n = 300; ft = 0; break;  /* it_8026B3A8 */
        case 13: n = 300; ft = 0; break;  /* it_8026B3C0 */
        case 14: n = 300; ft = 0; break;  /* it_8026B3F8 */
        case 15: n = 300; ft = 0; break;  /* it_Mex_RegisterIfEmpty */
        case 16: n = 300; ft = 0; break;  /* it_ResetStageArticleTable */
        case 17: n = 300; ft = 0; break;  /* Item_StageArticleTableTest */
        case 18: n = 300; ft = 0; break;  /* it_8026B40C */
        case 19: n = 300; ft = 0; break;  /* it_8026B424 */
        case 20: n = 300; ft = 0; break;  /* it_8026B47C */
        case 21: n = 300; ft = 0; break;  /* it_8026B4F0 */
        case 22: n = 300; ft = 0; break;  /* it_8026B54C */
        case 23: n = 300; ft = 0; break;  /* it_8026B560 */
        case 24: n = 300; ft = 0; break;  /* it_8026B574 */
        case 25: n = 300; ft = 0; break;  /* it_8026B588 */
        case 26: n = 300; ft = 0; break;  /* it_8026B594 */
        case 27: n = 300; ft = 0; break;  /* it_8026B5E4 */
        case 28: n = 300; ft = 0; break;  /* it_8026B634 */
        case 29: n = 300; ft = 0; break;  /* it_8026B684 */
        case 30: n = 300; ft = 0; break;  /* it_8026B6A8 */
        case 31: n = 300; ft = 0; break;  /* it_8026B6C8 */
        case 32: n = 300; ft = 0; break;  /* it_8026B718 */
        case 33: n = 300; ft = 0; break;  /* it_8026B724 */
        case 34: n = 300; ft = 0; break;  /* it_8026B73C */
        case 35: n = 300; ft = 0; break;  /* it_8026B774 */
        case 36: n = 300; ft = 0; break;  /* itGetMotionId */
        case 37: n = 300; ft = 0; break;  /* itGetTeamId */
        case 38: n = 300; ft = 0; break;  /* it_8026B7BC */
        case 39: n = 300; ft = 0; break;  /* it_8026B7CC */
        case 40: n = 300; ft = 0; break;  /* it_8026B7D8 */
        case 41: n = 300; ft = 0; break;  /* it_8026B7E0 */
        case 42: n = 300; ft = 0; break;  /* it_8026B7E8 */
        case 43: n = 300; ft = 0; break;  /* it_8026B7F8 */
        case 44: n = 300; ft = 0; break;  /* it_8026B894 */
        case 45: n = 300; ft = 0; break;  /* it_8026B924 */
        case 46: n = 300; ft = 0; break;  /* it_8026B960 */
        case 47: n = 300; ft = 0; break;  /* it_8026B9A8 */
        case 48: n = 300; ft = 0; break;  /* it_8026BAE8 */
        case 49: n = 300; ft = 0; break;  /* it_8026BB20 */
        case 50: n = 300; ft = 0; break;  /* it_8026BB44 */
        case 51: n = 300; ft = 0; break;  /* it_8026BB68 */
        case 52: n = 300; ft = 0; break;  /* it_8026BB88 */
        case 53: n = 300; ft = 0; break;  /* it_8026BBCC */
        case 54: n = 300; ft = 0; break;  /* it_8026BC14 */
        case 55: n = 300; ft = 0; break;  /* it_8026BC68 */
        case 56: n = 300; ft = 0; break;  /* itGetOwner */
        case 57: n = 300; ft = 0; break;  /* itGetAttackId */
        case 58: n = 300; ft = 0; break;  /* it_8026BC90 */
        case 59: n = 300; ft = 0; break;  /* it_8026BCF4 */
        case 60: n = 300; ft = 0; break;  /* it_8026BD0C */
        case 61: n = 300; ft = 0; break;  /* it_8026BD24 */
        case 62: n = 300; ft = 0; break;  /* it_8026BD3C */
        case 63: n = 300; ft = 0; break;  /* it_8026BD54 */
        case 64: n = 300; ft = 0; break;  /* it_8026BD6C */
        case 65: n = 300; ft = 0; break;  /* it_8026BD84 */
        case 66: n = 300; ft = 0; break;  /* it_8026BD9C */
        case 67: n = 300; ft = 0; break;  /* it_8026BDB4 */
        case 68: n = 300; ft = 0; break;  /* it_8026BDCC */
        case 69: n = 300; ft = 0; break;  /* it_8026BE28 */
        case 70: n = 300; ft = 0; break;  /* it_8026BE84 */
        case 71: n = 300; ft = 0; break;  /* it_8026C100 */
        case 72: n = 300; ft = 0; break;  /* it_8026C16C */
        case 73: n = 300; ft = 0; break;  /* it_8026C1B4 */
        case 74: n = 300; ft = 0; break;  /* it_8026C1D4 */
        case 75: n = 300; ft = 0; break;  /* it_8026C1E8 */
        case 76: n = 300; ft = 0; break;  /* it_8026C220 */
        case 77: n = 300; ft = 0; break;  /* it_8026C258 */
        case 78: n = 300; ft = 0; break;  /* it_8026C334 */
        case 79: n = 300; ft = 0; break;  /* it_8026C368 */
        case 80: n = 300; ft = 0; break;  /* it_8026C3FC */
        case 81: n = 300; ft = 0; break;  /* it_8026C42C */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: { HSD_GObj * a0 = &GO[rnd() % 4]; HitCapsule * a1 = pick_hit(); ret = B(it_8026B1D4(a0, a1));  } break;
            case 1: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_8026B294(a0, a1);  } break;
            case 2: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itIsHeavy(a0);  } break;
            case 3: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026B2D8(a0);  } break;
            case 4: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itGetKind(a0);  } break;
            case 5: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026B30C(a0);  } break;
            case 6: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itGetHoldKind(a0);  } break;
            case 7: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = B(itGetDamageMultiplier(a0));  } break;
            case 8: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_8026B344(a0, a1);  } break;
            case 9: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = B(itGetGrabRangeX(a0));  } break;
            case 10: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = B(itGetGrabRangeY(a0));  } break;
            case 11: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026B390(a0);  } break;
            case 12: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026B3A8(a0);  } break;
            case 13: { ItemKind a0 = (ItemKind) KINDS[rnd() % 28]; ret = (u32) it_8026B3C0(a0);  } break;
            case 14: { Article * a0 = &ART[rnd() % 2]; s32 a1 = (s32) (It_Kind_Kuriboh + fv() % 200); it_8026B3F8(a0, a1);  } break;
            case 15: { Article * a0 = &ART[rnd() % 2]; s32 a1 = (s32) (It_Kind_Kuriboh + fv() % 200); it_Mex_RegisterIfEmpty(a0, a1);  } break;
            case 16: {  it_ResetStageArticleTable();  } break;
            case 17: {  ret = (u32) Item_StageArticleTableTest();  } break;
            case 18: { Article * a0 = &ART[rnd() % 2]; s32 a1 = (s32) ((rnd() % 6 == 0) ? 5000 + fv() % 10 : It_Kind_Old_Kuri + fv() % 30); it_8026B40C(a0, a1);  } break;
            case 19: { int a0 = fv(); ret = B(it_8026B424(a0));  } break;
            case 20: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026B47C(a0);  } break;
            case 21: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026B4F0(a0);  } break;
            case 22: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = B(it_8026B54C(a0));  } break;
            case 23: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = B(it_8026B560(a0));  } break;
            case 24: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = B(it_8026B574(a0));  } break;
            case 25: {  ret = (u32) it_8026B588();  } break;
            case 26: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026B594(a0);  } break;
            case 27: { Vec3 * a0 = &VA[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; HSD_GObj * a2 = &GO[rnd() % 4]; ret = PI(it_8026B5E4(a0, a1, a2));  } break;
            case 28: { Vec3 * a0 = &VA[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; Item_GObj * a2 = &GO[rnd() % 4]; f32 a3 = rw(); ret = PI(it_8026B634(a0, a1, a2, a3));  } break;
            case 29: { Vec3 * a0 = &VA[rnd() % 4]; ret = B(it_8026B684(a0));  } break;
            case 30: { Vec3 * a0 = &VA[rnd() % 4]; HSD_GObj * a1 = &GO[rnd() % 4]; ret = B(it_8026B6A8(a0, a1));  } break;
            case 31: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026B6C8(a0);  } break;
            case 32: { HSD_GObj * a0 = &GO[rnd() % 4]; f32 a1 = rw(); it_8026B718(a0, a1);  } break;
            case 33: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026B724(a0);  } break;
            case 34: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026B73C(a0);  } break;
            case 35: { HSD_GObj * a0 = &GO[rnd() % 4]; u8 a1 = fv(); ret = (u32) it_8026B774(a0, a1);  } break;
            case 36: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itGetMotionId(a0);  } break;
            case 37: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itGetTeamId(a0);  } break;
            case 38: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026B7BC(a0);  } break;
            case 39: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026B7CC(a0);  } break;
            case 40: {  ret = (u32) it_8026B7D8();  } break;
            case 41: {  ret = (u32) it_8026B7E0();  } break;
            case 42: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026B7E8(a0);  } break;
            case 43: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026B7F8(a0);  } break;
            case 44: { HSD_GObj * a0 = &GO[rnd() % 4]; HSD_GObj * a1 = &GO[rnd() % 4]; ret = (u32) it_8026B894(a0, a1);  } break;
            case 45: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026B924(a0);  } break;
            case 46: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = B(it_8026B960(a0));  } break;
            case 47: { HSD_GObj * a0 = &GO[rnd() % 4]; HSD_GObj * a1 = &GO[rnd() % 4]; Fighter_Part a2 = fv(); it_8026B9A8(a0, a1, a2);  } break;
            case 48: { HSD_GObj * a0 = &GO[rnd() % 4]; f32 a1 = rw(); it_8026BAE8(a0, a1);  } break;
            case 49: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BB20(a0);  } break;
            case 50: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BB44(a0);  } break;
            case 51: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_8026BB68(a0, a1);  } break;
            case 52: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_8026BB88(a0, a1);  } break;
            case 53: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_8026BBCC(a0, a1);  } break;
            case 54: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BC14(a0);  } break;
            case 55: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026BC68(a0);  } break;
            case 56: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = PI(itGetOwner(a0));  } break;
            case 57: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itGetAttackId(a0);  } break;
            case 58: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_8026BC90(a0, a1);  } break;
            case 59: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BCF4(a0);  } break;
            case 60: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BD0C(a0);  } break;
            case 61: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BD24(a0);  } break;
            case 62: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BD3C(a0);  } break;
            case 63: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BD54(a0);  } break;
            case 64: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BD6C(a0);  } break;
            case 65: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BD84(a0);  } break;
            case 66: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BD9C(a0);  } break;
            case 67: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BDB4(a0);  } break;
            case 68: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BDCC(a0);  } break;
            case 69: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026BE28(a0);  } break;
            case 70: { BobOmbRain * a0 = &BOR[rnd() & 1]; ret = PI(it_8026BE84(a0));  } break;
            case 71: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = PI(it_8026C100(a0));  } break;
            case 72: { HSD_GObj * a0 = &GO[rnd() % 4]; bool a1 = fv(); it_8026C16C(a0, a1);  } break;
            case 73: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026C1B4(a0);  } break;
            case 74: {  ret = (u32) it_8026C1D4();  } break;
            case 75: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026C1E8(a0);  } break;
            case 76: { HSD_GObj * a0 = &GO[rnd() % 4]; HSD_GObj * a1 = &GO[rnd() % 4]; it_8026C220(a0, a1);  } break;
            case 77: { Vec3 * a0 = &VA[rnd() % 4]; f32 a1 = rw(); ret = PI(it_8026C258(a0, a1));  } break;
            case 78: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_8026C334(a0, a1);  } break;
            case 79: { HSD_GObj * a0 = &GO[rnd() % 4]; it_8026C368(a0);  } break;
            case 80: {  it_8026C3FC();  } break;
            case 81: {  it_8026C42C();  } break;
            }
            hash_world(ft);
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
