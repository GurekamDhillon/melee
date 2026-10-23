extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/it/kinds/itclimbersice.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */
#include <Runtime/platform.h>
#include <melee/ef/efsync.h>
#include <melee/ef/efasync.h>
#define NARGN_(_1,_2,_3,_4,_5,_6,_7,_8,_9,N,...) N
#define NARGN(...) NARGN_(__VA_ARGS__,9,8,7,6,5,4,3,2,1,0)
#define EFA(x) _Generic((x), Vec3*: 3, float*: 2, int: 0, default: 1), (unsigned long long) (x)
#define CAT_(a,b) a##b
#define CAT(a,b) CAT_(a,b)
#define EFS_2(id, g) oracle_efsync_n(0, id, g)
#define EFS_3(id, g, a) oracle_efsync_n(1, id, g, EFA(a))
#define EFS_4(id, g, a, b) oracle_efsync_n(2, id, g, EFA(a), EFA(b))
#define EFS_5(id, g, a, b, c) oracle_efsync_n(3, id, g, EFA(a), EFA(b), EFA(c))
#define EFS_6(id, g, a, b, c, d) oracle_efsync_n(4, id, g, EFA(a), EFA(b), EFA(c), EFA(d))
#define EFAS_5(g, q, k, id, j) oracle_efasync_n(0, g, q, k, id, j)
#define EFAS_6(g, q, k, id, j, a) oracle_efasync_n(1, g, q, k, id, j, EFA(a))
#define EFAS_7(g, q, k, id, j, a, b) oracle_efasync_n(2, g, q, k, id, j, EFA(a), EFA(b))
#define EFAS_8(g, q, k, id, j, a, b, c) oracle_efasync_n(3, g, q, k, id, j, EFA(a), EFA(b), EFA(c))
#define EFAS_9(g, q, k, id, j, a, b, c, d) oracle_efasync_n(4, g, q, k, id, j, EFA(a), EFA(b), EFA(c), EFA(d))
#define efSync_Spawn(...) CAT(EFS_, NARGN(__VA_ARGS__))(__VA_ARGS__)
#define efAsync_Spawn(...) CAT(EFAS_, NARGN(__VA_ARGS__))(__VA_ARGS__)
void* oracle_efsync_n(int, int, HSD_GObj*, ...);
void oracle_efasync_n(int, HSD_GObj*, void*, u32, u32, HSD_JObj*, ...);
static void* ARCP;

#include TU_SRC
#include "mtu_common.h"
#include "world.h"
#include "itclimbersice_gen.h"

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

static u32 SATTR[2][13];

static void setup_stem(void)
{
    int i;
    (void) i;
    it_804D6D28 = &ICD[0];
{ int q, w; for (q = 0; q < 2; q++) { ART[q].x4_specialAttributes = SATTR[q]; for (w = 0; w < 13; w++) { if (w == 11 || w == 12) ((int*) SATTR[q])[w] = (int) (fv() % 6); else ((float*) SATTR[q])[w] = rw(); } } } { int q; for (q = 0; q < 4; q++) { u32 r = rnd(); IT[q].kind = It_Kind_IceClimber_Ice + (r & 1); *(void**) &IT[q].xDD4_itemVar = (r & 6) == 0 ? (void*) 0 : (void*) &GO[4 + (r >> 8) % 3]; } }
}

static void hash_stem(void)
{
    int i;
    (void) i;
    hashg_all();
    HM(PI(it_804D6D28));
{ int q, w; for (q = 0; q < 2; q++) for (w = 0; w < 13; w++) HM(SATTR[q][w]); }
}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    freg_all();
    freg_world();
