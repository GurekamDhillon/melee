extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/it/it_2725.c */
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
#include "it_2725_gen.h"

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

static void* DAT[2][2];

static void setup_stem(void)
{
    int i;
    (void) i;
    HSD_GObjPLinkHead = PLINK;
    it_804D6D28 = &ICD[0];
{ int q; setup_xpools(); for (q = 0; q < 2; q++) { DAT[q][0] = &FT[rnd() % 3]; DAT[q][1] = 0; ART[q].x4_specialAttributes = (void*) DAT[q]; ART[q].xC_itemStates = &STA[q]; ART[q].x10_modelDesc = &MD[q]; MD[q].x4_bone_count = (rnd() % 3 == 0) ? 1 : 0; MD[q].x8_bone_attach_id = MD[q].x4_bone_count ? (s32) (rnd() % 4) : (s32) (rnd() % 2); MD[q].xC_bit_field = (u8) (((rnd() % 3 == 0) ? 1 : 0) << 6 | (rnd() & 0x3F)); } it_804D6D28->x44_float = (s32) (1 + fv() % 7); for (q = 0; q < 8; q++) JO[q].flags &= ~JOBJ_USE_QUATERNION; }
}

static void hash_stem(void)
{
    int i;
    (void) i;
    hashg_all();
    HM(PI(HSD_GObjPLinkHead));
    HM(PI(it_804D6D28));
hash_xpools();
}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    freg_all();
    freg_world();
