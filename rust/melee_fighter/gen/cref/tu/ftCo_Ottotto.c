/* TU: src/melee/ft/kinds/ftCommon/ftCo_Ottotto.c */
/* C oracle for src/tu/ftCo_Ottotto.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_misc8_fill.h"

CMS(1)
BSTUB(ftAnim_IsFramesRemaining, 2)
BSTUB(ft_800827A0, 3)
VSTUB(ftCo_Fall_Enter, 4)
void mpFloorGetRight(int l, Vec3* p) { T(5); T(l); p->x = rw(); p->y = rg(); p->z = rg(); }
void mpFloorGetLeft(int l, Vec3* p) { T(6); T(l); p->x = rw(); p->y = rg(); p->z = rg(); }
VSTUB(ft_8008A2BC, 7)
void ft_80088770(Fighter* f) { T(8); T(PI(f)); }
void ft_800887CC(Fighter* f) { T(9); T(PI(f)); }
void ft_80088328(Fighter* f, int s, u8 v, u8 p) { T(10); T(PI(f)); T(s); T(v); T(p); }
BSTUB(ftCo_SpecialS_CheckInput, 11)
BSTUB(ftCo_Attack100_CheckInput, 12)
BSTUB(ftCo_800D6824, 13)
BSTUB(ftCo_800D68C0, 14)
BSTUB(ftCo_Catch_CheckInput, 15)
BSTUB(ftCo_AttackS4_CheckInput, 16)
BSTUB(ftCo_AttackHi4_CheckInput, 17)
BSTUB(ftCo_AttackLw4_CheckInput, 18)
BSTUB(ftCo_AttackS3_CheckInput, 19)
BSTUB(ftCo_AttackHi3_CheckInput, 20)
BSTUB(ftCo_AttackLw3_CheckInput, 21)
BSTUB(ftCo_Attack1_CheckInput, 22)
BSTUB(ftCo_80091A4C, 23)
BSTUB(ftCo_800DE9D8, 24)
BSTUB(ftCo_Jump_CheckInput, 25)
BSTUB(ftCo_Dash_CheckInput, 26)
BSTUB(ftCo_800D5FB0, 27)
BSTUB(ftCo_Turn_CheckInput, 28)
BSTUB(ftCo_Walk_CheckInput_Ottotto, 29)

#define NOPS 12
#define NCASE 2000

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            tu_case(op, k);
            m8_fill();
            switch (op) {
            case 0: ret = ftCo_8009A3C8(&GO); break;
            case 1: ftCo_8009A410(&GO); break;
            case 2: ftCo_Ottotto_Anim(&GO); break;
            case 3: ftCo_Ottotto_IASA(&GO); break;
            case 4: ftCo_Ottotto_Phys(&GO); break;
            case 5: ftCo_Ottotto_Coll(&GO); break;
            case 6: ftCo_8009A6B8(&GO); break;
            case 7: ftCo_OttottoWait_Anim(&GO); break;
            case 8: ftCo_OttottoWait_IASA(&GO); break;
            case 9: ftCo_OttottoWait_Phys(&GO); break;
            case 10: ftCo_OttottoWait_Coll(&GO); break;
            case 11: ftCo_Ottotto_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
