/* TU: src/melee/it/kinds/ittaru.c */
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

static ItemModelDesc MD;
static HSD_Joint JNT[1];

#include "ittaru_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        int* a = (int*) SPEC[0];
        a[0] = (int) (rnd() % 4); a[1] = (int) (rnd() % 4);
        for (i = 2; i < 14; i++) ((float*) a)[i] = rw();
        ((float*) a)[2] = (float) (rnd() % 3); ((float*) a)[3] = (float) (rnd() % 120);
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
    for (k = 0; k < 3; k++) {
        IT[k].xDD4_itemVar.taru.xDD4 = (int) (rnd() % 3 == 0);
        IT[k].xDD4_itemVar.taru.xDD8 = (int) (rnd() % 4) - 1;
        IT[k].xDD4_itemVar.taru.xDDC = rg(); IT[k].xDD4_itemVar.taru.xDE0 = rg(); IT[k].xDD4_itemVar.taru.xDE4 = rg();
        IT[k].xDD4_itemVar.taru.xDE8.x = rg(); IT[k].xDD4_itemVar.taru.xDE8.y = rg(); IT[k].xDD4_itemVar.taru.xDE8.z = rg();
        IT[k].xDAC_itcmd_var0 = (u32) (rnd() % 3);
        IT[k].xDB0_itcmd_var1 = (u32) (rnd() & 1);
        IT[k].msid = (int) (rnd() % 8);
        IT[k].xC9C = (int) (rnd() % 120);
        IT[k].xD44_lifeTimer = (float) (rnd() % 4);
        IT[k].facing_dir = (rnd() % 3 == 0) ? rw() : ((rnd() & 1) ? 1.0f : -1.0f);
        IT[k].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
        IT[k].x378_itemColl.env_flags = (rnd() % 5 == 0 ? 0 : (rnd() & 1) ? 0x18000 : (rnd() & 1) ? 0x3F : (rnd() & 1) ? 0xFC0 : 0x18000 | 0x3F);
        ART[k].x10_modelDesc = &MD;
        MD.x0_joint = &JNT[0];
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    preg(&MD, sizeof MD); preg(JNT, sizeof JNT);
    freg_all();
    for (op = 0; op < 43; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_3F14_Logic2_Spawned(&GO[0]); } break; /* it_3F14_Logic2_Spawned */
            case 1: {  it_802874F0(&GO[0]); } break; /* it_802874F0 */
            case 2: {  it_80287690(&GO[0]); } break; /* it_80287690 */
            case 3: {  ret = (u32) ((itTaru_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itTaru_UnkMotion0_Anim */
            case 4: {  itTaru_UnkMotion0_Phys(&GO[0]); } break; /* itTaru_UnkMotion0_Phys */
            case 5: {  ret = (u32) ((itTaru_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itTaru_UnkMotion0_Coll */
            case 6: {  it_80287D0C(&GO[0]); } break; /* it_80287D0C */
            case 7: {  ret = (u32) ((itTaru_UnkMotion1_Anim(&GO[0])) != 0); } break; /* itTaru_UnkMotion1_Anim */
            case 8: {  itTaru_UnkMotion1_Phys(&GO[0]); } break; /* itTaru_UnkMotion1_Phys */
            case 9: {  ret = (u32) ((itTaru_UnkMotion1_Coll(&GO[0])) != 0); } break; /* itTaru_UnkMotion1_Coll */
            case 10: {  itTaru_Logic2_PickedUp(&GO[0]); } break; /* itTaru_Logic2_PickedUp */
            case 11: {  ret = (u32) ((itTaru_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itTaru_UnkMotion2_Anim */
            case 12: {  itTaru_UnkMotion2_Phys(&GO[0]); } break; /* itTaru_UnkMotion2_Phys */
            case 13: {  itTaru_Logic2_Dropped(&GO[0]); } break; /* itTaru_Logic2_Dropped */
            case 14: {  itTaru_Logic2_Thrown(&GO[0]); } break; /* itTaru_Logic2_Thrown */
            case 15: {  it_80287F20(&GO[0]); } break; /* it_80287F20 */
            case 16: {  ret = (u32) ((itTaru_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itTaru_UnkMotion3_Anim */
            case 17: {  itTaru_UnkMotion3_Phys(&GO[0]); } break; /* itTaru_UnkMotion3_Phys */
            case 18: {  ret = (u32) ((itTaru_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itTaru_UnkMotion3_Coll */
            case 19: {  it_80288194(&GO[0]); } break; /* it_80288194 */
            case 20: {  it_802881B4(&GO[0]); } break; /* it_802881B4 */
            case 21: {  it_802881FC(&GO[0]); } break; /* it_802881FC */
            case 22: {  ret = (u32) ((itTaru_UnkMotion5_Anim(&GO[0])) != 0); } break; /* itTaru_UnkMotion5_Anim */
            case 23: {  itTaru_UnkMotion5_Phys(&GO[0]); } break; /* itTaru_UnkMotion5_Phys */
            case 24: {  ret = (u32) ((itTaru_UnkMotion5_Coll(&GO[0])) != 0); } break; /* itTaru_UnkMotion5_Coll */
            case 25: {  ret = (u32) ((itTaru_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itTaru_UnkMotion4_Anim */
            case 26: {  itTaru_UnkMotion4_Phys(&GO[0]); } break; /* itTaru_UnkMotion4_Phys */
            case 27: {  ret = (u32) ((itTaru_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itTaru_UnkMotion4_Coll */
            case 28: {  it_802885C8(&GO[0]); } break; /* it_802885C8 */
            case 29: {  ret = (u32) ((itTaru_UnkMotion6_Anim(&GO[0])) != 0); } break; /* itTaru_UnkMotion6_Anim */
            case 30: {  itTaru_UnkMotion6_Phys(&GO[0]); } break; /* itTaru_UnkMotion6_Phys */
            case 31: {  ret = (u32) ((itTaru_UnkMotion6_Coll(&GO[0])) != 0); } break; /* itTaru_UnkMotion6_Coll */
            case 32: {  it_802886C4(&GO[0]); } break; /* it_802886C4 */
            case 33: {  ret = (u32) ((itTaru_UnkMotion7_Anim(&GO[0])) != 0); } break; /* itTaru_UnkMotion7_Anim */
            case 34: {  itTaru_UnkMotion7_Phys(&GO[0]); } break; /* itTaru_UnkMotion7_Phys */
            case 35: {  ret = (u32) ((itTaru_UnkMotion7_Coll(&GO[0])) != 0); } break; /* itTaru_UnkMotion7_Coll */
            case 36: {  ret = (u32) ((it_3F14_Logic2_DmgDealt(&GO[0])) != 0); } break; /* it_3F14_Logic2_DmgDealt */
            case 37: {  ret = (u32) ((it_3F14_Logic2_Clanked(&GO[0])) != 0); } break; /* it_3F14_Logic2_Clanked */
            case 38: {  ret = (u32) ((it_3F14_Logic2_HitShield(&GO[0])) != 0); } break; /* it_3F14_Logic2_HitShield */
            case 39: {  ret = (u32) ((it_3F14_Logic2_Reflected(&GO[0])) != 0); } break; /* it_3F14_Logic2_Reflected */
            case 40: {  ret = (u32) ((it_3F14_Logic2_DmgReceived(&GO[0])) != 0); } break; /* it_3F14_Logic2_DmgReceived */
            case 41: {  itTaru_Logic2_EvtUnk(&GO[0], &GO[1]); } break; /* itTaru_Logic2_EvtUnk */
            case 42: { { int j; for (j = 0; j < 8; j++) { HM((u32) it_803F58E0[j].anim_id); HM(FI((const void*) it_803F58E0[j].animated)); HM(FI((const void*) it_803F58E0[j].physics_updated)); HM(FI((const void*) it_803F58E0[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
