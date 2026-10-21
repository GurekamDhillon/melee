/* TU: src/melee/ft/kinds/ftCommon/ftCo_Turn.c */
/* C oracle for src/tu/ftCo_Turn.rs: the whole TU (TARGET_PC) included verbatim; every callee out
 * of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

#define CK(name, id) bool name(Fighter_GObj* g) { T(id); T(g == &GO); return SR(id) && SR(id + 16) && SR(id + 32); }
#define VS(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
CK(ftCo_SpecialS_CheckInput, 1)
CK(ftCo_800D68C0, 2)
CK(ftCo_Attack100_CheckInput, 3)
CK(ftCo_Catch_CheckInput, 4)
CK(ftCo_AttackS4_CheckInput, 5)
CK(ftCo_AttackHi4_CheckInput, 6)
CK(ftCo_AttackLw4_CheckInput, 7)
CK(ftCo_AttackS3_CheckInput, 8)
CK(ftCo_AttackHi3_CheckInput, 9)
CK(ftCo_AttackLw3_CheckInput, 10)
CK(ftCo_Attack1_CheckInput, 11)
CK(ftCo_80091A4C, 12)
CK(ftCo_800DE9D8, 13)
CK(ftCo_Jump_CheckInput, 14)
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(20); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
VS(ftAnim_8006EBA4, 21)
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(22); return SR(22); }
VS(ft_8008A2BC, 23)
void ftCo_Dash_Enter(Fighter_GObj* g, int a) { T(24); T(g == &GO); T(a); }
VS(ft_80084F3C, 25)
VS(ft_80083F88, 26)
int ScriptGame_CapsActionAllowed(Fighter* f, int flag) { T(27); T(f == &F); T(flag); return SR(27); }
static int UCF;
int Replay_UcfVersion(int port) { T(28); T(port); return UCF; }
int Replay_RawStickBack(int port, int which, int back) { T(29); T(port); T(which); T(back); return (int) (rnd() % 256) - 128; }
int ftUcf_PadRaw(int port, int which_y, int back) { T(30); T(port); T(which_y); T(back); return (int) (rnd() % 256) - 128; }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.facing_dir)); HM(F.input.pressed_buttons);
    HM(F.mv.co.turn.has_turned); HM(F.mv.co.turn.just_turned); HM(B(F.mv.co.turn.facing_after));
    HM(B(F.mv.co.turn.frames_to_turn)); HM(B(F.mv.co.turn.x8)); HM(F.mv.co.turn.x1C);
    return H;
}

int main(void)
{
    static const int ucfs[4] = { 0, 73, 84, 84 };
    int op, k;
    tu_init();
    for (op = 0; op < 10; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            UCF = ucfs[rnd() % 4];
            CD.x34 = rg();
            CD.dash_smash_stick_threshold = rg();
            CD.dash_smash_window = (int) (rnd() % 8);
            F.input.lstick[0].x = rg();
            F.input.pressed_buttons = rnd() & 0x3FF;
            F.facing_dir = rg();
            F.active_timer.lstick.x = rnd() % 4;
            F.player_id = rnd() % 6;
            F.x618_player_id = rnd() % 6;
            F.is_sub_fighter = rnd() & 1;
            F.x3E4_fighterCmdScript.frame_count = (rnd() & 1) ? 2.0f : rg();
            F.co_attrs.standing_turn_frames = rw();
            F.mv.co.turn.has_turned = rnd() & 1;
            F.mv.co.turn.just_turned = rnd() & 1;
            F.mv.co.turn.facing_after = rg();
            F.mv.co.turn.frames_to_turn = (rnd() & 1) ? rg() : 0.0f;
            F.mv.co.turn.x8 = (rnd() & 1) ? rg() : 0.0f;
            F.mv.co.turn.x1C = rnd() & 0x3FF;
            switch (op) {
            case 0: ret = ftCo_800C97A8(&GO); break;
            case 1: ret = ftCo_Turn_CheckInput(&GO); break;
            case 2: { float a = rw(); float b = rw(); float c = rw(); u32 m = rnd() % 400; u32 fl = rnd(); ftCo_Turn_Enter(&GO, (FtMotionId) m, fl, a, b, c); } break;
            case 3: ftCo_Turn_Enter_Basic(&GO); break;
            case 4: ftCo_Turn_Anim(&GO); break;
            case 5: ftCo_Turn_IASA(&GO); break;
            case 6: ftCo_Turn_Phys(&GO); ftCo_Turn_Coll(&GO); break;
            case 7: ret = fn_800C9C2C(&GO); break;
            case 8: ftCo_Turn_Enter_Smash(&GO); break;
            case 9: ftCo_Turn_Anim_Inner(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
