/* TU: src/melee/ft/kinds/ftCommon/ftCo_SquatWait.c */
/* C oracle for src/tu/ftCo_SquatWait.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "s217_fill.h"

static int ALLNO;
static struct ftData_80085FD4_ret S_RET;
#define STUB_Q(name, id) bool name(Fighter_GObj* g) { T(id); T(PI(g)); return ALLNO ? 0 : chk(id); }
CMS_STUB
bool ftCo_8008A698(Fighter* f) { T(10); T(PI(f)); return chk(10); }
struct ftData_80085FD4_ret* ftData_80085FD4(Fighter* f, FtMotionId m) { T(11); T(PI(f)); T((u32) m); S_RET.x8 = SRV[4] & 3; return &S_RET; }
void ftCo_8008A6D8(Fighter_GObj* g, s32 a) { T(12); T(PI(g)); T((u32) a); }
void ftCommon_8007EFC0(Fighter* f, u32 v) { T(13); T(PI(f)); T(v); }
void ftLk_AttackAir_800EB3BC(HSD_GObj* g) { T(14); T(PI(g)); }
STUB_V(ftCl_Init_8014919C, 15)
void ftCo_8008A7A8(Fighter_GObj* g, WaitStruct* w) { T(16); T(PI(g)); T(PI(w)); }
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
STUB_Q(ftCo_80099F9C, 31)
STUB_Q(ftCo_Dash_CheckInput, 34)
STUB_Q(ftCo_SquatRv_CheckInput, 35)
STUB_V(ft_80084F3C, 32)
STUB_V(ftColl_8007AEE0, 36)
STUB_V(ft_80083F88, 33)

int main(void)
{
    int op, k;
    tu_init();
    s_init();
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            s_fill();
            ALLNO = SRV[2] & 1;
            switch (op) {
            case 0: ret = ftCo_SquatWait_CheckInput(&GO); break;
            case 1: fn_800D62C4(&GO); break;
            case 2: ftCo_800D638C(&GO); break;
            case 3: ftCo_SquatWait_Anim(&GO); break;
            case 4: ftCo_SquatWait_IASA(&GO); break;
            case 5: ftCo_SquatWait_Phys(&GO); break;
            case 6: ftCo_SquatWait_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
