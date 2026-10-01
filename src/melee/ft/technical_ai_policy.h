/* Original bounded technical input policy. No imported training/20XX code.
 * Pure observations -> a shoulder pulse; no action/physics/timer mutation. */
#ifndef MELEE_TECHNICAL_AI_POLICY_H
#define MELEE_TECHNICAL_AI_POLICY_H

typedef struct TechAiPolicy {
    unsigned seed;
    int skill, delay, wait, type, motion, fired, accepted, cooldown;
    int opportunities, lcancels, techs, misses, last_event;
} TechAiPolicy;
typedef struct TechAiObservation {
    int type, motion, floor, fresh_shoulder, tech_unlocked;
    float eta;
} TechAiObservation;

static void tech_ai_policy_init(TechAiPolicy* s, int skill, unsigned seed)
{
    TechAiPolicy empty = { 0 };
    *s = empty;
    s->skill = skill;
    s->seed = seed;
    s->delay = skill == 1 ? 6 : skill == 2 ? 4 : 2;
}

static unsigned tech_ai_random(TechAiPolicy* s)
{
    unsigned x = s->seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    s->seed = x;
    return x;
}

/* type 1: light analogue L-cancel; type 2: digital ground-tech attempt.
 * One attempt per opportunity, then a cooldown. The physics engine judges it. */
static int tech_ai_policy_step(TechAiPolicy* s, TechAiObservation const* o)
{
    int chance, event;
    if (s->skill < 1 || s->skill > 3) return 0;
    if (s->cooldown > 0) --s->cooldown;
    if (!o->floor || o->type < 1 || o->type > 2 ||
        !(o->eta > 0.0f && o->eta <= 12.0f))
    {
        s->type = 0;
        s->fired = 0;
        return 0;
    }
    if (s->type != o->type || s->motion != o->motion) {
        s->type = o->type;
        s->motion = o->motion;
        s->wait = s->delay;
        s->fired = 0;
        chance = s->skill == 1 ? 60 : s->skill == 2 ? 80 : 95;
        s->accepted = (int) (tech_ai_random(s) % 100) < chance;
        ++s->opportunities;
        if (!s->accepted) ++s->misses;
    }
    if (s->fired || !s->accepted) return 0;
    if (s->wait > 0) { --s->wait; return 0; }
    if (s->cooldown > 0 || !o->fresh_shoulder) return 0;
    if (o->type == 1 && o->eta > 5.0f) return 0;
    if (o->type == 2 && (!o->tech_unlocked || o->eta > 10.0f)) return 0;
    event = o->type;
    s->fired = 1;
    s->last_event = event;
    s->cooldown = event == 1 ? 8 : 40;
    if (event == 1) ++s->lcancels; else ++s->techs;
    return event;
}
#endif
