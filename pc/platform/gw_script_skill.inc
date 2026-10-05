/* Skill events (included by gw_script.c after gw_script_fighter_caps.inc).
 *
 * The producer is game-side (pc/gameworld/script_skill.inc): read-only observers of the game's own decisions that
 * append a row to a snapshotted ring and call Script_SkillEvent. This file queues that row as engine event 18 (the
 * armour event is 17; the dispatch table, the spine envelope and the producer all use the same numbers, and
 * crit is 19), delivers it after the frame like every other engine event, and exposes the ring.
 *
 * A script receives each skill event twice if it defines both hooks: on_<kind>(info) for the technique and
 * on_skill(info) for every technique, plus the generic on_event envelope. All three carry the same table.
 * Delivery is post-frame, in the frame that produced the row, so a script that reacts with a gd.fighter_*
 * write has it applied on the next logic frame (the same rule as every other event hook). Read-only. */
extern void gw_ScriptGame_SkillFrame(void);
extern int gw_ScriptGame_SkillRowCount(void);
extern int gw_ScriptGame_SkillRow(int idx,int field);
extern int gw_ScriptGame_SkillState(int entity,int field);
extern int gw_ScriptGame_SkillConst(int idx);
extern void gw_ScriptGame_SkillReset(void);
#define GS_EV_SKILL 18
#define GS_SKILL_KINDS 25
static const char* const gs_skill_names[GS_SKILL_KINDS]={"","lcancel","lcancel_miss","auto_cancel","lcancel_hit","wavedash","waveland","ledge_dash","air_dodge","perfect_shield","tech","tech_miss","dash_dance","short_hop","full_hop","fast_fall","shield_drop","jump_cancel_grab","jump_cancel_usmash","sdi","combo","combo_end","shock","shock_hit","shock_end"};
static const char* const gs_skill_hooks[GS_SKILL_KINDS]={"","on_lcancel","on_lcancel_miss","on_auto_cancel","on_lcancel_hit","on_wavedash","on_waveland","on_ledge_dash","on_air_dodge","on_perfect_shield_skill","on_tech","on_tech_miss","on_dash_dance","on_short_hop","on_full_hop","on_fast_fall","on_shield_drop","on_jump_cancel_grab","on_jump_cancel_usmash","on_sdi","on_combo","on_combo_end","on_shock","on_shock_hit","on_shock_end"};
static float gs_skill_f(int bits){union{int i;float f;}u;u.i=bits;return u.f;}

