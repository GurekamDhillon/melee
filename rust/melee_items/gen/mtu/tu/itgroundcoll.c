extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/it/itgroundcoll.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
#include "world.h"
#include "itgroundcoll_gen.h"

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



static void setup_stem(void)
{
    int i;
    (void) i;
    it_804D6D28 = &ICD[0];

}

static void hash_stem(void)
{
    int i;
    (void) i;
    hashg_all();
    HM(PI(it_804D6D28));

}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    freg_all();
    freg_world();

    for (op = 0; op < 45; op++) {
        int n = 0, ft = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; ft = 0; break;  /* it_8026D564 */
        case 1: n = 300; ft = 0; break;  /* it_8026D5CC */
        case 2: n = 300; ft = 0; break;  /* it_8026D604 */
        case 3: n = 300; ft = 0; break;  /* it_8026D62C */
        case 4: n = 300; ft = 0; break;  /* it_8026D6F4 */
        case 5: n = 300; ft = 0; break;  /* it_8026D78C */
        case 6: n = 300; ft = 0; break;  /* it_8026D82C */
        case 7: n = 300; ft = 0; break;  /* it_8026D8A4 */
        case 8: n = 300; ft = 0; break;  /* it_8026D938 */
        case 9: n = 300; ft = 0; break;  /* it_8026D9A0 */
        case 10: n = 300; ft = 0; break;  /* it_8026DA08 */
        case 11: n = 300; ft = 0; break;  /* it_8026DA70 */
        case 12: n = 300; ft = 0; break;  /* it_8026DAA8 */
        case 13: n = 300; ft = 0; break;  /* it_8026DB40 */
        case 14: n = 300; ft = 0; break;  /* it_8026DBC8 */
        case 15: n = 300; ft = 0; break;  /* it_8026DC24 */
        case 16: n = 300; ft = 0; break;  /* it_8026DD5C */
        case 17: n = 300; ft = 0; break;  /* it_8026DDFC */
        case 18: n = 300; ft = 0; break;  /* it_8026DE98 */
        case 19: n = 300; ft = 0; break;  /* it_8026DF34 */
        case 20: n = 300; ft = 0; break;  /* it_8026DFB0 */
        case 21: n = 300; ft = 0; break;  /* it_8026E058 */
        case 22: n = 300; ft = 0; break;  /* it_8026E0F4 */
        case 23: n = 300; ft = 0; break;  /* it_8026E_inline */
        case 24: n = 300; ft = 0; break;  /* it_8026E15C_inline1 */
        case 25: n = 300; ft = 0; break;  /* it_8026E15C_inline2 */
        case 26: n = 300; ft = 0; break;  /* it_8026E15C */
        case 27: n = 300; ft = 0; break;  /* it_8026E248_inline */
        case 28: n = 300; ft = 0; break;  /* it_8026DE98_outline */
        case 29: n = 300; ft = 0; break;  /* it_8026DE98_outline2 */
        case 30: n = 300; ft = 0; break;  /* it_8026DE98_outline3 */
        case 31: n = 300; ft = 0; break;  /* it_8026DE98_outline4 */
        case 32: n = 300; ft = 0; break;  /* it_8026E248 */
        case 33: n = 300; ft = 0; break;  /* it_8026E32C_inline */
        case 34: n = 300; ft = 0; break;  /* it_8026E32C */
        case 35: n = 300; ft = 0; break;  /* it_8026E414 */
        case 36: n = 300; ft = 0; break;  /* it_8026E4D0 */
        case 37: n = 300; ft = 0; break;  /* it_8026E5A0 */
        case 38: n = 300; ft = 0; break;  /* it_8026E664 */
        case 39: n = 300; ft = 0; break;  /* it_8026E71C */
        case 40: n = 300; ft = 0; break;  /* it_8026E7E0 */
        case 41: n = 300; ft = 0; break;  /* it_8026E8C4 */
        case 42: n = 300; ft = 0; break;  /* it_8026E9A4 */
        case 43: n = 300; ft = 0; break;  /* it_8026EA20 */
        case 44: n = 300; ft = 0; break;  /* it_8026EA9C */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026D564(a0);  } break;
            case 1: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026D5CC(a0);  } break;
            case 2: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026D604(a0);  } break;
            case 3: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); it_8026D62C(a0, a1);  } break;
            case 4: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); it_8026D6F4(a0, a1);  } break;
            case 5: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); it_8026D78C(a0, a1);  } break;
            case 6: { Item_GObj * a0 = &GO[rnd() % 4]; it_8026D82C(a0);  } break;
            case 7: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); ret = (u32) it_8026D8A4(a0, a1);  } break;
            case 8: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); it_8026D938(a0, a1);  } break;
            case 9: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026D9A0(a0);  } break;
            case 10: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DA08(a0);  } break;
            case 11: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DA70(a0);  } break;
            case 12: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DAA8(a0);  } break;
            case 13: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DB40(a0);  } break;
            case 14: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DBC8(a0);  } break;
            case 15: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DC24(a0);  } break;
            case 16: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DD5C(a0);  } break;
            case 17: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DDFC(a0);  } break;
            case 18: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DE98(a0);  } break;
            case 19: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DF34(a0);  } break;
            case 20: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DFB0(a0);  } break;
            case 21: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026E058(a0);  } break;
            case 22: { Item_GObj * a0 = &GO[rnd() % 4]; it_8026E0F4(a0);  } break;
            case 23: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026E_inline(a0);  } break;
            case 24: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026E15C_inline1(a0);  } break;
            case 25: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026E15C_inline2(a0);  } break;
            case 26: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); it_8026E15C(a0, a1);  } break;
            case 27: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026E248_inline(a0);  } break;
            case 28: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DE98_outline(a0);  } break;
            case 29: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DE98_outline2(a0);  } break;
            case 30: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DE98_outline3(a0);  } break;
            case 31: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026DE98_outline4(a0);  } break;
            case 32: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); it_8026E248(a0, a1);  } break;
            case 33: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8026E32C_inline(a0);  } break;
            case 34: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); ret = (u32) it_8026E32C(a0, a1);  } break;
            case 35: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); it_8026E414(a0, a1);  } break;
            case 36: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); it_8026E4D0(a0, a1);  } break;
            case 37: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); it_8026E5A0(a0, a1);  } break;
            case 38: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); it_8026E664(a0, a1);  } break;
            case 39: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); it_8026E71C(a0, a1);  } break;
            case 40: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); it_8026E7E0(a0, a1);  } break;
            case 41: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); HSD_GObjEvent a2 = (HSD_GObjEvent) pick_cb(1); it_8026E8C4(a0, a1, a2);  } break;
            case 42: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; Vec3 * a2 = &VA[rnd() % 4]; Vec3 * a3 = &VA[rnd() % 4]; ret = (u32) it_8026E9A4(a0, a1, a2, a3);  } break;
            case 43: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; Vec3 * a2 = &VA[rnd() % 4]; Vec3 * a3 = &VA[rnd() % 4]; int * a4 = IA; u32 * a5 = UA; ret = (u32) it_8026EA20(a0, a1, a2, a3, a4, a5);  } break;
            case 44: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; Vec3 * a2 = &VA[rnd() % 4]; Vec3 * a3 = &VA[rnd() % 4]; int * a4 = IA; u32 * a5 = UA; s32 a6 = fv(); ret = (u32) it_8026EA9C(a0, a1, a2, a3, a4, a5, a6);  } break;
            }
            hash_world(ft);
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
