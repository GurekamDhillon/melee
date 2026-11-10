/* TU: src/melee/it/kinds/itstarrodstar.c */
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


#include "itstarrodstar_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        float* a = (float*) SPEC[0];
        a[0] = rw(); a[1] = rw(); a[2] = rw(); a[3] = rw(); a[4] = rw();
        ((int*) a)[5] = (int) (rnd() % 6); ((int*) a)[6] = (int) (rnd() % 6); a[7] = rw();
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

    IT[0].owner = (rnd() & 3) ? &GO[1] : 0;
    IT[0].xC64_reflectGObj = (rnd() & 3) ? &GO[1] : 0;
    if (rnd() & 1) IT[0].msid = (int) (rnd() % 14);
    if (rnd() & 1) IT[0].facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
    IT[0].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
    spec_fill(prof);
    for (k = 0; k < 3; k++) {
        IT[k].xDD4_itemVar.starrodstar.x0 = (int) (rnd() & 1);
        IT[k].xD44_lifeTimer = (float) (rnd() % 4);
        IT[k].xDD4_itemVar.starrodstar.x4 = rg();
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

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 13; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: { int v2 = (int) ((int) (rnd() & 1)); float v3 = (float) (rw()); it_802988E4(&GO[0], &VA[0], v2, v3); } break; /* it_802988E4 */
            case 1: {  it_802989C8(&GO[0], &GO[1]); } break; /* it_802989C8 */
            case 2: {  ret = (u32) ((itStarrodstar_UnkMotion0_Anim(&GO[0])) != 0); } break; /* itStarrodstar_UnkMotion0_Anim */
            case 3: {  itStarrodstar_UnkMotion0_Phys(&GO[0]); } break; /* itStarrodstar_UnkMotion0_Phys */
            case 4: {  ret = (u32) ((itStarrodstar_UnkMotion0_Coll(&GO[0])) != 0); } break; /* itStarrodstar_UnkMotion0_Coll */
            case 5: {  ret = (u32) ((itStarRodStar_Logic36_DmgDealt(&GO[0])) != 0); } break; /* itStarRodStar_Logic36_DmgDealt */
            case 6: {  ret = (u32) ((itStarRodStar_Logic36_Clanked(&GO[0])) != 0); } break; /* itStarRodStar_Logic36_Clanked */
            case 7: {  ret = (u32) ((itStarRodStar_Logic36_HitShield(&GO[0])) != 0); } break; /* itStarRodStar_Logic36_HitShield */
            case 8: {  ret = (u32) ((itStarRodStar_Logic36_Absorbed(&GO[0])) != 0); } break; /* itStarRodStar_Logic36_Absorbed */
            case 9: {  ret = (u32) ((itStarRodStar_Logic36_Reflected(&GO[0])) != 0); } break; /* itStarRodStar_Logic36_Reflected */
            case 10: {  ret = (u32) ((itStarRodStar_Logic36_ShieldBounced(&GO[0])) != 0); } break; /* itStarRodStar_Logic36_ShieldBounced */
            case 11: {  itStarRodStar_Logic36_EvtUnk(&GO[0], &GO[1]); } break; /* itStarRodStar_Logic36_EvtUnk */
            case 12: { { int j; for (j = 0; j < 1; j++) { HM((u32) it_803F6530[j].anim_id); HM(FI((const void*) it_803F6530[j].animated)); HM(FI((const void*) it_803F6530[j].physics_updated)); HM(FI((const void*) it_803F6530[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
