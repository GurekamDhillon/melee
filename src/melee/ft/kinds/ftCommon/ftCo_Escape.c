#include "ftCo_Escape.h"

#include <Runtime/platform.h>

#include <melee/ft/forward.h>

#include "forward.h"
#include "ftCo_ItemThrow.h"
#include "types.h"
#include <dolphin/mtx.h>
#include <melee/ft/fighter.h>
#include <melee/ft/ft_081B.h>
#include <melee/ft/ft_084E.h>
#include <melee/ft/ft_0892.h>
#include <melee/ft/ft_0DF1.h>
#include <melee/ft/ftanim.h>
#include <melee/ft/ftcommon.h>
#include <melee/ft/ftparts.h>
#include <melee/ft/kinds/ftSamus/ftsamusspeciallw1.h>
#include <melee/ft/kinds/ftSamus/types.h>
#include <melee/ft/kinds/ftYoshi/ftyoshi.h>
#include <melee/ft/kinds/ftYoshi/ftyoshiguard.h>
#include <melee/ft/types.h>

/* 0992A8 */ static void ftCo_800992A8(Fighter_GObj* gobj, FtMotionId msid,
                                       bool);
/* 099314 */ static void ftCo_80099314(Fighter_GObj* gobj, FtMotionId msid,
                                       bool arg2);
/* 099390 */ static void ftCo_80099390(Fighter_GObj* gobj, FtMotionId msid,
                                       bool arg2);
/* 099438 */ static void ftCo_80099438(Fighter_GObj* gobj, FtMotionId msid,
                                       bool arg2);
/* 099564 */ static void ftCo_80099564(Fighter_GObj* gobj);
/* 099644 */ static void ftCo_80099644(Fighter_GObj* gobj);
/* 099754 */ static void ftCo_80099754(Fighter_GObj* gobj);
/* 099894 */ static void ftCo_80099894(Fighter_GObj* gobj);
/* 0998EC */ static void ftCo_800998EC(Fighter_GObj* gobj);
/* 099954 */ static void ftCo_80099954(Fighter_GObj* gobj);

static inline bool inlineA1(Fighter* fp)
{
    if (ABS(fp->input.lstick[0].x) >= p_ftCommonData->x31C &&
        fp->active_timer.lstick.x < p_ftCommonData->x320)
    {
        return true;
    }
    return false;
}

bool ftCo_8009917C(Fighter_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    float stick_x;
    if (inlineA1(fp)) {
        stick_x = fp->input.lstick[0].x;
    } else if (ftCo_800DF8B0(fp)) {
        stick_x = fp->input.cstick[0].x;
    } else {
        return false;
    }
    {
        FtMotionId msid =
            stick_x * fp->facing_dir >= 0 ? ftCo_MS_EscapeF : ftCo_MS_EscapeB;
        ftCo_800992A8(gobj, msid, p_ftCommonData->x324);
    }
    return true;
}

bool ftCo_80099264(Fighter_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    if (fp->input.held_buttons[0] & HSD_PAD_LR) {
        ftCo_800992A8(gobj, ftCo_MS_EscapeF, false);
        return true;
    }
    return false;
}

void ftCo_800992A8(Fighter_GObj* gobj, FtMotionId msid, bool arg2)
{
    Fighter* fp = gobj->user_data;
    switch (fp->kind) {
    case Ft_Kind_Samus:
        ftCo_80099390(gobj, msid, arg2);
        break;
    case Ft_Kind_Yoshi:
        ftCo_80099438(gobj, msid, arg2);
        break;
    default:
        ftCo_80099314(gobj, msid, arg2);
        break;
    }
    ftCommon_8007EBAC(fp, 23, 0);
}

void ftCo_80099314(Fighter_GObj* gobj, FtMotionId msid, bool arg2)
{
    Fighter* fp = gobj->user_data;
    fp->throw_flags = 0;
    Fighter_ChangeMotionState(gobj, msid, Ft_MF_None, 0, 1, 0, NULL);
    ftAnim_8006EBA4(gobj);
    fp->x221D_b5 = true;
    fp->mv.co.escape.x0 = arg2;
}

void ftCo_80099390(Fighter_GObj* gobj, FtMotionId msid, bool arg2)
{
    u8 _[8] = { 0 };
    Fighter* fp = gobj->user_data;
    fp->cmd_vars[0] = 0;
    fp->mv.co.escape.x4 = false;
    ftCo_80099314(gobj, msid, arg2);
    fp->anim_cb = ftCo_80099564;
    fp->coll_cb = ftCo_80099754;
}

