#include "ftCo_SquatRv.h"

#include <Runtime/platform.h>

#include <melee/ft/forward.h>

#include <stdbool.h>

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
#include "ftCo_Guard.h"
#include "ftCo_Jump.h"
#include "ftCo_Walk.h"
#include <melee/ft/fighter.h>
#include <melee/ft/ft_081B.h>
#include <melee/ft/ft_084E.h>
#include <melee/ft/ft_0892.h>
#include <melee/ft/inlines.h>
#include <melee/ft/types.h>

/* 0D6620 */ static void ftCo_SquatRv_Enter(Fighter_GObj* gobj);

bool ftCo_SquatRv_CheckInput(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);

#if defined(TARGET_PC)
    {
        /* Ported from m-ex (https://github.com/akaneia/m-ex):
         * asm/qol/UCF 0.84/UCF DBOOC SquatRv Fix.asm, inserted at 0x800D65EC. When
         * the stick is tilted past the 1.0 cardinal during the first frame of a turn
         * (active_timer.lstick.x == 0), the SquatRv threshold is relaxed from
         * p_ftCommonData->x94 to 0.59 so a full-tilt dashback out of crouch reads as
         * a dash, not a crouch-reverse.
         * Opt-in: MELEE_MEX=ucf_dbooc_squatrv_fix. */
        extern int Mex_Enabled(const char *);
        if (Mex_Enabled("ucf_dbooc_squatrv_fix")) {
            float deadzone = p_ftCommonData->x94;
            if (fp->active_timer.lstick.x < 1) {
                int x = (int) (fabsf(fp->input.lstick[0].x) * 80.0f -
                               0.0001f) +
                        2;
                int y = (int) (fabsf(fp->input.lstick[0].y) * 80.0f -
                               0.0001f) +
                        2;
                if (x * x + y * y > 6400) {
                    deadzone = 0.59f;
                }
            }
            if (fp->input.lstick[0].y > -deadzone) {
                ftCo_SquatRv_Enter(gobj);
                return true;
            }
            return false;
        }
    }
#endif

    if (fp->input.lstick[0].y > -p_ftCommonData->x94) {
        ftCo_SquatRv_Enter(gobj);
        return true;
    }

    return false;
}

void ftCo_SquatRv_Enter(Fighter_GObj* gobj)
{
    Fighter_ChangeMotionState(gobj, ftCo_MS_SquatRv, Ft_MF_None, 0.0F, 1.0F,
                              0.0F, NULL);
}

void ftCo_SquatRv_Anim(Fighter_GObj* gobj)
{
    if (!ftAnim_IsFramesRemaining(gobj)) {
        ft_8008A2BC(gobj);
    }
}

void ftCo_SquatRv_IASA(Fighter_GObj* gobj)
{
    RETURN_IF(ftCo_800D68C0(gobj));
    RETURN_IF(ftCo_Attack100_CheckInput(gobj));
    RETURN_IF(ftCo_AttackS4_CheckInput(gobj));
    RETURN_IF(ftCo_AttackHi4_CheckInput(gobj));
    RETURN_IF(ftCo_AttackLw4_CheckInput(gobj));
    RETURN_IF(ftCo_AttackS3_CheckInput(gobj));
    RETURN_IF(ftCo_AttackHi3_CheckInput(gobj));
    RETURN_IF(ftCo_AttackLw3_CheckInput(gobj));
    RETURN_IF(ftCo_Attack1_CheckInput(gobj));
    RETURN_IF(ftCo_80091A4C(gobj));
    RETURN_IF(ftCo_800DE9D8(gobj));
    RETURN_IF(ftCo_Jump_CheckInput(gobj));
    RETURN_IF(ftCo_Walk_CheckInput(gobj));
}

void ftCo_SquatRv_Phys(Fighter_GObj* gobj)
{
    ft_80084F3C(gobj);
}

void ftCo_SquatRv_Coll(Fighter_GObj* gobj)
{
    ft_80083F88(gobj);
}
