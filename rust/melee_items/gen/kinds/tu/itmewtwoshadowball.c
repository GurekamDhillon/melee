/* TU: src/melee/it/kinds/itmewtwoshadowball.c */
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


#include "itmewtwoshadowball_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itMewtwoShadowball_DatAttrs* a = (itMewtwoShadowball_DatAttrs*) SPEC[0];
        a->x0 = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw(); a->x14 = rw(); a->x18 = rw(); a->x1C = rw();
        a->x20 = (int) (rnd() % 6); a->x24 = rw(); a->x28 = rw(); a->x2C = rw(); a->x30 = rw(); a->x34 = rw(); a->x38 = rw(); a->x3C = rw();
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
        JO[3 + k].child = &JO[9 + k];
        IT[k].xDD4_itemVar.mewtwoshadowball.x0 = rw();
        i = (int) (rnd() % 2000) - 1000;
        IT[k].xDD4_itemVar.mewtwoshadowball.x4.x = (float) i / 37.0f; IT[k].xDD4_itemVar.mewtwoshadowball.x4.y = rw(); IT[k].xDD4_itemVar.mewtwoshadowball.x4.z = rw();
        IT[k].xDD4_itemVar.mewtwoshadowball.x10 = rw();
        IT[k].xDD4_itemVar.mewtwoshadowball.x14 = (int) (rnd() % 2);
        IT[k].xDD4_itemVar.mewtwoshadowball.x18 = (int) (rnd() % 6) - 1;
        IT[k].xDD4_itemVar.mewtwoshadowball.x1C = (int) (rnd() % 5) + 1;
        IT[k].xDD4_itemVar.mewtwoshadowball.x20 = (int) (rnd() % 5);
        IT[k].xDD4_itemVar.mewtwoshadowball.x24 = rnd() % 9;
        IT[k].xDD4_itemVar.mewtwoshadowball.x28 = (int) (rnd() % 2);
        IT[k].xDD4_itemVar.mewtwoshadowball.x2C = (rnd() & 3) ? &GO[1] : 0;
        IT[k].xDD4_itemVar.mewtwoshadowball.x30.x = rw(); IT[k].xDD4_itemVar.mewtwoshadowball.x30.y = rw(); IT[k].xDD4_itemVar.mewtwoshadowball.x30.z = rw();
        IT[k].xDD4_itemVar.mewtwoshadowball.x3C = rw(); IT[k].xDD4_itemVar.mewtwoshadowball.x40 = rw(); IT[k].xDD4_itemVar.mewtwoshadowball.x44 = rw();
        IT[k].xDD4_itemVar.mewtwoshadowball.x48 = (int) (rnd() % 6);
        IT[k].xDD4_itemVar.mewtwoshadowball.x4C = (int) (rnd() % 4) - 1;
        IT[k].xDD4_itemVar.mewtwoshadowball.x50 = rw(); IT[k].xDD4_itemVar.mewtwoshadowball.x54 = rw();
        if (rnd() % 3 == 0) { IT[k].xDD4_itemVar.mewtwoshadowball.x58.x = 0.0f; IT[k].xDD4_itemVar.mewtwoshadowball.x58.y = 0.0f; IT[k].xDD4_itemVar.mewtwoshadowball.x58.z = 0.0f; }
        else { IT[k].xDD4_itemVar.mewtwoshadowball.x58.x = rw(); IT[k].xDD4_itemVar.mewtwoshadowball.x58.y = rw(); IT[k].xDD4_itemVar.mewtwoshadowball.x58.z = rw(); }
        IT[k].xDD4_itemVar.mewtwoshadowball.x64 = rw();
        if (k == 0) {
            i = (int) (rnd() % 3);
            if (i == 0) IT[0].kind = It_Kind_Mewtwo_ShadowBall;
            else if (i == 1) IT[0].kind = It_Kind_Kirby_MewtwoShadowBall;
        }
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.mewtwoshadowball.x2C)); IT[k].xDD4_itemVar.mewtwoshadowball.x2C = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 31; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_802C4D10(&GO[0]); } break; /* it_802C4D10 */
            case 1: { ret = (u32) it_802C4F50(&GO[0], &IT[0].x378_itemColl); } break; /* it_802C4F50 */
            case 2: { s32 v2 = (s32) ((int) fv()); s32 v3 = (s32) ((int) fv()); float v4 = (float) (rw()); ret = PI(it_802C5000(&GO[0], &VA[0], v2, v3, v4)); } break; /* it_802C5000 */
            case 3: { s32 v2 = (s32) ((int) fv()); s32 v3 = (s32) ((int) (rnd() % 5) + 1); float v4 = (float) (rw()); float v5 = (float) (rw()); ret = PI(it_802C519C(&GO[0], &VA[0], v2, v3, v4, v5)); } break; /* it_802C519C */
            case 4: { float v2 = (float) (rw()); float v3 = (float) ((float) (rnd() % 9) * 0.5f - 1.0f); float v4 = (float) ((int) (rnd() % 5) + 1); it_802C53F0(&GO[0], &VA[0], v2, v3, v4); } break; /* it_802C53F0 */
            case 5: {  it_2725_Logic101_Destroyed(&GO[0]); } break; /* it_2725_Logic101_Destroyed */
            case 6: {  it_802C573C(&GO[0]); } break; /* it_802C573C */
            case 7: {  it_2725_Logic101_PickedUp(&GO[0]); } break; /* it_2725_Logic101_PickedUp */
            case 8: {  ret = (u32) itMewtwoshadowball_UnkMotion0_Anim(&GO[0]); } break; /* itMewtwoshadowball_UnkMotion0_Anim */
            case 9: {  itMewtwoshadowball_UnkMotion0_Phys(&GO[0]); } break; /* itMewtwoshadowball_UnkMotion0_Phys */
            case 10: {  ret = (u32) itMewtwoshadowball_UnkMotion0_Coll(&GO[0]); } break; /* itMewtwoshadowball_UnkMotion0_Coll */
            case 11: {  it_802C5B18(&GO[0], &GO[1]); } break; /* it_802C5B18 */
            case 12: {  ret = (u32) itMewtwoshadowball_UnkMotion8_Anim(&GO[0]); } break; /* itMewtwoshadowball_UnkMotion8_Anim */
            case 13: {  itMewtwoshadowball_UnkMotion8_Phys(&GO[0]); } break; /* itMewtwoshadowball_UnkMotion8_Phys */
            case 14: {  ret = (u32) itMewtwoshadowball_UnkMotion8_Coll(&GO[0]); } break; /* itMewtwoshadowball_UnkMotion8_Coll */
            case 15: {  ret = (u32) itMewtwoshadowball_UnkMotion9_Anim(&GO[0]); } break; /* itMewtwoshadowball_UnkMotion9_Anim */
            case 16: {  itMewtwoshadowball_UnkMotion9_Phys(&GO[0]); } break; /* itMewtwoshadowball_UnkMotion9_Phys */
            case 17: {  ret = (u32) itMewtwoshadowball_UnkMotion9_Coll(&GO[0]); } break; /* itMewtwoshadowball_UnkMotion9_Coll */
            case 18: {  fn_802C5E18(&GO[0]); } break; /* fn_802C5E18 */
            case 19: {  it_802C5E5C(&GO[0]); } break; /* it_802C5E5C */
            case 20: {  ret = (u32) itMewtwoshadowball_UnkMotion17_Anim(&GO[0]); } break; /* itMewtwoshadowball_UnkMotion17_Anim */
            case 21: {  itMewtwoshadowball_UnkMotion17_Phys(&GO[0]); } break; /* itMewtwoshadowball_UnkMotion17_Phys */
            case 22: {  ret = (u32) itMewtwoshadowball_UnkMotion17_Coll(&GO[0]); } break; /* itMewtwoshadowball_UnkMotion17_Coll */
            case 23: {  ret = (u32) itMewtwoShadowball_Logic101_DmgDealt(&GO[0]); } break; /* itMewtwoShadowball_Logic101_DmgDealt */
            case 24: {  ret = (u32) itMewtwoShadowball_Logic101_Clanked(&GO[0]); } break; /* itMewtwoShadowball_Logic101_Clanked */
            case 25: {  ret = (u32) itMewtwoShadowball_Logic101_Absorbed(&GO[0]); } break; /* itMewtwoShadowball_Logic101_Absorbed */
            case 26: {  ret = (u32) it_2725_Logic101_Reflected(&GO[0]); } break; /* it_2725_Logic101_Reflected */
            case 27: {  ret = (u32) itMewtwoShadowball_Logic101_HitShield(&GO[0]); } break; /* itMewtwoShadowball_Logic101_HitShield */
            case 28: {  ret = (u32) it_2725_Logic101_ShieldBounced(&GO[0]); } break; /* it_2725_Logic101_ShieldBounced */
            case 29: {  itMewtwoShadowball_Logic101_EvtUnk(&GO[0], &GO[1]); } break; /* itMewtwoShadowball_Logic101_EvtUnk */
            case 30: { { int j; for (j = 0; j < 18; j++) { HM((u32) it_803F7760[j].anim_id); HM(FI((const void*) it_803F7760[j].animated)); HM(FI((const void*) it_803F7760[j].physics_updated)); HM(FI((const void*) it_803F7760[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
