/* A launch request is consumed once in the first gameplay scene. The host
 * loader remains up while the owning Lua director stages and warms content. */
extern const char* gw_SceneLaunch_Mission(void);
extern const char* gw_SceneLaunch_Mod(void);
extern int gw_SceneLaunch_MazeSeed(void);
extern int gw_SceneLaunch_MazeSize(void);
extern unsigned gw_SceneLaunch_Revision(void);
static int gs_launch_consumed, gs_launch_pending, gs_launch_owner, gs_launch_delivered, gs_launch_warned;
static unsigned gs_launch_revision;
static int gs_launch_configured(void)
{
    return *gw_SceneLaunch_Mission() || gw_SceneLaunch_MazeSize()>0;
}
int gw_Script_LaunchPending(void)
{
    if(gs_launch_revision!=gw_SceneLaunch_Revision()) {
        gs_launch_revision=gw_SceneLaunch_Revision();gs_launch_consumed=0;gs_launch_pending=0;
    }
    return gs_launch_pending || (!gs_launch_consumed && gs_launch_configured());
}
static void gs_launch_table(lua_State* L)
{
    lua_newtable(L);gs_setstr(L,"mission",gw_SceneLaunch_Mission());
    gs_setint(L,"seed",gw_SceneLaunch_MazeSeed());gs_setint(L,"size",gw_SceneLaunch_MazeSize());
    gs_setstr(L,"mod",gw_SceneLaunch_Mod());gs_setbool(L,"pending",gs_launch_pending);
}
static int l_launch_request(lua_State* L) { gs_launch_table(L);return 1; }
static int l_launch_ready(lua_State* L)
{
    int i;
    gs_require_stage(L,"launch_ready");
    if(!gs_launch_pending || gs_launch_owner!=gw_Script_StageResourceOwner())
        return luaL_error(L,"launch_ready requires the launch owner");
    for(i=0;i<GS_WARM_JOBS;++i) if(gs_warm_jobs[i].token &&
        gs_warm_jobs[i].owner==gs_launch_owner && gs_warm_remaining(&gs_warm_jobs[i])!=0) {
        lua_pushboolean(L,0);return 1;
    }
    gs_launch_pending=0;gw_log("launch: staging and warm declarations ready");lua_pushboolean(L,1);return 1;
}
static int l_launch_cancel(lua_State* L)
{
    gs_require_offline(L,"launch_cancel");
    if(gs_launch_owner && gs_launch_owner!=gw_Script_StageResourceOwner())
        return luaL_error(L,"launch_cancel requires the launch owner");
    gs_launch_pending=0;gs_launch_consumed=1;
    gw_log("launch: cancelled: %s",luaL_optstring(L,1,"script requested cancellation"));return 0;
}
static void gs_launch_begin(void)
{
    if(!gs_launch_consumed && gs_launch_configured() && gw_ScriptGame_StageGameplayScene()) {
        gs_launch_consumed=1;gs_launch_pending=1;gs_launch_owner=0;gs_launch_delivered=0;gs_launch_warned=0;
    } else if(gs_launch_pending) {
        gs_launch_pending=0;gw_log("launch: staging cancelled by scene transition");
    }
}
static void gs_launch_tick(void)
{
    int i, candidates=0, candidate=-1;const char* mod=gw_SceneLaunch_Mod();
    if(!gs_launch_pending || gs_launch_delivered)return;
    // The held scene never calls FramePost. Detect the initialized host here,
    // without advancing simulation, before granting the usual match API surface.
    if(!gs.match_active) {
        int port, present=0;
        if(!gw_ScriptGame_StageGameplayScene())return;
        for(port=0;port<6;++port) {
            int exists=gs_players_present(port);present |= exists;
            gs.last_action[port]=exists?gw_ScriptGame_FighterI(port,SI_ACTION):-1;gs.state_frame[port]=0;
        }
        if(!present)return;
        gs.match_active=1;gs.match_frame=0;
        gs_hook_all("on_match_start",0,0,0);
    }
    for(i=0;i<gs.n;++i) {
        int previous;char root[MAX_PATH];
        if(!gs.s[i].gameplay || !gs.s[i].used || gs.s[i].disabled)continue;
        previous=gs.cur;gs.cur=i;
        if(*mod && (!gs_mission_script_root(root) || strcmp(gw_path_separator(root)?gw_path_separator(root)+1:root,mod))) {
            gs.cur=previous;continue;
        }gs.cur=previous;
        if(gs_get_hook(i,"on_launch")) {lua_pop(gs.L,1);candidate=i;++candidates;}
    }
    if(candidates!=1) {
        if(!gs_launch_warned) {gs_launch_warned=1;gw_log("launch: staging held: expected one on_launch gameplay owner, found %d; add hook or gd.launch_cancel(reason)",candidates);}
        return;
    }
    gs_launch_owner=gs.s[candidate].stage_owner;gs_launch_delivered=1;
    if(gs_get_hook(candidate,"on_launch")) {
        gs_launch_table(gs.L);
        if(gs_pcall(candidate,1,0,"on_launch")<0)
            gw_log("launch: on_launch failed; loading remains covered for retry or gd.launch_cancel");
    }
}
static void gs_launch_retire(int owner)
{
    if(gs_launch_pending && gs_launch_owner==owner) {
        gs_launch_owner=0;gs_launch_delivered=0;gs_launch_warned=0;
        gw_log("launch: staging owner retired; waiting for replacement on_launch hook");
    }
}
