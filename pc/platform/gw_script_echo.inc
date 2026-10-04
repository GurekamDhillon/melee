/* Native Lua adapter. Authoritative history, handles and target memory remain
 * in game BSS; this module never passes a native address into game memory. */
#define GS_ECHO_MAX 8
#define GS_ECHO_FIELDS 9
extern int gw_ScriptGame_FighterHistoryDepth(void);
extern int gw_ScriptGame_FighterHistoryRead(int,int,int,int);
extern int gw_ScriptGame_EchoAddWithHandle(int,int,int,int,int,int,int,int,int,int,int);
extern int gw_ScriptGame_EchoRemove(int,int);
extern void gw_ScriptGame_EchoClear(int);
extern int gw_ScriptGame_EchoRead(int,int,int);
extern int gw_ScriptGame_EchoOwner(int);
extern int gw_ScriptGame_EchoNextHandle(void);
extern int gw_ScriptGame_EchoReplace(int,int,int);
typedef struct { int count, rules[GS_ECHO_MAX][GS_ECHO_FIELDS]; } GsEchoRules;
static const int *gs_echo_input;
int gw_Script_EchoInput(int at) { return gs_echo_input && at>=0 && at<GS_ECHO_MAX*GS_ECHO_FIELDS ? gs_echo_input[at] : 0; }
static int gs_echo_move(lua_State *L,int at) {
    const char *s=luaL_checkstring(L,at); int i;
    if(!strcmp(s,"any")) return -1;
    for(i=0;i<9;++i) if(!strcmp(s,gs_hr_moves[i])) return i;
    /* Neutral-air is the precise action selector, not all aerial attacks. */
    if(!strcmp(s,"nair")) return 10;
    if(!strcmp(s,"fair")) return 11;if(!strcmp(s,"bair")) return 12;if(!strcmp(s,"uair")) return 13;if(!strcmp(s,"dair")) return 14;
    return luaL_error(L,"unknown echo move tag");
}
static void gs_echo_rule(lua_State *L,int at,int *b) {
    static const char *const keys[]={"handle","delay","match","damage","knockback","element","once_per_move","sub"};
    static const char *const matches[]={"move","element","airborne"};
    int j; at=lua_absindex(L,at); luaL_checktype(L,at,LUA_TTABLE); gs_hr_keys(L,at,keys,8);
    memset(b,0,GS_ECHO_FIELDS*sizeof *b); b[2]=b[3]=b[4]=b[7]=-1; b[5]=b[6]=gs_fbits(1); b[8]=1;
    b[0]=gs_hr_int(L,at,"handle",0,0,INT_MAX); b[1]=gs_hr_int(L,at,"delay",1,1,60);
    lua_getfield(L,at,"match");
    if(!lua_isnil(L,-1)) { int m=lua_absindex(L,-1); luaL_checktype(L,m,LUA_TTABLE); gs_hr_keys(L,m,matches,3);
        lua_getfield(L,m,"move"); if(!lua_isnil(L,-1)) b[2]=gs_echo_move(L,-1); lua_pop(L,1);
        lua_getfield(L,m,"element"); if(!lua_isnil(L,-1)) b[3]=gs_hr_element(L,-1); lua_pop(L,1);
        lua_getfield(L,m,"airborne"); if(!lua_isnil(L,-1)) {luaL_checktype(L,-1,LUA_TBOOLEAN);b[4]=lua_toboolean(L,-1);} lua_pop(L,1);
    } lua_pop(L,1);
    for(j=0;j<2;++j) {double n;lua_getfield(L,at,j?"knockback":"damage");n=lua_isnil(L,-1)?1:luaL_checknumber(L,-1);lua_pop(L,1);
        if(!isfinite(n)||n<.1||n>4)luaL_error(L,"echo scale must be finite and 0.1..4");b[5+j]=gs_fbits((float)n);}
    lua_getfield(L,at,"element");if(!lua_isnil(L,-1))b[7]=gs_hr_element(L,-1);lua_pop(L,1);
    lua_getfield(L,at,"once_per_move");if(!lua_isnil(L,-1)){luaL_checktype(L,-1,LUA_TBOOLEAN);b[8]=lua_toboolean(L,-1);}lua_pop(L,1);
}
static void gs_echo_owner(lua_State *L,int entity) {int owner=gw_ScriptGame_EchoOwner(entity);if(owner&&owner!=gs.cur+1)luaL_error(L,"echoes owned by another script");}
static int gs_echo_sub(lua_State *L,int at) {if(lua_isnoneornil(L,at))return 0;luaL_checktype(L,at,LUA_TBOOLEAN);return lua_toboolean(L,at);}
static void gs_echo_parse(lua_State *L,int at,GsEchoRules *r) {
    size_t n;int i;at=lua_absindex(L,at);memset(r,0,sizeof *r);lua_getfield(L,at,"rules");luaL_checktype(L,-1,LUA_TTABLE);n=lua_rawlen(L,-1);
    if(n>GS_ECHO_MAX)luaL_error(L,"echo capacity is 8 per fighter entity");r->count=(int)n;
    for(i=0;i<r->count;++i){lua_rawgeti(L,-1,i+1);gs_echo_rule(L,-1,r->rules[i]);lua_pop(L,1);}lua_pop(L,1);
}
/* Canonicalize zero handles before the journal is stored. A single caller
 * cursor covers every echo operation in that commit; replay never allocates. */
