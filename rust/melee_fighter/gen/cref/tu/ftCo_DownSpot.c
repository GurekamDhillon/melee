/* TU: src/melee/ft/kinds/ftCommon/ftCo_DownSpot.c */
/* C oracle for src/tu/ftCo_DownSpot.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_dfe_fill.h"

static struct ftData_80085FD4_ret DR;
struct ftData_80085FD4_ret* ftData_80085FD4(Fighter* f, FtMotionId m) { T(1); T(f == &F); T(m); return &DR; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
DS_V(ftAnim_8006EBA4, 3)
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(4); T(f == &F); T(v); }
DS_CHK(ftAnim_IsFramesRemaining, 5)
DS_V(ftCo_80097E8C, 6)
DS_V(ft_80084F3C, 7)
DS_V(ft_80083F88, 8)

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 5; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            df_fill();
            DR.x14 = SRV[4] & 1;
            switch (op) {
            case 0: ftCo_DownSpot_Enter(&GO); break;
            case 1: ftCo_DownSpot_Anim(&GO); break;
            case 2: ftCo_DownSpot_IASA(&GO); break;
            case 3: ftCo_DownSpot_Phys(&GO); break;
            case 4: ftCo_DownSpot_Coll(&GO); break;
            }
            H = 0x811C9DC5u; df_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