void ftCo_80099438(Fighter_GObj* gobj, FtMotionId msid, bool arg2)
{
    u8 _[8] = { 0 };
    Fighter* fp = gobj->user_data;
    ftCo_80099314(gobj, msid, arg2);
    ftParts_80074B0C(gobj, 0, 1);
    ftYs_Init_8012BDA0(gobj);
    fp->anim_cb = ftCo_80099644;
}

void ftCo_Escape_Anim(Fighter_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    if (ftCheckThrowB3(fp)) {
        fp->facing_dir = -fp->facing_dir;
    }
    if (!ftAnim_IsFramesRemaining(gobj)) {
        fp->gr_vel = 0;
        ft_8008A2BC(gobj);
    }
}

void ftCo_80099564(Fighter_GObj* gobj)
{
    u8 _[8] = { 0 };
    Fighter* fp = gobj->user_data;
    if (fp->cmd_vars[0] && !fp->mv.co.escape.x4) {
        ftSs_SpecialLw_8012AEBC(gobj);
        fp->mv.co.escape.x4 = true;
    }
    if (fp->cmd_vars[0] == 0 && fp->mv.co.escape.x4) {
        ftSs_SpecialLw_8012AF38(gobj);
        fp->mv.co.escape.x4 = false;
    }
    ftCo_Escape_Anim(gobj);
}

/// @todo Shared code with #ftCo_Escape_Anim.
void ftCo_80099644(Fighter_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    if (ftCheckThrowB3(fp)) {
        fp->facing_dir = -fp->facing_dir;
    }
    if (!ftAnim_IsFramesRemaining(gobj)) {
        fp->gr_vel = 0;
        if (ftYs_Shield_8012CC1C(gobj)) {
            return;
        }
        ftYs_Init_8012BE3C(gobj);
        ft_8008A2BC(gobj);
    }
    ftYs_Init_8012B8A4(gobj);
}

void ftCo_Escape_IASA(Fighter_GObj* gobj)
{
    RETURN_IF(ftCo_8009563C(gobj));
}

void ftCo_Escape_Phys(Fighter_GObj* gobj)
{
    ft_80085004(gobj);
}

void ftCo_Escape_Coll(Fighter_GObj* gobj)
{
    ft_80084104(gobj);
}

void ftCo_80099754(Fighter_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    ftSs_DatAttrs* da = fp->dat_attrs;
    if (fp->cmd_vars[0]) {
        ft_800847D0(gobj, &da->height_attributes);
    } else {
        ft_80084104(gobj);
    }
}

static inline bool inlineB0(Fighter* fp)
{
    if (fp->input.lstick[0].y <= p_ftCommonData->x314 &&
        fp->active_timer.lstick.y < p_ftCommonData->x318)
    {
        return true;
    }
    return false;
}

#if defined(TARGET_PC)
/* UCF 0.73 shield drop, as Slippi's console codes ran it in 2019 ("UCF 0.73 Shield Drop - Check
 * for Toggle.asm", slippi-ssbm-asm eb9abdc, @ 0x800998A4 = the entry of ftCo_80099894, which it
 * aborts so the spotdodge check below returns false): when the C-stick is not what pulled the
 * shield down, the control stick is on the rim ((trunc(|x|*80 - 1e-5) + 2) / 80 per axis, summed
 * squares >= 1), its x-timer is past 3, and y is above -walk_fast_stick_threshold (-0.8), the
 * spotdodge does not happen - the shield drop does. Applied for MELEE_SLP playback of ports whose
 * recording had UCF on. */
static f32 ftCo_Ucf073RimAxis(f32 v)
{
    f32 t = (v < 0.0F ? -v : v) * 80.0F - 9.953975677490234e-06F;
    return (f32) ((int) t + 2) / 80.0F;
}

