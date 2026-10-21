/* TU: src/melee/ft/kinds/ftCommon/ftCo_CatchCut.c */
/* C oracle for src/tu/ftCo_CatchCut.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCo_CaptureCut_Enter(Fighter_GObj* g) { T(2); T(g == &GO2); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(3); T(g == &GO); return SR(1); }
void ftCommon_8007D92C(Fighter_GObj* g) { T(4); T(g == &GO); }
void ftCommon_CalcGroundAccel_Deaccel(Fighter* fp, float f) { T(5); T(fp == &F); T(B(f)); }
void ftCommon_SetSelfMovementFromGroundedMovement(Fighter_GObj* g) { T(6); T(g == &GO); }
void ft_80084DB0(Fighter_GObj* g) { T(7); T(g == &GO); }
void ft_80084104(Fighter_GObj* g) { T(8); T(g == &GO); }
void ft_80082B1C(Fighter_GObj* g) { T(9); T(g == &GO); }
void ft_8008370C(Fighter_GObj* g, HSD_GObjEvent cb) { T(10); T(g == &GO); T(cb == ft_80082B1C); cb(g); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.self_vel.x)); HM(B(F.self_vel.y)); HM(B(F.self_vel.z)); HM(B(F.gr_vel));
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 5; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.ground_or_air = rnd() & 1;
            F.facing_dir = rg(); F.victim_gobj = &GO2;
            CD.x370 = rw(); CD.x374 = rw(); CD.x378 = rw(); CD.x64 = rg();
            F.co_attrs.ground_friction = rg();
            F.self_vel.x = rg(); F.self_vel.y = rg(); F.self_vel.z = rg(); F.gr_vel = rg();
            switch (op) {
            case 0: ftCo_800DA698(&GO, rnd() & 1); break;
            case 1: ftCo_CatchCut_Anim(&GO); break;
            case 2: ftCo_CatchCut_IASA(&GO); break;
            case 3: ftCo_CatchCut_Phys(&GO); break;
            case 4: ftCo_CatchCut_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