static void gs_echo_prepare(lua_State *L,int entity,GsEchoRules *r,int *next) {
    int i,j,k,e;gs_echo_owner(L,entity);
    for(i=0;i<r->count;++i) {
        int *b=r->rules[i];
        if(!b[0])for(j=0;j<GS_ECHO_MAX;++j) {
            int h=gw_ScriptGame_EchoRead(entity,j,0),used=0;if(!h)continue;
            for(k=0;k<i;++k)if(r->rules[k][0]==h)used=1;if(used)continue;
            for(k=1;k<GS_ECHO_FIELDS;++k)if(b[k]!=gw_ScriptGame_EchoRead(entity,j,k))break;
            if(k==GS_ECHO_FIELDS){b[0]=h;break;}
        }
        if(!b[0]){if(*next<1||*next==INT_MAX)luaL_error(L,"echo handle space exhausted");b[0]=(*next)++;}
        for(j=0;j<i;++j)if(r->rules[j][0]==b[0])luaL_error(L,"duplicate echo handle");
        for(e=0;e<12;++e)for(j=0;j<GS_ECHO_MAX;++j)if(gw_ScriptGame_EchoRead(e,j,0)==b[0]&&e!=entity)luaL_error(L,"echo handle belongs to another entity");
        if(b[0]>=*next){if(b[0]==INT_MAX)luaL_error(L,"echo handle space exhausted");*next=b[0]+1;}
    }
}
static void gs_echo_pair(lua_State *L,int entity_a,const GsEchoRules *a,int entity_b,const GsEchoRules *b) {
    int i,j;if(entity_a==entity_b)luaL_error(L,"sim_commit permits one echo replacement per entity");
    for(i=0;i<a->count;++i)for(j=0;j<b->count;++j)if(a->rules[i][0]>0&&a->rules[i][0]==b->rules[j][0])luaL_error(L,"sim_commit duplicate echo handle across entities");
}
static int gs_echo_apply(int entity,int owner,const GsEchoRules *r) {
    int ok;gs_echo_input=&r->rules[0][0];ok=gw_ScriptGame_EchoReplace(entity,owner,r->count);gs_echo_input=NULL;return ok;
}
static int l_echo_add(lua_State *L) {
    int port=gs_slot_arg(L,1),sub,entity,b[GS_ECHO_FIELDS],handle;
    gs_require_gameplay(L,"echo_add");gs_echo_rule(L,2,b);lua_getfield(L,2,"sub");sub=gs_echo_sub(L,-1);lua_pop(L,1);entity=port*2+sub;gs_echo_owner(L,entity);
    if(b[0])return luaL_error(L,"echo_add does not accept a preassigned handle");gs_rw_branch();
    handle=gw_ScriptGame_EchoAddWithHandle(entity,gs.cur+1,b[1],b[2],b[3],b[4],b[5],b[6],b[7],b[8],0);
    if(!handle)return luaL_error(L,"echo add refused (capacity or fighter unavailable)");lua_pushinteger(L,handle);return 1;
}
static int l_echo_remove(lua_State *L) {
    lua_Integer h=luaL_checkinteger(L,1);int ok;gs_require_gameplay(L,"echo_remove");if(h<1||h>INT_MAX)return luaL_error(L,"invalid echo handle");
    gs_rw_branch();ok=gw_ScriptGame_EchoRemove((int)h,gs.cur+1);lua_pushboolean(L,ok);return 1;
}
static int l_fighter_history_depth(lua_State *L){lua_pushinteger(L,gw_ScriptGame_FighterHistoryDepth());return 1;}
static float gs_echo_float(int bits){union{int i;float f;}v;v.i=bits;return v.f;}
static int l_fighter_history(lua_State *L) {
    static const char *const floats[]={"action_frame","x","y","z","facing"};
    static const char *const hf[]={"ax","ay","az","bx","by","bz","radius"};
    lua_Integer raw_age=luaL_checkinteger(L,2);int port=gs_slot_arg(L,1),age,entity=port*2+gs_echo_sub(L,3),i,j,n,dense=0;
    if(raw_age<0||raw_age>=gw_ScriptGame_FighterHistoryDepth())return luaL_argerror(L,2,"history age 0..60");age=(int)raw_age;
    if(!gw_ScriptGame_FighterHistoryRead(entity,age,-1,0)){lua_pushnil(L);return 1;}
    lua_createtable(L,0,12);gs_setint(L,"age",age);gs_setint(L,"generation",gw_ScriptGame_FighterHistoryRead(entity,age,-1,1));
    gs_setint(L,"attack_id",gw_ScriptGame_FighterHistoryRead(entity,age,-1,11));
    gs_setint(L,"action_state",gw_ScriptGame_FighterHistoryRead(entity,age,-1,2));gs_setint(L,"move",gw_ScriptGame_FighterHistoryRead(entity,age,-1,3));
    gs_setbool(L,"airborne",gw_ScriptGame_FighterHistoryRead(entity,age,-1,4));
    gs_setbool(L,"grounded",!gw_ScriptGame_FighterHistoryRead(entity,age,-1,4));
    for(j=0;j<5;++j)gs_setnum(L,floats[j],gs_echo_float(gw_ScriptGame_FighterHistoryRead(entity,age,-1,5+j)));
    n=gw_ScriptGame_FighterHistoryRead(entity,age,-1,10);lua_createtable(L,n,0);
    for(i=0;i<5;++i)if(gw_ScriptGame_FighterHistoryRead(entity,age,i,0)) {
        lua_createtable(L,0,20);{int source=gw_ScriptGame_FighterHistoryRead(entity,age,i,18);gs_setint(L,"source_entity",source);gs_setint(L,"owner_port",source/2+1);gs_setbool(L,"owner_sub",source%2);gs_setint(L,"source_index",gw_ScriptGame_FighterHistoryRead(entity,age,i,19));}gs_setint(L,"identity",gw_ScriptGame_FighterHistoryRead(entity,age,i,1));gs_setnum(L,"damage",gs_echo_float(gw_ScriptGame_FighterHistoryRead(entity,age,i,2)));
        gs_setint(L,"angle",gw_ScriptGame_FighterHistoryRead(entity,age,i,3));gs_setint(L,"growth",gw_ScriptGame_FighterHistoryRead(entity,age,i,4));gs_setint(L,"base",gw_ScriptGame_FighterHistoryRead(entity,age,i,5));gs_setint(L,"fixed",gw_ScriptGame_FighterHistoryRead(entity,age,i,6));gs_setint(L,"element",gw_ScriptGame_FighterHistoryRead(entity,age,i,7));
        gs_setint(L,"original_element",gw_ScriptGame_FighterHistoryRead(entity,age,i,15));gs_setint(L,"move_tag",gw_ScriptGame_FighterHistoryRead(entity,age,i,16));gs_setbool(L,"grounded",gw_ScriptGame_FighterHistoryRead(entity,age,i,17));
        for(j=0;j<7;++j)gs_setnum(L,hf[j],gs_echo_float(gw_ScriptGame_FighterHistoryRead(entity,age,i,8+j)));lua_rawseti(L,-2,++dense);
    }lua_setfield(L,-2,"hitboxes");return 1;
}
static int l_echoes(lua_State *L) {
    int e=gs_slot_arg(L,1)*2+gs_echo_sub(L,2),i,j,dense=0;
    lua_createtable(L,GS_ECHO_MAX,3);gs_setbool(L,"journal",1);gs_setint(L,"capacity",GS_ECHO_MAX);gs_setint(L,"owner",gw_ScriptGame_EchoOwner(e));
    for(i=0;i<GS_ECHO_MAX;++i) {int b[GS_ECHO_FIELDS];for(j=0;j<GS_ECHO_FIELDS;++j)b[j]=gw_ScriptGame_EchoRead(e,i,j);if(!b[0])continue;
        lua_createtable(L,0,8);gs_setint(L,"handle",b[0]);gs_setint(L,"delay",b[1]);gs_setnum(L,"damage",gs_echo_float(b[5]));gs_setnum(L,"knockback",gs_echo_float(b[6]));gs_setbool(L,"once_per_move",b[8]);
        if(b[7]>=0)gs_setstr(L,"element",gs_hr_element_name(b[7]));lua_createtable(L,0,3);{static const char *const aerial[]={"nair","fair","bair","uair","dair"};gs_setstr(L,"move",b[2]<0?"any":b[2]>=10&&b[2]<=14?aerial[b[2]-10]:b[2]<9?gs_hr_moves[b[2]]:"unknown");}gs_setstr(L,"element",gs_hr_element_name(b[3]));if(b[4]>=0)gs_setbool(L,"airborne",b[4]);lua_setfield(L,-2,"match");lua_rawseti(L,-2,++dense);
    }return 1;
}
