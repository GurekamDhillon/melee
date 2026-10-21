/* TU: src/melee/ft/kinds/ftCommon/ftCo_DownStand.c */
/* C oracle for src/tu/ftCo_DownStand.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_dfe_fill.h"

float ftCo_GetLStickAngle(Fighter* f) { T(1); T(f == &F); return rg(); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
DS_CHK(ftAnim_IsFramesRemaining, 3)
DS_V(ft_8008A2BC, 4)
DS_V(ft_80084F3C, 5)
DS_V(ft_80083F88, 6)

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            int ms;
            tu_case(op, k);
            df_fill();
            ms = rnd() % 400;
            switch (op) {
            case 0: ret = ftCo_800980BC(&GO); break;
            case 1: ftCo_80098160(&GO, ms); break;
            case 2: ftCo_DownStand_Anim(&GO); break;
            case 3: ftCo_DownStand_IASA(&GO); break;
            case 4: ftCo_DownStand_Phys(&GO); break;
            case 5: ftCo_DownStand_Coll(&GO); break;
            }
            H = 0x811C9DC5u; df_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
