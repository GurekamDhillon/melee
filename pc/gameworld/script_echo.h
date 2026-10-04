#ifndef SCRIPT_ECHO_H
#define SCRIPT_ECHO_H
#include "script_echo_limits.h"
#include <melee/ft/forward.h>
#include <melee/lb/forward.h>
int ScriptGame_FighterHistoryDepth(void);
int ScriptGame_FighterHistoryRead(int entity,int age,int hit,int field);
int ScriptGame_EchoAdd(int entity,int owner,int delay,int move,int match_element,int airborne,int damage,int knockback,int element,int once);
int ScriptGame_EchoAddWithHandle(int entity,int owner,int delay,int move,int match_element,int airborne,int damage,int knockback,int element,int once,int handle);
int ScriptGame_EchoRemove(int handle,int owner);
void ScriptGame_EchoClear(int owner);
int ScriptGame_EchoRead(int entity,int index,int field);
int ScriptGame_EchoOwner(int entity);
int ScriptGame_EchoNextHandle(void);
int ScriptGame_EchoReplace(int entity,int owner,int count);
void ScriptGame_EchoFrame(void);
void ScriptGame_EchoReset(void);
void ScriptGame_EchoCapture(Fighter* fighter);
void ScriptGame_EchoPrepare(void);
int ScriptGame_EchoState(int entity,int handle,int field);
int ScriptGame_EchoReportContext(Fighter* attacker,Fighter* victim,HitCapsule* hit);
int ScriptGame_EchoCapsuleCount(Fighter* fighter);
HitCapsule* ScriptGame_EchoCapsule(Fighter* fighter,int index);
int ScriptGame_EchoIsCapsule(HitCapsule* hit);
int ScriptGame_EchoHitIndex(Fighter* fighter,HitCapsule* hit);
void ScriptGame_EchoConnected(HitCapsule* hit,Fighter* victim);
int ScriptGame_EchoBlocked(HitCapsule* hit,Fighter* victim);
int ScriptGame_EchoCredit(Fighter* attacker,Fighter* victim,HitCapsule* hit,float damage);
int ScriptGame_EchoSourceX(HitCapsule* hit,int fallback);
int ScriptGame_EchoSameGroup(HitCapsule* a,HitCapsule* b);
int ScriptGame_EchoTargetAllowed(Fighter* source,Fighter* target);
#endif
