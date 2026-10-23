/* TU: src/melee/it/kinds/itbat.c */
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


#include "itbat_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itBatAttributes* a = (itBatAttributes*) SPEC[0];
        a->x0 = (int) (rnd() % 5); a->x4 = (int) (rnd() % 5); a->x8 = (int) (rnd() % 5); a->xC_vel = rw();
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

}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;

    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

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
            case 0: {  ret = PI(it_80284854(&VA[0])); } break; /* it_80284854 */
            case 1: {  itBat_Logic11_Spawned(&GO[0]); } break; /* itBat_Logic11_Spawned */
            case 2: {  it_80284974(&GO[0]); } break; /* it_80284974 */
            case 3: {  ret = (u32) itBat_UnkMotion0_Anim(&GO[0]); } break; /* itBat_UnkMotion0_Anim */
            case 4: {  itBat_UnkMotion0_Phys(&GO[0]); } break; /* itBat_UnkMotion0_Phys */
            case 5: {  ret = (u32) itBat_UnkMotion0_Coll(&GO[0]); } break; /* itBat_UnkMotion0_Coll */
            case 6: {  it_802849F0(&GO[0]); } break; /* it_802849F0 */
            case 7: {  ret = (u32) itBat_UnkMotion3_Anim(&GO[0]); } break; /* itBat_UnkMotion3_Anim */
            case 8: {  itBat_UnkMotion3_Phys(&GO[0]); } break; /* itBat_UnkMotion3_Phys */
            case 9: {  ret = (u32) itBat_UnkMotion3_Coll(&GO[0]); } break; /* itBat_UnkMotion3_Coll */
            case 10: {  itBat_Logic11_PickedUp(&GO[0]); } break; /* itBat_Logic11_PickedUp */
            case 11: {  ret = (u32) itBat_UnkMotion2_Anim(&GO[0]); } break; /* itBat_UnkMotion2_Anim */
            case 12: {  itBat_UnkMotion2_Phys(&GO[0]); } break; /* itBat_UnkMotion2_Phys */
            case 13: {  ret = (u32) itBat_UnkMotion2_Coll(&GO[0]); } break; /* itBat_UnkMotion2_Coll */
            case 14: {  itBat_Logic11_Dropped(&GO[0]); } break; /* itBat_Logic11_Dropped */
            case 15: {  itBat_Logic11_Thrown(&GO[0]); } break; /* itBat_Logic11_Thrown */
            case 16: {  itBat_Logic11_EnteredAir(&GO[0]); } break; /* itBat_Logic11_EnteredAir */
            case 17: {  ret = (u32) itBat_UnkMotion4_Anim(&GO[0]); } break; /* itBat_UnkMotion4_Anim */
            case 18: {  itBat_UnkMotion4_Phys(&GO[0]); } break; /* itBat_UnkMotion4_Phys */
            case 19: {  ret = (u32) itBat_UnkMotion4_Coll(&GO[0]); } break; /* itBat_UnkMotion4_Coll */
            case 20: {  ret = (u32) itBat_Logic11_DmgDealt(&GO[0]); } break; /* itBat_Logic11_DmgDealt */
            case 21: {  ret = (u32) itBat_Logic11_Reflected(&GO[0]); } break; /* itBat_Logic11_Reflected */
            case 22: {  ret = (u32) itBat_Logic11_Clanked(&GO[0]); } break; /* itBat_Logic11_Clanked */
            case 23: {  ret = (u32) itBat_Logic11_HitShield(&GO[0]); } break; /* itBat_Logic11_HitShield */
            case 24: {  ret = (u32) itBat_Logic11_ShieldBounced(&GO[0]); } break; /* itBat_Logic11_ShieldBounced */
            case 25: {  itBat_Logic11_EvtUnk(&GO[0], &GO[1]); } break; /* itBat_Logic11_EvtUnk */
            case 26: { { int j; for (j = 0; j < 5; j++) { HM((u32) it_803F57B0[j].anim_id); HM(FI((const void*) it_803F57B0[j].animated)); HM(FI((const void*) it_803F57B0[j].physics_updated)); HM(FI((const void*) it_803F57B0[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