/* Observational queue only: skipped while resimulating, never read by gameplay. */
void gw_Script_SkillEvent(int kind,int entity,int i1,int i2,int i3,int f1,int f2)
{
    GsEvent* e;
    if(!gs.L||!gs.want_events||gw_Snap_Resimulating()||kind<1||kind>=GS_SKILL_KINDS||entity<1||entity>12) return;
    if(gs.nev>=GS_MAX_EVENTS){++gs.ev_dropped;return;}
    e=&gs.ev[gs.nev++];memset(e,0,sizeof *e);
    e->what=GS_EV_SKILL;e->a=entity;e->b=kind;e->c=i1;e->d=i2;e->item_amount=i3;e->item_y=f1;e->item_colour=f2;
    e->item_reason=gw_ScriptGame_SkillRow(0,0); /* the ring row appended just before this call */
    gs_spine_event_capture(e);
}
static void gs_skill_fill(lua_State* L,int kind,int entity,int frame,int i1,int i2,int i3,int f1b,int f2b)
{
    static const char* const aerials[]={"nair","fair","bair","uair","dair"};
    static const char* const tech_dir[]={"in_place","toward","away","wall","ceiling"};
    float f1=gs_skill_f(f1b),f2=gs_skill_f(f2b);
    lua_createtable(L,0,12);
    gs_setstr(L,"kind",gs_skill_names[kind]);gs_setint(L,"port",(entity-1)%6+1);gs_setint(L,"entity",entity);
    gs_setbool(L,"subfighter",entity>6);gs_setint(L,"frame",frame);
    switch(kind){
    case 1:case 2:case 3:case 4:
        gs_setstr(L,"aerial",i1>=0&&i1<5?aerials[i1]:"?");gs_setbool(L,"hit",i2!=0);
        if(kind!=3){gs_setint(L,"trigger_age",i3);gs_setnum(L,"lag",f1);gs_setnum(L,"lag_cancelled",f2);}
        break;
    case 5:case 6:
        gs_setint(L,"frames_after_jump",i1);gs_setint(L,"dodge_frames",i2);gs_setnum(L,"speed",f1);gs_setnum(L,"speed_abs",f2);break;
    case 7:
        gs_setint(L,"frames_since_ledge",i1);gs_setint(L,"dodge_frames",i2);gs_setnum(L,"speed",f1);gs_setnum(L,"speed_abs",f2);break;
    case 8:
        gs_setint(L,"frames_after_jump",i1);gs_setnum(L,"stick_x",f1);gs_setnum(L,"stick_y",f2);break;
    case 9:
        gs_setbool(L,"projectile",i1!=0);if(i2>0)gs_setint(L,"attacker",i2);
        gs_setint(L,"window_frame",i3);gs_setnum(L,"damage",f1);break;
    case 10:
        gs_setstr(L,"direction",i1>=0&&i1<5?tech_dir[i1]:"?");gs_setbool(L,"roll",i2!=0);gs_setbool(L,"wall_jump",i3!=0);break;
    case 11:
        gs_setbool(L,"facing_down",i1!=0);break;
    case 12:
        gs_setint(L,"reversals",i1);gs_setint(L,"frames_since_dash",i2);break;
    case 13:case 14:
        gs_setint(L,"jumpsquat",i1);gs_setnum(L,"speed",f1);gs_setnum(L,"expected",f2);break;
    case 15:
        gs_setint(L,"air_frames",i1);gs_setnum(L,"speed",f1);break;
    case 16:
        gs_setint(L,"shield_frames",i1);break;
    case 17:case 18:
        gs_setint(L,"jumpsquat",i1);break;
    case 19:
        gs_setint(L,"count",i1);gs_setint(L,"hitlag_left",i2);gs_setnum(L,"dx",f1);gs_setnum(L,"dy",f2);break;
    case 20:
        gs_setint(L,"count",i1);if(i2>0)gs_setint(L,"attacker",i2);gs_setnum(L,"damage",f1);gs_setnum(L,"last_damage",f2);break;
    case 21:
        gs_setint(L,"count",i1);if(i2>0)gs_setint(L,"attacker",i2);gs_setbool(L,"ko",i3!=0);gs_setnum(L,"damage",f1);break;
    case 22:
        gs_setint(L,"frames",i1);gs_setint(L,"charges",i2);gs_setint(L,"bonus",i3);gs_setnum(L,"hitstun_multiplier",f1);break;
    case 23:
        gs_setint(L,"base_hitstun",i1);gs_setint(L,"extra_hitstun",i2);gs_setint(L,"charges_left",i3);gs_setnum(L,"hitstun_multiplier",f1);break;
    case 24: {
        static const char* const why[]={"?","expired","spent","cleared","replaced"};
        gs_setstr(L,"reason",i1>=1&&i1<5?why[i1]:"?");gs_setint(L,"hits",i2);break; }
    }
}
static void gs_skill_push(lua_State* L,const GsEvent* e)
{
    gs_skill_fill(L,e->b,e->a,e->item_reason,e->c,e->d,e->item_amount,e->item_y,e->item_colour);
}
/* The kind-specific hook: called before the generic on_skill so a script can listen to one technique. */
static void gs_skill_dispatch_specific(int script,const GsEvent* e)
{
    const char* hook;
    if(e->b<1||e->b>=GS_SKILL_KINDS) return;
    hook=gs_skill_hooks[e->b];
    if(gs_get_hook(script,hook)){gs_skill_push(gs.L,e);gs_pcall(script,1,0,hook);}
}
/* gd.skill_history(entity [, n [, kind]]): the newest rows first, same tables as the events. */
static int l_skill_history(lua_State* L)
{
    int e=gs_caps_entity(L,1),n=lua_isnoneornil(L,2)?16:(int)luaL_checkinteger(L,2),want=0,i,out=0,total=gw_ScriptGame_SkillRowCount();
    if(n<0||n>256) return luaL_error(L,"skill history length must be 0-256");
    if(!lua_isnoneornil(L,3)){
        const char* k=luaL_checkstring(L,3);
        for(i=1;i<GS_SKILL_KINDS;++i) if(!strcmp(k,gs_skill_names[i])) want=i;
        if(!want) return luaL_error(L,"unknown skill kind");
    }
    lua_newtable(L);
    for(i=0;i<total&&out<n;++i){
        int kind=gw_ScriptGame_SkillRow(i,1);
        if(gw_ScriptGame_SkillRow(i,2)!=e||(want&&kind!=want)||kind<1||kind>=GS_SKILL_KINDS) continue;
        gs_skill_fill(L,kind,e,gw_ScriptGame_SkillRow(i,0),gw_ScriptGame_SkillRow(i,3),gw_ScriptGame_SkillRow(i,4),
                      gw_ScriptGame_SkillRow(i,5),gw_ScriptGame_SkillRow(i,6),gw_ScriptGame_SkillRow(i,7));
        lua_rawseti(L,-2,++out);
    }
    return 1;
}
static int l_skill_state(lua_State* L)
{
    int e=gs_caps_entity(L,1);
    lua_createtable(L,0,6);
    gs_setint(L,"combo_count",gw_ScriptGame_SkillState(e,0));gs_setnum(L,"combo_damage",gs_skill_f(gw_ScriptGame_SkillState(e,1)));
    gs_setint(L,"combo_attacker",gw_ScriptGame_SkillState(e,2));gs_setint(L,"air_frames",gw_ScriptGame_SkillState(e,3));
    gs_setint(L,"frames_in_state",gw_ScriptGame_SkillState(e,4));gs_setbool(L,"aerial_hit",gw_ScriptGame_SkillState(e,5)!=0);
    return 1;
}
static int l_skill_kinds(lua_State* L)
{
    int i;lua_newtable(L);
    for(i=1;i<GS_SKILL_KINDS;++i){lua_pushstring(L,gs_skill_names[i]);lua_rawseti(L,-2,i);}
    return 1;
}
static int l_skill_thresholds(lua_State* L)
{
    lua_createtable(L,0,6);
    gs_setnum(L,"wave_min_speed",gs_skill_f(gw_ScriptGame_SkillConst(0)));gs_setint(L,"wavedash_max_airborne_frames",gw_ScriptGame_SkillConst(1));
    gs_setint(L,"ledge_dash_frames",gw_ScriptGame_SkillConst(2));gs_setint(L,"dash_dance_frames",gw_ScriptGame_SkillConst(3));
    gs_setint(L,"lcancel_window_frames",gw_ScriptGame_SkillConst(4));gs_setint(L,"powershield_window_frames",gw_ScriptGame_SkillConst(5));
    return 1;
}

