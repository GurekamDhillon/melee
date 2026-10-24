/* TU: src/melee/it/kinds/itstarrod.c */
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


#include "itstarrod_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        int* a = (int*) SPEC[0];
        a[0] = (int) (rnd() % 4); ((float*) a)[1] = rw(); ((float*) a)[2] = rw(); ((float*) a)[3] = rw();
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
        IT[k].xD4C = (int) (rnd() % 4) - 1;
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 30; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  itStarRod_Logic22_Spawned(&GO[0]); } break; /* itStarRod_Logic22_Spawned */
            case 1: {  it_80292394(&GO[0], &VA[0]); } break; /* it_80292394 */
            case 2: { float v2 = (float) (rw()); int v3 = (int) ((int) (rnd() % 4)); it_802923BC(&GO[0], &VA[0], v2, v3); } break; /* it_802923BC */
            case 3: {  it_802923F8(&GO[0]); } break; /* it_802923F8 */
            case 4: {  ret = (u32) ((itStarrod_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itStarrod_UnkMotion0_Anim */
            case 5: {  itStarrod_UnkMotion0_Phys(&GO[0]); } break; /* itStarrod_UnkMotion0_Phys */
            case 6: {  ret = (u32) ((itStarrod_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itStarrod_UnkMotion0_Coll */
            case 7: {  it_80292488(&GO[0]); } break; /* it_80292488 */
            case 8: {  ret = (u32) ((itStarrod_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itStarrod_UnkMotion4_Anim */
            case 9: {  itStarrod_UnkMotion1_Phys(&GO[0]); } break; /* itStarrod_UnkMotion1_Phys */
            case 10: {  ret = (u32) ((itStarrod_UnkMotion1_Coll(&GO[0])) != 0); } break; /* itStarrod_UnkMotion1_Coll */
            case 11: {  itStarRod_Logic22_PickedUp(&GO[0]); } break; /* itStarRod_Logic22_PickedUp */
            case 12: {  ret = (u32) ((itStarrod_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itStarrod_UnkMotion2_Anim */
            case 13: {  itStarrod_UnkMotion2_Phys(&GO[0]); } break; /* itStarrod_UnkMotion2_Phys */
            case 14: {  itStarRod_Logic22_Dropped(&GO[0]); } break; /* itStarRod_Logic22_Dropped */
            case 15: {  ret = (u32) ((itStarrod_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itStarrod_UnkMotion4_Coll */
            case 16: {  itStarRod_Logic22_Thrown(&GO[0]); } break; /* itStarRod_Logic22_Thrown */
            case 17: {  itStarrod_UnkMotion4_Phys(&GO[0]); } break; /* itStarrod_UnkMotion4_Phys */
            case 18: {  ret = (u32) ((itStarrod_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itStarrod_UnkMotion3_Coll */
            case 19: {  ret = (u32) ((itStarRod_Logic22_DmgDealt(&GO[0])) != 0); } break; /* itStarRod_Logic22_DmgDealt */
            case 20: {  itStarRod_Logic22_EnteredAir(&GO[0]); } break; /* itStarRod_Logic22_EnteredAir */
            case 21: {  ret = (u32) ((itStarrod_UnkMotion5_Anim(&GO[0])) != 0); } break; /* itStarrod_UnkMotion5_Anim */
            case 22: {  itStarrod_UnkMotion5_Phys(&GO[0]); } break; /* itStarrod_UnkMotion5_Phys */
            case 23: {  ret = (u32) ((itStarrod_UnkMotion5_Coll(&GO[0])) != 0); } break; /* itStarrod_UnkMotion5_Coll */
            case 24: {  ret = (u32) ((itStarRod_Logic22_Clanked(&GO[0])) != 0); } break; /* itStarRod_Logic22_Clanked */
            case 25: {  ret = (u32) ((itStarRod_Logic22_Reflected(&GO[0])) != 0); } break; /* itStarRod_Logic22_Reflected */
            case 26: {  ret = (u32) ((itStarRod_Logic22_HitShield(&GO[0])) != 0); } break; /* itStarRod_Logic22_HitShield */
            case 27: {  ret = (u32) ((itStarRod_Logic22_ShieldBounced(&GO[0])) != 0); } break; /* itStarRod_Logic22_ShieldBounced */
            case 28: {  itStarRod_Logic22_EvtUnk(&GO[0], &GO[1]); } break; /* itStarRod_Logic22_EvtUnk */
            case 29: { { int j; for (j = 0; j < 6; j++) { HM((u32) it_803F5F90[j].anim_id); HM(FI((const void*) it_803F5F90[j].animated)); HM(FI((const void*) it_803F5F90[j].physics_updated)); HM(FI((const void*) it_803F5F90[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
