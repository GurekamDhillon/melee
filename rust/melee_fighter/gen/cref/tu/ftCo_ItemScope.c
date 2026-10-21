/* TU: src/melee/ft/kinds/ftCommon/ftCo_ItemScope.c */
/* C oracle for src/tu/ftCo_ItemScope.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

FtMotionId fn_800D769C(Fighter* f, FtMotionId m) { T(1); T(f == &F); T(m); return SR(0) ? m + 8 : m; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(2); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_ClampAirDrift(Fighter* f) { T(3); }
void ftCommon_8007D7FC(Fighter* f) { T(4); }
void ftCommon_8007D5D4(Fighter* f) { T(5); }
void fn_800D8378(Fighter_GObj* g) { T(6); }
void ftCo_ItemScopeFire_Anim(Fighter_GObj* g) { T(7); T(g == &GO); }
void ftCo_800CDE54(Fighter_GObj* g) { T(8); }
void ftCo_800CDE74(Fighter_GObj* g) { T(9); }
void ft_800841B8(Fighter_GObj* g, HSD_GObjEvent cb) { T(10); T(cb == fn_800D8838); cb(g); }
void ft_80082C74(Fighter_GObj* g, HSD_GObjEvent cb) { T(11); T(cb == fn_800D87C0); cb(g); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.take_dmg_cb == NULL ? 0 : F.take_dmg_cb == fn_800D8378 ? 2 : 9);
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 12; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.motion_id = 158 + rnd() % 16;
            F.cur_anim_frame = rw(); F.frame_speed_mul = rw();
            switch (op) {
            case 0: fn_800D86E0(&GO); break;
            case 1: fn_800D874C(&GO); break;
            case 2: fn_800D87C0(&GO); break;
            case 3: fn_800D8838(&GO); break;
            case 4: ftCo_ItemScopeEnd_Anim(&GO); break;
            case 5: ftCo_ItemScopeAirEnd_Anim(&GO); break;
            case 6: ftCo_ItemScopeEnd_IASA(&GO); break;
            case 7: ftCo_ItemScopeAirEnd_IASA(&GO); break;
            case 8: ftCo_ItemScopeEnd_Phys(&GO); break;
            case 9: ftCo_ItemScopeAirEnd_Phys(&GO); break;
            case 10: ftCo_ItemScopeEnd_Coll(&GO); break;
            case 11: ftCo_ItemScopeAirEnd_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
