/* TU: src/melee/it/kinds/itlgunbeam.c */
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


#include "itlgunbeam_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        { int q; for (q = 0; q < 3; q++) {
        ItLGunBeamAttr* a = (ItLGunBeamAttr*) SPEC[q];
        a->lifetime = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw();
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

    IT[0].owner = (rnd() & 3) ? &GO[1] : 0;
    IT[0].xC64_reflectGObj = (rnd() & 3) ? &GO[1] : 0;
    if (rnd() & 1) IT[0].msid = (int) (rnd() % 14);
    if (rnd() & 1) IT[0].facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
    IT[0].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
    spec_fill(prof);
    for (k = 0; k < 3; k++) {
        IT[k].xDD4_itemVar.lgunbeam.angle0 = rw(); IT[k].xDD4_itemVar.lgunbeam.angle1 = rw();
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 15; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: { int v1 = (int) ((int) fv()); it_802993E0(&GO[0], v1); } break; /* it_802993E0 */
            case 1: { int v1 = (int) ((int) fv()); it_80299528(&GO[0], v1); } break; /* it_80299528 */
            case 2: { u32 v2 = (u32) (fv()); f32 v3 = (f32) (rw()); ret = PI(it_802996D0(&GO[0], &VA[0], v2, v3)); } break; /* it_802996D0 */
            case 3: { s32 v2 = (s32) ((int) fv()); it_802998A0(&GO[0], &GO[1], v2); } break; /* it_802998A0 */
            case 4: {  ret = (u32) itLgunbeam_UnkMotion0_Anim(&GO[0]); } break; /* itLgunbeam_UnkMotion0_Anim */
            case 5: {  itLgunbeam_UnkMotion0_Phys(&GO[0]); } break; /* itLgunbeam_UnkMotion0_Phys */
            case 6: {  ret = (u32) itLgunbeam_UnkMotion0_Coll(&GO[0]); } break; /* itLgunbeam_UnkMotion0_Coll */
            case 7: {  ret = (u32) itLGunBeam_Logic39_DmgDealt(&GO[0]); } break; /* itLGunBeam_Logic39_DmgDealt */
            case 8: {  ret = (u32) itLGunBeam_Logic39_Reflected(&GO[0]); } break; /* itLGunBeam_Logic39_Reflected */
            case 9: {  ret = (u32) itLGunBeam_Logic39_Clanked(&GO[0]); } break; /* itLGunBeam_Logic39_Clanked */
            case 10: {  ret = (u32) itLGunBeam_Logic39_Absorbed(&GO[0]); } break; /* itLGunBeam_Logic39_Absorbed */
            case 11: {  ret = (u32) itLGunBeam_Logic39_ShieldBounced(&GO[0]); } break; /* itLGunBeam_Logic39_ShieldBounced */
            case 12: {  ret = (u32) itLGunBeam_Logic39_HitShield(&GO[0]); } break; /* itLGunBeam_Logic39_HitShield */
            case 13: {  itLGunBeam_Logic39_EvtUnk(&GO[0], &GO[1]); } break; /* itLGunBeam_Logic39_EvtUnk */
            case 14: { { int j; for (j = 0; j < 1; j++) { HM((u32) it_803F6630[j].anim_id); HM(FI((const void*) it_803F6630[j].animated)); HM(FI((const void*) it_803F6630[j].physics_updated)); HM(FI((const void*) it_803F6630[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
