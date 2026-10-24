/* TU: src/melee/ft/ftcommon.c */
/* C oracle for ftCo_GetLStickAngle (src/ftcommon_c.rs -> ftcommon3): ftcommon.c whole with lbtrigf.c's atan2f.
 * Each line: case result-bits trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftcommon_traps.h"
#include "ftcommon_hand.h"
#include <melee/lb/lbtrigf.c>

void ftPartSetRotZ(Fighter* fp, int part, float v) { (void) fp; (void) part; (void) v; }

int main(void)
{
    int k;
    tu_init();
    for (k = 0; k < 20000; k++) {
        float r;
        tu_case(0, k);
        F.input.lstick[0].x = (rnd() & 1) ? rw() : rg();
        F.input.lstick[0].y = (rnd() & 1) ? rw() : rg();
        r = ftCo_GetLStickAngle(&F);
        printf("%d %08X %08X\n", k, B(r), TR);
    }
    return 0;
}
