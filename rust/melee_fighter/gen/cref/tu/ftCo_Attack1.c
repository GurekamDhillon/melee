/* TU: src/melee/ft/kinds/ftCommon/ftCo_Attack1.c */
/* C oracle for src/tu/ftCo_Attack1.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_attack_fill.h"

enum_t it_8026B30C(Item_GObj* i) { T(1); T(af_item(i)); return SRV[0] % 5; }
void ftCo_800957F4(Fighter_GObj* g, FtMotionId m) { T(2); T(g == &GO); T(m); }
void ftCo_Attack_800CCF58(Fighter_GObj* g, enum_t a) { T(3); T(g == &GO); T(a); }
STUB_V(ftCo_Attack_800CDD14, 4)
STUB_V(ft_800892A0, 5)
STUB_V(ft_80089824, 6)
STUB_V(ftGw_Attack11_Enter, 7)
STUB_CHK(ftpickupitem_80094790, 8)
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(9); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
STUB_V(ftAnim_8006EBA4, 10)
STUB_CHK(ftAnim_IsFramesRemaining, 11)
STUB_V(ft_8008A2BC, 12)
STUB_CHK(ftCo_AttackS4_CheckInput, 13)
STUB_CHK(ftCo_AttackHi4_CheckInput, 14)
STUB_CHK(ftCo_AttackLw4_CheckInput, 15)
STUB_CHK(ftCo_AttackS3_CheckInput, 16)
STUB_CHK(ftCo_AttackHi3_CheckInput, 17)
STUB_CHK(ftCo_AttackLw3_CheckInput, 18)
STUB_CHK(ftCo_Attack_800D6A50, 19)
STUB_CHK(ftCo_Jump_CheckInput, 20)
STUB_CHK(ftCo_Dash_CheckInput, 21)
STUB_CHK(ftCo_800D5FB0, 22)
STUB_CHK(ftCo_Turn_CheckInput, 23)
STUB_CHK(ftCo_Walk_CheckInput, 24)
STUB_V(ftCo_Wait_IASA, 25)
STUB_V(ft_80084FA8, 26)
STUB_V(ft_80084104, 27)
int Geno_DefineTemplateSkeleton(int k) { T(28); T(k); return chk(28); }
int Geno_DefineBaseKind(int k) { T(29); T(k); return SR(2) ? Ft_Kind_Mars : k; }

int main(void)
{
    int op, k;
    tu_init();
    af_init();
    for (op = 0; op < 19; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            af_fill();
            switch (op) {
            case 0: ret = ftCo_Attack1_CheckInput(&GO); break;
            case 1: onPkPc21EC(&GO); break;
            case 2: decideAttack11(&GO); break;
            case 3: ret = getMotionFlags(&F) | ((F.x21EC == onPkPc21EC) << 8); break;
            case 4: checkAttack11(&GO); break;
            case 5: ftCo_Attack11_Anim(&GO); break;
            case 6: ftCo_Attack11_IASA(&GO); break;
            case 7: ftCo_Attack11_Phys(&GO); break;
            case 8: ftCo_Attack11_Coll(&GO); break;
            case 9: doAttack12Rapid(&GO); break;
            case 10: doAttack12Normal(&GO); break;
            case 11: doAttack12(&GO); break;
            case 12: ret = checkAttack12(&GO); break;
            case 13: ftCo_Attack12_Anim(&GO); break;
            case 14: ftCo_Attack12_IASA(&GO); break;
            case 15: doAttack13(&GO); break;
            case 16: ret = checkAttack13(&GO); break;
            case 17: ftCo_Attack13_Anim(&GO); break;
            case 18: ftCo_Attack13_IASA(&GO); break;
            }
            H = 0x811C9DC5u; af_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
