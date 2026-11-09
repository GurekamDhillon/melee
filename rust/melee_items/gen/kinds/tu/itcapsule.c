/* TU: src/melee/it/kinds/itcapsule.c */
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


#include "itcapsule_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        ItCapsuleAttr* a = (ItCapsuleAttr*) SPEC[0];
        a->x0 = (int) (rnd() % 5); a->x4 = (int) (rnd() % 3);
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
        IT[k].xDD4_itemVar.capsule.x0 = (int) (rnd() & 1);
        IT[k].xDD4_itemVar.capsule.x4 = (int) (rnd() & 1);
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 35; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  itCapsule_Logic0_Spawned(&GO[0]); } break; /* itCapsule_Logic0_Spawned */
            case 1: {  it_8027CF30(&GO[0]); } break; /* it_8027CF30 */
            case 2: {  it_8027CFE8(&GO[0]); } break; /* it_8027CFE8 */
            case 3: {  it_8027D0B8(&GO[0]); } break; /* it_8027D0B8 */
            case 4: {  ret = (u32) ((itCapsule_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itCapsule_UnkMotion0_Anim */
            case 5: {  itCapsule_UnkMotion0_Phys(&GO[0]); } break; /* itCapsule_UnkMotion0_Phys */
            case 6: {  ret = (u32) ((itCapsule_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itCapsule_UnkMotion0_Coll */
            case 7: {  it_8027D148(&GO[0]); } break; /* it_8027D148 */
            case 8: {  ret = (u32) ((itCapsule_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itCapsule_UnkMotion4_Anim */
            case 9: {  itCapsule_UnkMotion4_Phys(&GO[0]); } break; /* itCapsule_UnkMotion4_Phys */
            case 10: {  ret = (u32) ((itCapsule_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itCapsule_UnkMotion4_Coll */
            case 11: {  itCapsule_Logic0_PickedUp(&GO[0]); } break; /* itCapsule_Logic0_PickedUp */
            case 12: {  ret = (u32) ((itCapsule_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itCapsule_UnkMotion2_Anim */
            case 13: {  itCapsule_UnkMotion2_Phys(&GO[0]); } break; /* itCapsule_UnkMotion2_Phys */
            case 14: {  itCapsule_Logic0_Dropped(&GO[0]); } break; /* itCapsule_Logic0_Dropped */
            case 15: {  itCapsule_Logic0_Thrown(&GO[0]); } break; /* itCapsule_Logic0_Thrown */
            case 16: {  itCapsule_UnkMotion3_Phys(&GO[0]); } break; /* itCapsule_UnkMotion3_Phys */
            case 17: {  ret = (u32) ((itCapsule_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itCapsule_UnkMotion3_Coll */
            case 18: {  it_8027D2DC(&GO[0]); } break; /* it_8027D2DC */
            case 19: {  ret = (u32) ((itCapsule_UnkMotion5_Anim(&GO[0])) != 0); } break; /* itCapsule_UnkMotion5_Anim */
            case 20: {  itCapsule_UnkMotion5_Phys(&GO[0]); } break; /* itCapsule_UnkMotion5_Phys */
            case 21: {  ret = (u32) ((itCapsule_UnkMotion5_Coll(&GO[0])) != 0); } break; /* itCapsule_UnkMotion5_Coll */
            case 22: {  ret = (u32) ((itCapsule_Logic0_DmgDealt(&GO[0])) != 0); } break; /* itCapsule_Logic0_DmgDealt */
            case 23: {  ret = (u32) ((itCapsule_Logic0_DmgDealt_autoinlined(&GO[0])) != 0); } break; /* itCapsule_Logic0_DmgDealt_autoinlined */
            case 24: {  ret = (u32) ((itCapsule_Logic0_DmgReceived(&GO[0])) != 0); } break; /* itCapsule_Logic0_DmgReceived */
            case 25: {  itCapsule_Logic0_EnteredAir(&GO[0]); } break; /* itCapsule_Logic0_EnteredAir */
            case 26: {  ret = (u32) ((itCapsule_UnkMotion6_Anim(&GO[0])) != 0); } break; /* itCapsule_UnkMotion6_Anim */
            case 27: {  itCapsule_UnkMotion6_Phys(&GO[0]); } break; /* itCapsule_UnkMotion6_Phys */
            case 28: {  ret = (u32) ((itCapsule_UnkMotion6_Coll(&GO[0])) != 0); } break; /* itCapsule_UnkMotion6_Coll */
            case 29: {  ret = (u32) ((itCapsule_Logic0_Clanked(&GO[0])) != 0); } break; /* itCapsule_Logic0_Clanked */
            case 30: {  ret = (u32) ((itCapsule_Logic0_Reflected(&GO[0])) != 0); } break; /* itCapsule_Logic0_Reflected */
            case 31: {  ret = (u32) ((itCapsule_Logic0_HitShield(&GO[0])) != 0); } break; /* itCapsule_Logic0_HitShield */
            case 32: {  ret = (u32) ((itCapsule_Logic0_ShieldBounced(&GO[0])) != 0); } break; /* itCapsule_Logic0_ShieldBounced */
            case 33: {  itCapsule_Logic0_EvtUnk(&GO[0], &GO[1]); } break; /* itCapsule_Logic0_EvtUnk */
            case 34: { { int j; for (j = 0; j < 7; j++) { HM((u32) it_803F5468[j].anim_id); HM(FI((const void*) it_803F5468[j].animated)); HM(FI((const void*) it_803F5468[j].physics_updated)); HM(FI((const void*) it_803F5468[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
