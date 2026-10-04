/* Include after CapsFighter/CapsEntity. BSS is included in the LAB snapshot. */
typedef struct ScriptCapsMovement {
    int owner;
    int jumps;
    int flags;
} ScriptCapsMovement;
static ScriptCapsMovement script_caps_movement[12];

int ScriptGame_CapsMovement(int entity, int owner, int jumps, int flags)
{
    ScriptCapsMovement* value;
    if (entity < 1 || entity > 12 || owner <= 0 || jumps < -1 ||
        jumps > 8 || flags < 0 || flags > 31) return 0;
    value = &script_caps_movement[entity - 1];
    if (value->owner && value->owner != owner) return 0;
    if (jumps == -1 && flags == 0) {
        memset(value, 0, sizeof *value);
        OSReport("script fighter movement: entity=%d cleared owner=%d\n", entity, owner);
        return 1;
    }
    value->owner = owner;
    value->jumps = jumps;
    value->flags = flags;
    OSReport("script fighter movement: entity=%d owner=%d air_jumps=%d flags=%d\n",
             entity, owner, jumps, flags);
    return 1;
}

int ScriptGame_CapsMovementRead(int entity, int field)
{
    ScriptCapsMovement* value;
    if (entity < 1 || entity > 12) return field == 1 ? -1 : 0;
    if (field == 3) {
        Fighter* fp=ScriptGame_CapsFighter(entity);
        return fp ? (fp->co_attrs.max_jumps>0 ? fp->co_attrs.max_jumps-1 : 0) : -1;
    }
    value = &script_caps_movement[entity - 1];
    if (field == 0) return value->owner;
    if (field == 1) return value->owner ? value->jumps : -1;
    if (field == 2) return value->owner ? value->flags : 0;
    return 0;
}

void ScriptGame_CapsMovementRelease(int owner)
{
    int i;
    for (i = 0; i < 12; ++i) {
        if (!owner || script_caps_movement[i].owner == owner)
            memset(&script_caps_movement[i], 0, sizeof script_caps_movement[i]);
    }
}

static int script_caps_max_jumps_value(Fighter* fp, int entity)
{
    ScriptCapsMovement* value;
    if (!fp) return 0;
    if (entity < 1 || entity > 12) return fp->co_attrs.max_jumps;
    value = &script_caps_movement[entity - 1];
    /* Retail counts the initial ground jump. Preserve its landing/ledge/hit
     * counter resets; overlay the limit at every decision and exhaustion write. */
    return value->owner && value->jumps >= 0 ? value->jumps + 1 :
                                               fp->co_attrs.max_jumps;
}

static int script_caps_action_allowed_value(int entity, int flag)
{
    if (entity < 1 || entity > 12) return 1;
    return !(script_caps_movement[entity - 1].owner &&
             (script_caps_movement[entity - 1].flags & flag));
}

int ScriptGame_CapsMaxJumps(Fighter* fp)
{
    return script_caps_max_jumps_value(fp, ScriptGame_CapsEntity(fp));
}
int ScriptGame_CapsActionAllowed(Fighter* fp, int flag)
{
    return script_caps_action_allowed_value(ScriptGame_CapsEntity(fp), flag);
}
#include "script_fighter_caps_movement_tests.inc"