static bool ftCo_Ucf073BlocksSpotdodge(Fighter* fp)
{
    extern int Replay_UcfVersion(int port);
    f32 x, y;
    if (Replay_UcfVersion(fp->x618_player_id) != 73) {
        return false;
    }
    if (!(fp->input.cstick[0].y > p_ftCommonData->x314)) {
        return false;
    }
    x = ftCo_Ucf073RimAxis(fp->input.lstick[0].x);
    y = ftCo_Ucf073RimAxis(fp->input.lstick[0].y);
    if (!(y * y + x * x >= 1.0F)) {
        return false;
    }
    if (fp->active_timer.lstick.x <= 3) {
        return false;
    }
    if (-p_ftCommonData->walk_fast_stick_threshold >= fp->input.lstick[0].y) {
        return false;
    }
    return true;
}
/* UCF 0.84 shield drop (Slippi External/UCF 0.84/UCF/UCF Shield Drop.asm, @ 0x800998A4 - the entry
 * of ftCo_80099894, from which it returns past the caller's "return true", so the spotdodge check
 * returns false). On a platform (floor line flag 0x100), when the C-stick is not what pulled the
 * shield down, the stick's x-timer has reached ftCommonData x320, y is above -0.8, and the stick
 * is on the rim (UCF's rim units, squared and summed, over 6400), the spotdodge does not happen -
 * the shield drop through the platform does. */
static int ftCo_Ucf084RimUnits(f32 v)
{
    f32 t = (f32) ((double) (v < 0.0F ? -v : v) * 80.0 - (double) 9.99999974738e-05F);
    return (int) t + 2;
}

static bool ftCo_Ucf084BlocksSpotdodge(Fighter* fp)
{
    extern int Replay_UcfVersion(int port);
    int rx, ry;
    if (Replay_UcfVersion(fp->x618_player_id) != 84) {
        return false;
    }
    if (!(fp->input.cstick[0].y > p_ftCommonData->x314)) {
        return false;
    }
    if (fp->active_timer.lstick.x < p_ftCommonData->x320) {
        return false;
    }
    if (!(fp->input.lstick[0].y > -0.8F)) {
        return false;
    }
    if (fp->coll_data.floor.index == -1 || !(fp->coll_data.floor.flags & 0x100)) {
        return false;
    }
    ry = ftCo_Ucf084RimUnits(fp->input.lstick[0].y);
    rx = ftCo_Ucf084RimUnits(fp->input.lstick[0].x);
    return ry * ry + rx * rx > 6400;
}
#endif

bool ftCo_80099794(Fighter_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    if (fp->input.held_buttons[0] & HSD_PAD_LR && inlineB0(fp)) {
#if defined(TARGET_PC)
        if (ftCo_Ucf073BlocksSpotdodge(fp) || ftCo_Ucf084BlocksSpotdodge(fp)) {
            return false;
        }
#endif
        ftCo_80099894(gobj);
        return true;
    }
    return false;
}

bool ftCo_8009980C(Fighter_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    if (inlineB0(fp) || ftCo_800DF8E8(fp)) {
#if defined(TARGET_PC)
        if (ftCo_Ucf073BlocksSpotdodge(fp) || ftCo_Ucf084BlocksSpotdodge(fp)) {
            return false;
        }
#endif
        ftCo_80099894(gobj);
        return true;
    }
    return false;
}

void ftCo_80099894(Fighter_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    switch (fp->kind) {
    case Ft_Kind_Yoshi:
        ftCo_80099954(gobj);
        break;
    default:
        ftCo_800998EC(gobj);
        break;
    }
    ftCommon_8007EBAC(fp, 23, 0);
}

void ftCo_800998EC(Fighter_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    Fighter_ChangeMotionState(gobj, ftCo_MS_EscapeN, Ft_MF_None, 0, 1, 0,
                              NULL);
    ftAnim_8006EBA4(gobj);
    fp->x221D_b5 = true;
}

void ftCo_80099954(Fighter_GObj* gobj)
{
    u8 _[8] = { 0 };
    Fighter* fp = gobj->user_data;
    if (fp->x5F4_arr[0].idx == 1) {
        ftYs_Init_8012BE3C(gobj);
    }
    ftCo_800998EC(gobj);
}

void ftCo_EscapeN_Anim(Fighter_GObj* gobj)
{
    if (!ftAnim_IsFramesRemaining(gobj)) {
        ft_8008A2BC(gobj);
    }
}

void ftCo_EscapeN_IASA(Fighter_GObj* gobj) {}

void ftCo_EscapeN_Phys(Fighter_GObj* gobj)
{
    ft_80084F3C(gobj);
}

void ftCo_EscapeN_Coll(Fighter_GObj* gobj)
{
    ft_80084104(gobj);
}
