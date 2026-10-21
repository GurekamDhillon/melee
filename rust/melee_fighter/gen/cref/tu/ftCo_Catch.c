/* TU: src/melee/ft/kinds/ftCommon/ftCo_Catch.c */
/* C oracle for src/tu/ftCo_Catch.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

bool ftCo_800951D0(Fighter_GObj* g) { T(1); T(g == &GO); return SR(1); }
bool ftCo_80095254(Fighter_GObj* g) { T(2); T(g == &GO); return SR(2); }
bool ftCo_800952DC(Fighter_GObj* g) { T(3); T(g == &GO); return SR(3); }
int ScriptGame_CapsActionAllowed(Fighter* fp, int flag) { T(4); T(fp == &F); T(flag); return SR(4) ? (int) (SRV[9] % 3) : 1; }
bool fn_800D952C(Fighter_GObj* g) { T(5); T(g == &GO); return SR(5); }
bool fn_800D8EC8(Fighter_GObj* g) { T(6); T(g == &GO); return SR(6); }
bool fn_800D9228(Fighter_GObj* g) { T(7); T(g == &GO); return SR(7); }
void fn_800D949C(Fighter_GObj* g) { T(8); T(g == &GO); }
bool fn_800D9558(Fighter_GObj* g) { T(9); T(g == &GO); return SR(8); }
bool fn_800D9930(Fighter_GObj* g) { T(10); T(g == &GO); return SR(9); }
void fn_800D9C64(Fighter_GObj* g) { T(11); T(g == &GO); }
void ftpickupitem_800948A8(Fighter_GObj* g, Item_GObj* i) { T(12); T(g == &GO); T(i == (Item_GObj*) &IGO); }
void ftpickupitem_80094694(Fighter_GObj* g, FtMotionId m, bool b) { T(13); T(g == &GO); T(m); T(b); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(14); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void fn_800D9CE8(Fighter_GObj* g) { T(15); T(g == &GO); }
void fn_800DAADC(Fighter_GObj* g, Fighter_GObj* o) { T(16); T(g == &GO); T(o == &GO2); }
void ftCommon_8007E2D0(Fighter* fp, s16 a, HSD_GObjEvent g0, HSD_GObjEvent g1, HSD_GObjInteraction g2)
{
    T(17); T(fp == &F); T(a); T(g0 == fn_800D9CE8); T(g1 == fn_800D8BFC); T(g2 == fn_800DAADC);
    g0(&GO); g1(&GO); g2(&GO, &GO2);
}
void Mex_OnCatchDispatch(int kind, void* g, void* v) { T(18); T(kind); T(g == &GO); T(v == NULL); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(19); T(g == &GO); return SR(10); }
void ft_8008A2BC(Fighter_GObj* g) { T(20); T(g == &GO); }
void ftCommon_CalcGroundAccel_Deaccel(Fighter* fp, float f) { T(21); T(fp == &F); T(B(f)); }
void ftCommon_SetSelfMovementFromGroundedMovement(Fighter_GObj* g) { T(22); T(g == &GO); }
void ft_80085030(Fighter_GObj* g, float a, float b) { T(23); T(g == &GO); T(B(a)); T(B(b)); }
void ft_800841B8(Fighter_GObj* g, HSD_GObjEvent cb) { T(24); T(g == &GO); T(cb == fn_800D8E30); cb(g); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(25); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.x74_self_accel.x)); HM(B(F.x74_self_accel.y)); HM(B(F.x74_self_accel.z));
    HM((u32) F.mv.co.common.x0); HM((u32) F.mv.co.guard.x24);
    HM(F.target_item_gobj == NULL); HM(F.x1A64 == NULL);
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 15; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.kind = rnd() % 34;
            F.u.lk.xC = (rnd() & 1) ? (Item_GObj*) &IGO : NULL;
            F.input.held_buttons[0] = rnd() & 0x80000000u;
            F.input.pressed_buttons = rnd() & 0x1FF;
            F.mv.co.common.x0 = rnd() % 3;
            F.mv.co.guard.x24 = rnd() % 3;
            F.target_item_gobj = (rnd() & 1) ? (Item_GObj*) &IGO : NULL;
            F.x1A64 = (void*) &CD;
            F.victim_gobj = &GO2;
            CD.x64 = rg(); F.co_attrs.ground_friction = rg(); F.facing_dir = rg();
            F.x74_self_accel.x = rg(); F.x74_self_accel.y = rg(); F.x74_self_accel.z = rg();
            switch (op) {
            case 0: ret = ftCo_Catch_CheckInput(&GO); break;
            case 1: ret = ftCo_800D8A38(&GO); break;
            case 2: ret = ftCo_800D8AE0(&GO); break;
            case 3: ret = ftCo_800D8B9C(&GO); break;
            case 4: fn_800D8BFC(&GO); break;
            case 5: ftCo_800D8C54(&GO, rnd() % 400); break;
            case 6: ftCo_Catch_Anim(&GO); break;
            case 7: ftCo_CatchDash_Anim(&GO); break;
            case 8: ftCo_Catch_IASA(&GO); ftCo_CatchDash_IASA(&GO); break;
            case 9: ftCo_Catch_Phys(&GO); break;
            case 10: ftCo_CatchDash_Phys(&GO); break;
            case 11: ftCo_Catch_Coll(&GO); break;
            case 12: fn_800D8E30(&GO); break;
            case 13: ftCo_CatchDash_Coll(&GO); break;
            case 14: ret = fn_800D8E94(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
