/* TU: src/melee/ft/kinds/ftCommon/ftCo_Wait.c */
/* C oracle for src/tu/ftCo_Wait.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

#define CK(name, id) bool name(Fighter_GObj* g) { T(id); T(g == &GO); return SR(id) && SR(id + 16) && SR(id + 32); }
#define VS(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
CK(ftCo_SpecialS_CheckInput, 1)
CK(ftCo_Attack100_CheckInput, 2)
CK(ftCo_800D6824, 3)
CK(ftCo_800D68C0, 4)
CK(ftCo_Catch_CheckInput, 5)
CK(ftCo_AttackS4_CheckInput, 6)
CK(ftCo_AttackHi4_CheckInput, 7)
CK(ftCo_AttackLw4_CheckInput, 8)
CK(ftCo_AttackS3_CheckInput, 9)
CK(ftCo_AttackHi3_CheckInput, 10)
CK(ftCo_AttackLw3_CheckInput, 11)
CK(ftCo_Attack1_CheckInput, 12)
CK(ftCo_80099794, 13)
CK(ftCo_80091A4C, 14)
CK(ftFx_AppealS_CheckInput, 15)
CK(ftCo_800DE9D8, 32 + 0)
CK(ftCo_Jump_CheckInput, 32 + 1)
CK(ftCo_Dash_CheckInput, 32 + 2)
CK(ftCo_800D5FB0, 32 + 3)
CK(ftCo_Turn_CheckInput, 32 + 4)
CK(ftCo_Walk_CheckInput, 32 + 5)
VS(ftCo_DownSpot_Enter, 40)
void ftCo_8008A7A8(Fighter_GObj* g, WaitStruct* ws) { T(41); T(g == &GO); T(ws == (void*) &IGO); }
VS(ft_80084F3C, 42)
VS(ftColl_8007AEE0, 43)
VS(ft_80084280, 44)

static ftData FD;

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 4; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&FD, sizeof FD);
            FD.x24 = (void*) &IGO;
            F.ft_data = &FD;
            F.x2224_b2 = rnd() & 1;
            switch (op) {
            case 0: ftCo_Wait_Anim(&GO); break;
            case 1: ftCo_Wait_IASA(&GO); break;
            case 2: ftCo_Wait_Phys(&GO); break;
            case 3: ftCo_Wait_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, 0u, TR);
        }
    }
    return 0;
}
