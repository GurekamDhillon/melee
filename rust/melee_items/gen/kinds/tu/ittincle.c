/* TU: src/melee/it/kinds/ittincle.c */
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


#include "ittincle_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itTincleAttributes* a = (itTincleAttributes*) SPEC[0];
        a->x4 = (int) (rnd() % 8) + 0;
        a->x8 = (int) (rnd() % 8) + 0;
        a->x28 = (int) (rnd() % 5) + 1;
        a->x2C = (int) (rnd() % 6) + 0;
        a->x30 = (int) (rnd() % 3) + 1;
        a->x3C = (int) (rnd() % 8) + 0;
        a->x40 = (int) (rnd() % 6) + 0;
        a->xC = rw();
        a->x10 = rw();
        a->x14 = rw();
        a->x18 = rw();
        a->x1C = rw();
        a->x20 = rw();
        a->x24 = rw();
        a->x34 = rw();
        a->x38 = rw();
        a->x44 = rw();
        a->x48 = rw();
        a->x4C = rw();
        a->x50 = rw();
        a->x54 = (s8) (rnd() % 6); a->x55 = (s8) (rnd() % 9);
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
        IT[k].xDD4_itemVar.tincle.x20 = (int) (rnd() % 7) - 2;
        IT[k].xDD4_itemVar.tincle.x24 = (int) (rnd() % 6) + 0;
        IT[k].xDD4_itemVar.tincle.x28 = (int) (rnd() % 6) + 0;
        IT[k].xDD4_itemVar.tincle.x2C = (int) (rnd() % 7) - 1;
        IT[k].xDD4_itemVar.tincle.x30 = (rnd() & 1) ? rg() : rw();
        IT[k].xDD4_itemVar.tincle.x34 = (rnd() & 1) ? rg() : rw();
        IT[k].xDD4_itemVar.tincle.x38 = (rnd() & 1) ? rg() : rw();
        IT[k].xDD4_itemVar.tincle.x3C = (rnd() & 1) ? rg() : rw();
        IT[k].xDD4_itemVar.tincle.x40 = (rnd() & 1) ? rg() : rw();
        IT[k].xDD4_itemVar.tincle.x44 = (rnd() & 1) ? rg() : rw();
        IT[k].xDD4_itemVar.tincle.x48 = (rnd() & 1) ? rg() : rw();
        IT[k].xDD4_itemVar.tincle.x4C = (rnd() & 1) ? rg() : rw();
        IT[k].xDD4_itemVar.tincle.x50 = (rnd() & 1) ? rg() : rw();
        IT[k].xDD4_itemVar.tincle.x54 = (rnd() & 1) ? rg() : rw();
        IT[k].xDD4_itemVar.tincle.x58 = (rnd() & 1) ? rg() : rw();
        IT[k].xDD4_itemVar.tincle.x5C = (rnd() & 1) ? rg() : rw();
        IT[k].xDD4_itemVar.tincle.x64 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        if (rnd() & 1) IT[k].msid = (int) (rnd() % 14);
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.tincle.x64)); IT[k].xDD4_itemVar.tincle.x64 = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 61; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_802EB5C8(&GO[0]); } break; /* it_802EB5C8 */
            case 1: {  ret = (u32) ((itTincle_Logic13_DmgReceived(&GO[0])) != 0); } break; /* itTincle_Logic13_DmgReceived */
            case 2: {  ret = (u32) ((itTincle_Logic13_DmgDealt(&GO[0])) != 0); } break; /* itTincle_Logic13_DmgDealt */
            case 3: {  it_802EB6DC(&GO[0]); } break; /* it_802EB6DC */
            case 4: {  ret = (u32) ((itTincle_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itTincle_UnkMotion0_Anim */
            case 5: {  itTincle_UnkMotion0_Phys(&GO[0]); } break; /* itTincle_UnkMotion0_Phys */
            case 6: {  ret = (u32) ((itTincle_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itTincle_UnkMotion0_Coll */
            case 7: {  it_802EB870(&GO[0]); } break; /* it_802EB870 */
            case 8: {  ret = (u32) ((itTincle_UnkMotion1_Anim(&GO[0])) != 0); } break; /* itTincle_UnkMotion1_Anim */
            case 9: {  itTincle_UnkMotion1_Phys(&GO[0]); } break; /* itTincle_UnkMotion1_Phys */
            case 10: {  ret = (u32) ((itTincle_UnkMotion1_Coll(&GO[0])) != 0); } break; /* itTincle_UnkMotion1_Coll */
            case 11: {  it_802EBA00(&GO[0]); } break; /* it_802EBA00 */
            case 12: {  ret = (u32) ((itTincle_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itTincle_UnkMotion2_Anim */
            case 13: {  itTincle_UnkMotion2_Phys(&GO[0]); } break; /* itTincle_UnkMotion2_Phys */
            case 14: {  ret = (u32) ((itTincle_UnkMotion2_Coll(&GO[0])) != 0); } break; /* itTincle_UnkMotion2_Coll */
            case 15: {  it_802EBD14(&GO[0]); } break; /* it_802EBD14 */
            case 16: {  ret = (u32) ((itTincle_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itTincle_UnkMotion3_Anim */
            case 17: {  itTincle_UnkMotion3_Phys(&GO[0]); } break; /* itTincle_UnkMotion3_Phys */
            case 18: {  ret = (u32) ((itTincle_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itTincle_UnkMotion3_Coll */
            case 19: {  it_802EBE5C(&GO[0]); } break; /* it_802EBE5C */
            case 20: {  ret = (u32) ((itTincle_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itTincle_UnkMotion4_Anim */
            case 21: {  itTincle_UnkMotion4_Phys(&GO[0]); } break; /* itTincle_UnkMotion4_Phys */
            case 22: {  ret = (u32) ((itTincle_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itTincle_UnkMotion4_Coll */
            case 23: {  it_802EBFAC(&GO[0]); } break; /* it_802EBFAC */
            case 24: {  ret = (u32) ((itTincle_UnkMotion5_Anim(&GO[0])) != 0); } break; /* itTincle_UnkMotion5_Anim */
            case 25: {  itTincle_UnkMotion5_Phys(&GO[0]); } break; /* itTincle_UnkMotion5_Phys */
            case 26: {  ret = (u32) ((itTincle_UnkMotion5_Coll(&GO[0])) != 0); } break; /* itTincle_UnkMotion5_Coll */
            case 27: {  it_802EC18C(&GO[0]); } break; /* it_802EC18C */
            case 28: {  it_802EC1F4(&GO[0]); } break; /* it_802EC1F4 */
            case 29: {  ret = (u32) ((itTincle_UnkMotion7_Anim(&GO[0])) != 0); } break; /* itTincle_UnkMotion7_Anim */
            case 30: {  itTincle_UnkMotion7_Phys(&GO[0]); } break; /* itTincle_UnkMotion7_Phys */
            case 31: {  ret = (u32) ((itTincle_UnkMotion7_Coll(&GO[0])) != 0); } break; /* itTincle_UnkMotion7_Coll */
            case 32: {  it_802EC35C(&GO[0]); } break; /* it_802EC35C */
            case 33: {  ret = (u32) ((itTincle_UnkMotion8_Anim(&GO[0])) != 0); } break; /* itTincle_UnkMotion8_Anim */
            case 34: {  itTincle_UnkMotion8_Phys(&GO[0]); } break; /* itTincle_UnkMotion8_Phys */
            case 35: {  ret = (u32) ((itTincle_UnkMotion8_Coll(&GO[0])) != 0); } break; /* itTincle_UnkMotion8_Coll */
            case 36: {  it_802EC3F4(&GO[0]); } break; /* it_802EC3F4 */
            case 37: {  ret = (u32) ((itTincle_UnkMotion9_Anim(&GO[0])) != 0); } break; /* itTincle_UnkMotion9_Anim */
            case 38: {  itTincle_UnkMotion9_Phys(&GO[0]); } break; /* itTincle_UnkMotion9_Phys */
            case 39: {  ret = (u32) ((itTincle_UnkMotion9_Coll(&GO[0])) != 0); } break; /* itTincle_UnkMotion9_Coll */
            case 40: {  it_802EC4D0(&GO[0]); } break; /* it_802EC4D0 */
            case 41: {  ret = (u32) ((itTincle_UnkMotion10_Anim(&GO[0])) != 0); } break; /* itTincle_UnkMotion10_Anim */
            case 42: {  itTincle_UnkMotion10_Phys(&GO[0]); } break; /* itTincle_UnkMotion10_Phys */
            case 43: {  ret = (u32) ((itTincle_UnkMotion10_Coll(&GO[0])) != 0); } break; /* itTincle_UnkMotion10_Coll */
            case 44: {  it_802EC604(&GO[0]); } break; /* it_802EC604 */
            case 45: {  ret = (u32) ((itTincle_UnkMotion11_Anim(&GO[0])) != 0); } break; /* itTincle_UnkMotion11_Anim */
            case 46: {  itTincle_UnkMotion11_Phys(&GO[0]); } break; /* itTincle_UnkMotion11_Phys */
            case 47: {  ret = (u32) ((itTincle_UnkMotion11_Coll(&GO[0])) != 0); } break; /* itTincle_UnkMotion11_Coll */
            case 48: {  it_802EC69C(&GO[0]); } break; /* it_802EC69C */
            case 49: {  ret = (u32) ((itTincle_UnkMotion12_Anim(&GO[0])) != 0); } break; /* itTincle_UnkMotion12_Anim */
            case 50: {  itTincle_UnkMotion12_Phys(&GO[0]); } break; /* itTincle_UnkMotion12_Phys */
            case 51: {  ret = (u32) ((itTincle_UnkMotion12_Coll(&GO[0])) != 0); } break; /* itTincle_UnkMotion12_Coll */
            case 52: {  it_802EC830(&GO[0]); } break; /* it_802EC830 */
            case 53: {  it_802EC850(&GO[0], &GO[1]); } break; /* it_802EC850 */
            case 54: { int v1 = (int) ((int) (rnd() % 2)); it_802EC870(&GO[0], v1); } break; /* it_802EC870 */
            case 55: {  it_802EC9E8(&GO[0]); } break; /* it_802EC9E8 */
            case 56: {  ret = PI(it_802ECA70(&GO[0])); } break; /* it_802ECA70 */
            case 57: {  ret = (u32) it_802ECC8C(&GO[0]); } break; /* it_802ECC8C */
            case 58: { f32 v1 = (f32) ((int) (rnd() % 2)); it_802ECC98(&GO[0], v1); } break; /* it_802ECC98 */
            case 59: { { int kv = (int) fv(); it_802ECCA4(&GO[0], &kv, &VA[0]); ret = (u32) kv; } } break; /* it_802ECCA4 */
            case 60: { { int j; for (j = 0; j < 13; j++) { HM((u32) it_803F9000[j].anim_id); HM(FI((const void*) it_803F9000[j].animated)); HM(FI((const void*) it_803F9000[j].physics_updated)); HM(FI((const void*) it_803F9000[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
