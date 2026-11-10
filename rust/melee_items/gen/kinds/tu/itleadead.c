/* TU: src/melee/it/kinds/itleadead.c */
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

static itSpecialAttrsHead LDX[3]; static it_804D6D40_t D40;
#include "itleadead_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        { int q; for (q = 0; q < 3; q++) {
        itLeadeadAttributes* a = (itLeadeadAttributes*) SPEC[q];
        LDX[q].x0 = (int) (rnd() % 60); LDX[q].x4 = rw();
        a->x0 = &LDX[q]; a->x4 = (int) (rnd() % 30); a->x8 = (float) (rnd() % 4); a->xC = rw(); a->x10 = rw(); a->x14 = rw();
        a->x18 = (s16) (rnd() % 5); a->x1A = (s16) (rnd() % 4); a->x1C = (s16) (rnd() % 4); a->x1E = (s8) (rnd() % 4);
        } }
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
        IT[k].xDD4_itemVar.leadead.x38 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xDD4_itemVar.leadead.x3C = rw();
        IT[k].xDD4_itemVar.leadead.x44 = (int) (rnd() % 0x120);
        IT[k].xDD4_itemVar.leadead.x48 = (int) (rnd() % 0x20) - 3;
        IT[k].xDD4_itemVar.leadead.x4C = (int) (rnd() % 4) - 1;
        IT[k].xDD4_itemVar.leadead.x50 = (int) (rnd() % 4);
        IT[k].xDD4_itemVar.leadead.xE14 = (int) (rnd() % 8);
        IT[k].xC9C = (int) (rnd() % 80);
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.leadead.x38)); IT[k].xDD4_itemVar.leadead.x38 = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 79; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_802E8BCC(&GO[0]); } break; /* it_802E8BCC */
            case 1: {  ret = (u32) ((it_802E8CD8(&GO[0])) != 0); } break; /* it_802E8CD8 */
            case 2: {  it_802E8F24(&GO[0]); } break; /* it_802E8F24 */
            case 3: {  ret = (u32) ((itLeadead_UnkMotion1_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion1_Anim */
            case 4: {  itLeadead_UnkMotion1_Phys(&GO[0]); } break; /* itLeadead_UnkMotion1_Phys */
            case 5: {  ret = (u32) ((itLeadead_UnkMotion1_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion1_Coll */
            case 6: {  it_802E9038(&GO[0]); } break; /* it_802E9038 */
            case 7: {  ret = (u32) ((itLeadead_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion2_Anim */
            case 8: {  itLeadead_UnkMotion2_Phys(&GO[0]); } break; /* itLeadead_UnkMotion2_Phys */
            case 9: {  ret = (u32) ((itLeadead_UnkMotion2_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion2_Coll */
            case 10: {  it_802E9308(&GO[0]); } break; /* it_802E9308 */
            case 11: {  ret = (u32) ((itLeadead_UnkMotion10_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion10_Anim */
            case 12: {  itLeadead_UnkMotion10_Phys(&GO[0]); } break; /* itLeadead_UnkMotion10_Phys */
            case 13: {  ret = (u32) ((itLeadead_UnkMotion10_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion10_Coll */
            case 14: {  it_802E93C8(&GO[0]); } break; /* it_802E93C8 */
            case 15: {  it_802E9494(&GO[0]); } break; /* it_802E9494 */
            case 16: {  ret = (u32) ((itLeadead_UnkMotion11_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion11_Anim */
            case 17: {  ret = (u32) ((itLeadead_UnkMotion12_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion12_Anim */
            case 18: {  itLeadead_UnkMotion12_Phys(&GO[0]); } break; /* itLeadead_UnkMotion12_Phys */
            case 19: {  ret = (u32) ((itLeadead_UnkMotion12_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion12_Coll */
            case 20: {  it_802E9738(&GO[0]); } break; /* it_802E9738 */
            case 21: {  ret = (u32) ((itLeadead_UnkMotion7_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion7_Anim */
            case 22: {  itLeadead_UnkMotion7_Phys(&GO[0]); } break; /* itLeadead_UnkMotion7_Phys */
            case 23: {  ret = (u32) ((itLeadead_UnkMotion7_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion7_Coll */
            case 24: {  it_802E98E0(&GO[0]); } break; /* it_802E98E0 */
            case 25: {  ret = (u32) ((itLeadead_UnkMotion8_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion8_Anim */
            case 26: {  itLeadead_UnkMotion8_Phys(&GO[0]); } break; /* itLeadead_UnkMotion8_Phys */
            case 27: {  ret = (u32) ((itLeadead_UnkMotion8_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion8_Coll */
            case 28: {  it_802E9A00(&GO[0]); } break; /* it_802E9A00 */
            case 29: {  ret = (u32) ((itLeadead_UnkMotion9_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion9_Anim */
            case 30: {  itLeadead_UnkMotion9_Phys(&GO[0]); } break; /* itLeadead_UnkMotion9_Phys */
            case 31: {  ret = (u32) ((itLeadead_UnkMotion9_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion9_Coll */
            case 32: {  it_802E9BA0(&GO[0]); } break; /* it_802E9BA0 */
            case 33: {  ret = (u32) ((itLeadead_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion3_Anim */
            case 34: {  itLeadead_UnkMotion3_Phys(&GO[0]); } break; /* itLeadead_UnkMotion3_Phys */
            case 35: {  ret = (u32) ((itLeadead_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion3_Coll */
            case 36: {  it_802E9D50(&GO[0]); } break; /* it_802E9D50 */
            case 37: {  ret = (u32) ((itLeadead_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion4_Anim */
            case 38: {  itLeadead_UnkMotion4_Phys(&GO[0]); } break; /* itLeadead_UnkMotion4_Phys */
            case 39: {  itLeadead_UnkMotion4_Coll_inline(&GO[0]); } break; /* itLeadead_UnkMotion4_Coll_inline */
            case 40: {  ret = (u32) ((itLeadead_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion4_Coll */
            case 41: {  ret = (u32) ((itLeadead_UnkMotion5_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion5_Anim */
            case 42: {  itLeadead_UnkMotion5_Phys(&GO[0]); } break; /* itLeadead_UnkMotion5_Phys */
            case 43: {  ret = (u32) ((itLeadead_UnkMotion5_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion5_Coll */
            case 44: {  ret = (u32) ((itLeadead_UnkMotion6_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion6_Anim */
            case 45: {  itLeadead_UnkMotion6_Phys(&GO[0]); } break; /* itLeadead_UnkMotion6_Phys */
            case 46: {  ret = (u32) ((itLeadead_UnkMotion6_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion6_Coll */
            case 47: {  itLeadead_Logic1_PickedUp(&GO[0]); } break; /* itLeadead_Logic1_PickedUp */
            case 48: {  ret = (u32) ((itLeadead_UnkMotion13_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion13_Anim */
            case 49: {  itLeadead_UnkMotion13_Phys(&GO[0]); } break; /* itLeadead_UnkMotion13_Phys */
            case 50: {  itLeadead_Logic1_Dropped(&GO[0]); } break; /* itLeadead_Logic1_Dropped */
            case 51: {  itLeadead_Logic1_Thrown(&GO[0]); } break; /* itLeadead_Logic1_Thrown */
            case 52: {  ret = (u32) ((itLeadead_UnkMotion14_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion14_Anim */
            case 53: {  itLeadead_UnkMotion14_Phys(&GO[0]); } break; /* itLeadead_UnkMotion14_Phys */
            case 54: {  ret = (u32) ((itLeadead_UnkMotion14_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion14_Coll */
            case 55: {  it_802EA2A0(&GO[0]); } break; /* it_802EA2A0 */
            case 56: {  ret = (u32) ((itLeadead_UnkMotion15_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion15_Anim */
            case 57: {  itLeadead_UnkMotion15_Phys(&GO[0]); } break; /* itLeadead_UnkMotion15_Phys */
            case 58: {  ret = (u32) ((itLeadead_UnkMotion15_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion15_Coll */
            case 59: {  it_802EA334(&GO[0]); } break; /* it_802EA334 */
            case 60: {  ret = (u32) ((itLeadead_UnkMotion16_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion16_Anim */
            case 61: {  itLeadead_UnkMotion16_Phys(&GO[0]); } break; /* itLeadead_UnkMotion16_Phys */
            case 62: {  ret = (u32) ((itLeadead_UnkMotion16_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion16_Coll */
            case 63: {  ret = (u32) ((itLeadead_UnkMotion17_Anim(&GO[0])) != 0); } break; /* itLeadead_UnkMotion17_Anim */
            case 64: {  itLeadead_UnkMotion17_Phys(&GO[0]); } break; /* itLeadead_UnkMotion17_Phys */
            case 65: {  ret = (u32) ((itLeadead_UnkMotion17_Coll(&GO[0])) != 0); } break; /* itLeadead_UnkMotion17_Coll */
            case 66: {  itLeadead_Logic1_Destroyed(&GO[0]); } break; /* itLeadead_Logic1_Destroyed */
            case 67: { s32 v1 = (s32) ((int) (rnd() % 5)); s32 v2 = (s32) ((int) (rnd() % 6)); it_802EA478(&GO[0], v1, v2); } break; /* it_802EA478 */
            case 68: {  ret = (u32) it_802EA674(&GO[0]); } break; /* it_802EA674 */
            case 69: {  ret = (u32) it_802EA6F4(&GO[0]); } break; /* it_802EA6F4 */
            case 70: { f32 v1 = (f32) (rw()); ret = (u32) ((it_802EA804(&GO[0], v1)) != 0); } break; /* it_802EA804 */
            case 71: { f32 v1 = (f32) (rw()); f32 v2 = (f32) (rw()); ret = (u32) it_802EA988(&GO[0], v1, v2); } break; /* it_802EA988 */
            case 72: { s32 v1 = (s32) ((rnd() & 1) ? 1 : -1); ret = PI(it_802EA9FC(&VA[0], v1)); } break; /* it_802EA9FC */
            case 73: { int v2 = (int) ((int) (rnd() % 8)); f32 v3 = (f32) (rw()); it_802EAAEC(&GO[0], &GO[1], v2, v3); } break; /* it_802EAAEC */
            case 74: {  it_802EAC8C(&GO[0]); } break; /* it_802EAC8C */
            case 75: {  it_802EADD8(&GO[0]); } break; /* it_802EADD8 */
            case 76: {  it_802EAE80(&GO[0]); } break; /* it_802EAE80 */
            case 77: {  ret = PI(it_802EAF28(&GO[0])); } break; /* it_802EAF28 */
            case 78: { { int j; for (j = 0; j < 18; j++) { HM((u32) it_803F8EB0[j].anim_id); HM(FI((const void*) it_803F8EB0[j].animated)); HM(FI((const void*) it_803F8EB0[j].physics_updated)); HM(FI((const void*) it_803F8EB0[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
