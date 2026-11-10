/* TU: src/melee/it/kinds/itnesspkflash.c */
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


#include "itnesspkflash_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {
        { int j; for (j = 0; j < 3; j++) { itFlashAttributes* a = (itFlashAttributes*) SPEC[j];
            int q; float* f = (float*) a; for (q = 0; q < 11; q++) f[q] = rw(); a->x28_FLASH_EXPLOSION_DELAY = (float) (rnd() % 8); a->x4_FLASH_HITBOX_SIZE_MUL = (float) (rnd() % 6); } }
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
        i = (int) (rnd() % 4);
        IT[k].xDD4_itemVar.pkflush.xDE0_PKFlash_Owner = (i == 0) ? 0 : (i == 1) ? IT[k].owner : &GO[(k + i) % 3];
        IT[k].xDD4_itemVar.pkflush.xDD8_PKFlash = rw();
        IT[k].xDD4_itemVar.pkflush.xDDC_PKFlash = (int) (rnd() & 1);
        i = (int) (rnd() % 3);
        IT[k].kind = (i == 0) ? It_Kind_Ness_PKFlush : (i == 1) ? It_Kind_Kirby_NessPKFlush : It_Kind_Ness_Bat;
        IT[k].msid = (int) (rnd() % 4);
        IT[k].xDAC_itcmd_var0 = (int) (rnd() % 8);
        IT[k].x40_vel.x = rg(); IT[k].x40_vel.y = rg();
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.pkflush.xDE0_PKFlash_Owner)); IT[k].xDD4_itemVar.pkflush.xDE0_PKFlash_Owner = 0; }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    freg_all();
    for (op = 0; op < 21; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: {  ret = PI(it_802AA7E4(&GO[0])); } break; /* it_802AA7E4 */
            case 1: {  ret = (u32) it_802AA7F0(&GO[0]); } break; /* it_802AA7F0 */
            case 2: { ret = (u32) it_802AA810(&GO[0], &IT[0].x378_itemColl); } break; /* it_802AA810 */
            case 3: { ItemKind v2 = (ItemKind) ((int) (rnd() % 150)); f32 v3 = (f32) (rw()); ret = PI(it_802AA8C0(&GO[0], &VA[0], v2, v3)); } break; /* it_802AA8C0 */
            case 4: {  it_2725_Logic102_Destroyed(&GO[0]); } break; /* it_2725_Logic102_Destroyed */
            case 5: {  it_802AAA50(&GO[0]); } break; /* it_802AAA50 */
            case 6: {  it_802AAA80(&GO[0]); } break; /* it_802AAA80 */
            case 7: {  ret = (u32) itNesspkflash_UnkMotion0_Anim(&GO[0]); } break; /* itNesspkflash_UnkMotion0_Anim */
            case 8: {  ret = (u32) itNesspkflash_UnkMotion1_Anim(&GO[0]); } break; /* itNesspkflash_UnkMotion1_Anim */
            case 9: {  ret = (u32) itNesspkflash_UnkMotion2_Anim(&GO[0]); } break; /* itNesspkflash_UnkMotion2_Anim */
            case 10: {  itNesspkflash_UnkMotion0_Phys(&GO[0]); } break; /* itNesspkflash_UnkMotion0_Phys */
            case 11: {  itNesspkflash_UnkMotion1_Phys(&GO[0]); } break; /* itNesspkflash_UnkMotion1_Phys */
            case 12: {  itNesspkflash_UnkMotion2_Phys(&GO[0]); } break; /* itNesspkflash_UnkMotion2_Phys */
            case 13: {  ret = (u32) itNesspkflash_UnkMotion0_Coll(&GO[0]); } break; /* itNesspkflash_UnkMotion0_Coll */
            case 14: {  ret = (u32) itNesspkflash_UnkMotion1_Coll(&GO[0]); } break; /* itNesspkflash_UnkMotion1_Coll */
            case 15: {  ret = (u32) itNesspkflash_UnkMotion2_Coll(&GO[0]); } break; /* itNesspkflash_UnkMotion2_Coll */
            case 16: {  ret = (u32) itNesspkflash_Logic102_Reflected(&GO[0]); } break; /* itNesspkflash_Logic102_Reflected */
            case 17: {  ret = (u32) itNesspkflash_Logic102_Clanked(&GO[0]); } break; /* itNesspkflash_Logic102_Clanked */
            case 18: {  ret = (u32) itNesspkflash_Logic102_Absorbed(&GO[0]); } break; /* itNesspkflash_Logic102_Absorbed */
            case 19: {  itNesspkflash_Logic102_EvtUnk(&GO[0], &GO[1]); } break; /* itNesspkflash_Logic102_EvtUnk */
            case 20: { { int j; for (j = 0; j < 3; j++) { HM((u32) it_803F6B70[j].anim_id); HM(FI((const void*) it_803F6B70[j].animated)); HM(FI((const void*) it_803F6B70[j].physics_updated)); HM(FI((const void*) it_803F6B70[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
