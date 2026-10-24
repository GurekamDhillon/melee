/* TU: src/melee/it/kinds/ittools.c */
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


#include "ittools_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itToolsAttributes* a = (itToolsAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = (float) (rnd() % 60); a->x8 = rw(); a->xC = (int) (rnd() % 100);
        for (i = 0; i < 4; i++) {
            float* m = (float*) &a->motions[0] + 7 * i;
            m[0] = rw(); m[1] = rw(); m[2] = rw(); m[3] = rw(); m[4] = rw(); m[5] = rw(); m[6] = rw();
        }
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
        IT[k].xDD4_itemVar.tools.x0 = (int) (rnd() % 4);
        IT[k].xD44_lifeTimer = (rnd() & 1) ? (float) (int) (rnd() % 8) : rg();
        IT[k].xD48_halfLifeTimer = rg();
        IT[k].xCA0 = (int) (rnd() % 50);
        IT[k].xCCC_incDamageDirection = rg();
        IT[k].x5D4_hitboxes[0].hit.damage = (float) (int) (rnd() % 30);
        IT[k].xCEC_fighterGObj = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xCEC_fighterGObj)); IT[k].xCEC_fighterGObj = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 19; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: { s32 v1 = (s32) ((int) (rnd() % 4)); f32 v2 = (f32) (rw()); ret = PI(it_802EEFA8(&VA[0], v1, v2)); } break; /* it_802EEFA8 */
            case 1: {  ret = (u32) itTools_Logic22_DmgDealt(&GO[0]); } break; /* itTools_Logic22_DmgDealt */
            case 2: { s32 v1 = (s32) ((int) (rnd() % 4)); it_802EF098(&GO[0], v1); } break; /* it_802EF098 */
            case 3: {  ret = (u32) itTools_UnkMotion4_Anim(&GO[0]); } break; /* itTools_UnkMotion4_Anim */
            case 4: {  itTools_UnkMotion4_Phys(&GO[0]); } break; /* itTools_UnkMotion4_Phys */
            case 5: {  ret = (u32) itTools_UnkMotion4_Coll(&GO[0]); } break; /* itTools_UnkMotion4_Coll */
            case 6: {  it_802EF320(&GO[0]); } break; /* it_802EF320 */
            case 7: {  ret = (u32) itTools_UnkMotion9_Anim(&GO[0]); } break; /* itTools_UnkMotion9_Anim */
            case 8: {  itTools_UnkMotion9_Phys(&GO[0]); } break; /* itTools_UnkMotion9_Phys */
            case 9: {  ret = (u32) itTools_UnkMotion9_Coll(&GO[0]); } break; /* itTools_UnkMotion9_Coll */
            case 10: {  it_802EF548(&GO[0]); } break; /* it_802EF548 */
            case 11: {  ret = (u32) itTools_Logic22_DmgReceived(&GO[0]); } break; /* itTools_Logic22_DmgReceived */
            case 12: {  ret = (u32) it_2725_Logic22_Clanked(&GO[0]); } break; /* it_2725_Logic22_Clanked */
            case 13: {  ret = (u32) it_2725_Logic22_HitShield(&GO[0]); } break; /* it_2725_Logic22_HitShield */
            case 14: {  ret = (u32) it_2725_Logic22_Absorbed(&GO[0]); } break; /* it_2725_Logic22_Absorbed */
            case 15: {  ret = (u32) itTools_Logic22_ShieldBounced(&GO[0]); } break; /* itTools_Logic22_ShieldBounced */
            case 16: {  ret = (u32) itTools_Logic22_Reflected(&GO[0]); } break; /* itTools_Logic22_Reflected */
            case 17: {  it_802EFA24(&GO[0], &GO[1]); } break; /* it_802EFA24 */
            case 18: { { int j; for (j = 0; j < 10; j++) { HM((u32) it_803F9260[j].anim_id); HM(FI((const void*) it_803F9260[j].animated)); HM(FI((const void*) it_803F9260[j].physics_updated)); HM(FI((const void*) it_803F9260[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
