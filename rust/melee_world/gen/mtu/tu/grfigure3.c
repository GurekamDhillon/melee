/* TU: src/melee/gr/grfigure3.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_stage_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj SGO;
static Ground SGP;
static char SJO[16];
static char YPB[64];
static Vec3 SV;

#include "grfigure3_gen.h"



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
    preg(grEF2_StageCallbacks, sizeof grEF2_StageCallbacks);
    preg(&grEF3_StageData, sizeof grEF3_StageData);
    freg_all();
    for (op = 0; op < 21; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { grFigure3_8020E504((int) fv()); } break;
            case 1: { grFigure3_8020E508(); } break;
            case 2: { grFigure3_OnLoad(); } break;
            case 3: { grFigure3_OnStart(); } break;
            case 4: { ret = (u32) grFigure3_8020E5A0(); } break;
            case 5: { ret = PI(grFigure3_8020E5A8((int) (rnd() % 3))); } break;
            case 6: { stageGObj0_OnInit(&SGO); } break;
            case 7: { ret = (u32) grFigure3_8020E6BC(&SGO); } break;
            case 8: { grFigure3_8020E6C4(&SGO); } break;
            case 9: { grFigure3_8020E6C8(&SGO); } break;
            case 10: { grFigure3_8020E6CC(&SGO); } break;
            case 11: { ret = (u32) grFigure3_8020E6F8(&SGO); } break;
            case 12: { grFigure3_8020E700(&SGO); } break;
            case 13: { grFigure3_8020E724(&SGO); } break;
            case 14: { stageGObj2_OnInit(&SGO); } break;
            case 15: { ret = (u32) grFigure3_8020E778(&SGO); } break;
            case 16: { grFigure3_8020E780(&SGO); } break;
            case 17: { grFigure3_8020E784(&SGO); } break;
            case 18: { ret = PI(grFigure3_8020E788((enum_t) fv())); } break;
            case 19: { ret = (u32) grFigure3_8020E790(&SV, (int) fv(), (HSD_JObj*) SJO); } break;
            case 20: { { int i; for (i = 0; i < 3; i++) hash_StageCallbacks(&grEF2_StageCallbacks[i]); hash_StageData(&grEF3_StageData); HM(SH(grEF3_StageData.data1)); HM((u32) grEF3_StageData.joint_count); } } break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
