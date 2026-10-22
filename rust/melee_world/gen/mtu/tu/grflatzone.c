/* TU: src/melee/gr/grflatzone.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_stage_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj SGO;
static Ground SGP;
static char SJO[16];
static char YPB[64];
static Vec3 SV;
static HSD_JObj SJOB;
static struct grFlatzone_YakumonoParam YP;
static HSD_GObj MGO[10], IGO;
static Ground MGP[10];
static HSD_JObj MJO[10];
static Item ITD;
static grDynamicAttr_UnkStruct DYN[2];

#include "grflatzone_gen.h"



static void setup(void)
{
    int i, n;
    (void) i; (void) n;
    zero(&SGO, sizeof SGO); zero(&SGP, sizeof SGP);
    fill_HSD_GObj(&SGO); fill_Ground(&SGP); fill_StageInfo(&stage_info);
    SGO.user_data = &SGP;
    SGO.hsd_obj = SJO;
    SV.x = rw(); SV.y = rw(); SV.z = rw();
    zero(&SJOB, sizeof SJOB); zero(&YP, sizeof YP); zero(MGO, sizeof MGO); zero(&IGO, sizeof IGO);
    zero(MGP, sizeof MGP); zero(MJO, sizeof MJO); zero(&ITD, sizeof ITD); zero(DYN, sizeof DYN);
    fill_HSD_JObj(&SJOB);
    SGO.hsd_obj = &SJOB;
    fill_grFlatzone_YakumonoParam(&YP);
    YP.unk0 = (int) (rnd() % 60); YP.unk4 = (int) (rnd() % 60); YP.unk8 = (int) (rnd() % 60); YP.unkC = (int) (rnd() % 60);
    YP.unk10 = (int) (rnd() % 100); YP.unk14 = 1 + (int) (rnd() % 6); YP.unk18 = (int) (rnd() % 80) - 40;
    YP.unk1C = (int) (rnd() % 80) - 40; YP.unk20 = (int) (rnd() % 50); YP.unk2C = 1 + (int) (rnd() % 5);
    YP.unk30 = (int) (rnd() % 10); YP.unk34 = (int) (rnd() % 10); YP.unk3C = (int) (rnd() % 80);
    YP.unk24 = rw(); YP.unk28 = rw(); YP.unk38 = rw();
    yakumono_param = &YP;
    fill_HSD_GObj(&IGO); IGO.user_data = ((rnd() & 3) == 0) ? 0 : &ITD;
    fill_CollData(&ITD.x378_itemColl);
    for (i = 0; i < 2; i++) fill_grDynamicAttr_UnkStruct(&DYN[i]);
    for (i = 0; i < 10; i++) {
        fill_HSD_GObj(&MGO[i]); fill_Ground(&MGP[i]); fill_HSD_JObj(&MJO[i]);
        MGO[i].user_data = ((rnd() & 7) == 0) ? 0 : &MGP[i];
        MGO[i].hsd_obj = ((rnd() & 7) == 0) ? 0 : (void*) &MJO[i];
    }
    n = (int) (rnd() % 4);
    SGP.u.flatzone2.xD0 = (n == 0) ? 2 : ((n == 1) ? 1 : (int) (rnd() % 7) - 1);
    SGP.u.flatzone2.timer = (int) (rnd() % 45) - 3;
    SGP.u.flatzone2.xC8 = (rnd() & 1) ? ((rnd() & 1) ? 1.0f : -1.0f) : rw();
    SGP.u.flatzone2.xCC = 0;

}

static void hash_all(void)
{
    int i;
    (void) i;
    for (i = 0; i < 10; i++) { HM(PIB(MGP[i].u.flatzone2.xCC)); MGP[i].u.flatzone2.xCC = 0; }
    HM(PIB(SGP.u.flatzone2.xCC)); SGP.u.flatzone2.xCC = 0;
    hash_HSD_GObj(&SGO); hash_Ground(&SGP); hash_StageInfo(&stage_info);
    HM(B(SV.x)); HM(B(SV.y)); HM(B(SV.z));
    hash_HSD_JObj(&SJOB); hash_grFlatzone_YakumonoParam(&YP); HM(PI(yakumono_param));
    for (i = 0; i < 10; i++) { hash_HSD_GObj(&MGO[i]); hash_Ground(&MGP[i]); hash_HSD_JObj(&MJO[i]); }
    hash_HSD_GObj(&IGO); hash_CollData(&ITD.x378_itemColl);
    for (i = 0; i < 2; i++) hash_grDynamicAttr_UnkStruct(&DYN[i]);

}

int main(void)
{
    int op, k;
    preg(&SGO, sizeof SGO);
    preg(&SGP, sizeof SGP);
    preg(SJO, sizeof SJO);
    preg(YPB, sizeof YPB);
    preg(&SV, sizeof SV);
    preg(&SJOB, sizeof SJOB);
    preg(&YP, sizeof YP);
    preg(MGO, sizeof MGO);
    preg(&IGO, sizeof IGO);
    preg(MGP, sizeof MGP);
    preg(MJO, sizeof MJO);
    preg(&ITD, 16);
    preg(DYN, sizeof DYN);
    preg(grFz_StageCallbacks, sizeof grFz_StageCallbacks);
    preg(&grFz_StageData, sizeof grFz_StageData);
    freg_all();
    for (op = 0; op < 56; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { grFlatzone_80216E74((int) fv()); } break;
            case 1: { grFlatzone_80216E78(); } break;
            case 2: { grFlatzone_UnkStage0_OnLoad(); } break;
            case 3: { grFlatzone_UnkStage0_OnStart(); } break;
            case 4: { ret = (u32) grFlatzone_80216F40(); } break;
            case 5: { ret = PI(grFlatzone_80216F48((int) (rnd() % 9))); } break;
            case 6: { stageGObj0_OnInit(&SGO); } break;
            case 7: { ret = (u32) grFlatzone_8021705C(&SGO); } break;
            case 8: { grFlatzone_80217064(&SGO); } break;
            case 9: { grFlatzone_80217068(&SGO); } break;
            case 10: { fn_8021706C(&SGO); } break;
            case 11: { grFlatzone_8021707C(&SGO); } break;
            case 12: { ret = (u32) grFlatzone_802171CC(&SGO); } break;
            case 13: { grFlatzone_802171D4(&SGO); } break;
            case 14: { grFlatzone_802173D0(&SGO); } break;
            case 15: { grFlatzone_802173D4(&SGO); } break;
            case 16: { ret = (u32) grFlatzone_80217430(&SGO); } break;
            case 17: { grFlatzone_80217438(&SGO); } break;
            case 18: { grFlatzone_8021743C(&SGO); } break;
            case 19: { fn_80217440(&SGO); } break;
            case 20: { grFlatzone_80217450(&SGO); } break;
            case 21: { ret = (u32) grFlatzone_802174E4(&SGO); } break;
            case 22: { SGP.u.flatzone3.xC7 = (rnd() % 5 == 4) ? 8 : (u8) (rnd() % 4); SGP.u.flatzone3.xC9 = (u8) (rnd() % 5); grFlatzone_802174EC(&SGO); } break;
            case 23: { grFlatzone_80217644(&SGO); } break;
            case 24: { grFlatzone_80217648(&SGO); } break;
            case 25: { ret = (u32) grFlatzone_802176B4(&SGO); } break;
            case 26: { SGP.u.flatzone.xC4 = (u8) (rnd() & 1); SGP.u.flatzone2.xCC = (rnd() & 1) ? &DYN[0] : 0; grFlatzone_802176BC(&SGO); } break;
            case 27: { grFlatzone_80217CD0(&SGO); } break;
            case 28: { grFlatzone_80217CD4(&SGO); } break;
            case 29: { ret = (u32) grFlatzone_80217D28(&SGO); } break;
            case 30: { grFlatzone_80217D30(&SGO); } break;
            case 31: { grFlatzone_80217DA4(&SGO); } break;
            case 32: { grFlatzone_80217DA8(&SGO); } break;
            case 33: { ret = (u32) grFlatzone_80217DFC(&SGO); } break;
            case 34: { grFlatzone_80217E04(&SGO); } break;
            case 35: { grFlatzone_80217E78(&SGO); } break;
            case 36: { grFlatzone_80217E7C(&SGO); } break;
            case 37: { ret = (u32) grFlatzone_80217E80(&SGO); } break;
            case 38: { grFlatzone_80217E88(&SGO); } break;
            case 39: { grFlatzone_80217E8C(&SGO); } break;
            case 40: { grFlatzone_80217E90(&SGO); } break;
            case 41: { ret = (u32) grFlatzone_80217EE8(&SGO); } break;
            case 42: { grFlatzone_80217EF0(&SGO); } break;
            case 43: { grFlatzone_8021805C(&SGO); } break;
            case 44: { grFlatzone_80218060((int) (rnd() & 1)); } break;
            case 45: { grFlatzone_802181B4(); } break;
            case 46: { grFlatzone_80218260(); } break;
            case 47: { ret = PI(grFlatzone_802182B4((enum_t) fv())); } break;
            case 48: { ret = (u32) grFlatzone_802182BC(&SV, (int) fv(), (HSD_JObj*) SJO); } break;
            case 49: { SGP.u.flatzone2.xCC = (rnd() & 1) ? &DYN[0] : 0; SGP.u.flatzone.xC4 = 0; SGP.u.flatzone2.xD0 = 1; SGP.u.flatzone2.timer = YP.unk2C * (int) (rnd() % 5) + 1; SGP.u.flatzone2.xC8 = rw(); grFlatzone_802176BC(&SGO); } break;
            case 50: { SGP.u.flatzone2.xCC = (rnd() & 1) ? &DYN[0] : 0; SGP.u.flatzone.xC4 = 0; SGP.u.flatzone2.xD0 = 2; SGP.u.flatzone2.xC8 = rw(); grFlatzone_802176BC(&SGO); } break;
            case 51: { { int j; grFlatzone_80217648(&SGO); SGP.u.flatzone2.xCC = (rnd() & 1) ? &DYN[0] : 0; SGP.u.flatzone.xC4 = 1; for (j = 0; j < 90; j++) { grFlatzone_802176BC(&SGO); if ((rnd() & 31) == 0) SGP.u.flatzone.xC4 = 1; } } } break;
            case 52: { { int j; grFlatzone_80217450(&SGO); SGP.u.flatzone3.xC4 = 0; SGP.u.flatzone3.xC7 = (u8) (rnd() % 4); SGP.u.flatzone3.xC9 = (u8) (1 + rnd() % 4); for (j = 0; j < 60; j++) { grFlatzone_802174EC(&SGO); if ((rnd() & 15) == 0) SGP.u.flatzone3.xC7 = (u8) (rnd() % 4); } } } break;
            case 53: { { int j; grFlatzone_80217E90(&SGO); SGP.u.flatzone3.xC4 = 1; for (j = 0; j < 80; j++) grFlatzone_80217EF0(&SGO); } } break;
            case 54: { { int j; grFlatzone_8021707C(&SGO); SGP.u.flatzone.xC4 = 0; for (j = 0; j < 200; j++) grFlatzone_802171D4(&SGO); } } break;
            case 55: { { int i; for (i = 0; i < 9; i++) hash_StageCallbacks(&grFz_StageCallbacks[i]); hash_StageData(&grFz_StageData); HM(SH(grFz_StageData.data1)); HM((u32) grFz_StageData.joint_count); } } break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
