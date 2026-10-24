/* TU: src/melee/it/kinds/itseakchain.c */
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

static ItemLink LK[16];
static HSD_GObj LGO[16];
static HSD_JObj LJO[32];
static Fighter FGT;
static FighterBone FBN[32];
static int LKC, LGC;
static unsigned NL;

#include "itseakchain_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itSeakChain_Attrs* a = (itSeakChain_Attrs*) SPEC[0];
        a->x0 = 3 + (int) (rnd() % 12); a->x4 = (float) (rnd() % 8) + ((rnd() & 1) ? 0.5f : 0.0f);
        a->x10 = rw(); a->x14 = rw(); a->x18 = rw(); a->x1C = rw(); a->x20 = rw(); a->x24 = rw(); a->x28 = rw(); a->x2C = rw();
        a->x30 = rw(); a->x34 = rw(); a->x38 = rw(); a->x3C = rw(); a->x40 = rw(); a->x44 = rw(); a->x48 = rw();
        a->x54 = rw(); a->x58 = rw(); a->x5C = rw(); a->x60 = rw();
        a->x64_joint = (rnd() & 1) ? (HSD_Joint*) &JO[0] : 0; a->x68_joint = (rnd() & 1) ? (HSD_Joint*) &JO[1] : 0;
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
    NL = 3 + rnd() % 5;
    LKC = 0; LGC = 0;
    for (i = 0; i < 16; i++) { fill_ItemLink(&LK[i]); fill_HSD_GObj(&LGO[i]); }
    for (i = 0; i < 32; i++) { fill_HSD_JObj(&LJO[i]); LJO[i].flags &= ~JOBJ_USE_QUATERNION; }
    zero(&FGT, sizeof FGT); zero(FBN, sizeof FBN);
    FGT.u.sk.lstick_delta.x = rw(); FGT.u.sk.lstick_delta.y = rw(); FGT.u.sk.lstick_delta.z = rw();
    FGT.mv.co.common.x1C = (rnd() & 1) ? 0.0f : rw();
    FGT.parts = FBN; FBN[26].joint = &LJO[16];
    GO[2].user_data = &FGT;
    for (i = 0; i < 16; i++) {
        LK[i].prev = (i > 0 && (unsigned) i < NL) ? &LK[i - 1] : 0;
        LK[i].next = ((unsigned) i + 1 < NL) ? &LK[i + 1] : 0;
        LK[i].gobj = &LGO[i]; LGO[i].hsd_obj = &LJO[i]; LK[i].jobj = &LJO[16 + i];
        LK[i].x2C_b0 = (i == 0) ? 1 : (rnd() & 1);
        LK[i].x2C_b1 = rnd() & 1; LK[i].x2C_b2 = rnd() & 1;
    }
    for (k = 0; k < 3; k++) {
        int j;
        IT[k].xDD4_itemVar.seakchain.x0 = &LK[0]; IT[k].xDD4_itemVar.seakchain.x4 = &LK[NL - 1]; IT[k].xDD4_itemVar.seakchain.parent_gobj = &GO[2];
        IT[k].xDD4_itemVar.seakchain.x10 = (int) (rnd() & 1); IT[k].xDD4_itemVar.seakchain.x14 = (int) (rnd() & 1); IT[k].xDD4_itemVar.seakchain.x18 = (int) (rnd() & 1);
        for (j = 0; j < 15; j++) { IT[k].xDD4_itemVar.seakchain.history[j].x = rw(); IT[k].xDD4_itemVar.seakchain.history[j].y = rw(); IT[k].xDD4_itemVar.seakchain.history[j].z = rw(); }
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) {
        HM(PI(IT[k].xDD4_itemVar.seakchain.x0)); HM(PI(IT[k].xDD4_itemVar.seakchain.x4)); HM(PI(IT[k].xDD4_itemVar.seakchain.parent_gobj));
        IT[k].xDD4_itemVar.seakchain.x0 = 0; IT[k].xDD4_itemVar.seakchain.x4 = 0; IT[k].xDD4_itemVar.seakchain.parent_gobj = 0;
    }
    for (i = 0; i < 16; i++) { hash_ItemLink(&LK[i]); hash_HSD_GObj(&LGO[i]); }
    for (i = 0; i < 32; i++) hash_HSD_JObj(&LJO[i]);
    HM(B(FGT.u.sk.lstick_delta.x)); HM(B(FGT.u.sk.lstick_delta.y)); HM(B(FGT.u.sk.lstick_delta.z)); HM(B(FGT.mv.co.common.x1C));
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    preg(LK, sizeof LK); preg(LGO, sizeof LGO); preg(LJO, sizeof LJO); preg(&FGT, sizeof FGT); preg(FBN, sizeof FBN);
    freg_all();
    for (op = 0; op < 30; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_802BAEEC(&GO[0]); } break; /* it_802BAEEC */
            case 1: {  it_802BAF0C(&GO[0]); } break; /* it_802BAF0C */
            case 2: {  it_802BAF2C(&IT[0], &JO[3]); } break; /* it_802BAF2C */
            case 3: {  inlineA0(&GO[0]); } break; /* inlineA0 */
            case 4: {  it_802BB20C(&GO[0]); } break; /* it_802BB20C */
            case 5: { { float f = rw(); ret = PI(itSeakChain_Spawn(&GO[2], &VA[0], f)); } } break; /* itSeakChain_Spawn */
            case 6: {  fn_802BB428(&GO[0]); } break; /* fn_802BB428 */
            case 7: {  fn_802BB44C(&GO[0]); } break; /* fn_802BB44C */
            case 8: {  fn_802BB574(&GO[0]); } break; /* fn_802BB574 */
            case 9: {  fn_802BB694(&GO[0]); } break; /* fn_802BB694 */
            case 10: {  fn_802BB784(&GO[0]); } break; /* fn_802BB784 */
            case 11: {  ret = (u32) notInSpecialS(&GO[0]); } break; /* notInSpecialS */
            case 12: {  ret = (u32) itSeakchain_UnkMotion4_Anim(&GO[0]); } break; /* itSeakchain_UnkMotion4_Anim */
            case 13: { int v1 = (int) ((int) fv()); float v2 = (float) (rw()); ret = (u32) it_802BB938(&LK[rnd() % NL], v1, v2); } break; /* it_802BB938 */
            case 14: { s32 v1 = (s32) ((int) fv()); f32 v2 = (f32) (rw()); ret = (u32) it_802BBAEC(&LK[rnd() % NL], v1, v2); } break; /* it_802BBAEC */
            case 15: { float v3 = (float) (rw()); it_802BBB0C(&LK[rnd() % NL], &VA[0], (itSeakChain_Attrs*) SPEC[0], v3); } break; /* it_802BBB0C */
            case 16: { f32 v3 = (f32) (rw()); it_802BBC38(&LK[rnd() % NL], &VA[0], (itSeakChain_Attrs*) SPEC[0], v3); } break; /* it_802BBC38 */
            case 17: {  ret = (u32) it_802BBD64(&LK[rnd() % NL], &VA[0], (itSeakChain_Attrs*) SPEC[0]); } break; /* it_802BBD64 */
            case 18: {  ret = (u32) it_802BBED0(&LK[rnd() % NL], &VA[0], (itSeakChain_Attrs*) SPEC[0]); } break; /* it_802BBED0 */
            case 19: {  it_802BC080(&LK[rnd() % NL], &VA[0], &IT[0]); } break; /* it_802BC080 */
            case 20: { f32 v3 = (f32) (rw()); ret = (u32) it_802BC94C(&LK[rnd() % NL], &VA[0], (itSeakChain_Attrs*) SPEC[0], v3); } break; /* it_802BC94C */
            case 21: {  it_802BCA30(&IT[0]); } break; /* it_802BCA30 */
            case 22: {  it_802BCB88(&IT[0], &VA[0]); } break; /* it_802BCB88 */
            case 23: {  it_2725_Logic54_PickedUp(&GO[0]); } break; /* it_2725_Logic54_PickedUp */
            case 24: {  it_802BCED4(&GO[0]); } break; /* it_802BCED4 */
            case 25: {  it_802BCF2C(&GO[0]); } break; /* it_802BCF2C */
            case 26: {  it_802BCF84(&GO[0]); } break; /* it_802BCF84 */
            case 27: {  it_802BCFC4(&GO[0], &VA[0]); } break; /* it_802BCFC4 */
            case 28: { itSeakChain_Logic54_EvtUnk(&GO[0], (rnd() & 1) ? &GO[2] : &GO[1]); } break; /* itSeakChain_Logic54_EvtUnk */
            case 29: { { int j; for (j = 0; j < 5; j++) { HM((u32) it_803F7438[j].anim_id); HM(FI((const void*) it_803F7438[j].animated)); HM(FI((const void*) it_803F7438[j].physics_updated)); HM(FI((const void*) it_803F7438[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
