/* TU: src/melee/it/kinds/itmarumine.c */
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


#include "itmarumine_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itPokemonAttributes* a = (itPokemonAttributes*) SPEC[0];
        a->x0 = rw(); a->timer = (int) (rnd() % 3); a->max = (int) (rnd() % 200); a->xC = (int) (rnd() % 4);
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
        IT[k].xDD4_itemVar.pokemon.timer = (int) (rnd() % 5) - 2;
        IT[k].xDD4_itemVar.pokemon.x64 = (int) (rnd() % 3);
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 38; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_802D09D0(&GO[0]); } break; /* it_802D09D0 */
            case 1: {  itMarumine_Logic16_Spawned(&GO[0]); } break; /* itMarumine_Logic16_Spawned */
            case 2: {  it_802D0AAC(&GO[0]); } break; /* it_802D0AAC */
            case 3: {  it_802D0AB0(&GO[0], &GO[1]); } break; /* it_802D0AB0 */
            case 4: {  ret = (u32) it_802D0AD0(&GO[0]); } break; /* it_802D0AD0 */
            case 5: {  it_802D0B04(&GO[0]); } break; /* it_802D0B04 */
            case 6: {  ret = (u32) itMarumine_UnkMotion1_Anim(&GO[0]); } break; /* itMarumine_UnkMotion1_Anim */
            case 7: {  itMarumine_UnkMotion1_Phys(&GO[0]); } break; /* itMarumine_UnkMotion1_Phys */
            case 8: {  ret = (u32) itMarumine_UnkMotion1_Coll(&GO[0]); } break; /* itMarumine_UnkMotion1_Coll */
            case 9: {  it_802D0C44(&GO[0]); } break; /* it_802D0C44 */
            case 10: {  ret = (u32) itMarumine_UnkMotion2_Anim(&GO[0]); } break; /* itMarumine_UnkMotion2_Anim */
            case 11: {  itMarumine_UnkMotion2_Phys(&GO[0]); } break; /* itMarumine_UnkMotion2_Phys */
            case 12: {  ret = (u32) itMarumine_UnkMotion2_Coll(&GO[0]); } break; /* itMarumine_UnkMotion2_Coll */
            case 13: {  it_802D0D18(&GO[0]); } break; /* it_802D0D18 */
            case 14: {  ret = (u32) itMarumine_UnkMotion3_Anim(&GO[0]); } break; /* itMarumine_UnkMotion3_Anim */
            case 15: {  itMarumine_UnkMotion3_Phys(&GO[0]); } break; /* itMarumine_UnkMotion3_Phys */
            case 16: {  ret = (u32) itMarumine_UnkMotion3_Coll(&GO[0]); } break; /* itMarumine_UnkMotion3_Coll */
            case 17: {  it_802D0DBC(&GO[0]); } break; /* it_802D0DBC */
            case 18: {  it_802D0E30(&GO[0]); } break; /* it_802D0E30 */
            case 19: {  it_802D0E90(&GO[0]); } break; /* it_802D0E90 */
            case 20: {  ret = (u32) itMarumine_UnkMotion4_Anim(&GO[0]); } break; /* itMarumine_UnkMotion4_Anim */
            case 21: {  itMarumine_UnkMotion4_Phys(&GO[0]); } break; /* itMarumine_UnkMotion4_Phys */
            case 22: {  ret = (u32) itMarumine_UnkMotion4_Coll(&GO[0]); } break; /* itMarumine_UnkMotion4_Coll */
            case 23: {  fn_802D0F98(&GO[0]); } break; /* fn_802D0F98 */
            case 24: {  it_802D100C(&GO[0]); } break; /* it_802D100C */
            case 25: {  ret = (u32) itMarumine_UnkMotion5_Anim(&GO[0]); } break; /* itMarumine_UnkMotion5_Anim */
            case 26: {  itMarumine_UnkMotion5_Phys(&GO[0]); } break; /* itMarumine_UnkMotion5_Phys */
            case 27: {  ret = (u32) itMarumine_UnkMotion5_Coll(&GO[0]); } break; /* itMarumine_UnkMotion5_Coll */
            case 28: {  it_802D1140(&GO[0]); } break; /* it_802D1140 */
            case 29: {  it_802D1204(&GO[0]); } break; /* it_802D1204 */
            case 30: {  ret = (u32) itMarumine_UnkMotion6_Anim(&GO[0]); } break; /* itMarumine_UnkMotion6_Anim */
            case 31: {  itMarumine_UnkMotion6_Phys(&GO[0]); } break; /* itMarumine_UnkMotion6_Phys */
            case 32: {  ret = (u32) itMarumine_UnkMotion6_Coll(&GO[0]); } break; /* itMarumine_UnkMotion6_Coll */
            case 33: {  it_802D1320(&GO[0]); } break; /* it_802D1320 */
            case 34: {  ret = (u32) itMarumine_UnkMotion0_Anim(&GO[0]); } break; /* itMarumine_UnkMotion0_Anim */
            case 35: {  itMarumine_UnkMotion0_Phys(&GO[0]); } break; /* itMarumine_UnkMotion0_Phys */
            case 36: {  ret = (u32) itMarumine_UnkMotion0_Coll(&GO[0]); } break; /* itMarumine_UnkMotion0_Coll */
            case 37: { { int j; for (j = 0; j < 7; j++) { HM((u32) it_803F7E78[j].anim_id); HM(FI((const void*) it_803F7E78[j].animated)); HM(FI((const void*) it_803F7E78[j].physics_updated)); HM(FI((const void*) it_803F7E78[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
