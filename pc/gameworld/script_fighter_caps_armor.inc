/* Typed armour adapter. Include after CapsFighter/CapsEntity, before effects.
 * All records and latest-reaction flags live in snapshotted game BSS. */
#include "script_fighter_caps_armor_core.h"
extern void Script_ArmorEvent(int entity,int type,int absorbed,int broke,
                              int damage_bits,int kb_bits);
struct script_caps_armor_record {
    Fighter* fighter;
    int owners[6], absorbed;
    ScriptArmorCore core;
};
static struct script_caps_armor_record script_caps_armor_records[12];
static int script_caps_armor_bits(float f)
{ union { float f; int i; } value; value.f=f; return value.i; }
static float script_caps_armor_float(int i)
{ union { float f; int i; } value; value.i=i; return value.f; }
static struct script_caps_armor_record* script_caps_armor_bound(Fighter* fp)
{
    int entity = ScriptGame_CapsEntity(fp);
    struct script_caps_armor_record* r;
    if (entity < 1 || entity > 12) return NULL;
    r = &script_caps_armor_records[entity-1];
    if (r->fighter != fp) return NULL;
    return r;
}
static int script_caps_armor_side(Fighter* fp)
{
    /* ftColl_8007A06C: victim right of source -> selected dir=-1.
     * This is source/reaction-facing, not the later launch-velocity sign. */
    return fp->dmg.facing_dir_1 * fp->facing_dir >= 0 ? 1 : -1;
}
static int script_caps_armor_element(Fighter* fp)
{
    switch (fp->dmg.x1860_element) {
    case HitElement_Normal: case HitElement_Fire: case HitElement_Electric:
    case HitElement_Slash: case HitElement_Coin: case HitElement_Dark:
    case HitElement_Scball: case HitElement_Lipstick: return 1;
    default: return 0; /* Preserve ice, sleep, bury, paralysis, grabs and cape. */
    }
}
int ScriptGame_ArmorSet(int entity,int owner,int type,int value_bits,int frames,
                        int state,int from_bits,int to_bits,int direction)
{
    Fighter* fp = ScriptGame_CapsFighter(entity);
    struct script_caps_armor_record* r;
    if (!fp || owner <= 0 || type < 1 || type > 6) return 0;
    r = &script_caps_armor_records[entity-1];
    if (r->fighter != fp) memset(r,0,sizeof *r);
    if (r->owners[type-1] && r->owners[type-1] != owner) return 0;
    if (!ScriptArmorCore_Set(&r->core,type,script_caps_armor_float(value_bits),frames,
        state,script_caps_armor_float(from_bits),script_caps_armor_float(to_bits),direction)) return 0;
    r->fighter = fp; r->owners[type-1] = owner;
    OSReport("script armour: entity=%d type=%d owner=%d frames=%d\n",entity,type,owner,frames);
    return 1;
}
int ScriptGame_ArmorRead(int entity,int type,int field)
{
    Fighter* fp;
    struct script_caps_armor_record* r;
    ScriptArmorCoreSlot* s;
    if (entity < 1 || entity > 12 || type < 1 || type > 6) return 0;
    fp = ScriptGame_CapsFighter(entity); r = &script_caps_armor_records[entity-1];
    if (!fp || r->fighter != fp || !r->owners[type-1]) return 0;
    s = &r->core.slots[type-1];
    switch (field) {
    case 0: return r->owners[type-1];
    case 1: return script_caps_armor_bits(s->value);
    case 2: return script_caps_armor_bits(s->remaining);
    case 3: return s->frames;
    case 4: return s->state;
    case 5: return script_caps_armor_bits(s->from);
    case 6: return script_caps_armor_bits(s->to);
    case 7: return s->direction;
    case 8: return s->enabled;
    }
    return 0;
}
int ScriptGame_ArmorClear(int entity,int owner,int type)
{
    struct script_caps_armor_record* r;
    int i, cleared = 0;
    if (entity < 1 || entity > 12 || owner <= 0 || type < 0 || type > 6) return 0;
    r = &script_caps_armor_records[entity-1];
    for (i=0;i<6;++i) {
        if (type && type != i+1) continue;
        if (r->owners[i] && r->owners[i] != owner) continue;
        memset(&r->core.slots[i],0,sizeof r->core.slots[i]);
        r->owners[i] = 0; cleared = 1;
    }
    if (cleared) r->absorbed = 0;
    return cleared;
}
void ScriptGame_ArmorRelease(int owner)
{
    int entity,i;
    for (entity=0;entity<12;++entity) {
        struct script_caps_armor_record* r = &script_caps_armor_records[entity];
        for (i=0;i<6;++i) if (!owner || r->owners[i] == owner) {
            memset(&r->core.slots[i],0,sizeof r->core.slots[i]); r->owners[i]=0;
        }
        r->absorbed = 0;
        if (!owner) memset(r,0,sizeof *r);
    }
}
void ScriptGame_ArmorFrame(void)
{
    int entity,i;
    for (entity=0;entity<12;++entity) {
        struct script_caps_armor_record* r = &script_caps_armor_records[entity];
        int expired;
        if (!r->fighter) continue;
        if (ScriptGame_CapsFighter(entity+1) != r->fighter) { memset(r,0,sizeof *r); continue; }
        expired = ScriptArmorCore_Tick(&r->core);
        for (i=0;i<6;++i) if (expired & (1<<i)) {
            memset(&r->core.slots[i],0,sizeof r->core.slots[i]); r->owners[i]=0;
        }
    }
}
void ScriptGame_ArmorResetReaction(Fighter* fp)
{
    struct script_caps_armor_record* r = script_caps_armor_bound(fp);
    if (r) r->absorbed = 0;
}
float ScriptGame_ArmorSubtract(Fighter* fp,float kb)
{
    struct script_caps_armor_record* r = script_caps_armor_bound(fp);
    if (!r) return kb;
    r->absorbed = 0;
    if (fp->x1828 != 0 || fp->no_kb || fp->victim_gobj != NULL || !script_caps_armor_element(fp)) return kb;
    /* Called after max(retail armor0,armor1)+metal subtraction, before kb_min floor. */
    return ScriptArmorCore_Subtract(&r->core,fp->motion_id,fp->cur_anim_frame,
                                   script_caps_armor_side(fp),kb);
}
int ScriptGame_ArmorReact(Fighter* fp)
{
    struct script_caps_armor_record* r = script_caps_armor_bound(fp);
    ScriptArmorCoreEvent events[6];
    float damage;
    int i,entity,side;
    if (!r) return 0;
    r->absorbed = 0;
    if (fp->x1828 != 0 || fp->no_kb || fp->victim_gobj != NULL || fp->dmg.kb_applied <= 0) return 0;
    damage = script_caps_armor_float(fp->dmg.x186c);
    entity = ScriptGame_CapsEntity(fp); side = script_caps_armor_side(fp);
    if (script_caps_armor_element(fp)) {
        r->absorbed = ScriptArmorCore_React(&r->core,fp->motion_id,fp->cur_anim_frame,
                                          side,damage,fp->dmg.kb_applied,events);
    } else {
        /* Special retail elements report eligibility but do not spend budgets. */
        memset(events,0,sizeof events);
        for (i=0;i<6;++i) if (ScriptArmorCore_Eligible(&r->core.slots[i],fp->motion_id,fp->cur_anim_frame,side)) {
            events[i].type=i+1; events[i].damage=damage; events[i].knockback=fp->dmg.kb_applied;
        }
    }
    for (i=0;i<6;++i) if (events[i].type)
        Script_ArmorEvent(entity,events[i].type,events[i].absorbed,events[i].broke,
                          script_caps_armor_bits(events[i].damage),script_caps_armor_bits(events[i].knockback));
    return r->absorbed != 0;
}
int ScriptGame_ArmorAbsorbed(Fighter* fp)
{
    struct script_caps_armor_record* r = script_caps_armor_bound(fp);
    return r && r->absorbed != 0;
}
