/* TU: src/melee/ft/kinds/ftCommon/ftCo_KneeBend.c */
/* C oracle for src/tu/ftCo_KneeBend.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(2); T(g == &GO); return SR(2); }
bool ft_800D2D0C(Fighter_GObj* g) { T(3); T(g == &GO); return SR(3); }
void ftCo_ItemScrew_Enter(Fighter_GObj* g) { T(4); T(g == &GO); }
void ftCo_Jump_Enter(Fighter_GObj* g) { T(5); T(g == &GO); }
bool ftCo_Attack100_CheckInput(Fighter_GObj* g) { T(6); T(g == &GO); return SR(6) & SR(22); }
bool ftCo_Catch_CheckInput(Fighter_GObj* g) { T(7); T(g == &GO); return SR(7) & SR(23); }
bool ftCo_AttackHi4_CheckInputNoD0(Fighter_GObj* g) { T(8); T(g == &GO); return SR(8) & SR(24); }
void ft_80084F3C(Fighter_GObj* g) { T(9); T(g == &GO); }
void ft_80083F88(Fighter_GObj* g) { T(10); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.mv.co.kneebend.jump_input); HM(F.mv.co.kneebend.is_short_hop);
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            int ji;
            tu_case(op, k);
            F.cur_anim_frame = rw(); F.co_attrs.jump_startup_time = rw();
            F.input.held_buttons[0] = rnd() & 0xC00;
            F.input.lstick[0].y = rg(); F.input.cstick[0].y = rg();
            CD.tap_jump_release_threshold = rg();
            F.mv.co.kneebend.jump_input = rnd() % 5;
            F.mv.co.kneebend.is_short_hop = rnd() & 1;
            ji = rnd() % 5;
            switch (op) {
            case 0: ftCo_KneeBend_Enter(&GO, ji); break;
            case 1: ftCo_KneeBend_Anim(&GO); break;
            case 2: ftCo_KneeBend_Check_ShortHop(&GO); break;
            case 3: ftCo_KneeBend_IASA(&GO); break;
            case 4: ftCo_KneeBend_Phys(&GO); break;
            case 5: ftCo_KneeBend_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
