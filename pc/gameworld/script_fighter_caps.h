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
int ScriptGame_CapsMaxJumps(Fighter* fp);
int ScriptGame_CapsActionAllowed(Fighter* fp, int flag);
#define FT_CAPS_MAX_JUMPS(fp) ScriptGame_CapsMaxJumps(fp)
#define FT_CAPS_ACTION_ALLOWED(fp, flag) ScriptGame_CapsActionAllowed(fp, flag)
#else
#define FT_CAPS_MAX_JUMPS(fp) ((fp)->co_attrs.max_jumps)
#define FT_CAPS_ACTION_ALLOWED(fp, flag) 1
#endif
#endif
