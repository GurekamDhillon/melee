/* TU: src/melee/ft/kinds/ftCommon/ftCo_SquatRv.c */
/* C oracle for src/tu/ftCo_SquatRv.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "s217_fill.h"

static int ALLNO;
#define STUB_Q(name, id) bool name(Fighter_GObj* g) { T(id); T(PI(g)); return ALLNO ? 0 : chk(id); }
CMS_STUB
float fabsf(float x) { union { float f; u32 u; } c; c.f = x; c.u &= 0x7FFFFFFFu; return c.f; }
int Mex_Enabled(const char* n) { T(40); T(n[0]); return SRV[3] & 1; }
STUB_CHK(ftAnim_IsFramesRemaining, 12)
STUB_V(ft_8008A2BC, 13)
STUB_Q(ftCo_800D68C0, 19)
STUB_Q(ftCo_Attack100_CheckInput, 17)
STUB_Q(ftCo_AttackS4_CheckInput, 21)
STUB_Q(ftCo_AttackHi4_CheckInput, 22)
STUB_Q(ftCo_AttackLw4_CheckInput, 23)
STUB_Q(ftCo_AttackS3_CheckInput, 24)
STUB_Q(ftCo_AttackHi3_CheckInput, 25)
STUB_Q(ftCo_AttackLw3_CheckInput, 26)
STUB_Q(ftCo_Attack1_CheckInput, 27)
STUB_Q(ftCo_80091A4C, 28)
STUB_Q(ftCo_800DE9D8, 29)
STUB_Q(ftCo_Jump_CheckInput, 30)
STUB_Q(ftCo_Walk_CheckInput, 31)
STUB_V(ft_80084F3C, 32)
STUB_V(ft_80083F88, 33)

int main(void)
{
    int op, k;
    tu_init();
    s_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            s_fill();
            ALLNO = SRV[2] & 1;
            switch (op) {
            case 0: ret = ftCo_SquatRv_CheckInput(&GO); break;
            case 1: ftCo_SquatRv_Enter(&GO); break;
            case 2: ftCo_SquatRv_Anim(&GO); break;
            case 3: ftCo_SquatRv_IASA(&GO); break;
            case 4: ftCo_SquatRv_Phys(&GO); break;
            case 5: ftCo_SquatRv_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
