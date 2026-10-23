extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/it/itcoll.c */
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
#include "itcoll_gen.h"

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

static ItHurtBoneList HBL[2]; static ItHurtBoneDesc HBD[2][2]; static ItCollDynamics DYN[2]; static ItCollDynamicsDesc DYD[2][2]; static struct Fighter_804D6524_t F6524;

static void setup_stem(void)
{
    int i;
    (void) i;
    HSD_GObjPLinkHead = PLINK;
    it_804D6D28 = &ICD[0];
{ int q, k; u32 r; zero(&F6524, sizeof F6524); F6524.x4 = rw(); Fighter_804D6524 = &F6524; it_804D6D18 = rnd() % 15; Item_804A0CCC.count = rnd() % 12; for (q = 0; q < 15; q++) { r = rnd(); it_804A0E70[q].x0 = (r & 1) ? 2 : 1; it_804A0E70[q].x4 = (r & 1) ? (void*) &IT[(r >> 4) % 4] : (void*) &FT[(r >> 4) % 3]; it_804A0E70[q].x8 = pick_hit(); it_804A0E70[q].xC = pick_hurt(); } for (q = 0; q < 4; q++) { IT[q].xAC4_ignoreItemID = rnd() % 3; IT[q].xAC8_hurtboxNum = rnd() % 3; IT[q].xB68 = rnd() % 3; for (k = 0; k < 4; k++) IT[q].x5D4_hitboxes[k].hit.element = rnd() % 17; } for (q = 0; q < 3; q++) { FT[q].hurt_capsules_len = rnd() % 16; for (k = 0; k < 4; k++) FT[q].x914[k].element = rnd() % 17; } zero(HBL, sizeof HBL); zero(HBD, sizeof HBD); zero(DYN, sizeof DYN); zero(DYD, sizeof DYD); for (q = 0; q < 2; q++) { HBL[q].count = rnd() % 3; HBL[q].descs = HBD[q]; DYN[q].count = rnd() % 3; DYN[q].descs = DYD[q]; for (k = 0; k < 2; k++) { HBD[q][k].bone_id = (rnd() % 4) as i32; HBD[q][k].a_offset.x = rw(); HBD[q][k].b_offset.y = rw(); HBD[q][k].scale = rw(); DYD[q][k].bone_id = rnd() % 4; DYD[q][k].offset.x = rw(); DYD[q][k].size = rw(); } ART[q].x8_hurtbones = (rnd() & 1) ? &HBL[q] : 0; ART[q].x14_dynamics = (rnd() & 1) ? (void*) &DYN[q] : 0; } }
}

static void hash_stem(void)
{
    int i;
    (void) i;
    hashg_all();
    HM(PI(HSD_GObjPLinkHead));
    HM(PI(it_804D6D28));
HM(PI(Fighter_804D6524));
}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    freg_all();
    freg_world();
