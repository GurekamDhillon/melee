/* Original offline FD isolation. Scalar bridge only; no shared host structs. */
#ifndef PC_SCRIPT_STAGE_ISOLATION_H
#define PC_SCRIPT_STAGE_ISOLATION_H
int mpScriptStageIsolationSet(int lines, int joints, int isolate);
int mpScriptStageIsolationActive(void);
int mpScriptStageIsolationTest(void);
int ScriptGame_StageIsolate(int owner, int isolate);
int ScriptGame_StageIsolationState(int owner);
void ScriptGame_StageIsolationClear(int owner);
void ScriptGame_StageIsolationWatch(void);
int ScriptGame_StageOriginalVisible(void);
#endif
