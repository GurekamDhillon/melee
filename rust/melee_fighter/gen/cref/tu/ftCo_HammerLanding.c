/* TU: src/melee/ft/kinds/ftCommon/ftCo_HammerLanding.c */
/* C oracle for src/tu/ftCo_HammerLanding.rs: the whole TU included verbatim; every callee out of
 * it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static float FR;
static u32 FL;
u32 ftCo_800C54C4(Fighter* f) { T(2); T(f == &F); return FL; }
float ftCo_800C548C(Fighter* f) { T(3); T(f == &F); return FR; }
void ftCo_800C4E94(Fighter* f) { T(4); T(f == &F); }
void ftCo_800C4F64(Fighter_GObj* g) { T(5); T(g == &GO); }
void ftCo_Landing_Enter(Fighter_GObj* g, FtMotionId m, bool ai, MotionFlags fl, float a, float b)
{ T(6); T(g == &GO); T(m); T(ai); T(fl); T(B(a)); T(B(b)); }
void ftCo_800C4ED8(Fighter_GObj* g) { T(7); T(g == &GO); }
void ft_80084F3C(Fighter_GObj* g) { T(8); T(g == &GO); }
void ftCo_HammerWait_Coll(Fighter_GObj* g) { T(9); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.mv.co.hammerlanding.x4));
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
            FR = rw(); FL = rnd();
            F.mv.co.hammerlanding.x4 = rg() * 3;
            CD.x6B4 = (int) (rnd() % 64) - 8;
            switch (op) {
            case 0: ftCo_HammerLanding_Enter(&GO); break;
            case 1: ftCo_HammerLanding_Anim(&GO); break;
            case 2: ftCo_HammerLanding_IASA(&GO); break;
            case 3: ftCo_HammerLanding_Phys(&GO); break;
            case 4: ftCo_HammerLanding_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
