/* TU: src/melee/it/kinds/itseakneedlethrown.c */
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


#include "itseakneedlethrown_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itSeakNeedleThrownAttributes* a = (itSeakNeedleThrownAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = rw(); a->x8 = rw();
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
        IT[k].xDD4_itemVar.seakneedlethrown.xDD4 = rw(); IT[k].xDD4_itemVar.seakneedlethrown.xDD8 = rw(); IT[k].xDD4_itemVar.seakneedlethrown.xDDC = rw(); IT[k].xDD4_itemVar.seakneedlethrown.xDE0 = rw();
        IT[k].xDD4_itemVar.seakneedlethrown.xDE4.x = rw(); IT[k].xDD4_itemVar.seakneedlethrown.xDE4.y = rw(); IT[k].xDD4_itemVar.seakneedlethrown.xDE4.z = rw();
        IT[k].xDD4_itemVar.seakneedlethrown.xDF0 = rw();
        IT[k].xDD4_itemVar.seakneedlethrown.xDF4 = (rnd() & 1) ? -1 : (int) (rnd() % 8);
        IT[k].xDD4_itemVar.seakneedlethrown.xDF8 = (rnd() & 1) ? 0.5f : rw(); IT[k].xDD4_itemVar.seakneedlethrown.xDFC = (rnd() & 1) ? 0.5f : rw();
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 28; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: { u32 v2 = (u32) (fv()); float v3 = (float) (rw()); ret = PI(it_802AFD8C(&GO[0], &VA[0], v2, v3)); } break; /* it_802AFD8C */
            case 1: { u32 v2 = (u32) ((rnd() % 3)); it_802AFEA8(&GO[0], &GO[1], v2); } break; /* it_802AFEA8 */
            case 2: {  itSeakNeedleThrown_Logic109_Destroyed(&GO[0]); } break; /* itSeakNeedleThrown_Logic109_Destroyed */
            case 3: {  it_802AFF08(&GO[0], &GO[1]); } break; /* it_802AFF08 */
            case 4: {  it_802B00F4(&GO[0], &GO[1]); } break; /* it_802B00F4 */
            case 5: {  ret = (u32) itSeakneedlethrown_UnkMotion0_Anim(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion0_Anim */
            case 6: {  ret = (u32) itSeakneedlethrown_UnkMotion1_Anim(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion1_Anim */
            case 7: {  ret = (u32) itSeakneedlethrown_UnkMotion2_Anim(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion2_Anim */
            case 8: {  ret = (u32) itSeakneedlethrown_UnkMotion3_Anim(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion3_Anim */
            case 9: {  ret = (u32) itSeakneedlethrown_UnkMotion4_Anim(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion4_Anim */
            case 10: {  itSeakneedlethrown_UnkMotion0_Phys(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion0_Phys */
            case 11: {  itSeakneedlethrown_UnkMotion1_Phys(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion1_Phys */
            case 12: {  itSeakneedlethrown_UnkMotion2_Phys(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion2_Phys */
            case 13: {  itSeakneedlethrown_UnkMotion3_Phys(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion3_Phys */
            case 14: {  itSeakneedlethrown_UnkMotion4_Phys(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion4_Phys */
            case 15: {  ret = (u32) itSeakneedlethrown_UnkMotion0_Coll(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion0_Coll */
            case 16: {  ret = (u32) itSeakneedlethrown_UnkMotion1_Coll(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion1_Coll */
            case 17: {  ret = (u32) itSeakneedlethrown_UnkMotion2_Coll(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion2_Coll */
            case 18: {  ret = (u32) itSeakneedlethrown_UnkMotion3_Coll(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion3_Coll */
            case 19: {  ret = (u32) itSeakneedlethrown_UnkMotion4_Coll(&GO[0]); } break; /* itSeakneedlethrown_UnkMotion4_Coll */
            case 20: {  ret = (u32) it_2725_Logic109_DmgDealt(&GO[0]); } break; /* it_2725_Logic109_DmgDealt */
            case 21: {  ret = (u32) it_2725_Logic109_Clanked(&GO[0]); } break; /* it_2725_Logic109_Clanked */
            case 22: {  ret = (u32) it_2725_Logic109_DmgReceived(&GO[0]); } break; /* it_2725_Logic109_DmgReceived */
            case 23: {  ret = (u32) it_2725_Logic109_Reflected(&GO[0]); } break; /* it_2725_Logic109_Reflected */
            case 24: {  ret = (u32) it_2725_Logic109_ShieldBounced(&GO[0]); } break; /* it_2725_Logic109_ShieldBounced */
            case 25: {  ret = (u32) it_2725_Logic109_HitShield(&GO[0]); } break; /* it_2725_Logic109_HitShield */
            case 26: {  itSeakNeedleThrown_Logic109_EvtUnk(&GO[0], &GO[1]); } break; /* itSeakNeedleThrown_Logic109_EvtUnk */
            case 27: { { int j; for (j = 0; j < 5; j++) { HM((u32) it_803F6F50[j].anim_id); HM(FI((const void*) it_803F6F50[j].animated)); HM(FI((const void*) it_803F6F50[j].physics_updated)); HM(FI((const void*) it_803F6F50[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
