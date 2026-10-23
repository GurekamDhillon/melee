/* TU: src/melee/it/kinds/itlinkbomb.c */
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

static ItemStateArray ISA[3];
#include "itlinkbomb_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        { int q; for (q = 0; q < 3; q++) {
        itLinkBombAttributes* a = (itLinkBombAttributes*) SPEC[q];
        a->lifetime = rnd() % 100; a->x4 = rw(); a->x8 = rw(); a->xC = rnd() % 60; a->x10 = (int) (rnd() % 10);
        a->x14 = rw(); a->x18 = rw(); a->x1C = rw(); a->x20 = rw(); a->x24 = rw(); a->x28 = rw(); a->x2C = rw(); a->x30 = rw();
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
        ART[k].xC_itemStates = &ISA[k]; IT[k].xD0_itemStateDesc = 0;
        zero(&IT[k].xDD4_itemVar.linkbomb.x0, sizeof IT[k].xDD4_itemVar.linkbomb.x0); IT[k].xDD4_itemVar.linkbomb.x0.b0 = rnd() & 1; IT[k].xDD4_itemVar.linkbomb.x0.b1 = rnd() & 1; IT[k].xDD4_itemVar.linkbomb.x0.b2 = rnd() & 1;
        IT[k].xDD4_itemVar.linkbomb.x4 = rw(); IT[k].xDD4_itemVar.linkbomb.x8 = rw(); IT[k].xDD4_itemVar.linkbomb.xC = rw();
        IT[k].xDD4_itemVar.linkbomb.x10 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xD44_lifeTimer = ((rnd() % 3) == 0) ? 0.0f : (float) (rnd() % 80) - 5.0f;
        IT[k].xCA0 = (int) (rnd() % 12);
        IT[k].xCCC_incDamageDirection = (rnd() & 1) ? 1.0f : -1.0f;
        IT[k].owner = (rnd() & 1) ? IT[k].xDD4_itemVar.linkbomb.x10 : (((rnd() & 1) ? &GO[(k + 2) % 3] : 0));
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) {
        HM(PI(IT[k].xDD4_itemVar.linkbomb.x10));
        HM(IT[k].xDD4_itemVar.linkbomb.x0.b0); HM(IT[k].xDD4_itemVar.linkbomb.x0.b1); HM(IT[k].xDD4_itemVar.linkbomb.x0.b2); HM(IT[k].xDD4_itemVar.linkbomb.x0.b3); HM(IT[k].xDD4_itemVar.linkbomb.x0.b4); HM(IT[k].xDD4_itemVar.linkbomb.x0.b5); HM(IT[k].xDD4_itemVar.linkbomb.x0.b6); HM(IT[k].xDD4_itemVar.linkbomb.x0.b7);
        IT[k].xDD4_itemVar.linkbomb.x10 = 0; zero(&IT[k].xDD4_itemVar.linkbomb.x0, sizeof IT[k].xDD4_itemVar.linkbomb.x0);
    }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    preg(ISA, sizeof ISA);
    freg_all();
    for (op = 0; op < 38; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_8029D968(&GO[0]); } break; /* it_8029D968 */
            case 1: { enum_t v1 = (enum_t) ((int) (rnd() % 8)); Item_StateChangeFlags v2 = (Item_StateChangeFlags) (rnd() % 32); it_8029D9A4(&GO[0], v1, v2); } break; /* it_8029D9A4 */
            case 2: {  it_8029DB5C(&GO[0]); } break; /* it_8029DB5C */
            case 3: { u32 v2 = (u32) (rnd() % 32); int v3 = (int) ((int) (rnd() % 400)); f32 v4 = (f32) (rw()); ret = PI(it_8029DD58(&GO[0], &VA[0], v2, v3, v4)); } break; /* it_8029DD58 */
            case 4: {  it_8029DEB0(&GO[0]); } break; /* it_8029DEB0 */
            case 5: {  ret = (u32) itLinkbomb_UnkMotion0_Anim(&GO[0]); } break; /* itLinkbomb_UnkMotion0_Anim */
            case 6: {  itLinkbomb_UnkMotion0_Phys(&GO[0]); } break; /* itLinkbomb_UnkMotion0_Phys */
            case 7: {  fn_8029E21C(&GO[0]); } break; /* fn_8029E21C */
            case 8: {  ret = (u32) itLinkbomb_UnkMotion1_Anim(&GO[0]); } break; /* itLinkbomb_UnkMotion1_Anim */
            case 9: {  itLinkbomb_UnkMotion1_Phys(&GO[0]); } break; /* itLinkbomb_UnkMotion1_Phys */
            case 10: {  ret = (u32) itLinkbomb_UnkMotion1_Coll(&GO[0]); } break; /* itLinkbomb_UnkMotion1_Coll */
            case 11: {  it_8029E5D0(&GO[0]); } break; /* it_8029E5D0 */
            case 12: {  ret = (u32) itLinkbomb_UnkMotion2_Anim(&GO[0]); } break; /* itLinkbomb_UnkMotion2_Anim */
            case 13: {  itLinkbomb_UnkMotion2_Phys(&GO[0]); } break; /* itLinkbomb_UnkMotion2_Phys */
            case 14: {  ret = (u32) itLinkbomb_UnkMotion2_Coll(&GO[0]); } break; /* itLinkbomb_UnkMotion2_Coll */
            case 15: {  it_8029EC34(&GO[0]); } break; /* it_8029EC34 */
            case 16: {  ret = (u32) itLinkbomb_UnkMotion3_Anim(&GO[0]); } break; /* itLinkbomb_UnkMotion3_Anim */
            case 17: {  itLinkbomb_UnkMotion3_Phys(&GO[0]); } break; /* itLinkbomb_UnkMotion3_Phys */
            case 18: {  ret = (u32) itLinkbomb_UnkMotion3_Coll(&GO[0]); } break; /* itLinkbomb_UnkMotion3_Coll */
            case 19: {  it_8029F18C(&GO[0]); } break; /* it_8029F18C */
            case 20: {  ret = (u32) itLinkbomb_UnkMotion4_Anim(&GO[0]); } break; /* itLinkbomb_UnkMotion4_Anim */
            case 21: {  itLinkbomb_UnkMotion4_Phys(&GO[0]); } break; /* itLinkbomb_UnkMotion4_Phys */
            case 22: {  ret = (u32) itLinkbomb_UnkMotion4_Coll(&GO[0]); } break; /* itLinkbomb_UnkMotion4_Coll */
            case 23: {  it_8029F69C(&GO[0]); } break; /* it_8029F69C */
            case 24: {  ret = (u32) itLinkbomb_UnkMotion5_Anim(&GO[0]); } break; /* itLinkbomb_UnkMotion5_Anim */
            case 25: {  itLinkbomb_UnkMotion5_Phys(&GO[0]); } break; /* itLinkbomb_UnkMotion5_Phys */
            case 26: {  ret = (u32) itLinkbomb_UnkMotion5_Coll(&GO[0]); } break; /* itLinkbomb_UnkMotion5_Coll */
            case 27: {  ret = (u32) it_8029F960(&GO[0]); } break; /* it_8029F960 */
            case 28: {  ret = (u32) itLinkBomb_Logic16_DmgReceived(&GO[0]); } break; /* itLinkBomb_Logic16_DmgReceived */
            case 29: {  itLinkBomb_Logic16_EnteredAir(&GO[0]); } break; /* itLinkBomb_Logic16_EnteredAir */
            case 30: {  ret = (u32) itLinkbomb_UnkMotion6_Anim(&GO[0]); } break; /* itLinkbomb_UnkMotion6_Anim */
            case 31: {  itLinkbomb_UnkMotion6_Phys(&GO[0]); } break; /* itLinkbomb_UnkMotion6_Phys */
            case 32: {  ret = (u32) itLinkbomb_UnkMotion6_Coll(&GO[0]); } break; /* itLinkbomb_UnkMotion6_Coll */
            case 33: {  ret = (u32) itLinkBomb_Logic16_Reflected(&GO[0]); } break; /* itLinkBomb_Logic16_Reflected */
            case 34: {  ret = (u32) itLinkBomb_Logic16_HitShield(&GO[0]); } break; /* itLinkBomb_Logic16_HitShield */
            case 35: {  ret = (u32) itLinkBomb_Logic16_ShieldBounced(&GO[0]); } break; /* itLinkBomb_Logic16_ShieldBounced */
            case 36: {  it_8029FD84(&GO[0], &GO[1]); } break; /* it_8029FD84 */
            case 37: { { int j; for (j = 0; j < 7; j++) { HM((u32) it_803F6888[j].anim_id); HM(FI((const void*) it_803F6888[j].animated)); HM(FI((const void*) it_803F6888[j].physics_updated)); HM(FI((const void*) it_803F6888[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