preg_xpools(); preg(DAT, sizeof DAT);
    for (op = 0; op < 96; op++) {
        int n = 0, ft = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; ft = 0; break;  /* it_80272560 */
        case 1: n = 300; ft = 0; break;  /* it_802725D4 */
        case 2: n = 300; ft = 0; break;  /* it_80272674 */
        case 3: n = 300; ft = 0; break;  /* it_80272784 */
        case 4: n = 300; ft = 0; break;  /* it_80272818 */
        case 5: n = 300; ft = 0; break;  /* it_80272828 */
        case 6: n = 300; ft = 0; break;  /* it_80272860 */
        case 7: n = 300; ft = 0; break;  /* it_802728C8 */
        case 8: n = 300; ft = 0; break;  /* it_80272940 */
        case 9: n = 300; ft = 0; break;  /* it_80272980 */
        case 10: n = 300; ft = 0; break;  /* it_80272A18 */
        case 11: n = 300; ft = 0; break;  /* it_80272A3C */
        case 12: n = 300; ft = 0; break;  /* it_80272A60 */
        case 13: n = 300; ft = 0; break;  /* it_80272AC4 */
        case 14: n = 300; ft = 0; break;  /* it_80272B40 */
        case 15: n = 300; ft = 0; break;  /* it_80272BA4 */
        case 16: n = 300; ft = 0; break;  /* it_80272C08 */
        case 17: n = 300; ft = 0; break;  /* it_80272C6C */
        case 18: n = 300; ft = 0; break;  /* it_80272C90 */
        case 19: n = 300; ft = 0; break;  /* it_80272CC0 */
        case 20: n = 300; ft = 0; break;  /* it_80272D1C */
        case 21: n = 300; ft = 0; break;  /* it_80272D40 */
        case 22: n = 300; ft = 0; break;  /* itColl_BounceOffVictim */
        case 23: n = 300; ft = 0; break;  /* it_80272DE4 */
        case 24: n = 300; ft = 0; break;  /* it_80272F7C */
        case 25: n = 300; ft = 0; break;  /* it_80273030 */
        case 26: n = 300; ft = 0; break;  /* itColl_BounceOffShield */
        case 27: n = 300; ft = 0; break;  /* it_80273130 */
        case 28: n = 300; ft = 0; break;  /* it_80273168 */
        case 29: n = 300; ft = 0; break;  /* it_802731A4 */
        case 30: n = 300; ft = 0; break;  /* it_802731E0 */
        case 31: n = 300; ft = 0; break;  /* it_8027321C */
        case 32: n = 300; ft = 0; break;  /* it_8027327C */
        case 33: n = 300; ft = 0; break;  /* it_802732E4 */
        case 34: n = 300; ft = 0; break;  /* it_80273318 */
        case 35: n = 300; ft = 0; break;  /* it_80273408 */
        case 36: n = 300; ft = 0; break;  /* it_80273454 */
        case 37: n = 300; ft = 0; break;  /* it_8027346C */
        case 38: n = 300; ft = 0; break;  /* it_80273484 */
        case 39: n = 300; ft = 0; break;  /* it_8027349C */
        case 40: n = 300; ft = 0; break;  /* it_802734B4 */
        case 41: n = 300; ft = 0; break;  /* it_80273500 */
        case 42: n = 300; ft = 0; break;  /* it_80273598 */
        case 43: n = 300; ft = 0; break;  /* it_80273600 */
        case 44: n = 300; ft = 0; break;  /* it_80273648 */
        case 45: n = 300; ft = 0; break;  /* it_80273670 */
        case 46: n = 300; ft = 0; break;  /* it_80273748 */
        case 47: n = 300; ft = 0; break;  /* it_80273B50 */
        case 48: n = 300; ft = 0; break;  /* it_80273F34 */
        case 49: n = 300; ft = 0; break;  /* it_80274198 */
        case 50: n = 300; ft = 0; break;  /* it_802741F4 */
        case 51: n = 300; ft = 0; break;  /* it_80274250 */
        case 52: n = 300; ft = 0; break;  /* it_8027429C */
        case 53: n = 300; ft = 0; break;  /* it_802742F4 */
        case 54: n = 300; ft = 0; break;  /* it_80274484 */
        case 55: n = 300; ft = 0; break;  /* it_80274574 */
        case 56: n = 300; ft = 0; break;  /* it_80274594 */
        case 57: n = 300; ft = 0; break;  /* it_80274658 */
        case 58: n = 300; ft = 0; break;  /* it_802746F8 */
        case 59: n = 300; ft = 0; break;  /* it_80274740 */
        case 60: n = 300; ft = 0; break;  /* it_80274990 */
        case 61: n = 300; ft = 0; break;  /* it_80274A64 */
        case 62: n = 300; ft = 0; break;  /* it_80274C60 */
        case 63: n = 300; ft = 0; break;  /* it_80274C78 */
        case 64: n = 300; ft = 0; break;  /* it_80274C88 */
        case 65: n = 300; ft = 0; break;  /* it_80274CAC */
        case 66: n = 300; ft = 0; break;  /* it_80274D04 */
        case 67: n = 300; ft = 0; break;  /* it_80274D6C */
        case 68: n = 300; ft = 0; break;  /* it_80274DAC */
        case 69: n = 300; ft = 0; break;  /* it_80274DFC */
        case 70: n = 300; ft = 0; break;  /* it_80274E44 */
        case 71: n = 300; ft = 0; break;  /* it_80274ECC */
        case 72: n = 300; ft = 0; break;  /* it_80274ED8 */
        case 73: n = 300; ft = 0; break;  /* it_80274EE8 */
        case 74: n = 300; ft = 0; break;  /* it_80274EF8 */
        case 75: n = 300; ft = 0; break;  /* it_80274F10 */
        case 76: n = 300; ft = 0; break;  /* it_80274F28 */
        case 77: n = 300; ft = 0; break;  /* it_80274F48 */
        case 78: n = 300; ft = 0; break;  /* it_80274FDC */
        case 79: n = 300; ft = 0; break;  /* it_80275070 */
        case 80: n = 300; ft = 0; break;  /* it_802750E8 */
        case 81: n = 300; ft = 0; break;  /* it_802750F8 */
        case 82: n = 300; ft = 0; break;  /* it_80275158 */
        case 83: n = 300; ft = 0; break;  /* it_80275174 */
        case 84: n = 300; ft = 0; break;  /* it_8027518C */
        case 85: n = 300; ft = 0; break;  /* it_802751D8 */
        case 86: n = 300; ft = 0; break;  /* it_80275210 */
        case 87: n = 300; ft = 0; break;  /* it_80275228 */
        case 88: n = 300; ft = 0; break;  /* it_80275240 */
        case 89: n = 300; ft = 0; break;  /* it_80275258 */
        case 90: n = 300; ft = 0; break;  /* it_80275270 */
        case 91: n = 300; ft = 0; break;  /* it_80275288 */
        case 92: n = 300; ft = 0; break;  /* it_802752D8 */
        case 93: n = 300; ft = 0; break;  /* it_80275328 */
        case 94: n = 300; ft = 0; break;  /* it_80275390 */
        case 95: n = 300; ft = 0; break;  /* it_802753BC */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = (s32) (fv() % 4); it_80272560(a0, a1);  } break;
            case 1: { Item_GObj * a0 = &GO[rnd() % 4]; it_802725D4(a0);  } break;
            case 2: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = (s32) (fv() % 4); it_80272674(a0, a1);  } break;
            case 3: { Item_GObj * a0 = &GO[rnd() % 4]; it_80272784(a0);  } break;
            case 4: { Item * a0 = &IT[rnd() % 4]; ret = PI(it_80272818(a0));  } break;
            case 5: { ItemKind a0 = fv(); ret = PI(it_80272828(a0));  } break;
            case 6: { Item_GObj * a0 = &GO[rnd() % 4]; f32 a1 = rw(); f32 a2 = rw(); it_80272860(a0, a1, a2);  } break;
            case 7: { Item_GObj * a0 = &GO[rnd() % 4]; it_802728C8(a0);  } break;
            case 8: { Item_GObj * a0 = &GO[rnd() % 4]; it_80272940(a0);  } break;
            case 9: { Item_GObj * a0 = &GO[rnd() % 4]; it_80272980(a0);  } break;
            case 10: { HSD_JObj * a0 = &JO[rnd() % 8]; it_80272A18(a0);  } break;
            case 11: { HSD_JObj * a0 = &JO[rnd() % 8]; it_80272A3C(a0);  } break;
            case 12: { Item_GObj * a0 = &GO[rnd() % 4]; it_80272A60(a0);  } break;
            case 13: { Item_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_80272AC4(a0, a1);  } break;
            case 14: { Item_GObj * a0 = &GO[rnd() % 4]; it_80272B40(a0);  } break;
            case 15: { Item_GObj * a0 = &GO[rnd() % 4]; it_80272BA4(a0);  } break;
            case 16: { Item_GObj * a0 = &GO[rnd() % 4]; it_80272C08(a0);  } break;
            case 17: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_80272C6C(a0);  } break;
            case 18: { Item_GObj * a0 = &GO[rnd() % 4]; ret = PI(it_80272C90(a0));  } break;
            case 19: { Item_GObj * a0 = &GO[rnd() % 4]; enum_t a1 = (enum_t) (fv() % 3); ret = PI(it_80272CC0(a0, a1));  } break;
            case 20: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_80272D1C(a0);  } break;
            case 21: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_80272D40(a0);  } break;
            case 22: { Item_GObj * a0 = &GO[rnd() % 4]; itColl_BounceOffVictim(a0);  } break;
            case 23: { HSD_JObj * a0 = &JO[rnd() % 8]; f32 a1 = rw(); it_80272DE4(a0, a1);  } break;
            case 24: { HSD_JObj * a0 = &JO[rnd() % 8]; f32 a1 = rw(); it_80272F7C(a0, a1);  } break;
            case 25: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_80273030(a0);  } break;
            case 26: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itColl_BounceOffShield(a0);  } break;
            case 27: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_80273130(a0);  } break;
            case 28: { Item_GObj * a0 = &GO[rnd() % 4]; it_80273168(a0);  } break;
            case 29: { Item_GObj * a0 = &GO[rnd() % 4]; it_802731A4(a0);  } break;
            case 30: { Item_GObj * a0 = &GO[rnd() % 4]; it_802731E0(a0);  } break;
            case 31: { Item_GObj * a0 = &GO[rnd() % 4]; it_8027321C(a0);  } break;
            case 32: { Item_GObj * a0 = &GO[rnd() % 4]; enum_t a1 = fv(); enum_t a2 = fv(); it_8027327C(a0, a1, a2);  } break;
            case 33: { Item * a0 = &IT[rnd() % 4]; s32 a1 = fv(); it_802732E4(a0, a1);  } break;
            case 34: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_Joint * a1 = &JT[rnd() % 2]; it_80273318(a0, a1);  } break;
            case 35: { Item_GObj * a0 = &GO[rnd() % 4]; it_80273408(a0);  } break;
            case 36: { Item_GObj * a0 = &GO[rnd() % 4]; it_80273454(a0);  } break;
            case 37: { Item_GObj * a0 = &GO[rnd() % 4]; it_8027346C(a0);  } break;
            case 38: { Item_GObj * a0 = &GO[rnd() % 4]; it_80273484(a0);  } break;
            case 39: { Item_GObj * a0 = &GO[rnd() % 4]; it_8027349C(a0);  } break;
            case 40: { Item_GObj * a0 = &GO[rnd() % 4]; it_802734B4(a0);  } break;
            case 41: { Item_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_80273500(a0, a1);  } break;
            case 42: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv(); s32 a2 = fv(); it_80273598(a0, a1, a2);  } break;
            case 43: { Item_GObj * a0 = &GO[rnd() % 4]; it_80273600(a0);  } break;
            case 44: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv(); s32 a2 = fv(); it_80273648(a0, a1, a2);  } break;
            case 45: { Item_GObj * a0 = &GO[rnd() % 4]; int a1 = (int) (fv() % 8); f32 a2 = rw(); it_80273670(a0, a1, a2);  } break;
            case 46: { Item_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; Vec3 * a2 = &VA[rnd() % 4]; it_80273748(a0, a1, a2);  } break;
            case 47: { Item_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_80273B50(a0, a1);  } break;
            case 48: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObj * a1 = &GO[rnd() % 4]; it_80273F34(a0, a1);  } break;
            case 49: { Item_GObj * a0 = &GO[rnd() % 4]; bool a1 = fv(); it_80274198(a0, a1);  } break;
            case 50: { Item_GObj * a0 = &GO[rnd() % 4]; bool a1 = fv(); it_802741F4(a0, a1);  } break;
            case 51: { Item_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_80274250(a0, a1);  } break;
            case 52: { Item_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; it_8027429C(a0, a1);  } break;
            case 53: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObj * a1 = &GO[rnd() % 4]; Fighter_Part a2 = fv(); it_802742F4(a0, a1, a2);  } break;
            case 54: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_JObj * a1 = &JO[rnd() % 8]; f32 a2 = rw(); it_80274484(a0, a1, a2);  } break;
            case 55: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274574(a0);  } break;
            case 56: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274594(a0);  } break;
            case 57: { Item_GObj * a0 = &GO[rnd() % 4]; f32 a1 = rw(); ret = (u32) it_80274658(a0, a1);  } break;
            case 58: { Item_GObj * a0 = &GO[rnd() % 4]; ret = PI(it_802746F8(a0));  } break;
            case 59: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274740(a0);  } break;
            case 60: { Item_GObj * a0 = &GO[rnd() % 4]; ret = B(it_80274990(a0));  } break;
            case 61: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274A64(a0);  } break;
            case 62: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274C60(a0);  } break;
            case 63: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_80274C78(a0);  } break;
            case 64: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274C88(a0);  } break;
            case 65: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274CAC(a0);  } break;
            case 66: { Item_GObj * a0 = &GO[rnd() % 4]; struct lb_80014638_arg1_t * a1 = &LBA[0]; it_80274D04(a0, a1);  } break;
            case 67: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274D6C(a0);  } break;
            case 68: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274DAC(a0);  } break;
            case 69: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274DFC(a0);  } break;
            case 70: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274E44(a0);  } break;
            case 71: { Item_GObj * a0 = &GO[rnd() % 4]; bool a1 = fv(); it_80274ECC(a0, a1);  } break;
            case 72: {  it_80274ED8();  } break;
            case 73: {  it_80274EE8();  } break;
            case 74: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274EF8(a0);  } break;
            case 75: { Item_GObj * a0 = &GO[rnd() % 4]; it_80274F10(a0);  } break;
            case 76: { Item * a0 = &IT[rnd() % 4]; s8 a1 = (s8) fv(); HSD_GObjEvent a2 = (HSD_GObjEvent) pick_cb(1); HSD_GObjInteraction a3 = (HSD_GObjInteraction) pick_cb(3); it_80274F28(a0, a1, a2, a3);  } break;
            case 77: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = (s32) (fv() % 4); HSD_GObj * a2 = &GO[rnd() % 4]; Fighter_Part a3 = fv(); it_80274F48(a0, a1, a2, a3);  } break;
            case 78: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = (s32) (fv() % 4); Fighter_GObj * a2 = &GO[rnd() % 4]; Fighter_Part a3 = fv(); it_80274FDC(a0, a1, a2, a3);  } break;
            case 79: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = (s32) (fv() % 4); it_80275070(a0, a1);  } break;
            case 80: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv(); ret = (u32) it_802750E8(a0, a1);  } break;
            case 81: { Item_GObj * a0 = &GO[rnd() % 4]; it_802750F8(a0);  } break;
            case 82: { Item_GObj * a0 = &GO[rnd() % 4]; f32 a1 = rw(); it_80275158(a0, a1);  } break;
            case 83: { Item_GObj * a0 = &GO[rnd() % 4]; f32 a1 = rw(); it_80275174(a0, a1);  } break;
            case 84: { Item_GObj * a0 = &GO[rnd() % 4]; it_8027518C(a0);  } break;
            case 85: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_802751D8(a0);  } break;
            case 86: { Item_GObj * a0 = &GO[rnd() % 4]; it_80275210(a0);  } break;
            case 87: { Item_GObj * a0 = &GO[rnd() % 4]; it_80275228(a0);  } break;
            case 88: { Item_GObj * a0 = &GO[rnd() % 4]; it_80275240(a0);  } break;
            case 89: { Item_GObj * a0 = &GO[rnd() % 4]; it_80275258(a0);  } break;
            case 90: { Item_GObj * a0 = &GO[rnd() % 4]; it_80275270(a0);  } break;
            case 91: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv(); s32 a2 = fv(); it_80275288(a0, a1, a2);  } break;
            case 92: { Item_GObj * a0 = &GO[rnd() % 4]; s32 a1 = fv(); s32 a2 = fv(); it_802752D8(a0, a1, a2);  } break;
            case 93: { HSD_GObj * a0 = &GO[rnd() % 4]; ItemLink * a1 = &LK[0]; it_80275328(a0, a1);  } break;
            case 94: { Item_GObj * a0 = &GO[rnd() % 4]; it_80275390(a0);  } break;
            case 95: { Item_GObj * a0 = &GO[rnd() % 4]; s16 a1 = (s16) fv(); it_802753BC(a0, a1);  } break;
            }
            hash_world(ft);
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
