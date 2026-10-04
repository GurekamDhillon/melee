/* GAME THREAD ONLY presentation descriptors and scalar style capture.
 * Authoritative armed/contact state comes from game BSS. Copy captured values
 * into the renderer Binding payload; rendering must never call this seam. */
extern int gw_ScriptGame_EchoState(int,int,int);
#define GS_ECHO_VISUALS 64
typedef struct { int owner,emitter,entity,copies,spacing,custom[12];float brightness[12],tint[12][4]; } GsEchoVisual;
static GsEchoVisual gs_echo_visuals[GS_ECHO_VISUALS];
static void gs_echo_visual_release(int owner) {int i;for(i=0;i<GS_ECHO_VISUALS;++i)if(!owner||gs_echo_visuals[i].owner==owner)memset(&gs_echo_visuals[i],0,sizeof gs_echo_visuals[i]);}
static void gs_echo_visual_sweep(void) {
    int v;for(v=0;v<GS_ECHO_VISUALS;++v)if(gs_echo_visuals[v].owner){GwMotionOptions live;if(!gw_motion_get((unsigned)gs_echo_visuals[v].owner,gs_echo_visuals[v].emitter,&live)||live.kind!=GW_MOTION_AFTERIMAGE)memset(&gs_echo_visuals[v],0,sizeof gs_echo_visuals[v]);}
}
static int gs_echo_visual_find(int owner,int emitter) {
    GwMotionOptions o;int v,free_slot=-1;
    gs_echo_visual_sweep();
    if(!gw_motion_get((unsigned)owner,emitter,&o)||o.kind!=GW_MOTION_AFTERIMAGE){for(v=0;v<GS_ECHO_VISUALS;++v)if(gs_echo_visuals[v].owner==owner&&gs_echo_visuals[v].emitter==emitter)memset(&gs_echo_visuals[v],0,sizeof gs_echo_visuals[v]);return -1;}
    for(v=0;v<GS_ECHO_VISUALS;++v){if(gs_echo_visuals[v].owner==owner&&gs_echo_visuals[v].emitter==emitter)break;if(!gs_echo_visuals[v].owner&&free_slot<0)free_slot=v;}
    if(v==GS_ECHO_VISUALS){if(free_slot<0)return -1;v=free_slot;gs_echo_visuals[v]=(GsEchoVisual){owner,emitter,(o.port-1)*2+o.sub,o.copies,o.spacing};}
    gs_echo_visuals[v].entity=(o.port-1)*2+o.sub;gs_echo_visuals[v].copies=o.copies;gs_echo_visuals[v].spacing=o.spacing;return v;
}
/* fields:0 age,1 handle,2 armed,3 element,4 flash frames,5 brightness bits,
 * 6..9 rgba bits. Return -1 when the copy is not described by this helper.
 * port is zero based. GAME THREAD ONLY: snapshot returned values into the
 * rendering Binding payload; a render/replay thread may not read game BSS. */
