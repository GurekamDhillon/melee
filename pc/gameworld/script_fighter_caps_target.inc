/* Target queries and event-carried timed values. All authoritative fields
 * remain in script_game.c's snapshotted MEM1 BSS; no Lua timer participates. */
#include <melee/gm/gmvs.h>

static Fighter* ScriptGame_CapsFighter(int entity)
{
    HSD_GObj* g;
    if (entity<1 || entity>12) return NULL;
    g=Player_GetEntityAtIndex((entity-1)%6,(entity-1)/6);
    return script_gobj_live(g) && g->user_data ? GET_FIGHTER(g) : NULL;
}
int ScriptGame_CapsEntity(Fighter* fp)
{
    int entity;
    if (!fp) return 0;
    for (entity=1;entity<=12;++entity)
        if (ScriptGame_CapsFighter(entity)==fp) return entity;
    return 0;
}
static int script_caps_opponent(Fighter* from,Fighter* to,int teams)
{
    return from && to && from!=to && from->player_id!=to->player_id &&
        (!teams || from->team!=to->team) && to->motion_id>=ftCo_MS_Wait &&
        !to->no_normal_motion;
}
static float script_caps_distance2(Fighter* a,Fighter* b)
{
    float x=a->cur_pos.x-b->cur_pos.x, y=a->cur_pos.y-b->cur_pos.y;
    return x*x+y*y;
}
/* nth=0 gives closest. nth=1..12 enumerates in entity-ID order, so equal
 * distances are stable and a query has no native scratch state to rewind. */
int ScriptGame_CapsOpponent(int entity,int radius_bits,int nth,int exclude)
{
    Fighter* from=ScriptGame_CapsFighter(entity);
    float radius=script_mesh_float(radius_bits), best=0;
    int id, result=0, count=0, teams;
    if (!from || !(radius>=0 && radius<=100000) || nth<0 || nth>12 || exclude<0 || exclude>12) return 0;
    teams=gm_8016B168()!=0;
    for (id=1;id<=12;++id) {
        Fighter* to=ScriptGame_CapsFighter(id);
        float d;
        if (id==exclude || !script_caps_opponent(from,to,teams)) continue;
        d=script_caps_distance2(from,to);
        if (!(d<=radius*radius)) continue;
        if (nth) { if (++count==nth) return id; }
        else if (!result || d<best) { best=d; result=id; }
    }
    return nth ? 0 : result;
}

typedef struct { int owner,frames; Fighter* fighter; float value; } ScriptCapsStatus;
static ScriptCapsStatus script_caps_status[12][4];
static int script_caps_status_set(ScriptCapsStatus* s,Fighter* fp,int owner,float value,int frames)
{
    if (!fp || owner<=0 || frames<0 || frames>3600 ||
        !(value>=-100000 && value<=100000) || (s->owner && s->owner!=owner)) return 0;
    memset(s,0,sizeof *s);
    if (frames) { s->owner=owner; s->frames=frames; s->fighter=fp; s->value=value; }
    return 1;
}
static void script_caps_status_tick(ScriptCapsStatus* s,Fighter* live)
{
    if (s->owner && (live!=s->fighter || --s->frames<=0)) memset(s,0,sizeof *s);
}
int ScriptGame_CapsStatus(int entity,int channel,int owner,int value_bits,int frames)
{
    int rc;
    if (entity<1 || entity>12 || channel<1 || channel>4) return 0;
    rc=script_caps_status_set(&script_caps_status[entity-1][channel-1],
        ScriptGame_CapsFighter(entity),owner,script_mesh_float(value_bits),frames);
    if (rc) OSReport("script fighter status: entity=%d channel=%d frames=%d owner=%d\n",entity,channel,frames,owner);
    return rc;
}
int ScriptGame_CapsStatusRead(int entity,int channel,int field)
{
    ScriptCapsStatus* s;
    if (entity<1 || entity>12 || channel<1 || channel>4) return 0;
    s=&script_caps_status[entity-1][channel-1];
    if (s->fighter!=ScriptGame_CapsFighter(entity)) return 0;
    if (field==0) return s->frames;
    if (field==1) return script_mesh_bits(s->value);
    if (field==2) return s->owner;
    return 0;
}
void ScriptGame_CapsStatusRelease(int owner)
{
    int e,c;
    for(e=0;e<12;++e) for(c=0;c<4;++c)
        if (!owner || script_caps_status[e][c].owner==owner)
            memset(&script_caps_status[e][c],0,sizeof script_caps_status[e][c]);
}
static void script_caps_status_frame(void)
{
    int e,c;
    for(e=0;e<12;++e) {
        Fighter* fp=ScriptGame_CapsFighter(e+1);
        for(c=0;c<4;++c) script_caps_status_tick(&script_caps_status[e][c],fp);
    }
}
#include "script_fighter_caps_target_tests.inc"
