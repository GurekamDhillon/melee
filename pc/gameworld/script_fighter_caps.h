#ifndef SCRIPT_FIGHTER_CAPS_H
#define SCRIPT_FIGHTER_CAPS_H
/* Only scalar calls cross the native/game boundary. Fighter* hooks stay game-side. */
#include <melee/ft/forward.h>
enum ScriptCapsAction {
    ScriptCaps_NoShield = 1,
    ScriptCaps_NoAirDodge = 2,
    ScriptCaps_NoRun = 4,
    ScriptCaps_NoGrab = 8,
    ScriptCaps_NoSpecials = 16
};
#if defined(TARGET_PC)
void ScriptGame_ArmorResetReaction(Fighter* fp);
float ScriptGame_ArmorSubtract(Fighter* fp, float kb);
int ScriptGame_ArmorReact(Fighter* fp);
int ScriptGame_ArmorAbsorbed(Fighter* fp);
void ScriptGame_ArmorRelease(int owner);
void ScriptGame_ArmorFrame(void);
int ScriptGame_CapsMaxJumps(Fighter* fp);
/* interrupt window (script_fighter_interrupt.inc): open on a connecting hit (kind 0 fighter,
 * 1 shield, 2 item), run after the fighter's own input callback, hash word for RB_GameHash */
void ScriptGame_IntrWinHit(Fighter* fp, int kind);
void ScriptGame_IntrWinTick(Fighter_GObj* gobj);
unsigned ScriptGame_IntrWinHashWord(int entity);
int ScriptGame_CapsActionAllowed(Fighter* fp, int flag);
#define FT_CAPS_MAX_JUMPS(fp) ScriptGame_CapsMaxJumps(fp)
#define FT_CAPS_ACTION_ALLOWED(fp, flag) ScriptGame_CapsActionAllowed(fp, flag)
#else
#define FT_CAPS_MAX_JUMPS(fp) ((fp)->co_attrs.max_jumps)
#define FT_CAPS_ACTION_ALLOWED(fp, flag) 1
#endif
#endif