int gw_EchoCopyStyle(int port,int sub,int copy,int field) {
    static const float colors[5][3]={{.45f,.8f,1},{1,.3f,.08f},{1,.9f,.15f},{.4f,.9f,1},{.65f,.2f,1}};
    int v,r,age,handle=0,armed=0,element=0,flash=0,color=0;
    if(port<0||port>=6||sub<0||sub>1||copy<1||copy>12||field<0||field>9)return -1;
    for(v=0;v<GS_ECHO_VISUALS;++v)if(gs_echo_visuals[v].owner&&gs_echo_visuals[v].entity==port*2+sub&&copy<=gs_echo_visuals[v].copies)break;
    if(v==GS_ECHO_VISUALS)return -1;{GwMotionOptions o;if(!gw_motion_get((unsigned)gs_echo_visuals[v].owner,gs_echo_visuals[v].emitter,&o)){memset(&gs_echo_visuals[v],0,sizeof gs_echo_visuals[v]);return -1;}}age=copy*gs_echo_visuals[v].spacing;
    for(r=0;r<GS_ECHO_MAX;++r)if(gw_ScriptGame_EchoRead(port*2+sub,r,0)&&gw_ScriptGame_EchoRead(port*2+sub,r,1)==age) {
        int h=gw_ScriptGame_EchoRead(port*2+sub,r,0),a=gw_ScriptGame_EchoState(port*2+sub,h,0),f=gw_ScriptGame_EchoState(port*2+sub,h,2);
        if(!handle||a||f>flash){handle=h;element=gw_ScriptGame_EchoState(port*2+sub,h,1);}armed=armed||a;if(f>flash)flash=f;
    }
    color=element==1?1:element==2?2:element==5?3:element==13?4:0;
    if(field==0)return age;if(field==1)return handle;if(field==2)return armed;if(field==3)return element;if(field==4)return flash;
    if(field==5)return gs_fbits(flash>0?1.0f:armed?.9f:gs_echo_visuals[v].custom[copy-1]?gs_echo_visuals[v].brightness[copy-1]:.5f);
    return gs_fbits(flash>0?1.0f:!armed&&gs_echo_visuals[v].custom[copy-1]?gs_echo_visuals[v].tint[copy-1][field-6]:field==9?1.0f:colors[color][field-6]);
}
static int l_afterimage_copy_set(lua_State *L) {
    static const char *const keys[]={"brightness","tint"};lua_Integer h=luaL_checkinteger(L,1),raw=luaL_checkinteger(L,2);int v,copy;
    float brightness=.5f,tint[4]={.45f,.8f,1,1};
    if(h<1||h>INT_MAX||raw<1||raw>12)return luaL_error(L,"invalid afterimage copy address");copy=(int)raw;
    v=gs_echo_visual_find(gs.cur+1,(int)h);
    if(v<0||copy>gs_echo_visuals[v].copies)return luaL_error(L,"stale or foreign afterimage copy");
    {GwMotionOptions o;if(!gw_motion_get((unsigned)gs.cur+1,(int)h,&o))return luaL_error(L,"stale afterimage emitter");}
    luaL_checktype(L,3,LUA_TTABLE);gs_hr_keys(L,3,keys,2);if(gs_echo_visuals[v].custom[copy-1]){brightness=gs_echo_visuals[v].brightness[copy-1];memcpy(tint,gs_echo_visuals[v].tint[copy-1],sizeof tint);}
    gs_motion_number(L,3,"brightness",&brightness,0,1);gs_motion_array(L,3,"tint",tint,4,0,1);
    gs_echo_visuals[v].custom[copy-1]=1;gs_echo_visuals[v].brightness[copy-1]=brightness;memcpy(gs_echo_visuals[v].tint[copy-1],tint,sizeof tint);lua_pushboolean(L,1);return 1;
}
static int l_afterimage_copy(lua_State *L) {
    lua_Integer h=luaL_checkinteger(L,1),raw=luaL_checkinteger(L,2);int v,copy;
    if(h<1||h>INT_MAX||raw<1||raw>12)return luaL_error(L,"invalid afterimage copy address");copy=(int)raw;
    v=gs_echo_visual_find(gs.cur+1,(int)h);
    if(v<0||copy>gs_echo_visuals[v].copies){lua_pushnil(L);return 1;}
    {GwMotionOptions o;if(!gw_motion_get((unsigned)gs.cur+1,(int)h,&o)){memset(&gs_echo_visuals[v],0,sizeof gs_echo_visuals[v]);lua_pushnil(L);return 1;}}
    {int p=gs_echo_visuals[v].entity/2,s=gs_echo_visuals[v].entity%2,j;lua_createtable(L,0,8);gs_setint(L,"index",copy);gs_setint(L,"age",gw_EchoCopyStyle(p,s,copy,0));gs_setint(L,"echo_handle",gw_EchoCopyStyle(p,s,copy,1));gs_setbool(L,"armed",gw_EchoCopyStyle(p,s,copy,2));gs_setint(L,"element",gw_EchoCopyStyle(p,s,copy,3));gs_setint(L,"flash",gw_EchoCopyStyle(p,s,copy,4));gs_setnum(L,"brightness",gs_echo_float(gw_EchoCopyStyle(p,s,copy,5)));lua_createtable(L,4,0);for(j=0;j<4;++j){lua_pushnumber(L,gs_echo_float(gw_EchoCopyStyle(p,s,copy,6+j)));lua_rawseti(L,-2,j+1);}lua_setfield(L,-2,"tint");}return 1;
}
static int l_echo_afterimage(lua_State *L) {
    static const char *const keys[]={"copies","spacing","echoes","visual","sub","presentation_only"};
    int port=gs_slot_arg(L,1),at=2,copies,spacing,sub,presentation,v,i,n,handles[GS_ECHO_MAX]={0},rows[GS_ECHO_MAX][GS_ECHO_FIELDS];
    GwMotionOptions o;const char *error=NULL;size_t count;int emitter;
    luaL_checktype(L,at,LUA_TTABLE);gs_hr_keys(L,at,keys,6);copies=gs_hr_int(L,at,"copies",3,1,12);spacing=gs_hr_int(L,at,"spacing",4,1,8);
    if(copies*spacing>=60)return luaL_error(L,"echo afterimage oldest age must be below 60 (renderer lifetime bound)");
    lua_getfield(L,at,"sub");sub=gs_echo_sub(L,-1);lua_pop(L,1);lua_getfield(L,at,"presentation_only");presentation=gs_echo_sub(L,-1);lua_pop(L,1);
    memset(rows,0,sizeof rows);lua_getfield(L,at,"echoes");luaL_checktype(L,-1,LUA_TTABLE);count=lua_rawlen(L,-1);if(count>GS_ECHO_MAX)return luaL_error(L,"echo afterimage allows 8 echo rules");n=(int)count;
    for(i=0;i<n;++i) {int copy,source,dest;lua_rawgeti(L,-1,i+1);luaL_checktype(L,-1,LUA_TTABLE);source=lua_absindex(L,-1);copy=gs_hr_int(L,source,"copy",1,1,copies);
        /* Copy index is removed only from a fresh table; caller data is intact. */
        lua_newtable(L);dest=lua_absindex(L,-1);lua_pushnil(L);while(lua_next(L,source)){if(lua_type(L,-2)!=LUA_TSTRING||strcmp(lua_tostring(L,-2),"copy")){lua_pushvalue(L,-2);lua_pushvalue(L,-2);lua_settable(L,dest);}lua_pop(L,1);}
        gs_echo_rule(L,-1,rows[i]);lua_getfield(L,source,"delay");if(!lua_isnil(L,-1)&&rows[i][1]!=copy*spacing)return luaL_error(L,"echo delay must equal copy index times spacing");lua_pop(L,1);rows[i][1]=copy*spacing;if(rows[i][0])return luaL_error(L,"helper echoes cannot preassign handles");lua_pop(L,2);
    }lua_pop(L,1);
    gs_echo_visual_sweep();for(v=0;v<GS_ECHO_VISUALS;++v)if(!gs_echo_visuals[v].owner)break;if(v==GS_ECHO_VISUALS)return luaL_error(L,"echo presentation capacity exhausted");
    gw_motion_defaults(&o,GW_MOTION_AFTERIMAGE);o.port=port+1;o.sub=sub;lua_getfield(L,at,"visual");if(!lua_isnil(L,-1))gs_motion_options(L,lua_absindex(L,-1),&o);lua_pop(L,1);
    if(o.port!=port+1||o.sub!=sub)return luaL_error(L,"echo visual entity must match argument");o.copies=copies;o.spacing=spacing;o.lifetime=copies*spacing+1;
    if(!presentation){int used=0;gs_require_gameplay(L,"echo_afterimage");gs_echo_owner(L,port*2+sub);for(i=0;i<GS_ECHO_MAX;++i)if(gw_ScriptGame_EchoRead(port*2+sub,i,0))++used;if(n>GS_ECHO_MAX-used)return luaL_error(L,"echo entity capacity exhausted");}
    emitter=gw_motion_add((unsigned)gs.cur+1,&o,&error);if(!emitter)return gs_motion_result(L,0,error);
    if(!presentation&&n){gs_rw_branch();for(i=0;i<n;++i){int *b=rows[i];handles[i]=gw_ScriptGame_EchoAddWithHandle(port*2+sub,gs.cur+1,b[1],b[2],b[3],b[4],b[5],b[6],b[7],b[8],0);if(!handles[i]){int j;for(j=0;j<i;++j)gw_ScriptGame_EchoRemove(handles[j],gs.cur+1);gw_motion_remove((unsigned)gs.cur+1,emitter);return luaL_error(L,"echo helper native add refused");}}}
    gs_echo_visuals[v]=(GsEchoVisual){gs.cur+1,emitter,port*2+sub,copies,spacing};lua_pushinteger(L,emitter);lua_createtable(L,n,0);for(i=0;i<n;++i){lua_pushinteger(L,handles[i]);lua_rawseti(L,-2,i+1);}return 2;
}
