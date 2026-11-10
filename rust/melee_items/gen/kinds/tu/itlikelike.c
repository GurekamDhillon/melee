/* TU: src/melee/it/kinds/itlikelike.c */
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

static Vec3 LLX[3]; static it_804D6D40_t D40;
#include "itlikelike_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        { int q; for (q = 0; q < 3; q++) {
        itLikelikeAttributes* a = (itLikelikeAttributes*) SPEC[q];
        *(int*) &LLX[q].x = (int) (rnd() % 40); LLX[q].y = rw(); LLX[q].z = rw();
        a->x0 = (itSpecialAttrsHead*) &LLX[q]; a->x4 = (int) (rnd() % 20); a->x8 = (int) (rnd() % 10); a->xC = (int) (rnd() % 14) + 1;
        a->x18 = (int) (rnd() % 4); a->x1C = (float) (rnd() % 30); a->x20 = (float) (rnd() % 30); a->x24 = (float) (rnd() % 30);
        a->x28 = (float) (rnd() % 30); a->x2C = (float) (rnd() % 30); a->x30 = rw(); a->x34 = rw(); a->x38 = (float) (rnd() % 45);
        a->x3C = (s8) (rnd() % 3); a->x3D = (s8) (rnd() % 4); a->x3E = (s8) (rnd() % 4);
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
        ART[k].x4_specialAttributes = (union ItemSpecialAttributes*) SPEC[k];
        for (i = 0; i < 14; i++) DBT[k].bones[i] = &JO[6 + 14 * k + i];
    }

    IT[0].owner = (rnd() & 3) ? &GO[1] : 0;
    IT[0].xC64_reflectGObj = (rnd() & 3) ? &GO[1] : 0;
    if (rnd() & 1) IT[0].msid = (int) (rnd() % 14);
    if (rnd() & 1) IT[0].facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
    IT[0].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
    spec_fill(prof);
        it_804D6D40 = &D40; D40.x8 = (float) (rnd() % 1000) / 1000.0f;
    for (k = 0; k < 3; k++) {
        IT[k].xDD4_itemVar.likelike.x18 = (int) (rnd() % 4);
        IT[k].xDD4_itemVar.likelike.x20.x = rw(); IT[k].xDD4_itemVar.likelike.x20.y = rw(); IT[k].xDD4_itemVar.likelike.x20.z = rw();
        IT[k].xDD4_itemVar.likelike.x38 = (int) (rnd() % 3);
        IT[k].xDD4_itemVar.likelike.x3C = (rnd() & 1) ? 1 : ((rnd() & 1) ? -1 : 0);
        IT[k].xDD4_itemVar.likelike.x44 = (int) (rnd() % 0x40) - 2;
        IT[k].xDD4_itemVar.likelike.x48 = (int) (rnd() % 7);
        IT[k].xDD4_itemVar.likelike.x4C = (int) (rnd() % 8) - 2;
        IT[k].xDD4_itemVar.likelike.x50 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].grab_victim = ((rnd() & 3) == 0) ? IT[k].xDD4_itemVar.likelike.x50 : (((rnd() & 1) ? &GO[(k + 2) % 3] : 0));
        IT[k].atk_victim = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xC9C = (int) (rnd() % 60);
        IT[k].x378_itemColl.ecb.top.y = rw(); IT[k].x378_itemColl.ecb.bottom.y = rw();
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.likelike.x50)); IT[k].xDD4_itemVar.likelike.x50 = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 91; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  ret = (u32) ((it_802D9A2C(&GO[0])) != 0); } break; /* it_802D9A2C */
            case 1: {  it_802D9B78(&GO[0]); } break; /* it_802D9B78 */
            case 2: {  it_802D9BA8(&GO[0]); } break; /* it_802D9BA8 */
            case 3: {  ret = (u32) ((it_2725_Logic5_DmgReceived(&GO[0])) != 0); } break; /* it_2725_Logic5_DmgReceived */
            case 4: {  it_802D9DDC(&GO[0]); } break; /* it_802D9DDC */
            case 5: {  ret = (u32) ((itLikelike_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion0_Anim */
            case 6: {  itLikelike_UnkMotion0_Phys(&GO[0]); } break; /* itLikelike_UnkMotion0_Phys */
            case 7: {  ret = (u32) ((itLikelike_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion0_Coll */
            case 8: {  ret = (u32) ((itLikelike_UnkMotion7_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion7_Anim */
            case 9: {  itLikelike_UnkMotion7_Phys(&GO[0]); } break; /* itLikelike_UnkMotion7_Phys */
            case 10: {  ret = (u32) ((itLikelike_UnkMotion7_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion7_Coll */
            case 11: {  it_802DA104(&GO[0]); } break; /* it_802DA104 */
            case 12: {  ret = (u32) ((itLikelike_UnkMotion1_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion1_Anim */
            case 13: {  itLikelike_UnkMotion1_Phys(&GO[0]); } break; /* itLikelike_UnkMotion1_Phys */
            case 14: {  ret = (u32) ((itLikelike_UnkMotion1_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion1_Coll */
            case 15: {  it_802DA4C0(&GO[0]); } break; /* it_802DA4C0 */
            case 16: {  ret = (u32) ((itLikelike_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion2_Anim */
            case 17: {  itLikelike_UnkMotion2_Phys(&GO[0]); } break; /* itLikelike_UnkMotion2_Phys */
            case 18: {  ret = (u32) ((itLikelike_UnkMotion2_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion2_Coll */
            case 19: {  it_802DA8D8(&GO[0]); } break; /* it_802DA8D8 */
            case 20: {  it_802DA960(&GO[0]); } break; /* it_802DA960 */
            case 21: {  it_802DAA10(&GO[0]); } break; /* it_802DAA10 */
            case 22: {  ret = (u32) ((itLikelike_UnkMotion5_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion5_Anim */
            case 23: {  itLikelike_UnkMotion5_Phys(&GO[0]); } break; /* itLikelike_UnkMotion5_Phys */
            case 24: {  ret = (u32) ((itLikelike_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion4_Coll */
            case 25: {  ret = (u32) ((itLikelike_UnkMotion5_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion5_Coll */
            case 26: {  it_802DABC0(&GO[0]); } break; /* it_802DABC0 */
            case 27: {  ret = (u32) ((itLikelike_UnkMotion17_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion17_Anim */
            case 28: {  itLikelike_UnkMotion17_Phys(&GO[0]); } break; /* itLikelike_UnkMotion17_Phys */
            case 29: {  ret = (u32) ((itLikelike_UnkMotion17_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion17_Coll */
            case 30: {  it_802DAD18(&GO[0]); } break; /* it_802DAD18 */
            case 31: {  ret = (u32) ((itLikelike_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion3_Anim */
            case 32: {  itLikelike_UnkMotion3_Phys(&GO[0]); } break; /* itLikelike_UnkMotion3_Phys */
            case 33: {  ret = (u32) ((itLikelike_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion3_Coll */
            case 34: {  it_802DAE6C(&GO[0]); } break; /* it_802DAE6C */
            case 35: {  ret = (u32) ((itLikelike_UnkMotion16_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion16_Anim */
            case 36: {  itLikelike_UnkMotion16_Phys(&GO[0]); } break; /* itLikelike_UnkMotion16_Phys */
            case 37: {  ret = (u32) ((itLikelike_UnkMotion16_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion16_Coll */
            case 38: {  it_802DB074(&GO[0]); } break; /* it_802DB074 */
            case 39: {  ret = (u32) ((itLikelike_UnkMotion8_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion8_Anim */
            case 40: {  itLikelike_UnkMotion8_Phys(&GO[0]); } break; /* itLikelike_UnkMotion8_Phys */
            case 41: {  ret = (u32) ((itLikelike_UnkMotion8_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion8_Coll */
            case 42: {  it_802DB358(&GO[0]); } break; /* it_802DB358 */
            case 43: {  it_802DB398(&GO[0]); } break; /* it_802DB398 */
            case 44: {  ret = (u32) ((itLikelike_UnkMotion12_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion12_Anim */
            case 45: {  itLikelike_UnkMotion12_Phys(&GO[0]); } break; /* itLikelike_UnkMotion12_Phys */
            case 46: {  ret = (u32) ((itLikelike_UnkMotion12_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion12_Coll */
            case 47: {  it_802DB5F0(&GO[0]); } break; /* it_802DB5F0 */
            case 48: {  ret = (u32) ((itLikelike_UnkMotion9_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion9_Anim */
            case 49: {  itLikelike_UnkMotion9_Phys(&GO[0]); } break; /* itLikelike_UnkMotion9_Phys */
            case 50: {  ret = (u32) ((itLikelike_UnkMotion9_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion9_Coll */
            case 51: {  it_802DB74C(&GO[0]); } break; /* it_802DB74C */
            case 52: {  ret = (u32) ((itLikelike_UnkMotion10_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion10_Anim */
            case 53: {  itLikelike_UnkMotion10_Phys(&GO[0]); } break; /* itLikelike_UnkMotion10_Phys */
            case 54: {  ret = (u32) ((itLikelike_UnkMotion10_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion10_Coll */
            case 55: {  it_802DB8A8(&GO[0]); } break; /* it_802DB8A8 */
            case 56: {  ret = (u32) ((itLikelike_UnkMotion13_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion13_Anim */
            case 57: {  itLikelike_UnkMotion13_Phys(&GO[0]); } break; /* itLikelike_UnkMotion13_Phys */
            case 58: {  ret = (u32) ((itLikelike_UnkMotion13_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion13_Coll */
            case 59: {  it_802DB9F4(&GO[0]); } break; /* it_802DB9F4 */
            case 60: {  it_802DBA68(&GO[0]); } break; /* it_802DBA68 */
            case 61: { s32 v1 = (s32) ((int) (rnd() % 3) - 1); s32 v2 = (s32) ((int) (rnd() % 3)); it_802DBAF0(&GO[0], v1, v2); } break; /* it_802DBAF0 */
            case 62: {  ret = (u32) ((itLikelike_UnkMotion14_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion14_Anim */
            case 63: {  itLikelike_UnkMotion14_Phys(&GO[0]); } break; /* itLikelike_UnkMotion14_Phys */
            case 64: {  ret = (u32) ((itLikelike_UnkMotion14_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion14_Coll */
            case 65: {  ret = (u32) ((itLikelike_UnkMotion15_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion15_Anim */
            case 66: {  itLikelike_UnkMotion15_Phys(&GO[0]); } break; /* itLikelike_UnkMotion15_Phys */
            case 67: {  ret = (u32) ((itLikelike_UnkMotion15_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion15_Coll */
            case 68: {  itLikeLike_Logic5_PickedUp(&GO[0]); } break; /* itLikeLike_Logic5_PickedUp */
            case 69: {  ret = (u32) ((itLikelike_UnkMotion18_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion18_Anim */
            case 70: {  itLikelike_UnkMotion18_Phys(&GO[0]); } break; /* itLikelike_UnkMotion18_Phys */
            case 71: {  it_2725_Logic5_Dropped(&GO[0]); } break; /* it_2725_Logic5_Dropped */
            case 72: {  itLikeLike_Logic5_Thrown(&GO[0]); } break; /* itLikeLike_Logic5_Thrown */
            case 73: {  ret = (u32) ((itLikelike_UnkMotion19_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion19_Anim */
            case 74: {  itLikelike_UnkMotion19_Phys(&GO[0]); } break; /* itLikelike_UnkMotion19_Phys */
            case 75: {  ret = (u32) ((itLikelike_UnkMotion19_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion19_Coll */
            case 76: {  it_802DC0AC(&GO[0]); } break; /* it_802DC0AC */
            case 77: {  ret = (u32) ((itLikelike_UnkMotion6_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion6_Anim */
            case 78: {  itLikelike_UnkMotion6_Phys(&GO[0]); } break; /* itLikelike_UnkMotion6_Phys */
            case 79: {  ret = (u32) ((itLikelike_UnkMotion6_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion6_Coll */
            case 80: {  it_802DC310(&GO[0]); } break; /* it_802DC310 */
            case 81: {  ret = (u32) ((itLikelike_UnkMotion20_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion20_Anim */
            case 82: {  itLikelike_UnkMotion20_Phys(&GO[0]); } break; /* itLikelike_UnkMotion20_Phys */
            case 83: {  ret = (u32) ((itLikelike_UnkMotion20_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion20_Coll */
            case 84: {  it_802DC3DC(&GO[0]); } break; /* it_802DC3DC */
            case 85: {  ret = (u32) ((itLikelike_UnkMotion21_Anim(&GO[0])) != 0); } break; /* itLikelike_UnkMotion21_Anim */
            case 86: {  itLikelike_UnkMotion21_Phys(&GO[0]); } break; /* itLikelike_UnkMotion21_Phys */
            case 87: {  ret = (u32) ((itLikelike_UnkMotion21_Coll(&GO[0])) != 0); } break; /* itLikelike_UnkMotion21_Coll */
            case 88: {  itLikeLike_Logic5_Destroyed(&GO[0]); } break; /* itLikeLike_Logic5_Destroyed */
            case 89: { s32 v0 = (s32) ((int) (rnd() % 3)); s32 v2 = (s32) ((int) (rnd() % 3)); ret = PI(it_802DC4BC(v0, &VA[0], v2)); } break; /* it_802DC4BC */
            case 90: { { int j; for (j = 0; j < 22; j++) { HM((u32) it_803F8468[j].anim_id); HM(FI((const void*) it_803F8468[j].animated)); HM(FI((const void*) it_803F8468[j].physics_updated)); HM(FI((const void*) it_803F8468[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
