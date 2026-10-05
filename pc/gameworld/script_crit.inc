/* Crits (included at the end of script_skill.inc, itself included by script_game.c).
 *
 * A seeded, deterministic crit decision at the same point hit rules apply percent-only damage: ScriptGame_CritHit is
 * called right after ScriptGame_HitRulePercentQueue in both collision paths (fighter hitbox, projectile). A crit adds
 * damage*(multiplier-1) to the victim's pending percent delta, so it rides the percent-only commit (it does not change
 * the launch the retail formula computes from the original damage); an optional per-crit launch multiplier is applied
 * where the hit-rule family launch is applied (ScriptGame_HitRuleContact, knockback). Every field below lives in this
 * translation unit's snapshotted BSS, so snapshots, LAB rewind and the rollback state carry it; the generator is
 * advanced by exactly one draw per eligible hit (plus one more when the multiplier has a range), in the collision
 * pass's own order, so a resimulation from the same snapshot reproduces the same crits. Configuration is a
 * gameplay write (offline only, owner-scoped, forks the timeline); online the table is empty, so nothing happens.
 *
 * Tag slots: 0 default, then the hit-rule move tags 1 jab 2 dash_attack 3 tilt 4 smash 5 aerial 6 grab 7 throw
 * 8 special 9 projectile. Slot values arrive fully resolved (the Lua side inherits missing keys from the default). */
extern void Script_CritEvent(int a, int b, int c, int d, int base, int final_dmg, int strength, int x, int y, int z, int hitlag, int mult);
#define SCRIPT_CRIT_SLOTS 10
typedef struct {
    int owner; Fighter* fighter; unsigned has;
    float chance[SCRIPT_CRIT_SLOTS], mult[SCRIPT_CRIT_SLOTS], mult_max[SCRIPT_CRIT_SLOTS], launch[SCRIPT_CRIT_SLOTS];
    float min_percent, max_mult; int force, hits, crits;
} ScriptCritCfg;
typedef struct { Fighter* attacker; int attacker_e, victim_e, tag, forced, projectile; float base, final_dmg, mult, strength, launch, x, y, z; } ScriptCritPending;
typedef struct { HitCapsule* hit; Fighter* victim; float launch; } ScriptCritLaunch;
static ScriptCritCfg script_crit_cfg[12];
static ScriptCritPending script_crit_pending[8];
static ScriptCritLaunch script_crit_launches[8];
static int script_crit_npending;
static unsigned script_crit_rng = 0x2545F491u;

static unsigned script_crit_next(void)
{
    unsigned x = script_crit_rng;
    if (!x) x = 0x2545F491u;
    x ^= x << 13; x ^= x >> 17; x ^= x << 5;
    script_crit_rng = x;
    return x;
}
static float script_crit_unit(void) { return (float) (script_crit_next() >> 8) / 16777216.0f; } /* [0,1) with 24 bits */
static float script_crit_fin(int bits) { union { int i; float f; } u; u.i = bits; return u.f; }
static int script_crit_ok(float f) { return (script_skill_bits(f) & 0x7f800000) != 0x7f800000; }

/* flags: 1 begin (claim and clear), 2 release, 4 write slot, 8 set min_percent and force. Returns 0 on an owner conflict
 * or a bad argument, 1 otherwise. */
