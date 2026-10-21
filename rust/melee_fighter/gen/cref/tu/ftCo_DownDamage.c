/* TU: src/melee/ft/kinds/ftCommon/ftCo_DownDamage.c */
/* C oracle for src/tu/ftCo_DownDamage.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "ftco_dfe_fill.h"

void ftCo_8008DCE0(Fighter_GObj* g, int m, float fd) { T(1); T(g == &GO); T(m); T(B(fd)); }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(2); T(f == &F); T(v); }
DS_CHK(ftAnim_IsFramesRemaining, 3)
DS_V(ftCo_80090780, 4)
DS_V(ftCo_Fall_Enter, 5)
void ftCo_80098160(Fighter_GObj* g, FtMotionId m) { T(6); T(g == &GO); T(m); }
DS_V(ftCo_80097F38, 7)
DS_V(ftCo_Damage_Phys, 8)
GroundOrAir ft_80082708(Fighter_GObj* g) { T(9); T(g == &GO); return chk(9); }
DS_V(ftCo_8008FC94, 10)
DS_CHK(ftCo_800C7CA0, 11)
DS_CHK(ft_80081DD4, 12)
DS_FV(ftCommon_8007D7FC, 13)
DS_CHK(ftCo_800C1D38, 14)
DS_CHK(ftCo_800C17CC, 15)

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            df_fill();
            switch (op) {
            case 0: ret = ftCo_8009F0F0(&GO); break;
            case 1: ftCo_8009F184(&GO); break;
            case 2: ftCo_DownDamage_Anim(&GO); break;
            case 3: ftCo_DownDamage_IASA(&GO); break;
            case 4: ftCo_DownDamage_Phys(&GO); break;
            case 5: ftCo_DownDamage_Coll(&GO); break;
            }
            H = 0x811C9DC5u; df_hash();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
