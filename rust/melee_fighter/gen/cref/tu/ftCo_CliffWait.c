/* TU: src/melee/ft/kinds/ftCommon/ftCo_CliffWait.c */
/* C oracle for src/tu/ftCo_CliffWait.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_cliff_fill.h"

bool ftCo_8009AE38(Fighter_GObj* g) { T(45); T(g == &GO); return SR(11); }
bool ftCo_8009AFD4(Fighter_GObj* g) { T(46); T(g == &GO); return SR(12); }
bool ftCo_8009B170(Fighter_GObj* g) { T(47); T(g == &GO); return SR(13); }
bool ftCo_8009AA0C(Fighter_GObj* g) { T(48); T(g == &GO); return SR(14); }

static u32 run(int op)
{
    switch (op) {
    case 0: ftCo_8009A804(&GO); break;
    case 1: ftCo_CliffWait_Anim(&GO); break;
    case 2: ftCo_CliffWait_IASA(&GO); break;
    case 3: ftCo_CliffWait_Phys(&GO); break;
    case 4: ftCo_CliffWait_Coll(&GO); break;
    case 5: return ftCo_8009A9AC(&GO);
    }
    return 0;
}
CLIFF_MAIN(6)
