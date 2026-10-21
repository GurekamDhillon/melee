/* TU: src/melee/ft/kinds/ftCommon/ftCo_KinokoGiantEnd.c */
/* C oracle for src/tu/ftCo_KinokoGiantEnd.rs: the whole TU included verbatim; every callee out of
 * it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

bool fn_800D2818(Fighter* f) { T(1); T(f == &F); return SR(0); }
void fn_800D2BF8(Fighter_GObj* g) { T(2); }
void ftCo_800D2770(Fighter_GObj* g, f32 s) { T(3); T(B(s)); }
void ftCommon_8007E2FC(Fighter_GObj* g) { T(4); T(g == &GO); }
void ft_8008A324(Fighter_GObj* g) { T(5); }
void fn_800D2B30(Fighter_GObj* g) { T(6); }
void fn_800D2890(Fighter_GObj* g, int id) { T(7); T(id); }
void fn_800D299C(Fighter_GObj* g, int a, f32 b, f32 c) { T(8); T(a); T(B(b)); T(B(c)); }
void fn_800D2B04(Fighter* f) { T(9); }
void fn_800D290C(Fighter_GObj* g) { T(10); }
void fn_800D2CA0(Fighter* f) { T(11); }
void fn_800D2CD4(Fighter* f) { T(12); }
bool fn_800D2A3C(Fighter_GObj* g) { T(13); return SR(1); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(14); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ft_8008403C(Fighter_GObj* g, HSD_GObjEvent cb) { T(15); T(cb == fn_800D1DB0); cb(g); }
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(16); T(cb == fn_800D1C40); cb(g); }
static void MUSH(Fighter_GObj* g) { T(17); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.x2220_b5); HM(F.mv.co.mushroom.x8);
    HM(F.mv.co.mushroom.x4 == NULL ? 0 : F.mv.co.mushroom.x4 == ftCo_800D1B04 ? 1 : 9);
    HM(F.take_dmg_cb == NULL ? 0 : F.take_dmg_cb == ftCo_800D1B04 ? 1 : 9);
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 10; op++) {
        for (k = 0; k < 1000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.ground_or_air = rnd() & 1;
            F.motion_id = rnd() % 400;
            F.x34_scale.x = rw(); F.x34_scale.y = rw();
            F.cur_anim_frame = rw(); F.frame_speed_mul = rw();
            F.mv.co.mushroom.x8 = rnd() & 1;
            F.mv.co.mushroom.x0 = MUSH;
            F.x2220_b5 = 1;
            switch (op) {
            case 0: ret = Fighter_SuperMushroomEnd(&GO); break;
            case 1: ftCo_800D1B04(&GO); break;
            case 2: { int a = rnd() % 3; ftCo_800D1BA0(&GO, a); break; }
            case 3: fn_800D1C40(&GO); break;
            case 4: ftCo_KinokoGiantEnd_Anim(&GO); break;
            case 5: ftCo_KinokoGiantEnd_Coll(&GO); break;
            case 6: { int a = rnd() % 3; ftCo_800D1D10(&GO, a); break; }
            case 7: fn_800D1DB0(&GO); break;
            case 8: ftCo_KinokoGiantEndAir_Anim(&GO); break;
            case 9: ftCo_KinokoGiantEndAir_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