/* ---- bonus-stage objectives ---------------------------------------------------------------------------------- */
extern int gw_ScriptGame_ObjectiveKind(void);
extern int gw_ScriptGame_ObjectiveDoor(int idx,int field);
extern int gw_ScriptGame_ObjectiveZone(int idx,int field);
extern int gw_ScriptGame_ObjectiveRoute(int group,int idx,int field);
/* gd.stage_objectives(): {targets={ {id=,x=,y=,z=,state=}, ... }, remaining=n, doors=nil, finish=nil,
 * supported={targets=true, doors=false, finish=false}}. Read-only; works offline and online (it only reads items). */
static int l_stage_objectives(lua_State* L)
{
    int i,n=gw_ScriptGame_ItemCount(),kind=gw_ScriptGame_ObjectiveKind(),count=0;
    lua_createtable(L,0,5);
    lua_newtable(L);
    for(i=0;i<n;++i){
        if(gw_ScriptGame_ItemI(i,SCRIPT_ITEM_KIND)!=kind) continue;
        lua_createtable(L,0,6);
        gs_setstr(L,"kind","target");gs_setint(L,"id",gw_ScriptGame_ItemI(i,SCRIPT_ITEM_ID));
        gs_setnum(L,"x",gw_ScriptGame_ItemF(i,SCRIPT_ITEM_X));gs_setnum(L,"y",gw_ScriptGame_ItemF(i,SCRIPT_ITEM_Y));gs_setnum(L,"z",gw_ScriptGame_ItemF(i,SCRIPT_ITEM_Z));
        gs_setint(L,"state",gw_ScriptGame_ItemI(i,SCRIPT_ITEM_STATE));gs_setbool(L,"intact",1);
        lua_rawseti(L,-2,++count);
    }
    lua_setfield(L,-2,"targets");
    gs_setint(L,"remaining",count);
    {   /* Race to the Finish: map points, not items (see ScriptGame_ObjectiveDoor). doors/finish stay nil on every other stage. */
        int race=gw_ScriptGame_ObjectiveDoor(-1,0);
        if(race){
            int d,k=0,latched=gw_ScriptGame_ObjectiveDoor(-1,1);
            lua_newtable(L);
            for(d=0;d<26;++d){
                if(!gw_ScriptGame_ObjectiveDoor(d,0)) continue;
                lua_createtable(L,0,10);
                gs_setstr(L,"kind","door");gs_setint(L,"index",d);gs_setint(L,"id",0x99+d);
                gs_setnum(L,"x",gs_skill_f(gw_ScriptGame_ObjectiveDoor(d,1)));gs_setnum(L,"y",gs_skill_f(gw_ScriptGame_ObjectiveDoor(d,2)));gs_setnum(L,"z",gs_skill_f(gw_ScriptGame_ObjectiveDoor(d,3)));
                gs_setint(L,"score",gw_ScriptGame_ObjectiveDoor(d,4));gs_setint(L,"coins",gw_ScriptGame_ObjectiveDoor(d,5));
                gs_setbool(L,"reached",gw_ScriptGame_ObjectiveDoor(d,6)!=0);gs_setbool(L,"inside",gw_ScriptGame_ObjectiveDoor(d,7)!=0);
                lua_rawseti(L,-2,++k);
            }
            lua_setfield(L,-2,"doors");
            lua_createtable(L,0,6);
            gs_setbool(L,"reached",latched>=0);if(latched>=0)gs_setint(L,"door",latched);
            gs_setbool(L,"touching",gw_ScriptGame_ObjectiveDoor(-1,2)!=0);
            gs_setint(L,"candidate",gw_ScriptGame_ObjectiveDoor(-1,3));gs_setnum(L,"scale",gs_skill_f(gw_ScriptGame_ObjectiveDoor(-1,4)));
            lua_newtable(L);
            for(d=0;d<4;++d){
                lua_createtable(L,0,4);
                gs_setnum(L,"x_min",gs_skill_f(gw_ScriptGame_ObjectiveZone(d,0)));gs_setnum(L,"x_max",gs_skill_f(gw_ScriptGame_ObjectiveZone(d,1)));
                gs_setnum(L,"y_top",gs_skill_f(gw_ScriptGame_ObjectiveZone(d,2)));gs_setnum(L,"y_bottom",gs_skill_f(gw_ScriptGame_ObjectiveZone(d,3)));
                lua_rawseti(L,-2,d+1);
            }
            lua_setfield(L,-2,"zones");
            lua_setfield(L,-2,"finish");
        }
        {   /* Underground Maze / F-Zero Grand Prix: route markers (see ScriptGame_ObjectiveRoute for what each point means). */
            int rk=gw_ScriptGame_ObjectiveRoute(0,0,0);
            if(rk){
                int g,k,inx,fin=0;
                lua_createtable(L,0,8);
                gs_setstr(L,"stage",rk==1?"underground_maze":"fzero_grand_prix");
                gs_setint(L,"candidate",gw_ScriptGame_ObjectiveRoute(0,0,1));
                gs_setint(L,"latched",gw_ScriptGame_ObjectiveRoute(0,0,2));
                if(rk==1){gs_setint(L,"phase",gw_ScriptGame_ObjectiveRoute(0,0,3));gs_setint(L,"done_mask",gw_ScriptGame_ObjectiveRoute(0,0,4));
                    gs_setbool(L,"exit_chosen",gw_ScriptGame_ObjectiveRoute(0,0,5)!=0);}
                else gs_setint(L,"checkpoints_passed",gw_ScriptGame_ObjectiveRoute(0,0,3));
                for(g=1;g<=2;++g){
                    lua_newtable(L);k=0;
                    for(inx=0;inx<26;++inx){
                        int id;
                        if(!gw_ScriptGame_ObjectiveRoute(g,inx,0)) continue;
                        id=gw_ScriptGame_ObjectiveRoute(g,inx,4);
                        lua_createtable(L,0,9);
                        gs_setint(L,"index",inx);gs_setint(L,"id",id);
                        gs_setnum(L,"x",gs_skill_f(gw_ScriptGame_ObjectiveRoute(g,inx,1)));gs_setnum(L,"y",gs_skill_f(gw_ScriptGame_ObjectiveRoute(g,inx,2)));gs_setnum(L,"z",gs_skill_f(gw_ScriptGame_ObjectiveRoute(g,inx,3)));
                        if(g==1){
                            int cand=gw_ScriptGame_ObjectiveRoute(0,0,1),lat=gw_ScriptGame_ObjectiveRoute(0,0,2);
                            int sk=gw_ScriptGame_ObjectiveRoute(1,inx,5);
                            gs_setstr(L,"role",rk==1?(sk==3?"exit":sk==1?"room":"spot"):"finish");
                            gs_setbool(L,"inside",cand==id);gs_setbool(L,"reached",lat==id);
                            if(rk==1){gs_setint(L,"symbol",sk);gs_setbool(L,"done",gw_ScriptGame_ObjectiveRoute(1,inx,6)!=0);}
                            if(rk==2&&lat==id) fin=1;
                        } else {
                            gs_setstr(L,"role","checkpoint");
                            if(rk==2) gs_setbool(L,"passed",inx<gw_ScriptGame_ObjectiveRoute(0,0,3));
                        }
                        lua_rawseti(L,-2,++k);
                    }
                    lua_setfield(L,-2,g==1?"points":"secondary");
                }
                if(rk==2) gs_setbool(L,"finished",fin!=0);
                lua_setfield(L,-2,"route");
            }
        }
        lua_createtable(L,0,4);gs_setbool(L,"targets",1);gs_setbool(L,"doors",race!=0);gs_setbool(L,"finish",race!=0);gs_setbool(L,"route",gw_ScriptGame_ObjectiveRoute(0,0,0)!=0);lua_setfield(L,-2,"supported");
    }
    return 1;
}