preg(HBL, sizeof HBL); preg(HBD, sizeof HBD); preg(DYN, sizeof DYN); preg(DYD, sizeof DYD);
    for (op = 0; op < 38; op++) {
        int n = 0, ft = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; ft = 0; break;  /* itColl_chkECBOverlap */
        case 1: n = 300; ft = 0; break;  /* it_8026F9A0 */
        case 2: n = 300; ft = 0; break;  /* it_8026F9AC */
        case 3: n = 300; ft = 0; break;  /* it_8026FA2C */
        case 4: n = 300; ft = 0; break;  /* it_8026FAC4 */
        case 5: n = 300; ft = 0; break;  /* it_8026FC00_inline */
        case 6: n = 300; ft = 0; break;  /* it_8026FC00 */
        case 7: n = 300; ft = 0; break;  /* it_8026FCF8 */
        case 8: n = 300; ft = 0; break;  /* it_8026FE68 */
        case 9: n = 300; ft = 0; break;  /* it_8026FAC4_noinline */
        case 10: n = 300; ft = 0; break;  /* it_802701BC */
        case 11: n = 300; ft = 0; break;  /* it_8026F9AC_noinline */
        case 12: n = 300; ft = 1; break;  /* it_802703E8 */
        case 13: n = 300; ft = 0; break;  /* it_802706D0_sub3 */
        case 14: n = 300; ft = 0; break;  /* it_802706D0 */
        case 15: n = 300; ft = 0; break;  /* it_80270CD8 */
        case 16: n = 300; ft = 0; break;  /* it_80270E30 */
        case 17: n = 300; ft = 0; break;  /* it_8027129C */
        case 18: n = 300; ft = 0; break;  /* it_8027137C */
        case 19: n = 300; ft = 0; break;  /* it_8027146C */
        case 20: n = 300; ft = 0; break;  /* it_802714C0 */
        case 21: n = 300; ft = 0; break;  /* it_80271508 */
        case 22: n = 300; ft = 0; break;  /* it_80271534 */
        case 23: n = 300; ft = 0; break;  /* it_80271590 */
        case 24: n = 300; ft = 0; break;  /* it_8027163C */
        case 25: n = 300; ft = 0; break;  /* it_80271830 */
        case 26: n = 300; ft = 0; break;  /* it_80271A58 */
        case 27: n = 300; ft = 0; break;  /* it_80271B60 */
        case 28: n = 300; ft = 0; break;  /* it_80271D2C */
        case 29: n = 300; ft = 0; break;  /* it_80271F78 */
        case 30: n = 300; ft = 0; break;  /* it_802721B8 */
        case 31: n = 300; ft = 0; break;  /* it_80272280 */
        case 32: n = 300; ft = 0; break;  /* it_80272298 */
        case 33: n = 300; ft = 0; break;  /* it_802722B0 */
        case 34: n = 300; ft = 0; break;  /* it_80272304 */
        case 35: n = 300; ft = 0; break;  /* it_8027236C */
        case 36: n = 300; ft = 0; break;  /* it_802723FC */
        case 37: n = 300; ft = 0; break;  /* it_80272460 */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: { f32 a0 = rw(); f32 a1 = rw(); itECB * a2 = &EC[rnd() % 3]; itECB * a3 = &EC[rnd() % 3]; Vec3 * a4 = &VA[rnd() % 4]; ret = (u32) itColl_chkECBOverlap(a0, a1, a2, a3, a4);  } break;
            case 1: {  it_8026F9A0();  } break;
            case 2: { s32 a0 = fv(); void * a1 = (void*) &FT[rnd() % 3]; HitCapsule * a2 = pick_hit(); Item * a3 = &IT[rnd() % 4]; HurtCapsule * a4 = pick_hurt(); it_8026F9AC(a0, a1, a2, a3, a4);  } break;
            case 3: { Item * a0 = &IT[rnd() % 4]; HitCapsule * a1 = pick_hit(); s32 a2 = fv(); Item * a3 = &IT[rnd() % 4]; bool a4 = fv(); it_8026FA2C(a0, a1, a2, a3, a4);  } break;
            case 4: { Item * a0 = &IT[rnd() % 4]; HitCapsule * a1 = pick_hit(); s32 a2 = fv(); void * a3 = (void*) &IT[rnd() % 4]; bool a4 = fv(); it_8026FAC4(a0, a1, a2, a3, a4);  } break;
            case 5: { Item * a0 = &IT[rnd() % 4]; HitCapsule * a1 = pick_hit(); int a2 = fv(); Fighter * a3 = &FT[rnd() % 3]; it_8026FC00_inline(a0, a1, a2, a3);  } break;
            case 6: { Item * a0 = &IT[rnd() % 4]; HitCapsule * a1 = pick_hit(); s32 a2 = fv(); Fighter * a3 = &FT[rnd() % 3]; it_8026FC00(a0, a1, a2, a3);  } break;
            case 7: { Item * a0 = &IT[rnd() % 4]; HitCapsule * a1 = pick_hit(); it_8026FCF8(a0, a1);  } break;
            case 8: { Item * a0 = &IT[rnd() % 4]; HitCapsule * a1 = pick_hit(); Item * a2 = &IT[rnd() % 4]; HitCapsule * a3 = pick_hit(); it_8026FE68(a0, a1, a2, a3);  } break;
            case 9: { Item * a0 = &IT[rnd() % 4]; HitCapsule * a1 = pick_hit(); s32 a2 = fv(); void * a3 = (void*) &IT[rnd() % 4]; bool a4 = fv(); it_8026FAC4_noinline(a0, a1, a2, a3, a4);  } break;
            case 10: { Item_GObj * a0 = &GO[rnd() % 4]; it_802701BC(a0);  } break;
            case 11: { s32 a0 = fv(); void * a1 = (void*) &FT[rnd() % 3]; HitCapsule * a2 = pick_hit(); Item * a3 = &IT[rnd() % 4]; HurtCapsule * a4 = pick_hurt(); it_8026F9AC_noinline(a0, a1, a2, a3, a4);  } break;
            case 12: { Item_GObj * a0 = &GO[rnd() % 4]; it_802703E8(a0);  } break;
            case 13: { Item * a0 = &IT[rnd() % 4]; Item * a1 = &IT[rnd() % 4]; HitCapsule * a2 = pick_hit(); HurtCapsule * a3 = pick_hurt(); it_802706D0_sub3(a0, a1, a2, a3);  } break;
            case 14: { Item_GObj * a0 = &GO[rnd() % 4]; it_802706D0(a0);  } break;
            case 15: { Item * a0 = &IT[rnd() % 4]; HitCapsule * a1 = pick_hit(); ret = B(it_80270CD8(a0, a1));  } break;
            case 16: { Item_GObj * a0 = &GO[rnd() % 4]; it_80270E30(a0);  } break;
            case 17: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv() & 3; it_8027129C(a0, a1);  } break;
            case 18: { Item_GObj * a0 = &GO[rnd() % 4]; it_8027137C(a0);  } break;
            case 19: { Item_GObj * a0 = &GO[rnd() % 4]; it_8027146C(a0);  } break;
            case 20: { Item_GObj * a0 = &GO[rnd() % 4]; it_802714C0(a0);  } break;
            case 21: { Item_GObj * a0 = &GO[rnd() % 4]; HurtCapsuleState a1 = fv(); it_80271508(a0, a1);  } break;
            case 22: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv() & 1; HurtCapsule * a2 = pick_hurt(); it_80271534(a0, a1, a2);  } break;
            case 23: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv() & 1; HurtCapsule * a2 = pick_hurt(); it_80271590(a0, a1, a2);  } break;
            case 24: { Item_GObj * a0 = &GO[rnd() % 4]; it_8027163C(a0);  } break;
            case 25: { Item * a0 = &IT[rnd() % 4]; f32 a1 = (float) ((int) (fv() % 200) - 100) * 0.37f; it_80271830(a0, a1);  } break;
            case 26: { Item_GObj * a0 = &GO[rnd() % 4]; it_80271A58(a0);  } break;
            case 27: { Item_GObj * a0 = &GO[rnd() % 4]; it_80271B60(a0);  } break;
            case 28: { Item_GObj * a0 = &GO[rnd() % 4]; it_80271D2C(a0);  } break;
            case 29: { Item_GObj * a0 = &GO[rnd() % 4]; it_80271F78(a0);  } break;
            case 30: { Item_GObj * a0 = &GO[rnd() % 4]; it_802721B8(a0);  } break;
            case 31: { Item_GObj * a0 = &GO[rnd() % 4]; it_80272280(a0);  } break;
            case 32: { Item_GObj * a0 = &GO[rnd() % 4]; it_80272298(a0);  } break;
            case 33: { Item_GObj * a0 = &GO[rnd() % 4]; it_802722B0(a0);  } break;
            case 34: { Item_GObj * a0 = &GO[rnd() % 4]; it_80272304(a0);  } break;
            case 35: { Item_GObj * a0 = &GO[rnd() % 4]; ret = PI(it_8027236C(a0));  } break;
            case 36: { Item_GObj * a0 = &GO[rnd() % 4]; ret = PI(it_802723FC(a0));  } break;
            case 37: { HitCapsule * a0 = pick_hit(); u32 a1 = fv(); Item_GObj * a2 = &GO[rnd() % 4]; it_80272460(a0, a1, a2);  } break;
            }
            hash_world(ft);
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
