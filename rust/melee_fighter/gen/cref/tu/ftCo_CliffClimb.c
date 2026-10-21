/* TU: src/melee/ft/kinds/ftCommon/ftCo_CliffClimb.c */
/* C oracle for src/tu/ftCo_CliffClimb.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_cliff_fill.h"

void ftCo_8009AE14(Fighter_GObj* g) { T(44); T(g == &GO); }

static u32 run(int op)
{
    switch (op) {
    case 0: return ftCo_8009AA0C(&GO);
    case 1: return ftCo_8009AAFC(&GO, SR(10), F.input.cstick[0].x, ANG[0]);
    case 2: ftCo_8009AB9C(&GO); break;
    case 3: ftCo_CliffClimb_Anim(&GO); break;
    case 4: ftCo_CliffClimb_IASA(&GO); break;
    case 5: ftCo_CliffClimb_Phys(&GO); break;
    case 6: ftCo_CliffClimb_Coll(&GO); break;
    }
    return 0;
}
CLIFF_MAIN(7)
