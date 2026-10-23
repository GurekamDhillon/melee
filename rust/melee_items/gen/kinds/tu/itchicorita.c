/* TU: src/melee/it/kinds/itchicorita.c */
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


#include "itchicorita_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itChicoritaAttr* a = (itChicoritaAttr*) SPEC[0];
        a->scale = rw(); a->x4 = (int) (rnd() % 4);
    } break;
    case 1: {
        itChicoritaLeafAttr* a = (itChicoritaLeafAttr*) SPEC[0];
        a->timer = (float) (rnd() % 20); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw();
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
        IT[k].xDD4_itemVar.chicorita.x60 = (int) (rnd() % 5) - 1;
        IT[k].xDD4_itemVar.chicorita.x64 = rg();
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 27; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_802C9588(&GO[0]); } break; /* it_802C9588 */
            case 1: {  it_802C9618(&GO[0]); } break; /* it_802C9618 */
            case 2: {  itChicorita_Logic1_EvtUnk(&GO[0], &GO[1]); } break; /* itChicorita_Logic1_EvtUnk */
            case 3: {  it_802C963C(&GO[0]); } break; /* it_802C963C */
            case 4: {  it_802C9670(&GO[0]); } break; /* it_802C9670 */
            case 5: {  ret = (u32) itChicorita_UnkMotion0_Anim(&GO[0]); } break; /* itChicorita_UnkMotion0_Anim */
            case 6: {  itChicorita_UnkMotion0_Phys(&GO[0]); } break; /* itChicorita_UnkMotion0_Phys */
            case 7: {  ret = (u32) itChicorita_UnkMotion0_Coll(&GO[0]); } break; /* itChicorita_UnkMotion0_Coll */
            case 8: {  it_802C989C(&GO[0]); } break; /* it_802C989C */
            case 9: {  it_802C98E4(&GO[0]); } break; /* it_802C98E4 */
            case 10: {  ret = (u32) itChicorita_UnkMotion1_Anim(&GO[0]); } break; /* itChicorita_UnkMotion1_Anim */
            case 11: {  itChicorita_UnkMotion1_Phys(&GO[0]); } break; /* itChicorita_UnkMotion1_Phys */
            case 12: {  ret = (u32) itChicorita_UnkMotion1_Coll(&GO[0]); } break; /* itChicorita_UnkMotion1_Coll */
            case 13: {  it_802C9A74(&GO[0]); } break; /* it_802C9A74 */
            case 14: {  ret = (u32) itChicorita_UnkMotion2_Anim(&GO[0]); } break; /* itChicorita_UnkMotion2_Anim */
            case 15: {  itChicorita_UnkMotion2_Phys(&GO[0]); } break; /* itChicorita_UnkMotion2_Phys */
            case 16: {  ret = (u32) itChicorita_UnkMotion2_Coll(&GO[0]); } break; /* itChicorita_UnkMotion2_Coll */
            case 17: {  it_802C9B20(&GO[0]); } break; /* it_802C9B20 */
            case 18: {  itChicoritaLeaf_Logic30_Spawned(&GO[0]); } break; /* itChicoritaLeaf_Logic30_Spawned */
            case 19: {  ret = (u32) itChicoritaLeaf_Logic30_HitShield(&GO[0]); } break; /* itChicoritaLeaf_Logic30_HitShield */
            case 20: {  itChicoritaLeaf_Logic30_EvtUnk(&GO[0], &GO[1]); } break; /* itChicoritaLeaf_Logic30_EvtUnk */
            case 21: {  ret = (u32) itChicoritaLeaf_Logic30_Reflected(&GO[0]); } break; /* itChicoritaLeaf_Logic30_Reflected */
            case 22: {  it_802C9CC0(&GO[0]); } break; /* it_802C9CC0 */
            case 23: {  ret = (u32) itChicoritaleaf_UnkMotion0_Anim(&GO[0]); } break; /* itChicoritaleaf_UnkMotion0_Anim */
            case 24: {  itChicoritaleaf_UnkMotion0_Phys(&GO[0]); } break; /* itChicoritaleaf_UnkMotion0_Phys */
            case 25: {  ret = (u32) itChicoritaleaf_UnkMotion0_Coll(&GO[0]); } break; /* itChicoritaleaf_UnkMotion0_Coll */
            case 26: { { int j; for (j = 0; j < 3; j++) { HM((u32) it_803F7A98[j].anim_id); HM(FI((const void*) it_803F7A98[j].animated)); HM(FI((const void*) it_803F7A98[j].physics_updated)); HM(FI((const void*) it_803F7A98[j].collided)); } } { int j; for (j = 0; j < 1; j++) { HM((u32) it_803F7AC8[j].anim_id); HM(FI((const void*) it_803F7AC8[j].animated)); HM(FI((const void*) it_803F7AC8[j].physics_updated)); HM(FI((const void*) it_803F7AC8[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
