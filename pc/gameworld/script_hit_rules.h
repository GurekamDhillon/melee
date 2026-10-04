#ifndef SCRIPT_HIT_RULES_H
#define SCRIPT_HIT_RULES_H
#if defined(TARGET_PC)
#include <melee/ft/forward.h>
#include <melee/it/forward.h>
#include <melee/lb/forward.h>
void ScriptGame_HitRuleCreate(Fighter*,HitCapsule*,int);
void ScriptGame_HitRuleItemCreate(Item*,HitCapsule*);
void ScriptGame_HitRuleRetire(void*,unsigned);
float ScriptGame_HitRuleBaseDamage(HitCapsule*);
void ScriptGame_HitRuleDamageOnly(HitCapsule*);
void ScriptGame_HitRuleDamage(HitCapsule*);
void ScriptGame_HitRuleForget(HitCapsule*);
void ScriptGame_HitRuleSpecial(HitCapsule*);
float ScriptGame_HitRuleContact(HitCapsule*,Fighter*,float,int);
void ScriptGame_HitRuleWon(HitCapsule*,Fighter*);
void ScriptGame_HitRuleContext(HitCapsule*,Fighter*);
void ScriptGame_HitRuleReport(Fighter*,Fighter*,HitCapsule*,float);
#else
#define ScriptGame_HitRuleReport(attacker,victim,hit,damage) ((void)0)
#define ScriptGame_HitRuleBaseDamage(hit) ((hit)->damage)
#define ScriptGame_HitRuleContact(hit,victim,value,kb) (value)
#endif
#endif
