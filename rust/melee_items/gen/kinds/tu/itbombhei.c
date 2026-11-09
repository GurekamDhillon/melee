/* TU: src/melee/it/kinds/itbombhei.c */
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


#include "itbombhei_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itBombHeiAttributes* a = (itBombHeiAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = (float) (rnd() % 12); a->x8 = (float) (rnd() % 12); a->xC = rw();
        a->x10 = (float) (rnd() % 12); a->x14 = (float) (rnd() % 12); a->x18 = (float) (rnd() % 12); a->x1C = rw();
        a->x20.x = (float) (rnd() % 3) / 4.0f; a->x20.y = (float) (rnd() % 3) / 4.0f; a->x20.z = rw();
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
        IT[k].xDD4_itemVar.bombhei.xDD4 = (int) (rnd() % 12) - 3;
        IT[k].xDD4_itemVar.bombhei.xDD8 = (rnd() & 1) ? 1 : -1;
        IT[k].xDD4_itemVar.bombhei.xDDC = (int) (rnd() & 1);
        IT[k].xDD4_itemVar.bombhei.xDE0 = (int) (rnd() & 1);
        IT[k].xDD4_itemVar.bombhei.xDE4 = (int) (rnd() & 1);
        IT[k].xDD4_itemVar.bombhei.xDE8 = (rnd() & 1) ? 1.0f : rw();
        i = (int) (rnd() % 10) - 3;
        IT[k].xDD4_itemVar.bombhei.xDEC = (float) i;
        i = (int) (rnd() % 6) - 2;
        IT[k].xDD4_itemVar.bombhei.xDF0 = (float) i;
        IT[k].xDD4_itemVar.bombhei.xDF4 = (rnd() & 1) ? 1.0f : -1.0f;
        IT[k].xDD4_itemVar.bombhei.xDF8 = rw();
        IT[k].xDD4_itemVar.bombhei.xDFC = rw();
        IT[k].xDD4_itemVar.bombhei.xE04 = rw();
        IT[k].xDD4_itemVar.bombhei.xE0C.x = rw(); IT[k].xDD4_itemVar.bombhei.xE0C.y = rw(); IT[k].xDD4_itemVar.bombhei.xE0C.z = rw();
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 59; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  ret = PI(it_8027D670(&VA[0])); } break; /* it_8027D670 */
            case 1: {  it_8027D730(&GO[0]); } break; /* it_8027D730 */
            case 2: {  itBombhei_Logic6_Spawned(&GO[0]); } break; /* itBombhei_Logic6_Spawned */
            case 3: {  it_8027D820(&GO[0]); } break; /* it_8027D820 */
            case 4: {  fn_8027DAC8(&GO[0]); } break; /* fn_8027DAC8 */
            case 5: {  ret = (u32) ((itBombhei_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itBombhei_UnkMotion0_Anim */
            case 6: {  itBombhei_UnkMotion0_Phys(&GO[0]); } break; /* itBombhei_UnkMotion0_Phys */
            case 7: {  ret = (u32) ((itBombhei_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itBombhei_UnkMotion0_Coll */
            case 8: {  it_8027DE18(&GO[0]); } break; /* it_8027DE18 */
            case 9: {  ret = (u32) ((itBombhei_UnkMotion1_Anim(&GO[0])) != 0); } break; /* itBombhei_UnkMotion1_Anim */
            case 10: {  itBombhei_UnkMotion1_Phys(&GO[0]); } break; /* itBombhei_UnkMotion1_Phys */
            case 11: {  ret = (u32) ((itBombhei_UnkMotion1_Coll(&GO[0])) != 0); } break; /* itBombhei_UnkMotion1_Coll */
            case 12: {  itBombhei_Logic6_PickedUp(&GO[0]); } break; /* itBombhei_Logic6_PickedUp */
            case 13: {  ret = (u32) ((itBombhei_UnkMotion8_Anim(&GO[0])) != 0); } break; /* itBombhei_UnkMotion8_Anim */
            case 14: {  itBombhei_UnkMotion8_Phys(&GO[0]); } break; /* itBombhei_UnkMotion8_Phys */
            case 15: {  it_3F14_Logic6_Dropped(&GO[0]); } break; /* it_3F14_Logic6_Dropped */
            case 16: {  it_8027E978(&GO[0]); } break; /* it_8027E978 */
            case 17: {  ret = (u32) ((itBombhei_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itBombhei_UnkMotion3_Anim */
            case 18: {  itBombhei_UnkMotion3_Phys(&GO[0]); } break; /* itBombhei_UnkMotion3_Phys */
            case 19: {  ret = (u32) ((itBombhei_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itBombhei_UnkMotion3_Coll */
            case 20: {  it_8027EE04(&GO[0]); } break; /* it_8027EE04 */
            case 21: {  ret = (u32) ((itBombhei_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itBombhei_UnkMotion2_Anim */
            case 22: {  itBombhei_UnkMotion2_Phys(&GO[0]); } break; /* itBombhei_UnkMotion2_Phys */
            case 23: {  ret = (u32) ((itBombhei_UnkMotion2_Coll(&GO[0])) != 0); } break; /* itBombhei_UnkMotion2_Coll */
            case 24: {  it_8027F42C(&GO[0]); } break; /* it_8027F42C */
            case 25: {  ret = (u32) ((itBombhei_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itBombhei_UnkMotion4_Anim */
            case 26: {  itBombhei_UnkMotion4_Phys(&GO[0]); } break; /* itBombhei_UnkMotion4_Phys */
            case 27: {  ret = (u32) ((itBombhei_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itBombhei_UnkMotion4_Coll */
            case 28: {  it_8027F8E0(&GO[0]); } break; /* it_8027F8E0 */
            case 29: {  ret = (u32) ((itBombhei_UnkMotion5_Anim(&GO[0])) != 0); } break; /* itBombhei_UnkMotion5_Anim */
            case 30: {  itBombhei_UnkMotion5_Phys(&GO[0]); } break; /* itBombhei_UnkMotion5_Phys */
            case 31: {  ret = (u32) ((itBombhei_UnkMotion5_Coll(&GO[0])) != 0); } break; /* itBombhei_UnkMotion5_Coll */
            case 32: {  fn_8027FCA8(&GO[0]); } break; /* fn_8027FCA8 */
            case 33: {  ret = (u32) ((itBombhei_UnkMotion6_Anim(&GO[0])) != 0); } break; /* itBombhei_UnkMotion6_Anim */
            case 34: {  itBombhei_UnkMotion6_Phys(&GO[0]); } break; /* itBombhei_UnkMotion6_Phys */
            case 35: {  fn_8028007C(&GO[0]); } break; /* fn_8028007C */
            case 36: {  ret = (u32) ((itBombhei_UnkMotion6_Coll(&GO[0])) != 0); } break; /* itBombhei_UnkMotion6_Coll */
            case 37: {  it_3F14_Logic6_Thrown(&GO[0]); } break; /* it_3F14_Logic6_Thrown */
            case 38: {  ret = (u32) ((itBombhei_UnkMotion10_Anim(&GO[0])) != 0); } break; /* itBombhei_UnkMotion10_Anim */
            case 39: {  itBombhei_UnkMotion10_Phys(&GO[0]); } break; /* itBombhei_UnkMotion10_Phys */
            case 40: {  fn_80280974(&GO[0]); } break; /* fn_80280974 */
            case 41: {  ret = (u32) ((itBombhei_UnkMotion10_Coll(&GO[0])) != 0); } break; /* itBombhei_UnkMotion10_Coll */
            case 42: {  it_80280B60(&GO[0]); } break; /* it_80280B60 */
            case 43: {  it_80280DC0(&GO[0]); } break; /* it_80280DC0 */
            case 44: {  ret = (u32) ((it_3F14_Logic6_DmgDealt(&GO[0])) != 0); } break; /* it_3F14_Logic6_DmgDealt */
            case 45: {  ret = (u32) ((it_3F14_Logic6_DmgReceived(&GO[0])) != 0); } break; /* it_3F14_Logic6_DmgReceived */
            case 46: {  ret = (u32) ((itBombhei_UnkMotion11_Anim(&GO[0])) != 0); } break; /* itBombhei_UnkMotion11_Anim */
            case 47: {  itBombhei_UnkMotion11_Phys(&GO[0]); } break; /* itBombhei_UnkMotion11_Phys */
            case 48: {  ret = (u32) ((itBombhei_UnkMotion11_Coll(&GO[0])) != 0); } break; /* itBombhei_UnkMotion11_Coll */
            case 49: {  it_3F14_Logic6_EnteredAir(&GO[0]); } break; /* it_3F14_Logic6_EnteredAir */
            case 50: {  ret = (u32) ((itBombhei_UnkMotion12_Anim(&GO[0])) != 0); } break; /* itBombhei_UnkMotion12_Anim */
            case 51: {  itBombhei_UnkMotion12_Phys(&GO[0]); } break; /* itBombhei_UnkMotion12_Phys */
            case 52: {  ret = (u32) ((itBombhei_UnkMotion12_Coll(&GO[0])) != 0); } break; /* itBombhei_UnkMotion12_Coll */
            case 53: {  ret = (u32) ((itBombhei_Logic6_Clanked(&GO[0])) != 0); } break; /* itBombhei_Logic6_Clanked */
            case 54: {  ret = (u32) ((itBombhei_Logic6_Reflected(&GO[0])) != 0); } break; /* itBombhei_Logic6_Reflected */
            case 55: {  ret = (u32) ((it_3F14_Logic6_HitShield(&GO[0])) != 0); } break; /* it_3F14_Logic6_HitShield */
            case 56: {  ret = (u32) ((it_3F14_Logic6_ShieldBounced(&GO[0])) != 0); } break; /* it_3F14_Logic6_ShieldBounced */
            case 57: {  itBombhei_Logic6_EvtUnk(&GO[0], &GO[1]); } break; /* itBombhei_Logic6_EvtUnk */
            case 58: { { int j; for (j = 0; j < 13; j++) { HM((u32) it_803F54D8[j].anim_id); HM(FI((const void*) it_803F54D8[j].animated)); HM(FI((const void*) it_803F54D8[j].physics_updated)); HM(FI((const void*) it_803F54D8[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
