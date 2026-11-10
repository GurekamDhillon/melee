/* TU: src/melee/it/kinds/itpeachturnip.c */
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

static const int KK[6] = { 99, 6, 7, 12, 5, 99 };
#include "itpeachturnip_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        { int j; for (j = 0; j < 3; j++) {
            int* w = (int*) SPEC[j];
            int m;
            ((float*) SPEC[j])[0] = rw();
            w[1] = (int) (rnd() % 10) - 1;
            for (m = 0; m < 9; m++) { w[2 + 2 * m] = (int) (rnd() % 6); w[3 + 2 * m] = (int) (rnd() % 20); }
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
        IT[k].kind = KK[rnd() % 6];
        IT[k].xDD4_itemVar.peachturnip.xDD8 = (int) (rnd() % 10) - 1;
        IT[k].xDD4_itemVar.peachturnip.xDD4.x0.b0 = (int) (rnd() & 1);
        IT[k].xDD4_itemVar.peachturnip.xDDC_damage = (int) (rnd() % 30);
        IT[k].xDD4_itemVar.peachturnip.xDE0_scl = rw();
        IT[k].xDD4_itemVar.peachturnip.xDE4_owner = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xDD4_itemVar.peachturnip.xDE8_scl = rw();
        IT[k].xD44_lifeTimer = (rnd() & 1) ? 1.0f : rw();
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.peachturnip.xDE4_owner)); IT[k].xDD4_itemVar.peachturnip.xDE4_owner = 0; HM(IT[k].xDD4_itemVar.peachturnip.xDD4.x0.b0); zero(&IT[k].xDD4_itemVar.peachturnip.xDD4, sizeof IT[k].xDD4_itemVar.peachturnip.xDD4); }
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
            case 0: {  ret = (u32) ((decrease_lifetimer(&GO[0])) != 0); } break; /* decrease_lifetimer */
            case 1: {  ret = (u32) it_802BD32C(&GO[0]); } break; /* it_802BD32C */
            case 2: {  it_802BD45C(&GO[0]); } break; /* it_802BD45C */
            case 3: {  itPeachTurnip_Logic56_Destroyed(&GO[0]); } break; /* itPeachTurnip_Logic56_Destroyed */
            case 4: { Fighter_Part v2 = (Fighter_Part) ((int) fv()); ItemKind v3 = (ItemKind) (KK[rnd() % 6]); f32 v4 = (f32) (rw()); ret = PI(it_802BD4AC(&GO[0], &VA[0], v2, v3, v4)); } break; /* it_802BD4AC */
            case 5: {  itPeachTurnip_Logic56_PickedUp(&GO[0]); } break; /* itPeachTurnip_Logic56_PickedUp */
            case 6: {  ret = (u32) ((itPeachturnip_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itPeachturnip_UnkMotion4_Anim */
            case 7: {  itPeachturnip_UnkMotion4_Phys(&GO[0]); } break; /* itPeachturnip_UnkMotion4_Phys */
            case 8: {  ret = (u32) ((itPeachturnip_UnkMotion1_Anim(&GO[0])) != 0); } break; /* itPeachturnip_UnkMotion1_Anim */
            case 9: {  itPeachturnip_UnkMotion1_Phys(&GO[0]); } break; /* itPeachturnip_UnkMotion1_Phys */
            case 10: {  itPeachTurnip_Logic56_Thrown(&GO[0]); } break; /* itPeachTurnip_Logic56_Thrown */
            case 11: {  ret = (u32) ((itPeachturnip_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itPeachturnip_UnkMotion3_Anim */
            case 12: {  itPeachturnip_UnkMotion3_Phys(&GO[0]); } break; /* itPeachturnip_UnkMotion3_Phys */
            case 13: {  ret = (u32) ((itPeachturnip_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itPeachturnip_UnkMotion3_Coll */
            case 14: {  itPeachTurnip_Logic56_Dropped(&GO[0]); } break; /* itPeachTurnip_Logic56_Dropped */
            case 15: {  ret = (u32) ((itPeachTurnip_Logic56_DmgDealt(&GO[0])) != 0); } break; /* itPeachTurnip_Logic56_DmgDealt */
            case 16: {  ret = (u32) ((itPeachTurnip_Logic56_Clanked(&GO[0])) != 0); } break; /* itPeachTurnip_Logic56_Clanked */
            case 17: {  ret = (u32) ((itPeachTurnip_Logic56_Reflected(&GO[0])) != 0); } break; /* itPeachTurnip_Logic56_Reflected */
            case 18: {  ret = (u32) ((itPeachTurnip_Logic56_HitShield(&GO[0])) != 0); } break; /* itPeachTurnip_Logic56_HitShield */
            case 19: {  ret = (u32) ((itPeachTurnip_Logic56_ShieldBounced(&GO[0])) != 0); } break; /* itPeachTurnip_Logic56_ShieldBounced */
            case 20: {  itPeachTurnip_Logic56_EvtUnk(&GO[0], &GO[1]); } break; /* itPeachTurnip_Logic56_EvtUnk */
            case 21: { { int j; for (j = 0; j < 5; j++) { HM((u32) it_803F74A8[j].anim_id); HM(FI((const void*) it_803F74A8[j].animated)); HM(FI((const void*) it_803F74A8[j].physics_updated)); HM(FI((const void*) it_803F74A8[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
