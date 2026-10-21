/* TU: src/melee/ft/kinds/ftCommon/ftCo_TurnRun.c */
/* C oracle for src/tu/ftCo_TurnRun.rs: the whole TU (TARGET_PC) included verbatim; every callee
 * out of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static float MV[2];
static int nmv;
int ScriptGame_CapsActionAllowed(Fighter* f, int flag) { T(1); T(f == &F); T(flag); return SR(1); }
float ScriptGame_ModValue(int slot, int field, float base) { T(2); T(slot); T(field); T(B(base)); return SR(2) ? base : MV[nmv++ & 1]; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(3); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ft_PlaySFX(Fighter* f, enum_t id, u8 a, u8 b) { T(4); T(f == &F); T(id); T(a); T(b); }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(5); T(g == &GO); T(B(r)); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(6); return SR(6); }
bool fn_800CA644(Fighter_GObj* g) { T(7); return SR(7); }
void ft_8008A2BC(Fighter_GObj* g) { T(8); T(g == &GO); }
bool fn_800CAF78(Fighter_GObj* g) { T(9); return SR(9); }
void ftCommon_CalcGroundAccel_Deaccel(Fighter* f, float fr) { T(10); T(f == &F); T(B(fr)); }
void ftCommon_SetSelfMovementFromGroundedMovement(Fighter_GObj* g) { T(11); T(g == &GO); }
bool ft_800827A0(Fighter_GObj* g) { T(12); return SR(12); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(13); T(g == &GO); }
void ftCommon_8007E2FC(Fighter_GObj* g) { T(14); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.cmd_vars[1]); HM(B(F.mv.co.turnrun.accel_mul)); HM(F.mv.co.turnrun.x14);
    HM(B(F.facing_dir)); HM(B(F.xE4_ground_accel_1));
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 4000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            nmv = 0;
            MV[0] = (rnd() & 3) ? rw() : 0.0f; MV[1] = (rnd() & 3) ? rw() : 0.0f;
            CD.x38_someLStickXThreshold = rg();
            CD.run_dash_turn_friction_multiplier = rg();
            F.input.lstick[0].x = rg();
            F.facing_dir = rg();
            F.cur_anim_frame = rw();
            F.player_id = rnd() % 6;
            F.x197C = (rnd() & 1) ? &GO2 : NULL;
            F.cmd_vars[1] = rnd() & 1;
            F.gr_vel = rw();
            F.co_attrs.dash_accel_mul = rg();
            F.co_attrs.dash_accel_base = rg(); if (F.co_attrs.dash_accel_base != F.co_attrs.dash_accel_base) F.co_attrs.dash_accel_base = 0.5f; /* NaN+NaN picks an operand by codegen order */
            F.co_attrs.dash_max_velocity = rw();
            F.co_attrs.ground_friction = rg();
            F.xE4_ground_accel_1 = rw();
            F.coll_data.env_flags = rnd() & 0x00300000;
            F.mv.co.turnrun.accel_mul = rg();
            F.mv.co.turnrun.x14 = rnd() & 1;
            F.mv.co.walk.middle_anim_frame = rg();
            switch (op) {
            case 0: ret = fn_800C9CEC(&GO); break;
            case 1: ret = fn_800C9D40(&GO); break;
            case 2: ftCo_TurnRun_Enter(&GO, rw()); break;
            case 3: ftCo_TurnRun_Anim(&GO); break;
            case 4: ftCo_TurnRun_IASA(&GO); break;
            case 5: ftCo_TurnRun_Phys(&GO); break;
            case 6: ftCo_TurnRun_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
