/* TU: src/melee/it/kinds/itwhitebea.c */
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
static int SFXA[4];
#include "itwhitebea_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itWhiteBeaAttributes* a = (itWhiteBeaAttributes*) SPEC[0];
        a->x0 = (__typeof__(a->x0)) (SPEC[0] + 64);
        a->x0->x0 = (int) (rnd() % 20); a->x0->x4 = rw();
        a->x4 = rw(); a->x8 = (short) (rnd() % 30); a->xA = (short) (rnd() % 30); a->xC = (short) (rnd() % 20);
        a->xE = (short) (a->xC + (int) (rnd() % 25) - 3); a->x10 = rw(); a->x14 = (short) (rnd() % 50);
    } break;
    case 1: {
        itOldottoseaAttributes* a = (itOldottoseaAttributes*) SPEC[0];
        a->x14 = (float) (rnd() % 30); a->x18 = (float) (rnd() % 40); a->x1C = rw(); a->x20 = rw();
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
    zero(&D40, sizeof D40); D40.x8 = (float) (rnd() % 11) / 10.0f; it_804D6D40 = &D40;
    SFXA[0] = -1; for (i = 1; i < 4; i++) SFXA[i] = (int) (rnd() % 500) - 1;
    for (k = 0; k < 3; k++) { IT[k].xDD4_itemVar.whitebea.x20 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0; }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.whitebea.x20)); IT[k].xDD4_itemVar.whitebea.x20 = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 73; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_802E31F8(&GO[0]); } break; /* it_802E31F8 */
            case 1: {  ret = (u32) ((itOldottosea_UnkMotion8_Anim(&GO[0])) != 0); } break; /* itOldottosea_UnkMotion8_Anim */
            case 2: {  itOldottosea_UnkMotion8_Phys(&GO[0]); } break; /* itOldottosea_UnkMotion8_Phys */
            case 3: {  it_802E32B4(&GO[0]); } break; /* it_802E32B4 */
            case 4: {  it_802E3314(&GO[0]); } break; /* it_802E3314 */
            case 5: {  ret = (u32) ((itOldottosea_UnkMotion9_Anim(&GO[0])) != 0); } break; /* itOldottosea_UnkMotion9_Anim */
            case 6: {  itOldottosea_UnkMotion9_Phys(&GO[0]); } break; /* itOldottosea_UnkMotion9_Phys */
            case 7: {  ret = (u32) ((itOldottosea_UnkMotion9_Coll(&GO[0])) != 0); } break; /* itOldottosea_UnkMotion9_Coll */
            case 8: {  it_802E3400(&GO[0]); } break; /* it_802E3400 */
            case 9: {  ret = (u32) ((itOldottosea_UnkMotion10_Anim(&GO[0])) != 0); } break; /* itOldottosea_UnkMotion10_Anim */
            case 10: {  itOldottosea_UnkMotion10_Phys(&GO[0]); } break; /* itOldottosea_UnkMotion10_Phys */
            case 11: {  ret = (u32) ((itOldottosea_UnkMotion10_Coll(&GO[0])) != 0); } break; /* itOldottosea_UnkMotion10_Coll */
            case 12: {  it_2725_Logic3_Destroyed(&GO[0]); } break; /* it_2725_Logic3_Destroyed */
            case 13: {  it_802E3528(&GO[0]); } break; /* it_802E3528 */
            case 14: {  ret = (u32) ((itOldottosea_UnkMotion11_Anim(&GO[0])) != 0); } break; /* itOldottosea_UnkMotion11_Anim */
            case 15: {  itOldottosea_UnkMotion11_Phys(&GO[0]); } break; /* itOldottosea_UnkMotion11_Phys */
            case 16: {  ret = (u32) ((itOldottosea_UnkMotion11_Coll(&GO[0])) != 0); } break; /* itOldottosea_UnkMotion11_Coll */
            case 17: {  ret = (u32) ((it_802E35CC(&GO[0])) != 0); } break; /* it_802E35CC */
            case 18: {  it_802E3784(&GO[0], &GO[1]); } break; /* it_802E3784 */
            case 19: {  it_802E37A4(&GO[0]); } break; /* it_802E37A4 */
            case 20: {  it_802E37BC(&GO[0]); } break; /* it_802E37BC */
            case 21: {  ret = (u32) ((it_802E3884(&GO[0])) != 0); } break; /* it_802E3884 */
            case 22: {  fn_802E398C(&GO[0]); } break; /* fn_802E398C */
            case 23: {  ret = (u32) ((itWhitebea_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion0_Anim */
            case 24: {  itWhitebea_UnkMotion0_Phys(&GO[0]); } break; /* itWhitebea_UnkMotion0_Phys */
            case 25: {  ret = (u32) ((itWhitebea_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion0_Coll */
            case 26: {  it_802E3AC8(&GO[0]); } break; /* it_802E3AC8 */
            case 27: {  ret = (u32) ((itWhitebea_UnkMotion1_Anim(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion1_Anim */
            case 28: {  itWhitebea_UnkMotion1_Phys(&GO[0]); } break; /* itWhitebea_UnkMotion1_Phys */
            case 29: {  ret = (u32) ((itWhitebea_UnkMotion1_Coll(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion1_Coll */
            case 30: {  it_802E3DA0(&GO[0]); } break; /* it_802E3DA0 */
            case 31: {  ret = (u32) ((itWhitebea_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion3_Anim */
            case 32: {  itWhitebea_UnkMotion3_Phys(&GO[0]); } break; /* itWhitebea_UnkMotion3_Phys */
            case 33: {  ret = (u32) ((itWhitebea_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion3_Coll */
            case 34: {  it_802E3ED0(&GO[0]); } break; /* it_802E3ED0 */
            case 35: {  ret = (u32) ((itWhitebea_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion4_Anim */
            case 36: {  itWhitebea_UnkMotion4_Phys(&GO[0]); } break; /* itWhitebea_UnkMotion4_Phys */
            case 37: {  ret = (u32) ((itWhitebea_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion4_Coll */
            case 38: {  it_802E40A4(&GO[0]); } break; /* it_802E40A4 */
            case 39: {  it_802E4190(&GO[0]); } break; /* it_802E4190 */
            case 40: {  ret = (u32) ((itWhitebea_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion2_Anim */
            case 41: {  itWhitebea_UnkMotion2_Phys(&GO[0]); } break; /* itWhitebea_UnkMotion2_Phys */
            case 42: {  ret = (u32) ((itWhitebea_UnkMotion2_Coll(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion2_Coll */
            case 43: {  it_802E436C(&GO[0]); } break; /* it_802E436C */
            case 44: {  ret = (u32) ((itWhitebea_UnkMotion5_Anim(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion5_Anim */
            case 45: {  itWhitebea_UnkMotion5_Phys(&GO[0]); } break; /* itWhitebea_UnkMotion5_Phys */
            case 46: {  ret = (u32) ((itWhitebea_UnkMotion5_Coll(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion5_Coll */
            case 47: {  it_802E4464(&GO[0]); } break; /* it_802E4464 */
            case 48: {  ret = (u32) ((itWhitebea_UnkMotion6_Anim(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion6_Anim */
            case 49: {  itWhitebea_UnkMotion6_Phys(&GO[0]); } break; /* itWhitebea_UnkMotion6_Phys */
            case 50: {  ret = (u32) ((itWhitebea_UnkMotion6_Coll(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion6_Coll */
            case 51: {  it_802E4558(&GO[0]); } break; /* it_802E4558 */
            case 52: {  ret = (u32) ((itWhitebea_UnkMotion7_Anim(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion7_Anim */
            case 53: {  itWhitebea_UnkMotion7_Phys(&GO[0]); } break; /* itWhitebea_UnkMotion7_Phys */
            case 54: {  ret = (u32) ((itWhitebea_UnkMotion7_Coll(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion7_Coll */
            case 55: {  itWhiteBea_Logic9_PickedUp(&GO[0]); } break; /* itWhiteBea_Logic9_PickedUp */
            case 56: {  ret = (u32) ((itWhitebea_UnkMotion8_Anim(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion8_Anim */
            case 57: {  itWhitebea_UnkMotion8_Phys(&GO[0]); } break; /* itWhitebea_UnkMotion8_Phys */
            case 58: {  it_2725_Logic9_Dropped(&GO[0]); } break; /* it_2725_Logic9_Dropped */
            case 59: {  it_2725_Logic9_Thrown(&GO[0]); } break; /* it_2725_Logic9_Thrown */
            case 60: {  ret = (u32) ((itWhitebea_UnkMotion9_Anim(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion9_Anim */
            case 61: {  itWhitebea_UnkMotion9_Phys(&GO[0]); } break; /* itWhitebea_UnkMotion9_Phys */
            case 62: {  ret = (u32) ((itWhitebea_UnkMotion9_Coll(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion9_Coll */
            case 63: {  it_802E48B4(&GO[0]); } break; /* it_802E48B4 */
            case 64: {  ret = (u32) ((itWhitebea_UnkMotion10_Anim(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion10_Anim */
            case 65: {  itWhitebea_UnkMotion10_Phys(&GO[0]); } break; /* itWhitebea_UnkMotion10_Phys */
            case 66: {  ret = (u32) ((itWhitebea_UnkMotion10_Coll(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion10_Coll */
            case 67: {  it_802E4980(&GO[0]); } break; /* it_802E4980 */
            case 68: {  ret = (u32) ((itWhitebea_UnkMotion11_Anim(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion11_Anim */
            case 69: {  itWhitebea_UnkMotion11_Phys(&GO[0]); } break; /* itWhitebea_UnkMotion11_Phys */
            case 70: {  ret = (u32) ((itWhitebea_UnkMotion11_Coll(&GO[0])) != 0); } break; /* itWhitebea_UnkMotion11_Coll */
            case 71: {  it_802E4A24(&GO[0], &GO[1]); } break; /* it_802E4A24 */
            case 72: { { int j; for (j = 0; j < 12; j++) { HM((u32) it_803F8A88[j].anim_id); HM(FI((const void*) it_803F8A88[j].animated)); HM(FI((const void*) it_803F8A88[j].physics_updated)); HM(FI((const void*) it_803F8A88[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
