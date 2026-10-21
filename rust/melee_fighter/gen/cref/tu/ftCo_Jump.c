/* TU: src/melee/ft/kinds/ftCommon/ftCo_Jump.c */
/* C oracle for src/tu/ftCo_Jump.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static ftData FD;
static FtSFX SFX;
static HSD_GObj X197C;

int FighterControls_TapJumpOff(int port) { T(1); T(port); return SR(0); }
float ScriptGame_ModJump(int s, int f, float b) { T(2); T(s); T(f); T(B(b)); return SR(1) ? b * 2.0f : b; }
bool ftCo_800C5240(Fighter_GObj* g) { T(3); T(g == &GO); return SR(2); }
bool ftCo_800C5A50(Fighter_GObj* g) { T(4); T(g == &GO); return SR(3); }
void ftCo_KneeBend_Enter(Fighter_GObj* g, ftCo_JumpInput i) { T(5); T(g == &GO); T(i); }
bool ftCo_800DF910(Fighter* f) { T(6); T(f == &F); return SR(4); }
void ft_800881D8(Fighter* f, int id, u8 v, u8 p) { T(7); T(f == &F); T(id); T(v); T(p); }
void ft_PlaySFX(Fighter* f, enum_t id, u8 v, u8 p) { T(8); T(f == &F); T(id); T(v); T(p); }
void ftCommon_8007D5D4(Fighter* f) { T(9); T(f == &F); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(10); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(11); T(g == &GO); return SR(5); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(12); T(g == &GO); }
bool ftCo_SpecialAir_CheckInput(Fighter_GObj* g) { T(13); return SR(6); }
bool ftCo_80095328(Fighter_GObj* g, bool* p) { T(14); T(p == NULL); return SR(7); }
bool ftCo_800D7100(Fighter_GObj* g) { T(15); return SR(8); }
bool ftCo_800C3B10(Fighter_GObj* g) { T(16); return SR(9); }
bool ftCo_80099A58(Fighter_GObj* g) { T(17); return SR(10); }
bool ftCo_AttackAir_CheckItemThrowInput(Fighter_GObj* g) { T(18); return SR(11); }
bool ftCo_800D705C(Fighter_GObj* g) { T(19); return SR(12); }
bool ftCo_800CEE70(Fighter_GObj* g) { T(20); return SR(13); }
bool ftPe_8011BA54(HSD_GObj* g) { T(21); return SR(14); }
bool ftCo_800CB870(Fighter_GObj* g) { T(22); return SR(15); }
bool ftPe_8011BAD8(HSD_GObj* g) { T(23); return SR(16); }
void ft_80084DB0(Fighter_GObj* g) { T(24); T(g == &GO); }
bool ftCo_80096CC8(Fighter_GObj* g, int l) { T(25); return 0; }
void ft_80082B1C(Fighter_GObj* g) { T(26); }
void ft_800835B0(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), HSD_GObjEvent b)
{ T(27); T(g == &GO); T(a == ftCo_80096CC8); T(b == ft_80082B1C); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.self_vel.x)); HM(B(F.self_vel.y)); HM(B(F.self_vel.z));
    HM(F.mv.co.jump.x0); HM(F.mv.co.jump.x4); HM(B(F.mv.co.jump.jump_mul));
    HM(F.active_timer.lstick.y); HM(F.x2227_b0);
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 11; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&FD, sizeof FD); zero(&SFX, sizeof SFX);
            SFX.x10 = rnd() % 1000; FD.x4C_sfx = &SFX; F.ft_data = &FD;
            F.x197C = (rnd() & 1) ? &X197C : NULL;
            F.player_id = rnd() % 6;
            F.input.lstick[0].x = rg(); F.input.lstick[0].y = rg();
            F.active_timer.lstick.y = rnd() % 12;
            F.input.pressed_buttons = rnd() & 0xF00;
            CD.tap_jump_threshold = rg(); CD.tap_jump_window = rnd() % 12;
            CD.relaxed_tap_jump_threshold = rg(); CD.x438 = rg(); CD.x78 = rg();
            F.co_attrs.ground_to_air_jump_momentum_multiplier = rw();
            F.co_attrs.jump_h_initial_velocity = rw();
            F.co_attrs.hop_v_initial_velocity = rw();
            F.co_attrs.jump_v_initial_velocity = rw();
            F.co_attrs.jump_h_max_velocity = rw();
            F.mv.co.jump.x0 = rnd() & 1; F.mv.co.jump.x4 = rnd() & 1;
            F.self_vel.x = rw(); F.self_vel.y = rw(); F.self_vel.z = rw();
            F.facing_dir = rg();
            switch (op) {
            case 0: ret = ftCo_Jump_GetInput(&GO); break;
            case 1: ret = ftCo_Jump_CheckInput(&GO); break;
            case 2: ret = fn_800CAF78(&GO); break;
            case 3: ret = ftCo_800CB024(&GO); break;
            case 4: { int a = rnd() & 1; float m = rw(); ftCo_800CB110(&GO, a, m); break; }
            case 5: ftCo_Jump_Enter(&GO); break;
            case 6: ftCo_Jump_Anim(&GO); break;
            case 7: ftCo_Jump_IASA(&GO); break;
            case 8: ftCo_Jump_Phys_Inner(&GO); break;
            case 9: ftCo_Jump_Phys(&GO); break;
            case 10: ftCo_Jump_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
