/* TU: src/melee/ft/kinds/ftCommon/ftCo_CargoWalk.c */
/* C oracle for src/tu/ftCo_CargoWalk.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static ftDonkeyAttributes DK;
static float CBV;
bool ftWalkCommon_800DFC70(Fighter_GObj* g) { T(1); T(g == &GO); return SR(1); }
void ftWalkCommon_800DFCA4(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float s, float a, float b, float c, float d, float e, float f, float am)
{ T(2); T(g == &GO); T(m); T(fl); T(B(s)); T(B(a)); T(B(b)); T(B(c)); T(B(d)); T(B(e)); T(B(f)); T(B(am)); }
FtWalkType ftWalkCommon_GetWalkType(Fighter_GObj* g) { T(3); T(g == &GO); return SRV[7] % 3; }
void ftWalkCommon_800DFDDC(Fighter_GObj* g) { T(4); T(g == &GO); }
void ftWalkCommon_800DFEC8(Fighter_GObj* g, void (*cb)(Fighter_GObj*, float)) { T(5); T(g == &GO); cb(g, CBV); }
void ftWalkCommon_800E0060(Fighter_GObj* g) { T(6); T(g == &GO); }
void ftCo_8009C5A4(Fighter_GObj* g, int m) { T(7); T(g == &GO2); T(m); }
bool ftCo_8009BF3C(Fighter_GObj* g) { T(8); return SR(2); }
bool ftCo_8009BB1C(Fighter_GObj* g) { T(9); return SR(3); }
bool ftCo_8009C4F8(Fighter_GObj* g) { T(10); return SR(4); }
bool ftCo_8009B4D0(Fighter_GObj* g) { T(11); return SR(5); }
void ftCo_CargoWait_Coll(Fighter_GObj* g) { T(12); T(g == &GO); }

static u32 state_hash(void) { H = 0x811C9DC5u; HM(F.motion_id); return H; }

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&DK, sizeof DK);
            DK.x4_motion_state = rnd() % 400;
            DK.x8 = rw(); DK.xC = rw(); DK.x10 = rw(); DK.x14 = rw(); DK.x18 = rw(); DK.x1C = rw();
            CBV = rw();
            F.x2CC = &DK; F.victim_gobj = &GO2;
            switch (op) {
            case 0: ret = ftCo_8009B67C(&GO); break;
            case 1: ftCo_8009B6C8(&GO, CBV); break;
            case 2: ftCo_CargoWalk_Anim(&GO); break;
            case 3: ftCo_CargoWalk_IASA(&GO); break;
            case 4: ftCo_CargoWalk_Phys(&GO); break;
            case 5: ftCo_CargoWalk_Coll(&GO); break;
            case 6: ftCo_8009B6C8(&GO, 0); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
