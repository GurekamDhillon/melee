/* TU: src/melee/ft/kinds/ftCommon/ftCo_CargoTurn.c */
/* C oracle for src/tu/ftCo_CargoTurn.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static ftDonkeyAttributes DK;
bool ftCo_800C97A8(Fighter_GObj* g) { T(1); T(g == &GO); return SR(1); }
void ftCo_Turn_Enter(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); }
void ftCo_8009C5A4(Fighter_GObj* g, int m) { T(3); T(g == &GO2); T(m); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(4); T(g == &GO); return SR(3); }
void ftCo_8009B518(Fighter_GObj* g) { T(5); T(g == &GO); }
bool ftCo_8009BF3C(Fighter_GObj* g) { T(6); T(g == &GO); return SR(5); }
bool ftCo_8009BB1C(Fighter_GObj* g) { T(7); T(g == &GO); return SR(6); }
void ft_80084F3C(Fighter_GObj* g) { T(8); T(g == &GO); }
void ftCo_CargoWait_Coll(Fighter_GObj* g) { T(9); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.facing_dir)); HM(B(F2.facing_dir)); HM(F.mv.co.cargoturn.x0); HM(B(F.mv.co.walk.fast_anim_frame));
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&DK, sizeof DK);
            DK.x4_motion_state = rnd() % 400; DK.cargo_hold.x20_TURN_SPEED = rg();
            F.x2CC = &DK; F.victim_gobj = &GO2;
            F.mv.co.cargoturn.x0 = rnd() & 1;
            F.mv.co.walk.fast_anim_frame = rg() * 2;
            F.facing_dir = rg(); F2.facing_dir = rg();
            switch (op) {
            case 0: ret = ftCo_8009B818(&GO); break;
            case 1: ftCo_8009B860(&GO); break;
            case 2: ftCo_CargoTurn_Anim(&GO); break;
            case 3: ftCo_CargoTurn_IASA(&GO); break;
            case 4: ftCo_CargoTurn_Phys(&GO); break;
            case 5: ftCo_CargoTurn_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
