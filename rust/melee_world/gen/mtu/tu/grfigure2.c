/* TU: src/melee/gr/grfigure2.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_stage_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj SGO;
static Ground SGP;
static char SJO[16];
static char YPB[64];
static Vec3 SV;

#include "grfigure2_gen.h"



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
    preg(&grEF2_StageData, sizeof grEF2_StageData);
    freg_all();
    for (op = 0; op < 20; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { grFigure2_8020E274(); } break;
            case 1: { grFigure2_OnLoad(); } break;
            case 2: { grFigure2_OnStart(); } break;
            case 3: { ret = (u32) grFigure2_8020E30C(); } break;
            case 4: { ret = PI(grFigure2_8020E314((int) (rnd() % 3))); } break;
            case 5: { stageGObj0_OnInit(&SGO); } break;
            case 6: { ret = (u32) grFigure2_8020E428(&SGO); } break;
            case 7: { grFigure2_8020E430(&SGO); } break;
            case 8: { grFigure2_8020E434(&SGO); } break;
            case 9: { grFigure2_8020E438(&SGO); } break;
            case 10: { ret = (u32) grFigure2_8020E464(&SGO); } break;
            case 11: { grFigure2_8020E46C(&SGO); } break;
            case 12: { grFigure2_8020E490(&SGO); } break;
            case 13: { stageGObj2_OnInit(&SGO); } break;
            case 14: { ret = (u32) grFigure2_8020E4E4(&SGO); } break;
            case 15: { grFigure2_8020E4EC(&SGO); } break;
            case 16: { grFigure2_8020E4F0(&SGO); } break;
            case 17: { ret = PI(grFigure2_8020E4F4((enum_t) fv())); } break;
            case 18: { ret = (u32) grFigure2_8020E4FC(&SV, (int) fv(), (HSD_JObj*) SJO); } break;
            case 19: { { int i; for (i = 0; i < 3; i++) hash_StageCallbacks(&grEF2_StageCallbacks[i]); hash_StageData(&grEF2_StageData); HM(SH(grEF2_StageData.data1)); HM((u32) grEF2_StageData.joint_count); } } break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
