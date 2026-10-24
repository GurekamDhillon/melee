/* TU: src/melee/it/kinds/itpikachuthunder.c */
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


#include "itpikachuthunder_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        { int j; for (j = 0; j < 3; j++) {
            float* a = (float*) SPEC[j];
            a[0] = rw(); a[1] = (rnd() & 1) ? (float) (1 + rnd() % 40) : rw(); a[2] = rw();
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
        IT[k].xDD4_itemVar.pikachuthunder.x0 = (int) (rnd() % 3);
        IT[k].xDD4_itemVar.pikachuthunder.x4 = (int) (rnd() & 1);
        IT[k].xDD4_itemVar.pikachuthunder.x8 = (int) (rnd() % 4) - 1;
        IT[k].xDD4_itemVar.pikachuthunder.xC = (rnd() & 1) ? (float) (rnd() % 40) : rw();
        IT[k].xDD4_itemVar.pikachuthunder.x10 = rw();
        IT[k].xDD4_itemVar.pikachuthunder.x14 = (rnd() & 1) ? 3.0f : rw();
        IT[k].xDD4_itemVar.pikachuthunder.x18 = rw();
        IT[k].xDD4_itemVar.pikachuthunder.x1C.x = rw();
        IT[k].xDD4_itemVar.pikachuthunder.x1C.y = (rnd() & 1) ? -(float) (rnd() % 8) : rw();
        IT[k].xDD4_itemVar.pikachuthunder.x1C.z = rw();
        IT[k].xDD4_itemVar.pikachuthunder.x28.x = rw(); IT[k].xDD4_itemVar.pikachuthunder.x28.y = rw(); IT[k].xDD4_itemVar.pikachuthunder.x28.z = rw();
        IT[k].xDD4_itemVar.pikachuthunder.x34 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xDD4_itemVar.pikachuthunder.x38 = (rnd() & 1) ? &GO[(k + 2) % 3] : 0;
        IT[k].xD44_lifeTimer = (rnd() & 1) ? 1.0f : rw();
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.pikachuthunder.x34)); HM(PI(IT[k].xDD4_itemVar.pikachuthunder.x38)); IT[k].xDD4_itemVar.pikachuthunder.x34 = 0; IT[k].xDD4_itemVar.pikachuthunder.x38 = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 19; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  ret = (u32) it_802B1DEC(&GO[0]); } break; /* it_802B1DEC */
            case 1: { s32 v3 = (s32) ((int) (rnd() % 4)); s32 v4 = (s32) ((int) (rnd() % 4)); s32 v5 = (s32) ((int) fv()); ret = PI(it_802B1DF8(&GO[0], &VA[0], &VA[1], v3, v4, v5)); } break; /* it_802B1DF8 */
            case 2: {  it_802B1FC8(&GO[0]); } break; /* it_802B1FC8 */
            case 3: {  it_802B1FE8(&GO[0], &VA[0]); } break; /* it_802B1FE8 */
            case 4: {  it_2725_Logic39_Destroyed(&GO[0]); } break; /* it_2725_Logic39_Destroyed */
            case 5: {  it_802B2080(&GO[0]); } break; /* it_802B2080 */
            case 6: {  ret = (u32) itPikachuthunder_UnkMotion0_Anim(&GO[0]); } break; /* itPikachuthunder_UnkMotion0_Anim */
            case 7: {  it_802B211C(&GO[0]); } break; /* it_802B211C */
            case 8: {  ret = (u32) itPikachuthunder_UnkMotion1_Anim(&GO[0]); } break; /* itPikachuthunder_UnkMotion1_Anim */
            case 9: {  ret = (u32) itPikachuthunder_UnkMotion1_Coll(&GO[0]); } break; /* itPikachuthunder_UnkMotion1_Coll */
            case 10: {  it_802B22B8(&GO[0]); } break; /* it_802B22B8 */
            case 11: { f32 v0 = (f32) (rw()); f32 v1 = (f32) (rw()); ret = B(pika_scale(v0, v1)); } break; /* pika_scale */
            case 12: {  ret = (u32) itPikachuthunder_UnkMotion2_Anim(&GO[0]); } break; /* itPikachuthunder_UnkMotion2_Anim */
            case 13: {  ret = (u32) itPikachuThunder_Logic39_DmgDealt(&GO[0]); } break; /* itPikachuThunder_Logic39_DmgDealt */
            case 14: {  ret = (u32) itPikachuThunder_Logic39_HitShield(&GO[0]); } break; /* itPikachuThunder_Logic39_HitShield */
            case 15: {  ret = (u32) itPikachuThunder_Logic39_Clanked(&GO[0]); } break; /* itPikachuThunder_Logic39_Clanked */
            case 16: {  ret = (u32) itPikachuThunder_Logic39_Absorbed(&GO[0]); } break; /* itPikachuThunder_Logic39_Absorbed */
            case 17: {  itPikachuThunder_Logic39_EvtUnk(&GO[0], &GO[1]); } break; /* itPikachuThunder_Logic39_EvtUnk */
            case 18: { { int j; for (j = 0; j < 3; j++) { HM((u32) it_803F70C8[j].anim_id); HM(FI((const void*) it_803F70C8[j].animated)); HM(FI((const void*) it_803F70C8[j].physics_updated)); HM(FI((const void*) it_803F70C8[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