int ScriptGame_CritSet(int entity, int owner, int slot, int chance, int mult, int mult_max, int launch, int min_percent, int force, int flags)
{
    ScriptCritCfg* c; Fighter* fp; int i;
    if (entity < 1 || entity > 12 || owner <= 0) return 0;
    fp = ScriptGame_CapsFighter(entity); c = &script_crit_cfg[entity-1];
    if (c->owner && c->fighter == fp && c->owner != owner) return 0;
    if (c->owner && c->fighter != fp) memset(c, 0, sizeof *c);
    if (flags & 2) { if (c->owner == owner) memset(c, 0, sizeof *c); return 1; }
    if (!fp) return 0;
    if (flags & 1) { memset(c, 0, sizeof *c); c->owner = owner; c->fighter = fp; c->min_percent = 0; }
    if (c->owner != owner) return 0;
    if (flags & 8) {
        float mp = script_crit_fin(min_percent);
        if (!script_crit_ok(mp) || mp < 0 || mp > 999 || force < 0 || force > 255) return 0;
        c->min_percent = mp; c->force = force;
    }
    if (flags & 4) {
        float ch = script_crit_fin(chance), m = script_crit_fin(mult), mm = script_crit_fin(mult_max), l = script_crit_fin(launch);
        if (slot < 0 || slot >= SCRIPT_CRIT_SLOTS || !script_crit_ok(ch) || !script_crit_ok(m) || !script_crit_ok(mm) || !script_crit_ok(l) ||
            ch < 0 || ch > 1 || m < 1 || m > 16 || mm < m || mm > 16 || l < 1 || l > 4) return 0;
        c->chance[slot] = ch; c->mult[slot] = m; c->mult_max[slot] = mm; c->launch[slot] = l; c->has |= 1u << slot;
        c->max_mult = 1;
        for (i = 0; i < SCRIPT_CRIT_SLOTS; ++i) if ((c->has & (1u << i)) && c->mult_max[i] > c->max_mult) c->max_mult = c->mult_max[i];
    }
    return 1;
}
/* field: 0 owner, 1 has mask, 2 min_percent bits, 3 force, 4 hits, 5 crits, 6 max_mult bits; slot fields 10+slot*4+k: chance, mult, mult_max, launch. */
int ScriptGame_CritRead(int entity, int field)
{
    ScriptCritCfg* c;
    if (entity < 1 || entity > 12) return 0;
    c = &script_crit_cfg[entity-1];
    if (!c->owner || c->fighter != ScriptGame_CapsFighter(entity)) return 0;
    switch (field) {
    case 0: return c->owner; case 1: return (int) c->has; case 2: return script_skill_bits(c->min_percent);
    case 3: return c->force; case 4: return c->hits; case 5: return c->crits; case 6: return script_skill_bits(c->max_mult);
    }
    if (field >= 10 && field < 10 + SCRIPT_CRIT_SLOTS * 4) {
        int slot = (field - 10) / 4, k = (field - 10) % 4;
        return script_skill_bits(k == 0 ? c->chance[slot] : k == 1 ? c->mult[slot] : k == 2 ? c->mult_max[slot] : c->launch[slot]);
    }
    return 0;
}
void ScriptGame_CritSeed(int seed) { script_crit_rng = seed ? (unsigned) seed : 0x2545F491u; }
int ScriptGame_CritSeedRead(void) { return (int) script_crit_rng; }
void ScriptGame_CritRelease(int owner)
{
    int i;
    for (i = 0; i < 12; ++i) if (!owner || script_crit_cfg[i].owner == owner) memset(&script_crit_cfg[i], 0, sizeof script_crit_cfg[i]);
    if (!owner) { script_crit_npending = 0; memset(script_crit_launches, 0, sizeof script_crit_launches); }
}
static float script_crit_launch(HitCapsule* hit, Fighter* victim)
{
    int i;
    for (i = 0; i < 8; ++i) if (script_crit_launches[i].hit == hit && script_crit_launches[i].victim == victim) return script_crit_launches[i].launch;
    return 1.0f;
}
static void script_crit_launch_clear(Fighter* victim)
{
    int i;
    for (i = 0; i < 8; ++i) if (script_crit_launches[i].victim == victim) memset(&script_crit_launches[i], 0, sizeof script_crit_launches[i]);
}

