/* Retail 1P lifecycle. Offline only, hooks at frame/scene boundaries; the
 * transition barrier keeps the old retail scene frozen before next preload. */
extern int gw_ScriptGame_OnePI(int field,int slot);
extern void gw_ScriptGame_OnePTemplatesClear(void);
extern int gw_ScriptGame_OnePTemplate(int,int,int,int,int,int,int,int,int,int,int,int);
extern void gw_ScriptGame_OnePEndBarrier(void);
extern void gw_ScriptGame_OnePTemplatesRelease(int owner);
extern void gw_ScriptGame_OnePTintClear(int slot,int owner);
extern int gw_SceneLaunch_OnePDirect(int mode);
static struct {
    int mode, target_mode, stage, flags, player, owner, scripted, loop, loops;
    int events, barrier, dispatch, held, hold_owner, remaining, used_hold, hold_spent;
    double hold_ms;
    int ended, completed, aborting, boss_port, boss_kind, spawn[6], generation[6], tint[6], tint_owner[6];
    char launch[256];
} gs_1p;
static int gs_1p_offline(void) {
    return !gw_RB_Enabled() && !gw_Netplay_Enabled() && !gw_Snap_Resimulating();
}
int gw_Script_OnePScripted(void) { return gs_1p.scripted && gs_1p_offline(); }
static const char* gs_1p_kind(void) {
    if(gs_1p.flags & 0x80) return "bonus";
    if(gs_1p.flags & 0x20) return "boss";
    if(gs_1p.flags & 2) return "giant";
    if(gs_1p.flags & 4) return "metal";
    if(gs_1p.flags & (8|0x10)) return "team";
    return "battle";
}
/* Keep initial and replacement payload filtering on the same live slot data. */
static int gs_1p_enemy(int slot) {
    return gs_players_present(slot) && slot!=gs_1p.player &&
        gw_ScriptGame_FighterI(slot,SI_SLOT_TYPE)==1 &&
        gw_ScriptGame_OnePI(1,slot)!=gw_ScriptGame_OnePI(1,gs_1p.player);
}
static void gs_1p_push(lua_State* L) {
    int s,n=0;
    lua_createtable(L,0,9);
    gs_setstr(L,"mode",(gs_1p.mode?gs_1p.mode:gs_1p.target_mode)==3?"classic":"adventure");
    gs_setint(L,"stage_index",gs_1p.stage);
    gs_setstr(L,"stage_kind",gs_1p_kind());
    gs_setint(L,"loop",gs_1p.loops);
    gs_setint(L,"player_port",gs_1p.player+1);
    gs_setbool(L,"held",gs_1p.held);
    gs_setbool(L,"final",gs_1p.completed);
    lua_newtable(L);
    for(s=0;s<6;++s) if(gs_1p_enemy(s)) {
        lua_createtable(L,0,4);gs_setint(L,"port",s+1);
        gs_setint(L,"char",gw_ScriptGame_FighterI(s,SI_CHAR));
        gs_setint(L,"kind",gw_ScriptGame_FighterI(s,SI_KIND));
        gs_setint(L,"entity",gs_1p.generation[s]);
        lua_rawseti(L,-2,++n);
    }
    lua_setfield(L,-2,"opponents");
}
static void gs_1p_hook(const char* name,int port,int entity) {
    int i;
    if(!gs.L || !gs_1p_offline())return;
    GsEvent env;int made=0;
    for(i=0;i<gs.n;++i)if(gs_may_run(i)&&gs.s[i].gameplay) {
        if(!made&&(gs_get_hook(i,"on_event"))){lua_pop(gs.L,1);gs_spine_signal_begin(&env,6,-1,port,entity);made=1;}
        if(made)gs_spine_signal_deliver(i,&env,name);
        if(!gs_get_hook(i,name))continue;
        gs_1p_push(gs.L);
        if(port>=0){gs_setint(gs.L,"port",port+1);gs_setint(gs.L,"entity",entity);}
        if(!strcmp(name,"on_1p_boss_defeated"))
            gs_setstr(gs.L,"boss_kind",entity==0x1b?"master_hand":entity==0x1c?"crazy_hand":entity==5?"bowser":entity==0x1f?"giga_bowser":"boss");
        gs_pcall(i,1,0,name);
    }
}
void gw_Script_OnePStage(int mode,int stage,int flags,int player) {
    if(!gs_1p_offline()||(mode!=3&&mode!=4))return;
    if(gs_1p.aborting)memset(&gs_1p,0,sizeof gs_1p); /* a new mode entry confirms teardown */
    gs_1p.mode=mode;gs_1p.stage=stage;gs_1p.flags=flags;gs_1p.player=player;
    gs_1p.events|=1;gs_1p.ended=gs_1p.completed=0;
    memset(gs_1p.spawn,0,sizeof gs_1p.spawn);
    memset(gs_1p.tint_owner,0,sizeof gs_1p.tint_owner);
    gw_ScriptGame_OnePTemplatesClear();
}
void gw_Script_OnePSpawn(int port,int generation) {
    if(!gs_1p.mode||!gs_1p_offline()||port<0||port>=6)return;
    gs_1p.spawn[port]=1;gs_1p.generation[port]=generation;
}
static int gs_1p_has_hook(const char* name) {
    int i;
    if(!gs.L)return 0;
    for(i=0;i<gs.n;++i)if(gs_may_run(i)&&gs.s[i].gameplay&&gs_get_hook(i,name)) {
        lua_pop(gs.L,1);return 1;
    }
    return 0;
}
void gw_Script_OnePClear(void) {
    if(!gs_1p.mode||!gs_1p_offline())return;
    if(gs_1p.mode==4&&gw_ScriptGame_OnePI(0,0)==11) {
        int s;
        for(s=1;s<6;++s)if(gs_players_present(s)) {
            int kind=gw_ScriptGame_FighterI(s,SI_KIND);
            if(kind==5||kind==0x1f) {
                gs_1p.boss_port=s;gs_1p.boss_kind=kind;gs_1p.events|=16;
                break;
            }
        }
    }
    gs_1p.events|=2;gs_1p.barrier=gs_1p.scripted||gs_1p_has_hook("on_1p_stage_clear")||
        ((gs_1p.events&16)&&gs_1p_has_hook("on_1p_boss_defeated"));gs_1p.used_hold=0;
}
void gw_Script_OnePGameOver(void) {
    if(!gs_1p.mode||!gs_1p_offline())return;
    if(!gs_1p.barrier)gs_1p.used_hold=0;
    gs_1p.events|=4;gs_1p.barrier=gs_1p.scripted||gs_1p_has_hook("on_1p_game_over");gs_1p.ended=1;gs_1p.loop=0;
}
int gw_Script_OnePComplete(void) {
    if(!gs_1p.mode||!gs_1p_offline())return 0;
    if(!gs_1p.barrier)gs_1p.used_hold=0;
    gs_1p.events|=8;gs_1p.barrier|=gs_1p.scripted||gs_1p_has_hook("on_1p_complete");gs_1p.completed=1;
    return gs_1p.scripted;
}
int gw_Script_OnePBarrierPending(void) { return gs_1p.barrier&&gs_1p_offline(); }
static int gs_1p_owner_live(int owner) {
    int i=owner-1;
    return i>=0&&i<gs.n&&gs.s[i].used&&!gs.s[i].disabled&&gs.s[i].gameplay;
}
static void gs_1p_tints(void) {
    int s;
    for(s=0;s<6;++s) if(gs_1p.spawn[s]&&gs_1p.tint_owner[s]&&gs_players_present(s)) {
        int d,n=gw_ScriptGame_PartsI(s,0,PART_I_REFRESH);
        for(d=0;d<n;++d)gw_ScriptGame_PartsControl(s,d,PART_CTL_TINT,
            gs_1p.tint[s],gs_1p.tint_owner[s]);
    }
}
static void gs_1p_start_dispatch(void) {
    int s;
    if(gs_1p.events&1) {
        int any=0;for(s=0;s<6;++s)any|=gs_players_present(s);
        if(!any)return;
        gs_1p.events&=~1;
        if(!gs.match_active) {
            gs.match_active=1;gs.match_frame=0;
            gs_hook_all("on_match_start",0,0,0);
        } /* APIs may write before the first fighter logic frame. */
        gs_1p_hook("on_1p_stage_start",-1,0);
    }
    gs_1p_tints();
    for(s=0;s<6;++s)if(gs_1p.spawn[s]&&gs_players_present(s)) {
        gs_1p.spawn[s]=0;gs_1p_hook("on_1p_spawn",s,gs_1p.generation[s]);
    }
}
/* The interstage hold is counted in 60 Hz logic units, not host ticks: a host tick is a render tick (60 or 120 a
   second), so the old tick count made the hold half as long at 120 Hz. One unit is 1000/60 ms of wall time
   (gs_1p_hold_clock); a hitch is clamped so a stalled frame cannot spend the hold. The default is 1800 units
   (30 s); a script may ask for up to GS_1P_HOLD_MAX (120 s) in all, counted from the claim, and may renew a live
   claim with another gd.hold_1p call until that ceiling is spent. */
