/* TU: src/melee/ft/kinds/ftCommon/ftCo_HammerKneeBend.c */
/* C oracle for src/tu/ftCo_HammerKneeBend.rs: the whole TU included verbatim; every callee out of
 * it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static float FR;
static u32 FL;
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
u32 ftCo_800C54C4(Fighter* f) { T(2); T(f == &F); return FL; }
float ftCo_800C548C(Fighter* f) { T(3); T(f == &F); return FR; }
void ftCo_800C4E94(Fighter* f) { T(4); T(f == &F); }
void ftCo_800C4F64(Fighter_GObj* g) { T(5); T(g == &GO); }
void ftCo_800C5A98(Fighter_GObj* g) { T(6); T(g == &GO); }
void ftCo_KneeBend_Check_ShortHop(Fighter_GObj* g) { T(7); T(g == &GO); }
void ft_80084F3C(Fighter_GObj* g) { T(8); T(g == &GO); }
void ftCo_HammerWait_Coll(Fighter_GObj* g) { T(9); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.mv.co.hammerkneebend.x0); HM(F.mv.co.hammerkneebend.x4); HM(B(F.mv.co.hammerkneebend.x8));
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
            F.mv.co.hammerkneebend.x0 = rnd(); F.mv.co.hammerkneebend.x4 = rnd();
            F.mv.co.hammerkneebend.x8 = rg() * 3;
            F.co_attrs.jump_startup_time = rw();
            switch (op) {
            case 0: ftCo_800C5B88(&GO, (int) rnd()); break;
            case 1: ftCo_HammerKneeBend_Anim(&GO); break;
            case 2: ftCo_HammerKneeBend_IASA(&GO); break;
            case 3: ftCo_HammerKneeBend_Phys(&GO); break;
            case 4: ftCo_HammerKneeBend_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
