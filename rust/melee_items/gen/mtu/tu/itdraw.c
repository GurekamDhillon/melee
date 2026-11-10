extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/it/itdraw.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */
static void* COBP;
#include TU_SRC
#include "mtu_common.h"
#include "world.h"
#include "itdraw_gen.h"

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
        IT[i].xBC0 = (EF_QueuedEffect*) BOGUS;
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

static int CUR_OP = -1;
enum { OP_it_8026EB18 = 0, OP_it_8026EBC8 = 1, OP_it_8026EC54 = 2, OP_it_8026ECE0 = 3, OP_it_8026EECC_inline_1 = 4, OP_it_8026EECC_inline_2 = 5, OP_it_8026EECC_inline_3 = 6, OP_it_8026EECC_inline_0 = 7, OP_it_8026EECC_inline_sw = 8, OP_it_8026EECC = 9 };
static HSD_CObj COB; static u8 IDXA[16]; static it_266F_ItemVars IV[2];

static void setup_stem(void)
{
    int i;
    (void) i;

{ int q; COBP = &COB; zero(&COB, sizeof COB); zero(IV, sizeof IV); for (q = 0; q < 16; q++) IDXA[q] = rnd() & 3; for (q = 0; q < 12; q++) ((float*) COB.view_mtx)[q] = rw(); for (q = 0; q < 2; q++) { IV[q].x0 = fv() % 5; IV[q].x4 = IDXA; IV[q].x8 = fv() % 5; IV[q].xC = IDXA + 4; } for (q = 0; q < 4; q++) { IT[q].xAC8_hurtboxNum = rnd() % 3; if (rnd() % 3 == 0) IT[q].kind = It_Kind_Unk4; *(void**) &IT[q].xDD4_itemVar = &IV[q & 1]; } }
}

static void hash_stem(void)
{
    int i;
    (void) i;
    hashg_all();


}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    freg_all();
    freg_world();
preg(&COB, sizeof COB); preg(IDXA, sizeof IDXA); preg(IV, sizeof IV);
    for (op = 0; op < 10; op++) {
        int n = 0, ft = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; ft = 0; break;  /* it_8026EB18 */
        case 1: n = 300; ft = 0; break;  /* it_8026EBC8 */
        case 2: n = 300; ft = 0; break;  /* it_8026EC54 */
        case 3: n = 300; ft = 0; break;  /* it_8026ECE0 */
        case 4: n = 300; ft = 0; break;  /* it_8026EECC_inline_1 */
        case 5: n = 300; ft = 0; break;  /* it_8026EECC_inline_2 */
        case 6: n = 300; ft = 0; break;  /* it_8026EECC_inline_3 */
        case 7: n = 300; ft = 0; break;  /* it_8026EECC_inline_0 */
        case 8: n = 300; ft = 0; break;  /* it_8026EECC_inline_sw */
        case 9: n = 300; ft = 0; break;  /* it_8026EECC */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
            CUR_OP = op;
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: { HSD_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv(); Vec3 * a2 = &VA[rnd() % 4]; it_8026EB18(a0, a1, a2); { int q; for (q = 0; q < 4; q++) *(void**) &IT[q].xDD4_itemVar = 0; } } break;
            case 1: { HSD_GObj * a0 = &GO[rnd() % 4]; u16 a1 = (u16) (fv() % 5); u8 * a2 = IDXA; it_8026EBC8(a0, a1, a2); { int q; for (q = 0; q < 4; q++) *(void**) &IT[q].xDD4_itemVar = 0; } } break;
            case 2: { HSD_GObj * a0 = &GO[rnd() % 4]; u16 a1 = (u16) (fv() % 5); u8 * a2 = IDXA; it_8026EC54(a0, a1, a2); { int q; for (q = 0; q < 4; q++) *(void**) &IT[q].xDD4_itemVar = 0; } } break;
            case 3: { Item_GObj * a0 = &GO[rnd() % 4]; u32 a1 = fv(); ret = (u32) it_8026ECE0(a0, a1); { int q; for (q = 0; q < 4; q++) *(void**) &IT[q].xDD4_itemVar = 0; } } break;
            case 4: { HSD_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv(); Vec3 * a2 = &VA[rnd() % 4]; it_8026EECC_inline_1(a0, a1, a2); { int q; for (q = 0; q < 4; q++) *(void**) &IT[q].xDD4_itemVar = 0; } } break;
            case 5: { HSD_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv(); Vec3 * a2 = &VA[rnd() % 4]; it_8026EECC_inline_2(a0, a1, a2); { int q; for (q = 0; q < 4; q++) *(void**) &IT[q].xDD4_itemVar = 0; } } break;
            case 6: { HSD_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv(); Vec3 * a2 = &VA[rnd() % 4]; it_8026EECC_inline_3(a0, a1, a2); { int q; for (q = 0; q < 4; q++) *(void**) &IT[q].xDD4_itemVar = 0; } } break;
            case 7: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; ret = PI(it_8026EECC_inline_0(a0, a1)); { int q; for (q = 0; q < 4; q++) *(void**) &IT[q].xDD4_itemVar = 0; } } break;
            case 8: { HSD_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv(); Vec3 * a2 = &VA[rnd() % 4]; it_8026EECC_inline_sw(a0, a1, a2); { int q; for (q = 0; q < 4; q++) *(void**) &IT[q].xDD4_itemVar = 0; } } break;
            case 9: { HSD_GObj * a0 = &GO[rnd() % 4]; intptr_t a1 = fv(); it_8026EECC(a0, a1); { int q; for (q = 0; q < 4; q++) *(void**) &IT[q].xDD4_itemVar = 0; } } break;
            }
            hash_world(ft);
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
