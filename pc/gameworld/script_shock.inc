/* Shock: a real status (included at the end of script_crit.inc, itself part of script_skill.inc / script_game.c).
 *
 * The design spec (docs/superpowers/specs/2026-10-04-envoy-loot-and-modifiers-design.md, "Statuses") defines Shock as
 * "next hit taken has more hitstun". That is what this is: while a fighter is shocked, the next `charges` hits it TAKES
 * that put it into a damage reaction (ftCo_Damage_Enter...: the reaction frame count mv.co.damage.x0, the same count
 * Geno's HBSTUN bonus extends) get extra hitstun frames
 *     extra = round(base * (mult - 1)) + bonus   (capped at SHOCK_MAX_EXTRA)
 * and each such hit uses up one charge; the status also ends when its `frames` run out or the fighter object is replaced
 * (respawn). Shock never changes damage, knockback or launch, only how long the fighter stays in the reaction, so it
 * composes with crits, armour (an armoured hit enters no reaction: it does not consume a charge) and hit rules.
 *
 * Determinism and rewind: every field is plain data in this translation unit's snapshotted BSS, there is no RNG and no
 * wall clock; the timer ticks once per logic frame in ScriptGame_SkillFrame, the hit is consumed where retail computes the
 * reaction (a collision-pass order the resimulation repeats). Configuration is a gameplay write (offline, owner-scoped,
 * forks the timeline) or a journal op ({op="shock"} in gd.sim_commit), so a replay reapplies it exactly. Online the table
 * is empty. Events go to the skill ring and to on_skill / on_shock / on_shock_hit / on_shock_end:
 *   shock      i1 frames, i2 charges, i3 extra flat frames, f1 hitstun multiplier   (applied or refreshed)
 *   shock_hit  i1 base hitstun frames, i2 extra frames added, i3 charges left, f1 multiplier
 *   shock_end  i1 why (1 expired, 2 spent its charges, 3 cleared, 4 fighter replaced) */
#define SCRIPT_SHOCK_MAX_FRAMES 3600
#define SCRIPT_SHOCK_MAX_CHARGES 8
#define SCRIPT_SHOCK_MAX_EXTRA 120
typedef struct { int owner; Fighter* fighter; int frames, charges, bonus, hits, applied; float mult; } ScriptShock;
static ScriptShock script_shock[12];

static void script_shock_end(int entity, ScriptShock* s, int why)
{
    OSReport("script shock: entity=%d end why=%d hits=%d\n", entity, why, s->hits);
    script_skill_emit(SK_SHOCK_END, entity, why, s->hits, 0, s->mult, 0);
    memset(s, 0, sizeof *s);
}

/* flags: 1 apply (replace), 2 apply and stack (add charges, refresh frames; the larger multiplier and bonus win),
 * 4 clear. Returns 0 on an owner conflict, a missing fighter or a bad argument, 1 otherwise. */
