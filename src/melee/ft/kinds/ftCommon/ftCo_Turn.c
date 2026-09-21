#include "ftCo_Turn.h"

#include <melee/ft/forward.h>

#include <placeholder.h>

#include "forward.h"
#include "ftCo_AppealS.h"
#include "ftCo_Attack1.h"
#include "ftCo_Attack100.h"
#include "ftCo_AttackHi3.h"
#include "ftCo_AttackHi4.h"
#include "ftCo_AttackLw3.h"
#include "ftCo_AttackLw4.h"
#include "ftCo_AttackS3.h"
#include "ftCo_AttackS4.h"
#include "ftCo_Dash.h"
#include "ftCo_Guard.h"
#include "ftCo_Jump.h"
#include "ftCo_SpecialS.h"
#include <melee/ft/fighter.h>
#include <melee/ft/ft_081B.h>
#include <melee/ft/ft_084E.h>
#include <melee/ft/ft_0892.h>
#include <melee/ft/inlines.h>
#include <melee/ft/types.h>

bool ftCo_800C97A8(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);

    if (fp->input.lstick[0].x * fp->facing_dir <= p_ftCommonData->x34) {
        return true;
    }
    return false;
}

bool ftCo_Turn_CheckInput(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);

    if (ftCo_800C97A8(gobj)) {
        ftCo_Turn_Enter_Basic(gobj);
        return true;
    }
    return false;
}

void ftCo_Turn_Enter(Fighter_GObj* gobj, FtMotionId msid, MotionFlags flags,
                     f32 arg3, f32 frames_to_turn, f32 anim_start)
{
    Fighter* fp = GET_FIGHTER(gobj);

    fp->mv.co.turn.has_turned = false;
    fp->mv.co.turn.just_turned = 0;
    fp->mv.co.turn.facing_after = -fp->facing_dir;
    fp->mv.co.turn.frames_to_turn = frames_to_turn;
    fp->mv.co.turn.x8 = arg3;
    fp->mv.co.turn.x1C = 0;
    Fighter_ChangeMotionState(gobj, msid, flags, anim_start, 1.0F, 0.0F, NULL);
    ftAnim_8006EBA4(gobj);
}

void ftCo_Turn_Enter_Basic(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    float frames = fp->co_attrs.standing_turn_frames;
    PAD_STACK(8);
    ftCo_Turn_Enter(gobj, ftCo_MS_Turn, Ft_MF_None, 0.0F, frames, 0.0F);
}

void ftCo_Turn_Anim_Inner(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);

    if (fp->mv.co.turn.frames_to_turn > 0.0F) {
        fp->mv.co.turn.frames_to_turn -= 1.0F;
        return;
    }

    if (!fp->mv.co.turn.has_turned) {
        fp->mv.co.turn.has_turned = true;
        fp->mv.co.turn.just_turned = true;
        fp->facing_dir = -fp->facing_dir;
    }
}

void ftCo_Turn_Anim(Fighter_GObj* gobj)
{
    ftCo_Turn_Anim_Inner(gobj);

    if (!ftAnim_IsFramesRemaining(gobj)) {
        ft_8008A2BC(gobj);
    }
}

#if defined(TARGET_PC)
/* UCF 0.73 dashback, as Slippi's console codes ran it in 2019 ("UCF 0.73 Dashback - Check for
 * Toggle.asm", slippi-ssbm-asm eb9abdc, @ 0x800C9A44 = the first facing flip below): on the turn
 * frame whose script frame count is 2, with the stick past the dash threshold in the new direction
 * and its x-timer under 2, a raw X swing of more than 75 units against two frames earlier turns
 * the tilt turn into a dash (has_turned and just_turned set; the dash check further down then
 * fires). Applied for MELEE_SLP playback of ports whose recording had UCF on. Not ported: the
 * Ice Climbers branch that retroactively points Nana's follow input the new way. */
