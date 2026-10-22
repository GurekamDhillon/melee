/* TU: src/melee/gm/gmgiant.c */
/* C oracle for src/tu/gmgiant.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static GameModeState ST;
static struct gmm_x0 GM;
static PlayerInitData PID, PID2;
#include "gmgiant_gen.h"

int main(void)
{
    int op, k, i;
    preg(&ST, sizeof ST);
    preg(&GM, sizeof GM);
    preg(&PID, sizeof PID);
    preg(&PID2, sizeof PID2);
    preg(gm_Mode_GiantVs_States, sizeof gm_Mode_GiantVs_States);
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
            zero(&PID, sizeof PID);
            zero(&PID2, sizeof PID2);
            gmMainLib_804D3EE0 = &GM;
            fill_PlayerInitData(&PID);
            switch (op) {
            case 0: gm_801B8FB8(&ST); break;
            case 1: gm_801B8FE4(&ST); break;
            case 2: gm_801B900C(&ST); break;
            case 3: gm_801B9034(&ST); break;
            case 4: gm_801B9084(&ST); break;
            case 5: gm_801B90B8(&ST); break;
            case 6: gm_801B90E0(&ST); break;
            case 7: gm_801B9114(&ST); break;
            case 8: gm_801B9134(&ST); break;
            case 9: gm_801B9154(&ST); break;
            case 10: fn_801B9060(&PID, &PID2); break;
            case 11: gm_Mode_GiantVs_OnInit(); break;
            case 12: gm_Mode_GiantVs_OnLoad(); break;
            case 13: for (i = 0; i < 9; i++) hash_GameModeState(&gm_Mode_GiantVs_States[i]); break;
            }
            hash_PlayerInitData(&PID);
            hash_PlayerInitData(&PID2);
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
