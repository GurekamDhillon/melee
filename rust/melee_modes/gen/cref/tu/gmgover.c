/* TU: src/melee/gm/gmgover.c */
/* C oracle for src/tu/gmgover.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static GameModeState ST;
static signed char EXB[1];
#include "gmgover_gen.h"

int main(void)
{
    int op, k, i;
    preg(&ST, sizeof ST);
    preg(EXB, sizeof EXB);
    preg(gm_8049C178, sizeof gm_8049C178);
    preg(gm_804D6920, sizeof gm_804D6920);
    preg(gm_Mode_GOver_States, sizeof gm_Mode_GOver_States);
    preg(gm_Mode_DebugGOver_States, sizeof gm_Mode_DebugGOver_States);
    freg_all();
    for (op = 0; op < 16; op++) {
        for (k = 0; k < 1000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&ST, sizeof ST);
            for (i = 0; i < 16; i++) gm_8049C178[i] = (u8) fv();
            EXB[0] = (signed char) fv();
            ST.info.exit_data = EXB;
            switch (op) {
            case 0: gm_801BEE9C(&ST); break;
            case 1: gm_801BEF84(&ST); break;
            case 2: gm_801BEFA4((int) fv()); break;
            case 3: gm_801BEFC0((int) fv()); break;
            case 4: gm_801BEFE0((s8) fv()); break;
            case 5: gm_801BF000((s8) fv()); break;
            case 6: gm_801BF020((s8) fv()); break;
            case 7: gm_801BF040((s8) fv()); break;
            case 8: ret = (u32) gm_801BEFB0(); break;
            case 9: ret = (u32) gm_801BEFD0(); break;
            case 10: ret = (u32) gm_801BEFF0(); break;
            case 11: ret = (u32) gm_801BF010(); break;
            case 12: ret = (u32) gm_801BF030(); break;
            case 13: ret = (u32) gm_801BF050(); break;
            case 14: for (i = 0; i < 5; i++) hash_GameModeState(&gm_Mode_GOver_States[i]); break;
            case 15: for (i = 0; i < 3; i++) hash_GameModeState(&gm_Mode_DebugGOver_States[i]); break;
            }
            for (i = 0; i < 16; i++) HM(gm_8049C178[i]);
            HM((u32) (int) EXB[0]);
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
