/* TU: src/melee/ft/kinds/ftCommon/ftCo_StopWall.c */
/* C oracle for src/tu/ftCo_StopWall.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "s217_fill.h"

CMS_STUB
void ftKb_SpecialN_800F1F1C(Fighter_GObj* g, Vec3* v) { T(10); T(PI(g)); T(B(v->x)); T(B(v->y)); T(B(v->z)); if (chk(10)) v->x = v->x + 1.0f; }
STUB_V(ft_800843FC, 11)
STUB_V(ftCommon_8007E2FC, 12)
STUB_CHK(ftAnim_IsFramesRemaining, 13)
STUB_V(ft_8008A2BC, 14)

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
            switch (op) {
            case 0: ret = ftCo_8009EDA4(&GO); break;
            case 1: ftCo_8009EE30(&GO); break;
            case 2: ftCo_StopWall_Anim(&GO); break;
            case 3: ftCo_StopWall_IASA(&GO); break;
            case 4: ftCo_StopWall_Phys(&GO); break;
            case 5: ftCo_StopWall_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
