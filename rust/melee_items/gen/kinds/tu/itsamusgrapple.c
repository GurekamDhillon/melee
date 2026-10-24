/* TU: src/melee/it/kinds/itsamusgrapple.c */
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

static ItemLink LK[24];
static HSD_GObj LGO[24];
static Fighter FT[2];
static FighterBone FB[256];
static ftSs_DatAttrs DA;
static ftData FD;
static itSamusGrappleAttributes AG[3];
static itSamusGrapple_HitboxData HBD;
static void* ANIMP[8];
static int LKN, LGN, NL;

#include "itsamusgrapple_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        ;
    } break;
    case 1: {
        ;
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
    {
        int j;
        zero(LK, sizeof LK); zero(LGO, sizeof LGO); zero(FT, sizeof FT); zero(FB, sizeof FB); zero(&DA, sizeof DA);
        zero(&FD, sizeof FD); zero(AG, sizeof AG); zero(&HBD, sizeof HBD); zero(ANIMP, sizeof ANIMP);
        LKN = 0; LGN = 0;
        NL = 2 + (int) (rnd() % 5);
        fill_Fighter(&FT[0]); fill_Fighter(&FT[1]); fill_ftSs_DatAttrs(&DA); fill_itSamusGrapple_HitboxData(&HBD);
        for (j = 0; j < 24; j++) { fill_ItemLink(&LK[j]); fill_HSD_GObj(&LGO[j]); }
        for (j = 0; j < 8; j++) ANIMP[j] = (rnd() & 1) ? (void*) &ANIMP[(j + 1) % 8] : 0;
        for (j = 0; j < 256; j++) FB[j].joint = &JO[j % 48];
        for (j = 0; j < 8; j++) JO[16 + j].child = &JO[32 + j];
        for (j = 0; j < NL; j++) {
            LK[j].next = j > 0 ? &LK[j - 1] : 0;
            LK[j].prev = j < NL - 1 ? &LK[j + 1] : 0;
            LK[j].gobj = &LGO[j];
            LK[j].jobj = &JO[3 + (int) (rnd() % 3)];
            LGO[j].hsd_obj = &JO[24 + j];
        }
        for (j = 0; j < 3; j++) ART[j].x4_specialAttributes = &AG[j];
        for (j = 0; j < 4; j++) FT[0].x914[j].owner = 0;
        IT[0].owner = &GO[1];
        GO[1].user_data = &FT[0];
        FT[0].gobj = &GO[1];
        FT[0].ft_data = &FD; FD.ext_attr = &DA; FT[0].dat_attrs = &DA; FT[0].parts = FB;
        DA.xA0 = (int) (rnd() % 4); DA.xB0 = (int) (rnd() % 4); DA.xC0 = (int) (rnd() % 4);
        DA.xD0 = (void*) (unsigned long long) (rnd() % 40);
        FT[0].u.ss.x223C = (prof == 1 && (rnd() & 3) == 0) ? 0 : &GO[0];
        FT[0].u.ss.x2240 = (u8) (rnd() % 6);
        {
            static const int mids[6] = { 0xD4, 0xD6, 0x165, 0x166, 0x100, 0x14 };
            FT[0].motion_id = mids[rnd() % 6];
        }
        FT[0].mv.ss.unk7.x0 = (float) (rnd() % 4);
        FT[0].mv.ss.grapple.x4 = (float) ((int) (rnd() % 6) - 2);
        FT[0].input.held_buttons[0] = rnd() & 0x4C;
        FT[0].input.pressed_buttons = (rnd() & 1) ? 0x100 : 0;
        FT[0].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
        FT[0].x34_scale.y = (float) (rnd() % 4 + 1) / 2.0f;
        HBD.create_hitbox.create_hitbox_0.id = rnd() % 4;
        {
            itSamusGrapple_ItemVars* g = &IT[0].xDD4_itemVar.samusgrapple;
            g->x0 = &LK[NL - 1]; g->x4 = &LK[0]; g->x8 = (rnd() & 7) ? &GO[1] : 0; g->xC = &JO[3];
            g->unk_10 = (rnd() & 1) ? fn_802B7E34 : 0;
            g->x14 = rnd() & 1; g->x15 = (u8) rnd(); g->x16 = (u8) (rnd() % 3);
        }
        {
            itSamusGrappleAttributes* a = &AG[0];
            a->x0 = rg(); a->x4 = rw(); a->x8 = rg(); a->xC = (int) (rnd() % 5);
            a->x10 = (float) (rnd() % 8 + 1) / 4.0f; a->x14 = (float) (rnd() % 8 + 1) / 4.0f; a->x18 = (float) (rnd() % 8 + 1) / 4.0f;
            a->x1C = (float) (rnd() % 8 + 1) / 4.0f; a->x20 = (float) (rnd() % 8 + 1) / 4.0f; a->x24 = (float) (rnd() % 8 + 1) / 4.0f;
            a->x28 = (float) (rnd() % 8 + 1) / 4.0f; a->x2C = (float) (rnd() % 8 + 1) / 4.0f; a->x30 = (float) (rnd() % 8 + 1) / 4.0f;
            a->x34 = (int) (rnd() % 6);
            a->x38 = (float) (rnd() % 30 + 1) / 4.0f; a->x3C = (float) (rnd() % 10) / 4.0f;
            a->x40 = rw(); a->x44 = rw(); a->x48 = rw(); a->x4C = (float) (rnd() % 8) / 2.0f; a->x50 = rw(); a->x54 = (float) (rnd() % 8) / 2.0f; a->x58 = (float) (rnd() % 8) / 4.0f;
            a->x5C = (float) (rnd() % 4 + 1) / 2.0f; a->x60 = (float) (rnd() % 4 + 1) / 2.0f;
            a->x64 = (void*) &JO[40]; a->x68 = (void*) &JO[41]; a->x6C = (void*) &JO[42]; a->x70 = (void*) &JO[43];
        a->x74 = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->x78 = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->x7C = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->x80 = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->x84 = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->x88 = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->x8C = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->x90 = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->x94 = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->x98 = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->x9C = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->xA0 = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->xA4 = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->xA8 = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        a->xAC = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;
        }
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    {
        int j;
        HM(PI(FT[0].u.ss.x223C)); HM(FT[0].u.ss.x2240); zero(&FT[0].u, sizeof FT[0].u);
        for (j = 0; j < 4; j++) { HM(FT[0].x914[j].hit_grabbed_victim_only); FT[0].x914[j].owner = 0; }
        for (j = 0; j < 2; j++) hash_Fighter(&FT[j]);
        for (j = 0; j < 24; j++) { hash_ItemLink(&LK[j]); hash_HSD_GObj(&LGO[j]); }
        for (j = 0; j < 256; j++) hash_FighterBone(&FB[j]);
        hash_ftSs_DatAttrs(&DA); hash_itSamusGrappleAttributes(&AG[0]); hash_itSamusGrapple_HitboxData(&HBD);
        for (j = 0; j < 8; j++) HM(PI(ANIMP[j]));
        HM(LKN); HM(LGN);
        {
            itSamusGrapple_ItemVars* g = &IT[0].xDD4_itemVar.samusgrapple;
            HM(PI(g->x0)); HM(PI(g->x4)); HM(PI(g->x8)); HM(PI(g->xC)); HM(FI((const void*) g->unk_10)); HM(g->x14); HM(g->x15); HM(g->x16);
        }
        for (j = 0; j < 3; j++) zero(&IT[j].xDD4_itemVar, sizeof IT[j].xDD4_itemVar);
    }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    preg(LK, sizeof LK); preg(LGO, sizeof LGO); preg(FT, sizeof FT); preg(FB, sizeof FB); preg(&DA, sizeof DA);
    preg(&FD, sizeof FD); preg(AG, sizeof AG); preg(&HBD, sizeof HBD); preg(ANIMP, sizeof ANIMP);
    freg_all();
    for (op = 0; op < 49; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  itSamusGrapple_Logic53_Spawned(&GO[0]); } break; /* itSamusGrapple_Logic53_Spawned */
            case 1: { it_802B7160(&GO[1], &HBD); } break; /* it_802B7160 */
            case 2: { it_802B743C(&LGO[0], &IT[0], (int) (fv() % 3)); } break; /* it_802B743C */
            case 3: { s32 v2 = (s32) ((int) (rnd() % 2)); f32 v3 = (f32) ((float) (rnd() % 4 + 1) / 2.0f); ret = PI(it_802B75FC(&IT[0], &JO[3], v2, v3)); } break; /* it_802B75FC */
            case 4: {  it_802B7B84(&GO[0]); } break; /* it_802B7B84 */
            case 5: { float v0 = rw(); ret = PI(it_802B7C18(&GO[1], &VA[0], v0)); } break; /* it_802B7C18 */
            case 6: {  fn_802B7E34(&GO[0]); } break; /* fn_802B7E34 */
            case 7: {  itSamusgrapple_UnkMotion0_Phys(&GO[0]); } break; /* itSamusgrapple_UnkMotion0_Phys */
            case 8: {  fn_802B805C(&GO[0]); } break; /* fn_802B805C */
            case 9: {  itSamusgrapple_UnkMotion1_Phys(&GO[0]); } break; /* itSamusgrapple_UnkMotion1_Phys */
            case 10: {  fn_802B8384(&GO[0]); } break; /* fn_802B8384 */
            case 11: {  itSamusgrapple_UnkMotion2_Phys(&GO[0]); } break; /* itSamusgrapple_UnkMotion2_Phys */
            case 12: {  fn_802B8524(&GO[0]); } break; /* fn_802B8524 */
            case 13: {  itSamusgrapple_UnkMotion3_Phys(&GO[0]); } break; /* itSamusgrapple_UnkMotion3_Phys */
            case 14: {  fn_802B8684(&GO[0]); } break; /* fn_802B8684 */
            case 15: {  itSamusgrapple_UnkMotion4_Phys(&GO[0]); } break; /* itSamusgrapple_UnkMotion4_Phys */
            case 16: {  fn_802B8814(&GO[0]); } break; /* fn_802B8814 */
            case 17: {  itSamusgrapple_UnkMotion5_Phys(&GO[0]); } break; /* itSamusgrapple_UnkMotion5_Phys */
            case 18: {  fn_802B895C(&GO[0]); } break; /* fn_802B895C */
            case 19: {  itSamusgrapple_UnkMotion6_Phys(&GO[0]); } break; /* itSamusgrapple_UnkMotion6_Phys */
            case 20: {  fn_802B8B54(&GO[0]); } break; /* fn_802B8B54 */
            case 21: {  itSamusgrapple_UnkMotion7_Phys(&GO[0]); } break; /* itSamusgrapple_UnkMotion7_Phys */
            case 22: {  fn_802B8D38(&GO[0]); } break; /* fn_802B8D38 */
            case 23: {  itSamusgrapple_UnkMotion8_Phys(&GO[0]); } break; /* itSamusgrapple_UnkMotion8_Phys */
            case 24: { ItemLink* v0 = &LK[rnd() % NL]; f32 v3 = (f32) ((float) (rnd() % 40) / 4.0f); it_802B900C(v0, &VA[0], &AG[0], v3); } break; /* it_802B900C */
            case 25: { ItemLink* v0 = &LK[rnd() % NL]; f32 v3 = (f32) ((float) (rnd() % 40) / 4.0f); it_802B91C4(v0, &VA[0], &AG[0], v3); } break; /* it_802B91C4 */
            case 26: { ItemLink* v0 = &LK[rnd() % NL]; ret = (u32) it_802B9328(v0, &VA[0], &AG[0], &FT[0]); } break; /* it_802B9328 */
            case 27: { ItemLink* v0 = &LK[rnd() % NL]; ret = (u32) it_802B99A0(v0, &VA[0], &AG[0], &FT[0]); } break; /* it_802B99A0 */
            case 28: { ItemLink* v0 = &LK[rnd() % NL]; it_802B9CE8(v0, &VA[0], &AG[0], &FT[0]); } break; /* it_802B9CE8 */
            case 29: { ItemLink* v0 = &LK[rnd() % NL]; ret = (u32) it_802B9FD4(v0, &VA[0], &AG[0]); } break; /* it_802B9FD4 */
            case 30: { ItemLink* v0 = &LK[rnd() % NL]; f32 v3 = (f32) ((float) (rnd() % 40) / 4.0f); ret = (u32) it_802BA194(v0, &VA[0], &AG[0], v3); } break; /* it_802BA194 */
            case 31: { ItemLink* v0 = &LK[rnd() % NL]; f32 v3 = (f32) ((float) (rnd() % 40) / 4.0f); ret = (u32) it_802BA2D8(v0, &VA[0], &AG[0], v3); } break; /* it_802BA2D8 */
            case 32: { f32 v4 = (f32) ((float) (rnd() % 40) / 4.0f); ret = (u32) it_802BA3BC(&LK[0], &LK[NL - 1], &VA[0], &AG[0], v4); } break; /* it_802BA3BC */
            case 33: {  it_802BA5DC(&LK[0], &LK[NL - 1], &VA[0], &AG[0]); } break; /* it_802BA5DC */
            case 34: { ItemLink* v0 = &LK[rnd() % NL]; ret = (u32) it_802BA760(v0, &VA[0], &AG[0], &FT[0]); } break; /* it_802BA760 */
            case 35: {  itSamusGrapple_Logic53_PickedUp(&GO[0]); } break; /* itSamusGrapple_Logic53_PickedUp */
            case 36: {  it_802BA9B8(&GO[0]); } break; /* it_802BA9B8 */
            case 37: {  it_802BAA08(&GO[0]); } break; /* it_802BAA08 */
            case 38: {  it_802BAA58(&GO[0]); } break; /* it_802BAA58 */
            case 39: {  it_802BAA94(&GO[0]); } break; /* it_802BAA94 */
            case 40: {  it_802BAAE4(&GO[0], &VA[0]); } break; /* it_802BAAE4 */
            case 41: {  it_802BAB40(&GO[0]); } break; /* it_802BAB40 */
            case 42: {  it_802BAB7C(&GO[0]); } break; /* it_802BAB7C */
            case 43: {  it_802BABB8(&GO[0]); } break; /* it_802BABB8 */
            case 44: { it_802BAC3C(&GO[1]); } break; /* it_802BAC3C */
            case 45: { it_802BAC80(&GO[1]); } break; /* it_802BAC80 */
            case 46: { it_802BACC4(&GO[1]); } break; /* it_802BACC4 */
            case 47: {  itSamusGrapple_Logic53_EvtUnk(&GO[0], &GO[1]); } break; /* itSamusGrapple_Logic53_EvtUnk */
            case 48: { { int j; for (j = 0; j < 9; j++) { HM((u32) it_803F73A8[j].anim_id); HM(FI((const void*) it_803F73A8[j].animated)); HM(FI((const void*) it_803F73A8[j].physics_updated)); HM(FI((const void*) it_803F73A8[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
