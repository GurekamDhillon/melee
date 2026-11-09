/* TU: src/melee/it/kinds/itsamusbomb.c */
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


#include "itsamusbomb_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itSamusBombAttributes* a = (itSamusBombAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = (float) (rnd() % 30); a->x8 = rw(); a->xC = (float) (rnd() % 5) / 2.0f; a->x10 = rw(); a->x14 = rw(); a->x18 = rw();
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
        IT[k].xD44_lifeTimer = (float) ((int) (rnd() % 30) - 2);
        IT[k].xDD4_itemVar.samusbomb.x0 = (int) (rnd() & 1);
        IT[k].xDD4_itemVar.samusbomb.owner = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        if (rnd() & 1) { IT[k].x7C.x = (float) ((int) (rnd() % 7) - 3) / 2.0f; IT[k].x7C.y = (float) ((int) (rnd() % 7) - 3) / 2.0f; }
        if (rnd() & 1) IT[k].x40_vel.x = 0.0f;
        if (k == 0 && (rnd() & 1)) {
            float m = (float) (rnd() % 4 + 1) / 2.0f;
            union { float f; unsigned u; } cv;
            IT[k].x7C.x = 3.0f * m; IT[k].x7C.y = 4.0f * m;
            cv.f = 5.0f * m; cv.u += (unsigned) ((int) (rnd() % 3) - 1);
            ((itSamusBombAttributes*) SPEC[0])->xC = cv.f;
        }
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.samusbomb.owner)); IT[k].xDD4_itemVar.samusbomb.owner = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 24; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: { f32 v2 = (f32) (rw()); ret = PI(it_802B4AC8(&GO[0], &VA[0], v2)); } break; /* it_802B4AC8 */
            case 1: {  it_802B4BA0(&GO[0]); } break; /* it_802B4BA0 */
            case 2: {  it_802B4C10(&GO[0]); } break; /* it_802B4C10 */
            case 3: {  ret = (u32) ((itSamusbomb_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itSamusbomb_UnkMotion0_Anim */
            case 4: {  itSamusbomb_UnkMotion0_Phys(&GO[0]); } break; /* itSamusbomb_UnkMotion0_Phys */
            case 5: {  ret = (u32) ((itSamusbomb_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itSamusbomb_UnkMotion0_Coll */
            case 6: {  it_802B4CF4(&GO[0]); } break; /* it_802B4CF4 */
            case 7: {  ret = (u32) ((itSamusbomb_UnkMotion1_Anim(&GO[0])) != 0); } break; /* itSamusbomb_UnkMotion1_Anim */
            case 8: {  itSamusbomb_UnkMotion1_Phys(&GO[0]); } break; /* itSamusbomb_UnkMotion1_Phys */
            case 9: {  ret = (u32) ((itSamusbomb_UnkMotion1_Coll(&GO[0])) != 0); } break; /* itSamusbomb_UnkMotion1_Coll */
            case 10: {  itSamusBomb_Logic50_EnteredAir(&GO[0]); } break; /* itSamusBomb_Logic50_EnteredAir */
            case 11: {  ret = (u32) ((itSamusbomb_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itSamusbomb_UnkMotion2_Anim */
            case 12: {  itSamusbomb_UnkMotion2_Phys(&GO[0]); } break; /* itSamusbomb_UnkMotion2_Phys */
            case 13: {  ret = (u32) ((itSamusbomb_UnkMotion2_Coll(&GO[0])) != 0); } break; /* itSamusbomb_UnkMotion2_Coll */
            case 14: {  ret = (u32) ((itSamusBomb_Logic50_DmgDealt(&GO[0])) != 0); } break; /* itSamusBomb_Logic50_DmgDealt */
            case 15: {  ret = (u32) ((itSamusBomb_Logic50_Clanked(&GO[0])) != 0); } break; /* itSamusBomb_Logic50_Clanked */
            case 16: {  ret = (u32) ((itSamusBomb_Logic50_HitShield(&GO[0])) != 0); } break; /* itSamusBomb_Logic50_HitShield */
            case 17: {  ret = (u32) ((itSamusBomb_Logic50_ShieldBounced(&GO[0])) != 0); } break; /* itSamusBomb_Logic50_ShieldBounced */
            case 18: {  ret = (u32) ((it_2725_Logic50_Reflected(&GO[0])) != 0); } break; /* it_2725_Logic50_Reflected */
            case 19: {  it_802B53CC(&GO[0]); } break; /* it_802B53CC */
            case 20: {  ret = (u32) ((itSamusbomb_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itSamusbomb_UnkMotion3_Anim */
            case 21: {  it_802B5478(&GO[0]); } break; /* it_802B5478 */
            case 22: {  itSamusBomb_Logic50_EvtUnk(&GO[0], &GO[1]); } break; /* itSamusBomb_Logic50_EvtUnk */
            case 23: { { int j; for (j = 0; j < 4; j++) { HM((u32) it_803F7220[j].anim_id); HM(FI((const void*) it_803F7220[j].animated)); HM(FI((const void*) it_803F7220[j].physics_updated)); HM(FI((const void*) it_803F7220[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
