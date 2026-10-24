/* TU: src/melee/it/kinds/itlinkarrow.c */
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


#include "itlinkarrow_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        { int q; for (q = 0; q < 3; q++) {
        itLinkArrowAttributes* a = (itLinkArrowAttributes*) SPEC[q];
        a->x0 = (float) (rnd() % 100); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw(); a->x14 = rw();
        a->x18 = (float) (rnd() % 100); a->x1C = rw(); a->x20 = rw();
        a->x24 = (rnd() & 1) ? (HSD_Joint*) &JO[3 + q] : 0; a->x28 = (rnd() & 1) ? (HSD_Joint*) &JO[3 + q] : 0; a->x2C = rw();
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
        IT[k].xDD4_itemVar.linkarrow.x18.x = rw(); IT[k].xDD4_itemVar.linkarrow.x18.y = rw(); IT[k].xDD4_itemVar.linkarrow.x18.z = rw();
        IT[k].xDD4_itemVar.linkarrow.x24.x = rw(); IT[k].xDD4_itemVar.linkarrow.x24.y = rw(); IT[k].xDD4_itemVar.linkarrow.x24.z = rw();
        IT[k].xDD4_itemVar.linkarrow.x30.x = rw(); IT[k].xDD4_itemVar.linkarrow.x30.y = rw(); IT[k].xDD4_itemVar.linkarrow.x30.z = rw();
        IT[k].xDD4_itemVar.linkarrow.x3C.x = rw(); IT[k].xDD4_itemVar.linkarrow.x3C.y = rw(); IT[k].xDD4_itemVar.linkarrow.x3C.z = rw();
        IT[k].xDD4_itemVar.linkarrow.x48.x = rw(); IT[k].xDD4_itemVar.linkarrow.x48.y = rw(); IT[k].xDD4_itemVar.linkarrow.x48.z = rw();
        IT[k].xDD4_itemVar.linkarrow.x54.x = rw(); IT[k].xDD4_itemVar.linkarrow.x54.y = rw(); IT[k].xDD4_itemVar.linkarrow.x54.z = rw(); IT[k].xDD4_itemVar.linkarrow.x54.w = rw();
        IT[k].xDD4_itemVar.linkarrow.x64.x = rw(); IT[k].xDD4_itemVar.linkarrow.x64.y = rw(); IT[k].xDD4_itemVar.linkarrow.x64.z = rw(); IT[k].xDD4_itemVar.linkarrow.x64.w = rw();
        IT[k].xDD4_itemVar.linkarrow.x74.x = rw(); IT[k].xDD4_itemVar.linkarrow.x74.y = rw(); IT[k].xDD4_itemVar.linkarrow.x74.z = rw(); IT[k].xDD4_itemVar.linkarrow.x74.w = rw();
        IT[k].xDD4_itemVar.linkarrow.x84.x = rw(); IT[k].xDD4_itemVar.linkarrow.x84.y = rw(); IT[k].xDD4_itemVar.linkarrow.x84.z = rw(); IT[k].xDD4_itemVar.linkarrow.x84.w = rw();
        IT[k].xDD4_itemVar.linkarrow.x94 = rw(); IT[k].xDD4_itemVar.linkarrow.x98 = rw();
        IT[k].xDD4_itemVar.linkarrow.x9C = (rnd() & 7) == 0 ? (u32) (rnd() % 12) : (u32) (rnd() % 9);
        IT[k].xDD4_itemVar.linkarrow.xA0 = (int) (rnd() % 2); IT[k].xDD4_itemVar.linkarrow.xA4 = rnd() % 10; IT[k].xDD4_itemVar.linkarrow.xA8 = rw(); IT[k].xDD4_itemVar.linkarrow.xAC = rw();
        IT[k].xDD4_itemVar.linkarrow.xB0 = (int) (rnd() % 4);
        IT[k].xDD4_itemVar.linkarrow.xB4[0] = (rnd() & 1) ? &JO[6 + 14 * k + 0] : 0; IT[k].xDD4_itemVar.linkarrow.xB4[1] = (rnd() & 1) ? &JO[6 + 14 * k + 1] : 0; IT[k].xDD4_itemVar.linkarrow.xB4[2] = 0;
        IT[k].xDD4_itemVar.linkarrow.xC0 = rw();
        IT[k].xDD4_itemVar.linkarrow.xC4 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xDD4_itemVar.linkarrow.xC8 = rw(); IT[k].xDD4_itemVar.linkarrow.xCC = rw(); IT[k].xDD4_itemVar.linkarrow.xD0 = 0; IT[k].xDD4_itemVar.linkarrow.xD4 = rw(); IT[k].xDD4_itemVar.linkarrow.xD8 = rw(); IT[k].xDD4_itemVar.linkarrow.xDC = rw();
        IT[k].xDD4_itemVar.linkarrow.xE0 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].owner = (rnd() & 3) ? IT[k].xDD4_itemVar.linkarrow.xE0 : (((rnd() & 1) ? &GO[(k + 2) % 3] : 0));
        IT[k].xDD4_itemVar.linkarrow.xE4 = (rnd() & 3) ? (int) (rnd() % 20) : -1;
        IT[k].xDD4_itemVar.linkarrow.xE8 = rw(); IT[k].xDD4_itemVar.linkarrow.xEC = rw(); IT[k].xDD4_itemVar.linkarrow.xF0 = ((rnd() & 3) == 0) ? 0x12B + (int) (rnd() % 3) : (int) (rnd() % 0x130) - 1;
        IT[k].kind = (rnd() & 1) ? (rnd() % 4 == 0 ? It_Kind_Kirby_LinkArrow : It_Kind_Link_Arrow) : (rnd() % 4 == 0 ? It_Kind_Kirby_CLinkArrow : It_Kind_CLink_Arrow);
        if ((rnd() & 7) == 0) IT[k].kind = (rnd() % 400);
        IT[k].xCF4_fighterGObjUnk = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xDAC_itcmd_var0 = (rnd() & 3) ? (int) (rnd() % 3) : 1; IT[k].xDB0_itcmd_var1 = (int) (rnd() % 3); IT[k].xDB4_itcmd_var2 = (int) (rnd() % 2);
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) {
        HM(PI(IT[k].xDD4_itemVar.linkarrow.xB4[0])); HM(PI(IT[k].xDD4_itemVar.linkarrow.xB4[1])); HM(PI(IT[k].xDD4_itemVar.linkarrow.xB4[2])); HM(PI(IT[k].xDD4_itemVar.linkarrow.xC4)); HM(PI(IT[k].xDD4_itemVar.linkarrow.xD0)); HM(PI(IT[k].xDD4_itemVar.linkarrow.xE0));
        IT[k].xDD4_itemVar.linkarrow.xB4[0] = 0; IT[k].xDD4_itemVar.linkarrow.xB4[1] = 0; IT[k].xDD4_itemVar.linkarrow.xB4[2] = 0; IT[k].xDD4_itemVar.linkarrow.xC4 = 0; IT[k].xDD4_itemVar.linkarrow.xD0 = 0; IT[k].xDD4_itemVar.linkarrow.xE0 = 0;
    }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 33; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: { int v1 = (int) ((int) (rnd() % 3)); it_802A7D8C(&GO[0], v1); } break; /* it_802A7D8C */
            case 1: {  it_802A7E40(&GO[0]); } break; /* it_802A7E40 */
            case 2: {  itLinkArrow_802A81C4(&GO[0]); } break; /* itLinkArrow_802A81C4 */
            case 3: {  it_802A8330(&GO[0]); } break; /* it_802A8330 */
            case 4: {  it_802A8398(&GO[0], &VA[0], &VA[1]); } break; /* it_802A8398 */
            case 5: { f32 v0 = (f32) (rw()); Fighter_Part v3 = (Fighter_Part) ((int) fv()); s32 v4 = (s32) ((int) (rnd() % 3)); ret = PI(it_802A83E0(v0, &GO[0], &VA[0], v3, v4)); } break; /* it_802A83E0 */
            case 6: { float v2 = rw(); float v3 = rw(); float v4 = rw(); int cond = (IT[0].xDD4_itemVar.linkarrow.xE0 != 0 && IT[0].owner == IT[0].xDD4_itemVar.linkarrow.xE0); u32 r = (u32) itLinkArrow_802A850C(&GO[0], &VA[0], &VA[1], v2, v3, v4); if (cond) ret = r; } break; /* itLinkArrow_802A850C */
            case 7: {  itLinkArrow_Logic98_Destroyed(&GO[0]); } break; /* itLinkArrow_Logic98_Destroyed */
            case 8: {  it_802A8A7C(&GO[0]); } break; /* it_802A8A7C */
            case 9: {  itLinkArrow_Logic98_PickedUp(&GO[0]); } break; /* itLinkArrow_Logic98_PickedUp */
            case 10: {  ret = (u32) itLinkarrow_UnkMotion0_Anim(&GO[0]); } break; /* itLinkarrow_UnkMotion0_Anim */
            case 11: {  itLinkarrow_UnkMotion0_Phys(&GO[0]); } break; /* itLinkarrow_UnkMotion0_Phys */
            case 12: {  ret = (u32) itLinkarrow_UnkMotion0_Coll(&GO[0]); } break; /* itLinkarrow_UnkMotion0_Coll */
            case 13: {  it_802A8C7C(&GO[0]); } break; /* it_802A8C7C */
            case 14: {  ret = (u32) itLinkarrow_UnkMotion1_Anim(&GO[0]); } break; /* itLinkarrow_UnkMotion1_Anim */
            case 15: {  itLinkarrow_UnkMotion1_Phys(&GO[0]); } break; /* itLinkarrow_UnkMotion1_Phys */
            case 16: {  ret = (u32) itLinkarrow_UnkMotion1_Coll(&GO[0]); } break; /* itLinkarrow_UnkMotion1_Coll */
            case 17: {  ret = (u32) itLinkarrow_UnkMotion2_Anim(&GO[0]); } break; /* itLinkarrow_UnkMotion2_Anim */
            case 18: {  itLinkarrow_UnkMotion2_Phys(&GO[0]); } break; /* itLinkarrow_UnkMotion2_Phys */
            case 19: {  ret = (u32) itLinkarrow_UnkMotion2_Coll(&GO[0]); } break; /* itLinkarrow_UnkMotion2_Coll */
            case 20: {  ret = (u32) itLinkarrow_UnkMotion3_Anim(&GO[0]); } break; /* itLinkarrow_UnkMotion3_Anim */
            case 21: {  itLinkarrow_UnkMotion3_Phys(&GO[0]); } break; /* itLinkarrow_UnkMotion3_Phys */
            case 22: {  ret = (u32) itLinkarrow_UnkMotion3_Coll(&GO[0]); } break; /* itLinkarrow_UnkMotion3_Coll */
            case 23: {  it_802A9458(&GO[0]); } break; /* it_802A9458 */
            case 24: {  ret = (u32) itLinkarrow_UnkMotion4_Anim(&GO[0]); } break; /* itLinkarrow_UnkMotion4_Anim */
            case 25: {  itLinkarrow_UnkMotion4_Phys(&GO[0]); } break; /* itLinkarrow_UnkMotion4_Phys */
            case 26: {  ret = (u32) itLinkarrow_UnkMotion4_Coll(&GO[0]); } break; /* itLinkarrow_UnkMotion4_Coll */
            case 27: {  ret = (u32) itLinkArrow_Logic98_DmgDealt(&GO[0]); } break; /* itLinkArrow_Logic98_DmgDealt */
            case 28: {  ret = (u32) itLinkArrow_Logic98_HitShield(&GO[0]); } break; /* itLinkArrow_Logic98_HitShield */
            case 29: {  ret = (u32) itLinkArrow_Logic98_Clanked(&GO[0]); } break; /* itLinkArrow_Logic98_Clanked */
            case 30: {  ret = (u32) itLinkArrow_Logic98_Reflected(&GO[0]); } break; /* itLinkArrow_Logic98_Reflected */
            case 31: {  itLinkArrow_Logic98_EvtUnk(&GO[0], &GO[1]); } break; /* itLinkArrow_Logic98_EvtUnk */
            case 32: { { int j; for (j = 0; j < 5; j++) { HM((u32) it_803F6A28[j].anim_id); HM(FI((const void*) it_803F6A28[j].animated)); HM(FI((const void*) it_803F6A28[j].physics_updated)); HM(FI((const void*) it_803F6A28[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
