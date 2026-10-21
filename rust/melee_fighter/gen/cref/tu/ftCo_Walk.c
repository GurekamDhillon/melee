/* TU: src/melee/ft/kinds/ftCommon/ftCo_Walk.c */
/* C oracle for src/tu/ftCo_Walk.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

#define CK(name, id) bool name(Fighter_GObj* g) { T(id); T(g == &GO); return SR(id) && SR(id + 16) && SR(id + 32); }
#define VS(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
CK(ftCo_Catch_CheckInput, 1)
CK(ftCo_SpecialS_CheckInput, 2)
CK(ftCo_Attack100_CheckInput, 3)
CK(ftCo_800D6824, 4)
CK(ftCo_800D68C0, 5)
CK(ftCo_AttackS4_CheckInput, 6)
CK(ftCo_AttackHi4_CheckInput, 7)
CK(ftCo_AttackLw4_CheckInput, 8)
CK(ftCo_AttackS3_CheckInput, 9)
CK(ftCo_AttackHi3_CheckInput, 10)
CK(ftCo_AttackLw3_CheckInput, 11)
CK(ftCo_Attack1_CheckInput, 12)
CK(ftCo_80091A4C, 13)
CK(ftCo_800DE9D8, 14)
CK(ftCo_Jump_CheckInput, 15)
CK(ftCo_Dash_CheckInput, 32)
CK(ftCo_800D5FB0, 33)
CK(ft_8008A244, 34)
bool ftWalkCommon_800DFC70(Fighter_GObj* g) { T(40); T(g == &GO); return SR(40); }
void ftWalkCommon_800DFCA4(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, float d,
                           float e, float f, float h, float accel)
{ T(41); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(B(d)); T(B(e)); T(B(f)); T(B(h)); T(B(accel)); }
VS(ftWalkCommon_800DFDDC, 42)
void ftWalkCommon_800DFEC8(Fighter_GObj* g, void (*cb)(Fighter_GObj*, float))
{ T(43); T(g == &GO); if (SR(43)) cb(g, 0.5f); }
VS(ftWalkCommon_800E0060, 44)
VS(ft_80084280, 45)
static float KR;
float ftCo_CalcYScaledKnockback(float a, float s, float c) { T(46); T(B(a)); T(B(s)); T(B(c)); return KR; }

struct Fighter_804D651C_t* Fighter_804D651C;
struct Fighter_804D6520_t* Fighter_804D6520;
struct Fighter_804D6524_t* Fighter_804D6524;
static struct Fighter_804D651C_t M1;
static struct Fighter_804D6520_t M2;
static struct Fighter_804D6524_t M3;

int main(void)
{
    int op, k;
    tu_init();
    Fighter_804D651C = &M1; Fighter_804D6520 = &M2; Fighter_804D6524 = &M3;
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            M1.x0 = rw(); M2.x0 = rw(); M3.xC = rg(); KR = rw();
            CD.teeter_walk_threshold = rg();
            F.input.lstick[0].x = rg();
            F.facing_dir = rg();
            F.is_metal = rnd() & 1;
            F.x34_scale.y = (rnd() & 1) ? 1.0f : rg();
            F.x197C = (rnd() & 1) ? &GO2 : NULL;
            F.x2DC = rw(); F.x2E0 = rw(); F.x2E4 = rw();
            F.co_attrs.slow_walk_max = rw(); F.co_attrs.mid_walk_point = rw(); F.co_attrs.fast_walk_min = rw();
            switch (op) {
            case 0: ret = ftCo_Walk_CheckInput(&GO); break;
            case 1: ret = ftCo_Walk_CheckInput_Ottotto(&GO); break;
            case 2: ftCo_Walk_Enter(&GO, rw()); break;
            case 3: ftCo_Walk_Anim(&GO); break;
            case 4: ftCo_Walk_IASA(&GO); break;
            case 5: ftCo_Walk_Phys(&GO); break;
            case 6: ftCo_Walk_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, 0u, TR);
        }
    }
    return 0;
}
