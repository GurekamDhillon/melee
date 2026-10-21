/* TU: src/melee/ft/ftcommon.c */
/* C oracle for src/tu/ftcommon.rs (mirrored by tests/tu_ftcommon.rs): ftCommon_8007E5AC over
 * generated floor normals (MSL atan2f), ftPartSetRotZ recorded. Each line: case hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftcommon_traps.h"
#include "ftcommon_hand.h"
/* the game links lbtrigf.c's atan2f (not the CRT's) */
#include <melee/lb/lbtrigf.c>

void ftPartSetRotZ(Fighter* fp, int part, float v) { T(1); T(fp == &F); T(part); T(B(v)); }

int main(void)
{
    int k;
    tu_init();
    for (k = 0; k < 20000; k++) {
        tu_case(0, k);
        F.ground_or_air = GA_Ground;
        F.coll_data.floor.normal.x = (rnd() & 1) ? rw() : rg();
        F.coll_data.floor.normal.y = (rnd() & 1) ? rw() : rg();
        ftCommon_8007E5AC(&F);
        printf("%d %08X\n", k, TR);
    }
    return 0;
}
