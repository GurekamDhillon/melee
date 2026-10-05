/* Crits, script side (included after gw_script_skill.inc). The decision is game-side (pc/gameworld/script_crit.inc);
 * this file owns the configuration calls and the on_crit event (engine event 19). */
extern int gw_ScriptGame_CritSet(int entity,int owner,int slot,int chance,int mult,int mult_max,int launch,int min_percent,int force,int flags);
extern int gw_ScriptGame_CritRead(int entity,int field);
extern void gw_ScriptGame_CritSeed(int seed);
extern int gw_ScriptGame_CritSeedRead(void);
extern void gw_ScriptGame_CritRelease(int owner);
#define GS_EV_CRIT 19
static const char* const gs_crit_tags[10]={"default","jab","dash_attack","tilt","smash","aerial","grab","throw","special","projectile"};
/* Observational queue only. Released by the game-side frame observer after the victim's hitlag is known. */
void gw_Script_CritEvent(int a,int b,int c,int d,int base,int final_dmg,int strength,int x,int y,int z,int hitlag,int mult)
{
    GsEvent* e;
    if(!gs.L||!gs.want_events||gw_Snap_Resimulating()||a<1||a>12||b<1||b>12) return;
    if(gs.nev>=GS_MAX_EVENTS){++gs.ev_dropped;return;}
    e=&gs.ev[gs.nev++];memset(e,0,sizeof *e);
    e->what=GS_EV_CRIT;e->a=a;e->b=b;e->c=c;e->d=d;
    e->hitf[0]=gs_skill_f(base);e->hitf[1]=gs_skill_f(final_dmg);e->hitf[2]=gs_skill_f(strength);
    e->hitf[3]=gs_skill_f(x);e->hitf[4]=gs_skill_f(y);e->hitf[5]=gs_skill_f(z);e->hitf[6]=gs_skill_f(hitlag);e->hitf[7]=gs_skill_f(mult);
    e->item_reason=gw_ScriptGame_SkillRow(0,0);
    gs_spine_event_capture(e);
}
static void gs_crit_push(lua_State* L,const GsEvent* e)
{
    int tag=e->c&0xff;
    lua_createtable(L,0,20);
    gs_setint(L,"attacker",(e->a-1)%6+1);gs_setint(L,"attacker_entity",e->a);
    gs_setint(L,"victim",(e->b-1)%6+1);gs_setint(L,"victim_entity",e->b);
    gs_setstr(L,"move_tag",gs_crit_tags[tag>=1&&tag<10?tag:0]);
    gs_setnum(L,"base_damage",e->hitf[0]);gs_setnum(L,"final_damage",e->hitf[1]);gs_setnum(L,"added_damage",e->hitf[1]-e->hitf[0]);
    gs_setnum(L,"multiplier",e->hitf[7]);gs_setnum(L,"strength",e->hitf[2]);
    gs_setnum(L,"x",e->hitf[3]);gs_setnum(L,"y",e->hitf[4]);gs_setnum(L,"z",e->hitf[5]);
    gs_setnum(L,"hitlag_frames",e->hitf[6]);gs_setnum(L,"launch",gs_skill_f(e->d));
    gs_setbool(L,"forced",(e->c&0x100)!=0);gs_setbool(L,"projectile",(e->c&0x200)!=0);
}
typedef struct {float chance,mult,mult_max,launch;} GsCritSlot;
static void gs_crit_slot_option(lua_State* L,int at,GsCritSlot* s,int is_default)
{
    static const char* const keys[]={"chance","multiplier","multiplier_max","launch","min_percent","force","tags",NULL};
    static const char* const tag_keys[]={"chance","multiplier","multiplier_max","launch",NULL};
    int has_mult=0,has_max=0;
    luaL_checktype(L,at,LUA_TTABLE);gs_item_keys(L,at,is_default?keys:tag_keys);
    lua_getfield(L,at,"chance");if(!lua_isnil(L,-1))s->chance=gs_caps_number(L,-1,0,1);lua_pop(L,1);
    lua_getfield(L,at,"multiplier");if(!lua_isnil(L,-1)){s->mult=gs_caps_number(L,-1,1,16);has_mult=1;}lua_pop(L,1);
    lua_getfield(L,at,"multiplier_max");if(!lua_isnil(L,-1)){s->mult_max=gs_caps_number(L,-1,1,16);has_max=1;}lua_pop(L,1);
    lua_getfield(L,at,"launch");if(!lua_isnil(L,-1))s->launch=gs_caps_number(L,-1,1,4);lua_pop(L,1);
    if(has_mult&&!has_max)s->mult_max=s->mult;
    if(s->mult_max<s->mult) luaL_error(L,"crit multiplier_max is below multiplier");
}
/* gd.crit(entity) reads; gd.crit(entity, nil) releases; gd.crit(entity, {chance=0..1, multiplier=1..16 [, multiplier_max],
 * launch=1..4, min_percent=0..999 (victim must already have this percent), force=0..255 (the next n eligible hits
 * crit), tags={aerial={chance=,multiplier=,...}}}) replaces this script's configuration. */
