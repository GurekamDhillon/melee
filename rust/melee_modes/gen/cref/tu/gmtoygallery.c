/* TU: src/melee/gm/gmtoygallery.c */
/* C oracle for src/tu/gmtoygallery.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void gm_SetPendingGameMode(u8 m) { T(1); T(m); }
void gm_SetNewGameModePending(void) { T(2); }

int main(void)
{
    int k;
    for (k = 0; k < 4; k++) {
        GameModeState gs;
        GameModeState* t = gm_Mode_ToyGallery_States;
        int i;
        tu_case(0, k);
        for (i = 0; i < 2; i++) {
            T(t[i].id); T(t[i].preload); T(t[i].flags); T(t[i].on_enter != 0);
            T(t[i].info.scene_kind); T(t[i].info.enter_data != 0); T(t[i].info.exit_data != 0);
        }
        T(t[0].info.exit_data == (void*) &exit_data);
        T(toy_collection_exit_data[0] == 0 && toy_collection_exit_data[1] == 0);
        fillb(&gs, sizeof gs);
        t[0].on_exit(&gs);
        printf("%d %d %08X %08X %08X\n", 0, k, 0u, 0u, TR);
    }
    return 0;
}
