/* TU: src/melee/it/kinds/itarwinglaser.c */
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


#include "itarwinglaser_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        ArwingLaserAttr* a = (ArwingLaserAttr*) SPEC[0];
        a->x0 = (itSpecialAttrsHead*) &ATTR[0]; a->x4 = rw(); a->x8 = rw();
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
        IT[k].xDD4_itemVar.arwinglaser.xE38 = (int) (rnd() % 6);
        IT[k].xDD4_itemVar.arwinglaser.xE30 = (s16) (rnd() % 3);
        IT[k].xDD4_itemVar.arwinglaser.xDF4 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        i = (int) (rnd() % 3);
        IT[k].xDD4_itemVar.arwinglaser.xDF8 = (rnd() & 1) ? &JO[3 + i] : 0;
        IT[k].xDD4_itemVar.arwinglaser.xE40 = (rnd() & 1) ? &GO[(k + 2) % 3] : 0;
        IT[k].xDD4_itemVar.arwinglaser.xE44 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xDD4_itemVar.arwinglaser.xE48 = (int) (rnd() % 5);
        IT[k].xDD4_itemVar.arwinglaser.xE4C = (int) (rnd() % 3);
        i = (int) (rnd() % 80) - 40;
        IT[k].xDD4_itemVar.arwinglaser.xE24.z = (rnd() & 1) ? (float) i : rw();
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) {
        HM(PI(IT[k].xDD4_itemVar.arwinglaser.xDF4)); HM(PI(IT[k].xDD4_itemVar.arwinglaser.xDF8)); HM(PI(IT[k].xDD4_itemVar.arwinglaser.xE40)); HM(PI(IT[k].xDD4_itemVar.arwinglaser.xE44));
        IT[k].xDD4_itemVar.arwinglaser.xDF4 = 0; IT[k].xDD4_itemVar.arwinglaser.xDF8 = 0; IT[k].xDD4_itemVar.arwinglaser.xE40 = 0; IT[k].xDD4_itemVar.arwinglaser.xE44 = 0;
    }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 18; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  ret = (u32) it_802E70BC(&GO[0]); } break; /* it_802E70BC */
            case 1: { s32 v2 = (s32) ((int) (rnd() % 6)); f32 v3 = (f32) (rw()); f32 v4 = (f32) (rw()); ret = PI(it_802E72E0(&GO[0], &JO[3], v2, v3, v4)); } break; /* it_802E72E0 */
            case 2: { s32 v3 = (s32) ((int) (rnd() % 6)); s32 v4 = (s32) ((int) (rnd() % 5)); f32 v5 = (f32) (rw()); it_802E7654(&GO[0], &JO[3], &VA[0], v3, v4, v5); } break; /* it_802E7654 */
            case 3: {  it_802E79C8(&GO[0]); } break; /* it_802E79C8 */
            case 4: {  it_802E7A4C(&GO[0]); } break; /* it_802E7A4C */
            case 5: {  itArwinglaser_UnkMotion2_Anim(&GO[0]); } break; /* itArwinglaser_UnkMotion2_Anim */
            case 6: {  itArwinglaser_UnkMotion3_Anim(&GO[0]); } break; /* itArwinglaser_UnkMotion3_Anim */
            case 7: {  itArwinglaser_UnkMotion2_Phys(&GO[0]); } break; /* itArwinglaser_UnkMotion2_Phys */
            case 8: {  itArwinglaser_UnkMotion3_Phys(&GO[0]); } break; /* itArwinglaser_UnkMotion3_Phys */
            case 9: {  ret = (u32) ((itArwinglaser_UnkMotion2_Coll(&GO[0])) != 0); } break; /* itArwinglaser_UnkMotion2_Coll */
            case 10: {  ret = (u32) ((itArwinglaser_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itArwinglaser_UnkMotion3_Coll */
            case 11: {  ret = (u32) ((itArwinglaser_UnkMotion5_Coll(&GO[0])) != 0); } break; /* itArwinglaser_UnkMotion5_Coll */
            case 12: {  ret = (u32) ((it_802E838C(&GO[0])) != 0); } break; /* it_802E838C */
            case 13: {  ret = (u32) ((it_802E8418(&GO[0])) != 0); } break; /* it_802E8418 */
            case 14: {  ret = (u32) ((it_802E8420(&GO[0])) != 0); } break; /* it_802E8420 */
            case 15: {  ret = (u32) ((it_802E85F4(&GO[0])) != 0); } break; /* it_802E85F4 */
            case 16: {  ret = (u32) ((it_802E8784(&GO[0])) != 0); } break; /* it_802E8784 */
            case 17: { { int j; for (j = 0; j < 6; j++) { HM((u32) it_803F8DE8[j].anim_id); HM(FI((const void*) it_803F8DE8[j].animated)); HM(FI((const void*) it_803F8DE8[j].physics_updated)); HM(FI((const void*) it_803F8DE8[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