static int l_crit(lua_State* L)
{
    int e=gs_caps_entity(L,1),i,owner;
    GsCritSlot slots[10];unsigned has=1;float min_percent=0;int force=0;
    if(lua_gettop(L)==1){
        int ow=gw_ScriptGame_CritRead(e,0);
        if(!ow){lua_pushnil(L);return 1;}
        lua_createtable(L,0,10);gs_setint(L,"owner",ow);
        gs_setnum(L,"chance",gs_skill_f(gw_ScriptGame_CritRead(e,10)));gs_setnum(L,"multiplier",gs_skill_f(gw_ScriptGame_CritRead(e,11)));
        gs_setnum(L,"multiplier_max",gs_skill_f(gw_ScriptGame_CritRead(e,12)));gs_setnum(L,"launch",gs_skill_f(gw_ScriptGame_CritRead(e,13)));
        gs_setnum(L,"min_percent",gs_skill_f(gw_ScriptGame_CritRead(e,2)));gs_setint(L,"force",gw_ScriptGame_CritRead(e,3));
        gs_setint(L,"hits",gw_ScriptGame_CritRead(e,4));gs_setint(L,"crits",gw_ScriptGame_CritRead(e,5));
        gs_setnum(L,"max_multiplier",gs_skill_f(gw_ScriptGame_CritRead(e,6)));
        lua_newtable(L);has=(unsigned)gw_ScriptGame_CritRead(e,1);
        for(i=1;i<10;++i) if(has&(1u<<i)){
            lua_createtable(L,0,4);gs_setnum(L,"chance",gs_skill_f(gw_ScriptGame_CritRead(e,10+i*4)));gs_setnum(L,"multiplier",gs_skill_f(gw_ScriptGame_CritRead(e,11+i*4)));
            gs_setnum(L,"multiplier_max",gs_skill_f(gw_ScriptGame_CritRead(e,12+i*4)));gs_setnum(L,"launch",gs_skill_f(gw_ScriptGame_CritRead(e,13+i*4)));
            lua_setfield(L,-2,gs_crit_tags[i]);
        }
        lua_setfield(L,-2,"tags");
        return 1;
    }
    gs_caps_write(L,"crit");
    owner=gs.cur+1;
    {int ow=gw_ScriptGame_CritRead(e,0);if(ow&&ow!=owner){lua_pushboolean(L,0);return 1;}}
    if(lua_isnil(L,2)){gs_rw_branch();lua_pushboolean(L,gw_ScriptGame_CritSet(e,owner,0,0,0,0,0,0,0,2));return 1;}
    slots[0].chance=0;slots[0].mult=1.5f;slots[0].mult_max=1.5f;slots[0].launch=1;
    gs_crit_slot_option(L,2,&slots[0],1);
    lua_getfield(L,2,"min_percent");if(!lua_isnil(L,-1))min_percent=gs_caps_number(L,-1,0,999);lua_pop(L,1);
    lua_getfield(L,2,"force");if(!lua_isnil(L,-1)){lua_Integer n=luaL_checkinteger(L,-1);if(n<0||n>255)luaL_error(L,"crit force must be 0-255");force=(int)n;}lua_pop(L,1);
    lua_getfield(L,2,"tags");
    if(!lua_isnil(L,-1)){
        int t=lua_gettop(L);luaL_checktype(L,t,LUA_TTABLE);lua_pushnil(L);
        while(lua_next(L,t)){
            const char* name=lua_type(L,-2)==LUA_TSTRING?lua_tostring(L,-2):NULL;int k;
            for(k=1;k<10;++k) if(name&&!strcmp(name,gs_crit_tags[k])) break;
            if(k==10) luaL_error(L,"unknown crit move tag");
            slots[k]=slots[0];gs_crit_slot_option(L,lua_gettop(L),&slots[k],0);has|=1u<<k;
            lua_pop(L,1);
        }
    }
    lua_pop(L,1);
    gs_rw_branch();
    if(!gw_ScriptGame_CritSet(e,owner,0,0,0,0,0,gs_fbits(min_percent),force,1|8)){lua_pushboolean(L,0);return 1;}
    for(i=0;i<10;++i) if(has&(1u<<i)){
        if(!gw_ScriptGame_CritSet(e,owner,i,gs_fbits(slots[i].chance),gs_fbits(slots[i].mult),gs_fbits(slots[i].mult_max),gs_fbits(slots[i].launch),0,0,4)){lua_pushboolean(L,0);return 1;}
    }
    lua_pushboolean(L,1);return 1;
}
static int l_crit_force(lua_State* L)
{
    int e=gs_caps_entity(L,1),owner=gs.cur+1;lua_Integer n=luaL_checkinteger(L,2);
    if(n<0||n>255) return luaL_error(L,"crit force must be 0-255");
    gs_caps_write(L,"crit_force");
    if(gw_ScriptGame_CritRead(e,0)!=owner){lua_pushboolean(L,0);return 1;}
    gs_rw_branch();
    lua_pushboolean(L,gw_ScriptGame_CritSet(e,owner,0,0,0,0,0,gw_ScriptGame_CritRead(e,2),(int)n,8));return 1;
}
static int l_crit_seed(lua_State* L)
{
    lua_Integer n=luaL_checkinteger(L,1);
    if(n<1||n>2147483647) return luaL_error(L,"crit seed must be 1-2147483647");
    gs_caps_write(L,"crit_seed");gs_rw_branch();gw_ScriptGame_CritSeed((int)n);return 0;
}
