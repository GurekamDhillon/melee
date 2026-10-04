/* Multipliers are read overlays, never attribute edits. This BSS and the
 * player's original vanilla ratios are captured by the LAB/rollback snapshot. */
typedef struct { int owner; float value[11]; } ScriptFighterMod;
static ScriptFighterMod script_fighter_mod[6];
static void script_mod_shield_clamp(int slot);
float ScriptGame_ModValue(int slot, int field, float base)
{
    if (slot < 0 || slot >= 6 || field < 0 || field >= 11 || !script_fighter_mod[slot].owner) return base;
    return base * script_fighter_mod[slot].value[field];
}
/* Height scales with launch velocity squared at unchanged gravity. Read the
 * live post-item/Geno attribute; never edit or restore DAT fields. */
float ScriptGame_ModJump(int slot, int field, float base)
{
    return base * sqrtf(ScriptGame_ModValue(slot,field,1.0f));
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
    if (slot<0 || slot>=6 || field<0 || field>=11 || !script_fighter_mod[slot].owner) return 0;
    return script_mesh_bits(script_fighter_mod[slot].value[field]);
}
static void script_mod_shield_reconcile(Fighter* fp, float base)
{
    float cap=FT_SCRIPT_VALUE(fp,4,base);
    if (fp->shield_health>cap) fp->shield_health=cap;
}
int ScriptGame_FighterMod(int slot, int owner, int enabled, int a, int b, int c, int d, int e, int f, int g, int h)
{
    ScriptFighterMod* m;
    int bits[8], i;
    float v[8];
    if (slot < 0 || slot >= 6 || owner <= 0) return 0;
    m = &script_fighter_mod[slot];
    if (m->owner && m->owner != owner) return 0;
    if (!enabled) {
        if (m->owner) OSReport("script fighter mod: cleared port=%d owner=%d\n", slot+1,owner);
        memset(m,0,sizeof *m); script_mod_shield_clamp(slot); return 1;
    }
    bits[0]=a; bits[1]=b; bits[2]=c; bits[3]=d; bits[4]=e; bits[5]=f; bits[6]=g; bits[7]=h;
    for (i=0; i<8; ++i) {
        v[i]=script_mesh_float(bits[i]);
        if (!(v[i]>=0.1f && v[i]<=4.0f)) return 0;
    }
    m->owner=owner;
    memcpy(m->value,v,sizeof v);
    for (i=8; i<11; ++i) m->value[i]=1.0f;
    script_mod_shield_clamp(slot);
    OSReport("script fighter mod: port=%d owner=%d dealt=%g taken=%g run=%g air=%g shield=%g jump=%g airjump=%g kb=%g\n",
             slot+1,owner,v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7]);
    return 1;
}
/* Extra overlays share the original owner and replacement lifecycle. */
int ScriptGame_FighterModExtra(int slot,int owner,int fall,int weight,int regen)
{
    int bits[3],i; float values[3];
    ScriptFighterMod* m;
    if (slot<0 || slot>=6 || owner<=0) return 0;
    m=&script_fighter_mod[slot];
    if (m->owner!=owner) return 0;
    bits[0]=fall; bits[1]=weight; bits[2]=regen;
    for (i=0;i<3;++i) {
        values[i]=script_mesh_float(bits[i]);
        if (!(values[i]>=0.1f && values[i]<=4.0f)) return 0;
    }
    for (i=0;i<3;++i) m->value[8+i]=values[i];
    return 1;
}
void ScriptGame_FighterModsRelease(int owner)
{
    int slot;
    for (slot=0; slot<6; ++slot) {
        ScriptFighterMod* m=&script_fighter_mod[slot];
        if (m->owner && (!owner || m->owner==owner))
            ScriptGame_FighterMod(slot,m->owner,0,0,0,0,0,0,0,0,0);
    }
}
#include "script_fighter_mod_tests.inc"
