/* TU: src/melee/it/kinds/itnessyoyo.c */
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

static ItemLink LK[6];
static HSD_GObj LG[6];
static HSD_JObj LJ[6];
static HSD_JObj LB[6];
static Fighter FPL;
static HSD_GObj GF;
static FighterBone PARTS[64];
static unsigned CNT[4];

#include "itnessyoyo_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        { int j; for (j = 0; j < 3; j++) { itYoyoAttributes* a = (itYoyoAttributes*) SPEC[j];
            a->x0_CHARGE_SPAWN_POS = (int) (rnd() % 5); a->x4_UPSMASH_CHARGE_STRING_LENGTH = (int) (rnd() % 6); a->x8_DOWNSMASH_CHARGE_STRING_LENGTH = (int) (rnd() % 6); a->xC_SIZE = rw(); a->x10_UNK1 = rw(); a->x28_YOYO_PULL_STRENGTH = rw(); a->x2C_UNK3_MOD = rw(); a->x34_GRAVITY = rw(); a->x38_GRAVITY_MOD = rw(); a->x3C_YOYO_PULL_STRENGTH_2 = rw(); } }
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
        int n, j;
        zero(LK, sizeof LK); zero(LG, sizeof LG); zero(LJ, sizeof LJ); zero(LB, sizeof LB);
        zero(&FPL, sizeof FPL); zero(&GF, sizeof GF); zero(PARTS, sizeof PARTS); zero(CNT, sizeof CNT);
        for (j = 0; j < 6; j++) {
            fill_ItemLink(&LK[j]); fill_HSD_GObj(&LG[j]); fill_HSD_JObj(&LJ[j]); fill_HSD_JObj(&LB[j]);
            LJ[j].flags &= ~JOBJ_USE_QUATERNION; LB[j].flags &= ~JOBJ_USE_QUATERNION;
            LJ[j].child = (rnd() & 1) ? &LB[j] : 0;
        }
        n = 1 + (int) (rnd() % 4);
        for (j = 0; j < 6; j++) {
            LK[j].prev = (j < n && j + 1 < n) ? &LK[j + 1] : 0;
            LK[j].next = (j < n && j > 0) ? &LK[j - 1] : 0;
            LK[j].gobj = &LG[j];
            LK[j].jobj = &LB[j];
            LG[j].hsd_obj = &LJ[j];
        }
        j = (int) (rnd() % (u32) n);
        LK[j].x2C_b0 = 1;
        FPL.u.ns.yoyo_hitbox_pos.x = rw(); FPL.u.ns.yoyo_hitbox_pos.y = rw(); FPL.u.ns.yoyo_hitbox_pos.z = rw();
        FPL.u.ns.x223C = rw();
        FPL.parts = PARTS;
        PARTS[FtPart_R2ndNa].joint = &LB[0];
        GF.user_data = &FPL;
        for (k = 0; k < 3; k++) {
            i = (int) (rnd() % 3);
            IT[k].xDD4_itemVar.nessyoyo.x0 = (i == 0) ? 0x156 : (i == 1) ? 0x159 : (int) (rnd() % 0x20);
            IT[k].xDD4_itemVar.nessyoyo.x4 = rw();
            IT[k].xDD4_itemVar.nessyoyo.x8 = &LK[n - 1];
            IT[k].xDD4_itemVar.nessyoyo.xC = &LK[0];
            IT[k].xDD4_itemVar.nessyoyo.x10 = &GF;
            i = (int) (rnd() % 4);
            IT[k].xDD4_itemVar.nessyoyo.x18 = (i == 0) ? 0 : &LJ[(int) (rnd() % (u32) n)];
        }
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) {
        HM(PI(IT[k].xDD4_itemVar.nessyoyo.x8)); IT[k].xDD4_itemVar.nessyoyo.x8 = 0; HM(PI(IT[k].xDD4_itemVar.nessyoyo.xC)); IT[k].xDD4_itemVar.nessyoyo.xC = 0; HM(PI(IT[k].xDD4_itemVar.nessyoyo.x10)); IT[k].xDD4_itemVar.nessyoyo.x10 = 0; HM(PI(IT[k].xDD4_itemVar.nessyoyo.x18)); IT[k].xDD4_itemVar.nessyoyo.x18 = 0;
    }
    for (i = 0; i < 6; i++) { hash_ItemLink(&LK[i]); hash_HSD_GObj(&LG[i]); hash_HSD_JObj(&LJ[i]); hash_HSD_JObj(&LB[i]); }
    HM(B(FPL.u.ns.yoyo_hitbox_pos.x)); HM(B(FPL.u.ns.yoyo_hitbox_pos.y)); HM(B(FPL.u.ns.yoyo_hitbox_pos.z));
    HM(B(FPL.u.ns.x223C)); HM(CNT[0]); HM(CNT[1]); HM(CNT[2]);
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    preg(LK, sizeof LK); preg(LG, sizeof LG); preg(LJ, sizeof LJ); preg(LB, sizeof LB); preg(&FPL, sizeof FPL); preg(&GF, sizeof GF); preg(PARTS, sizeof PARTS);
    freg_all();
    for (op = 0; op < 24; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  it_802BE598(&GO[0]); } break; /* it_802BE598 */
            case 1: {  it_802BE5B8(&GO[0]); } break; /* it_802BE5B8 */
            case 2: { it_802BE5D8(&GO[0], rw()); } break; /* it_802BE5D8 */
            case 3: {  ret = PI(it_802BE65C(&IT[0], &JO[3])); } break; /* it_802BE65C */
            case 4: {  it_802BE958(&GO[0]); } break; /* it_802BE958 */
            case 5: { { float f = rw(); int a = (rnd() % 3 == 0) ? 0x159 : (int) (rnd() % 0x20); ret = PI(it_802BE9D8(&GF, f, &VA[0], a)); } } break; /* it_802BE9D8 */
            case 6: {  itNessyoyo_UnkMotion0_Phys(&GO[0]); } break; /* itNessyoyo_UnkMotion0_Phys */
            case 7: {  itNessyoyo_UnkMotion1_Phys(&GO[0]); } break; /* itNessyoyo_UnkMotion1_Phys */
            case 8: {  itNessyoyo_UnkMotion2_Phys(&GO[0]); } break; /* itNessyoyo_UnkMotion2_Phys */
            case 9: {  itNessyoyo_UnkMotion3_Phys(&GO[0]); } break; /* itNessyoyo_UnkMotion3_Phys */
            case 10: {  ret = (u32) itNessyoyo_UnkMotion3_Anim(&GO[0]); } break; /* itNessyoyo_UnkMotion3_Anim */
            case 11: { { int ix = (int) (rnd() % 3); int a = (int) fv(); float o = rw(); ret = (u32) it_802BF030(&LK[ix], a, o); } } break; /* it_802BF030 */
            case 12: { { int ix = (int) (rnd() % 3); float l = rw(); it_802BF180(&LK[ix], &VA[0], (itYoyoAttributes*) SPEC[0], l); } } break; /* it_802BF180 */
            case 13: { { int ix = (int) (rnd() % 3); ret = (u32) it_802BF28C(&LK[ix], &VA[0], (itYoyoAttributes*) SPEC[0], &IT[0]); } } break; /* it_802BF28C */
            case 14: { { int ix = (int) (rnd() % 3); ret = (u32) it_802BF4A0(&LK[ix], &VA[0], (itYoyoAttributes*) SPEC[0], &IT[0]); } } break; /* it_802BF4A0 */
            case 15: { { int ix = (int) (rnd() % 3); float d = rw(); ret = (u32) it_802BF800(&LK[ix], &VA[0], (itYoyoAttributes*) SPEC[0], &IT[0], d); } } break; /* it_802BF800 */
            case 16: {  it_802BF900(&IT[0]); } break; /* it_802BF900 */
            case 17: {  it_802BFAFC(&IT[0], &VA[0]); } break; /* it_802BFAFC */
            case 18: {  itNessYoyo_Logic59_PickedUp(&GO[0]); } break; /* itNessYoyo_Logic59_PickedUp */
            case 19: { float v2 = (float) (rw()); it_802BFE5C(&GO[0], &VA[0], v2); } break; /* it_802BFE5C */
            case 20: {  it_802BFEC4(&GO[0]); } break; /* it_802BFEC4 */
            case 21: {  it_802C0010(&GO[0], &VA[0]); } break; /* it_802C0010 */
            case 22: {  it_2725_Logic59_EvtUnk(&GO[0], &GO[1]); } break; /* it_2725_Logic59_EvtUnk */
            case 23: { { int j; for (j = 0; j < 4; j++) { HM((u32) it_803F7558[j].anim_id); HM(FI((const void*) it_803F7558[j].animated)); HM(FI((const void*) it_803F7558[j].physics_updated)); HM(FI((const void*) it_803F7558[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
