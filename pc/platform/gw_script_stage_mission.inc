/* Mission-slot provider: immutable exported GXMS assets + their generated
 * collision sidecars. No mission runtime, objectives, streaming or hazard code
 * runs in a slot. This is the small registration hook for the missions mod. */
typedef struct {
    int nlines,nmodels,music,bytes;
    struct {float x0,y0,x1,y1;int kind,flags;} lines[768];
    struct {int asset,handle,field[SM_FIELDS];} models[64];
    int bounds[8],spawn[261][4];
} GsStageMissionInput;
static const GsStageMissionInput* gs_stage_mission_input;
extern int gw_ScriptGame_StageSlotMission(int,int);
int gw_Script_StageSlotMissionInput(int kind,int row,int field)
{
    const GsStageMissionInput* p=gs_stage_mission_input;
    if(!p)return 0;
    if(kind==0)return field==0?p->nlines:field==1?p->nmodels:field==2?p->music:field==3?p->bytes:0;
    if(kind==1 && row>=0 && row<p->nlines) {
        switch(field) {
        case 0:return gs_fbits(p->lines[row].x0);case 1:return gs_fbits(p->lines[row].y0);
        case 2:return gs_fbits(p->lines[row].x1);case 3:return gs_fbits(p->lines[row].y1);
        case 4:return p->lines[row].kind;case 5:return p->lines[row].flags;
        }
    }
    if(kind==2 && row>=0 && row<p->nmodels)return p->models[row].handle;
    if(kind==3 && row>=0 && row<8)return p->bounds[row];
    if(kind==4 && row>=0 && row<261 && field>=0 && field<4)return p->spawn[row][field];
    return 0;
}
static int gs_stage_mission_load(lua_State* L)
{
    GsStageMissionInput* p;int i,k,j,rc,h,models,cap;
    static const char* boxkeys[]={"left","right","top","bottom"};
    p=lua_newuserdatauv(L,sizeof *p,0);memset(p,0,sizeof *p);p->music=-1;
    lua_getfield(L,1,"models");luaL_checktype(L,-1,LUA_TTABLE);models=lua_gettop(L);
    p->nmodels=(int)lua_rawlen(L,models);if(p->nmodels<1 || p->nmodels>64)return luaL_error(L,"mission slot needs 1..64 model assets");
    cap=gw_ScriptGame_StageSlotRead(0,0,33,0);
    for(i=0;i<p->nmodels;++i) {
        int asset,field[SM_FIELDS]={0};float rot,scale,sx,sy,c,s,x,y;
        GmCollision* collision;
        lua_rawgeti(L,models,i+1);luaL_checktype(L,-1,LUA_TTABLE);
        lua_getfield(L,-1,"asset");asset=gs_model_asset(L,-1);lua_pop(L,1);p->models[i].asset=asset;
        field[SM_SCALE]=field[SM_SX]=field[SM_SY]=field[SM_SZ]=gs_fbits(1);field[SM_TINT]=-1;
        field[SM_ALPHA]=gs_stage_models[asset].collision.alpha;
        gs_model_options(L,lua_absindex(L,-1),field);field[SM_VISIBLE]=0;memcpy(p->models[i].field,field,sizeof field);
        rot=gs_camera_float(field[SM_ROT])*0.017453292519943295f;scale=gs_camera_float(field[SM_SCALE]);
        sx=scale*gs_camera_float(field[SM_SX]);sy=scale*gs_camera_float(field[SM_SY]);
        c=cosf(rot);s=sinf(rot);x=gs_camera_float(field[SM_X]);y=gs_camera_float(field[SM_Y]);
        collision=&gs_stage_models[asset].collision;
        if(p->nlines+collision->count>=cap)return gs_stage_refuse(L,-7,1);
        for(k=0;k<collision->count;++k) {
            const GmLine* line=&collision->line[k];float a[4];int kind=line->kind;
            a[0]=x+c*sx*line->x0-s*sy*line->y0;a[1]=y+s*sx*line->x0+c*sy*line->y0;
            a[2]=x+c*sx*line->x1-s*sy*line->y1;a[3]=y+s*sx*line->x1+c*sy*line->y1;
            if(sx*sy<0){float t=a[0];a[0]=a[2];a[2]=t;t=a[1];a[1]=a[3];a[3]=t;}
            if(kind<=2 && sy<0)kind=3-kind;if(kind>=3 && sx<0)kind=7-kind;
            for(j=0;j<4;++j)if(!(a[j]>=-49000 && a[j]<=49000))return gs_stage_refuse(L,-16,1);
            if((kind==1 && a[0]>=a[2]) || (kind==2 && a[0]<=a[2]) || (kind==3 && a[1]<=a[3]) ||
               (kind==4 && a[1]>=a[3]))return luaL_error(L,"mission transform reverses a collision line kind; re-export it");
            j=p->nlines++;p->lines[j].x0=a[0];p->lines[j].y0=a[1];p->lines[j].x1=a[2];p->lines[j].y1=a[3];
            p->lines[j].kind=kind;p->lines[j].flags=kind==1?line->flags:0;
        }
        p->bytes+=gs_stage_models[asset].bytes+gs_stage_models[asset].image_bytes+gs_stage_models[asset].glow_bytes;
        lua_pop(L,1);
    }
    lua_pop(L,1);
    if(!p->nlines)return luaL_error(L,"mission slot has no exported collision");
    for(i=0;i<2;++i) {
        lua_getfield(L,1,i?"blast":"camera");luaL_checktype(L,-1,LUA_TTABLE);
        for(k=0;k<4;++k){float v;lua_getfield(L,-1,boxkeys[k]);v=gs_stage_num(L,-1);lua_pop(L,1);p->bounds[4*i+k]=gs_fbits(v);}
        if(gs_camera_float(p->bounds[4*i])>=gs_camera_float(p->bounds[4*i+1]) ||
           gs_camera_float(p->bounds[4*i+3])>=gs_camera_float(p->bounds[4*i+2]))return luaL_error(L,"mission camera/blast bounds are inverted");
        lua_pop(L,1);
    }
    lua_getfield(L,1,"spawns");
    if(!lua_isnil(L,-1)) {
        luaL_checktype(L,-1,LUA_TTABLE);lua_pushnil(L);
        while(lua_next(L,-2)) {
            lua_Integer id=luaL_checkinteger(L,-2);if(id<0 || id>260)return luaL_error(L,"spawn id must be 0..260");
            luaL_checktype(L,-1,LUA_TTABLE);p->spawn[id][0]=1;
            for(k=0;k<3;++k){float v;lua_getfield(L,-1,k==0?"x":k==1?"y":"z");v=lua_isnil(L,-1)?0:gs_stage_num(L,-1);lua_pop(L,1);p->spawn[id][k+1]=gs_fbits(v);}
            lua_pop(L,1);
        }
    }
    lua_pop(L,1);lua_getfield(L,1,"music_id");if(!lua_isnil(L,-1)){lua_Integer m=luaL_checkinteger(L,-1);if(m< -1||m>4095)return luaL_error(L,"music_id must be -1..4095");p->music=(int)m;}lua_pop(L,1);
    if(gs_stage_handle_serial>INT_MAX-65)return luaL_error(L,"stage handle space exhausted");
    /* All Lua validation precedes mutation; rollback only the instances created
     * here when either instance or slot allocation refuses. */
    gs_rw_branch();h=gs_stage_next_handle(L);
    for(i=0;i<p->nmodels;++i) {
        int asset=p->models[i].asset;
        memcpy(gs_model_input,p->models[i].field,sizeof p->models[i].field);
        for(k=0;k<3;++k)gs_model_input[SM_FIELDS+k]=gs_fbits(gs_stage_models[asset].center[k]);
        p->models[i].handle=gs_stage_next_handle(L);
        rc=gw_ScriptGame_ModelSpawn(asset,gs_stage_models[asset].token,p->models[i].handle,0,0);
        if(rc<0) {for(k=0;k<i;++k)gw_ScriptGame_ModelDespawn(p->models[k].handle);return gs_stage_refuse(L,-4,1);}
    }
    gs_stage_mission_input=p;rc=gw_ScriptGame_StageSlotMission(gs.cur+1,h);gs_stage_mission_input=NULL;
    if(rc<0){for(k=0;k<p->nmodels;++k)gw_ScriptGame_ModelDespawn(p->models[k].handle);return gs_stage_refuse(L,rc,1);}
    ++gs_stage_session_epoch;gs_rw_stop();lua_pushinteger(L,h);return 1;
}
