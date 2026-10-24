/* TU: src/melee/it/kinds/itmaril.c */
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


#include "itmaril_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        float* a = (float*) SPEC[0];
        a[0] = rw();
        a[1] = rw();
        a[2] = rw();
        a[3] = rw();
        a[4] = rw();
        a[5] = (float) (rnd() % 50);
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

    IT[0].owner = (rnd() & 3) ? &GO[1] : 0;
    IT[0].xC64_reflectGObj = (rnd() & 3) ? &GO[1] : 0;
    if (rnd() & 1) IT[0].msid = (int) (rnd() % 14);
    if (rnd() & 1) IT[0].facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
    IT[0].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
    spec_fill(prof);
    for (k = 0; k < 3; k++) {
        IT[k].xDD4_itemVar.maril.x60.flags = 0;
        IT[k].xDD4_itemVar.maril.x60.x0 = rnd() & 1;
        IT[k].xC9C = (int) (rnd() % 60);
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(IT[k].xDD4_itemVar.maril.x60.x0); IT[k].xDD4_itemVar.maril.x60.flags = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 31; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  ret = (u32) itMaril_UnkMotion2_Anim_inline(&GO[0]); } break; /* itMaril_UnkMotion2_Anim_inline */
            case 1: {  ret = (u32) it_802D66F8(&GO[0]); } break; /* it_802D66F8 */
            case 2: {  it_802D6740(&GO[0]); } break; /* it_802D6740 */
            case 3: {  ret = (u32) it_802D6798(&GO[0]); } break; /* it_802D6798 */
            case 4: {  ret = (u32) it_802D6808(&GO[0]); } break; /* it_802D6808 */
            case 5: {  ret = (u32) it_802D6810(&GO[0]); } break; /* it_802D6810 */
            case 6: {  ret = (u32) it_802D6830(&GO[0]); } break; /* it_802D6830 */
            case 7: {  ret = (u32) it_802D6838(&GO[0]); } break; /* it_802D6838 */
            case 8: {  ret = (u32) it_802D6840(&GO[0]); } break; /* it_802D6840 */
            case 9: {  ret = (u32) it_802D6848(&GO[0]); } break; /* it_802D6848 */
            case 10: {  it_802D6850(&GO[0], &GO[1]); } break; /* it_802D6850 */
            case 11: {  itMaril_Logic28_Spawned(&GO[0]); } break; /* itMaril_Logic28_Spawned */
            case 12: {  it_802D68FC(&GO[0]); } break; /* it_802D68FC */
            case 13: {  itMaril_UnkMotion1_Coll_inline(&GO[0]); } break; /* itMaril_UnkMotion1_Coll_inline */
            case 14: {  ret = (u32) itMaril_UnkMotion0_Anim(&GO[0]); } break; /* itMaril_UnkMotion0_Anim */
            case 15: {  itMaril_UnkMotion0_Phys(&GO[0]); } break; /* itMaril_UnkMotion0_Phys */
            case 16: {  ret = (u32) itMaril_UnkMotion0_Coll(&GO[0]); } break; /* itMaril_UnkMotion0_Coll */
            case 17: {  it_802D69E4(&GO[0]); } break; /* it_802D69E4 */
            case 18: {  it_802D6A54(&GO[0]); } break; /* it_802D6A54 */
            case 19: {  ret = (u32) itMaril_UnkMotion1_Anim(&GO[0]); } break; /* itMaril_UnkMotion1_Anim */
            case 20: {  itMaril_UnkMotion1_Phys(&GO[0]); } break; /* itMaril_UnkMotion1_Phys */
            case 21: {  ret = (u32) itMaril_UnkMotion1_Coll(&GO[0]); } break; /* itMaril_UnkMotion1_Coll */
            case 22: {  it_802D6DDC(&GO[0]); } break; /* it_802D6DDC */
            case 23: {  ret = (u32) itMaril_UnkMotion2_Anim(&GO[0]); } break; /* itMaril_UnkMotion2_Anim */
            case 24: {  itMaril_UnkMotion2_Phys(&GO[0]); } break; /* itMaril_UnkMotion2_Phys */
            case 25: {  ret = (u32) itMaril_UnkMotion2_Coll(&GO[0]); } break; /* itMaril_UnkMotion2_Coll */
            case 26: {  ret = (u32) itMaril_UnkMotion3_Anim(&GO[0]); } break; /* itMaril_UnkMotion3_Anim */
            case 27: {  itMaril_UnkMotion3_Phys(&GO[0]); } break; /* itMaril_UnkMotion3_Phys */
            case 28: {  ret = (u32) it_802D6F00(&GO[0]); } break; /* it_802D6F00 */
            case 29: {  it_802D6FB0(&GO[0], &GO[1]); } break; /* it_802D6FB0 */
            case 30: { { int j; for (j = 0; j < 4; j++) { HM((u32) it_803F82B0[j].anim_id); HM(FI((const void*) it_803F82B0[j].animated)); HM(FI((const void*) it_803F82B0[j].physics_updated)); HM(FI((const void*) it_803F82B0[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
