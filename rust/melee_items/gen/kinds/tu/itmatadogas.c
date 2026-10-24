/* TU: src/melee/it/kinds/itmatadogas.c */
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


#include "itmatadogas_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itMatadogasAttributes* a = (itMatadogasAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = (float) (rnd() % 6); a->x8 = rw(); a->xC = rw();
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
        IT[k].xDD4_itemVar.matadogas.x60 = (int) (rnd() % 4);
        IT[k].xDD4_itemVar.matadogas.x64 = (int) (rnd() & 1);
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 23; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_802CAFD4(&GO[0]); } break; /* it_802CAFD4 */
            case 1: {  it_802CB0F4(&GO[0]); } break; /* it_802CB0F4 */
            case 2: {  it_802CB0F8(&GO[0], &GO[1]); } break; /* it_802CB0F8 */
            case 3: {  ret = (u32) it_802CB118(&GO[0]); } break; /* it_802CB118 */
            case 4: {  it_802CB14C(&GO[0]); } break; /* it_802CB14C */
            case 5: {  ret = (u32) it_802CB150(&GO[0]); } break; /* it_802CB150 */
            case 6: {  ret = (u32) itMatadogas_UnkMotion1_Anim(&GO[0]); } break; /* itMatadogas_UnkMotion1_Anim */
            case 7: {  itMatadogas_UnkMotion1_Phys(&GO[0]); } break; /* itMatadogas_UnkMotion1_Phys */
            case 8: {  ret = (u32) itMatadogas_UnkMotion1_Coll(&GO[0]); } break; /* itMatadogas_UnkMotion1_Coll */
            case 9: {  it_802CB2B0(&GO[0]); } break; /* it_802CB2B0 */
            case 10: {  it_802CB350(&GO[0]); } break; /* it_802CB350 */
            case 11: {  ret = (u32) itMatadogas_UnkMotion2_Anim(&GO[0]); } break; /* itMatadogas_UnkMotion2_Anim */
            case 12: {  itMatadogas_UnkMotion2_Phys(&GO[0]); } break; /* itMatadogas_UnkMotion2_Phys */
            case 13: {  ret = (u32) itMatadogas_UnkMotion2_Coll(&GO[0]); } break; /* itMatadogas_UnkMotion2_Coll */
            case 14: { s32 v1 = (s32) ((int) fv()); f32 v2 = (f32) (rw()); it_802CB4F0(&GO[0], v1, v2); } break; /* it_802CB4F0 */
            case 15: {  it_2725_Logic32_Spawned(&GO[0]); } break; /* it_2725_Logic32_Spawned */
            case 16: {  it_2725_Logic33_Spawned(&GO[0]); } break; /* it_2725_Logic33_Spawned */
            case 17: {  itMatadogas_Logic33_EvtUnk(&GO[0], &GO[1]); } break; /* itMatadogas_Logic33_EvtUnk */
            case 18: {  it_802CB798(&GO[0]); } break; /* it_802CB798 */
            case 19: {  ret = (u32) it_802CB810(&GO[0]); } break; /* it_802CB810 */
            case 20: {  it_802CB844(&GO[0]); } break; /* it_802CB844 */
            case 21: {  ret = (u32) it_802CB8A4(&GO[0]); } break; /* it_802CB8A4 */
            case 22: { { int j; for (j = 0; j < 3; j++) { HM((u32) it_803F7B58[j].anim_id); HM(FI((const void*) it_803F7B58[j].animated)); HM(FI((const void*) it_803F7B58[j].physics_updated)); HM(FI((const void*) it_803F7B58[j].collided)); } } { int j; for (j = 0; j < 1; j++) { HM((u32) it_803F7BB0[j].anim_id); HM(FI((const void*) it_803F7BB0[j].animated)); HM(FI((const void*) it_803F7BB0[j].physics_updated)); HM(FI((const void*) it_803F7BB0[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
