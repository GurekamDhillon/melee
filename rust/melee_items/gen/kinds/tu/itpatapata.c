/* TU: src/melee/it/kinds/itpatapata.c */
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


#include "itpatapata_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itPatapataAttributes* a = (itPatapataAttributes*) SPEC[0];
        a->x0 = (void*) (SPEC[0] + 64); ((float*) (SPEC[0] + 64))[1] = rw();
        a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = (int) (rnd() % 6); a->x14 = rw(); a->x18 = rw();
        a->x1C = (int) (rnd() % 20); a->x20 = rw(); a->x24 = (int) (rnd() % 20); a->x28 = (float) (rnd() % 30);
        a->x2C = (int) (rnd() % 6); a->x30 = (float) (rnd() % 5); a->x34 = (float) (rnd() % 5); a->x38 = rw(); a->x3C = rw();
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
        for (i = 14; i < 19; i++) DBT[k].bones[i] = &JO[6 + 14 * k + (i - 14)];
        IT[k].xDD4_itemVar.patapata.x40 = (int) (rnd() % 3);
        IT[k].xDD4_itemVar.patapata.x20 = (int) (rnd() % 5) - 1;
        IT[k].xDD4_itemVar.patapata.x24 = (int) (rnd() % 5) - 1;
        IT[k].xDD4_itemVar.patapata.x28 = (int) (rnd() % 5) - 1;
        IT[k].xDD4_itemVar.patapata.x2C = rw();
        IT[k].xDD4_itemVar.patapata.x30 = rw();
        IT[k].xDD4_itemVar.patapata.x44 = (int) (rnd() & 1);
        IT[k].xDD4_itemVar.patapata.x4 = (int) (rnd() % 4);
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
            case 0: {  it_802E05A0(&GO[0]); } break; /* it_802E05A0 */
            case 1: {  it_802E0678(&GO[0]); } break; /* it_802E0678 */
            case 2: {  ret = (u32) ((itPatapata_UnkMotion1_Anim(&GO[0])) != 0); } break; /* itPatapata_UnkMotion1_Anim */
            case 3: {  itPatapata_UnkMotion1_Phys(&GO[0]); } break; /* itPatapata_UnkMotion1_Phys */
            case 4: {  ret = (u32) ((itPatapata_UnkMotion1_Coll(&GO[0])) != 0); } break; /* itPatapata_UnkMotion1_Coll */
            case 5: {  it_802E0734(&GO[0]); } break; /* it_802E0734 */
            case 6: {  ret = (u32) ((itPatapata_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itPatapata_UnkMotion2_Anim */
            case 7: {  itPatapata_UnkMotion3_Phys(&GO[0]); } break; /* itPatapata_UnkMotion3_Phys */
            case 8: {  ret = (u32) ((itPatapata_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itPatapata_UnkMotion3_Coll */
            case 9: {  it_802E0D9C(&GO[0]); } break; /* it_802E0D9C */
            case 10: {  ret = (u32) ((itPatapata_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itPatapata_UnkMotion3_Anim */
            case 11: {  ret = (u32) ((it_2725_Logic4_DmgReceived(&GO[0])) != 0); } break; /* it_2725_Logic4_DmgReceived */
            case 12: {  ret = (u32) ((it_802E0F1C(&GO[0])) != 0); } break; /* it_802E0F1C */
            case 13: {  itPatapata_Logic4_PickedUp(&GO[0]); } break; /* itPatapata_Logic4_PickedUp */
            case 14: {  ret = (u32) ((itPatapata_UnkMotion5_Anim(&GO[0])) != 0); } break; /* itPatapata_UnkMotion5_Anim */
            case 15: {  itPatapata_UnkMotion5_Phys(&GO[0]); } break; /* itPatapata_UnkMotion5_Phys */
            case 16: {  it_2725_Logic4_Dropped(&GO[0]); } break; /* it_2725_Logic4_Dropped */
            case 17: {  ret = (u32) ((itPatapata_UnkMotion7_Anim(&GO[0])) != 0); } break; /* itPatapata_UnkMotion7_Anim */
            case 18: {  itPatapata_UnkMotion7_Phys(&GO[0]); } break; /* itPatapata_UnkMotion7_Phys */
            case 19: {  ret = (u32) ((itPatapata_UnkMotion7_Coll(&GO[0])) != 0); } break; /* itPatapata_UnkMotion7_Coll */
            case 20: {  it_2725_Logic4_Thrown(&GO[0]); } break; /* it_2725_Logic4_Thrown */
            case 21: {  ret = (u32) ((itPatapata_UnkMotion6_Anim(&GO[0])) != 0); } break; /* itPatapata_UnkMotion6_Anim */
            case 22: {  itPatapata_UnkMotion6_Phys(&GO[0]); } break; /* itPatapata_UnkMotion6_Phys */
            case 23: {  ret = (u32) ((itPatapata_UnkMotion6_Coll(&GO[0])) != 0); } break; /* itPatapata_UnkMotion6_Coll */
            case 24: {  it_802E11E0(&GO[0]); } break; /* it_802E11E0 */
            case 25: {  ret = (u32) ((itPatapata_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itPatapata_UnkMotion4_Anim */
            case 26: {  itPatapata_UnkMotion4_Phys(&GO[0]); } break; /* itPatapata_UnkMotion4_Phys */
            case 27: {  ret = (u32) ((itPatapata_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itPatapata_UnkMotion4_Coll */
            case 28: {  it_802E15B0(&GO[0]); } break; /* it_802E15B0 */
            case 29: { int v1 = (int) ((int) fv()); int v2 = (int) ((int) fv()); it_802E1648(&GO[0], v1, v2); } break; /* it_802E1648 */
            case 30: {  it_802E1694(&GO[0]); } break; /* it_802E1694 */
            case 31: {  it_802E16D8(&GO[0], &GO[1]); } break; /* it_802E16D8 */
            case 32: { s32 v0 = (s32) ((int) (rnd() % 3)); s32 v2 = (s32) ((rnd() & 1) ? 1 : -1); ret = PI(it_802E16F8(v0, &VA[0], v2)); } break; /* it_802E16F8 */
            case 33: { { int j; for (j = 0; j < 8; j++) { HM((u32) it_803F88C0[j].anim_id); HM(FI((const void*) it_803F88C0[j].animated)); HM(FI((const void*) it_803F88C0[j].physics_updated)); HM(FI((const void*) it_803F88C0[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
