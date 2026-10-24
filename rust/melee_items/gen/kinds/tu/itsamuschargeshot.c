/* TU: src/melee/it/kinds/itsamuschargeshot.c */
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


#include "itsamuschargeshot_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itSamusChargeShot_Attributes* a = (itSamusChargeShot_Attributes*) SPEC[0];
        a->lifetime = rw(); a->x4 = fv(); a->x8 = rw(); a->xC = rw(); a->x10 = rw(); a->x14 = rw(); a->x18 = rw(); a->x1C = rw();
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
        switch (rnd() % 3) { case 0: IT[k].kind = It_Kind_Samus_Charge; break; case 1: IT[k].kind = It_Kind_Kirby_SamusCharge; break; default: break; }
        IT[k].xDD4_itemVar.samuschargeshot.xE00 = (rnd() & 3) ? IT[k].owner : &GO[2];
        IT[k].xDD4_itemVar.samuschargeshot.xDEC = (int) (rnd() % 11) - 2;
        IT[k].xDD4_itemVar.samuschargeshot.xDF0 = (int) (rnd() % 9);
        IT[k].xDD4_itemVar.samuschargeshot.xDF4 = (int) (rnd() % 5);
        IT[k].xDD4_itemVar.samuschargeshot.xDE8 = (int) (rnd() & 1);
        IT[k].xDD4_itemVar.samuschargeshot.xDD8 = rw();
        IT[k].xDD4_itemVar.samuschargeshot.xDDC = rw();
        IT[k].xDD4_itemVar.samuschargeshot.xDE4 = rw();
        JO[3 + k].child = &JO[40 + k];
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.samuschargeshot.xE00)); IT[k].xDD4_itemVar.samuschargeshot.xE00 = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 22; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: { ret = (u32) it_802B5518(&GO[0], &IT[0].x378_itemColl); } break; /* it_802B5518 */
            case 1: { enum Fighter_Part v2 = (enum Fighter_Part) ((int) fv()); s32 v3 = (s32) ((int) fv()); f32 v4 = (f32) (rw()); ret = PI(it_802B55C8(&GO[0], &VA[0], v2, v3, v4)); } break; /* it_802B55C8 */
            case 2: { f32 v2 = (f32) (rw()); f32 v3 = (f32) (rw()); f32 v4 = (f32) (rw()); it_802B56E4(&GO[0], &VA[0], v2, v3, v4); } break; /* it_802B56E4 */
            case 3: {  it_2725_Logic108_Destroyed(&GO[0]); } break; /* it_2725_Logic108_Destroyed */
            case 4: {  it_802B5974(&GO[0]); } break; /* it_802B5974 */
            case 5: {  it_2725_Logic108_PickedUp(&GO[0]); } break; /* it_2725_Logic108_PickedUp */
            case 6: {  ret = (u32) itSamuschargeshot_UnkMotion0_Anim(&GO[0]); } break; /* itSamuschargeshot_UnkMotion0_Anim */
            case 7: {  itSamuschargeshot_UnkMotion0_Phys(&GO[0]); } break; /* itSamuschargeshot_UnkMotion0_Phys */
            case 8: {  ret = (u32) itSamuschargeshot_UnkMotion0_Coll(&GO[0]); } break; /* itSamuschargeshot_UnkMotion0_Coll */
            case 9: {  it_802B5CBC(&GO[0], &GO[1]); } break; /* it_802B5CBC */
            case 10: {  ret = (u32) itSamuschargeshot_UnkMotion8_Anim(&GO[0]); } break; /* itSamuschargeshot_UnkMotion8_Anim */
            case 11: {  itSamuschargeshot_UnkMotion8_Phys(&GO[0]); } break; /* itSamuschargeshot_UnkMotion8_Phys */
            case 12: {  ret = (u32) itSamuschargeshot_UnkMotion8_Coll(&GO[0]); } break; /* itSamuschargeshot_UnkMotion8_Coll */
            case 13: {  it_802B5EDC(&GO[0]); } break; /* it_802B5EDC */
            case 14: {  ret = (u32) itSamusChargeshot_Logic108_DmgDealt(&GO[0]); } break; /* itSamusChargeshot_Logic108_DmgDealt */
            case 15: {  ret = (u32) itSamusChargeshot_Logic108_Clanked(&GO[0]); } break; /* itSamusChargeshot_Logic108_Clanked */
            case 16: {  ret = (u32) itSamusChargeshot_Logic108_Absorbed(&GO[0]); } break; /* itSamusChargeshot_Logic108_Absorbed */
            case 17: {  ret = (u32) it_2725_Logic108_Reflected(&GO[0]); } break; /* it_2725_Logic108_Reflected */
            case 18: {  ret = (u32) itSamusChargeshot_Logic108_HitShield(&GO[0]); } break; /* itSamusChargeshot_Logic108_HitShield */
            case 19: {  ret = (u32) it_2725_Logic108_ShieldBounced(&GO[0]); } break; /* it_2725_Logic108_ShieldBounced */
            case 20: {  itSamusChargeshot_Logic108_EvtUnk(&GO[0], &GO[1]); } break; /* itSamusChargeshot_Logic108_EvtUnk */
            case 21: { { int j; for (j = 0; j < 9; j++) { HM((u32) it_803F7288[j].anim_id); HM(FI((const void*) it_803F7288[j].animated)); HM(FI((const void*) it_803F7288[j].physics_updated)); HM(FI((const void*) it_803F7288[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
