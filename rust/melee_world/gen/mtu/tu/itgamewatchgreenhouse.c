/* TU: src/melee/it/kinds/itgamewatchgreenhouse.c (written by gen/mtu/mk_gamewatch.py: do not edit) */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj G, NEWG, OWN, PAR;
static Item ITEM, NEWITEM;
static HSD_JObj JT[3];
static Article ART;
static void* ATTRS[2];
static int ATTR0[4];
static Vec POS;
#include "itgamewatchgreenhouse_gen.h"

static HSD_GObj* pickg(void)
{
    switch (rnd() % 3) {
    case 0: return 0;
    case 1: return &OWN;
    default: return &PAR;
    }
}
static void fillitem(Item* ip)
{
    unsigned i;
    unsigned char* b = (unsigned char*) ip;
    for (i = 0; i < sizeof *ip; i++) b[i] = (unsigned char) rnd();
}
static void setup(void)
{
    int i;
    float f;
    fillitem(&ITEM);
    fillitem(&NEWITEM);
    *(unsigned char*) &ITEM.xDCC_flag = (rnd() & 1) ? 0xFF : 0;
    *(unsigned char*) &NEWITEM.xDCC_flag = (rnd() & 1) ? 0xFF : 0;
    zero(&G, sizeof G); zero(&NEWG, sizeof NEWG); zero(&OWN, sizeof OWN); zero(&PAR, sizeof PAR);
    zero(&ART, sizeof ART); zero(ATTRS, sizeof ATTRS); zero(ATTR0, sizeof ATTR0);
    for (i = 0; i < 3; i++) {
        zero(&JT[i], sizeof JT[i]);
        JT[i].child = i < 2 ? &JT[i + 1] : 0;
        JT[i].flags = rnd() & ~(u32) JOBJ_USE_QUATERNION;
        JT[i].rotate.x = rw();
        JT[i].rotate.y = rw();
    }
    G.user_data = &ITEM; G.hsd_obj = &JT[0];
    NEWG.user_data = &NEWITEM; NEWG.hsd_obj = &JT[0];
    ATTRS[0] = &ATTR0[0];
    ART.x4_specialAttributes = ATTRS;
    ITEM.xC4_article_data = &ART; NEWITEM.xC4_article_data = &ART;
    ITEM.owner = pickg(); NEWITEM.owner = pickg();
    ITEM.xDD4_itemVar.gamewatchrescue.xDD8 = pickg();
    NEWITEM.xDD4_itemVar.gamewatchrescue.xDD8 = pickg();
    switch (rnd() % 4) { case 0: f = 3.0f; break; case 1: f = 30.0f; break; case 2: f = rw(); break; default: f = 0.0f; break; }
    ITEM.x5CC_currentAnimFrame = f;
    switch (rnd() % 4) { case 0: f = 0.0f; break; case 1: f = 1.0f; break; case 2: f = rw(); break; default: f = -0.0f; break; }
    ITEM.x5D0_animFrameSpeed = f;
    ITEM.xDAC_itcmd_var0 = fv(); ITEM.xDB0_itcmd_var1 = fv(); ITEM.xDB4_itcmd_var2 = fv(); ITEM.xDB8_itcmd_var3 = fv();
    NEWITEM.xDAC_itcmd_var0 = fv(); NEWITEM.xDB0_itcmd_var1 = fv(); NEWITEM.xDB4_itcmd_var2 = fv(); NEWITEM.xDB8_itcmd_var3 = fv();
    ITEM.ground_or_air = fv() % 3;
    POS.x = rw(); POS.y = rw(); POS.z = rw();
}
static void hash_item(const Item* ip)
{
    Item t = *ip;
    t.owner = (HSD_GObj*) (unsigned long long) PI(ip->owner);
    t.xDD4_itemVar.gamewatchrescue.xDD8 = (HSD_GObj*) (unsigned long long) PI(ip->xDD4_itemVar.gamewatchrescue.xDD8);
    t.xC4_article_data = (Article*) (unsigned long long) PI(ip->xC4_article_data);
    HM((u32) ip->xDCC_flag.b3);
    *(unsigned char*) &t.xDCC_flag = 0;
    HM(RH(&t, sizeof t));
}
static void hash_all(void)
{
    int i;
    hash_item(&ITEM); hash_item(&NEWITEM);
    for (i = 0; i < 3; i++) {
        HM(JT[i].flags); HM(B(JT[i].rotate.x)); HM(B(JT[i].rotate.y)); HM(B(JT[i].rotate.z)); HM(B(JT[i].rotate.w));
        HM(PI(JT[i].child));
    }
    HM(B(POS.x)); HM(B(POS.y)); HM(B(POS.z));
}

int main(void)
{
    int op, k, e;
    preg(&G, sizeof G); preg(&NEWG, sizeof NEWG); preg(&OWN, sizeof OWN); preg(&PAR, sizeof PAR);
    preg(&ITEM, sizeof ITEM); preg(&NEWITEM, sizeof NEWITEM); preg(JT, sizeof JT); preg(&ART, sizeof ART);
    preg(ATTRS, sizeof ATTRS); preg(ATTR0, sizeof ATTR0); preg(&POS, sizeof POS);
    freg_all();
    for (op = 0; op < 14; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { HSD_GObj* pa = pickg(); float d = rw(); u32 part = fv(); ret = PI(itGamewatchGreenhouse_Spawn(pa, &POS, part, d)); } break;
            case 1: { itGamewatchGreenhouse_Destroyed(&G); } break;
            case 2: { itGamewatchGreenhouse_802C6328(&G); } break;
            case 3: { itGamewatchGreenhouse_802C6374(&G); } break;
            case 4: { itGamewatchGreenhouse_802C6394(&G); } break;
            case 5: { itGamewatchGreenhouse_PickedUp(&G); } break;
            case 6: { itGamewatchGreenhouse_802C6430(&G); } break;
            case 7: { itGamewatchGreenhouse_802C6458(&G); } break;
            case 8: { itGamewatchGreenhouse_802C6480(&G); } break;
            case 9: { itGamewatchGreenhouse_802C64A8(&G); } break;
            case 10: { ret = (u32) itGamewatchGreenhouse_Motion3_Anim(&G); } break;
            case 11: { ret = (u32) itGamewatchGreenhouse_Motion2_Anim(&G); } break;
            case 12: { HSD_GObj* o = pickg(); itGamewatchGreenhouse_EvtUnk(&G, o); } break;
            case 13:
                for (e = 0; e < 4; e++) {
                    HM((u32) it_803F7898[e].anim_id); HM(FI((const void*) it_803F7898[e].animated));
                    HM(FI((const void*) it_803F7898[e].physics_updated)); HM(FI((const void*) it_803F7898[e].collided));
                }
                break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