preg(SATTR, sizeof SATTR);
    for (op = 0; op < 33; op++) {
        int n = 0, ft = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; ft = 0; break;  /* itClimbersIce_sub_x4 */
        case 1: n = 300; ft = 0; break;  /* itClimbersice_Spawn */
        case 2: n = 300; ft = 0; break;  /* itClimbersice_Spawn2 */
        case 3: n = 300; ft = 0; break;  /* it_802C1590 */
        case 4: n = 300; ft = 0; break;  /* it_802C16F8 */
        case 5: n = 300; ft = 0; break;  /* it_802C17DC */
        case 6: n = 300; ft = 0; break;  /* it_2725_Logic90_Destroyed */
        case 7: n = 300; ft = 0; break;  /* it_802C1854 */
        case 8: n = 300; ft = 0; break;  /* it_802C1950 */
        case 9: n = 300; ft = 0; break;  /* itClimbersice_UnkMotion0_Anim */
        case 10: n = 300; ft = 0; break;  /* itClimbersice_UnkMotion0_Phys */
        case 11: n = 300; ft = 0; break;  /* itClimbersice_UnkMotion0_Coll */
        case 12: n = 300; ft = 0; break;  /* it_802C1A58 */
        case 13: n = 300; ft = 0; break;  /* itClimbersice_UnkMotion1_Anim */
        case 14: n = 300; ft = 0; break;  /* itClimbersice_UnkMotion1_Phys */
        case 15: n = 300; ft = 0; break;  /* itClimbersice_UnkMotion1_Coll */
        case 16: n = 300; ft = 0; break;  /* it_802C1AE4 */
        case 17: n = 300; ft = 0; break;  /* itClimbersice_UnkMotion2_Anim */
        case 18: n = 300; ft = 0; break;  /* itClimbersice_Phys_inline */
        case 19: n = 300; ft = 0; break;  /* itClimbersice_UnkMotion2_Phys */
        case 20: n = 300; ft = 0; break;  /* itClimbersice_Coll */
        case 21: n = 300; ft = 0; break;  /* itClimbersice_UnkMotion2_Coll */
        case 22: n = 300; ft = 0; break;  /* fn_802C1D44 */
        case 23: n = 300; ft = 0; break;  /* itClimbersice_UnkMotion3_Anim */
        case 24: n = 300; ft = 0; break;  /* itClimbersice_UnkMotion3_Phys */
        case 25: n = 300; ft = 0; break;  /* itClimbersice_UnkMotion3_Coll */
        case 26: n = 300; ft = 0; break;  /* itClimbersIce_Logic90_DmgDealt */
        case 27: n = 300; ft = 0; break;  /* itClimbersIce_Logic90_Reflected */
        case 28: n = 300; ft = 0; break;  /* itClimbersIce_Logic90_Clanked */
        case 29: n = 300; ft = 0; break;  /* it_2725_Logic90_HitShield */
        case 30: n = 300; ft = 0; break;  /* itClimbersIce_Logic90_Absorbed */
        case 31: n = 300; ft = 0; break;  /* itClimbersIce_Logic90_ShieldBounced */
        case 32: n = 300; ft = 0; break;  /* itClimbersIce_Logic90_EvtUnk */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: { Item_GObj * a0 = &GO[rnd() % 4]; itClimbersIce_sub_x4(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 1: { Item_GObj * a0 = &GO[rnd() % 4]; u32 a1 = fv(); u32 a2 = fv(); itClimbersice_Spawn(a0, a1, a2); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 2: { Item_GObj * a0 = &GO[rnd() % 4]; itClimbersice_Spawn2(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 3: { Item_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; enum_t a2 = fv(); f32 a3 = rw(); ret = PI(it_802C1590(a0, a1, a2, a3)); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 4: { Item_GObj * a0 = &GO[rnd() % 4]; it_802C16F8(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 5: { Item_GObj * a0 = &GO[rnd() % 4]; it_802C17DC(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 6: { Item_GObj * a0 = &GO[rnd() % 4]; it_2725_Logic90_Destroyed(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 7: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_802C1854(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 8: { Item_GObj * a0 = &GO[rnd() % 4]; it_802C1950(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 9: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersice_UnkMotion0_Anim(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 10: { Item_GObj * a0 = &GO[rnd() % 4]; itClimbersice_UnkMotion0_Phys(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 11: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersice_UnkMotion0_Coll(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 12: { Item_GObj * a0 = &GO[rnd() % 4]; it_802C1A58(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 13: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersice_UnkMotion1_Anim(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 14: { Item_GObj * a0 = &GO[rnd() % 4]; itClimbersice_UnkMotion1_Phys(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 15: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersice_UnkMotion1_Coll(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 16: { Item_GObj * a0 = &GO[rnd() % 4]; it_802C1AE4(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 17: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersice_UnkMotion2_Anim(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 18: { Item_GObj * a0 = &GO[rnd() % 4]; itClimbersice_Phys_inline(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 19: { Item_GObj * a0 = &GO[rnd() % 4]; itClimbersice_UnkMotion2_Phys(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 20: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersice_Coll(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 21: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersice_UnkMotion2_Coll(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 22: { Item_GObj * a0 = &GO[rnd() % 4]; fn_802C1D44(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 23: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersice_UnkMotion3_Anim(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 24: { Item_GObj * a0 = &GO[rnd() % 4]; itClimbersice_UnkMotion3_Phys(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 25: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersice_UnkMotion3_Coll(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 26: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersIce_Logic90_DmgDealt(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 27: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersIce_Logic90_Reflected(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 28: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersIce_Logic90_Clanked(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 29: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_2725_Logic90_HitShield(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 30: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersIce_Logic90_Absorbed(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 31: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itClimbersIce_Logic90_ShieldBounced(a0); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            case 32: { Item_GObj * a0 = &GO[rnd() % 4]; Item_GObj * a1 = &GO[rnd() % 4]; itClimbersIce_Logic90_EvtUnk(a0, a1); { int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } } } break;
            }
            hash_world(ft);
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
