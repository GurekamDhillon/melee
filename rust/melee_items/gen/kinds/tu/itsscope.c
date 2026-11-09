/* TU: src/melee/it/kinds/itsscope.c */
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


#include "itsscope_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itSScopeAttributes* a = (itSScopeAttributes*) SPEC[0];
        *(int*) SPEC[0] = (int) (rnd() % 40);
        a->x4 = (int) (rnd() % 20); a->x8 = (int) (rnd() % 6);
        for (i = 0; i < 10; i++) a->xC[i] = (float) (rnd() % 12);
        a->x34.x = rw(); a->x34.y = rw(); a->x34.z = rw();
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
    it_804D6D28 = &ICD;
    IT[0].owner = (rnd() & 3) ? &GO[1] : 0;
    IT[0].xC64_reflectGObj = (rnd() & 3) ? &GO[1] : 0;
    if (rnd() & 1) IT[0].msid = (int) (rnd() % 14);
    if (rnd() & 1) IT[0].facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
    IT[0].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
    spec_fill(prof);
    for (k = 0; k < 3; k++) {
        IT[k].xD4C = (int) (rnd() % 40) - 3;
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 33; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  ret = PI(it_80291BE0(&VA[0])); } break; /* it_80291BE0 */
            case 1: {  itSScope_Logic21_Spawned(&GO[0]); } break; /* itSScope_Logic21_Spawned */
            case 2: {  it_80291CCC(&GO[0], &VA[0]); } break; /* it_80291CCC */
            case 3: { s32 v1 = (s32) ((int) (rnd() % 120) - 5); ret = (u32) it_80291CF4(&GO[0], v1); } break; /* it_80291CF4 */
            case 4: { s32 v1 = (s32) ((int) (rnd() % 10)); ret = (u32) it_80291D38(&GO[0], v1); } break; /* it_80291D38 */
            case 5: { int v1 = (int) ((int) (rnd() % 120) - 5); ret = (u32) it_80291DAC(&GO[0], v1); } break; /* it_80291DAC */
            case 6: { int v1 = (int) ((int) (rnd() % 10)); it_80291F14(&GO[0], v1); } break; /* it_80291F14 */
            case 7: { int v2 = (int) ((int) (rnd() % 10)); float v3 = (float) (rw()); it_80291FA8(&GO[0], &VA[0], v2, v3); } break; /* it_80291FA8 */
            case 8: {  it_80292030(&GO[0]); } break; /* it_80292030 */
            case 9: {  ret = (u32) ((itSscope_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itSscope_UnkMotion0_Anim */
            case 10: {  itSscope_UnkMotion0_Phys(&GO[0]); } break; /* itSscope_UnkMotion0_Phys */
            case 11: {  ret = (u32) ((itSscope_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itSscope_UnkMotion0_Coll */
            case 12: {  it_802920B8(&GO[0]); } break; /* it_802920B8 */
            case 13: {  ret = (u32) ((itSscope_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itSscope_UnkMotion3_Anim */
            case 14: {  itSscope_UnkMotion1_Phys(&GO[0]); } break; /* itSscope_UnkMotion1_Phys */
            case 15: {  ret = (u32) ((itSscope_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itSscope_UnkMotion3_Coll */
            case 16: {  itSScope_Logic21_PickedUp(&GO[0]); } break; /* itSScope_Logic21_PickedUp */
            case 17: {  ret = (u32) ((itSscope_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itSscope_UnkMotion2_Anim */
            case 18: {  itSscope_UnkMotion2_Phys(&GO[0]); } break; /* itSscope_UnkMotion2_Phys */
            case 19: {  itSScope_Logic21_Dropped(&GO[0]); } break; /* itSScope_Logic21_Dropped */
            case 20: {  itSScope_Logic21_Thrown(&GO[0]); } break; /* itSScope_Logic21_Thrown */
            case 21: {  itSscope_UnkMotion3_Phys(&GO[0]); } break; /* itSscope_UnkMotion3_Phys */
            case 22: {  ret = (u32) ((itSScope_Logic21_DmgDealt(&GO[0])) != 0); } break; /* itSScope_Logic21_DmgDealt */
            case 23: {  ret = (u32) ((itSScope_Logic21_Clanked(&GO[0])) != 0); } break; /* itSScope_Logic21_Clanked */
            case 24: {  ret = (u32) ((itSScope_Logic21_HitShield(&GO[0])) != 0); } break; /* itSScope_Logic21_HitShield */
            case 25: {  ret = (u32) ((itSScope_Logic21_Reflected(&GO[0])) != 0); } break; /* itSScope_Logic21_Reflected */
            case 26: {  ret = (u32) ((itSScope_Logic21_ShieldBounced(&GO[0])) != 0); } break; /* itSScope_Logic21_ShieldBounced */
            case 27: {  itSScope_Logic21_EnteredAir(&GO[0]); } break; /* itSScope_Logic21_EnteredAir */
            case 28: {  ret = (u32) ((itSscope_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itSscope_UnkMotion4_Anim */
            case 29: {  itSscope_UnkMotion4_Phys(&GO[0]); } break; /* itSscope_UnkMotion4_Phys */
            case 30: {  ret = (u32) ((itSscope_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itSscope_UnkMotion4_Coll */
            case 31: {  itSScope_Logic21_EvtUnk(&GO[0], &GO[1]); } break; /* itSScope_Logic21_EvtUnk */
            case 32: { { int j; for (j = 0; j < 5; j++) { HM((u32) it_803F5F40[j].anim_id); HM(FI((const void*) it_803F5F40[j].animated)); HM(FI((const void*) it_803F5F40[j].physics_updated)); HM(FI((const void*) it_803F5F40[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
