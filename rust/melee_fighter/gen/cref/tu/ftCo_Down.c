/* TU: src/melee/ft/kinds/ftCommon/ftCo_Down.c */
/* C oracle for src/tu/ftCo_Down.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static float ANG;
float ftCo_GetLStickAngle(Fighter* f) { T(1); T(f == &F); return ANG; }
bool ftCo_800DF678(Fighter* f) { T(2); T(f == &F); return SR(1); }
bool ftCo_800DF644(Fighter* f) { T(3); T(f == &F); return SR(2); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(4); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(5); T(g == &GO); }
void ftCommon_SetGroundedKnockbackIfLanded(Fighter* f) { T(6); T(f == &F); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(7); return SR(3); }
void ft_8008A2BC(Fighter_GObj* g) { T(8); T(g == &GO); }
void ft_80084FA8(Fighter_GObj* g) { T(9); T(g == &GO); }
void ft_80084104(Fighter_GObj* g) { T(10); T(g == &GO); }
void ftCo_8009856C(Fighter_GObj* g, FtMotionId m) { T(11); T(g == &GO); T(m); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.motion_id);
    return H;
}

int main(void)
{
    static const FtMotionId ms[4] = { ftCo_MS_DownWaitU, ftCo_MS_DownBoundU, ftCo_MS_DownWaitD, 0 };
    int op, k;
    tu_init();
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            ANG = rg();
            CD.x248 = rg(); CD.x20_radians = rg(); CD.x24C = rw();
            F.input.lstick[0].x = rg();
            F.input.cstick[0].x = rg();
            F.facing_dir = rg();
            F.motion_id = ms[rnd() % 4];
            F.x67C = rnd() % 20; F.x67D = rnd() % 20;
            switch (op) {
            case 0: ret = ftCo_Down_CheckInput(&GO); break;
            case 1: ftCo_80098324(&GO, rnd() % 400); break;
            case 2: ftCo_Down_Anim(&GO); break;
            case 3: ftCo_Down_IASA(&GO); break;
            case 4: ftCo_Down_Phys(&GO); break;
            case 5: ftCo_Down_Coll(&GO); break;
            case 6: ret = ftCo_80098400(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
