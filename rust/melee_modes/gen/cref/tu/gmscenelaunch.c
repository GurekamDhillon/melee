/* TU: src/melee/gm/gmscenelaunch.h (via gmscenelaunch_src.c) */
/* C oracle for src/tu/gmscenelaunch.rs: the header included verbatim; every callee is a recording
 * stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static VsModeData VSD;
static PreloadedGameModeState PCS;
static GameRules GR;
static struct GamePrefs GP;
#include "gmscenelaunch_gen.h"

int main(void)
{
    int op, k;
    preg(&VSD, sizeof VSD);
    preg(&PCS, sizeof PCS);
    preg(&GR, sizeof GR);
    preg(&GP, sizeof GP);
    preg(&gm_80473814, sizeof gm_80473814);
    freg_all();
    for (op = 0; op < 3; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            fill_VsModeData(&VSD);
            fill_PreloadedGameModeState(&PCS);
            fill_GameRules(&GR);
            fill_GamePrefs(&GP);
            fill_CssSubStruct(&gm_80473814);
            switch (op) {
            case 0: ret = (u32) w_SeedPlayers(&VSD, k & 1); break;
            case 1: ret = (u32) w_SeedVs(&VSD, (k >> 1) & 1); break;
            default: w_ApplyItemFrequency(&VSD.start.rules, &GP); break;
            }
            hash_VsModeData(&VSD);
            hash_PreloadedGameModeState(&PCS);
            hash_GameRules(&GR);
            hash_GamePrefs(&GP);
            hash_CssSubStruct(&gm_80473814);
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
