/* TU: src/melee/it/kinds/itmsbomb.c */
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


#include "itmsbomb_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itMsBomb_Attrs* a = (itMsBomb_Attrs*) SPEC[0];
        a->x0 = rw(); a->x4 = rw(); a->x8.top = rw(); a->x8.bottom = rw(); a->x8.right = rw(); a->x8.left = rw();
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
    it_804D6D28 = &ICD;
    IT[0].owner = (rnd() & 3) ? &GO[1] : 0;
    IT[0].xC64_reflectGObj = (rnd() & 3) ? &GO[1] : 0;
    if (rnd() & 1) IT[0].msid = (int) (rnd() % 14);
    if (rnd() & 1) IT[0].facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
    IT[0].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
    spec_fill(prof);
        for (k = 0; k < 3; k++) { IT[k].xDD4_itemVar.msbomb.xDD4_b0 = rnd() & 1; }
        IT[0].msid = (int) (rnd() % 8);
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(IT[k].xDD4_itemVar.msbomb.xDD4_b0); zero(&IT[k].xDD4_itemVar.msbomb, sizeof IT[k].xDD4_itemVar.msbomb); }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 32; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_8028FE90(&GO[0]); } break; /* it_8028FE90 */
            case 1: {  itMSBomb_Logic19_Spawned(&GO[0]); } break; /* itMSBomb_Logic19_Spawned */
            case 2: {  it_8028FF1C(&GO[0]); } break; /* it_8028FF1C */
            case 3: {  ret = (u32) itMsbomb_UnkMotion0_Coll(&GO[0]); } break; /* itMsbomb_UnkMotion0_Coll */
            case 4: {  it_8028FF8C(&GO[0]); } break; /* it_8028FF8C */
            case 5: {  itMsbomb_UnkMotion1_Phys(&GO[0]); } break; /* itMsbomb_UnkMotion1_Phys */
            case 6: {  ret = (u32) itMsbomb_UnkMotion1_Coll(&GO[0]); } break; /* itMsbomb_UnkMotion1_Coll */
            case 7: {  itMSBomb_Logic19_PickedUp(&GO[0]); } break; /* itMSBomb_Logic19_PickedUp */
            case 8: {  itMSBomb_Logic19_Dropped(&GO[0]); } break; /* itMSBomb_Logic19_Dropped */
            case 9: {  itMSBomb_Logic19_Thrown(&GO[0]); } break; /* itMSBomb_Logic19_Thrown */
            case 10: {  itMsbomb_UnkMotion3_Phys(&GO[0]); } break; /* itMsbomb_UnkMotion3_Phys */
            case 11: {  ret = (u32) itMsbomb_UnkMotion3_Coll(&GO[0]); } break; /* itMsbomb_UnkMotion3_Coll */
            case 12: {  it_80290238(&GO[0]); } break; /* it_80290238 */
            case 13: {  it_80290314(&GO[0]); } break; /* it_80290314 */
            case 14: {  ret = (u32) itMsbomb_UnkMotion4_Coll(&GO[0]); } break; /* itMsbomb_UnkMotion4_Coll */
            case 15: {  it_8029047C(&GO[0]); } break; /* it_8029047C */
            case 16: {  itMsbomb_UnkMotion5_Phys(&GO[0]); } break; /* itMsbomb_UnkMotion5_Phys */
            case 17: {  ret = (u32) itMsbomb_UnkMotion5_Coll(&GO[0]); } break; /* itMsbomb_UnkMotion5_Coll */
            case 18: {  it_802905D8(&GO[0]); } break; /* it_802905D8 */
            case 19: {  it_80290614(&GO[0]); } break; /* it_80290614 */
            case 20: {  ret = (u32) itMsbomb_UnkMotion6_Anim(&GO[0]); } break; /* itMsbomb_UnkMotion6_Anim */
            case 21: {  ret = (u32) itMSBomb_Logic19_DmgDealt(&GO[0]); } break; /* itMSBomb_Logic19_DmgDealt */
            case 22: {  ret = (u32) itMSBomb_Logic19_DmgReceived(&GO[0]); } break; /* itMSBomb_Logic19_DmgReceived */
            case 23: {  itMSBomb_Logic19_EnteredAir(&GO[0]); } break; /* itMSBomb_Logic19_EnteredAir */
            case 24: {  ret = (u32) itMsbomb_UnkMotion7_Coll(&GO[0]); } break; /* itMsbomb_UnkMotion7_Coll */
            case 25: {  ret = (u32) itMSBomb_Logic19_Clanked(&GO[0]); } break; /* itMSBomb_Logic19_Clanked */
            case 26: {  ret = (u32) itMSBomb_Logic19_Reflected(&GO[0]); } break; /* itMSBomb_Logic19_Reflected */
            case 27: {  ret = (u32) itMSBomb_Logic19_HitShield(&GO[0]); } break; /* itMSBomb_Logic19_HitShield */
            case 28: {  ret = (u32) itMSBomb_Logic19_ShieldBounced(&GO[0]); } break; /* itMSBomb_Logic19_ShieldBounced */
            case 29: {  ret = (u32) it_802908D8(&GO[0]); } break; /* it_802908D8 */
            case 30: {  itMSBomb_Logic19_EvtUnk(&GO[0], &GO[1]); } break; /* itMSBomb_Logic19_EvtUnk */
            case 31: { { int j; for (j = 0; j < 8; j++) { HM((u32) ItemStateTable_MsBomb[j].anim_id); HM(FI((const void*) ItemStateTable_MsBomb[j].animated)); HM(FI((const void*) ItemStateTable_MsBomb[j].physics_updated)); HM(FI((const void*) ItemStateTable_MsBomb[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
