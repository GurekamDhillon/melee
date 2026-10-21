/* TU: src/melee/ft/kinds/ftCommon/ftCo_AttackHi4.c */
/* C oracle for src/tu/ftCo_AttackHi4.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_attack_fill.h"

bool ftCo_800DF2D8(Fighter* f) { T(1); T(f == &F); return chk(1); }
bool ftCo_80094E54(Fighter* f) { T(2); T(f == &F); return chk(2); }
bool ftCo_800DF30C(Fighter* f) { T(3); T(f == &F); return chk(3); }
void ftCo_800957F4(Fighter_GObj* g, FtMotionId m) { T(4); T(g == &GO); T(m); }
STUB_V(ftNs_AttackHi4_Enter, 5)
void Mex_OnUSmashDispatch(int kind, void* g, void* v) { T(6); T(kind); T(g == &GO); T(v == NULL); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(7); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
STUB_V(ftAnim_8006EBA4, 8)
STUB_CHK(ftAnim_IsFramesRemaining, 9)
STUB_V(ft_8008A2BC, 10)
STUB_V(ftCo_Wait_IASA, 11)
STUB_V(ft_80084F3C, 12)
STUB_V(ft_80084104, 13)

int main(void)
{
    int op, k;
    tu_init();
    af_init();
    for (op = 0; op < 9; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            af_fill();
            switch (op) {
            case 0: ret = checkLStick(&F); break;
            case 1: ret = ftCo_AttackHi4_CheckInput(&GO); break;
            case 2: ret = checkLStickNoD0(&F); break;
            case 3: ret = ftCo_AttackHi4_CheckInputNoD0(&GO); break;
            case 4: doEnter(&GO); break;
            case 5: ftCo_AttackHi4_Anim(&GO); break;
            case 6: ftCo_AttackHi4_IASA(&GO); break;
            case 7: ftCo_AttackHi4_Phys(&GO); break;
            case 8: ftCo_AttackHi4_Coll(&GO); break;
            }
            H = 0x811C9DC5u; af_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