static void ftCo_Turn_Ucf073Dashback(Fighter* fp)
{
    extern int Replay_UcfVersion(int port);
    extern int Replay_RawStickBack(int port, int which, int back);
    union {
        f32 f;
        u32 u;
    } fc;
    int port = fp->player_id, d;
    if (Replay_UcfVersion(fp->x618_player_id) != 73) {
        return;
    }
    fc.f = fp->x3E4_fighterCmdScript.frame_count;
    if ((fc.u >> 16) != 0x4000) {
        return;
    }
    if (!(fp->input.lstick[0].x * fp->mv.co.turn.facing_after >=
          p_ftCommonData->dash_smash_stick_threshold))
    {
        return;
    }
    if (fp->active_timer.lstick.x >= 2 || fp->is_sub_fighter) {
        return;
    }
    d = Replay_RawStickBack(port, 0, 0) - Replay_RawStickBack(port, 0, 2);
    if (d * d > 0x15F9) {
        fp->mv.co.turn.just_turned = true;
        fp->mv.co.turn.has_turned = true;
    }
}
#endif

void ftCo_Turn_IASA(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);

    if (fp->mv.co.turn.just_turned) {
        fp->input.pressed_buttons |= fp->mv.co.turn.x1C;
    }
    if (!fp->mv.co.turn.has_turned) {
#if defined(TARGET_PC)
        ftCo_Turn_Ucf073Dashback(fp);
#endif
        fp->facing_dir = -fp->facing_dir;
    }

    RETURN_IF(ftCo_SpecialS_CheckInput(gobj));
    RETURN_IF(ftCo_800D68C0(gobj));
    RETURN_IF(ftCo_Attack100_CheckInput(gobj));
    RETURN_IF(ftCo_Catch_CheckInput(gobj));
    RETURN_IF(ftCo_AttackS4_CheckInput(gobj));
    RETURN_IF(ftCo_AttackHi4_CheckInput(gobj));
    RETURN_IF(ftCo_AttackLw4_CheckInput(gobj));
    RETURN_IF(ftCo_AttackS3_CheckInput(gobj));
    RETURN_IF(ftCo_AttackHi3_CheckInput(gobj));
    RETURN_IF(ftCo_AttackLw3_CheckInput(gobj));
    RETURN_IF(ftCo_Attack1_CheckInput(gobj));

    if (!fp->mv.co.turn.has_turned) {
        fp->facing_dir = -fp->facing_dir;
    }

    RETURN_IF(ftCo_80091A4C(gobj));
    RETURN_IF(ftCo_800DE9D8(gobj));
    RETURN_IF(ftCo_Jump_CheckInput(gobj));

    fn_800C9C2C(gobj);
    if (fp->mv.co.turn.just_turned && fp->mv.co.turn.x8) {
        if (fp->input.lstick[0].x * fp->mv.co.turn.facing_after >=
            p_ftCommonData->dash_smash_stick_threshold)
        {
            ftCo_Dash_Enter(gobj, 0);
        }
    }

    if (fp->input.pressed_buttons & HSD_PAD_A) {
        fp->mv.co.turn.x1C |= HSD_PAD_A;
    }

    if (fp->input.pressed_buttons & HSD_PAD_B) {
        fp->mv.co.turn.x1C |= HSD_PAD_B;
    }

    if (fp->mv.co.turn.just_turned) {
        fp->mv.co.turn.just_turned = false;
    }
}

void ftCo_Turn_Phys(Fighter_GObj* gobj)
{
    ft_80084F3C(gobj);
}

void ftCo_Turn_Coll(Fighter_GObj* gobj)
{
    ft_80083F88(gobj);
}

bool fn_800C9C2C(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    if (fp->input.lstick[0].x * fp->mv.co.turn.facing_after >=
            p_ftCommonData->dash_smash_stick_threshold &&
        fp->active_timer.lstick.x < p_ftCommonData->dash_smash_window)
    {
        fp->mv.co.turn.x8 = fp->mv.co.turn.facing_after;
        return true;
    }
    return false;
}

void ftCo_Turn_Enter_Smash(Fighter_GObj* gobj)
{
    Fighter* fp_r7 = GET_FIGHTER(gobj);
    float facing = fp_r7->facing_dir;
    PAD_STACK(1);

    fp_r7->mv.co.turn.has_turned = false;
    fp_r7->mv.co.turn.just_turned = false;
    fp_r7->mv.co.turn.facing_after = -fp_r7->facing_dir;
    fp_r7->mv.co.turn.frames_to_turn = 0.0F;
    fp_r7->mv.co.turn.x8 = facing;
    fp_r7->mv.co.turn.x1C = 0;

    Fighter_ChangeMotionState(gobj, ftCo_MS_Turn, Ft_MF_None, 0.0F, 1.0F, 0.0F,
                              NULL);
    ftAnim_8006EBA4(gobj);
}
