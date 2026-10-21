/* TU: src/melee/ft/kinds/ftCommon/ftCo_CliffEscape.c */
/* C oracle for src/tu/ftCo_CliffEscape.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_cliff_fill.h"

void ftCo_CliffClimb_Anim(Fighter_GObj* g) { T(40); T(g == &GO); }
void ftCo_CliffClimb_Phys(Fighter_GObj* g) { T(41); T(g == &GO); }
void ftCo_CliffClimb_Coll(Fighter_GObj* g) { T(42); T(g == &GO); }

static u32 run(int op)
{
    switch (op) {
    case 0: ftCo_8009B040(&GO); break;
    case 1: ftCo_CliffEscape_Anim(&GO); break;
    case 2: ftCo_CliffEscape_IASA(&GO); break;
    case 3: ftCo_CliffEscape_Phys(&GO); break;
    case 4: ftCo_CliffEscape_Coll(&GO); break;
    }
    return 0;
}
CLIFF_MAIN(5)
