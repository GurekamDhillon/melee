/* TU: src/melee/gm/gmfixedcamera.c */
/* C oracle for src/tu/gmfixedcamera.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static GameModeState ST;
static struct gmm_x0 GM;
static StartMeleeData SMD, SMD2;
#include "gmfixedcamera_gen.h"

int main(void)
{
    int op, k, i;
    preg(&ST, sizeof ST);
    preg(&GM, sizeof GM);
    preg(&SMD, sizeof SMD);
    preg(&SMD2, sizeof SMD2);
    preg(gm_Mode_CameraVs_States, sizeof gm_Mode_CameraVs_States);
    preg(&gmVsMelee_ApproachData, sizeof gmVsMelee_ApproachData);
    preg(&gmVsMelee_CssData, sizeof gmVsMelee_CssData);
    preg(&gmVsMelee_ResultsEnterData, sizeof gmVsMelee_ResultsEnterData);
    preg(&gmVsMelee_SssData, sizeof gmVsMelee_SssData);
    preg(&gmVsMelee_StartData, sizeof gmVsMelee_StartData);
    preg(&gmVsMelee_SuddenDeathExitInfo, sizeof gmVsMelee_SuddenDeathExitInfo);
    preg(&gmVsMelee_VsExitInfo, sizeof gmVsMelee_VsExitInfo);
    preg(&if_Scene_Prize_EnterData, sizeof if_Scene_Prize_EnterData);
    freg_all();
    for (op = 0; op < 14; op++) {
        for (k = 0; k < 300; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&ST, sizeof ST);
            zero(&SMD, sizeof SMD);
            zero(&SMD2, sizeof SMD2);
            gmMainLib_804D3EE0 = &GM;
            fill_StartMeleeData(&SMD);
            switch (op) {
            case 0: gm_801B9F10(&ST); break;
            case 1: gm_801B9F3C(&ST); break;
            case 2: gm_801B9F64(&ST); break;
            case 3: gm_801B9F8C(&ST); break;
            case 4: gm_801B9FC8(&ST); break;
            case 5: gm_801B9FFC(&ST); break;
            case 6: gm_801BA024(&ST); break;
            case 7: gm_801BA058(&ST); break;
            case 8: gm_801BA078(&ST); break;
            case 9: gm_801BA098(&ST); break;
            case 10: fn_801B9FB8(&SMD, &SMD2); break;
            case 11: gm_Mode_CameraVs_OnInit(); break;
            case 12: gm_Mode_CameraVs_OnLoad(); break;
            case 13: for (i = 0; i < 9; i++) hash_GameModeState(&gm_Mode_CameraVs_States[i]); break;
            }
            hash_StartMeleeData(&SMD);
            hash_StartMeleeData(&SMD2);
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
