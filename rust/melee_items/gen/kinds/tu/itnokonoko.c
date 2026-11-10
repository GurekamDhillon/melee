/* TU: src/melee/it/kinds/itnokonoko.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/kinds/mk_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
#include "kinds_common.h"
static HSD_GObj GO[3];
static Item IT[3];
static HSD_JObj JO[48];
static DynamicBoneTable DBT[3];
static Article ART[3];
static ItemAttr ATTR[3];
static ItemCommonData ICD;
static unsigned char SPEC[3][128] __attribute__((aligned(16)));
static Vec3 VA[4];


#include "itnokonoko_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itNokoNoko_DatAttrs* a = (itNokoNoko_DatAttrs*) SPEC[0];
        a->x0 = (void*) (SPEC[0] + 64); ((int*) (SPEC[0] + 64))[0] = (int) (rnd() % 30); ((float*) (SPEC[0] + 64))[1] = rw();
        a->x4 = rw(); a->x8 = rw();
    } break;
    }
}
static void setup(int prof)
{
    int i, k;
    (void) i; (void) k;
    zero(GO, sizeof GO); zero(IT, sizeof IT); zero(JO, sizeof JO); zero(DBT, sizeof DBT); zero(ART, sizeof ART);
    zero(ATTR, sizeof ATTR); zero(&ICD, sizeof ICD); zero(SPEC, sizeof SPEC); zero(VA, sizeof VA);
    for (k = 0; k < 3; k++) { fill_HSD_GObj(&GO[k]); fill_Item(&IT[k]); fill_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) { fill_HSD_JObj(&JO[i]); JO[i].flags &= ~JOBJ_USE_QUATERNION; }
    fill_ItemCommonData(&ICD);
    for (i = 0; i < 4; i++) { VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (k = 0; k < 3; k++) {
        GO[k].user_data = &IT[k];
        GO[k].hsd_obj = &JO[k];
        JO[k].child = &JO[3 + k];
        IT[k].entity = &GO[k];
        IT[k].xBBC_dynamicBoneTable = &DBT[k];
        IT[k].xC4_article_data = &ART[k];
        IT[k].xCC_item_attr = &ATTR[k];
        ART[k].x0_common_attr = &ATTR[k];
        ART[k].x4_specialAttributes = (union ItemSpecialAttributes*) SPEC[k];
        for (i = 0; i < 14; i++) DBT[k].bones[i] = &JO[6 + 14 * k + i];
    }

    IT[0].owner = (rnd() & 3) ? &GO[1] : 0;
    IT[0].xC64_reflectGObj = (rnd() & 3) ? &GO[1] : 0;
    if (rnd() & 1) IT[0].msid = (int) (rnd() % 14);
    if (rnd() & 1) IT[0].facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
    IT[0].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
    spec_fill(prof);
    for (k = 0; k < 3; k++) {
        IT[k].xDD4_itemVar.nokonoko.x20 = (int) (rnd() % 4) - 1;
        IT[k].xDD4_itemVar.nokonoko.x28 = (int) (rnd() % 4) - 1;
        IT[k].xDD4_itemVar.nokonoko.x34 = rw();
        IT[k].xDD4_itemVar.nokonoko.x38 = (rnd() & 1) ? 1.0f : -1.0f;
        IT[k].xDD4_itemVar.nokonoko.x3C = (int) (rnd() % 8);
        IT[k].xDD4_itemVar.nokonoko.x40 = (int) (rnd() % 4);
        IT[k].xDD4_itemVar.nokonoko.x4 = (int) (rnd() % 4);
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;

    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 48; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_802DC69C(&GO[0]); } break; /* it_802DC69C */
            case 1: {  ret = (u32) ((itNokonoko_Logic3_DmgReceived(&GO[0])) != 0); } break; /* itNokonoko_Logic3_DmgReceived */
            case 2: {  ret = (u32) ((it_802DC83C(&GO[0])) != 0); } break; /* it_802DC83C */
            case 3: {  fn_802DC8B8(&GO[0]); } break; /* fn_802DC8B8 */
            case 4: {  ret = (u32) ((itNokonoko_UnkMotion1_Anim(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion1_Anim */
            case 5: {  itNokonoko_UnkMotion1_Phys(&GO[0]); } break; /* itNokonoko_UnkMotion1_Phys */
            case 6: {  ret = (u32) ((itNokonoko_UnkMotion1_Coll(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion1_Coll */
            case 7: {  it_802DC990(&GO[0]); } break; /* it_802DC990 */
            case 8: {  ret = (u32) ((itNokonoko_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion2_Anim */
            case 9: {  itNokonoko_UnkMotion2_Phys(&GO[0]); } break; /* itNokonoko_UnkMotion2_Phys */
            case 10: {  ret = (u32) ((itNokonoko_UnkMotion2_Coll(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion2_Coll */
            case 11: {  it_802DCB9C(&GO[0]); } break; /* it_802DCB9C */
            case 12: {  ret = (u32) ((itNokonoko_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion4_Anim */
            case 13: {  itNokonoko_UnkMotion4_Phys(&GO[0]); } break; /* itNokonoko_UnkMotion4_Phys */
            case 14: {  ret = (u32) ((itNokonoko_UnkMotion6_Coll(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion6_Coll */
            case 15: {  it_802DCCCC(&GO[0]); } break; /* it_802DCCCC */
            case 16: {  ret = (u32) ((itNokonoko_UnkMotion8_Anim(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion8_Anim */
            case 17: {  itNokonoko_UnkMotion7_Phys(&GO[0]); } break; /* itNokonoko_UnkMotion7_Phys */
            case 18: {  ret = (u32) ((itNokonoko_UnkMotion8_Coll(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion8_Coll */
            case 19: {  it_802DCE00(&GO[0]); } break; /* it_802DCE00 */
            case 20: {  itNokonoko_UnkMotion8_Phys(&GO[0]); } break; /* itNokonoko_UnkMotion8_Phys */
            case 21: {  it_802DCEC4(&GO[0]); } break; /* it_802DCEC4 */
            case 22: {  it_802DCFBC(&GO[0]); } break; /* it_802DCFBC */
            case 23: {  ret = (u32) ((itNokonoko_UnkMotion5_Anim(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion5_Anim */
            case 24: {  itNokonoko_UnkMotion5_Phys(&GO[0]); } break; /* itNokonoko_UnkMotion5_Phys */
            case 25: {  ret = (u32) ((itNokonoko_UnkMotion5_Coll(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion5_Coll */
            case 26: {  ret = (u32) ((itNokonoko_UnkMotion6_Anim(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion6_Anim */
            case 27: {  itNokonoko_UnkMotion6_Phys(&GO[0]); } break; /* itNokonoko_UnkMotion6_Phys */
            case 28: {  it_802DD290(&GO[0]); } break; /* it_802DD290 */
            case 29: {  it_802DD2DC(&GO[0]); } break; /* it_802DD2DC */
            case 30: {  ret = (u32) ((itNokonoko_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion3_Anim */
            case 31: {  itNokonoko_UnkMotion3_Phys(&GO[0]); } break; /* itNokonoko_UnkMotion3_Phys */
            case 32: {  ret = (u32) ((itNokonoko_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion3_Coll */
            case 33: { s32 v1 = (s32) ((int) fv()); enum Item_StateChangeFlags v2 = (enum Item_StateChangeFlags) ((int) fv()); it_802DD4A8(&GO[0], v1, v2); } break; /* it_802DD4A8 */
            case 34: {  it_802DD4F4(&GO[0]); } break; /* it_802DD4F4 */
            case 35: {  ret = (u32) ((itNokonoko_UnkMotion9_Anim(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion9_Anim */
            case 36: {  itNokonoko_UnkMotion9_Phys(&GO[0]); } break; /* itNokonoko_UnkMotion9_Phys */
            case 37: {  it_802DD59C_inline(&GO[0]); } break; /* it_802DD59C_inline */
            case 38: {  it_802DD59C(&GO[0]); } break; /* it_802DD59C */
            case 39: {  it_802DD67C(&GO[0]); } break; /* it_802DD67C */
            case 40: {  ret = (u32) ((itNokonoko_UnkMotion10_Anim(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion10_Anim */
            case 41: {  itNokonoko_UnkMotion10_Phys(&GO[0]); } break; /* itNokonoko_UnkMotion10_Phys */
            case 42: {  ret = (u32) ((itNokonoko_UnkMotion10_Coll(&GO[0])) != 0); } break; /* itNokonoko_UnkMotion10_Coll */
            case 43: {  it_802DD78C(&GO[0]); } break; /* it_802DD78C */
            case 44: {  it_802DD7D0(&GO[0], &GO[1]); } break; /* it_802DD7D0 */
            case 45: { s32 v0 = (s32) ((int) (rnd() % 4)); s32 v3 = (s32) ((rnd() & 1) ? 1 : -1); ret = PI(it_802DD7F0(v0, &VA[0], &VA[1], v3)); } break; /* it_802DD7F0 */
            case 46: {  ret = (u32) ((it_802DDA84(&GO[0])) != 0); } break; /* it_802DDA84 */
            case 47: { { int j; for (j = 0; j < 11; j++) { HM((u32) it_803F85F0[j].anim_id); HM(FI((const void*) it_803F85F0[j].animated)); HM(FI((const void*) it_803F85F0[j].physics_updated)); HM(FI((const void*) it_803F85F0[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
