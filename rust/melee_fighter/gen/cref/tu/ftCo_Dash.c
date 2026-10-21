/* TU: src/melee/ft/kinds/ftCommon/ftCo_Dash.c */
/* C oracle for src/tu/ftCo_Dash.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static float FR, MV[2];
static int nmv;
int ScriptGame_CapsActionAllowed(Fighter* f, int flag) { T(1); T(f == &F); T(flag); return SR(20); }
float ScriptGame_ModValue(int slot, int field, float base) { T(2); T(slot); T(field); T(B(base)); return MV[nmv++ & 1]; }
void ftCo_Turn_Enter_Smash(Fighter_GObj* g) { T(3); T(g == &GO); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(4); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(5); T(g == &GO); }
void ftCommon_800804A0(Fighter* f, float v) { T(6); T(f == &F); T(B(v)); }
void ft_PlaySFX(Fighter* f, enum_t id, u8 a, u8 b) { T(7); T(f == &F); T(id); T(a); T(b); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(8); return SR(3); }
void ft_8008A2BC(Fighter_GObj* g) { T(9); T(g == &GO); }
bool ftCo_SpecialS_CheckInput(Fighter_GObj* g) { T(10); return SR(4); }
bool ftCo_80094E54(Fighter* f) { T(11); T(f == &F); return SR(5); }
void ftCo_800957F4(Fighter_GObj* g, FtMotionId m) { T(12); T(g == &GO); T(m); }
bool ftCo_800D8A38(Fighter_GObj* g) { T(13); return SR(6); }
bool ftCo_AttackS4_8008C114(Fighter_GObj* g) { T(14); return SR(7); }
bool ftCo_80099264(Fighter_GObj* g) { T(15); return SR(8); }
bool ftCo_AttackDash_CheckInput(Fighter_GObj* g) { T(16); return SR(9); }
void ftCo_AttackDash_SetMv0(Fighter_GObj* g) { T(17); T(g == &GO); }
bool ftCo_80091AD8(Fighter_GObj* g, int n) { T(18); T(n); return SR(10); }
bool ftCo_80091A4C(Fighter_GObj* g) { T(19); return SR(11); }
void ftCo_80091B9C(Fighter_GObj* g) { T(20); T(g == &GO); }
bool ftCo_800DE9D8(Fighter_GObj* g) { T(21); return SR(12); }
bool fn_800CAF78(Fighter_GObj* g) { T(22); return SR(13); }
bool fn_800CA5F0(Fighter_GObj* g) { T(23); return SR(14); }
float ft_GetGroundFrictionMultiplier(Fighter* f) { T(24); T(f == &F); return FR; }
void ftCommon_CalcGroundAccel_DashRun(Fighter* f, float a, float t, float fr) { T(25); T(f == &F); T(B(a)); T(B(t)); T(B(fr)); }
void ftCommon_SetSelfMovementFromGroundedMovement(Fighter_GObj* g) { T(26); T(g == &GO); }
void ft_800844EC(Fighter_GObj* g) { T(27); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.cmd_vars[0]); HM(F.active_timer.lstick.x); HM(B(F.mv.co.dash.x0)); HM(F.mv.co.dash.x4);
    HM(B(F.gr_vel));
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            nmv = 0;
            FR = rg(); MV[0] = rw(); MV[1] = rw();
            CD.dash_smash_stick_threshold = rg(); CD.dash_smash_window = rnd() % 8;
            CD.x44 = rg(); CD.x48 = rg(); CD.x4C = rw(); CD.x54 = rg();
            CD.run_dash_turn_friction_multiplier = rg();
            F.input.lstick[0].x = rg();
            F.facing_dir = rg();
            F.active_timer.lstick.x = rnd() % 10;
            F.co_attrs.dash_initial_velocity = rw(); F.co_attrs.dash_accel_mul = rg();
            F.co_attrs.dash_accel_base = rg(); if (F.co_attrs.dash_accel_base != F.co_attrs.dash_accel_base) F.co_attrs.dash_accel_base = 0.5f; /* NaN+NaN picks an operand by codegen order */ F.co_attrs.dash_max_velocity = rw();
            F.co_attrs.ground_friction = rg();
            F.gr_vel = rw();
            F.player_id = rnd() % 6;
            F.x197C = (rnd() & 1) ? &GO2 : NULL;
            F.item_gobj = (rnd() & 1) ? (Item_GObj*) &IGO : NULL;
            F.cmd_vars[0] = rnd() & 1;
            F.cur_anim_frame = rw();
            F.mv.co.dash.x0 = (rnd() & 1) ? rg() : 0.0f;
            F.mv.co.dash.x4 = rnd() & 1;
            switch (op) {
            case 0: ret = ftCo_Dash_CheckInput(&GO); break;
            case 1: ftCo_Dash_Enter(&GO, rnd() % 3); break;
            case 2: ftCo_Dash_Anim(&GO); break;
            case 3: ftCo_Dash_IASA(&GO); break;
            case 4: ftCo_Dash_Phys(&GO); break;
            case 5: ftCo_Dash_Coll(&GO); break;
            case 6: ftCo_Dash_IASA(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
