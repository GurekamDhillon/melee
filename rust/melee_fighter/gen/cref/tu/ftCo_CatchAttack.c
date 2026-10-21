/* TU: src/melee/ft/kinds/ftCommon/ftCo_CatchAttack.c */
/* C oracle for src/tu/ftCo_CatchAttack.rs: the whole TU included verbatim; every callee out of
 * it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_8007E2F4(Fighter* fp, s16 v) { T(2); T(fp == &F); T(v); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(3); T(g == &GO); return SR(1); }
void fn_800DA2B0(Fighter_GObj* g) { T(4); T(g == &GO); }
void fn_800DA054(Fighter_GObj* g) { T(5); T(g == &GO); }
void ftCommon_CalcGroundAccel_Deaccel(Fighter* fp, float f) { T(6); T(fp == &F); T(B(f)); }
void ftCommon_SetSelfMovementFromGroundedMovement(Fighter_GObj* g) { T(7); T(g == &GO); }
void ft_800841B8(Fighter_GObj* g, HSD_GObjEvent cb) { T(8); T(g == &GO); T(cb == fn_800DA618); cb(g); }
void ftCo_800DC920(Fighter_GObj* g, Fighter_GObj* v) { T(9); T(g == &GO); T(v == &GO2); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(10); T(g == &GO); T(g == &GO2); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.gr_vel)); HM(F.accessory1_cb == fn_800DA678); HM(F.take_dmg_cb == fn_800DA668); HM(F.take_dmg_cb == NULL);
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 8; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.victim_gobj = &GO2; F.gr_vel = rg();
            CD.x64 = rg(); F.co_attrs.ground_friction = rg();
            F.take_dmg_cb = (rnd() & 1) ? ft_80084104 : NULL;
            switch (op) {
            case 0: fn_800DA4FC(&GO); break;
            case 1: ftCo_CatchAttack_Anim(&GO); break;
            case 2: ftCo_CatchAttack_IASA(&GO); break;
            case 3: ftCo_CatchAttack_Phys(&GO); break;
            case 4: ftCo_CatchAttack_Coll(&GO); break;
            case 5: fn_800DA618(&GO); break;
            case 6: fn_800DA668(&GO); break;
            case 7: fn_800DA678(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
void ft_80084104(Fighter_GObj* g) { (void) g; }
