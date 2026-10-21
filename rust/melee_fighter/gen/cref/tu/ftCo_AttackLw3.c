/* TU: src/melee/ft/kinds/ftCommon/ftCo_AttackLw3.c */
/* C oracle for src/tu/ftCo_AttackLw3.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret(^first mv word, x21EC==callUnk in bit 31) state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_attack_fill.h"

float ftCo_GetLStickAngle(Fighter* f) { T(1); T(f == &F); return angle_of(SRV[11]); }
bool ftCo_80094E54(Fighter* f) { T(2); T(f == &F); return chk(2); }
void ftCo_800957F4(Fighter_GObj* g, FtMotionId m) { T(3); T(g == &GO); T(m); }
STUB_V(ft_800892A0, 4)
STUB_V(ft_80089824, 5)
STUB_V(ftGw_AttackLw3_Enter, 6)
STUB_CHK(ftpickupitem_80094790, 7)
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(8); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
STUB_V(ftAnim_8006EBA4, 9)
STUB_CHK(ftAnim_IsFramesRemaining, 10)
STUB_V(ftCo_800D638C, 11)
STUB_CHK(ftCo_AttackS4_CheckInput, 12)
STUB_CHK(ftCo_AttackHi4_CheckInput, 13)
STUB_CHK(ftCo_AttackLw4_CheckInput, 14)
STUB_CHK(ftCo_AttackS3_CheckInput, 15)
STUB_CHK(ftCo_AttackHi3_CheckInput, 16)
STUB_CHK(ftCo_Attack1_CheckInput, 17)
STUB_CHK(ftCo_Jump_CheckInput, 18)
STUB_CHK(ftCo_Dash_CheckInput, 19)
STUB_CHK(ftCo_Squat_CheckInput, 20)
STUB_CHK(ftCo_Turn_CheckInput, 21)
STUB_CHK(ftCo_Walk_CheckInput, 22)
STUB_V(ft_80084F3C, 23)
STUB_V(ft_80084104, 24)

int main(void)
{
    int op, k;
    tu_init();
    af_init();
    for (op = 0; op < 11; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            af_fill();
            *(u32*) &F.mv = rnd() & 1;
            switch (op) {
            case 0: ret = ftCo_AttackLw3_CheckInput(&GO); break;
            case 1: callUnk(&GO); break;
            case 2: decideFighter(&GO); break;
            case 3: doEnter(&GO); break;
            case 4: wrapper(&GO); break;
            case 5: ftCo_AttackLw3_Anim(&GO); break;
            case 6: ret = checkPadA(&GO); break;
            case 7: ret = checkItemThrowInput(&GO); break;
            case 8: ftCo_AttackLw3_IASA(&GO); break;
            case 9: ftCo_AttackLw3_Phys(&GO); break;
            case 10: ftCo_AttackLw3_Coll(&GO); break;
            }
            ret ^= *(u32*) &F.mv;
            ret ^= (u32) (F.x21EC == callUnk) << 31;
            H = 0x811C9DC5u; af_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
