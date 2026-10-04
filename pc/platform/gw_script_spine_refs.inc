/* Scalar identity codec and compatibility bridge. No native actor pointers,
 * allocation watermark or query-side simulation writes. */
extern int gw_ScriptGame_EntityField(int,int,int);
typedef struct {unsigned epoch,kind,id,generation,definition;} GsEntityRef;
static const char* const gs_source_kinds[]={"","drive","opponent","fighter","item","stage"};
static lua_CFunction gs_spine_call_target(const char* name);
static int gs_ref_uint(const char** cursor,unsigned* out,char delimiter)
{
    const char* p=*cursor;unsigned n=0;int digits=0;
    if(*p=='0'&&p[1]>='0'&&p[1]<='9')return 0;
    while(*p>='0'&&*p<='9'){
        unsigned d=(unsigned)(*p++-'0');if(n>(UINT_MAX-d)/10U)return 0;
        n=n*10U+d;++digits;
    }
    if(!digits||*p!=delimiter)return 0;
    *out=n;*cursor=p+(delimiter!=0);return 1;
}
static int gs_ref_parse(const char* text,size_t length,GsEntityRef* r)
{
    const char* p=text;
    if(length<3||strlen(text)!=length||strncmp(p,"e1:",3))return 0;p+=3;
    return gs_ref_uint(&p,&r->epoch,':')&&gs_ref_uint(&p,&r->kind,':')&&
        gs_ref_uint(&p,&r->id,':')&&gs_ref_uint(&p,&r->generation,':')&&
        gs_ref_uint(&p,&r->definition,0)&&r->epoch&&r->kind>=1&&r->kind<=5;
}
static int gs_ref_current(unsigned kind,unsigned id,GsEntityRef* r)
{
    if(!gw_ScriptGame_EntityField((int)kind,(int)id,0))return 0;
    r->kind=kind;r->id=id;
    r->epoch=(unsigned)gw_ScriptGame_EntityField((int)kind,(int)id,1);
    r->generation=(unsigned)gw_ScriptGame_EntityField((int)kind,(int)id,2);
    r->definition=(unsigned)gw_ScriptGame_EntityField((int)kind,(int)id,3);return r->epoch!=0;
}
static void gs_ref_text(const GsEntityRef* r,char out[80])
{
    snprintf(out,80,"e1:%u:%u:%u:%u:%u",r->epoch,r->kind,r->id,r->generation,r->definition);
}
static int gs_ref_live(const GsEntityRef* r)
{
    GsEntityRef now;
    return gs_ref_current(r->kind,r->id,&now)&&r->epoch==now.epoch&&
        r->generation==now.generation&&r->definition==now.definition;
}
static int gs_ref_arg(lua_State* L,int at,GsEntityRef* r,int required)
{
    size_t n;const char* s;
    if(lua_type(L,at)!=LUA_TSTRING)return required?luaL_argerror(L,at,"versioned entity reference required"):0;
    s=lua_tolstring(L,at,&n);
    if(!gs_ref_parse(s,n,r)||!gs_ref_live(r))return required?luaL_argerror(L,at,"malformed or stale entity reference"):0;
    return 1;
}
static void gs_entity_push_current(lua_State* L,int kind,int id,const char* field)
{
    GsEntityRef r;char text[80];
    if(gs_ref_current((unsigned)kind,(unsigned)id,&r)){gs_ref_text(&r,text);lua_pushstring(L,text);}
    else lua_pushnil(L);
    lua_setfield(L,-2,field);
}
static unsigned gs_ref_integer(lua_State* L,int at,unsigned max)
{
    lua_Integer n=luaL_checkinteger(L,at);
    if(n<0||(unsigned long long)n>max)luaL_argerror(L,at,"reference integer out of range");return (unsigned)n;
}
static int l_entity_ref(lua_State* L)
{
    GsEntityRef r;unsigned kind=1,id=0;char text[80];
    if(lua_type(L,1)==LUA_TSTRING){gs_ref_arg(L,1,&r,1);lua_pushvalue(L,1);return 1;}
    if(lua_type(L,1)==LUA_TNUMBER){unsigned port=gs_ref_integer(L,1,6);if(!port)return luaL_argerror(L,1,"port1..6");id=(port-1)*2;}
    else {
        const char* k;size_t kn;int sub=0;luaL_checktype(L,1,LUA_TTABLE);
        lua_getfield(L,1,"kind");k=lua_isnil(L,-1)?"fighter":luaL_checklstring(L,-1,&kn);
        if(!lua_isnil(L,-1)&&strlen(k)!=kn)return luaL_error(L,"malformed entity class");
        kind=!strcmp(k,"fighter")?1:!strcmp(k,"item")||!strcmp(k,"article")||!strcmp(k,"projectile")?2:!strcmp(k,"echo")?3:!strcmp(k,"stage_object")?4:!strcmp(k,"stage_actor")?5:0;
        if(!kind)return luaL_error(L,"unknown entity class");lua_pop(L,1);
        if(kind==1){unsigned port;lua_getfield(L,1,"port");port=gs_ref_integer(L,-1,6);lua_pop(L,1);if(!port)return luaL_error(L,"port1..6");
            lua_getfield(L,1,"sub");if(!lua_isnil(L,-1)){luaL_checktype(L,-1,LUA_TBOOLEAN);sub=lua_toboolean(L,-1);}lua_pop(L,1);id=(port-1)*2+sub;
        }else{lua_getfield(L,1,"id");id=gs_ref_integer(L,-1,UINT_MAX);lua_pop(L,1);}
    }
    if(!gs_ref_current(kind,id,&r)){lua_pushnil(L);return 1;}gs_ref_text(&r,text);lua_pushstring(L,text);return 1;
}
static void gs_ref_push(lua_State* L,const GsEntityRef* r)
{
    static const char* const names[]={"","fighter","item","echo","stage_object","stage_actor"};
    int owner=gw_ScriptGame_EntityField((int)r->kind,(int)r->id,4);
    lua_createtable(L,0,12);gs_setint(L,"version",1);gs_setstr(L,"kind",names[r->kind]);
    lua_pushinteger(L,r->epoch);lua_setfield(L,-2,"epoch");lua_pushinteger(L,r->id);lua_setfield(L,-2,"id");
    lua_pushinteger(L,r->generation);lua_setfield(L,-2,"generation");lua_pushinteger(L,r->definition);lua_setfield(L,-2,"definition");
    gs_setint(L,"roles",gw_ScriptGame_EntityField((int)r->kind,(int)r->id,5));
    if(owner){gs_setint(L,"owner_port",(owner-1)/2+1);gs_setbool(L,"owner_sub",(owner-1)%2);gs_entity_push_current(L,1,owner-1,"owner_ref");}
    if(r->kind==1){gs_setint(L,"port",r->id/2+1);gs_setbool(L,"sub",r->id%2);gs_setint(L,"collision_entity",r->id);gs_setint(L,"capability_entity",r->id/2+1+(r->id%2)*6);}
}
static int l_entity_resolve(lua_State* L){GsEntityRef r;if(!gs_ref_arg(L,1,&r,0)){lua_pushnil(L);return 1;}gs_ref_push(L,&r);return 1;}
static int l_entity_valid(lua_State* L){GsEntityRef r;lua_pushboolean(L,gs_ref_arg(L,1,&r,0));return 1;}
static int gs_source_key(const char* s,size_t n)
{
    size_t i;if(!n||n>96||strlen(s)!=n)return 0;
    for(i=0;i<n;++i)if(!((s[i]>='a'&&s[i]<='z')||(s[i]>='A'&&s[i]<='Z')||(s[i]>='0'&&s[i]<='9')||s[i]=='_'||s[i]=='-'))return 0;return 1;
}
static int l_source_ref(lua_State* L)
{
    size_t n,kn;const char* kind=luaL_checklstring(L,1,&kn);const char* key=luaL_checklstring(L,2,&n);int k;unsigned revision;const char* binding="-";GsEntityRef r;
    if(strlen(kind)!=kn)return luaL_error(L,"malformed rule source class");
    for(k=1;k<=5&&strcmp(kind,gs_source_kinds[k]);++k){}if(k>5)return luaL_error(L,"unknown rule source class");
    if(!gs_source_key(key,n))return luaL_error(L,"source key must be1..96 letters/digits/underscore/hyphen");
    revision=lua_isnoneornil(L,3)?1:gs_ref_integer(L,3,INT_MAX);if(!revision)return luaL_error(L,"source revision must be positive");
    if(!lua_isnoneornil(L,4)){gs_ref_arg(L,4,&r,1);binding=lua_tostring(L,4);}
    lua_pushfstring(L,"s1:%d:%d/%s/%s",k,(int)revision,binding,key);return 1;
}
static int l_source_resolve(lua_State* L)
{
    size_t n;const char* s=luaL_checklstring(L,1,&n),*p,*slash;unsigned kind,revision;char binding[80];GsEntityRef r;
    if(strlen(s)!=n||strncmp(s,"s1:",3)){lua_pushnil(L);return 1;}p=s+3;
    if(!gs_ref_uint(&p,&kind,':')||kind<1||kind>5||!gs_ref_uint(&p,&revision,'/')||!revision||revision>INT_MAX){lua_pushnil(L);return 1;}
    slash=strchr(p,'/');if(!slash||slash-p>=80||!gs_source_key(slash+1,strlen(slash+1))){lua_pushnil(L);return 1;}
    memcpy(binding,p,(size_t)(slash-p));binding[slash-p]=0;
    if(strcmp(binding,"-")&&(!gs_ref_parse(binding,strlen(binding),&r)||!gs_ref_live(&r))){lua_pushnil(L);return 1;}
    lua_newtable(L);gs_setint(L,"version",1);gs_setstr(L,"kind",gs_source_kinds[kind]);gs_setint(L,"revision",revision);gs_setstr(L,"key",slash+1);
    if(strcmp(binding,"-"))gs_setstr(L,"entity_ref",binding);return 1;
}
static void gs_ref_selector(lua_State* L,int at,int sub,int numeric)
{
    if(!lua_isnoneornil(L,at)){
        int supplied;if(numeric){lua_Integer n=luaL_checkinteger(L,at);if(n<0||n>1)luaL_error(L,"sub selector0..1");supplied=(int)n;}
        else {luaL_checktype(L,at,LUA_TBOOLEAN);supplied=lua_toboolean(L,at);}
        if(supplied!=sub)luaL_error(L,"selector conflicts with entity reference");
    }
    while(lua_gettop(L)<at)lua_pushnil(L);
    if(numeric)lua_pushinteger(L,sub);else lua_pushboolean(L,sub);lua_replace(L,at);
}
static int l_entity_call(lua_State* L)
{
    GsEntityRef r;const char* op;size_t on;lua_CFunction fn;int sub,port;
    gs_ref_arg(L,1,&r,1);if(r.kind!=1)return luaL_error(L,"this compatibility operation requires a fighter reference");
    op=luaL_checklstring(L,2,&on);if(strlen(op)!=on)return luaL_error(L,"malformed compatibility operation");fn=gs_spine_call_target(op);if(!fn)return luaL_error(L,"unsupported entity compatibility operation");
    sub=r.id%2;port=r.id/2+1;
    /* Remove operation name; existing setter keeps its own permission/owner gates. */
    lua_remove(L,2);lua_pushinteger(L,port);lua_replace(L,1);
    if(!strcmp(op,"fighter_caps")||!strcmp(op,"fighter_effect")||!strcmp(op,"fighter_armour")||!strcmp(op,"fighter_armor")||!strcmp(op,"fighter_timed_status")||!strcmp(op,"give_item")||!strcmp(op,"nearest_opponent")||!strcmp(op,"opponents_in_radius")){
        lua_pushinteger(L,port+sub*6);lua_replace(L,1);
    }else if(!strcmp(op,"echo_add")){
        int src=2,dest;luaL_checktype(L,src,LUA_TTABLE);lua_getfield(L,src,"sub");
        if(!lua_isnil(L,-1)){luaL_checktype(L,-1,LUA_TBOOLEAN);if(lua_toboolean(L,-1)!=sub)return luaL_error(L,"selector conflicts with entity reference");}lua_pop(L,1);
        lua_newtable(L);dest=lua_gettop(L);lua_pushnil(L);while(lua_next(L,src)){lua_pushvalue(L,-2);lua_pushvalue(L,-2);lua_settable(L,dest);lua_pop(L,1);}
        lua_pushboolean(L,sub);lua_setfield(L,dest,"sub");lua_replace(L,2);
    }else if(!strcmp(op,"zones_at"))gs_ref_selector(L,2,sub,1);
    else if(!strcmp(op,"hit_rules")||!strcmp(op,"hit_rules_clear")||!strcmp(op,"echoes"))gs_ref_selector(L,2,sub,0);
    else gs_ref_selector(L,3,sub,0);
    return fn(L);
}
