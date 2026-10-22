/* TU: src/melee/gm/gmhanyucss.c */
/* C oracle for src/tu/gmhanyucss.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static VsModeData VSD;
static GameModeState ST;
#include "gmhanyucss_gen.h"

int main(void)
{
    int op, k, i;
    preg(&ST, sizeof ST);
    preg(&gm_8049BEE8, sizeof gm_8049BEE8);
    preg(&VSD, sizeof VSD);
    preg(gm_Mode_HanyuCss_States, sizeof gm_Mode_HanyuCss_States);
    freg_all();
    for (op = 0; op < 3; op++) {
        for (k = 0; k < 1000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&ST, sizeof ST);
            zero(&gm_8049BEE8, sizeof gm_8049BEE8);
            zero(&VSD, sizeof VSD);
            ST.info.enter_data = &gm_8049BEE8;
            fill_CSSData(&gm_8049BEE8);
            fill_VsModeData(&VSD);
            switch (op) {
            case 0: gm_801BED3C(&ST); break;
            case 1: gm_801BEDA8(&ST); break;
            case 2:
                for (i = 0; i < 2; i++) hash_GameModeState(&gm_Mode_HanyuCss_States[i]);
                break;
            }
            hash_CSSData(&gm_8049BEE8);
            hash_VsModeData(&VSD);
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
