/* Included after script_stage in script_game.c. FD rendering is suppressed at
 * grDisplay's original ground callback; custom stage models have other callbacks. */
#include "script_stage_isolation.h"
static int script_stage_isolation_owner;
extern int Script_StageIsolationAllowed(int owner);
extern void Script_StageIsolationReleased(int owner);
extern int Netplay_Enabled(void);
extern int RB_Enabled(void);
extern int Replay_Active(void);
#include "script_stage_host.inc"

void ScriptGame_StageIsolationClear(int owner)
{
    if (owner == 0 || (owner > 0 && script_stage_isolation_owner == owner)) {
        int released = script_stage_isolation_owner;
        mpScriptStageIsolationSet(0, 0, 0);
        script_stage_host_hide(0);
        script_stage_isolation_owner = 0;
        if (released) Script_StageIsolationReleased(released);
    }
}

void ScriptGame_StageIsolationWatch(void)
{
    if (script_stage_isolation_owner &&
        (!Script_StageIsolationAllowed(script_stage_isolation_owner) ||
         Netplay_Enabled() || RB_Enabled() || Replay_Active() ||
         !script_stage_host_allowed() || !mpScriptStageIsolationActive()))
        ScriptGame_StageIsolationClear(0);
}

int ScriptGame_StageIsolate(int owner, int isolate)
{
    if (owner <= 0 || (script_stage_isolation_owner && script_stage_isolation_owner != owner)) return 0;
    if (!isolate) { ScriptGame_StageIsolationClear(owner); return 1; }
    if (!script_stage.map || script_stage.cap <= 0 ||
        script_stage.map != mpLib_8004D164() || !script_stage_host_allowed() ||
        Netplay_Enabled() || RB_Enabled() || Replay_Active()) return 0;
    if (!mpScriptStageIsolationSet(script_stage.base_l, script_stage.base_j, 1)) return 0;
    script_stage_isolation_owner = owner;
    script_stage_host_hide(1);
    return 1;
}

int ScriptGame_StageIsolationState(int owner)
{
    ScriptGame_StageIsolationWatch();
    return owner > 0 && script_stage_isolation_owner == owner && mpScriptStageIsolationActive();
}

int ScriptGame_StageOriginalVisible(void)
{
    ScriptGame_StageIsolationWatch();
    return !script_stage_isolation_owner;
}
