/* TU: src/melee/ft/kinds/ftCommon/ftCo_KinokoGiantStart.c */
/* C oracle for src/tu/ftCo_KinokoGiantStart.rs: the whole TU included verbatim; every callee out
 * of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static void CB0(HSD_GObj* g) { T(90); T(g == &GO); }
static void CB4(HSD_GObj* g) { T(91); T(g == &GO); }
static u32 cbid(HSD_GObjEvent e)
{
    return e == NULL ? 0 : e == CB0 ? 1 : e == CB4 ? 2 : e == ftCo_800D15D0 ? 3 : 99;
}
static u32 evid(HSD_GObjEvent e) { return e == fn_800D19BC ? 4 : e == fn_800D17FC ? 5 : 99; }

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
#define FSTUB(name, id) void name(Fighter* f) { T(id); T(f == &F); }
bool ftCo_800D27C4(Fighter* f) { T(1); T(f == &F); return SR(1) | SR(17); }
void ftCo_800D2490(Fighter_GObj* g, int a) { T(2); T(g == &GO); T(a); }
void ftCo_800D2600(Fighter_GObj* g, int a) { T(3); T(g == &GO); T(a); }
VSTUB(fn_800D2BF8, 4)
void ftCo_800D2770(Fighter_GObj* g, float s) { T(5); T(g == &GO); T(B(s)); }
VSTUB(ftCommon_8007E2FC, 6)
VSTUB(ft_8008A324, 7)
VSTUB(fn_800D2B30, 8)
void fn_800D2890(Fighter_GObj* g, int m) { T(9); T(g == &GO); T(m); }
void fn_800D299C(Fighter_GObj* g, int i, float a, float b) { T(10); T(g == &GO); T(i); T(B(a)); T(B(b)); }
FSTUB(fn_800D2AD8, 11)
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(12); T(f == &F); T(a); T(b); }
VSTUB(fn_800D290C, 13)
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(14); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
FSTUB(fn_800D2CA0, 15)
FSTUB(fn_800D2CD4, 16)
bool fn_800D2A3C(Fighter_GObj* g) { T(17); T(g == &GO); return SR(18); }
void ft_8008403C(Fighter_GObj* g, HSD_GObjEvent cb) { T(18); T(g == &GO); T(evid(cb)); }
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(19); T(g == &GO); T(evid(cb)); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(cbid(F.mv.co.mushroom.x0)); HM(cbid(F.mv.co.mushroom.x4)); HM(F.mv.co.mushroom.x8);
    HM(cbid(F.take_dmg_cb)); HM(F.x2220_b5); HM(F.x2220_b6); HM(F.x2008);
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 10; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.motion_id = 310 + rnd() % 16;
            F.x2220_b5 = rnd() & 1; F.x2220_b6 = rnd() & 1;
            F.ground_or_air = rnd() & 1;
            CD.x688 = (int) (rnd() % 300) - 50; CD.x68C = (int) (rnd() % 300) - 50; CD.x690 = (int) (rnd() % 300) - 50;
            CD.x678 = rw(); CD.x67C = rw();
            F.dmg.x1830_percent = rw();
            F.x2008 = (int) (rnd() % 1000);
            F.x34_scale.x = (rnd() & 1) ? rg() + 1.0f : rw(); F.x34_scale.y = rw();
            F.cur_anim_frame = rw(); F.frame_speed_mul = rw();
            F.mv.co.mushroom.x8 = rnd() % 3;
            F.mv.co.walk.fast_anim_frame = rw();
            F.mv.co.mushroom.x0 = CB0; F.mv.co.mushroom.x4 = CB4;
            switch (op) {
            case 0: ret = Fighter_SuperMushroomApply(&GO); break;
            case 1: ftCo_800D15D0(&GO); break;
            case 2: ftCo_800D170C(&GO); break;
            case 3: fn_800D17FC(&GO); break;
            case 4: ftCo_KinokoGiantStart_Anim(&GO); break;
            case 5: ftCo_KinokoGiantStart_Coll(&GO); break;
            case 6: ftCo_800D18CC(&GO); break;
            case 7: fn_800D19BC(&GO); break;
            case 8: ftCo_KinokoGiantStartAir_Anim(&GO); break;
            case 9: ftCo_KinokoGiantStartAir_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
