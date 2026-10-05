/* Packet O: Lua binding, offline director and queue. Included after model API.
 * Native queue/cache lifetime is intentionally not snapshot-replayed. All state
 * load paths are guarded in gw_script.c while a slot session is live or changed.
 * No work is scheduled with no request/queue. Built-in covers use final post passes;
 * morph uses render-only scalar offsets and never moves collision during freeze. */
#include "gw_stage_queue.h"
extern int gw_ScriptGame_StageSlotLoad(int,int,int);
extern int gw_ScriptGame_StageSlotDynamic(int,int,int);
extern int gw_ScriptGame_StageSlotRead(int,int,int,int);
extern int gw_ScriptGame_StageSlotAt(int,int);
extern int gw_ScriptGame_StageSlotFree(int,int);
extern int gw_ScriptGame_StageSwitchCheck(int,int,int);
extern int gw_ScriptGame_StageSwitch(int,int,int,int);
extern void gw_ScriptGame_StageSlotsRelease(int);
extern int gw_ScriptGame_StageSlotsActive(void);
extern int gw_ScriptGame_StageSlotFile(int,int);
extern void gw_ScriptGame_StageMusicTick(int);
extern void gw_ScriptGame_StageMusicReset(void);
static struct {
    GwStageQueue queue;
    GwStageQueueEntry request;
    int pending,old,started,committed,post;
    double t0;
} gs_stage_director[GS_MAX_SCRIPTS+1];
static int gs_stage_directors,gs_stage_transition_owner,gs_stage_in_callback;
static const char* gs_stage_error(int rc)
{
    switch(rc) {
    case -1:return "stage slots require an offline gameplay match (no rollback/replay)";
    case -2:return "stage archive/metadata unavailable or invalid";
    case -3:return "stage slot pool full (16)";
    case -4:return "stage heap headroom or contiguous allocation unavailable";
    case -5:return "stale or foreign stage slot";
    case -6:return "host isolation or ownership unavailable";
    case -7:return "insufficient free collision lines (one safety line is reserved)";
    case -8:return "invalid fighter placement rule";
    case -9:return "fighter is captured, thrown, dead or respawning; retry later";
    case -11:return "stage DAT table/offset range invalid";
    case -12:return "stage DAT missing coll_data";
    case -13:return "stage DAT exceeds collision pool budget";
    case -14:return "stage DAT collision adjacency/group/vertex index invalid";
    case -15:return "stage DAT collision line kind invalid";
    case -16:return "stage DAT coordinates invalid or degenerate";
    case -17:return "stage DAT external patch chain invalid";
    case -18:return "dynamic stages currently support the six legal retail DATs only";
    default:return "stage slot operation refused";
    }
}
static int gs_stage_refuse(lua_State* L,int rc,int nil)
{ if(nil)lua_pushnil(L);else lua_pushboolean(L,0);lua_pushstring(L,gs_stage_error(rc));return 2; }
static void gs_stage_slot_info(lua_State* L,int h)
{
    int i,owner=gs.cur+1;
    static const char* keys[]={"slot","bytes","vertices","lines","joints","model_groups","active","music_id"};
    lua_newtable(L);
    for(i=0;i<8;++i) gs_setint(L,keys[i],gw_ScriptGame_StageSlotRead(h,owner,i,0));
    gs_setint(L,"heap_free",gw_ScriptGame_StageSlotRead(0,owner,30,0));
    gs_setint(L,"heap_capacity",gw_ScriptGame_StageSlotRead(0,owner,31,0));
    gs_setint(L,"headroom",gw_ScriptGame_StageSlotRead(0,owner,32,0));
    gs_setint(L,"model_bytes",gw_ScriptGame_StageSlotRead(h,owner,34,0));
    gs_setint(L,"dynamic_phase",gw_ScriptGame_StageSlotRead(h,owner,35,0));
    gs_setbool(L,"dynamic",gw_ScriptGame_StageSlotRead(h,owner,35,0)>0);
    gs_setbool(L,"callback_proof",gw_ScriptGame_StageSlotRead(h,owner,36,0)>0);
    gs_setint(L,"collision_capacity",gw_ScriptGame_StageSlotRead(0,owner,33,0));
    for(i=0;i<2;++i) {
        lua_newtable(L); gs_setnum(L,"left",gs_camera_float(gw_ScriptGame_StageSlotRead(h,owner,12+4*i,0)));
        gs_setnum(L,"right",gs_camera_float(gw_ScriptGame_StageSlotRead(h,owner,13+4*i,0)));
        gs_setnum(L,"top",gs_camera_float(gw_ScriptGame_StageSlotRead(h,owner,14+4*i,0)));
        gs_setnum(L,"bottom",gs_camera_float(gw_ScriptGame_StageSlotRead(h,owner,15+4*i,0)));
        lua_setfield(L,-2,i?"blast":"camera");
    }
    lua_newtable(L);
    for(i=0;i<261;++i) if(gw_ScriptGame_StageSlotRead(h,owner,8,i)) {
        lua_newtable(L);
        gs_setnum(L,"x",gs_camera_float(gw_ScriptGame_StageSlotRead(h,owner,9,i)));
        gs_setnum(L,"y",gs_camera_float(gw_ScriptGame_StageSlotRead(h,owner,10,i)));
        gs_setnum(L,"z",gs_camera_float(gw_ScriptGame_StageSlotRead(h,owner,11,i)));
        lua_rawseti(L,-2,i);
    }
    lua_setfield(L,-2,"spawns"); gs_setbool(L,"static",gw_ScriptGame_StageSlotRead(h,owner,35,0)==0);
}
static int l_stage_slot_info(lua_State* L)
{
    int h=gs_stage_handle_arg(L,1);
    if(!gw_ScriptGame_StageSlotRead(h,gs.cur+1,0,0)) {lua_pushnil(L);return 1;}
    gs_stage_slot_info(L,h);return 1;
}
static int l_stage_slots(lua_State* L)
{
    int i,n=0;{int m=lua_isboolean(L,1)?lua_toboolean(L,1):(int)lua_tointeger(L,1);if(m)gw_ScriptGame_StageSlotRead(0,0,40,m);}
    lua_newtable(L);
    for(i=0;i<16;++i) {int h=gw_ScriptGame_StageSlotAt(gs.cur+1,i);if(h){gs_stage_slot_info(L,h);lua_rawseti(L,-2,++n);}}
    gs_setint(L,"heap_free",gw_ScriptGame_StageSlotRead(0,0,30,0));
    gs_setint(L,"heap_capacity",gw_ScriptGame_StageSlotRead(0,0,31,0));
    gs_setint(L,"headroom",gw_ScriptGame_StageSlotRead(0,0,32,0));return 1;
}
static int gs_stage_kind_name(const char* name)
{
    static const struct {const char* name;int kind;} names[]={
        {"fd",37},{"final_destination",37},{"bf",36},{"battlefield",36},
        {"ys",10},{"yoshis_story",10},{"dl",28},{"dream_land",28},
        {"fod",12},{"fountain_of_dreams",12},{"ps",16},{"pokemon_stadium",16},
        {"peachs_castle",2},{"rainbow_cruise",3},{"kongo_jungle",4},{"jungle_japes",5},
        {"great_bay",6},{"temple",7},{"brinstar",8},{"brinstar_depths",9},
        {"yoshis_island",11},{"green_greens",13},{"corneria",14},{"venom",15},
        {"poke_floats",17},{"mute_city",18},{"big_blue",19},{"onett",20},
        {"fourside",21},{"icicle_mountain",22},{"mushroom_kingdom",24},
        {"mushroom_kingdom_ii",25},{"flat_zone",27},{"yoshis_island_64",29},{"kongo_jungle_64",30}};
    int i;for(i=0;i<(int)(sizeof names/sizeof names[0]);++i)if(!strcmp(name,names[i].name))return names[i].kind;
    return -1;
}
#include "gw_script_stage_mission.inc"
static int l_stage_slot_load(lua_State* L)
{
    char file[MAX_PATH],mission_file[MAX_PATH];const char* input;size_t n;int kind=-1,mission=0,h,rc,music=-1,i,dynamic=-1;
    gs_require_stage(L,"stage_slot_load");
    if(gs_stage_in_callback)return luaL_error(L,"slot lifetime cannot change inside on_stage_switch");
    if(lua_istable(L,1)) {
        lua_getfield(L,1,"dynamic");if(!lua_isnil(L,-1)){luaL_checktype(L,-1,LUA_TBOOLEAN);dynamic=lua_toboolean(L,-1);}lua_pop(L,1);
        lua_getfield(L,1,"models");i=!lua_isnil(L,-1);lua_pop(L,1);
        if(i){if(dynamic==1)return gs_stage_refuse(L,-18,1);return gs_stage_mission_load(L);}
        lua_getfield(L,1,"music_id");if(!lua_isnil(L,-1))music=(int)luaL_checkinteger(L,-1);lua_pop(L,1);
        lua_getfield(L,1,"file");
        if(lua_isnil(L,-1)){lua_pop(L,1);lua_getfield(L,1,"kind");}
    } else lua_pushvalue(L,1);
    if(lua_isinteger(L,-1)) kind=(int)lua_tointeger(L,-1);
    else {input=luaL_checklstring(L,-1,&n);if(n!=strlen(input))return luaL_error(L,"stage name contains NUL");kind=gs_stage_kind_name(input);}
    if(kind>=0) {
        for(i=0;i<(int)sizeof file-1;++i){int c=gw_ScriptGame_StageSlotFile(kind,i);file[i]=(char)c;if(!c)break;}
        file[sizeof file-1]=0;if(!file[0])return gs_stage_refuse(L,-2,1);
    } else {
        input=luaL_checklstring(L,-1,&n);
        if(n>=sizeof file || n<5 || strcmp(input+n-4,".dat"))return luaL_error(L,"unknown stage; use a vanilla name/kind or .dat file");
        memcpy(file,input,n+1);
    }
    lua_pop(L,1);n=strlen(file);mission=!strncmp(file,"missions/",9);
    if(mission ? !gs_mission_path_valid(file,n) : (n>30 || strchr(file,'/') || strchr(file,'\\') || strstr(file,"..")))
        return luaL_error(L,"stage DAT must be a root filename or a contained missions/ path");
    if(music< -1 || music>4095)return luaL_error(L,"music_id must be -1..4095");
    snprintf(mission_file,sizeof mission_file,"%s",file);snprintf(gs_stage_model_file,sizeof gs_stage_model_file,"%s",file);
    if(mission)gs_stage_archive_data=gs_mission_archive_read(L,mission_file,gs_stage_model_file,&gs_stage_archive_bytes,&gs_stage_archive_stamp);
    h=gs_stage_next_handle(L);gs_rw_branch();
    rc=gw_ScriptGame_StageSlotLoad(gs.cur+1,h,music);
    free(gs_stage_archive_data);gs_stage_archive_data=NULL;gs_stage_archive_bytes=0;gs_stage_archive_stamp=0;
    if(rc<0)return gs_stage_refuse(L,rc,1);
    rc=gw_ScriptGame_StageSlotDynamic(h,gs.cur+1,dynamic);
    if(rc<0){gw_ScriptGame_StageSlotFree(h,gs.cur+1);return gs_stage_refuse(L,rc,1);}
    ++gs_stage_session_epoch;gs_rw_stop();lua_pushinteger(L,h);return 1;
}
static int l_stage_slot_free(lua_State* L)
{
    int h=gs_stage_handle_arg(L,1),i;
    gs_require_stage(L,"stage_slot_free");
    if(gs_stage_in_callback)return luaL_error(L,"slot lifetime cannot change inside on_stage_switch");
    for(i=0;i<GS_MAX_SCRIPTS+1;++i)if(gs_stage_director[i].pending &&
        (gs_stage_director[i].request.slot==h || gs_stage_director[i].old==h))return gs_push_fail(L,"slot is in a transition");
    if(!gw_ScriptGame_StageSlotFree(h,gs.cur+1))return gs_push_fail(L,"slot is active, stale or belongs to another script");
    ++gs_stage_session_epoch;gs_rw_branch();gs_rw_stop();lua_pushboolean(L,1);return 1;
}
static void gs_stage_options(lua_State* L,int idx,GwStageQueueEntry* e)
{
    memset(e,0,sizeof *e);e->after=e->stocks=-1;e->duration=60;
    if(lua_isnoneornil(L,idx))return;
    luaL_checktype(L,idx,LUA_TTABLE);idx=lua_absindex(L,idx);
    lua_getfield(L,idx,"transition");
    if(lua_isinteger(L,-1)){e->transition=3;e->shader=gs_stage_handle_arg(L,-1);}
    else if(!lua_isnil(L,-1)) {
        const char* t=luaL_checkstring(L,-1);
        e->transition=!strcmp(t,"wipe")?0:!strcmp(t,"flash")?1:!strcmp(t,"morph")?2:-1;
        if(e->transition<0)luaL_error(L,"transition must be wipe, flash, morph or a post shader handle");
    }
    lua_pop(L,1);lua_getfield(L,idx,"place");
    if(!lua_isnil(L,-1)) {
        const char* p=luaL_checkstring(L,-1);
        if(!strcmp(p,"ko"))e->place=1;
        else if(strcmp(p,"keep") && strcmp(p,"nearest"))luaL_error(L,"place must be keep, nearest or ko");
    }
    lua_pop(L,1);lua_getfield(L,idx,"indicator");
    if(!lua_isnil(L,-1)){double t=luaL_checknumber(L,-1);if(!isfinite(t)||t<0||t>600)luaL_error(L,"indicator must be 0..600 seconds");e->indicator=(int)ceil(t*60);}
    lua_pop(L,1);lua_getfield(L,idx,"duration_frames");
    if(!lua_isnil(L,-1)){lua_Integer d=luaL_checkinteger(L,-1);if(d<1||d>600)luaL_error(L,"duration_frames must be 1..600");e->duration=(int)d;}
    lua_pop(L,1);
}
static int gs_stage_request(int script,const GwStageQueueEntry* e)
{
    int i,rc;
    if(gs_stage_transition_owner || gs_stage_director[script].pending)return -6;
    rc=gw_ScriptGame_StageSwitchCheck(e->slot,script+1,e->place);if(rc)return rc;
    gs_stage_director[script].old=0;
    for(i=0;i<16;++i) {
        int h=gw_ScriptGame_StageSlotAt(script+1,i);
        if(h && gw_ScriptGame_StageSlotRead(h,script+1,6,0))gs_stage_director[script].old=h;
    }
    gs_stage_director[script].request=*e;gs_stage_director[script].pending=1;
    gs_stage_transition_owner=script+1;gs_stage_directors=1;return 0;
}
static int l_stage_switch(lua_State* L)
{
    GwStageQueueEntry e;int h=gs_stage_handle_arg(L,1),rc;
    gs_require_stage(L,"stage_switch");if(gs_stage_in_callback)return gs_push_fail(L,"recursive stage switch refused");
    gs_stage_options(L,2,&e);e.slot=h;rc=gs_stage_request(gs.cur,&e);
    if(rc)return gs_stage_refuse(L,rc,0);
    lua_pushboolean(L,1);return 1;
}
static int l_stage_queue_clear(lua_State* L)
{
    GwStageQueue* q=&gs_stage_director[gs.cur].queue;
    gs_require_stage(L,"stage_queue_clear");free(q->entries);memset(q,0,sizeof *q);lua_pushboolean(L,1);return 1;
}
static int l_stage_queue(lua_State* L)
{
    GwStageQueue q={0};size_t i,n;
    gs_require_stage(L,"stage_queue");luaL_checktype(L,1,LUA_TTABLE);n=lua_rawlen(L,1);
    if(gs_stage_director[gs.cur].pending)return gs_push_fail(L,"cannot replace queue during a stage transition");
    if(n>SIZE_MAX/sizeof *q.entries)return luaL_error(L,"queue allocation overflow");
    /* Validate into Lua-owned userdata first: an option error cannot leak a C
     * allocation or partially replace the current queue. */
    q.entries=(GwStageQueueEntry*)lua_newuserdatauv(L,n*sizeof *q.entries,0);q.count=n;q.start=gs.match_frame;
    for(i=0;i<n;++i) {
        GwStageQueueEntry* e=&q.entries[i];int trigger=0;
        lua_rawgeti(L,1,i+1);gs_stage_options(L,-1,e);
        lua_getfield(L,-1,"slot");e->slot=gs_stage_handle_arg(L,-1);lua_pop(L,1);
        if(!gw_ScriptGame_StageSlotRead(e->slot,gs.cur+1,0,0))return luaL_error(L,"queue entry has a stale or foreign slot");
        lua_getfield(L,-1,"after");if(!lua_isnil(L,-1)){double t=luaL_checknumber(L,-1);if(!isfinite(t)||t<0||t>35791394)luaL_error(L,"after exceeds logic-frame range");e->after=(int)ceil(t*60);++trigger;}lua_pop(L,1);
        lua_getfield(L,-1,"at_stocks");if(!lua_isnil(L,-1)){lua_Integer s=luaL_checkinteger(L,-1);if(s<0||s>594)luaL_error(L,"at_stocks must be 0..594 total remaining stocks");e->stocks=(int)s;++trigger;}lua_pop(L,1);
        lua_getfield(L,-1,"on");if(!lua_isnil(L,-1)){size_t len;const char* name=luaL_checklstring(L,-1,&len);if(!len||len>=64||len!=strlen(name))luaL_error(L,"event name must be 1..63 bytes");memcpy(e->event,name,len+1);++trigger;}lua_pop(L,1);
        if(trigger!=1)return luaL_error(L,"queue entry needs exactly one of after, at_stocks, on");
        lua_pop(L,1);
    }
    lua_getfield(L,1,"loop");q.loop=lua_toboolean(L,-1);lua_pop(L,1);
    lua_getfield(L,1,"shuffle");q.shuffle=lua_toboolean(L,-1);lua_pop(L,1);
    lua_getfield(L,1,"seed");if(!lua_isnil(L,-1)){lua_Integer s=luaL_checkinteger(L,-1);if(s<0 || (lua_Unsigned)s>UINT32_MAX)luaL_error(L,"seed must be uint32");q.seed=(unsigned)s;}lua_pop(L,1);
    {
        GwStageQueueEntry* entries=n?(GwStageQueueEntry*)malloc(n*sizeof *entries):NULL;
        if(n&&!entries)return gs_push_fail(L,"queue allocation unavailable");
        if(n)memcpy(entries,q.entries,n*sizeof *entries);q.entries=entries;
    }
    gw_stage_shuffle(&q);free(gs_stage_director[gs.cur].queue.entries);gs_stage_director[gs.cur].queue=q;
    gs_stage_directors=1;lua_pushboolean(L,1);return 1;
}
static int l_stage_queue_next(lua_State* L)
{
    GwStageQueue* q=&gs_stage_director[gs.cur].queue;gs_require_stage(L,"stage_queue_next");
    if(!q->entries || q->next>=q->count)return gs_push_fail(L,"stage queue is empty");
    q->manual=1;lua_pushboolean(L,1);return 1;
}
static int l_stage_queue_event(lua_State* L)
{
    const char* name=luaL_checkstring(L,1);gs_require_stage(L,"stage_queue_event");
    gw_stage_signal(&gs_stage_director[gs.cur].queue,name);return 0;
}
static void gs_stage_signal_all(const char* name)
{ int i;if(!gs_stage_directors)return;for(i=0;i<gs.n;++i)gw_stage_signal(&gs_stage_director[i].queue,name); }
static void gs_stage_hook(const char* phase,int from,int to)
{
    int i;gs_stage_in_callback=1;
    if(!strcmp(phase,"after"))gs_zones_release(0); /* old level exits precede new level hooks */
    for(i=0;i<gs.n;++i) if(gs_get_hook(i,"on_stage_switch")) {
        lua_newtable(gs.L);gs_setstr(gs.L,"phase",phase);gs_setint(gs.L,"from",from);gs_setint(gs.L,"slot",to);
        gs_pcall(i,1,0,"on_stage_switch");
    }
    gs_stage_in_callback=0;
}
static void gs_stage_cancel(int script)
{
    int i,owner=gs.s[script].stage_owner;
    if(gs_stage_director[script].post)gw_Post_Remove(owner,gs_stage_director[script].post);
    if(gs_stage_director[script].started)for(i=0;i<GS_MAX_SCRIPTS+1;++i)gw_presentation_cancel(&gs_hitstop[i],owner);
    gs_stage_director[script].pending=gs_stage_director[script].started=gs_stage_director[script].committed=gs_stage_director[script].post=0;
    if(gs_stage_transition_owner==script+1)gs_stage_transition_owner=0;
}
static void gs_stage_slots_release(int script)
{
    if(script<0 || script>GS_MAX_SCRIPTS)return;
    gs_stage_cancel(script);free(gs_stage_director[script].queue.entries);
    memset(&gs_stage_director[script],0,sizeof gs_stage_director[script]);
    if(gw_ScriptGame_StageSlotAt(script+1,0) || gw_ScriptGame_StageSlotsActive())++gs_stage_session_epoch;
    gw_ScriptGame_StageSlotsRelease(script+1);
}
void gw_Script_StageSlotsEnded(void)
{
    gw_ScriptGame_StageMusicReset();
    int i;for(i=0;i<GS_MAX_SCRIPTS+1;++i){gs_stage_cancel(i);free(gs_stage_director[i].queue.entries);}
    memset(gs_stage_director,0,sizeof gs_stage_director);gs_stage_directors=gs_stage_transition_owner=0;
}
static void gs_stage_logic_tick(void)
{
    int i,s,stocks=0,oldcur=gs.cur;
    if(!gs_stage_directors)return;
    for(s=0;s<6;++s)if(gw_ScriptGame_FighterI(s,SI_PRESENT))stocks+=gw_ScriptGame_FighterI(s,SI_STOCKS);
    gs_stage_directors=0;
    for(i=0;i<gs.n;++i) {
        GwStageQueue* q=&gs_stage_director[i].queue;
        if(!gs.s[i].used || gs.s[i].disabled)continue;
        if(q->entries && q->next<q->count) {
            gs_stage_directors=1;
            if(!gs_stage_director[i].pending && gw_stage_due(q,gs.match_frame,stocks)) {
                int rc=gs_stage_request(i,&q->entries[q->next]);
                if(rc && rc!=-9) {gw_log("stage queue [%s] stopped: %s",gs_script_id(i),gs_stage_error(rc));free(q->entries);memset(q,0,sizeof *q);}
            }
        }
        if(gs_stage_director[i].pending) {gs_stage_directors=1;if(gs_stage_director[i].request.indicator>0)--gs_stage_director[i].request.indicator;}
    }
    gs.cur=oldcur;
}
/* Between logic frames, also during timed freeze. Wall time advances the visual
 * envelope only. The accepted request was scheduled on a logic frame. */
