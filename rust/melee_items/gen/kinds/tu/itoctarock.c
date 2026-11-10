/* TU: src/melee/it/kinds/itoctarock.c */
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

static it_804D6D40_t D40;
#include "itoctarock_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itOctarockAttributes* a = (itOctarockAttributes*) SPEC[0];
        a->x0 = (s32*) (SPEC[0] + 64); *a->x0 = (int) (rnd() % 30);
        a->x4 = rw(); a->x8 = (float) (rnd() % 40); a->xC = (float) (rnd() % 40); a->x10 = rw(); a->x14 = rw(); a->x18 = rw();
        a->x1C = (s16) (rnd() % 30);
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
    it_804D6D40 = &D40; D40.x8 = (float) (rnd() % 1000) / 1000.0f;
    for (k = 0; k < 3; k++) {
        IT[k].xDD4_itemVar.octarock.x32 = (s16) (rnd() % 4);
        IT[k].xDD4_itemVar.octarock.x2C = (int) (rnd() & 1);
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 43; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_802E4A44(&GO[0]); } break; /* it_802E4A44 */
            case 1: {  ret = (u32) ((it_802E4B00(&GO[0])) != 0); } break; /* it_802E4B00 */
            case 2: {  it_802E4C08(&GO[0]); } break; /* it_802E4C08 */
            case 3: {  ret = (u32) ((itOctarock_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itOctarock_UnkMotion0_Anim */
            case 4: {  itOctarock_UnkMotion0_Phys(&GO[0]); } break; /* itOctarock_UnkMotion0_Phys */
            case 5: {  ret = (u32) ((itOctarock_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itOctarock_UnkMotion0_Coll */
            case 6: {  it_802E4DB4(&GO[0]); } break; /* it_802E4DB4 */
            case 7: {  ret = (u32) ((itOctarock_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itOctarock_UnkMotion2_Anim */
            case 8: {  itOctarock_UnkMotion2_Phys(&GO[0]); } break; /* itOctarock_UnkMotion2_Phys */
            case 9: {  ret = (u32) ((itOctarock_UnkMotion2_Coll(&GO[0])) != 0); } break; /* itOctarock_UnkMotion2_Coll */
            case 10: {  it_802E4E6C(&GO[0]); } break; /* it_802E4E6C */
            case 11: {  ret = (u32) ((itOctarock_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itOctarock_UnkMotion3_Anim */
            case 12: {  itOctarock_UnkMotion3_Phys(&GO[0]); } break; /* itOctarock_UnkMotion3_Phys */
            case 13: {  ret = (u32) ((itOctarock_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itOctarock_UnkMotion3_Coll */
            case 14: {  it_802E503C(&GO[0]); } break; /* it_802E503C */
            case 15: {  ret = (u32) ((itOctarock_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itOctarock_UnkMotion4_Anim */
            case 16: {  itOctarock_UnkMotion4_Phys(&GO[0]); } break; /* itOctarock_UnkMotion4_Phys */
            case 17: {  ret = (u32) ((itOctarock_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itOctarock_UnkMotion4_Coll */
            case 18: {  ret = (u32) it_802E52E0(&GO[0]); } break; /* it_802E52E0 */
            case 19: {  it_802E53C8(&GO[0]); } break; /* it_802E53C8 */
            case 20: {  ret = (u32) ((itOctarock_UnkMotion1_Anim(&GO[0])) != 0); } break; /* itOctarock_UnkMotion1_Anim */
            case 21: {  itOctarock_UnkMotion1_Phys(&GO[0]); } break; /* itOctarock_UnkMotion1_Phys */
            case 22: {  ret = (u32) ((itOctarock_UnkMotion1_Coll(&GO[0])) != 0); } break; /* itOctarock_UnkMotion1_Coll */
            case 23: {  itOctarock_Logic2_PickedUp(&GO[0]); } break; /* itOctarock_Logic2_PickedUp */
            case 24: {  ret = (u32) ((itOctarock_UnkMotion5_Anim(&GO[0])) != 0); } break; /* itOctarock_UnkMotion5_Anim */
            case 25: {  itOctarock_UnkMotion5_Phys(&GO[0]); } break; /* itOctarock_UnkMotion5_Phys */
            case 26: {  itOctarock_Logic2_Dropped(&GO[0]); } break; /* itOctarock_Logic2_Dropped */
            case 27: {  itOctarock_Logic2_Thrown(&GO[0]); } break; /* itOctarock_Logic2_Thrown */
            case 28: {  ret = (u32) ((itOctarock_UnkMotion6_Anim(&GO[0])) != 0); } break; /* itOctarock_UnkMotion6_Anim */
            case 29: {  itOctarock_UnkMotion6_Phys(&GO[0]); } break; /* itOctarock_UnkMotion6_Phys */
            case 30: {  ret = (u32) ((itOctarock_UnkMotion6_Coll(&GO[0])) != 0); } break; /* itOctarock_UnkMotion6_Coll */
            case 31: {  it_802E57D4(&GO[0]); } break; /* it_802E57D4 */
            case 32: {  ret = (u32) ((itOctarock_UnkMotion7_Anim(&GO[0])) != 0); } break; /* itOctarock_UnkMotion7_Anim */
            case 33: {  itOctarock_UnkMotion7_Phys(&GO[0]); } break; /* itOctarock_UnkMotion7_Phys */
            case 34: {  ret = (u32) ((itOctarock_UnkMotion7_Coll(&GO[0])) != 0); } break; /* itOctarock_UnkMotion7_Coll */
            case 35: {  it_802E58A0(&GO[0]); } break; /* it_802E58A0 */
            case 36: {  ret = (u32) ((itOctarock_UnkMotion8_Anim(&GO[0])) != 0); } break; /* itOctarock_UnkMotion8_Anim */
            case 37: {  itOctarock_UnkMotion8_Phys(&GO[0]); } break; /* itOctarock_UnkMotion8_Phys */
            case 38: {  ret = (u32) ((itOctarock_UnkMotion8_Coll(&GO[0])) != 0); } break; /* itOctarock_UnkMotion8_Coll */
            case 39: {  it_802E5944(&GO[0]); } break; /* it_802E5944 */
            case 40: {  it_802E595C(&JO[3], &IT[0]); } break; /* it_802E595C */
            case 41: {  it_802E5AA4(&GO[0], &GO[1]); } break; /* it_802E5AA4 */
            case 42: { { int j; for (j = 0; j < 9; j++) { HM((u32) it_803F8B48[j].anim_id); HM(FI((const void*) it_803F8B48[j].animated)); HM(FI((const void*) it_803F8B48[j].physics_updated)); HM(FI((const void*) it_803F8B48[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
