/* TU: src/melee/it/kinds/itbox.c */
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


#include "itbox_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itBoxAttributes* a = (itBoxAttributes*) SPEC[0];
        a->spawn_weight_0 = (int) (rnd() % 5); a->spawn_weight_1 = (int) (rnd() % 5); a->spawn_weight_2 = (int) (rnd() % 5);
        a->empty_weight = (int) (rnd() % 5); a->special_spawn_weight = (int) (rnd() % 5);
        a->damage_threshold = (float) (rnd() % 100); a->bounce_angle_threshold = rw(); a->break_vel_threshold = rw();
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
        IT[k].xDD4_itemVar.box.opened = (int) (rnd() & 1);
        IT[k].xDD4_itemVar.box.despawn_timer = (int) (rnd() % 5) - 1;
        IT[k].xDD4_itemVar.box.spawned_gobj = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xDD4_itemVar.box.rot_vel_x = rg();
        IT[k].xDD4_itemVar.box.rot_vel_y = rg();
        IT[k].xC9C = (int) (rnd() % 120);
        if (rnd() & 1) IT[k].x378_itemColl.env_flags = Collide_FloorPush | Collide_FloorHug;
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.box.spawned_gobj)); IT[k].xDD4_itemVar.box.spawned_gobj = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 48; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  ret = PI(it_80286088(&GO[0])); } break; /* it_80286088 */
            case 1: {  itBox_Logic1_Spawned(&GO[0]); } break; /* itBox_Logic1_Spawned */
            case 2: {  itBox_Logic1_Destroyed(&GO[0]); } break; /* itBox_Logic1_Destroyed */
            case 3: { s32 v1 = (s32) ((int) (rnd() % 6)); s32 v2 = (s32) ((int) (rnd() % 6)); s32 v3 = (s32) ((int) (rnd() % 6)); s32 v4 = (s32) ((int) (rnd() % 6)); it_80286248(&GO[0], v1, v2, v3, v4); } break; /* it_80286248 */
            case 4: { s32 v1 = (s32) ((int) (rnd() % 6)); s32 v2 = (s32) ((int) (rnd() % 6)); s32 v3 = (s32) ((int) (rnd() % 6)); s32 v4 = (s32) ((int) (rnd() % 6)); ret = (u32) ((it_80286340(&GO[0], v1, v2, v3, v4)) != 0); } break; /* it_80286340 */
            case 5: {  ret = (u32) ((it_802863BC(&GO[0])) != 0); } break; /* it_802863BC */
            case 6: {  fn_80286480(&GO[0]); } break; /* fn_80286480 */
            case 7: {  ret = (u32) ((itBox_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itBox_UnkMotion0_Anim */
            case 8: {  itBox_UnkMotion0_Phys(&GO[0]); } break; /* itBox_UnkMotion0_Phys */
            case 9: {  ret = (u32) ((itBox_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itBox_UnkMotion0_Coll */
            case 10: {  it_8028655C(&GO[0]); } break; /* it_8028655C */
            case 11: {  ret = (u32) ((itBox_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itBox_UnkMotion4_Anim */
            case 12: {  itBox_UnkMotion1_Phys(&GO[0]); } break; /* itBox_UnkMotion1_Phys */
            case 13: {  ret = (u32) ((itBox_UnkMotion1_Coll(&GO[0])) != 0); } break; /* itBox_UnkMotion1_Coll */
            case 14: {  itBox_Logic1_PickedUp(&GO[0]); } break; /* itBox_Logic1_PickedUp */
            case 15: {  ret = (u32) ((itBox_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itBox_UnkMotion2_Anim */
            case 16: {  itBox_UnkMotion2_Phys(&GO[0]); } break; /* itBox_UnkMotion2_Phys */
            case 17: {  itBox_Logic1_Thrown(&GO[0]); } break; /* itBox_Logic1_Thrown */
            case 18: {  itBox_UnkMotion4_Phys(&GO[0]); } break; /* itBox_UnkMotion4_Phys */
            case 19: {  ret = (u32) ((itBox_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itBox_UnkMotion3_Coll */
            case 20: {  itBox_Logic1_Dropped(&GO[0]); } break; /* itBox_Logic1_Dropped */
            case 21: {  ret = (u32) ((itBox_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itBox_UnkMotion4_Coll */
            case 22: {  it_80286AA4(&GO[0]); } break; /* it_80286AA4 */
            case 23: {  ret = (u32) ((itBox_UnkMotion6_Anim(&GO[0])) != 0); } break; /* itBox_UnkMotion6_Anim */
            case 24: {  itBox_UnkMotion6_Phys(&GO[0]); } break; /* itBox_UnkMotion6_Phys */
            case 25: {  ret = (u32) ((itBox_UnkMotion6_Coll(&GO[0])) != 0); } break; /* itBox_UnkMotion6_Coll */
            case 26: {  it_80286BA0(&GO[0]); } break; /* it_80286BA0 */
            case 27: {  ret = (u32) ((itBox_UnkMotion7_Anim(&GO[0])) != 0); } break; /* itBox_UnkMotion7_Anim */
            case 28: {  itBox_UnkMotion7_Phys(&GO[0]); } break; /* itBox_UnkMotion7_Phys */
            case 29: {  ret = (u32) ((itBox_UnkMotion7_Coll(&GO[0])) != 0); } break; /* itBox_UnkMotion7_Coll */
            case 30: {  ret = (u32) ((itBox_Logic1_DmgDealt(&GO[0])) != 0); } break; /* itBox_Logic1_DmgDealt */
            case 31: {  ret = (u32) ((itBox_Logic1_Clanked(&GO[0])) != 0); } break; /* itBox_Logic1_Clanked */
            case 32: {  ret = (u32) ((itBox_Logic1_HitShield(&GO[0])) != 0); } break; /* itBox_Logic1_HitShield */
            case 33: {  ret = (u32) ((itBox_Logic1_Reflected(&GO[0])) != 0); } break; /* itBox_Logic1_Reflected */
            case 34: {  ret = (u32) ((itBox_Logic1_DmgReceived(&GO[0])) != 0); } break; /* itBox_Logic1_DmgReceived */
            case 35: {  itBox_Logic1_EnteredAir(&GO[0]); } break; /* itBox_Logic1_EnteredAir */
            case 36: {  ret = (u32) ((itBox_UnkMotion5_Anim(&GO[0])) != 0); } break; /* itBox_UnkMotion5_Anim */
            case 37: {  itBox_UnkMotion5_Phys(&GO[0]); } break; /* itBox_UnkMotion5_Phys */
            case 38: {  ret = (u32) ((itBox_UnkMotion5_Coll(&GO[0])) != 0); } break; /* itBox_UnkMotion5_Coll */
            case 39: {  itBox_Logic1_EvtUnk(&GO[0], &GO[1]); } break; /* itBox_Logic1_EvtUnk */
            case 40: {  it_802870A4(&GO[0]); } break; /* it_802870A4 */
            case 41: {  ret = (u32) ((itBox_UnkMotion8_Anim(&GO[0])) != 0); } break; /* itBox_UnkMotion8_Anim */
            case 42: {  itBox_UnkMotion8_Phys(&GO[0]); } break; /* itBox_UnkMotion8_Phys */
            case 43: {  ret = (u32) ((itBox_UnkMotion8_Coll(&GO[0])) != 0); } break; /* itBox_UnkMotion8_Coll */
            case 44: {  it_8028733C(&GO[0]); } break; /* it_8028733C */
            case 45: { ret = PI(it_80286088(0)); } break; /* it_80286088(NULL) */
            case 46: { GO[0].hsd_obj = 0; ret = PI(it_80286088(&GO[0])); } break; /* it_80286088(no jobj) */
            case 47: { { int j; for (j = 0; j < 9; j++) { HM((u32) it_803F5850[j].anim_id); HM(FI((const void*) it_803F5850[j].animated)); HM(FI((const void*) it_803F5850[j].physics_updated)); HM(FI((const void*) it_803F5850[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
