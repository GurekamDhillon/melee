/* TU: src/melee/ft/kinds/ftCommon/ftCo_HammerWalk.c */
/* C oracle for src/tu/ftCo_HammerWalk.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static struct Fighter_804D6524_t D6524;
struct Fighter_804D6524_t* Fighter_804D6524;
static u32 isf(Fighter* f) { return f == &F; }
bool ftWalkCommon_800DFC70(Fighter_GObj* g) { T(1); T(g == &GO); return SR(1); }
void ftWalkCommon_800E0060(Fighter_GObj* g) { T(2); T(g == &GO); }
float ftCo_CalcYScaledKnockback(float a, float s, float c) { T(3); T(B(a)); T(B(s)); T(B(c)); return rw(); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(4); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(5); T(g == &GO); }
u32 ftCo_800C54C4(Fighter* f) { T(6); T(isf(f)); return rnd(); }
float ftCo_800C548C(Fighter* f) { T(7); T(isf(f)); return rw(); }
void ftCo_800C4E94(Fighter* f) { T(8); T(isf(f)); }
void ftCo_800C4F64(Fighter_GObj* g) { T(9); T(g == &GO); }
bool ftCo_800C5A50(Fighter_GObj* g) { T(10); return SR(2); }
bool ftCo_800C60C8(Fighter_GObj* g) { T(11); return SR(3); }
bool ftCo_800C4E4C(Fighter_GObj* g) { T(12); return SR(4); }
void ftCo_HammerWait_Coll(Fighter_GObj* g) { T(13); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.motion_id); HM(B(F.mv.co.walk.accel_mul)); HM(B(F.mv.co.walk.x0));
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    Fighter_804D6524 = &D6524;
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.x34_scale.y = (rnd() & 1) ? 1.0f : rg();
            F.gr_vel = rw();
            D6524.xC = rg();
            switch (op) {
            case 0: ret = ftCo_800C55CC(&GO); break;
            case 1: ftCo_800C5614(&GO); break;
            case 2: ftCo_HammerWalk_Anim(&GO); break;
            case 3: ftCo_HammerWalk_IASA(&GO); break;
            case 4: ftCo_HammerWalk_Phys(&GO); break;
            case 5: ftCo_HammerWalk_Coll(&GO); break;
            case 6: ftCo_800C5614(&GO); ftCo_800C5614(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
