/* TU: src/melee/ft/kinds/ftCommon/ftCo_RunBrake.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_s209_fill.h"

#define VS(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define BS(name, id) bool name(Fighter_GObj* g) { T(id); T(PI(g)); return SR(id); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
void ft_PlaySFX(Fighter* f, enum_t id, u8 vol, u8 pan) { T(2); T(PI(f)); T(id); T(vol); T(pan); }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(3); T(PI(g)); T(B(r)); }
BS(ftAnim_IsFramesRemaining, 4)
VS(ft_8008A2BC, 5)
BS(fn_800CAF78, 6)
BS(fn_800C9CEC, 7)
BS(ftCo_800D5FB0, 8)
void ftCommon_CalcGroundAccel_Deaccel(Fighter* f, float v) { T(9); T(PI(f)); T(B(v)); }
VS(ftCommon_SetSelfMovementFromGroundedMovement, 10)
VS(ft_80084280, 11)

#define NOPS 6
#define NCASE 1000

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            tu_case(op, k);
            s209_fill();
            switch (op) {
            case 0: ret = ftCo_RunBrake_CheckInput(&GO); break;
            case 1: ftCo_RunBrake_Enter(&GO); break;
            case 2: ftCo_RunBrake_Anim(&GO); break;
            case 3: ftCo_RunBrake_IASA(&GO); break;
            case 4: ftCo_RunBrake_Phys(&GO); break;
            case 5: ftCo_RunBrake_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