int ScriptGame_ShockSet(int entity, int owner, int frames, int charges, int mult_bits, int bonus, int flags)
{
    ScriptShock* s; Fighter* fp; float mult;
    union { int i; float f; } u;
    if (entity < 1 || entity > 12 || owner <= 0) return 0;
    fp = ScriptGame_CapsFighter(entity); s = &script_shock[entity-1];
    if (s->owner && s->fighter == fp && s->owner != owner) return 0;
    if (s->owner && s->fighter != fp) memset(s, 0, sizeof *s);
    if (flags & 4) {
        if (s->owner == owner) script_shock_end(entity, s, 3);
        return 1;
    }
    u.i = mult_bits; mult = u.f;
    if (!fp || frames < 1 || frames > SCRIPT_SHOCK_MAX_FRAMES || charges < 1 || charges > SCRIPT_SHOCK_MAX_CHARGES ||
        bonus < 0 || bonus > SCRIPT_SHOCK_MAX_EXTRA || !(mult >= 1.0f && mult <= 4.0f)) return 0;
    if ((flags & 2) && s->owner == owner) {
        s->charges += charges; if (s->charges > SCRIPT_SHOCK_MAX_CHARGES) s->charges = SCRIPT_SHOCK_MAX_CHARGES;
        if (frames > s->frames) s->frames = frames;
        if (mult > s->mult) s->mult = mult;
        if (bonus > s->bonus) s->bonus = bonus;
    } else {
        memset(s, 0, sizeof *s);
        s->owner = owner; s->fighter = fp; s->frames = frames; s->charges = charges; s->mult = mult; s->bonus = bonus;
    }
    s->applied++;
    OSReport("script shock: entity=%d frames=%d charges=%d mult=%f bonus=%d owner=%d\n", entity, s->frames, s->charges, (double) s->mult, s->bonus, owner);
    script_skill_emit(SK_SHOCK, entity, s->frames, s->charges, s->bonus, s->mult, 0);
    return 1;
}
/* field: 0 owner, 1 frames, 2 charges, 3 bonus, 4 multiplier bits, 5 hits consumed, 6 applications. */
int ScriptGame_ShockRead(int entity, int field)
{
    ScriptShock* s;
    union { int i; float f; } u;
    if (entity < 1 || entity > 12) return 0;
    s = &script_shock[entity-1];
    if (!s->owner || s->fighter != ScriptGame_CapsFighter(entity)) return 0;
    switch (field) {
    case 0: return s->owner; case 1: return s->frames; case 2: return s->charges; case 3: return s->bonus;
    case 4: u.f = s->mult; return u.i;
    case 5: return s->hits; case 6: return s->applied;
    }
    return 0;
}
void ScriptGame_ShockRelease(int owner)
{
    int i;
    for (i = 0; i < 12; ++i) if (!owner || script_shock[i].owner == owner) memset(&script_shock[i], 0, sizeof script_shock[i]);
}
/* The extra frames for a hit of `base` hitstun frames; also the test hook (the hit path needs a live match). */
int ScriptGame_ShockExtra(int base, int mult_bits, int bonus)
{
    union { int i; float f; } u;
    int extra;
    u.i = mult_bits;
    if (!(u.f >= 1.0f && u.f <= 4.0f) || bonus < 0 || base < 0) return 0;
    extra = (int) ((float) base * (u.f - 1.0f) + 0.5f) + bonus;
    return extra > SCRIPT_SHOCK_MAX_EXTRA ? SCRIPT_SHOCK_MAX_EXTRA : extra;
}
/* Called where the damage reaction's hitstun is known (ftCo_Damage.c, next to Geno_TakeStunBonus). */
int ScriptGame_ShockTake(Fighter* vic, int base)
{
    int e = ScriptGame_CapsEntity(vic), extra;
    ScriptShock* s;
    if (e < 1) return 0;
    s = &script_shock[e-1];
    if (!s->owner || s->fighter != vic || s->charges < 1) return 0;
    { union { int i; float f; } u; u.f = s->mult; extra = ScriptGame_ShockExtra(base, u.i, s->bonus); }
    s->hits++; s->charges--;
    OSReport("script shock: entity=%d hit base=%d extra=%d charges_left=%d\n", e, base, extra, s->charges);
    script_skill_emit(SK_SHOCK_HIT, e, base, extra, s->charges, s->mult, 0);
    if (s->charges < 1) script_shock_end(e, s, 2);
    return extra;
}
/* Once per logic frame (ScriptGame_SkillFrame). */
static void script_shock_frame(void)
{
    int e;
    for (e = 1; e <= 12; ++e) {
        ScriptShock* s = &script_shock[e-1];
        Fighter* fp;
        if (!s->owner) continue;
        fp = ScriptGame_CapsFighter(e);
        if (fp != s->fighter) { script_shock_end(e, s, 4); continue; }
        if (--s->frames <= 0) script_shock_end(e, s, 1);
    }
}
