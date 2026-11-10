/* TU: src/melee/it/kinds/itkinoko.c (written by gen/mtu/mk_itk.py: do not edit) */
#include TU_SRC
#include "mtu_common.h"
static int VIEWPUN;
#define FLG(f) ((u32) ((f).x0.b0 | (f).x0.b1 << 1 | (f).x0.b2 << 2 | (f).x0.b3 << 3 | (f).x0.b4 << 4 | (f).x0.b5 << 5 | (f).x0.b6 << 6 | (f).x0.b7 << 7))
static HSD_GObj G, REFG, OWNG, SG;
static Item ITEM, SITEM;
static HSD_JObj JT, JC, SJT, BONES[24];
static DynamicBoneTable DBT, SDBT;
static Article ART, SART;
static KinokoAttrs ATTR, SATTR;
static ItemAttr IATTR, SIATTR;
static Vec3 VP;
static float FP1;
typedef struct KinokoAnim { HSD_AnimJoint* joint; } KinokoAnim;
static KinokoAnim KANIM[8];
static Article* KART[0x9F];
static HSD_AnimJoint AJ[8];
#include "itkinoko_gen.h"
static void setup(void)
{
    int i;
    zero(&G, sizeof G); zero(&REFG, sizeof REFG); zero(&OWNG, sizeof OWNG); zero(&SG, sizeof SG);
    zero(&ITEM, sizeof ITEM); zero(&SITEM, sizeof SITEM); zero(&JT, sizeof JT); zero(&JC, sizeof JC); zero(&SJT, sizeof SJT);
    zero(BONES, sizeof BONES); zero(&DBT, sizeof DBT); zero(&SDBT, sizeof SDBT); zero(&ART, sizeof ART); zero(&SART, sizeof SART);
    zero(&IATTR, sizeof IATTR); zero(&SIATTR, sizeof SIATTR); VIEWPUN = 0;
    zero(&ATTR, sizeof ATTR); zero(&SATTR, sizeof SATTR);
    fill_Item(&ITEM); fill_Item(&SITEM);
    fill_HSD_JObj(&JT); fill_HSD_JObj(&JC); fill_HSD_JObj(&SJT);
    fill_ItemAttr(&IATTR); fill_ItemAttr(&SIATTR);
    fill_KinokoAttrs(&ATTR); fill_KinokoAttrs(&SATTR);
    JT.flags &= ~0x20000u; JC.flags &= ~0x20000u; SJT.flags &= ~0x20000u;
    G.user_data = &ITEM; G.hsd_obj = &JT; SG.user_data = &SITEM; SG.hsd_obj = &SJT;
    JT.child = (rnd() & 1) ? &JC : 0;
    ITEM.xC4_article_data = &ART; SITEM.xC4_article_data = &SART;
    ART.x4_specialAttributes = (union ItemSpecialAttributes*) &ATTR; SART.x4_specialAttributes = (union ItemSpecialAttributes*) &SATTR;
    ITEM.xCC_item_attr = &IATTR; SITEM.xCC_item_attr = &SIATTR;
    ITEM.xBBC_dynamicBoneTable = &DBT; SITEM.xBBC_dynamicBoneTable = &SDBT;
    for (i = 0; i < 100; i++) { DBT.bones[i] = &BONES[i % 24]; SDBT.bones[i] = &BONES[(i + 5) % 24]; }
    ITEM.owner = (rnd() & 1) ? &OWNG : 0; SITEM.owner = (rnd() & 1) ? &OWNG : 0;
    for (i = 0; i < 8; i++) { KANIM[i].joint = &AJ[i]; }
    zero(KART, sizeof KART); KART[It_Kind_Kinoko] = &SART; SART.x4_specialAttributes = (union ItemSpecialAttributes*) KANIM; it_804D6D24 = KART;
    ART.x4_specialAttributes = (union ItemSpecialAttributes*) &ATTR;
}

static void hash_all(void)
{
    hash_Item(&ITEM); hash_Item(&SITEM); hash_HSD_JObj(&JT); hash_HSD_JObj(&JC); hash_HSD_JObj(&SJT);
    hash_ItemAttr(&IATTR);
    hash_KinokoAttrs(&ATTR); hash_KinokoAttrs(&SATTR);
    HM(B(VP.x)); HM(B(VP.y)); HM(B(VP.z)); HM(B(FP1));
}

int main(void)
{
    int op, k;
    preg(&G, sizeof G); preg(&REFG, sizeof REFG); preg(&OWNG, sizeof OWNG); preg(&SG, sizeof SG);
    preg(&ITEM, sizeof ITEM); preg(&SITEM, sizeof SITEM); preg(&JT, sizeof JT); preg(&JC, sizeof JC); preg(&SJT, sizeof SJT);
    preg(BONES, sizeof BONES); preg(&DBT, sizeof DBT); preg(&SDBT, sizeof SDBT); preg(&ART, sizeof ART); preg(&SART, sizeof SART);
    preg(&IATTR, sizeof IATTR); preg(&SIATTR, sizeof SIATTR); preg(&VP, sizeof VP); preg(&FP1, sizeof FP1);
    preg(&ATTR, sizeof ATTR); preg(&SATTR, sizeof SATTR);
    preg(KANIM, sizeof KANIM); preg(AJ, sizeof AJ);
    freg_all();
    for (op = 0; op < 13; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { s32 a0 = (s32) fv(); ret = PI(it_80293660((int) (fv() % 6))); break; }
            case 1: { itKinoko_Logic26_Spawned(&G); break; }
            case 2: { it_802936E4(&G); break; }
            case 3: { ret = (u32) itKinoko_UnkMotion0_Anim(&G); break; }
            case 4: { itKinoko_UnkMotion0_Phys(&G); break; }
            case 5: { ret = (u32) itKinoko_UnkMotion0_Coll(&G); break; }
            case 6: { it_8029385C(&G); break; }
            case 7: { it_80293884(&G); break; }
            case 8: { ret = (u32) itKinoko_UnkMotion1_Anim(&G); break; }
            case 9: { itKinoko_UnkMotion1_Phys(&G); break; }
            case 10: { ret = (u32) itKinoko_UnkMotion1_Coll(&G); break; }
            case 11: { ret = (u32) itKinoko_Logic26_DmgDealt(&G); break; }
            case 12: { itKinoko_Logic26_EvtUnk(&G, &REFG); break; }
            }
            hash_all();
            printf("%d %d %08X %08X %08X%c", op, k, ret, H, TR, 10);
        }
    }
    return 0;
}