#define GS_1P_HOLD_DEFAULT 1800
#define GS_1P_HOLD_MAX 7200
static int gs_1p_hold_accept(int frames,int owner) {
    if(frames<1)return 0;
    if(gs_1p.held&&gs_1p.hold_owner==owner) { /* renew: the new length from now, never past the ceiling */
        int cap=GS_1P_HOLD_MAX-gs_1p.hold_spent;
        if(cap<1)return 0;
        gs_1p.remaining=frames>cap?cap:frames;return 1;
    }
    if(!gs_1p.dispatch||!gs_1p.barrier||gs_1p.used_hold)return 0;
    gs_1p.held=gs_1p.used_hold=1;gs_1p.hold_owner=owner;gs_1p.hold_spent=0;gs_1p.hold_ms=gs_now_ms();
    gs_1p.remaining=frames>GS_1P_HOLD_MAX?GS_1P_HOLD_MAX:frames;return 1;
}
/* Whole 60 Hz units of wall time since the last call (the fraction carries over), at most 6 per call. */
static int gs_1p_hold_clock(void) {
    double now=gs_now_ms(),d=now-gs_1p.hold_ms;int u;
    if(d<0)d=0;
    if(d>100.0)d=100.0;
    u=(int)(d*0.06);
    gs_1p.hold_ms+=u*(1000.0/60.0);
    if(now-gs_1p.hold_ms>100.0)gs_1p.hold_ms=now;
    return u;
}
static int gs_1p_hold_step_units(int owner_live,int units) {
    if(!gs_1p.held)return 0;
    if(owner_live) {
        gs_1p.hold_spent+=units;gs_1p.remaining-=units;
        if(gs_1p.remaining>0&&gs_1p.hold_spent<GS_1P_HOLD_MAX)return 0;
    }
    gs_1p.held=0;return 1;
}
static int gs_1p_hold_step(int owner_live) {
    return gs_1p_hold_step_units(owner_live,gs_1p.held?gs_1p_hold_clock():0);
}
static void gs_1p_end(void);
static void gs_1p_tick(void) {
    int ev;
    if(!gs_1p_offline()) {
        int was_barrier=gs_1p.barrier;
        gs_1p.held=gs_1p.barrier=gs_1p.loop=0;
        if(was_barrier)gw_ScriptGame_OnePEndBarrier();
        return;
    }
    if(gs_1p.scripted&&!gs_1p.aborting&&!gs_1p_owner_live(gs_1p.owner))gs_1p_end();
    if(!gs_1p.barrier)return;
    ev=gs_1p.events&30;gs_1p.events&=~30;
    gs_1p.dispatch=1;
    if(ev&16)gs_1p_hook("on_1p_boss_defeated",gs_1p.boss_port,gs_1p.boss_kind);
    if(ev&2)gs_1p_hook("on_1p_stage_clear",-1,0);
    if(ev&8)gs_1p_hook("on_1p_complete",-1,0);
    if(ev&4)gs_1p_hook("on_1p_game_over",-1,0);
    gs_1p.dispatch=0;
    if(gs_1p_hold_step(gs_1p_owner_live(gs_1p.hold_owner))) {gw_log("1p: interstage hold released (timeout/owner)");
        if(gs_1p.completed)gs_1p.loop=0; /* no unsaved autonomous NG+ */
    }
    if(!gs_1p.held) {
        gs_1p.barrier=0;
        if(gs_1p.completed&&gs_1p.loop&&gs_1p_owner_live(gs_1p.owner)) {
            ++gs_1p.loops;gw_SceneLaunch_SetText(gs_1p.launch);
            gs_pending_launch=gw_SceneLaunch_BootGameMode();
        }else if(gs_1p.completed||gs_1p.ended) {
            gs_1p.mode=0;gs_1p.scripted=0;
            gw_SceneLaunch_SetText(NULL);
            gw_ScriptGame_OnePTemplatesClear();
            gw_ScriptGame_FighterModsRelease(0);
        }
        gw_ScriptGame_OnePEndBarrier();
    }
}
static int l_mode_1p(lua_State* L) {
    if(!gs_1p.mode||!gs_1p_offline())lua_pushnil(L);else gs_1p_push(L);
    return 1;
}
static int l_hold_1p(lua_State* L) {
    int n=(int)luaL_optinteger(L,1,GS_1P_HOLD_DEFAULT);
    gs_require_offline(L,"hold_1p");
    lua_pushboolean(L,gs_1p_hold_accept(n,gs.cur+1));return 1;
}
static int gs_1p_hold_release(int owner) {
    if(!gs_1p.held||gs_1p.hold_owner!=owner)return 0;
    gs_1p.held=0;return 1;
}
static int l_release_1p(lua_State* L) {
    gs_require_offline(L,"release_1p");
    lua_pushboolean(L,gs_1p_hold_release(gs.cur+1));return 1;
}
static int l_loop_1p(lua_State* L) {
    gs_require_offline(L,"loop_1p");
    if(!gs_1p.scripted||gs_1p.owner!=gs.cur+1){lua_pushboolean(L,0);return 1;}
    if(lua_toboolean(L,1)&&!gs_1p.launch[0]){lua_pushboolean(L,0);return 1;}
    gs_1p.loop=lua_toboolean(L,1);lua_pushboolean(L,1);return 1;
}
static int gs_1p_can_start(int game_mode) {
    return !gs_1p.scripted&&!gs_1p.mode&&game_mode!=3&&game_mode!=4;
}
static int l_start_1p(lua_State* L) {
    const char *mode,*fighter;int difficulty,stocks,loop,game;lua_Integer di,st;
    GsScript* s=gs_cur_script();char text[256];
    if(!s||!s->gameplay||gs.cur==gs.console||!gs_1p_offline()) {
        lua_pushboolean(L,0);lua_pushliteral(L,"offline gameplay mod required");return 2;
    }
    if(!gs_1p_can_start(gw_ScriptGame_GameMode())) {
        lua_pushboolean(L,0);lua_pushliteral(L,"1P run already active; end it before starting another");return 2;
    }
    if(!lua_istable(L,1)){lua_pushboolean(L,0);lua_pushliteral(L,"options required");return 2;}
    lua_getfield(L,1,"mode");mode=lua_type(L,-1)==LUA_TSTRING?lua_tostring(L,-1):NULL;
    lua_getfield(L,1,"fighter");fighter=lua_type(L,-1)==LUA_TSTRING?lua_tostring(L,-1):NULL;
    lua_getfield(L,1,"difficulty");di=lua_isnil(L,-1)?2:lua_isinteger(L,-1)?lua_tointeger(L,-1):-1;
    difficulty=di>=0&&di<=4?(int)di:-1;
    lua_getfield(L,1,"stocks");st=lua_isnil(L,-1)?3:lua_isinteger(L,-1)?lua_tointeger(L,-1):-1;
    stocks=st>=1&&st<=99?(int)st:-1;
    lua_getfield(L,1,"loop");loop=lua_toboolean(L,-1);
    if(!mode||(!strcmp(mode,"classic")?0:strcmp(mode,"adventure"))||!fighter||
       !fighter[0]||strlen(fighter)>80||strchr(fighter,';')||strchr(fighter,'/')||strchr(fighter,'=')||difficulty<0||difficulty>4||stocks<1||stocks>99) {
        lua_pushboolean(L,0);lua_pushliteral(L,"invalid 1P options");return 2;
    }
    snprintf(text,sizeof text,"mode=%s;p1=%s/stocks%d;difficulty=%d;step=0",mode,fighter,stocks,difficulty);
    gw_SceneLaunch_SetText(text);game=gw_SceneLaunch_BootGameMode();
    if((game!=3&&game!=4)||!gw_SceneLaunch_OnePDirect(game)){lua_pushboolean(L,0);lua_pushliteral(L,"scene refused");return 2;}
    memset(&gs_1p,0,sizeof gs_1p);gs_1p.scripted=1;gs_1p.target_mode=game;gs_1p.owner=gs.cur+1;
    gs_1p.loop=loop;snprintf(gs_1p.launch,sizeof gs_1p.launch,"%s",text);
    gs_pending_launch=game;lua_pushboolean(L,1);return 1;
}
static int l_spawn_1p(lua_State* L) {
    static const char* keys[]={"damage_dealt","damage_taken","run_speed","air_speed","shield_max","jump_height","air_jump_height","knockback_taken"};
    int slot=gs_slot_arg(L,1),i,enabled=!lua_isnoneornil(L,2),tint=-1;
    float value[8]={1,1,1,1,1,1,1,1};
    gs_require_offline(L,"spawn_1p");
    if(enabled&&(!gs_1p.mode||!gs.match_active||
        (gs_1p.scripted&&gs_1p.owner!=gs.cur+1))){lua_pushboolean(L,0);return 1;}
    if ((gw_ScriptGame_FighterModOwner(slot) && gw_ScriptGame_FighterModOwner(slot)!=gs.cur+1)||
        (gs_1p.tint_owner[slot] && gs_1p.tint_owner[slot]!=gs.cur+1)) {lua_pushboolean(L,0);return 1;}
    if(enabled) {
        luaL_checktype(L,2,LUA_TTABLE);lua_pushnil(L);
        while(lua_next(L,2)) {
            const char* key=lua_type(L,-2)==LUA_TSTRING?lua_tostring(L,-2):NULL;
            if(key&&!strcmp(key,"tint"))tint=(int)(gs_color_arg(L,-1,0xffffffffu)>>8);
            else {
                double v=luaL_checknumber(L,-1);
                for(i=0;i<8;++i)if(key&&!strcmp(key,keys[i]))break;
                if(i==8)return luaL_error(L,"unknown 1P spawn modifier");
                if(!isfinite(v))return luaL_error(L,"1P modifier must be finite");
                value[i]=(float)(v<.1?.1:v>4?4:v);
            }
            lua_pop(L,1);
        }
    }
    if(!gw_ScriptGame_OnePTemplate(slot,gs.cur+1,enabled,gs_fbits(value[0]),gs_fbits(value[1]),
       gs_fbits(value[2]),gs_fbits(value[3]),gs_fbits(value[4]),tint,gs_fbits(value[5]),gs_fbits(value[6]),gs_fbits(value[7]))){lua_pushboolean(L,0);return 1;}
    if(enabled&&!gs_1p.scripted) {
        /* Modifying a manually entered retail run must not produce vanilla records. */
        gs_1p.scripted=1;gs_1p.owner=gs.cur+1;gs_1p.target_mode=gs_1p.mode;
        gs_1p.loop=0;gs_1p.launch[0]='\0';
    }
    gs_rw_branch();
    if(!gw_ScriptGame_FighterMod(slot,gs.cur+1,enabled,gs_fbits(value[0]),gs_fbits(value[1]),
       gs_fbits(value[2]),gs_fbits(value[3]),gs_fbits(value[4]),gs_fbits(value[5]),gs_fbits(value[6]),gs_fbits(value[7]))){lua_pushboolean(L,0);return 1;}
    gw_ScriptGame_OnePTintClear(slot,gs.cur+1);
    gs_1p.tint_owner[slot]=tint>=0&&enabled?gs.cur+1:0;gs_1p.tint[slot]=tint;
    if(gs_1p.tint_owner[slot]) {
        int pending=gs_1p.spawn[slot];
        gs_1p.spawn[slot]=1;gs_1p_tints();gs_1p.spawn[slot]=pending;
    }
    lua_pushboolean(L,1);return 1;
}

