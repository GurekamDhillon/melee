/* TU: src/melee/it/kinds/itwstar.c */
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

static ftCollisionBox BOXV;
#include "itwstar_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itWstarAttributes* a = (itWstarAttributes*) SPEC[0];
        itWstarAttrEntry* en = (itWstarAttrEntry*) &a->x28_entries[0];
        a->x0 = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw(); a->x14 = rw(); a->x18 = rw();
        a->x1C = rw(); a->x20 = rw();
        a->x24_count = 1 + (int) (rnd() % 4);
        for (i = 0; i < 4; i++) { en[i].x0_anim_joint = (HSD_AnimJoint*) &JO[3 + i]; en[i].x4_sfx = (int) (rnd() % 500); }
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
    zero(&BOXV, sizeof BOXV);
    it_804D6D00 = (s8) ((int) (rnd() % 6) - 1);
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    HM((u32) (int) it_804D6D00);
    HM(B(BOXV.top)); HM(B(BOXV.bottom)); HM(B(BOXV.left.x)); HM(B(BOXV.left.y)); HM(B(BOXV.right.x)); HM(B(BOXV.right.y));
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 26; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  ret = PI(it_80294364(&GO[0])); } break; /* it_80294364 */
            case 1: { f32 v1 = (f32) (rw()); f32 v2 = (f32) (rw()); it_80294430(&GO[0], v1, v2); } break; /* it_80294430 */
            case 2: { it_802944AC(&GO[0], &BOXV); } break; /* it_802944AC */
            case 3: {  itWStar_Logic29_Spawned(&GO[0]); } break; /* itWStar_Logic29_Spawned */
            case 4: {  it_8029455C(&GO[0]); } break; /* it_8029455C */
            case 5: {  ret = (u32) itWstar_UnkMotion0_Anim(&GO[0]); } break; /* itWstar_UnkMotion0_Anim */
            case 6: {  itWstar_UnkMotion0_Phys(&GO[0]); } break; /* itWstar_UnkMotion0_Phys */
            case 7: {  ret = (u32) itWstar_UnkMotion0_Coll(&GO[0]); } break; /* itWstar_UnkMotion0_Coll */
            case 8: {  it_80294624(&GO[0]); } break; /* it_80294624 */
            case 9: {  ret = (u32) itWstar_UnkMotion1_Anim(&GO[0]); } break; /* itWstar_UnkMotion1_Anim */
            case 10: {  itWstar_UnkMotion1_Phys(&GO[0]); } break; /* itWstar_UnkMotion1_Phys */
            case 11: {  ret = (u32) itWstar_UnkMotion1_Coll(&GO[0]); } break; /* itWstar_UnkMotion1_Coll */
            case 12: {  it_802946B0(&GO[0]); } break; /* it_802946B0 */
            case 13: {  it_3F14_Logic29_PickedUp(&GO[0]); } break; /* it_3F14_Logic29_PickedUp */
            case 14: {  ret = (u32) itWstar_UnkMotion3_Anim(&GO[0]); } break; /* itWstar_UnkMotion3_Anim */
            case 15: {  itWStar_Logic29_Dropped(&GO[0]); } break; /* itWStar_Logic29_Dropped */
            case 16: {  it_802947CC(&GO[0], &VA[0]); } break; /* it_802947CC */
            case 17: {  ret = (u32) itWstar_UnkMotion5_Anim(&GO[0]); } break; /* itWstar_UnkMotion5_Anim */
            case 18: {  itWstar_UnkMotion5_Phys(&GO[0]); } break; /* itWstar_UnkMotion5_Phys */
            case 19: {  ret = (u32) itWstar_UnkMotion5_Coll(&GO[0]); } break; /* itWstar_UnkMotion5_Coll */
            case 20: {  itWStar_Logic29_EnteredAir(&GO[0]); } break; /* itWStar_Logic29_EnteredAir */
            case 21: {  ret = (u32) itWstar_UnkMotion4_Anim(&GO[0]); } break; /* itWstar_UnkMotion4_Anim */
            case 22: {  itWstar_UnkMotion4_Phys(&GO[0]); } break; /* itWstar_UnkMotion4_Phys */
            case 23: {  ret = (u32) itWstar_UnkMotion4_Coll(&GO[0]); } break; /* itWstar_UnkMotion4_Coll */
            case 24: {  itWStar_Logic30_EvtUnk(&GO[0], &GO[1]); } break; /* itWStar_Logic30_EvtUnk */
            case 25: { { int j; for (j = 0; j < 6; j++) { HM((u32) it_803F61B0[j].anim_id); HM(FI((const void*) it_803F61B0[j].animated)); HM(FI((const void*) it_803F61B0[j].physics_updated)); HM(FI((const void*) it_803F61B0[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
