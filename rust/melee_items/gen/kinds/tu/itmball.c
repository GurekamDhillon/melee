/* TU: src/melee/it/kinds/itmball.c */
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


#include "itmball_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        itMBallAttributes* a = (itMBallAttributes*) SPEC[0];
        a->x0 = (float) (rnd() % 20); a->x4 = (float) (rnd() % 20);
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
        for (k = 0; k < 3; k++) { IT[k].xDD4_itemVar.mball.b0 = rnd() & 1; IT[k].xDD4_itemVar.mball.b1 = rnd() & 1; }
        Item_804A0C64.x1C = (int) (rnd() % 20);
        if (rnd() & 1) IT[0].xD44_lifeTimer = ((itMBallAttributes*) SPEC[0])->x4;
        else IT[0].xD44_lifeTimer = (float) (rnd() % 4) - 1.0f;
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    HM((u32) Item_804A0C64.x1C);
    for (k = 0; k < 3; k++) { HM(IT[k].xDD4_itemVar.mball.b0); HM(IT[k].xDD4_itemVar.mball.b1); zero(&IT[k].xDD4_itemVar.mball, sizeof IT[k].xDD4_itemVar.mball); }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 37; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  itMball_Destroyed(&GO[0]); } break; /* itMball_Destroyed */
            case 1: {  itMball_Spawned(&GO[0]); } break; /* itMball_Spawned */
            case 2: {  itMball_80297944(&GO[0]); } break; /* itMball_80297944 */
            case 3: {  ret = (u32) itMball_Motion0_Anim(&GO[0]); } break; /* itMball_Motion0_Anim */
            case 4: {  itMball_Motion0_Phys(&GO[0]); } break; /* itMball_Motion0_Phys */
            case 5: {  ret = (u32) itMball_Motion0_Coll(&GO[0]); } break; /* itMball_Motion0_Coll */
            case 6: {  itMball_802979D4(&GO[0]); } break; /* itMball_802979D4 */
            case 7: {  ret = (u32) itMball_Motion3_Anim(&GO[0]); } break; /* itMball_Motion3_Anim */
            case 8: {  itMball_Motion1_Phys(&GO[0]); } break; /* itMball_Motion1_Phys */
            case 9: {  ret = (u32) itMball_Motion1_Coll(&GO[0]); } break; /* itMball_Motion1_Coll */
            case 10: {  itMball_PickedUp(&GO[0]); } break; /* itMball_PickedUp */
            case 11: {  ret = (u32) itMball_Motion2_Anim(&GO[0]); } break; /* itMball_Motion2_Anim */
            case 12: {  itMball_Motion2_Phys(&GO[0]); } break; /* itMball_Motion2_Phys */
            case 13: {  itMball_Dropped(&GO[0]); } break; /* itMball_Dropped */
            case 14: {  itMball_Thrown(&GO[0]); } break; /* itMball_Thrown */
            case 15: {  itMball_Motion3_Phys(&GO[0]); } break; /* itMball_Motion3_Phys */
            case 16: {  ret = (u32) itMball_Motion3_Coll(&GO[0]); } break; /* itMball_Motion3_Coll */
            case 17: {  ret = (u32) itMball_DmgDealt(&GO[0]); } break; /* itMball_DmgDealt */
            case 18: {  itMball_EnteredAir(&GO[0]); } break; /* itMball_EnteredAir */
            case 19: {  ret = (u32) itMball_Motion4_Anim(&GO[0]); } break; /* itMball_Motion4_Anim */
            case 20: {  itMball_Motion4_Phys(&GO[0]); } break; /* itMball_Motion4_Phys */
            case 21: {  ret = (u32) itMball_Motion4_Coll(&GO[0]); } break; /* itMball_Motion4_Coll */
            case 22: {  itMball_80297CC4(&GO[0]); } break; /* itMball_80297CC4 */
            case 23: {  itMball_OnAccessory(&GO[0]); } break; /* itMball_OnAccessory */
            case 24: {  ret = (u32) itMball_Motion5_Anim(&GO[0]); } break; /* itMball_Motion5_Anim */
            case 25: {  itMball_Motion5_Phys(&GO[0]); } break; /* itMball_Motion5_Phys */
            case 26: {  ret = (u32) itMball_Motion5_Coll(&GO[0]); } break; /* itMball_Motion5_Coll */
            case 27: {  itMball_80297E8C(&GO[0]); } break; /* itMball_80297E8C */
            case 28: {  ret = (u32) itMball_Motion6_Anim(&GO[0]); } break; /* itMball_Motion6_Anim */
            case 29: {  itMball_Motion6_Phys(&GO[0]); } break; /* itMball_Motion6_Phys */
            case 30: {  ret = (u32) itMball_Motion6_Coll(&GO[0]); } break; /* itMball_Motion6_Coll */
            case 31: {  ret = (u32) itMball_Clanked(&GO[0]); } break; /* itMball_Clanked */
            case 32: {  ret = (u32) itMball_Reflected(&GO[0]); } break; /* itMball_Reflected */
            case 33: {  ret = (u32) itMball_HitShield(&GO[0]); } break; /* itMball_HitShield */
            case 34: {  ret = (u32) itMball_ShieldBounced(&GO[0]); } break; /* itMball_ShieldBounced */
            case 35: {  itMball_EvtUnk(&GO[0], &GO[1]); } break; /* itMball_EvtUnk */
            case 36: { { int j; for (j = 0; j < 7; j++) { HM((u32) it_803F6488[j].anim_id); HM(FI((const void*) it_803F6488[j].animated)); HM(FI((const void*) it_803F6488[j].physics_updated)); HM(FI((const void*) it_803F6488[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
