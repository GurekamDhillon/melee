/* TU: src/melee/it/kinds/itoldkuri.c */
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
#include "itoldkuri_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itOldkuriAttributes* a = (itOldkuriAttributes*) SPEC[0];
        a->x0 = (s32*) (SPEC[0] + 64); *a->x0 = (int) (rnd() % 30); ((float*) a->x0)[1] = rw();
        a->x4 = rw(); a->x8 = rw(); a->xC = (float) (rnd() % 10);
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
    it_804D6D40 = &D40; D40.x8 = (float) (rnd() % 1000) / 1000.0f;
    for (k = 0; k < 3; k++) {
        IT[k].xDD4_itemVar.oldkuri.xDF8 = (int) (rnd() % 3);
        IT[k].xDD4_itemVar.oldkuri.xDFC = (int) (rnd() & 1);
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 53; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  itOldKuri_Logic29_EvtUnk(&GO[0], &GO[1]); } break; /* itOldKuri_Logic29_EvtUnk */
            case 1: {  it_802D73F0(&GO[0]); } break; /* it_802D73F0 */
            case 2: {  it_802D747C(&GO[0]); } break; /* it_802D747C */
            case 3: {  ret = (u32) itOldkuri_UnkMotion0_Anim(&GO[0]); } break; /* itOldkuri_UnkMotion0_Anim */
            case 4: {  itOldkuri_UnkMotion0_Phys(&GO[0]); } break; /* itOldkuri_UnkMotion0_Phys */
            case 5: {  ret = (u32) itOldkuri_UnkMotion0_Coll(&GO[0]); } break; /* itOldkuri_UnkMotion0_Coll */
            case 6: {  it_802D758C(&GO[0]); } break; /* it_802D758C */
            case 7: {  ret = (u32) itOldkuri_UnkMotion1_Anim(&GO[0]); } break; /* itOldkuri_UnkMotion1_Anim */
            case 8: {  itOldkuri_UnkMotion1_Phys(&GO[0]); } break; /* itOldkuri_UnkMotion1_Phys */
            case 9: {  ret = (u32) itOldkuri_UnkMotion1_Coll(&GO[0]); } break; /* itOldkuri_UnkMotion1_Coll */
            case 10: {  it_802D775C(&GO[0]); } break; /* it_802D775C */
            case 11: {  ret = (u32) itOldkuri_UnkMotion2_Anim(&GO[0]); } break; /* itOldkuri_UnkMotion2_Anim */
            case 12: {  itOldkuri_UnkMotion2_Phys(&GO[0]); } break; /* itOldkuri_UnkMotion2_Phys */
            case 13: {  ret = (u32) itOldkuri_UnkMotion2_Coll(&GO[0]); } break; /* itOldkuri_UnkMotion2_Coll */
            case 14: {  ret = (u32) itOldkuri_UnkMotion3_Anim(&GO[0]); } break; /* itOldkuri_UnkMotion3_Anim */
            case 15: {  itOldkuri_UnkMotion3_Phys(&GO[0]); } break; /* itOldkuri_UnkMotion3_Phys */
            case 16: {  ret = (u32) itOldkuri_UnkMotion3_Coll(&GO[0]); } break; /* itOldkuri_UnkMotion3_Coll */
            case 17: {  it_802D7AF0(&GO[0]); } break; /* it_802D7AF0 */
            case 18: {  ret = (u32) itOldkuri_UnkMotion4_Anim(&GO[0]); } break; /* itOldkuri_UnkMotion4_Anim */
            case 19: {  itOldkuri_UnkMotion4_Phys(&GO[0]); } break; /* itOldkuri_UnkMotion4_Phys */
            case 20: {  ret = (u32) itOldkuri_UnkMotion4_Coll(&GO[0]); } break; /* itOldkuri_UnkMotion4_Coll */
            case 21: {  itOldKuri_Logic0_PickedUp(&GO[0]); } break; /* itOldKuri_Logic0_PickedUp */
            case 22: {  ret = (u32) itOldkuri_UnkMotion5_Anim(&GO[0]); } break; /* itOldkuri_UnkMotion5_Anim */
            case 23: {  itOldkuri_UnkMotion5_Phys(&GO[0]); } break; /* itOldkuri_UnkMotion5_Phys */
            case 24: {  it_2725_Logic0_Dropped(&GO[0]); } break; /* it_2725_Logic0_Dropped */
            case 25: {  it_2725_Logic0_Thrown(&GO[0]); } break; /* it_2725_Logic0_Thrown */
            case 26: {  ret = (u32) itOldkuri_UnkMotion6_Anim(&GO[0]); } break; /* itOldkuri_UnkMotion6_Anim */
            case 27: {  itOldkuri_UnkMotion6_Phys(&GO[0]); } break; /* itOldkuri_UnkMotion6_Phys */
            case 28: {  ret = (u32) itOldkuri_UnkMotion6_Coll(&GO[0]); } break; /* itOldkuri_UnkMotion6_Coll */
            case 29: {  ret = (u32) itOldkuri_UnkMotion9_Anim(&GO[0]); } break; /* itOldkuri_UnkMotion9_Anim */
            case 30: {  itOldkuri_UnkMotion9_Phys(&GO[0]); } break; /* itOldkuri_UnkMotion9_Phys */
            case 31: {  ret = (u32) itOldkuri_UnkMotion9_Coll(&GO[0]); } break; /* itOldkuri_UnkMotion9_Coll */
            case 32: {  ret = (u32) it_2725_Logic0_DmgReceived(&GO[0]); } break; /* it_2725_Logic0_DmgReceived */
            case 33: {  ret = (u32) it_802D8098(&GO[0]); } break; /* it_802D8098 */
            case 34: {  it_802D813C(&GO[0]); } break; /* it_802D813C */
            case 35: {  ret = (u32) itOldkuri_UnkMotion7_Anim(&GO[0]); } break; /* itOldkuri_UnkMotion7_Anim */
            case 36: {  itOldkuri_UnkMotion7_Phys(&GO[0]); } break; /* itOldkuri_UnkMotion7_Phys */
            case 37: {  ret = (u32) itOldkuri_UnkMotion7_Coll(&GO[0]); } break; /* itOldkuri_UnkMotion7_Coll */
            case 38: {  it_802D81FC(&GO[0]); } break; /* it_802D81FC */
            case 39: {  ret = (u32) itOldkuri_UnkMotion8_Anim(&GO[0]); } break; /* itOldkuri_UnkMotion8_Anim */
            case 40: {  itOldkuri_UnkMotion8_Phys(&GO[0]); } break; /* itOldkuri_UnkMotion8_Phys */
            case 41: {  ret = (u32) itOldkuri_UnkMotion8_Coll(&GO[0]); } break; /* itOldkuri_UnkMotion8_Coll */
            case 42: {  it_802D82C4(&GO[0]); } break; /* it_802D82C4 */
            case 43: {  ret = (u32) itOldkuri_UnkMotion10_Anim(&GO[0]); } break; /* itOldkuri_UnkMotion10_Anim */
            case 44: {  itOldkuri_UnkMotion10_Phys(&GO[0]); } break; /* itOldkuri_UnkMotion10_Phys */
            case 45: {  ret = (u32) itOldkuri_UnkMotion10_Coll(&GO[0]); } break; /* itOldkuri_UnkMotion10_Coll */
            case 46: {  it_802D839C(&GO[0]); } break; /* it_802D839C */
            case 47: {  ret = (u32) itOldkuri_UnkMotion11_Anim(&GO[0]); } break; /* itOldkuri_UnkMotion11_Anim */
            case 48: {  itOldkuri_UnkMotion11_Phys(&GO[0]); } break; /* itOldkuri_UnkMotion11_Phys */
            case 49: {  ret = (u32) itOldkuri_UnkMotion11_Coll(&GO[0]); } break; /* itOldkuri_UnkMotion11_Coll */
            case 50: { s32 v1 = (s32) ((int) fv()); Item_StateChangeFlags v2 = (Item_StateChangeFlags) ((int) fv()); it_802D848C(&GO[0], v1, v2); } break; /* it_802D848C */
            case 51: {  it_802D84D8(&GO[0], &GO[1]); } break; /* it_802D84D8 */
            case 52: { { int j; for (j = 0; j < 12; j++) { HM((u32) it_803F8320[j].anim_id); HM(FI((const void*) it_803F8320[j].animated)); HM(FI((const void*) it_803F8320[j].physics_updated)); HM(FI((const void*) it_803F8320[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
