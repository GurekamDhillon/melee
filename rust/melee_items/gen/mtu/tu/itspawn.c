extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/it/itspawn.c */
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
static int CNT[64]; static int GMV; static unsigned long long FM64; static unsigned char AB[2048]; static unsigned AO;
#include TU_SRC
#include "mtu_common.h"
#include "world.h"
#include "itspawn_gen.h"

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

static ItemPickTable TB[2]; static u8 TBK[2][8]; static u16 TBW[2][8]; static struct it_8026C47C_arg0_t B47C;

static void setup_stem(void)
{
    int i;
    (void) i;
    it_804D6D28 = &ICD[0];
{ int q, k; u32 cum; for (q = 0; q < 64; q++) CNT[q] = (rnd() & 3) == 0 ? 0 : (s32) (fv() % 40); GMV = (rnd() % 8 == 0) ? -1 : (s32) (rnd() % 5); FM64 = (rnd() % 6 == 0) ? 0 : fv64(); AO = 0; zero(AB, sizeof AB); zero(&B47C, sizeof B47C); zero(&it_804A0E30, sizeof it_804A0E30); zero(&it_804A0E50, sizeof it_804A0E50); zero(&it_804A0E60, sizeof it_804A0E60); it_804A0E30.x0 = (s32) (fv() % 4); it_804A0E30.x18 = FM64; for (k = 0; k < 2; k++) { u32 r = rnd(); zero(&TB[k], sizeof TB[k]); TB[k].size = 1 + (r % 7); TB[k].x4 = TBK[k]; TB[k].xC = TBW[k]; TB[k].x8 = ((r >> 8) % 5 == 0) ? 0 : (u16) (1 + (r >> 12) % 60); cum = 0; for (q = 0; q < 8; q++) { TBK[k][q] = (rnd() % 4 == 0) ? 0x22 : (u8) (rnd() % 40); TBW[k][q] = (u16) cum; cum += rnd() % 12; } } it_804D6D28->x128[0] = (s32) (fv() % 20); }
}

static void hash_stem(void)
{
    int i;
    (void) i;
    hashg_all();
    HM(PI(it_804D6D28));
{ int q; hash_RandomItemSpawner(&it_804A0E30); hash_ItemPickTable(&it_804A0E50); hash_ItemPickTable(&it_804A0E60); hash_ItemPickTable(&TB[0]); hash_ItemPickTable(&TB[1]); for (q = 0; q < 64; q++) HM((u32) CNT[q]); HM((u32) GMV); HM64((double) 0); HM((u32) FM64); HM((u32) (FM64 >> 32)); HM(AO); HM(RH(AB, sizeof AB)); HM(RH(TBK, sizeof TBK)); HM(RH(TBW, sizeof TBW)); HM(RH(&B47C, sizeof B47C)); }
}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    freg_all();
    freg_world();
preg(CNT, sizeof CNT); preg(AB, sizeof AB); preg(TBK, sizeof TBK); preg(TBW, sizeof TBW);
    for (op = 0; op < 14; op++) {
        int n = 0, ft = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; ft = 0; break;  /* it_8026C47C */
        case 1: n = 300; ft = 0; break;  /* it_8026C65C */
        case 2: n = 300; ft = 0; break;  /* it_8026C704 */
        case 3: n = 300; ft = 0; break;  /* it_8026C75C */
        case 4: n = 300; ft = 0; break;  /* fn_8026C88C */
        case 5: n = 300; ft = 0; break;  /* it_8026CA4C */
        case 6: n = 300; ft = 0; break;  /* it_8026CB3C */
        case 7: n = 300; ft = 0; break;  /* it_8026CB9C */
        case 8: n = 300; ft = 0; break;  /* it_8026CD50 */
        case 9: n = 300; ft = 0; break;  /* it_8026CF04 */
        case 10: n = 300; ft = 0; break;  /* it_8026D018 */
        case 11: n = 300; ft = 0; break;  /* it_8026D258 */
        case 12: n = 300; ft = 0; break;  /* it_8026D324 */
        case 13: n = 300; ft = 0; break;  /* it_8026D3CC */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: { struct it_8026C47C_arg0_t * a0 = &B47C; it_8026C47C(a0);  } break;
            case 1: { ItemPickTable * a0 = &TB[0]; ret = (u32) it_8026C65C(a0);  } break;
            case 2: {  ret = (u32) it_8026C704();  } break;
            case 3: { ItemPickTable * a0 = &TB[rnd() & 1]; ret = (u32) it_8026C75C(a0);  } break;
            case 4: { HSD_GObj * a0 = (GMV = (s32) (rnd() % 5), &GO[rnd() % 4]); fn_8026C88C(a0);  } break;
            case 5: { ItemPickTable * a0 = &TB[rnd() & 1]; s32 * a1 = CNT; u64 a2 = fv64(); s32 a3 = (s32) (fv() % 12); f32 a4 = rw(); it_8026CA4C(a0, a1, a2, a3, a4);  } break;
            case 6: { Vec3 * a0 = &VA[rnd() % 4]; ret = (u32) it_8026CB3C(a0);  } break;
            case 7: { s32 * a0 = CNT; u64 a1 = fv64(); f32 a2 = rw(); it_8026CB9C(a0, a1, a2);  } break;
            case 8: { s32 * a0 = CNT; u64 a1 = fv64(); f32 a2 = rw(); it_8026CD50(a0, a1, a2);  } break;
            case 9: {  it_8026CF04();  } break;
            case 10: {  it_8026D018();  } break;
            case 11: { Vec3 * a0 = &VA[rnd() % 4]; ItemKind a1 = fv(); ret = (u32) it_8026D258(a0, a1);  } break;
            case 12: { ItemKind a0 = fv(); ret = (u32) it_8026D324(a0);  } break;
            case 13: {  ret = (u32) it_8026D3CC();  } break;
            }
            hash_world(ft);
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
