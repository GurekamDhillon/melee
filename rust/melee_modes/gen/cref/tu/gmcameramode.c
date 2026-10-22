/* TU: src/melee/gm/gmcameramode.c */
/* C oracle for src/tu/gmcameramode.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static GameModeState ST;
static struct gmm_x0 GM;
static PreloadedGameModeState PCS;
static unsigned char AR[16];
static unsigned char KOC[8];
static int EXI;
#include "gmcameramode_gen.h"

int main(void)
{
    int op, k, i;
    preg(&ST, sizeof ST);
    preg(&GM, sizeof GM);
    preg(&PCS, sizeof PCS);
    preg(AR, sizeof AR);
    preg(KOC, sizeof KOC);
    preg(&EXI, sizeof EXI);
    preg(gm_Mode_Camera_States, sizeof gm_Mode_Camera_States);
    preg(&gm_804D68C8, sizeof gm_804D68C8);
    preg(&gmVsMelee_CssData, sizeof gmVsMelee_CssData);
    preg(&gmVsMelee_SssData, sizeof gmVsMelee_SssData);
    preg(&gmVsMelee_StartData, sizeof gmVsMelee_StartData);
    preg(&gmVsMelee_VsExitInfo, sizeof gmVsMelee_VsExitInfo);
    freg_all();
    for (op = 0; op < 11; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&ST, sizeof ST);
            zero(&GM, sizeof GM);
            zero(&PCS, sizeof PCS);
            zero(&gmVsMelee_CssData, sizeof gmVsMelee_CssData);
            zero(&gmVsMelee_SssData, sizeof gmVsMelee_SssData);
            zero(&gmVsMelee_StartData, sizeof gmVsMelee_StartData);
            zero(&gmVsMelee_VsExitInfo, sizeof gmVsMelee_VsExitInfo);
            gmMainLib_804D3EE0 = &GM;
            for (i = 0; i < 15; i++) fill_VsModeData(&GM.modes.table[i]);
            fill_PreloadedGameModeState(&PCS);
            fill_CSSData(&gmVsMelee_CssData);
            fill_SSSData(&gmVsMelee_SssData);
            fill_StartMeleeData(&gmVsMelee_StartData);
            EXI = (int) (fv() % 4);
            switch (op) {
            case 0: gm_801B23F0(); break;
            case 1: gm_801B24B4(&ST); break;
            case 2: ST.info.exit_data = &EXI; gm_801B2510(&ST); break;
            case 3: ST.info.enter_data = &gmVsMelee_CssData; gm_801B254C(&ST); break;
            case 4: ST.info.exit_data = &gmVsMelee_CssData; gm_801B25D4(&ST); break;
            case 5: ST.info.enter_data = &gmVsMelee_SssData; gm_801B26AC(&ST); break;
            case 6: ST.info.exit_data = &gmVsMelee_SssData; gm_801B2704(&ST); break;
            case 7: ST.info.enter_data = &gmVsMelee_StartData; gm_PrepCameraModeVSScene(&ST); break;
            case 8: gm_801B2AF8(&ST); break;
            case 9: gm_Mode_Camera_OnInit(); break;
            case 10: for (i = 0; i < 5; i++) hash_GameModeState(&gm_Mode_Camera_States[i]); break;
            }
            for (i = 0; i < 15; i++) hash_VsModeData(&GM.modes.table[i]);
            hash_PreloadedGameModeState(&PCS);
            hash_CSSData(&gmVsMelee_CssData);
            hash_SSSData(&gmVsMelee_SssData);
            hash_StartMeleeData(&gmVsMelee_StartData);
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