static void gs_stage_presentation_tick(void)
{
    int script=gs_stage_transition_owner-1,oldcur=gs.cur,rc,i,owner;
    double elapsed;
    if(script<0)return;
    if(!gs.match_active || gw_RB_Enabled() || gw_Netplay_Enabled()) {gs_stage_slots_release(script);return;}
    if(gs_stage_director[script].request.indicator>0)return;
    gs.cur=script;owner=gs.s[script].stage_owner;
    if(!gs_stage_director[script].started) {
        int timer=-1;GwStageQueueEntry* e=&gs_stage_director[script].request;
        rc=gw_ScriptGame_StageSwitchCheck(e->slot,script+1,e->place);
        if(rc){gw_log("stage switch: %s",gs_stage_error(rc));gs_stage_cancel(script);gs.cur=oldcur;return;}
        if(e->transition==3 || e->transition<2) {
            char error[2048]="";
            gs_stage_director[script].post=e->transition==3?
                gw_Post_Add(owner,e->shader,100,1,0,0,error,sizeof error):
                gw_Post_StageCover(owner,e->transition==1,error,sizeof error);
            if(!gs_stage_director[script].post){gw_log("stage switch: shader refused: %s",error);gs_stage_cancel(script);gs.cur=oldcur;return;}
            gw_Post_Protect(owner,gs_stage_director[script].post);
        }
        for(i=0;i<GS_MAX_SCRIPTS+1;++i)if(gs_hitstop[i].owner==owner || !gs_hitstop[i].used){timer=i;break;}
        if(timer<0){gs_stage_cancel(script);gs.cur=oldcur;return;}
        gs_stage_director[script].t0=gs_now_ms();gs_stage_director[script].started=1;
        gw_presentation_start(&gs_hitstop[timer],owner,gs_stage_director[script].t0,e->duration+2);gs_presentation_active|=1;
    }
    elapsed=(gs_now_ms()-gs_stage_director[script].t0)*60.0/1000.0;
    if(gs_stage_director[script].request.transition<2 && gs_stage_director[script].post) {
        char params[96],error[256]="";
        double t=elapsed/gs_stage_director[script].request.duration;
        if(t>1)t=1;if(t<0)t=0;
        snprintf(params,sizeof params,"{\"progress\":%.7f}",t);
        if(!gw_Post_Set(owner,gs_stage_director[script].post,params,error,sizeof error)) {
            gw_log("stage transition: cover update refused: %s",error);
            gs_stage_cancel(script);gs.cur=oldcur;return;
        }
    }
    if(!gs_stage_director[script].committed && elapsed>=gs_stage_director[script].request.duration/2.0) {
        GwStageQueueEntry e=gs_stage_director[script].request;int from=gs_stage_director[script].old;
        gs_stage_hook("before",from,e.slot);
        if(!gs_stage_director[script].pending){gs.cur=oldcur;return;}
        if(gs_stage_handle_serial>INT_MAX-769){gs_stage_cancel(script);gs.cur=oldcur;return;}
        gs.cur=script;rc=gw_ScriptGame_StageSwitchCheck(e.slot,script+1,e.place);
        if(!rc) {
            gs_stage_isolation_owner=script+1;
            gs_rw_branch(); /* exactly one fork for the atomic switch */
            rc=gw_ScriptGame_StageSwitch(e.slot,script+1,gs_stage_handle_serial+1,e.place);
            gs_stage_handle_serial+=769;
        }
        if(rc){gw_log("stage switch: commit refused: %s",gs_stage_error(rc));gs_stage_cancel(script);gs.cur=oldcur;return;}
        ++gs_stage_session_epoch;gs_rw_stop();gs_stage_director[script].committed=1;
        gs_stage_hook("after",from,e.slot);
    }
    if(gs_stage_director[script].pending && elapsed>=gs_stage_director[script].request.duration) {
        if(gs_stage_director[script].committed && gs_stage_director[script].queue.entries)
            gw_stage_advance(&gs_stage_director[script].queue,gs.match_frame);
        gs_stage_cancel(script);
    }
    gs.cur=oldcur;
}
int gw_Script_StageSlotVisual(int handle)
{
    int script=gs_stage_transition_owner-1;
    if(script>=0 && gs_stage_director[script].started && gs_stage_director[script].request.transition==2) {
        float t=(float)((gs_now_ms()-gs_stage_director[script].t0)*60.0/(1000.0*gs_stage_director[script].request.duration));
        if(t<0)t=0;if(t>1)t=1;
        if(handle==gs_stage_director[script].request.slot)return gs_fbits(-180*(1-t));
        if(handle==gs_stage_director[script].old)return gs_fbits(-180*t);
    }
    return 0x7fc00000; /* no transition override; active selection stays game-owned */
}
/* Queried by the overlay every rendered frame, including frozen logic. */
static const GwScriptDraw* gs_stage_indicator_draw(void)
{
    static GwScriptDraw d;
    int script=gs_stage_transition_owner-1;
    GwStageQueueEntry* e;
    if(script<0)return NULL;
    e=&gs_stage_director[script].request;
    if(e->indicator<=0)return NULL;
    memset(&d,0,sizeof d);d.kind=GW_SDRAW_TEXT;d.size=1.25f;
    d.x=gw_Console_ScriptWidth()/2-115;d.y=32;d.rgba=0xffffffff;
    snprintf(d.text,sizeof d.text,"Stage switch in %.1fs",e->indicator/60.0);
    return &d;
}
