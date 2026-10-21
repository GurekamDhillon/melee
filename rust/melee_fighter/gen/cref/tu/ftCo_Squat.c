/* TU: src/melee/ft/kinds/ftCommon/ftCo_Squat.c */
/* C oracle for src/tu/ftCo_Squat.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "s217_fill.h"

static int ALLNO;
#define STUB_Q(name, id) bool name(Fighter_GObj* g) { T(id); T(PI(g)); return ALLNO ? 0 : chk(id); }
CMS_STUB
STUB_V(ftAnim_8006EBA4, 10)
void ftCommon_8007EFC0(Fighter* f, u32 v) { T(11); T(PI(f)); T(v); }
STUB_CHK(ftAnim_IsFramesRemaining, 12)
STUB_V(ftCo_800D638C, 13)
bool mpColl_IsOnPlatform(CollData* c) { T(14); T(PI(c)); return chk(14); }
STUB_V(ftCo_8009A228, 15)
STUB_Q(ftCo_SpecialS_CheckInput, 16)
STUB_Q(ftCo_Attack100_CheckInput, 17)
STUB_Q(ftCo_800D6824, 18)
STUB_Q(ftCo_800D68C0, 19)
STUB_Q(ftCo_Catch_CheckInput, 20)
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
STUB_Q(ftCo_80099F9C, 31)
STUB_V(ft_80084F3C, 32)
STUB_V(ft_80083F88, 33)

int main(void)
{
    int op, k;
    tu_init();
    s_init();
    for (op = 0; op < 8; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            s_fill();
            ALLNO = SRV[2] & 1;
            F.mv.co.squat.x0 = rnd() & 1;
            F.mv.co.squat.x4 = (rnd() & 1) ? 1.0f : rg();
            switch (op) {
            case 0: ret = ftCo_Squat_CheckInput(&GO); break;
            case 1: ret = fn_800D5F84(&GO); break;
            case 2: ret = ftCo_800D5FB0(&GO); break;
            case 3: ftCo_Squat_Enter(&GO); break;
            case 4: ftCo_Squat_Anim(&GO); break;
            case 5: ftCo_Squat_IASA(&GO); break;
            case 6: ftCo_Squat_Phys(&GO); break;
            case 7: ftCo_Squat_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
