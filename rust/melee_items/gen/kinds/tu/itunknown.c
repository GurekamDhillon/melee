/* TU: src/melee/it/kinds/itunknown.c */
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
static unsigned char SPEC[3][512] __attribute__((aligned(16)));
static Vec3 VA[4];


#include "itunknown_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itUnknownAttributes* a = (itUnknownAttributes*) SPEC[0];
        a->x0.f = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw(); a->x14 = rw();
        a->x18.i = (int) (rnd() % 6); a->x1C.i = (int) (rnd() % 6); a->x20.i = (int) (rnd() % 6);
        if (rnd() & 1) { a->x1C.f = rw(); a->x20.f = rw(); a->x18.f = rw(); a->x0.i = (int) (rnd() % 100); }
        for (i = 0; i < 26; i++) a->x24[i] = (rnd() & 1) ? (HSD_Joint*) &JO[3 + (rnd() % 3)] : 0;
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
    for (k = 0; k < 3; k++) {
        IT[k].xDD4_itemVar.unknown.x60 = (rnd() & 1) ? 1.0f : rg();
        IT[k].xDD4_itemVar.unknown.x64.i = (int) (rnd() % 5) - 2;
        IT[k].xDD4_itemVar.unknown.x68.i = (int) (rnd() % 4);
        IT[k].xDD4_itemVar.unknown.x6C.x = rw(); IT[k].xDD4_itemVar.unknown.x6C.y = rw(); IT[k].xDD4_itemVar.unknown.x6C.z = rw();
        IT[k].xDD4_itemVar.unknown.x78.vec.x = rw(); IT[k].xDD4_itemVar.unknown.x78.vec.y = rw(); IT[k].xDD4_itemVar.unknown.x78.vec.z = rw();
        IT[k].xDD4_itemVar.unknown.x84 = (int) (rnd() % 5);
        IT[k].xD44_lifeTimer = (rnd() & 1) ? (float) (int) (rnd() % 4) : rg();
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 22; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_802CE710(&GO[0]); } break; /* it_802CE710 */
            case 1: {  it_802CE7CC(&GO[0]); } break; /* it_802CE7CC */
            case 2: {  it_802CE7D0(&GO[0], &GO[1]); } break; /* it_802CE7D0 */
            case 3: {  ret = (u32) itUnknown_UnkMotion0_Anim(&GO[0]); } break; /* itUnknown_UnkMotion0_Anim */
            case 4: {  itUnknown_UnkMotion0_Phys(&GO[0]); } break; /* itUnknown_UnkMotion0_Phys */
            case 5: {  ret = (u32) itUnknown_UnkMotion0_Coll(&GO[0]); } break; /* itUnknown_UnkMotion0_Coll */
            case 6: {  it_802CE8D0(&GO[0]); } break; /* it_802CE8D0 */
            case 7: {  ret = (u32) itUnknown_UnkMotion1_Anim(&GO[0]); } break; /* itUnknown_UnkMotion1_Anim */
            case 8: {  itUnknown_UnkMotion1_Phys(&GO[0]); } break; /* itUnknown_UnkMotion1_Phys */
            case 9: {  ret = (u32) itUnknown_UnkMotion1_Coll(&GO[0]); } break; /* itUnknown_UnkMotion1_Coll */
            case 10: {  it_802CEC24(&GO[0]); } break; /* it_802CEC24 */
            case 11: {  ret = (u32) itUnknown_UnkMotion2_Anim(&GO[0]); } break; /* itUnknown_UnkMotion2_Anim */
            case 12: {  itUnknown_UnkMotion2_Phys(&GO[0]); } break; /* itUnknown_UnkMotion2_Phys */
            case 13: {  ret = (u32) itUnknown_UnkMotion2_Coll(&GO[0]); } break; /* itUnknown_UnkMotion2_Coll */
            case 14: {  it_802CED54(&GO[0]); } break; /* it_802CED54 */
            case 15: {  it_2725_Logic38_Spawned(&GO[0]); } break; /* it_2725_Logic38_Spawned */
            case 16: {  itUnknown_Logic38_EvtUnk(&GO[0], &GO[1]); } break; /* itUnknown_Logic38_EvtUnk */
            case 17: {  it_802CF0D4(&GO[0]); } break; /* it_802CF0D4 */
            case 18: {  ret = (u32) it_802CF120(&GO[0]); } break; /* it_802CF120 */
            case 19: {  it_802CF154(&GO[0]); } break; /* it_802CF154 */
            case 20: {  ret = (u32) it_802CF3D8(&GO[0]); } break; /* it_802CF3D8 */
            case 21: { { int j; for (j = 0; j < 4; j++) { HM((u32) it_803F7D60[j].anim_id); HM(FI((const void*) it_803F7D60[j].animated)); HM(FI((const void*) it_803F7D60[j].physics_updated)); HM(FI((const void*) it_803F7D60[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
