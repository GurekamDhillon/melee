#ifndef SCRIPT_SKILL_H
#define SCRIPT_SKILL_H
/* Skill telemetry and crits: read-only observers of the game's own decisions (script_skill.inc), plus the
 * seeded crit decision in the hit path (script_crit.inc). Scalars only cross to the native side. */
#include <melee/ft/forward.h>
#include <melee/lb/forward.h>
#if defined(TARGET_PC)
/* ftCo_LandingAir_EnterWithLag: mode 1 L-cancelled, 0 missed, -1 auto-cancelled. */
void ScriptGame_SkillLanding(Fighter* fp, int mode, float lag, float lag_final);
/* A fighter hitbox or a projectile connected and dealt damage (attacker may be NULL for an ownerless source). */
void ScriptGame_SkillHit(Fighter* attacker, Fighter* victim, float damage);
/* A shield took a hit inside the powershield window. */
void ScriptGame_SkillPerfectShield(Fighter* shielder, Fighter* attacker, int projectile, float damage);
/* The crit decision, called right after ScriptGame_HitRulePercentQueue (script_crit.inc). */
void ScriptGame_CritHit(Fighter* attacker, HitCapsule* hit, Fighter* victim, float damage, int projectile);
#endif
#endif
