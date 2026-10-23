/* TU: src/melee/it/kinds/itsword.c */
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


#include "itsword_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        float* a = (float*) SPEC[0];
        a[3] = rw(); a[4] = rw(); a[5] = rw(); ((int*) a)[6] = (rnd() & 1) ? 4 : (int) (rnd() % 3);
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
        IT[k].xDD4_itemVar.sword.x4 = (int) (rnd() % 6); IT[k].xDD4_itemVar.sword.x8 = (int) (rnd() % 6); IT[k].xDD4_itemVar.sword.xC = (int) (rnd() % 3) - 1;
        IT[k].xDD4_itemVar.sword.x10 = rg(); IT[k].xDD4_itemVar.sword.x14 = rg(); IT[k].xDD4_itemVar.sword.x18 = rg();
        IT[k].xDD4_itemVar.sword.x1C = (int) (rnd() % 6); IT[k].xDD4_itemVar.sword.x20 = (int) (rnd() % 6); IT[k].xDD4_itemVar.sword.x24 = (int) (rnd() % 3) - 1;
        IT[k].xDD4_itemVar.sword.x28 = rg(); IT[k].xDD4_itemVar.sword.x2C = rg(); IT[k].xDD4_itemVar.sword.x30 = rg();
        IT[k].xDD4_itemVar.sword.x34 = 4 * (int) (rnd() % 2); IT[k].xDD4_itemVar.sword.x38 = 4 * (int) (rnd() % 2); IT[k].xDD4_itemVar.sword.x3C = (int) (rnd() % 3) - 1;
        IT[k].xDD4_itemVar.sword.x40 = rg(); IT[k].xDD4_itemVar.sword.x44 = rg(); IT[k].xDD4_itemVar.sword.x48 = rg(); IT[k].xDD4_itemVar.sword.x4C = rg();
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 38; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  ret = PI(itSword_Spawn(&VA[0])); } break; /* itSword_Spawn */
            case 1: { int v1 = (int) ((int) (rnd() % 6)); it_80284E10(&GO[0], v1); } break; /* it_80284E10 */
            case 2: {  it_80284E30(&GO[0]); } break; /* it_80284E30 */
            case 3: { int v1 = (int) ((int) (rnd() % 6)); float v2 = (float) (rw()); it_80284FC4(&GO[0], v1, v2); } break; /* it_80284FC4 */
            case 4: { int v1 = (int) ((int) (rnd() % 6)); it_80285024(&GO[0], v1); } break; /* it_80285024 */
            case 5: {  it_80285084(&GO[0]); } break; /* it_80285084 */
            case 6: {  it_80285140(&GO[0]); } break; /* it_80285140 */
            case 7: {  it_802851FC(&GO[0]); } break; /* it_802851FC */
            case 8: { { float o1 = rw(); float o2 = rw(); it_802852B8(&GO[0], &o1, &o2); ret = B(o1) + 31u * B(o2); } } break; /* it_802852B8 */
            case 9: {  ret = PI(it_80285300(&GO[0])); } break; /* it_80285300 */
            case 10: {  ret = PI(it_80285314(&GO[0])); } break; /* it_80285314 */
            case 11: {  itSword_Logic12_Spawned(&GO[0]); } break; /* itSword_Logic12_Spawned */
            case 12: {  it_80285424(&GO[0]); } break; /* it_80285424 */
            case 13: {  ret = (u32) ((itSword_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itSword_UnkMotion0_Anim */
            case 14: {  itSword_UnkMotion0_Phys(&GO[0]); } break; /* itSword_UnkMotion0_Phys */
            case 15: {  ret = (u32) ((itSword_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itSword_UnkMotion0_Coll */
            case 16: {  it_802855F8(&GO[0]); } break; /* it_802855F8 */
            case 17: {  ret = (u32) ((itSword_UnkMotion3_Anim(&GO[0])) != 0); } break; /* itSword_UnkMotion3_Anim */
            case 18: {  itSword_UnkMotion3_Phys(&GO[0]); } break; /* itSword_UnkMotion3_Phys */
            case 19: {  ret = (u32) ((itSword_UnkMotion3_Coll(&GO[0])) != 0); } break; /* itSword_UnkMotion3_Coll */
            case 20: {  itSword_Logic12_PickedUp(&GO[0]); } break; /* itSword_Logic12_PickedUp */
            case 21: {  ret = (u32) ((itSword_UnkMotion2_Anim(&GO[0])) != 0); } break; /* itSword_UnkMotion2_Anim */
            case 22: {  itSword_UnkMotion2_Phys(&GO[0]); } break; /* itSword_UnkMotion2_Phys */
            case 23: {  ret = (u32) ((itSword_UnkMotion2_Coll(&GO[0])) != 0); } break; /* itSword_UnkMotion2_Coll */
            case 24: {  itSword_Logic12_Dropped(&GO[0]); } break; /* itSword_Logic12_Dropped */
            case 25: {  itSword_Logic12_Thrown(&GO[0]); } break; /* itSword_Logic12_Thrown */
            case 26: {  itSword_Logic12_EnteredAir(&GO[0]); } break; /* itSword_Logic12_EnteredAir */
            case 27: {  ret = (u32) ((itSword_UnkMotion4_Anim(&GO[0])) != 0); } break; /* itSword_UnkMotion4_Anim */
            case 28: {  itSword_UnkMotion4_Phys(&GO[0]); } break; /* itSword_UnkMotion4_Phys */
            case 29: {  ret = (u32) ((itSword_UnkMotion4_Coll(&GO[0])) != 0); } break; /* itSword_UnkMotion4_Coll */
            case 30: {  ret = (u32) ((itSword_Logic12_DmgDealt(&GO[0])) != 0); } break; /* itSword_Logic12_DmgDealt */
            case 31: {  ret = (u32) ((itSword_Logic12_Reflected(&GO[0])) != 0); } break; /* itSword_Logic12_Reflected */
            case 32: {  ret = (u32) ((itSword_Logic12_Clanked(&GO[0])) != 0); } break; /* itSword_Logic12_Clanked */
            case 33: {  ret = (u32) ((itSword_Logic12_HitShield(&GO[0])) != 0); } break; /* itSword_Logic12_HitShield */
            case 34: {  ret = (u32) ((itSword_Logic12_ShieldBounced(&GO[0])) != 0); } break; /* itSword_Logic12_ShieldBounced */
            case 35: {  itSword_Logic12_EvtUnk(&GO[0], &GO[1]); } break; /* itSword_Logic12_EvtUnk */
            case 36: { it_802852B8(&GO[0], 0, 0); } break; /* it_802852B8(NULL, NULL) */
            case 37: { { int j; for (j = 0; j < 5; j++) { HM((u32) it_803F5800[j].anim_id); HM(FI((const void*) it_803F5800[j].animated)); HM(FI((const void*) it_803F5800[j].physics_updated)); HM(FI((const void*) it_803F5800[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
