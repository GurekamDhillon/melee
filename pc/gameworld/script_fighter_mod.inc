/* Multipliers are read overlays, never attribute edits. This BSS and the
 * player's original vanilla ratios are captured by the LAB/rollback snapshot. */
typedef struct { int owner; float value[5]; } ScriptFighterMod;
static ScriptFighterMod script_fighter_mod[6];
static void script_mod_shield_clamp(int slot);
float ScriptGame_ModValue(int slot, int field, float base)
{
    if (slot < 0 || slot >= 6 || field < 0 || field >= 5 || !script_fighter_mod[slot].owner) return base;
    return base * script_fighter_mod[slot].value[field];
}
float ScriptGame_ModDamage(int slot, int field, float base)
{
    float damage;
    if (slot<0 || slot>=6 || (field!=0 && field!=1) || !script_fighter_mod[slot].owner) return base;
    damage=ScriptGame_ModValue(slot,field,base);
    return damage>500 ? 500 : damage;
}
int ScriptGame_FighterModOwner(int slot)
{
    return slot >= 0 && slot < 6 ? script_fighter_mod[slot].owner : -1;
}
int ScriptGame_FighterModRead(int slot, int field)
{
    if (slot<0 || slot>=6 || field<0 || field>=5 || !script_fighter_mod[slot].owner) return 0;
    return script_mesh_bits(script_fighter_mod[slot].value[field]);
}
static void script_mod_shield_reconcile(Fighter* fp, float base)
{
    float cap=FT_SCRIPT_VALUE(fp,4,base);
    if (fp->shield_health>cap) fp->shield_health=cap;
}
int ScriptGame_FighterMod(int slot, int owner, int enabled, int a, int b, int c, int d, int e)
{
    ScriptFighterMod* m;
    int bits[5], i;
    float v[5];
    if (slot < 0 || slot >= 6 || owner <= 0) return 0;
    m = &script_fighter_mod[slot];
    if (m->owner && m->owner != owner) return 0;
    if (!enabled) {
        if (m->owner) OSReport("script fighter mod: cleared port=%d owner=%d\n", slot+1,owner);
        memset(m,0,sizeof *m); script_mod_shield_clamp(slot); return 1;
    }
    bits[0]=a; bits[1]=b; bits[2]=c; bits[3]=d; bits[4]=e;
    for (i=0; i<5; ++i) {
        v[i]=script_mesh_float(bits[i]);
        if (!(v[i]>=0.1f && v[i]<=4.0f)) return 0;
    }
    m->owner=owner;
    memcpy(m->value,v,sizeof v);
    script_mod_shield_clamp(slot);
    OSReport("script fighter mod: port=%d owner=%d dealt=%g taken=%g run=%g air=%g shield=%g\n",
             slot+1,owner,v[0],v[1],v[2],v[3],v[4]);
    return 1;
}
void ScriptGame_FighterModsRelease(int owner)
{
    int slot;
    for (slot=0; slot<6; ++slot) {
        ScriptFighterMod* m=&script_fighter_mod[slot];
        if (m->owner && (!owner || m->owner==owner))
            ScriptGame_FighterMod(slot,m->owner,0,0,0,0,0,0);
    }
}
#include "script_fighter_mod_tests.inc"
