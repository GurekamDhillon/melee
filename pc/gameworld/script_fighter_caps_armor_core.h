#ifndef SCRIPT_FIGHTER_CAPS_ARMOR_CORE_H
#define SCRIPT_FIGHTER_CAPS_ARMOR_CORE_H
/* Portable deterministic policy. No game pointers, callbacks, allocation, or clocks.
 * Types are one-based; masks use bit(type-1). Caller supplies retail computed KB. */
typedef struct ScriptArmorCoreSlot {
    int enabled, frames, state, direction;
    float value, remaining, from, to;
} ScriptArmorCoreSlot;
typedef struct ScriptArmorCore { ScriptArmorCoreSlot slots[6]; } ScriptArmorCore;
typedef struct ScriptArmorCoreEvent {
    int type, absorbed, broke;
    float damage, knockback;
} ScriptArmorCoreEvent;

static int ScriptArmorCore_Set(ScriptArmorCore* core, int type, float value,
                               int frames, int state, float from, float to, int direction)
{
    ScriptArmorCoreSlot* s;
    if (!core || type < 1 || type > 6 || frames < 0 || frames > 36000 ||
        state < -1 || state > 65535 || direction < -1 || direction > 1 ||
        !(value >= 0 && value <= 1000) || (type != 4 && value <= 0) ||
        (type == 5 && (value > 255 || value != (int) value)) ||
        !(from == -1 || (from >= 0 && from <= 100000)) ||
        !(to == -1 || (to >= 0 && to <= 100000)) ||
        (from >= 0 && to >= 0 && from > to)) return 0;
    s = &core->slots[type-1];
    s->enabled = 1; s->frames = frames; s->state = state; s->direction = direction;
    s->value = value; s->remaining = value; s->from = from; s->to = to;
    return 1;
}
static int ScriptArmorCore_Eligible(const ScriptArmorCoreSlot* s, int state,
                                    float anim, int side)
{
    return s && s->enabled && (s->state < 0 || s->state == state) &&
        (s->from < 0 || anim >= s->from) && (s->to < 0 || anim <= s->to) &&
        (!s->direction || s->direction == side);
}
static float ScriptArmorCore_Subtract(const ScriptArmorCore* core, int state,
                                      float anim, int side, float kb)
{
    const ScriptArmorCoreSlot* s = &core->slots[0];
    return ScriptArmorCore_Eligible(s,state,anim,side) ? kb-s->value : kb;
}
static int ScriptArmorCore_React(ScriptArmorCore* core, int state, float anim,
                                int side, float damage, float kb,
                                ScriptArmorCoreEvent events[6])
{
    int i, mask = 0;
    /* Visit every eligible type in fixed order. Earlier protection does not
     * hide contacts from finite hit-count or damage-pool budgets. */
    for (i=0;i<6;++i) {
        ScriptArmorCoreSlot* s = &core->slots[i];
        ScriptArmorCoreEvent* e = &events[i];
        e->type = 0; e->absorbed = 0; e->broke = 0;
        e->damage = damage; e->knockback = kb;
        if (!ScriptArmorCore_Eligible(s,state,anim,side)) continue;
        e->type = i+1;
        switch (i+1) {
        case 1: break; /* Subtraction reduces KB; it never promises no flinch. */
        case 2: e->absorbed = damage < s->value; break;
        case 3: e->absorbed = kb < s->value; break;
        case 4: e->absorbed = 1; break;
        case 5:
            e->absorbed = s->remaining > 0;
            if (e->absorbed) {
                s->remaining -= 1;
                if (s->remaining <= 0) { s->remaining = 0; s->enabled = 0; e->broke = 1; }
            }
            break;
        case 6:
            e->absorbed = damage < s->remaining;
            if (damage > 0) s->remaining -= damage;
            if (s->remaining <= 0) { s->remaining = 0; s->enabled = 0; e->broke = 1; }
            break;
        }
        if (e->absorbed) mask |= 1<<i;
    }
    return mask;
}
static int ScriptArmorCore_Tick(ScriptArmorCore* core)
{
    int i, expired = 0;
    for (i=0;i<6;++i) {
        ScriptArmorCoreSlot* s = &core->slots[i];
        if (s->frames > 0 && --s->frames == 0) {
            s->enabled = 0;
            expired |= 1<<i;
        }
    }
    return expired;
}
#endif
