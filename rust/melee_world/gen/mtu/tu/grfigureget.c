/* TU: src/melee/gr/grfigureget.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_stage_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj SGO;
static Ground SGP;
static char SJO[16];
static char YPB[64];
static Vec3 SV;
static HSD_GObj FIG[3];
static HSD_GObj* PLINK[16];
static grFigureGet_Params YP;
static CollData FCD, CD;
static HSD_GObj IG[3], CGO;
static Item ITD;

#include "grfigureget_gen.h"



static void setup(void)
{
    int i, n;
    (void) i; (void) n;
    zero(&SGO, sizeof SGO); zero(&SGP, sizeof SGP);
    fill_HSD_GObj(&SGO); fill_Ground(&SGP); fill_StageInfo(&stage_info);
    SGO.user_data = &SGP;
    SGO.hsd_obj = SJO;
    SV.x = rw(); SV.y = rw(); SV.z = rw();
    zero(PLINK, sizeof PLINK); zero(FIG, sizeof FIG);
    for (i = 0; i < 3; i++) fill_HSD_GObj(&FIG[i]);
    HSD_GObjPLinkHead = PLINK;
    n = (int) (rnd() % 4);
    for (i = 0; i < n; i++) FIG[i].next = (i + 1 < n) ? &FIG[i + 1] : 0;
    PLINK[HSD_GOBJ_PLINK_FIGHTER] = n ? &FIG[0] : 0;
    zero(&YP, sizeof YP); zero(&FCD, sizeof FCD); zero(&CD, sizeof CD); zero(IG, sizeof IG); zero(&CGO, sizeof CGO);
    zero(&ITD, sizeof ITD);
    fill_grFigureGet_Params(&YP);
    YP.x8 = (int) (rnd() % 4);
    YP.x4 = (rnd() & 1) ? 0 : (int) (rnd() % 40);
    YP.x0 = (int) (rnd() % 30);
    YP.xC = 0.0f;
    YP.x10 = (float) (rnd() % 200);
    YP.x14 = rw();
    fill_CollData(&FCD); fill_CollData(&CD); fill_HSD_GObj(&CGO);
    for (i = 0; i < 3; i++) fill_HSD_GObj(&IG[i]);
    yakumono_param = &YP;
    SGP.u.figureget.x4 = (int) (rnd() % 3);
    SGP.u.figureget.x8 = (int) (rnd() % 4);
    SGP.u.figureget.xC = (int) (rnd() % 4);
    SGP.u.figureget.x0 = (int) (rnd() % 4);
    SGP.u.figureget.x28[0] = ((rnd() & 3) == 0) ? &CGO : 0;
    CD.x0_gobj = ((rnd() & 3) == 0) ? 0 : &CGO;
    CGO.classifier = (rnd() & 3) ? HSD_GOBJ_CLASS_ITEM : (u16) fv();
    CGO.user_data = &ITD;
    ITD.kind = (rnd() & 3) ? It_Kind_Coin : (int) fv();
    CD.x34_flags.b1234 = (rnd() & 3) ? 5 : fv();

}

static void hash_all(void)
{
    int i;
    (void) i;
    for (i = 0; i < 3; i++) {
        HM(PI(SGP.u.figureget.x28[i])); SGP.u.figureget.x28[i] = 0;
        HM(PI(SGP.u.figureget.x34[i])); SGP.u.figureget.x34[i] = 0;
    }
    hash_HSD_GObj(&SGO); hash_Ground(&SGP); hash_StageInfo(&stage_info);
    HM(B(SV.x)); HM(B(SV.y)); HM(B(SV.z));
    for (i = 0; i < 3; i++) hash_HSD_GObj(&FIG[i]);
    hash_grFigureGet_Params(&YP); hash_CollData(&FCD); hash_CollData(&CD); hash_HSD_GObj(&CGO);
    for (i = 0; i < 3; i++) hash_HSD_GObj(&IG[i]);
    HM(PI(yakumono_param));

}

int main(void)
{
    int op, k;
    preg(&SGO, sizeof SGO);
    preg(&SGP, sizeof SGP);
    preg(SJO, sizeof SJO);
    preg(YPB, sizeof YPB);
    preg(&SV, sizeof SV);
    preg(FIG, sizeof FIG);
    preg(PLINK, sizeof PLINK);
    preg(&YP, sizeof YP);
    preg(&FCD, sizeof FCD);
    preg(&CD, sizeof CD);
    preg(IG, sizeof IG);
    preg(&CGO, sizeof CGO);
    preg(&ITD, sizeof ITD);
    preg(grFigureGet_StageCallbacks, sizeof grFigureGet_StageCallbacks);
    preg(&grFigureGet_StageData, sizeof grFigureGet_StageData);
    freg_all();
    for (op = 0; op < 21; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { grFigureGet_OnDemoInit((int) fv()); } break;
            case 1: { grFigureGet_OnInit(); } break;
            case 2: { grFigureGet_OnLoad(); } break;
            case 3: { grFigureGet_OnStart(); } break;
            case 4: { ret = (u32) grFigureGet_802195C4(); } break;
            case 5: { ret = PI(grFigureGet_802195CC((int) (rnd() % 2))); } break;
            case 6: { stageGObj0_OnInit(&SGO); } break;
            case 7: { ret = (u32) grFigureGet_802196E0(&SGO); } break;
            case 8: { grFigureGet_802196E8(&SGO); } break;
            case 9: { grFigureGet_802196EC(&SGO); } break;
            case 10: { grFigureGet_802196F0(&SGO); } break;
            case 11: { ret = (u32) grFigureGet_80219890(&SGO); } break;
            case 12: { grFigureGet_80219898(&SGO); } break;
            case 13: { grFigureGet_80219B0C(&SGO); } break;
            case 14: { { int j = (int) fv(); int x = (int) fv(); float d = rw(); grFigureGet_80219B10(&SGP, j, &CD, x, (mpLib_GroundEnum) fv(), d); } } break;
            case 15: { { HSD_GObj* g = ((rnd() & 3) == 0) ? 0 : &SGO; grFigureGet_80219C34(g); } } break;
            case 16: { { HSD_GObj* g = ((rnd() & 3) == 0) ? 0 : &SGO; ret = (u32) grFigureGet_80219C50(g); } } break;
            case 17: { ret = PI(grFigureGet_OnTouchLine((enum_t) fv())); } break;
            case 18: { ret = (u32) grFigureGet_OnCheckShadowRender(&SV, (int) fv(), (HSD_JObj*) SJO); } break;
            case 19: { { int j; for (j = 0; j < 12; j++) { grFigureGet_80219898(&SGO); CD.cur_pos.x = rw(); if (rnd() & 1) { int a = (int) fv(); grFigureGet_80219B10(&SGP, 0, &CD, a, (mpLib_GroundEnum) 0, 0.0f); } } } } break;
            case 20: { { int i; for (i = 0; i < 2; i++) hash_StageCallbacks(&grFigureGet_StageCallbacks[i]); hash_StageData(&grFigureGet_StageData); HM(SH(grFigureGet_StageData.data1)); HM((u32) grFigureGet_StageData.joint_count); } } break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
