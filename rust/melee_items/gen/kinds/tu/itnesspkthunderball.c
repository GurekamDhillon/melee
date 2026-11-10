/* TU: src/melee/it/kinds/itnesspkthunderball.c */
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


#include "itnesspkthunderball_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        { int j; for (j = 0; j < 3; j++) { itPKThunderAttributes* a = (itPKThunderAttributes*) SPEC[j];
            a->x0_PKTHUNDER_LIFETIME = rw(); a->x4_PKTHUNDER_SPEED = rw(); a->x8_PKTHUNDER_SPAWN_ANGLE = rw(); a->xC_PKTHUNDER_STICK_THRESHOLD = rg(); a->x10_PKTHUNDER_TURN_RADIUS = (float) (rnd() % 100); } }
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
        i = (int) (rnd() % 4);
        IT[k].xDD4_itemVar.pkthunder.xF04 = (i == 0) ? 0 : (i == 1) ? IT[k].owner : &GO[(k + i) % 3];
        { int j;
        for (j = 0; j < 6; j++) { IT[k].xDD4_itemVar.pkthunder.xDD4[j] = (rnd() & 1) ? &GO[(k + 1 + j) % 3] : 0; }
        IT[k].xDD4_itemVar.pkthunder.xEF4 = 0;
        IT[k].xDD4_itemVar.pkthunder.xF08 = 0;
        IT[k].xDD4_itemVar.pkthunder.xEF8 = (int) (rnd() % 14);
        IT[k].xDD4_itemVar.pkthunder.xEFC = (int) (rnd() % 8);
        IT[k].xDD4_itemVar.pkthunder.xF00 = (rnd() % 4) == 0;
        IT[k].xDD4_itemVar.pkthunder.xEEC = rw();
        for (j = 0; j < 16; j++) { IT[k].xDD4_itemVar.pkthunder.angles[j] = rg() * 4.0f; } }
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) {
        int j;
        for (j = 0; j < 6; j++) { HM(PI(IT[k].xDD4_itemVar.pkthunder.xDD4[j])); IT[k].xDD4_itemVar.pkthunder.xDD4[j] = 0; }
        HM(PI(IT[k].xDD4_itemVar.pkthunder.xEF4)); IT[k].xDD4_itemVar.pkthunder.xEF4 = 0; HM(PI(IT[k].xDD4_itemVar.pkthunder.xF04)); IT[k].xDD4_itemVar.pkthunder.xF04 = 0; HM(PI(IT[k].xDD4_itemVar.pkthunder.xF08)); IT[k].xDD4_itemVar.pkthunder.xF08 = 0;
    }
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
            case 0: { s32 v2 = (s32) ((int) (rnd() % 8)); it_802AB3F0(&GO[0], &VA[0], v2); } break; /* it_802AB3F0 */
            case 1: { it_802AB468(&GO[0], &VA[0].x, (int) fv()); } break; /* it_802AB468 */
            case 2: { ret = (u32) it_802AB4B8(&GO[0], &IT[0].x378_itemColl); } break; /* it_802AB4B8 */
            case 3: {  ret = PI(it_802AB568(&GO[0])); } break; /* it_802AB568 */
            case 4: { f32 v2 = (f32) (rw()); ret = PI(it_802AB58C(&GO[0], &VA[0], v2)); } break; /* it_802AB58C */
            case 5: {  it_802AB90C(&GO[0]); } break; /* it_802AB90C */
            case 6: {  it_802AB9C0(&GO[0]); } break; /* it_802AB9C0 */
            case 7: {  it_802ABA4C(&GO[0]); } break; /* it_802ABA4C */
            case 8: {  ret = (u32) itNesspkthunderball_UnkMotion0_Anim(&GO[0]); } break; /* itNesspkthunderball_UnkMotion0_Anim */
            case 9: {  itNesspkthunderball_UnkMotion0_Phys(&GO[0]); } break; /* itNesspkthunderball_UnkMotion0_Phys */
            case 10: {  ret = (u32) itNesspkthunderball_UnkMotion0_Coll(&GO[0]); } break; /* itNesspkthunderball_UnkMotion0_Coll */
            case 11: {  ret = (u32) itNessPKThunderball_Logic26_DmgDealt(&GO[0]); } break; /* itNessPKThunderball_Logic26_DmgDealt */
            case 12: {  ret = (u32) it_802AC074(&GO[0]); } break; /* it_802AC074 */
            case 13: {  ret = (u32) it_802AC098(&GO[0]); } break; /* it_802AC098 */
            case 14: {  ret = (u32) it_802AC338(&GO[0]); } break; /* it_802AC338 */
            case 15: {  ret = (u32) it_802AC35C(&GO[0]); } break; /* it_802AC35C */
            case 16: {  ret = (u32) it_802AC3F8(&GO[0]); } break; /* it_802AC3F8 */
            case 17: {  it_802AC41C(&GO[0], &GO[1]); } break; /* it_802AC41C */
            case 18: { it_802AB3F0(0, &VA[0], 1); } break; /* it_802AB3F0(NULL) */
            case 19: { it_802AB3F0(&GO[0], 0, 1); } break; /* it_802AB3F0(out NULL) */
            case 20: { it_802AB468(0, &VA[0].x, 1); } break; /* it_802AB468(NULL) */
            case 21: { ret = PI(it_802AB568(0)); } break; /* it_802AB568(NULL) */
            case 22: { { int j; for (j = 0; j < 1; j++) { HM((u32) it_803F6BC8[j].anim_id); HM(FI((const void*) it_803F6BC8[j].animated)); HM(FI((const void*) it_803F6BC8[j].physics_updated)); HM(FI((const void*) it_803F6BC8[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
