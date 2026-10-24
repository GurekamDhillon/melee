/* TU: src/melee/it/kinds/itwhispyapple.c */
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


#include "itwhispyapple_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itWhispyAppleAttributes* a = (itWhispyAppleAttributes*) SPEC[0];
        a->x4 = (int) (rnd() % 30); a->x8 = (int) (rnd() % 30); a->x14 = rw(); a->x18 = rw();
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
        ART[k].x4_specialAttributes = SPEC[k];
        for (i = 0; i < 14; i++) DBT[k].bones[i] = &JO[6 + 14 * k + i];
    }
    it_804D6D28 = &ICD;
    IT[0].owner = (rnd() & 3) ? &GO[1] : 0;
    IT[0].xC64_reflectGObj = (rnd() & 3) ? &GO[1] : 0;
    if (rnd() & 1) IT[0].msid = (int) (rnd() % 14);
    if (rnd() & 1) IT[0].facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
    IT[0].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
    spec_fill(prof);
    for (k = 0; k < 3; k++) {
        IT[k].xDD4_itemVar.whispyapple.xDD8 = (int) (rnd() % 3);
        IT[k].xD44_lifeTimer = (rnd() & 1) ? (float) (int) (rnd() % 4) : rg();
        IT[k].pos.z = (rnd() & 1) ? (float) (int) (rnd() % 3) - 1 : rg();
        it_804D6D28->x34 = rnd() % 4;
        if (rnd() & 1) IT[k].msid = 7;
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 34; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  ret = (u32) itWhispyapple_UnkMotion0_Anim_inline(&GO[0]); } break; /* itWhispyapple_UnkMotion0_Anim_inline */
            case 1: { f32 v2 = (f32) ((float) (rnd() % 10) / 10.0f); f32 v3 = (f32) ((float) (rnd() % 10) / 10.0f); ret = PI(it_802EE200(&GO[0], &VA[0], v2, v3)); } break; /* it_802EE200 */
            case 2: {  it_802EE374(&GO[0]); } break; /* it_802EE374 */
            case 3: {  ret = (u32) itWhispyapple_UnkMotion0_Anim(&GO[0]); } break; /* itWhispyapple_UnkMotion0_Anim */
            case 4: {  itWhispyapple_UnkMotion0_Phys(&GO[0]); } break; /* itWhispyapple_UnkMotion0_Phys */
            case 5: {  ret = (u32) itWhispyapple_UnkMotion0_Coll(&GO[0]); } break; /* itWhispyapple_UnkMotion0_Coll */
            case 6: {  it_802EE6A0(&GO[0]); } break; /* it_802EE6A0 */
            case 7: {  ret = (u32) itWhispyapple_UnkMotion1_Anim(&GO[0]); } break; /* itWhispyapple_UnkMotion1_Anim */
            case 8: {  itWhispyapple_UnkMotion1_Phys(&GO[0]); } break; /* itWhispyapple_UnkMotion1_Phys */
            case 9: {  ret = (u32) itWhispyapple_UnkMotion1_Coll(&GO[0]); } break; /* itWhispyapple_UnkMotion1_Coll */
            case 10: {  fn_802EE7FC(&GO[0]); } break; /* fn_802EE7FC */
            case 11: {  ret = (u32) itWhispyapple_UnkMotion5_Anim(&GO[0]); } break; /* itWhispyapple_UnkMotion5_Anim */
            case 12: {  itWhispyapple_UnkMotion5_Phys(&GO[0]); } break; /* itWhispyapple_UnkMotion5_Phys */
            case 13: {  ret = (u32) itWhispyapple_UnkMotion5_Coll(&GO[0]); } break; /* itWhispyapple_UnkMotion5_Coll */
            case 14: {  it_802EEA08(&GO[0]); } break; /* it_802EEA08 */
            case 15: {  ret = (u32) itWhispyapple_UnkMotion3_Anim(&GO[0]); } break; /* itWhispyapple_UnkMotion3_Anim */
            case 16: {  itWhispyapple_UnkMotion3_Phys(&GO[0]); } break; /* itWhispyapple_UnkMotion3_Phys */
            case 17: {  it_802EEA70(&GO[0]); } break; /* it_802EEA70 */
            case 18: {  it_802EEB28(&GO[0]); } break; /* it_802EEB28 */
            case 19: {  itWhispyApple_Logic18_EnteredAir(&GO[0]); } break; /* itWhispyApple_Logic18_EnteredAir */
            case 20: {  ret = (u32) itWhispyapple_UnkMotion6_Anim(&GO[0]); } break; /* itWhispyapple_UnkMotion6_Anim */
            case 21: {  itWhispyapple_UnkMotion6_Phys(&GO[0]); } break; /* itWhispyapple_UnkMotion6_Phys */
            case 22: {  ret = (u32) itWhispyapple_UnkMotion6_Coll(&GO[0]); } break; /* itWhispyapple_UnkMotion6_Coll */
            case 23: {  it_802EED00(&GO[0]); } break; /* it_802EED00 */
            case 24: {  ret = (u32) itWhispyapple_UnkMotion7_Anim(&GO[0]); } break; /* itWhispyapple_UnkMotion7_Anim */
            case 25: {  itWhispyapple_UnkMotion7_Phys(&GO[0]); } break; /* itWhispyapple_UnkMotion7_Phys */
            case 26: {  ret = (u32) itWhispyapple_UnkMotion7_Coll(&GO[0]); } break; /* itWhispyapple_UnkMotion7_Coll */
            case 27: {  ret = (u32) it_802EEED0(&GO[0]); } break; /* it_802EEED0 */
            case 28: {  ret = (u32) it_802EEED8(&GO[0]); } break; /* it_802EEED8 */
            case 29: {  ret = (u32) it_802EEF10(&GO[0]); } break; /* it_802EEF10 */
            case 30: {  ret = (u32) it_802EEF30(&GO[0]); } break; /* it_802EEF30 */
            case 31: {  ret = (u32) it_802EEF68(&GO[0]); } break; /* it_802EEF68 */
            case 32: {  it_802EEF88(&GO[0], &GO[1]); } break; /* it_802EEF88 */
            case 33: { { int j; for (j = 0; j < 8; j++) { HM((u32) it_803F91D0[j].anim_id); HM(FI((const void*) it_803F91D0[j].animated)); HM(FI((const void*) it_803F91D0[j].physics_updated)); HM(FI((const void*) it_803F91D0[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
