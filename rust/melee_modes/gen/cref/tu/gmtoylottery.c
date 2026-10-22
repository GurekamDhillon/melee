/* TU: src/melee/gm/gmtoylottery.c */
/* C oracle for src/tu/gmtoylottery.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void gm_SetPendingGameMode(u8 m) { T(1); T(m); }
void gm_SetNewGameModePending(void) { T(2); }
void lbCardNew_AllocWorkArea(void) { T(3); }
void lbCardGame_LoadArchive(int a) { T(4); T(a); }
int gm_80172898(u16 a) { T(5); T(a); return 7; }
bool gm_80173754(u8 a, u8 b) { T(6); T(a); T(b); return SR(3); }

int main(void)
{
    int op, k;
    for (op = 0; op < 2; op++) {
        for (k = 0; k < 40; k++) {
            GameModeState gs;
            GameModeState* t = gm_Mode_ToyLottery_States;
            int i;
            tu_case(op, k);
            if (op == 0 && k == 0) {
                for (i = 0; i < 2; i++) {
                    T(t[i].id); T(t[i].preload); T(t[i].flags); T(t[i].on_enter != 0); T(t[i].on_exit != 0);
                    T(t[i].info.scene_kind); T(t[i].info.enter_data != 0); T(t[i].info.exit_data != 0);
                }
                T(t[0].info.exit_data == (void*) &exit_data);
            }
            fillb(&gs, sizeof gs);
            if (op == 0) t[0].on_enter(&gs); else t[0].on_exit(&gs);
            printf("%d %d %08X %08X %08X\n", op, k, 0u, 0u, TR);
        }
    }
    return 0;
}
