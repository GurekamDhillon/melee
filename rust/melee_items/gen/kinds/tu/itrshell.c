/* TU: src/melee/it/kinds/itrshell.c */
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


#include "itrshell_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itRShell_Attrs* a = (itRShell_Attrs*) SPEC[0];
        a->x0 = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = (float) (rnd() % 4) / 2.0f; a->x10 = (float) (rnd() % 5) / 2.0f;
        a->x14.x = rw(); a->x14.y = rw(); a->x14.z = rw(); a->x20 = rw(); a->x24 = rw(); a->x28 = rw(); a->x2C = rw();
        a->x38 = rw(); a->x3C = rw(); a->x40 = rw(); a->x44 = rw(); a->x48.x = rw(); a->x48.y = rw(); a->x48.z = rw();
        a->x54 = (int) (rnd() % 4);
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
        IT[k].msid = (int) (rnd() % 10) - 1;
        IT[k].xDD4_itemVar.rshell.xDD4 = (float) ((int) (rnd() % 4) - 1);
        IT[k].xDD4_itemVar.rshell.xDD8 = (float) ((int) (rnd() % 4) - 1);
        IT[k].xDD4_itemVar.rshell.xDDC = (float) ((int) (rnd() % 4) - 1);
        IT[k].xDD4_itemVar.rshell.xDE0 = (float) ((int) (rnd() % 5) - 2);
        IT[k].xDD4_itemVar.rshell.xDE4 = (float) ((int) (rnd() % 4) - 1);
        IT[k].xDD4_itemVar.rshell.xDE8 = (int) (rnd() & 1);
        IT[k].xDD4_itemVar.rshell.xDEC = (int) (rnd() % 4);
        IT[k].xDD4_itemVar.rshell.xDF0 = (int) (rnd() & 1);
        IT[k].xDD4_itemVar.rshell.xDF4_b0 = rnd() & 1;
        IT[k].xDCD_flag.b5 = rnd() & 1;
        if (rnd() & 1) IT[k].x40_vel.x = (float) ((int) (rnd() % 5) - 2) / 2.0f;
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(IT[k].xDD4_itemVar.rshell.xDF4_b0); zero(&IT[k].xDD4_itemVar.rshell, sizeof IT[k].xDD4_itemVar.rshell); HM(IT[k].xDCD_flag.b5); *(unsigned char*) &IT[k].xDCD_flag = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 50; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_8028CFE0(&GO[0]); } break; /* it_8028CFE0 */
            case 1: {  it_8028D090(&GO[0]); } break; /* it_8028D090 */
            case 2: {  it_8028D100(&GO[0]); } break; /* it_8028D100 */
            case 3: {  it_8028D26C(&GO[0]); } break; /* it_8028D26C */
            case 4: {  it_8028D390(&GO[0]); } break; /* it_8028D390 */
            case 5: {  it_8028D3B8(&GO[0]); } break; /* it_8028D3B8 */
            case 6: {  fn_8028D4A8(&GO[0]); } break; /* fn_8028D4A8 */
            case 7: {  it_8028D4E4(&GO[0]); } break; /* it_8028D4E4 */
            case 8: { f32 v1 = (f32) (rw()); f32 v2 = (f32) (rw()); ret = B(it_8028D56C(&GO[0], v1, v2)); } break; /* it_8028D56C */
            case 9: {  it_3F14_Logic15_Spawned(&GO[0]); } break; /* it_3F14_Logic15_Spawned */
            case 10: {  it_8028D62C(&GO[0]); } break; /* it_8028D62C */
            case 11: {  ret = (u32) itRshell_UnkMotion0_Anim(&GO[0]); } break; /* itRshell_UnkMotion0_Anim */
            case 12: {  itRshell_UnkMotion0_Phys(&GO[0]); } break; /* itRshell_UnkMotion0_Phys */
            case 13: {  ret = (u32) itRshell_UnkMotion0_Coll(&GO[0]); } break; /* itRshell_UnkMotion0_Coll */
            case 14: {  it_8028D7F0(&GO[0]); } break; /* it_8028D7F0 */
            case 15: {  ret = (u32) itRshell_UnkMotion1_Anim(&GO[0]); } break; /* itRshell_UnkMotion1_Anim */
            case 16: {  itRshell_UnkMotion1_Phys(&GO[0]); } break; /* itRshell_UnkMotion1_Phys */
            case 17: {  ret = (u32) itRshell_UnkMotion1_Coll(&GO[0]); } break; /* itRshell_UnkMotion1_Coll */
            case 18: {  itRShell_Logic15_PickedUp(&GO[0]); } break; /* itRShell_Logic15_PickedUp */
            case 19: {  ret = (u32) itRshell_UnkMotion2_Anim(&GO[0]); } break; /* itRshell_UnkMotion2_Anim */
            case 20: {  itRshell_UnkMotion2_Phys(&GO[0]); } break; /* itRshell_UnkMotion2_Phys */
            case 21: {  it_3F14_Logic15_Thrown(&GO[0]); } break; /* it_3F14_Logic15_Thrown */
            case 22: {  ret = (u32) itRshell_UnkMotion3_Anim(&GO[0]); } break; /* itRshell_UnkMotion3_Anim */
            case 23: {  itRshell_UnkMotion3_Phys(&GO[0]); } break; /* itRshell_UnkMotion3_Phys */
            case 24: {  ret = (u32) itRshell_UnkMotion3_Coll(&GO[0]); } break; /* itRshell_UnkMotion3_Coll */
            case 25: {  it_3F14_Logic15_Dropped(&GO[0]); } break; /* it_3F14_Logic15_Dropped */
            case 26: {  ret = (u32) itRshell_UnkMotion4_Anim(&GO[0]); } break; /* itRshell_UnkMotion4_Anim */
            case 27: {  itRshell_UnkMotion4_Phys(&GO[0]); } break; /* itRshell_UnkMotion4_Phys */
            case 28: {  ret = (u32) itRshell_UnkMotion4_Coll(&GO[0]); } break; /* itRshell_UnkMotion4_Coll */
            case 29: {  it_8028DAE4(&GO[0]); } break; /* it_8028DAE4 */
            case 30: {  ret = (u32) itRshell_UnkMotion5_Anim(&GO[0]); } break; /* itRshell_UnkMotion5_Anim */
            case 31: {  itRshell_UnkMotion5_Phys(&GO[0]); } break; /* itRshell_UnkMotion5_Phys */
            case 32: {  ret = (u32) itRshell_UnkMotion5_Coll(&GO[0]); } break; /* itRshell_UnkMotion5_Coll */
            case 33: {  it_8028E170(&GO[0]); } break; /* it_8028E170 */
            case 34: {  ret = (u32) itRshell_UnkMotion6_Anim(&GO[0]); } break; /* itRshell_UnkMotion6_Anim */
            case 35: {  itRshell_UnkMotion6_Phys(&GO[0]); } break; /* itRshell_UnkMotion6_Phys */
            case 36: {  ret = (u32) itRshell_UnkMotion6_Coll(&GO[0]); } break; /* itRshell_UnkMotion6_Coll */
            case 37: {  it_3F14_Logic15_EnteredAir(&GO[0]); } break; /* it_3F14_Logic15_EnteredAir */
            case 38: {  ret = (u32) itRshell_UnkMotion7_Anim(&GO[0]); } break; /* itRshell_UnkMotion7_Anim */
            case 39: {  itRshell_UnkMotion7_Phys(&GO[0]); } break; /* itRshell_UnkMotion7_Phys */
            case 40: {  ret = (u32) itRshell_UnkMotion7_Coll(&GO[0]); } break; /* itRshell_UnkMotion7_Coll */
            case 41: {  ret = (u32) it_3F14_Logic15_DmgDealt(&GO[0]); } break; /* it_3F14_Logic15_DmgDealt */
            case 42: {  ret = (u32) it_3F14_Logic15_DmgReceived(&GO[0]); } break; /* it_3F14_Logic15_DmgReceived */
            case 43: {  ret = (u32) itRShell_Logic15_Reflected(&GO[0]); } break; /* itRShell_Logic15_Reflected */
            case 44: {  ret = (u32) itRShell_Logic15_Clanked(&GO[0]); } break; /* itRShell_Logic15_Clanked */
            case 45: {  ret = (u32) it_3F14_Logic15_HitShield(&GO[0]); } break; /* it_3F14_Logic15_HitShield */
            case 46: {  ret = (u32) itRShell_Logic15_ShieldBounced(&GO[0]); } break; /* itRShell_Logic15_ShieldBounced */
            case 47: {  ret = (u32) it_8028E6C0(&GO[0]); } break; /* it_8028E6C0 */
            case 48: {  itRShell_Logic15_EvtUnk(&GO[0], &GO[1]); } break; /* itRShell_Logic15_EvtUnk */
            case 49: { { int j; for (j = 0; j < 8; j++) { HM((u32) it_803F5C48[j].anim_id); HM(FI((const void*) it_803F5C48[j].animated)); HM(FI((const void*) it_803F5C48[j].physics_updated)); HM(FI((const void*) it_803F5C48[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
