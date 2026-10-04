/* Native diagnostic jobs; offline, scene-scoped, never consulted by simulation. */
#include <aurora/gfx.h>
extern int gw_ScriptGame_WarmDraw(int category, int index);
extern int gw_ScriptGame_WarmModel(int model);
extern int gw_ScriptGame_WarmItemReady(int kind);
#define GS_WARM_JOBS 16
#define GS_WARM_OBJECTS 128
typedef struct { int kind, value; } GsWarmObject;
typedef struct {
    int token, owner, epoch, count, cursor, failed;
    GsWarmObject objects[GS_WARM_OBJECTS];
    unsigned captures[GS_WARM_OBJECTS];
} GsWarmJob;
static GsWarmJob gs_warm_jobs[GS_WARM_JOBS];
static int gs_warm_serial;
extern void AuroraGXSync(void);
void gw_Script_WarmSync(void) { AuroraGXSync(); }

void gw_Script_WarmModelDraw(int model, void* view)
{
    float identity[3][4] = {{1,0,0,0},{0,1,0,0},{0,0,1,0}};
    int alpha, mirrored;
    gs_model_batch_count = 0;
    for (alpha = 0; alpha < 2; ++alpha) for (mirrored = 0; mirrored < 2; ++mirrored) {
        identity[0][0] = mirrored ? -1 : 1;
        gs_model_bound = NULL;
        gs_model_draw(model + 1, view, identity, alpha ? 0xffffff80u : 0xffffffffu, alpha);
        // Instanced GXMS uses direct vertices; discover that distinct format too.
        memset(gs_model_batch,0,3*sizeof gs_model_batch[0]);
        gs_model_batch[0][3]=gs_model_batch[1][3]=gs_model_batch[2][3]=1;
        gs_model_batch_count=3;gs_model_bound=NULL;
        gs_model_draw(model + 1, view, identity, alpha ? 0xffffff80u : 0xffffffffu, alpha);
        gs_model_batch_count=0;
    }
    gs_model_bound = NULL;
}
static void gs_warm_clear(void)
{
    int i, j;
    for (i=0;i<GS_WARM_JOBS;++i) {
        for(j=0;j<gs_warm_jobs[i].count;++j)
            if(gs_warm_jobs[i].captures[j]) aurora_pipeline_warm_release(gs_warm_jobs[i].captures[j]);
        memset(&gs_warm_jobs[i],0,sizeof gs_warm_jobs[i]);
    }
}
static int gs_warm_remaining(GsWarmJob* job)
{
    int i, n = job->count - job->cursor;
    for(i=0;i<job->cursor;++i) if(job->captures[i]) {
        int pending=aurora_pipeline_warm_pending(job->captures[i]);
        if(pending < 0) { job->failed=1; return -1; }
        if(!pending) { aurora_pipeline_warm_release(job->captures[i]); job->captures[i]=0; }
        else n+=pending;
    }
    return job->failed ? -1 : n;
}
static GsWarmJob* gs_warm_arg(lua_State* L)
{
    lua_Integer token=luaL_checkinteger(L,1);int i;
    for(i=0;i<GS_WARM_JOBS;++i) if(gs_warm_jobs[i].token==token && token>0 &&
        gs_warm_jobs[i].epoch==gs.scene_epoch && gs_warm_jobs[i].owner==gw_Script_StageResourceOwner()) return &gs_warm_jobs[i];
    luaL_error(L,"stale or foreign warm handle");return NULL;
}
static int l_warm_done(lua_State* L)
{
    GsWarmJob* job=gs_warm_arg(L);int n=gs_warm_remaining(job);
    lua_pushboolean(L,n==0); if(n<0)lua_pushliteral(L,"warm material discovery failed");else lua_pushnil(L);return 2;
}
static int l_warm_status(lua_State* L)
{
    GsWarmJob* job=gs_warm_arg(L);int n=gs_warm_remaining(job);
    lua_newtable(L);gs_setint(L,"objects",job->count);gs_setint(L,"prepared",job->cursor);
    gs_setint(L,"pending",n);gs_setbool(L,"done",n==0);gs_setbool(L,"failed",n<0);return 1;
}
static int l_warm(lua_State* L)
{
    static const char* keys[]={"enemies","items","models","fighters","tracers",NULL};
    GsWarmJob request;int k,i,slot=-1;
    gs_require_stage(L,"warm");luaL_checktype(L,1,LUA_TTABLE);gs_item_keys(L,1,keys);
    memset(&request,0,sizeof request);
    for(k=0;k<4;++k) {
        lua_pushstring(L,keys[k]);lua_rawget(L,1);
        if(!lua_isnil(L,-1)) {
            size_t length;luaL_checktype(L,-1,LUA_TTABLE);length=lua_rawlen(L,-1);
            if(length>GS_WARM_OBJECTS-request.count)return luaL_error(L,"warm limit is 128 objects");
            for(i=1;i<=(int)length;++i) {
                int value=-1;lua_rawgeti(L,-1,i);
                if(k==0) {
                    const char* name=luaL_checkstring(L,-1);int j;
                    for(j=0;j<GS_ENEMY_KINDS;++j)if(!strcmp(name,gs_enemy_names[j]))value=j;
                    if(value<0)return luaL_error(L,"unknown warm enemy");
                } else if(k==1) {
                    if(lua_type(L,-1)==LUA_TSTRING) {
                        const char* name=lua_tostring(L,-1);int def=gw_Geno_ItemFind(name);
                        if(def>=0)value=GENO_ITEM_KIND_BASE+def;
                        else if(!strncmp(name,"vanilla:",8)||!strncmp(name,"mex:",4)) {
                            char* end;long kind=strtol(name+(name[0]=='v'?8:4),&end,10);
                            if(!*end && kind>=0 && kind<5000)value=(int)kind;
                        }
                    } else { lua_Integer v=luaL_checkinteger(L,-1);if(v>=0&&v<5000)value=(int)v; }
                    if(value<0)return luaL_error(L,"unknown warm item");
                    if(value>=GENO_ITEM_KIND_BASE && value<GENO_ITEM_KIND_END) {
                        if(!gw_Geno_ItemActive(value))return luaL_error(L,"inactive warm Geno item");
                    } else if(value>=237 && !gw_Mex_ItemReady(value))
                        return luaL_error(L,"unregistered warm m-ex item");
                } else if(k==2) value=gs_model_asset(L,-1);
                else { lua_Integer port=luaL_checkinteger(L,-1);if(port<1||port>6)return luaL_error(L,"warm fighter requires loaded port 1..6");value=(int)port-1; }
                request.objects[request.count++]=(GsWarmObject){k,value};lua_pop(L,1);
            }
        }lua_pop(L,1);
    }
    lua_getfield(L,1,"tracers");
    if(!lua_isnil(L,-1)){luaL_checktype(L,-1,LUA_TBOOLEAN);if(lua_toboolean(L,-1)){
        if(request.count==GS_WARM_OBJECTS)return luaL_error(L,"warm object capacity exhausted");
        request.objects[request.count++]=(GsWarmObject){4,0};
    }}lua_pop(L,1);
    for(i=0;i<GS_WARM_JOBS;++i) if(!gs_warm_jobs[i].token) {slot=i;break;}
    if(slot<0)return luaL_error(L,"warm job capacity exhausted; release completed handles");
    if(gs_warm_serial==INT_MAX)return luaL_error(L,"warm handle space exhausted");
    request.token=++gs_warm_serial;request.owner=gw_Script_StageResourceOwner();request.epoch=gs.scene_epoch;
    gs_warm_jobs[slot]=request;lua_pushinteger(L,request.token);return 1;
}
static int l_warm_release(lua_State* L)
{
    GsWarmJob* job=gs_warm_arg(L);int i;
    for(i=0;i<job->count;++i)if(job->captures[i])aurora_pipeline_warm_release(job->captures[i]);
    memset(job,0,sizeof *job);return 0;
}
static void gs_warm_retire(int owner)
{
    int i,j;
    for(i=0;i<GS_WARM_JOBS;++i)if(gs_warm_jobs[i].token && gs_warm_jobs[i].owner==owner) {
        for(j=0;j<gs_warm_jobs[i].count;++j)if(gs_warm_jobs[i].captures[j])
            aurora_pipeline_warm_release(gs_warm_jobs[i].captures[j]);
        memset(&gs_warm_jobs[i],0,sizeof gs_warm_jobs[i]);
    }
}
static void gs_warm_tick(void)
{
    int i;
    if(gw_RB_Enabled()||gw_Netplay_Enabled()||gw_Snap_Resimulating())return;
    for(i=0;i<GS_WARM_JOBS;++i) {
        GsWarmJob* job=&gs_warm_jobs[i];GsWarmObject* object;unsigned capture;int result=1;
        if(!job->token || job->failed || job->epoch!=gs.scene_epoch)continue;
        if(gs_warm_remaining(job)<0)continue;
        if(job->cursor==job->count)continue;
        capture=aurora_pipeline_warm_begin();if(!capture)return;
        object=&job->objects[job->cursor];
        { char origin[96];
          if(object->kind==0)snprintf(origin,sizeof origin,"enemy:%s",gs_enemy_names[object->value]);
          else if(object->kind==2)snprintf(origin,sizeof origin,"model:%016llx",(unsigned long long)gs_stage_models[object->value].content_stamp[0]);
          else snprintf(origin,sizeof origin,"%s:%d",object->kind==1?"item":"fighter-port",object->value);
          aurora_pipeline_warm_label(origin);
        }
        gs_rw_branch();
        if(object->kind==3)gw_motion_warm(object->value+1,1);
        if(object->kind==4)gw_motion_prepare_ribbons();
        else if(object->kind==2)result=gw_ScriptGame_WarmModel(object->value);
        else if(object->kind==1 && object->value>=GENO_ITEM_KIND_BASE && object->value<GENO_ITEM_KIND_END) {
            int def=object->value-GENO_ITEM_KIND_BASE,c,p,colours=gw_Geno_ItemColourVariants(def)?5:1;
            for(c=0;c<colours;++c)if(!gs_item_visual_ready(def,c))result=0;
            if(result)for(c=0;c<colours;++c)for(p=0;p<2;++p)
                if(gs_item_models[def][c][p]>0)result=gw_ScriptGame_WarmModel(gs_item_models[def][c][p]-1);
        }
        else result=gw_ScriptGame_WarmDraw(object->kind==3?2:object->kind,object->value);
        if(object->kind==3)gw_motion_warm(object->value+1,0);
        aurora_pipeline_warm_end();
        if(result<=0) {aurora_pipeline_warm_release(capture);if(result<0)job->failed=1;return;}
        job->captures[job->cursor++]=capture;
        return; /* One descriptor traversal across all jobs per tick. */
    }
}
static int l_item_kinds(lua_State* L)
{
    int kind,n=0;gs_require_stage(L,"item_kinds");lua_newtable(L);
    for(kind=0;kind<5000;++kind) {
        int def=kind>=GENO_ITEM_KIND_BASE && kind<GENO_ITEM_KIND_END?kind-GENO_ITEM_KIND_BASE:-1;
        int ready=def>=0?gw_Geno_ItemActive(kind):kind<237?gw_ScriptGame_WarmItemReady(kind):gw_Mex_ItemReady(kind);
        char name[32];if(!ready)continue;
        lua_newtable(L);gs_setint(L,"kind",kind);
        snprintf(name,sizeof name,"%s:%d",kind<237?"vanilla":"mex",kind);
        gs_setstr(L,"name",def>=0?gw_Geno_ItemName(def):name);
        gs_setbool(L,"spawnable",def>=0 || kind<=34 || kind>=237);
        lua_rawseti(L,-2,++n);
    }
    return 1;
}
