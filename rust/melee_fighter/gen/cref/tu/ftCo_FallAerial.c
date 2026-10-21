/* TU: src/melee/ft/kinds/ftCommon/ftCo_FallAerial.c */
/* C oracle for src/tu/ftCo_FallAerial.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_dfe_fill.h"

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCo_Fall_Anim_Inner(Fighter_GObj* g, f32* p, FtMotionId a, FtMotionId b, FtMotionId c)
{ T(2); T(g == &GO); T(p == &F.mv.co.fallaerial.x4); T(a); T(b); T(c); *p = rg(); }
void ftCo_800CC988(Fighter_GObj* g, f32 a) { T(3); T(g == &GO); T(B(a)); }
DS_CHK(ftCo_Fall_IASA_Inner, 4)
DS_V(ft_80084DB0, 5)

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 4; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            df_fill();
            switch (op) {
            case 0: ftCo_FallAerial_Enter(&GO); break;
            case 1: ftCo_FallAerial_Anim(&GO); break;
            case 2: ftCo_FallAerial_IASA(&GO); break;
            case 3: ftCo_FallAerial_Phys(&GO); break;
            }
            H = 0x811C9DC5u; df_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
