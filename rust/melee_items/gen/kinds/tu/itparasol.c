/* TU: src/melee/it/kinds/itparasol.c */
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


#include "itparasol_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        float* a = (float*) SPEC[0];
        a[0] = rw(); a[1] = rw(); a[2] = rw(); a[3] = rw();
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
            case 0: { int v1 = (int) (((const int[]){1, 2, 4, 5, 6, 7, 8, 9, 10})[rnd() % 9]); ret = (u32) it_8028B08C(&GO[0], v1); } break; /* it_8028B08C */
            case 1: {  itParasol_Logic13_Spawned(&GO[0]); } break; /* itParasol_Logic13_Spawned */
            case 2: {  it_8028B0EC(&GO[0]); } break; /* it_8028B0EC */
            case 3: {  ret = (u32) ((itParasol_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itParasol_UnkMotion0_Anim */
            case 4: {  itParasol_UnkMotion0_Phys(&GO[0]); } break; /* itParasol_UnkMotion0_Phys */
            case 5: {  ret = (u32) ((itParasol_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itParasol_UnkMotion0_Coll */
            case 6: {  it_8028B17C(&GO[0]); } break; /* it_8028B17C */
            case 7: {  ret = (u32) ((itParasol_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itParasol_UnkMotion2_Anim */
            case 8: {  itParasol_UnkMotion1_Phys(&GO[0]); } break; /* itParasol_UnkMotion1_Phys */
            case 9: {  ret = (u32) ((itParasol_UnkMotion2_Coll(&GO[0])) != 0); } break; /* itParasol_UnkMotion2_Coll */
            case 10: {  itParasol_Logic13_Dropped(&GO[0]); } break; /* itParasol_Logic13_Dropped */
            case 11: {  itParasol_Logic13_Thrown(&GO[0]); } break; /* itParasol_Logic13_Thrown */
            case 12: {  itParasol_UnkMotion2_Phys(&GO[0]); } break; /* itParasol_UnkMotion2_Phys */
            case 13: {  ret = (u32) ((itParasol_Logic13_DmgDealt(&GO[0])) != 0); } break; /* itParasol_Logic13_DmgDealt */
            case 14: {  ret = (u32) ((itParasol_Logic13_Clanked(&GO[0])) != 0); } break; /* itParasol_Logic13_Clanked */
            case 15: {  ret = (u32) ((itParasol_Logic13_HitShield(&GO[0])) != 0); } break; /* itParasol_Logic13_HitShield */
            case 16: {  ret = (u32) ((itParasol_Logic13_Reflected(&GO[0])) != 0); } break; /* itParasol_Logic13_Reflected */
            case 17: {  ret = (u32) ((itParasol_Logic13_ShieldBounced(&GO[0])) != 0); } break; /* itParasol_Logic13_ShieldBounced */
            case 18: {  itParasol_Logic13_EnteredAir(&GO[0]); } break; /* itParasol_Logic13_EnteredAir */
            case 19: {  ret = (u32) ((itParasol_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itParasol_UnkMotion3_Anim */
            case 20: {  itParasol_UnkMotion3_Phys(&GO[0]); } break; /* itParasol_UnkMotion3_Phys */
            case 21: {  ret = (u32) ((itParasol_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itParasol_UnkMotion3_Coll */
            case 22: {  itParasol_Logic13_PickedUp(&GO[0]); } break; /* itParasol_Logic13_PickedUp */
            case 23: {  ret = (u32) ((itParasol_UnkMotion10_Anim(&GO[0])) != 0); } break; /* itParasol_UnkMotion10_Anim */
            case 24: {  itParasol_UnkMotion10_Phys(&GO[0]); } break; /* itParasol_UnkMotion10_Phys */
            case 25: { f32 v1 = (f32) (rw()); it_8028B618(&GO[0], v1); } break; /* it_8028B618 */
            case 26: { f32 v1 = (f32) (rw()); it_8028B648(&GO[0], v1); } break; /* it_8028B648 */
            case 27: { f32 v1 = (f32) (rw()); it_8028B6B0(&GO[0], v1); } break; /* it_8028B6B0 */
            case 28: { f32 v1 = (f32) (rw()); it_8028B718(&GO[0], v1); } break; /* it_8028B718 */
            case 29: { f32 v1 = (f32) (rw()); it_8028B780(&GO[0], v1); } break; /* it_8028B780 */
            case 30: { f32 v1 = (f32) (rw()); it_8028B7E8(&GO[0], v1); } break; /* it_8028B7E8 */
            case 31: { f32 v1 = (f32) (rw()); it_8028B850(&GO[0], v1); } break; /* it_8028B850 */
            case 32: {  itParasol_Logic13_EvtUnk(&GO[0], &GO[1]); } break; /* itParasol_Logic13_EvtUnk */
            case 33: { { int j; for (j = 0; j < 11; j++) { HM((u32) it_803F5AB0[j].anim_id); HM(FI((const void*) it_803F5AB0[j].animated)); HM(FI((const void*) it_803F5AB0[j].physics_updated)); HM(FI((const void*) it_803F5AB0[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
