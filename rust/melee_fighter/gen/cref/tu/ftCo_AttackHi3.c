/* TU: src/melee/ft/kinds/ftCommon/ftCo_AttackHi3.c */
/* C oracle for src/tu/ftCo_AttackHi3.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_attack_fill.h"

float ftCo_GetLStickAngle(Fighter* f) { T(1); T(f == &F); return angle_of(SRV[11]); }
bool ftCo_80094E54(Fighter* f) { T(2); T(f == &F); return chk(2); }
void ftCo_800957F4(Fighter_GObj* g, FtMotionId m) { T(3); T(g == &GO); T(m); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(4); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
STUB_V(ftAnim_8006EBA4, 5)
STUB_CHK(ftAnim_IsFramesRemaining, 6)
STUB_V(ft_8008A2BC, 7)
STUB_V(ftCo_Wait_IASA, 8)
STUB_V(ft_80084F3C, 9)
STUB_V(ft_80084104, 10)

int main(void)
{
    int op, k;
    tu_init();
    af_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            af_fill();
            switch (op) {
            case 0: ret = ftCo_AttackHi3_CheckInput(&GO); break;
            case 1: doEnter(&GO); break;
            case 2: ftCo_AttackHi3_Anim(&GO); break;
            case 3: ftCo_AttackHi3_IASA(&GO); break;
            case 4: ftCo_AttackHi3_Phys(&GO); break;
            case 5: ftCo_AttackHi3_Coll(&GO); break;
            }
            H = 0x811C9DC5u; af_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
