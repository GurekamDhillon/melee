/* TU: src/melee/ft/kinds/ftCommon/ftCo_ItemParasolDamageFall.c */
/* C oracle for src/tu/ftCo_ItemParasolDamageFall.rs: the whole TU included verbatim; every
 * callee out of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_ClampAirDrift(Fighter* f) { T(2); T(f == &F); }
void ftCo_DamageFall_IASA(Fighter_GObj* g) { T(3); T(g == &GO); }
void ft_80084DB0(Fighter_GObj* g) { T(4); T(g == &GO); }
void ftCo_80090984(Fighter_GObj* g) { T(5); T(g == &GO); }
void ft_8008370C(Fighter_GObj* g, HSD_GObjEvent cb) { T(6); T(g == &GO); T(cb == (HSD_GObjEvent) ftCo_80090984); cb(g); }

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 5; op++) {
        for (k = 0; k < 500; k++) {
            tu_case(op, k);
            switch (op) {
            case 0: ftCo_800CF4DC(&GO); break;
            case 1: ftCo_ItemParasolDamageFall_Anim(&GO); break;
            case 2: ftCo_ItemParasolDamageFall_IASA(&GO); break;
            case 3: ftCo_ItemParasolDamageFall_Phys(&GO); break;
            case 4: ftCo_ItemParasolDamageFall_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, 0, F.motion_id, TR);
        }
    }
    return 0;
}