static void gs_1p_external_launch(void) {
    if(gs_1p.scripted) {
        gs_1p.loop=0;gs_1p_hook("on_1p_game_over",-1,0);
        /* A frame-post launch can precede the old retail state's exit.
         * Retain its record protection until actual reset/teardown. */
        gs_1p_end();
        if(!gs_1p.mode)memset(&gs_1p,0,sizeof gs_1p);
    }
}

int gw_Script_OnePOffline(void) { return gs_1p_offline(); }

static int gs_1p_state_active(void) { return gs_1p.mode || gs_1p.scripted; }

static void gs_1p_end(void) {
    int s;
    if(gs_1p.aborting||(!gs_1p.scripted&&!gs_1p.mode))return;
    gs_1p.loop=gs_1p.held=0;
    gw_ScriptGame_OnePTemplatesClear();
    gw_ScriptGame_FighterModsRelease(gs_1p.owner);
    for(s=0;s<6;++s) {
        if(gs_1p.tint_owner[s])gw_ScriptGame_OnePTintClear(s,gs_1p.tint_owner[s]);
        gs_1p.tint_owner[s]=0;
    }
    if(!gs_1p.mode && (gs_pending_launch==3||gs_pending_launch==4)) {
        gs_pending_launch=-1;gw_SceneLaunch_SetText(NULL);memset(&gs_1p,0,sizeof gs_1p);return;
    }
    gs_1p.aborting=1;
    gw_SceneLaunch_SetText("mode=menu");gs_pending_launch=1;
    if(gs_1p.barrier) {gs_1p.barrier=0;gw_ScriptGame_OnePEndBarrier();}
}
static int l_end_1p(lua_State* L) {
    GsScript* s=gs_cur_script();
    if(!s||!s->gameplay||!gs_1p_offline()||gs_1p.owner!=gs.cur+1) {
        lua_pushboolean(L,0);return 1;
    }
    gs_1p_end();lua_pushboolean(L,1);return 1;
}
static void gs_1p_scene_check(void) {
    int mode=gw_ScriptGame_GameMode();
    if(gs_1p.aborting&&mode!=3&&mode!=4)memset(&gs_1p,0,sizeof gs_1p);
    else if(gs_1p.mode&&mode!=gs_1p.mode&&(!gs_1p.scripted||!gs_1p.completed)) {
        gs_1p_hook("on_1p_game_over",-1,0);
        memset(&gs_1p,0,sizeof gs_1p);
    }
}
