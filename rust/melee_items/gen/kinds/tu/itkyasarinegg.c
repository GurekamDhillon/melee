/* TU: src/melee/it/kinds/itkyasarinegg.c */
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


#include "itkyasarinegg_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        { int q; for (q = 0; q < 3; q++) {
        itKyasarinEggAttributes* a = (itKyasarinEggAttributes*) SPEC[q];
        a->x0 = 0; a->x4 = rw(); a->x8 = rw(); a->xC = (int) (rnd() % 4); a->x10 = (int) (rnd() % 3);
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
        IT[k].xDD4_itemVar.kyasarinEgg.x20 = (int) (rnd() % 3);
        IT[k].xDD4_itemVar.kyasarinEgg.x24 = rw();
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 21; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: { float v2 = (float) (rw()); it_802EFA44(&GO[0], &VA[0], v2); } break; /* it_802EFA44 */
            case 1: {  it_802EFB0C(&GO[0]); } break; /* it_802EFB0C */
            case 2: {  ret = (u32) itKyasarinegg_UnkMotion0_Coll(&GO[0]); } break; /* itKyasarinegg_UnkMotion0_Coll */
            case 3: {  itKyasarinEgg_Logic28_PickedUp(&GO[0]); } break; /* itKyasarinEgg_Logic28_PickedUp */
            case 4: {  itKyasarinEgg_Logic28_Dropped(&GO[0]); } break; /* itKyasarinEgg_Logic28_Dropped */
            case 5: {  itKyasarinEgg_Logic28_Thrown(&GO[0]); } break; /* itKyasarinEgg_Logic28_Thrown */
            case 6: {  itKyasarinegg_UnkMotion3_Phys(&GO[0]); } break; /* itKyasarinegg_UnkMotion3_Phys */
            case 7: {  ret = (u32) itKyasarinegg_UnkMotion3_Coll(&GO[0]); } break; /* itKyasarinegg_UnkMotion3_Coll */
            case 8: {  it_802EFCC0(&GO[0]); } break; /* it_802EFCC0 */
            case 9: {  itKyasarinegg_UnkMotion1_Phys(&GO[0]); } break; /* itKyasarinegg_UnkMotion1_Phys */
            case 10: {  ret = (u32) itKyasarinegg_UnkMotion1_Coll(&GO[0]); } break; /* itKyasarinegg_UnkMotion1_Coll */
            case 11: {  it_802EFD84(&GO[0]); } break; /* it_802EFD84 */
            case 12: {  itKyasarinegg_UnkMotion4_Anim(&GO[0]); } break; /* itKyasarinegg_UnkMotion4_Anim */
            case 13: {  ret = (u32) it_2725_Logic28_DmgDealt(&GO[0]); } break; /* it_2725_Logic28_DmgDealt */
            case 14: {  ret = (u32) it_2725_Logic28_Clanked(&GO[0]); } break; /* it_2725_Logic28_Clanked */
            case 15: {  ret = (u32) it_2725_Logic28_HitShield(&GO[0]); } break; /* it_2725_Logic28_HitShield */
            case 16: {  ret = (u32) it_2725_Logic28_Reflected(&GO[0]); } break; /* it_2725_Logic28_Reflected */
            case 17: {  ret = (u32) itKyasarinEgg_Logic28_ShieldBounced(&GO[0]); } break; /* itKyasarinEgg_Logic28_ShieldBounced */
            case 18: {  ret = (u32) it_802F022C(&GO[0]); } break; /* it_802F022C */
            case 19: {  it_802F0320(&GO[0], &GO[1]); } break; /* it_802F0320 */
            case 20: { { int j; for (j = 0; j < 5; j++) { HM((u32) it_803F9328[j].anim_id); HM(FI((const void*) it_803F9328[j].animated)); HM(FI((const void*) it_803F9328[j].physics_updated)); HM(FI((const void*) it_803F9328[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
