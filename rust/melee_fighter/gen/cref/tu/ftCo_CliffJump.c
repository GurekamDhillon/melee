/* TU: src/melee/ft/kinds/ftCommon/ftCo_CliffJump.c */
/* C oracle for src/tu/ftCo_CliffJump.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_cliff_fill.h"

void ftCo_CliffClimb_Coll(Fighter_GObj* g) { T(42); T(g == &GO); }

static u32 run(int op)
{
    switch (op) {
    case 0: return ftCo_8009B170(&GO);
    case 1: ftCo_8009B1B8(&GO); break;
    case 2: ftCo_CliffJump1_Anim(&GO); break;
    case 3: ftCo_CliffJump1_IASA(&GO); break;
    case 4: ftCo_CliffJump1_Phys(&GO); break;
    case 5: ftCo_CliffJump1_Coll(&GO); break;
    case 6: ftCo_8009B2F8(&GO); break;
    case 7: ftCo_8009B390(&GO, ANG[1]); break;
    case 8: ftCo_CliffJump2_Anim(&GO); break;
    case 9: ftCo_CliffJump2_IASA(&GO); break;
    case 10: ftCo_CliffJump2_Phys(&GO); break;
    case 11: ftCo_CliffJump2_Coll(&GO); break;
    }
    return 0;
}
CLIFF_MAIN(12)
