/* TU: src/melee/ft/kinds/ftCommon/ftCo_ItemParasolFallSpecial.c */
/* C oracle for src/tu/ftCo_ItemParasolFallSpecial.rs: the whole TU included verbatim; every
 * callee out of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
bool ft_80087988(Fighter_GObj* g) { T(2); T(g == &GO); return SR(1); }
void ftCo_800968C8(Fighter_GObj* g) { T(3); T(g == &GO); }
bool ftCo_800CB870(Fighter_GObj* g) { T(4); return SR(2); }
void ftCo_ItemParasolOpen_Phys(Fighter_GObj* g) { T(5); T(g == &GO); }
void ftCo_LandingFallSpecial_Enter_Basic(Fighter_GObj* g) { T(6); T(g == &GO); }
void ft_8008370C(Fighter_GObj* g, HSD_GObjEvent cb) { T(7); T(g == &GO); T(cb == ftCo_LandingFallSpecial_Enter_Basic); cb(g); }

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 5; op++) {
        for (k = 0; k < 2000; k++) {
            tu_case(op, k);
            F.input.lstick[0].y = rg(); CD.close_parasol_threshold = rg();
            F.kind = (rnd() & 1) ? Ft_Kind_Peach : (FighterKind) (rnd() % 30);
            F.u.pe.parasol_gobj_0 = (rnd() & 1) ? (Item_GObj*) &IGO : NULL;
            switch (op) {
            case 0: ftCo_800CF3C8(&GO); break;
            case 1: ftCo_ItemParasolFallSpecial_Anim(&GO); break;
            case 2: ftCo_ItemParasolFallSpecial_IASA(&GO); break;
            case 3: ftCo_ItemParasolFallSpecial_Phys(&GO); break;
            case 4: ftCo_ItemParasolFallSpecial_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, 0, F.motion_id, TR);
        }
    }
    return 0;
}
