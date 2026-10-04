/* Offline 1P spawn templates. Port modifiers intentionally include a follower;
 * a replacement main entity starts from a cleared overlay and receives its own template. */
typedef struct { int owner, enabled, value[8], tint; } ScriptOnePTemplate;
static ScriptOnePTemplate script_onep_template[6];
static int script_onep_generation[6];
int ScriptGame_OnePI(int field, int slot)
{
    if (field == 0) return gm_GetCurrentSceneIndex() >> 3;
    if (field == 1) return Player_GetTeam(slot);
    if (field == 2) return script_onep_generation[slot];
    return 0;
}
void ScriptGame_OnePTemplatesClear(void)
{
    memset(script_onep_template, 0, sizeof script_onep_template);
}
int ScriptGame_OnePTemplate(int slot, int owner, int enabled, int a, int b,
                           int c, int d, int e, int tint, int f, int g, int h)
{
    ScriptOnePTemplate* t;
    if (slot < 0 || slot >= 6 || owner <= 0) return 0;
    if (enabled) {
        int i,bits[8];
        bits[0]=a;bits[1]=b;bits[2]=c;bits[3]=d;bits[4]=e;bits[5]=f;bits[6]=g;bits[7]=h;
        for(i=0;i<8;++i) {
            float v=script_mesh_float(bits[i]);
            if(!(v>=.1f&&v<=4.0f))return 0;
        }
    }
    t = &script_onep_template[slot];
    if (t->owner && t->owner != owner) return 0;
    t->owner = enabled ? owner : 0; t->enabled = enabled;
    t->value[0]=a; t->value[1]=b; t->value[2]=c; t->value[3]=d; t->value[4]=e; t->value[5]=f; t->value[6]=g; t->value[7]=h;
    t->tint=tint;
    return 1;
}
extern void Script_OnePSpawn(int port, int generation);
extern int Script_OnePOffline(void);
void ScriptGame_OnePSpawnTint(Fighter* fp,int owner,int rgb);
void ScriptGame_OnePSpawn(Fighter* fp)
{
    ScriptOnePTemplate* t;
    int slot = fp->player_id;
    if (!Script_OnePOffline() || slot < 0 || slot >= 6 || fp->is_sub_fighter ||
        (gm_GetCurrentGameMode()!=GM_CLASSIC && gm_GetCurrentGameMode()!=GM_ADVENTURE)) return;
    t=&script_onep_template[slot];
    memset(&script_fighter_mod[slot],0,sizeof script_fighter_mod[slot]);
    ++script_onep_generation[slot];
    if (t->enabled) {
        int i;
        script_fighter_mod[slot].owner=t->owner;
        for(i=0;i<8;++i)script_fighter_mod[slot].value[i]=script_mesh_float(t->value[i]);
        if(script_fighter_mod[slot].value[4]<1.0f)
            fp->shield_health*=script_fighter_mod[slot].value[4];
        if(t->tint>=0)ScriptGame_OnePSpawnTint(fp,t->owner,t->tint);
    }
    Script_OnePSpawn(slot,script_onep_generation[slot]);
}

void ScriptGame_OnePTemplatesRelease(int owner)
{
    int s;
    for(s=0;s<6;++s) if(script_onep_template[s].owner==owner)
        memset(&script_onep_template[s],0,sizeof script_onep_template[s]);
}

int ScriptGame_OnePTemplateTest(void)
{
    ScriptOnePTemplate saved[6];int one=script_mesh_bits(1.0f),bad=0;
    memcpy(saved,script_onep_template,sizeof saved);ScriptGame_OnePTemplatesClear();
    if(!ScriptGame_OnePTemplate(2,7,1,one,one,one,one,one,-1,script_mesh_bits(1),script_mesh_bits(1),script_mesh_bits(1)))bad=1;
    if(ScriptGame_OnePTemplate(2,8,1,one,one,one,one,one,-1,script_mesh_bits(1),script_mesh_bits(1),script_mesh_bits(1)))bad=1;
    if(ScriptGame_OnePTemplate(2,7,1,0x7fc00000,one,one,one,one,-1,script_mesh_bits(1),script_mesh_bits(1),script_mesh_bits(1)))bad=1;
    ScriptGame_OnePTemplatesRelease(8);
    if(script_onep_template[2].owner!=7)bad=1;
    ScriptGame_OnePTemplatesRelease(7);
    if(script_onep_template[2].owner||script_onep_template[2].enabled)bad=1;
    memcpy(script_onep_template,saved,sizeof saved);return bad;
}
