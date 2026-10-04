/* Lua adapter for the snapshot-covered scalar zone API. Reads never branch. */
#include "../gameworld/script_zones.h"
extern int gw_ScriptGame_ZoneField(int,int);
extern int gw_ScriptGame_ZoneText(int,int,int);
extern void gw_ScriptGame_ZoneDraft(int,int);
extern void gw_ScriptGame_ZoneDraftText(int,int,int);
extern int gw_ScriptGame_ZoneCommit(int,int,int);
extern int gw_ScriptGame_ZoneRemove(int,int);
extern void gw_ScriptGame_ZonesRelease(int);
extern void gw_ScriptGame_ZonesArm(int);
extern void gw_ScriptGame_ZonesFrame(int);
extern int gw_ScriptGame_ZoneEntity(int,int,int);
extern int gw_ScriptGame_ZoneEvent(int,int);
extern int gw_ScriptGame_ZoneEventText(int,int,int);
extern int gw_ScriptGame_ZoneEventCount(void);
static int gs_zone_dispatching;
static int gs_zone_serial; /* monotonic allocation watermark; not simulation input */
static int gs_zone_cleanup;
static int gs_zone_pending_release;
static SzEvent gs_zone_batch[SZ_EVENTS]; /* observer copy; never simulation input */
static const char* const gs_zone_hooks[]={"on_zone_enter","on_zone_exit","on_zone_none","on_zone_some"};
static float gs_zone_float(int b) {union {int i;float f;}u;u.i=b;return u.f;}
static int gs_zone_bits(float f) {union {int i;float f;}u;u.f=f;return u.i;}
static void gs_zone_read_text(int slot,int text,char* out,int cap,int event) {
    int i;for(i=0;i<cap-1;i++){out[i]=(char)(event ? gw_ScriptGame_ZoneEventText(slot,text,i):gw_ScriptGame_ZoneText(slot,text,i));if(!out[i])break;}out[i]=0;
}
static void gs_zone_read(int slot,SzDef* z) {
    int i;memset(z,0,sizeof *z);z->handle=gw_ScriptGame_ZoneField(slot,SZF_HANDLE);
    z->owner=gw_ScriptGame_ZoneField(slot,SZF_OWNER);z->live=gw_ScriptGame_ZoneField(slot,SZF_LIVE);
    z->model=gw_ScriptGame_ZoneField(slot,SZF_MODEL);z->n=gw_ScriptGame_ZoneField(slot,SZF_N);
    z->ntags=gw_ScriptGame_ZoneField(slot,SZF_TAGS);z->area=gs_zone_float(gw_ScriptGame_ZoneField(slot,SZF_AREA));
    z->dx=gs_zone_float(gw_ScriptGame_ZoneField(slot,SZF_DX));z->dy=gs_zone_float(gw_ScriptGame_ZoneField(slot,SZF_DY));
    gs_zone_read_text(slot,SZT_NAME,z->name,sizeof z->name,0);gs_zone_read_text(slot,SZT_LABEL,z->label,sizeof z->label,0);
    gs_zone_read_text(slot,SZT_KIND,z->kind,sizeof z->kind,0);
    for(i=0;i<z->ntags;i++)gs_zone_read_text(slot,SZT_TAG+i,z->tags[i],sizeof z->tags[i],0);
    for(i=0;i<z->n;i++){z->x[i]=gs_zone_float(gw_ScriptGame_ZoneField(slot,SZF_X+i));z->y[i]=gs_zone_float(gw_ScriptGame_ZoneField(slot,SZF_Y+i));}
}
static int gs_zone_slot(int handle) {
    int i;for(i=0;i<SZ_CAP;i++)if(gw_ScriptGame_ZoneField(i,SZF_LIVE)==1 && gw_ScriptGame_ZoneField(i,SZF_HANDLE)==handle)return i;return -1;
}
static void gs_zone_push(lua_State* L,const SzDef* z,int frames) {
    int i;lua_newtable(L);gs_setint(L,"handle",z->handle);gs_setstr(L,"name",z->name);
    gs_setstr(L,"label",z->label);gs_setstr(L,"kind",z->kind);gs_setint(L,"owner",z->owner);
    gs_setnum(L,"area",z->area);if(z->model)gs_setint(L,"model",z->model);
    if(frames>=0){gs_setint(L,"frames",frames);gs_setnum(L,"seconds",frames/60.0);}
    lua_newtable(L);for(i=0;i<z->ntags;i++){lua_pushstring(L,z->tags[i]);lua_rawseti(L,-2,i+1);}lua_setfield(L,-2,"tags");
    lua_newtable(L);for(i=0;i<z->n;i++){lua_newtable(L);lua_pushnumber(L,z->x[i]);lua_rawseti(L,-2,1);lua_pushnumber(L,z->y[i]);lua_rawseti(L,-2,2);lua_rawseti(L,-2,i+1);}lua_setfield(L,-2,"points");
    gs_setnum(L,"offset_x",z->dx);gs_setnum(L,"offset_y",z->dy);
}
static void gs_zone_string(lua_State* L,int at,const char* field,char* dst,size_t cap) {
    size_t n;const char* p;lua_getfield(L,at,field);
    if(!lua_isnil(L,-1)) {
        if(lua_type(L,-1)!=LUA_TSTRING)luaL_error(L,"zone %s must be text",field);
        p=lua_tolstring(L,-1,&n);if(!n || n>=cap || strlen(p)!=n)luaL_error(L,"zone %s needs 1..%d bytes without NUL",field,(int)cap-1);
        memcpy(dst,p,n+1);
    }lua_pop(L,1);
}
static int gs_zone_handle(lua_State* L,int at) {
    lua_Integer h=luaL_checkinteger(L,at);if(h<1 || h>INT_MAX)luaL_argerror(L,at,"positive zone handle required");return (int)h;
}
static void gs_zone_array(lua_State* L,int at,int n) {
    at=lua_absindex(L,at);lua_pushnil(L);
    while(lua_next(L,at)) {
        lua_Integer k;
        if(!lua_isinteger(L,-2))luaL_error(L,"zone arrays require consecutive integer keys");
        k=lua_tointeger(L,-2);if(k<1 || k>n)luaL_error(L,"zone arrays require consecutive integer keys");
        lua_pop(L,1);
    }
}
static int gs_zone_write(lua_State* L,int handle,int at) {
    SzDef z;int i,j,slot=handle ? gs_zone_slot(handle):-1,owner;int has_points,has_rect=0;float bounds[4];
    static const char* const coords[]={"x0","y0","x1","y1"};
    gs_require_offline(L,handle ? "zone_set":"zone_add");luaL_checktype(L,at,LUA_TTABLE);
    if(gs_zone_cleanup)return luaL_error(L,"zone definitions cannot change during resource teardown");
    if(!gs.match_active) return luaL_error(L,"zones require an active offline match");
    owner=gs_cur_script()->stage_owner;if(owner<=0)return luaL_error(L,"zone owner unavailable");
    if(handle){if(slot<0)return luaL_error(L,"zone does not exist");gs_zone_read(slot,&z);if(z.owner!=owner)return luaL_error(L,"zone belongs to another script");}
    else {memset(&z,0,sizeof z);strcpy(z.kind,"trigger");}
    lua_pushnil(L);while(lua_next(L,at)) {
        const char* k=lua_type(L,-2)==LUA_TSTRING ? lua_tostring(L,-2):NULL;
        if(!k || (strcmp(k,"name") && strcmp(k,"label") && strcmp(k,"kind") && strcmp(k,"tags") && strcmp(k,"model") && strcmp(k,"points") && strcmp(k,"x0") && strcmp(k,"y0") && strcmp(k,"x1") && strcmp(k,"y1")))return luaL_error(L,"unknown zone definition field");lua_pop(L,1);
    }
    gs_zone_string(L,at,"name",z.name,sizeof z.name);gs_zone_string(L,at,"label",z.label,sizeof z.label);gs_zone_string(L,at,"kind",z.kind,sizeof z.kind);
    if(!handle && !z.label[0])strcpy(z.label,z.name);
    lua_getfield(L,at,"model");if(!lua_isnil(L,-1)){lua_Integer v=luaL_checkinteger(L,-1);if(v<0 || v>INT_MAX)return luaL_error(L,"model must be zero or a positive handle");z.model=(int)v;}lua_pop(L,1);
    lua_getfield(L,at,"tags");if(!lua_isnil(L,-1)) {
        luaL_checktype(L,-1,LUA_TTABLE);z.ntags=(int)lua_rawlen(L,-1);if(z.ntags>8)return luaL_error(L,"zone permits eight tags");
        gs_zone_array(L,-1,z.ntags);memset(z.tags,0,sizeof z.tags);
        for(i=0;i<z.ntags;i++){size_t n;const char* p;lua_rawgeti(L,-1,i+1);if(lua_type(L,-1)!=LUA_TSTRING)return luaL_error(L,"zone tags must be text");p=lua_tolstring(L,-1,&n);if(!n || n>31 || strlen(p)!=n)return luaL_error(L,"zone tags need 1..31 bytes without NUL");memcpy(z.tags[i],p,n+1);lua_pop(L,1);}
    }lua_pop(L,1);
    for(i=0;i<4;i++){lua_getfield(L,at,coords[i]);if(!lua_isnil(L,-1)){has_rect++;bounds[i]=(float)luaL_checknumber(L,-1);}lua_pop(L,1);}
    lua_getfield(L,at,"points");has_points=!lua_isnil(L,-1);
    if(has_points && has_rect)return luaL_error(L,"choose rectangle or polygon");
    if(has_rect){if(has_rect!=4 || !(bounds[0]<bounds[2] && bounds[1]<bounds[3]))return luaL_error(L,"rectangle requires x0<x1 and y0<y1 with all four coordinates");z.n=4;z.x[0]=z.x[3]=bounds[0];z.x[1]=z.x[2]=bounds[2];z.y[0]=z.y[1]=bounds[1];z.y[2]=z.y[3]=bounds[3];}
    if(has_points){luaL_checktype(L,-1,LUA_TTABLE);z.n=(int)lua_rawlen(L,-1);if(z.n<3 || z.n>SZ_VERTICES)return luaL_error(L,"convex polygon requires 3..16 vertices");gs_zone_array(L,-1,z.n);for(i=0;i<z.n;i++){lua_rawgeti(L,-1,i+1);luaL_checktype(L,-1,LUA_TTABLE);gs_zone_array(L,-1,2);lua_rawgeti(L,-1,1);z.x[i]=(float)luaL_checknumber(L,-1);lua_pop(L,1);lua_rawgeti(L,-1,2);z.y[i]=(float)luaL_checknumber(L,-1);lua_pop(L,2);}}
    lua_pop(L,1);z.owner=owner;if(!sz_validate(&z))return luaL_error(L,"zone requires a name and finite, strictly convex, nonzero geometry within +/-49000");
    /* All Lua validation precedes the first game write. Commit also validates
     * capacity, name uniqueness, model existence and ownership atomically. */
    gs_rw_branch();gw_ScriptGame_ZoneDraft(-1,0);
    gw_ScriptGame_ZoneDraft(SZF_N,z.n);gw_ScriptGame_ZoneDraft(SZF_MODEL,z.model);gw_ScriptGame_ZoneDraft(SZF_TAGS,z.ntags);
    for(i=0;i<z.n;i++){gw_ScriptGame_ZoneDraft(SZF_X+i,gs_zone_bits(z.x[i]));gw_ScriptGame_ZoneDraft(SZF_Y+i,gs_zone_bits(z.y[i]));}
    for(i=0;i<SZT_TAG+z.ntags;i++) {
        const char* p=i==SZT_NAME ? z.name:i==SZT_LABEL ? z.label:i==SZT_KIND ? z.kind:z.tags[i-SZT_TAG];
        for(j=0;p[j];j++)gw_ScriptGame_ZoneDraftText(i,j,(unsigned char)p[j]);
    }
    i=gw_ScriptGame_ZoneCommit(handle,owner,gs_zone_serial);if(i<0)return luaL_error(L,"zone refused (%d): invalid definition, owner, duplicate name, capacity 64 or missing model",i);
    if(i>gs_zone_serial)gs_zone_serial=i;
    gw_log("script: zone %s %d (%s)",handle ? "updated":"added",i,z.name);lua_pushinteger(L,i);return 1;
}
static int l_zone_add(lua_State* L) { return gs_zone_write(L,0,1); }
static int l_zone_set(lua_State* L) { return gs_zone_write(L,gs_zone_handle(L,1),2); }
static int l_zone_remove(lua_State* L) {
    int h=gs_zone_handle(L,1),rc;gs_require_offline(L,"zone_remove");gs_rw_branch();
    rc=gw_ScriptGame_ZoneRemove(h,gs_cur_script()->stage_owner);if(!rc)return luaL_error(L,"zone missing or owned by another script");gw_log("script: zone removed %d",h);lua_pushboolean(L,1);return 1;
}
static int gs_zone_order(const void* aa,const void* bb) {
    const SzDef* a=aa;const SzDef* b=bb;int c=strcmp(a->kind,b->kind);
    return c ? c:a->area<b->area ? -1:a->area>b->area ? 1:a->handle<b->handle ? -1:a->handle>b->handle;
}
static void gs_zone_list(lua_State* L,int entity,int point,float x,float y) {
    SzDef list[SZ_CAP];int i,n=0;
    for(i=0;i<SZ_CAP;i++)if(gw_ScriptGame_ZoneField(i,SZF_LIVE)==1) {
        if(entity>=0 && !gw_ScriptGame_ZoneEntity(entity,3,i))continue;
        gs_zone_read(i,&list[n]);if(point && !sz_contains(&list[n],x,y))continue;n++;
    }
    qsort(list,n,sizeof list[0],gs_zone_order);lua_newtable(L);
    for(i=0;i<n;i++){int slot=gs_zone_slot(list[i].handle);gs_zone_push(L,&list[i],entity>=0 ? gw_ScriptGame_ZoneEntity(entity,4,slot):-1);lua_rawseti(L,-2,i+1);}
}
static int l_zones(lua_State* L) { gs_zone_list(L,-1,0,0,0);return 1; }
static int l_point_zones(lua_State* L) {
    float x=(float)luaL_checknumber(L,1),y=(float)luaL_checknumber(L,2);if(!isfinite(x)||!isfinite(y))return luaL_error(L,"point must be finite");gs_zone_list(L,-1,1,x,y);return 1;
}
static int l_zones_at(lua_State* L) {
    lua_Integer port=luaL_checkinteger(L,1),sub=luaL_optinteger(L,2,0);if(port<1 || port>6 || sub<0 || sub>1)return luaL_error(L,"zones_at requires port 1..6 and optional sub 0..1");gs_zone_list(L,((int)port-1)*2+(int)sub,0,0,0);return 1;
}
static int l_zone_members(lua_State* L) {
    int slot=gs_zone_slot(gs_zone_handle(L,1)),e,n=0;if(slot<0)return luaL_error(L,"zone does not exist");lua_newtable(L);
    for(e=0;e<SZ_ENTITIES;e++)if(gw_ScriptGame_ZoneEntity(e,3,slot)) {
        lua_newtable(L);gs_setint(L,"port",e/2+1);gs_setint(L,"sub",e%2);gs_setint(L,"entity",gw_ScriptGame_ZoneEntity(e,0,0));
        gs_setnum(L,"x",gs_zone_float(gw_ScriptGame_ZoneEntity(e,1,0)));gs_setnum(L,"y",gs_zone_float(gw_ScriptGame_ZoneEntity(e,2,0)));
        gs_setint(L,"frames",gw_ScriptGame_ZoneEntity(e,4,slot));lua_rawseti(L,-2,++n);
    }return 1;
}
static void gs_zones_arm(void) {
    int i,k,mask=0;if(gw_Snap_Resimulating())return;
    for(i=0;i<gs.n;i++)for(k=0;k<4;k++)if(gs_get_hook(i,gs_zone_hooks[k])){lua_pop(gs.L,1);mask|=1<<k;}
    gw_ScriptGame_ZonesArm(mask);
}
static void gs_zones_dispatch(void) {
    int n=gw_ScriptGame_ZoneEventCount(),e,i,k;SzEvent* batch;
    if(!n || !gs.L || gs_zone_dispatching || gw_Snap_Resimulating() || gw_RB_Enabled() || gw_Netplay_Enabled())return;
    /* Hooks can unload scripts: copy the batch before any Lua callback. */
    batch=gs_zone_batch;memset(batch,0,(size_t)n*sizeof *batch);
    for(e=0;e<n;e++) {
        SzEvent* v=&batch[e];v->what=gw_ScriptGame_ZoneEvent(e,SZE_WHAT);v->entity=gw_ScriptGame_ZoneEvent(e,SZE_ENTITY);v->port=gw_ScriptGame_ZoneEvent(e,SZE_PORT);v->sub=gw_ScriptGame_ZoneEvent(e,SZE_SUB);
        v->x=gs_zone_float(gw_ScriptGame_ZoneEvent(e,SZE_X));v->y=gs_zone_float(gw_ScriptGame_ZoneEvent(e,SZE_Y));v->zone.handle=gw_ScriptGame_ZoneEvent(e,SZE_ZONE);
        gs_zone_read_text(e,SZT_NAME,v->zone.name,sizeof v->zone.name,1);gs_zone_read_text(e,SZT_LABEL,v->zone.label,sizeof v->zone.label,1);gs_zone_read_text(e,SZT_KIND,v->zone.kind,sizeof v->zone.kind,1);
        for(k=0;k<SZ_CAP;k++)v->from[k]=gw_ScriptGame_ZoneEvent(e,SZE_FROM+k);
    }
    gs_zone_dispatching=1;
    for(e=0;e<n;e++)for(i=0;i<gs.n;i++) {
        SzEvent* v=&batch[e];const char* hook=gs_zone_hooks[v->what-1];lua_State* L=gs.L;
        if(!gs_may_run(i) || !gs_get_hook(i,hook))continue;
        lua_newtable(L);gs_setint(L,"zone",v->zone.handle);gs_setstr(L,"name",v->zone.name);gs_setstr(L,"label",v->zone.label);gs_setstr(L,"kind",v->zone.kind);gs_setint(L,"port",v->port);gs_setint(L,"entity",v->entity);gs_setint(L,"sub",v->sub);gs_setnum(L,"x",v->x);gs_setnum(L,"y",v->y);
        lua_newtable(L);for(k=0;k<SZ_CAP;k++)if(v->from[k]){lua_pushinteger(L,v->from[k]);lua_rawseti(L,-2,lua_rawlen(L,-2)+1);}lua_setfield(L,-2,"from");
        gs_pcall(i,1,0,hook);
    }
    gs_zone_dispatching=0;
    if(gs_zone_pending_release) {
        int terminal=gs_zone_pending_release;gs_zone_pending_release=0;
        gw_ScriptGame_ZonesFrame(terminal);gs_zones_dispatch();gs_zone_cleanup=0;
    }
}
static void gs_zones_release(int owner) {
    int terminal=owner ? 2:1;
    gw_ScriptGame_ZonesRelease(owner);gs_zone_cleanup=1;
    if(gs_zone_dispatching){if(!gs_zone_pending_release || terminal==1)gs_zone_pending_release=terminal;return;}
    gw_ScriptGame_ZonesFrame(terminal);gs_zones_dispatch();gs_zone_cleanup=0;
}
static void gs_zone_names(int port,char* out,size_t cap) {
    int i;size_t used=0;out[0]=0;
    for(i=0;i<SZ_CAP;i++)if(gw_ScriptGame_ZoneEntity(port*2,3,i)) {
        char name[81];gs_zone_read_text(i,SZT_NAME,name,sizeof name,0);
        if(used<cap)used+=(size_t)snprintf(out+used,cap-used,"%s%s",used ? ", ":"",name);
    }if(!out[0])snprintf(out,cap,"outside every zone");
}
static int gs_zone_has_name(int port,const char* name) {
    int i;for(i=0;i<SZ_CAP;i++)if(gw_ScriptGame_ZoneEntity(port*2,3,i)) {
        char value[81];gs_zone_read_text(i,SZT_NAME,value,sizeof value,0);if(!strcmp(value,name))return 1;
    }return 0;
}
