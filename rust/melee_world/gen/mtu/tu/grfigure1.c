/* TU: src/melee/gr/grfigure1.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_stage_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj SGO;
static Ground SGP;
static char SJO[16];
static char YPB[64];
static Vec3 SV;

#include "grfigure1_gen.h"



static void setup(void)
{
    int i, n;
    (void) i; (void) n;
    zero(&SGO, sizeof SGO); zero(&SGP, sizeof SGP);
    fill_HSD_GObj(&SGO); fill_Ground(&SGP); fill_StageInfo(&stage_info);
    SGO.user_data = &SGP;
    SGO.hsd_obj = SJO;
    SV.x = rw(); SV.y = rw(); SV.z = rw();
    yakumono_param = (rnd() & 1) ? (void*) YPB : 0;

}

static void hash_all(void)
{
    int i;
    (void) i;
    hash_HSD_GObj(&SGO); hash_Ground(&SGP); hash_StageInfo(&stage_info);
    HM(B(SV.x)); HM(B(SV.y)); HM(B(SV.z));
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
    preg(grEF1_StageCallbacks, sizeof grEF1_StageCallbacks);
    preg(&grEF1_StageData, sizeof grEF1_StageData);
    freg_all();
    for (op = 0; op < 22; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { grFigure1_OnDemoInit((int) fv()); } break;
            case 1: { grFigure1_OnInit(); } break;
            case 2: { grFigure1_OnLoad(); } break;
            case 3: { grFigure1_OnStart(); } break;
            case 4: { ret = (u32) grFigure1_8020E078(); } break;
            case 5: { ret = PI(grFigure1_8020E080((int) (rnd() % 3))); } break;
            case 6: { stageGObj0_OnInit(&SGO); } break;
            case 7: { ret = (u32) grFigure1_8020E194(&SGO); } break;
            case 8: { grFigure1_8020E19C(&SGO); } break;
            case 9: { grFigure1_8020E1A0(&SGO); } break;
            case 10: { grFigure1_8020E1A4(&SGO); } break;
            case 11: { ret = (u32) grFigure1_8020E1D0(&SGO); } break;
            case 12: { grFigure1_8020E1D8(&SGO); } break;
            case 13: { grFigure1_8020E1FC(&SGO); } break;
            case 14: { stageGObj2_OnInit(&SGO); } break;
            case 15: { ret = (u32) grFigure1_8020E250(&SGO); } break;
            case 16: { grFigure1_8020E258(&SGO); } break;
            case 17: { grFigure1_8020E25C(&SGO); } break;
            case 18: { ret = PI(grFigure1_OnTouchLine((enum_t) fv())); } break;
            case 19: { ret = (u32) grFigure1_OnCheckShadowRender(&SV, (int) fv(), (HSD_JObj*) SJO); } break;
            case 20: { grFigure1_8020E270((int) fv()); } break;
            case 21: { { int i; for (i = 0; i < 3; i++) hash_StageCallbacks(&grEF1_StageCallbacks[i]); hash_StageData(&grEF1_StageData); HM(SH(grEF1_StageData.data1)); HM((u32) grEF1_StageData.joint_count); } } break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