/* Called once per accepted, damaging contact whose attacker is a fighter (a projectile passes its owner). */
void ScriptGame_CritHit(Fighter* atk, HitCapsule* hit, Fighter* vic, float damage, int projectile)
{
    int ae = atk ? ScriptGame_CapsEntity(atk) : 0, slot, forced = 0, crit = 0, k;
    ScriptCritCfg* c; float chance, mult, strength, extra;
    if (ae < 1 || !vic || vic->player_id >= 6 || !script_crit_ok(damage) || damage <= 0 || !hr_ordinary(hit->element)) return;
    c = &script_crit_cfg[ae-1];
    if (!c->owner || c->fighter != atk || !(c->has & 1u)) return;
    slot = projectile ? 9 : script_hit_family_move(atk);
    if (slot < 1 || slot >= SCRIPT_CRIT_SLOTS || !(c->has & (1u << slot))) slot = 0;
    if (vic->dmg.x1830_percent < c->min_percent) return; /* not eligible: no draw, no count */
    c->hits++;
    chance = c->chance[slot];
    {   /* one draw per eligible hit, kept even when forced or certain, so configuration never shifts the stream */
        float r = script_crit_unit();
        if (c->force > 0) { forced = 1; crit = 1; c->force--; }
        else if (r < chance) crit = 1;
    }
    if (!crit) return;
    mult = c->mult[slot];
    if (c->mult_max[slot] > mult) mult += (c->mult_max[slot] - mult) * script_crit_unit();
    extra = damage * (mult - 1.0f);
    {
        int idx = script_hit_entity(vic);
        double d = (hr_finite(script_hit_percent_delta[idx]) ? (double) script_hit_percent_delta[idx] : 0) + (double) extra;
        script_hit_percent_delta[idx] = hr_safe_float(d);
    }
    c->crits++;
    strength = c->max_mult > 1.0f ? (mult - 1.0f) / (c->max_mult - 1.0f) : 1.0f;
    if (strength < 0) strength = 0;
    if (strength > 1) strength = 1;
    if (c->launch[slot] > 1.0f)
        for (k = 0; k < 8; ++k) if (!script_crit_launches[k].hit) {
            script_crit_launches[k].hit = hit; script_crit_launches[k].victim = vic; script_crit_launches[k].launch = c->launch[slot]; break;
        }
    if (script_crit_npending < 8) {
        ScriptCritPending* p = &script_crit_pending[script_crit_npending++];
        p->attacker = atk; p->attacker_e = ae; p->victim_e = ScriptGame_CapsEntity(vic);
        p->tag = slot == 0 ? (projectile ? 9 : script_hit_family_move(atk)) : slot;
        p->forced = forced; p->projectile = projectile; p->base = damage; p->final_dmg = damage * mult; p->mult = mult;
        p->strength = strength; p->launch = c->launch[slot];
        p->x = hit->hurt_coll_pos.x; p->y = hit->hurt_coll_pos.y; p->z = hit->hurt_coll_pos.z;
    }
}
/* The victim's hitlag is only set later in the same logic frame (Fighter damage commit), so the crit event is
 * released from the frame observer, which runs after the fighters. */
static void script_crit_flush(void)
{
    int i;
    for (i = 0; i < script_crit_npending; ++i) {
        ScriptCritPending* p = &script_crit_pending[i];
        Fighter* v = ScriptGame_CapsFighter(p->victim_e);
        float hitlag = v ? v->dmg.x195c_hitlag_frames : 0.0f;
        Script_CritEvent(p->attacker_e, p->victim_e, p->tag | (p->forced ? 0x100 : 0) | (p->projectile ? 0x200 : 0), script_skill_bits(p->launch),
                         script_skill_bits(p->base), script_skill_bits(p->final_dmg), script_skill_bits(p->strength),
                         script_skill_bits(p->x), script_skill_bits(p->y), script_skill_bits(p->z), script_skill_bits(hitlag), script_skill_bits(p->mult));
    }
    script_crit_npending = 0;
}
/* Test hook: the generator's behaviour for a seed without touching the live stream. Returns the number of draws below chance. */
int ScriptGame_CritDraws(int seed, int n, int chance_bits)
{
    unsigned saved = script_crit_rng; int i, hits = 0; float chance = script_crit_fin(chance_bits);
    script_crit_rng = seed ? (unsigned) seed : 0x2545F491u;
    for (i = 0; i < n; ++i) if (script_crit_unit() < chance) ++hits;
    script_crit_rng = saved;
    return hits;
}
