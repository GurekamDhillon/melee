/* TU: src/melee/ft/kinds/ftCommon/ftCo_ItemParasolFall.c */
/* C oracle for src/tu/ftCo_ItemParasolFall.rs: the whole TU included verbatim; every callee out
 * of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
bool ftCo_SpecialAir_CheckInput(Fighter_GObj* g) { T(2); return SR(1); }
bool ftCo_80095328(Fighter_GObj* g, bool* p) { T(3); T(p == NULL); return SR(2); }
bool ftCo_800C3B10(Fighter_GObj* g) { T(4); return SR(3); }
bool ftCo_80099A58(Fighter_GObj* g) { T(5); return SR(4); }
bool ftCo_AttackAir_CheckItemThrowInput(Fighter_GObj* g) { T(6); return SR(5); }
bool ft_80087988(Fighter_GObj* g) { T(7); T(g == &GO); return SR(6); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(8); T(g == &GO); }
bool ftCo_800CB870(Fighter_GObj* g) { T(9); return SR(7); }
void ftCo_ItemParasolOpen_Phys(Fighter_GObj* g) { T(10); T(g == &GO); }
void ftCo_Landing_Enter_Basic(Fighter_GObj* g) { T(11); T(g == &GO); }
void ft_8008370C(Fighter_GObj* g, HSD_GObjEvent cb) { T(12); T(g == &GO); T(cb == ftCo_Landing_Enter_Basic); cb(g); }

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 5; op++) {
        for (k = 0; k < 2000; k++) {
            tu_case(op, k);
            F.input.lstick[0].y = rg(); CD.close_parasol_threshold = rg();
            switch (op) {
            case 0: ftCo_800CF280(&GO); break;
            case 1: ftCo_ItemParasolFall_Anim(&GO); break;
            case 2: ftCo_ItemParasolFall_IASA(&GO); break;
            case 3: ftCo_ItemParasolFall_Phys(&GO); break;
            case 4: ftCo_ItemParasolFall_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, 0, F.motion_id, TR);
        }
    }
    return 0;
}
