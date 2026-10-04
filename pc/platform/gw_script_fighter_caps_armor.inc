/* Typed armour scalar boundary. Authoritative records are game BSS, never Lua/native caches. */
extern int gw_ScriptGame_ArmorSet(int entity,int owner,int type,int value_bits,int frames,
                                int state,int from_bits,int to_bits,int direction);
extern int gw_ScriptGame_ArmorRead(int entity,int type,int field);
extern int gw_ScriptGame_ArmorClear(int entity,int owner,int type);
extern void gw_ScriptGame_ArmorRelease(int owner);
static const char* const gs_armor_types[]={"knockback","damage_threshold",
    "knockback_threshold","super","hit_count","damage_pool",NULL};
static int gs_armor_integer(lua_State* L,int at,int low,int high)
{
    lua_Integer n=luaL_checkinteger(L,at);
    if(n<low || n>high) luaL_error(L,"armour integer outside range");
    return (int)n;
}
static float gs_armor_bound(lua_State* L,int at)
{
    double n=luaL_checknumber(L,at);
    if(!isfinite(n) || n < -1 || n > 100000) luaL_error(L,"armour animation bound outside range");
    if(n<0 && n!=-1) luaL_error(L,"armour animation bound must be -1 or nonnegative");
    return (float)n;
}
static double gs_armor_read_number(int entity,int type,int field)
{
    union{int i;float f;} v; v.i=gw_ScriptGame_ArmorRead(entity,type,field); return v.f;
}
static int l_fighter_armor(lua_State* L)
{
    int entity=gs_caps_entity(L,1),type,i,clear=0,frames=0,state=-1,direction=0;
    int has_value=0; double value=1; float from=-1,to=-1;
    if(lua_gettop(L)==1) {
        lua_newtable(L);
        for(type=1,i=1;type<=6;++type) {
            int dir;
            if(!gw_ScriptGame_ArmorRead(entity,type,0)) continue;
            lua_createtable(L,0,10);
            lua_pushstring(L,gs_armor_types[type-1]);lua_setfield(L,-2,"type");
            gs_setnum(L,"value",gs_armor_read_number(entity,type,1));
            gs_setnum(L,"remaining",gs_armor_read_number(entity,type,2));
            gs_setint(L,"frames",gw_ScriptGame_ArmorRead(entity,type,3));
            gs_setint(L,"state",gw_ScriptGame_ArmorRead(entity,type,4));
            gs_setnum(L,"from",gs_armor_read_number(entity,type,5));
            gs_setnum(L,"to",gs_armor_read_number(entity,type,6));
            dir=gw_ScriptGame_ArmorRead(entity,type,7);
            lua_pushstring(L,dir>0 ? "front" : dir<0 ? "back" : "any");lua_setfield(L,-2,"direction");
            gs_setbool(L,"enabled",gw_ScriptGame_ArmorRead(entity,type,8)!=0);
            lua_rawseti(L,-2,i++);
        }
        return 1;
    }
    gs_caps_write(L,"fighter_armor");
    if(lua_isnil(L,2)) {
        gs_rw_branch();lua_pushboolean(L,gw_ScriptGame_ArmorClear(entity,gs.cur+1,0));return 1;
    }
    luaL_checktype(L,2,LUA_TTABLE);
    lua_getfield(L,2,"type"); type=luaL_checkoption(L,-1,NULL,gs_armor_types)+1;lua_pop(L,1);
    lua_pushnil(L);
    while(lua_next(L,2)) {
        const char* key=lua_type(L,-2)==LUA_TSTRING ? lua_tostring(L,-2) : NULL;
        if(!key) return luaL_error(L,"armour option names must be strings");
        if(!strcmp(key,"type")) { /* Checked before iteration. */ }
        else if(!strcmp(key,"value")) {value=luaL_checknumber(L,-1);if(!isfinite(value) || value<0 || value>1000) return luaL_error(L,"armour value outside range");has_value=1;}
        else if(!strcmp(key,"frames")) frames=gs_armor_integer(L,-1,0,36000);
        else if(!strcmp(key,"state")) state=gs_armor_integer(L,-1,-1,65535);
        else if(!strcmp(key,"from")) from=gs_armor_bound(L,-1);
        else if(!strcmp(key,"to")) to=gs_armor_bound(L,-1);
        else if(!strcmp(key,"direction")) {
            static const char* const names[]={"any","front","back",NULL};
            int n=luaL_checkoption(L,-1,NULL,names);direction=n==1 ? 1 : n==2 ? -1 : 0;
        } else if(!strcmp(key,"clear")) {luaL_checktype(L,-1,LUA_TBOOLEAN);clear=lua_toboolean(L,-1);}
        else return luaL_error(L,"unknown armour option");
        lua_pop(L,1);
    }
    if(from>=0 && to>=0 && from>to) return luaL_error(L,"armour animation range is reversed");
    if(!clear) {
        if(!has_value && type!=4) return luaL_error(L,"armour value required");
        if(type!=4 && value<=0) return luaL_error(L,"armour value must be positive");
        if(type==5 && (value>255 || value!=(int)value)) return luaL_error(L,"hit_count value must be integer 1-255");
    }
    gs_rw_branch();
    lua_pushboolean(L,clear ? gw_ScriptGame_ArmorClear(entity,gs.cur+1,type) :
        gw_ScriptGame_ArmorSet(entity,gs.cur+1,type,gs_fbits(value),frames,state,gs_fbits(from),gs_fbits(to),direction));
    return 1;
}
/* Observational queue only: skipped while resimulating, never read by gameplay. */
void gw_Script_ArmorEvent(int entity,int type,int absorbed,int broke,int damage_bits,int kb_bits)
{
    GsEvent* e;
    if(!gs.L || !gs.want_events || gw_Snap_Resimulating() || entity<1 || entity>12 || type<1 || type>6) return;
    if(gs.nev>=GS_MAX_EVENTS) {++gs.ev_dropped;return;}
    e=&gs.ev[gs.nev++];memset(e,0,sizeof *e);
    e->what=17;e->a=entity;e->b=type;e->c=(absorbed ? 1 : 0)|(broke ? 2 : 0);
    e->d=damage_bits;e->item_y=kb_bits;
}
static void gs_armor_push(lua_State* L,const GsEvent* e)
{
    union{int i;float f;} v;
    lua_createtable(L,0,9);
    gs_setint(L,"port",(e->a-1)%6+1);gs_setint(L,"entity",e->a);
    gs_setbool(L,"subfighter",e->a>6);
    lua_pushstring(L,e->b>=1 && e->b<=6 ? gs_armor_types[e->b-1] : "unknown");lua_setfield(L,-2,"type");
    gs_setbool(L,"absorbed",(e->c&1)!=0);gs_setbool(L,"broke",(e->c&2)!=0);
    v.i=e->d;gs_setnum(L,"damage",v.f);v.i=e->item_y;gs_setnum(L,"knockback",v.f);
}
