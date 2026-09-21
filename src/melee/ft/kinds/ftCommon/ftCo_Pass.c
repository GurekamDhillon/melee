#include "ftCo_Pass.h"

#include <Runtime/platform.h>

#include <melee/ft/forward.h>

#include "ftCo_0C60.h"
#include "ftCo_AirCatch.h"
#include "ftCo_Attack100.h"
#include "ftCo_AttackAir.h"
#include "ftCo_EscapeAir.h"
#include "ftCo_Fall.h"
#include "ftCo_HammerWait.h"
#include "ftCo_ItemThrow.h"
#include "ftCo_JumpAerial.h"
#include "ftCo_SpecialAir.h"
#include "types.h"
#include <dolphin/mtx.h>
#include <melee/ft/fighter.h>
#include <melee/ft/ft_081B.h>
#include <melee/ft/ftanim.h>
#include <melee/ft/ftcommon.h>
#include <melee/ft/types.h>
#include <melee/mp/mpcoll.h>

bool ftCo_80099F1C(Fighter_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    if (fp->input.lstick[0].y <= -p_ftCommonData->x464 &&
        fp->active_timer.lstick.y < p_ftCommonData->x468 &&
        mpColl_IsOnPlatform(&fp->coll_data))
    {
        return true;
    }
    return false;
}

bool ftCo_80099F9C(Fighter_GObj* gobj)
{
    u8 _[8];
    Fighter* fp = gobj->user_data;
    if (ftCo_800C5240(gobj)) {
        return ftCo_800C60C8(gobj);
    }
    if (!fp->mv.co.pass.x0 && ftCo_80099F1C(gobj)) {
        fp->mv.co.pass.x0 = true;
        fp->mv.co.pass.x4 = p_ftCommonData->x470;
        return true;
    }
    return false;
}

#if defined(TARGET_PC)
/* ftCo_80099F1C as retail inlines it into ftCo_8009A080, with UCF 0.84's Shield Drop Extended
 * (Slippi External/UCF 0.84/UCF/UCF Shield Drop Extended.asm, @ 0x8009A0B8, on the `y <= -x464`
 * compare of that inlined copy only): the stick-down test also passes while UCF's flick counter
 * (the pad buffer's byte 9, fighter.c) is above 1. */
static bool ftCo_8009A080_Check(Fighter_GObj* gobj)
{
    extern int Replay_UcfVersion(int port);
    extern int ftUcf_FlickCounter(int port);
    Fighter* fp = gobj->user_data;
    bool down = fp->input.lstick[0].y <= -p_ftCommonData->x464;
    if (!down && Replay_UcfVersion(fp->x618_player_id) == 84) {
        down = ftUcf_FlickCounter(fp->x618_player_id) > 1;
    }
    return down && fp->active_timer.lstick.y < p_ftCommonData->x468 &&
           mpColl_IsOnPlatform(&fp->coll_data);
}
#define ftCo_80099F1C_8009A080 ftCo_8009A080_Check
#else
#define ftCo_80099F1C_8009A080 ftCo_80099F1C
#endif

bool ftCo_8009A080(Fighter_GObj* gobj)
{
    u8 _[8];
    Fighter* fp = gobj->user_data;
    if (fp->input.held_buttons[0] & HSD_PAD_LR && ftCo_80099F1C_8009A080(gobj)) {
        ftCo_8009A228(gobj);
        return true;
    }
    return false;
}

bool ftCo_8009A134(Fighter_GObj* gobj)
{
    u8 _[8];
    Fighter* fp = gobj->user_data;
    CollData* coll = &fp->coll_data;
    if (mpColl_IsOnPlatform(coll)) {
        mpUpdateFloorSkip(coll);
        return true;
    }
    return false;
}

void ftCo_8009A184(Fighter_GObj* gobj, FtMotionId msid, MotionFlags mf,
                   float anim_start)
{
    u8 _[8];
    Fighter* fp = gobj->user_data;
    ftCommon_8007D5D4(fp);
    ftCommon_ClampAirDrift(fp);
    fp->self_vel.y = p_ftCommonData->x46C;
    Fighter_ChangeMotionState(gobj, msid, mf, anim_start, 1, 0, NULL);
    mpUpdateFloorSkip(&fp->coll_data);
    fp->active_timer.lstick.y = 0xFE;
}

void ftCo_8009A228(Fighter_GObj* gobj)
{
    u8 _[8] = { 0 };
    Fighter* fp = gobj->user_data;
    ftCommon_8007D5D4(fp);
    ftCommon_ClampAirDrift(fp);
    fp->self_vel.y = p_ftCommonData->x46C;
    Fighter_ChangeMotionState(gobj, ftCo_MS_Pass, Ft_MF_None, 0, 1, 0, NULL);
    mpUpdateFloorSkip(&fp->coll_data);
    fp->active_timer.lstick.y = 0xFE;
}

void ftCo_Pass_Anim(Fighter_GObj* gobj)
{
    if (!ftAnim_IsFramesRemaining(gobj)) {
        ftCo_Fall_Enter(gobj);
    }
}

void ftCo_Pass_IASA(Fighter_GObj* gobj)
{
    RETURN_IF(ftCo_SpecialAir_CheckInput(gobj));
    RETURN_IF(ftCo_80095328(gobj, NULL));
    RETURN_IF(ftCo_800D7100(gobj));
    RETURN_IF(ftCo_800C3B10(gobj));
    RETURN_IF(ftCo_80099A58(gobj));
    RETURN_IF(ftCo_AttackAir_CheckItemThrowInput(gobj));
    RETURN_IF(ftCo_800D705C(gobj));
    RETURN_IF(ftCo_800CB870(gobj));
}

void ftCo_Pass_Phys(Fighter_GObj* gobj)
{
    ft_80084DB0(gobj);
}

void ftCo_Pass_Coll(Fighter_GObj* gobj)
{
    ft_80082F28(gobj);
}
