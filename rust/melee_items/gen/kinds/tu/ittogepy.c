/* TU: src/melee/it/kinds/ittogepy.c */
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


#include "ittogepy_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itPokemonAttributes* a = (itPokemonAttributes*) SPEC[0];
        a->x0 = rw(); a->timer = (int) (rnd() % 6) - 1; a->max = (int) (rnd() % 24) - 2; a->xC = (int) (rnd() % 8);
        a->x10 = (int) (rnd() % 8); a->x14 = (int) (rnd() % 8); a->x18 = (int) (rnd() % 8);
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
        IT[k].xDAC_itcmd_var0 = rnd() & 1;
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
            case 0: {  itTogepy_Logic21_Spawned(&GO[0]); } break; /* itTogepy_Logic21_Spawned */
            case 1: {  it_802D3728(&GO[0]); } break; /* it_802D3728 */
            case 2: {  itTogepy_Logic21_EvtUnk(&GO[0], &GO[1]); } break; /* itTogepy_Logic21_EvtUnk */
            case 3: {  ret = (u32) itTogepy_UnkMotion1_Anim(&GO[0]); } break; /* itTogepy_UnkMotion1_Anim */
            case 4: {  itTogepy_UnkMotion1_Phys(&GO[0]); } break; /* itTogepy_UnkMotion1_Phys */
            case 5: {  ret = (u32) itTogepy_UnkMotion1_Coll(&GO[0]); } break; /* itTogepy_UnkMotion1_Coll */
            case 6: {  it_802D3848(&GO[0]); } break; /* it_802D3848 */
            case 7: {  ret = (u32) itTogepy_UnkMotion6_Anim(&GO[0]); } break; /* itTogepy_UnkMotion6_Anim */
            case 8: {  itTogepy_UnkMotion6_Phys(&GO[0]); } break; /* itTogepy_UnkMotion6_Phys */
            case 9: {  ret = (u32) itTogepy_UnkMotion6_Coll(&GO[0]); } break; /* itTogepy_UnkMotion6_Coll */
            case 10: {  it_802D39F8(&GO[0]); } break; /* it_802D39F8 */
            case 11: {  ret = (u32) itTogepy_UnkMotion0_Anim(&GO[0]); } break; /* itTogepy_UnkMotion0_Anim */
            case 12: {  itTogepy_UnkMotion0_Phys(&GO[0]); } break; /* itTogepy_UnkMotion0_Phys */
            case 13: {  ret = (u32) itTogepy_UnkMotion0_Coll(&GO[0]); } break; /* itTogepy_UnkMotion0_Coll */
            case 14: { { int j; for (j = 0; j < 7; j++) { HM((u32) it_803F8058[j].anim_id); HM(FI((const void*) it_803F8058[j].animated)); HM(FI((const void*) it_803F8058[j].physics_updated)); HM(FI((const void*) it_803F8058[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
