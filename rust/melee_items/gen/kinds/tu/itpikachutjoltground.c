/* TU: src/melee/it/kinds/itpikachutjoltground.c */
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

static const int KG[6] = { It_Kind_Pikachu_TJolt_Ground, It_Kind_Pichu_TJolt_Ground, It_Kind_Kirby_PikachuTJolt_Ground, It_Kind_Kirby_PichuTJolt_Ground, 5, 0 };
#include "itpikachutjoltground_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        { int j; for (j = 0; j < 3; j++) {
            float* a = (float*) SPEC[j];
            a[0] = rw(); a[1] = rw(); a[2] = rw(); a[3] = (rnd() & 1) ? 0.0f : rw();
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
        IT[k].kind = KG[rnd() % 6];
        IT[k].xDD4_itemVar.pikachujoltground.xDD4 = rw();
        IT[k].xDD4_itemVar.pikachujoltground.xDD8 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xDD4_itemVar.pikachujoltground.xDDC = (rnd() & 1) ? &GO[(k + 2) % 3] : 0;
        IT[k].xDD4_itemVar.pikachujoltground.xDE0 = (int) (rnd() % 3);
        IT[k].xDD4_itemVar.pikachujoltground.xDE4 = (int) (rnd() % 5) - 2;
        IT[k].xDD4_itemVar.pikachujoltground.xDE8.x = rw(); IT[k].xDD4_itemVar.pikachujoltground.xDE8.y = rw(); IT[k].xDD4_itemVar.pikachujoltground.xDE8.z = rw();
        IT[k].xDD4_itemVar.pikachujoltground.xDF4.x = rw(); IT[k].xDD4_itemVar.pikachujoltground.xDF4.y = rw(); IT[k].xDD4_itemVar.pikachujoltground.xDF4.z = rw();
        IT[k].xDD4_itemVar.pikachujoltground.xE00.x = rw(); IT[k].xDD4_itemVar.pikachujoltground.xE00.y = rw(); IT[k].xDD4_itemVar.pikachujoltground.xE00.z = rw();
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.pikachujoltground.xDD8)); HM(PI(IT[k].xDD4_itemVar.pikachujoltground.xDDC)); IT[k].xDD4_itemVar.pikachujoltground.xDD8 = 0; IT[k].xDD4_itemVar.pikachujoltground.xDDC = 0; }
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
            case 0: {  ret = PI(it_802B3368(&GO[0])); } break; /* it_802B3368 */
            case 1: { f32 v2 = (f32) (rw()); ItemKind v3 = (ItemKind) ((int) fv()); ret = PI(itPikachuThunderJolt_Spawn(&GO[0], &VA[0], v2, v3)); } break; /* itPikachuThunderJolt_Spawn */
            case 2: {  it_2725_Logic106_Destroyed(&GO[0]); } break; /* it_2725_Logic106_Destroyed */
            case 3: {  it_802B3544(&GO[0]); } break; /* it_802B3544 */
            case 4: {  it_802B3554(&GO[0], &GO[1]); } break; /* it_802B3554 */
            case 5: {  ret = (u32) itPikachutjoltground_UnkMotion0_Anim(&GO[0]); } break; /* itPikachutjoltground_UnkMotion0_Anim */
            case 6: {  ret = (u32) itPikachutjoltground_UnkMotion1_Anim(&GO[0]); } break; /* itPikachutjoltground_UnkMotion1_Anim */
            case 7: {  itPikachutjoltground_UnkMotion0_Phys(&GO[0]); } break; /* itPikachutjoltground_UnkMotion0_Phys */
            case 8: {  itPikachutjoltground_UnkMotion1_Phys(&GO[0]); } break; /* itPikachutjoltground_UnkMotion1_Phys */
            case 9: {  ret = (u32) itPikachutjoltground_UnkMotion0_Coll(&GO[0]); } break; /* itPikachutjoltground_UnkMotion0_Coll */
            case 10: {  ret = (u32) itPikachutjoltground_UnkMotion1_Coll(&GO[0]); } break; /* itPikachutjoltground_UnkMotion1_Coll */
            case 11: {  ret = (u32) it_2725_Logic106_DmgDealt(&GO[0]); } break; /* it_2725_Logic106_DmgDealt */
            case 12: {  ret = (u32) it_2725_Logic106_Reflected(&GO[0]); } break; /* it_2725_Logic106_Reflected */
            case 13: {  ret = (u32) it_2725_Logic106_Clanked(&GO[0]); } break; /* it_2725_Logic106_Clanked */
            case 14: {  ret = (u32) it_2725_Logic106_Absorbed(&GO[0]); } break; /* it_2725_Logic106_Absorbed */
            case 15: {  ret = (u32) it_2725_Logic106_HitShield(&GO[0]); } break; /* it_2725_Logic106_HitShield */
            case 16: {  ret = (u32) it_2725_Logic106_ShieldBounced(&GO[0]); } break; /* it_2725_Logic106_ShieldBounced */
            case 17: {  itPikachuTJoltGround_Logic106_EvtUnk(&GO[0], &GO[1]); } break; /* itPikachuTJoltGround_Logic106_EvtUnk */
            case 18: { { int j; for (j = 0; j < 2; j++) { HM((u32) it_803F7190[j].anim_id); HM(FI((const void*) it_803F7190[j].animated)); HM(FI((const void*) it_803F7190[j].physics_updated)); HM(FI((const void*) it_803F7190[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
