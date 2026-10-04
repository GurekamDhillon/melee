#include "../gameworld/script_contacts.h"
extern int gw_ScriptGame_ContactField(int slot,int surface,int field,int line);
extern int gw_ScriptGame_ContactFlags(int slot);
extern int gw_ScriptGame_ContactCross(int slot,int after);
#include "gw_script_contacts_core.inc"
static const char* gs_contact_kinds[5]={"floor","left_wall","right_wall","ceiling","ledge"};
static float gs_contact_float(int bits) { union {int i;float f;}u;u.i=bits;return u.f; }
static void gs_contact_surface(int slot,int kind,int line,GsContactSurface* s)
{
    int k;
    float f[8];
    memset(s,0,sizeof *s);
    s->line=gw_ScriptGame_ContactField(slot,kind,SC_LINE,line);
    if(s->line<0)return;
    s->present=1;s->kind=kind;
    s->owner=gw_ScriptGame_ContactField(slot,kind,SC_OWNER,line);
    s->part=gw_ScriptGame_ContactField(slot,kind,SC_PART,line);
    s->passthrough=gw_ScriptGame_ContactField(slot,kind,SC_PASS,line);
    snprintf(s->label,sizeof s->label,"%s",gs_contact_label(s->owner));
    for(k=0;k<8;++k)f[k]=gs_contact_float(gw_ScriptGame_ContactField(slot,kind,SC_X0+k,line));
    s->x0=f[0];s->y0=f[1];s->x1=f[2];s->y1=f[3];s->nx=f[6];s->ny=f[7];
    gs_contact_project(s,f[4],f[5]);
    /* Ledges use the facing-selected endpoint, not the fighter's hanging ECB. */
    if(kind==SC_LEDGE) {s->t=gw_ScriptGame_FighterF(slot,SF_FACING)>0 ? 0:1;s->offset=s->t*hypotf(s->x1-s->x0,s->y1-s->y0);}
    if(line>=0 || kind==SC_LEDGE) {
        float dx=s->x1-s->x0,dy=s->y1-s->y0,n=hypotf(dx,dy);
        s->nx=n>0 ? -dy/n:0;s->ny=n>0 ? dx/n:1;
    }
}
static void gs_contact_picture(int slot,GsContactPicture* p)
{
    int i,flags;
    char buf[128],anim[128];
    const char* sym;
    memset(p,0,sizeof *p);
    if(!gs.match_active || !gs_players_present(slot))return;
    p->present=1;p->state=gw_ScriptGame_FighterI(slot,SI_ACTION);p->state_frame=gs.state_frame[slot];
    p->airborne=gw_ScriptGame_FighterI(slot,SI_AIRBORNE);
    p->jumps_left=gw_ScriptGame_LabI(slot,LAB_I_MAX_JUMPS)-gw_ScriptGame_LabI(slot,LAB_I_JUMPS_USED);
    if(p->jumps_left<0)p->jumps_left=0;
    p->x=gw_ScriptGame_FighterF(slot,SF_X);p->y=gw_ScriptGame_FighterF(slot,SF_Y);
    p->vx=gw_ScriptGame_FighterF(slot,SF_VX);p->vy=gw_ScriptGame_FighterF(slot,SF_VY);p->facing=gw_ScriptGame_FighterF(slot,SF_FACING);
    sym=gw_ScriptGame_LabAnimSymbol(slot);gs_anim_name(sym,anim,sizeof anim);
    snprintf(p->state_name,sizeof p->state_name,"%s",gs_motion_name_at(slot,gw_ScriptGame_LabI(slot,LAB_I_NAME_KIND),p->state,anim,buf,sizeof buf));
    flags=gw_ScriptGame_ContactFlags(slot);p->jump=flags&1;p->pass=(flags&2)!=0;p->knocked=(flags&4)!=0;p->dead=(flags&8)!=0;p->rebirth=(flags&16)!=0;
    for(i=0;i<5;++i)gs_contact_surface(slot,i,-1,&p->surface[i]);
    gs_zone_names(slot,p->zones,sizeof p->zones);
    for(i=0;i<SZ_CAP;i++)if(gw_ScriptGame_ZoneEntity(slot*2,3,i)){
        p->zone_handle[p->zone_count]=gw_ScriptGame_ZoneEntity(slot*2,3,i);
        p->zone_frames[p->zone_count++]=gw_ScriptGame_ZoneEntity(slot*2,4,i);
    }
}
static void gs_contact_push_surface(lua_State* L,const GsContactSurface* s)
{
    if(!s->present){lua_pushnil(L);return;}
    lua_newtable(L);
    if(s->owner)gs_setint(L,"owner",s->owner);else gs_setstr(L,"owner","stage");
    gs_setstr(L,"label",s->label);gs_setint(L,"part",s->part);gs_setint(L,"line",s->line);
    gs_setstr(L,"kind",gs_contact_kinds[s->kind]);gs_setbool(L,"passthrough",s->passthrough);
    gs_setnum(L,"x0",s->x0);gs_setnum(L,"y0",s->y0);gs_setnum(L,"x1",s->x1);gs_setnum(L,"y1",s->y1);
    gs_setnum(L,"offset",s->offset);gs_setnum(L,"t",s->t);gs_push_xy(L,"normal",s->nx,s->ny);
}
static void gs_contact_push_fighter(lua_State* L,const GsContactPicture* p)
{
    gs_setnum(L,"x",p->x);gs_setnum(L,"y",p->y);gs_setnum(L,"vx",p->vx);gs_setnum(L,"vy",p->vy);gs_setnum(L,"facing",p->facing);
    gs_setstr(L,"state",p->state_name);gs_setint(L,"state_id",p->state);gs_setint(L,"state_frame",p->state_frame);
    gs_setint(L,"jumps_left",p->jumps_left);gs_setbool(L,"airborne",p->airborne);
    gs_setstr(L,"zone_names",p->zones);
    lua_newtable(L);
    for(int zi=0;zi<p->zone_count;zi++){lua_newtable(L);gs_setint(L,"handle",p->zone_handle[zi]);gs_setint(L,"frames",p->zone_frames[zi]);lua_rawseti(L,-2,zi+1);}
    lua_setfield(L,-2,"zones");
}
static int l_contacts(lua_State* L)
{
    GsContactPicture p;
    lua_Integer port=luaL_checkinteger(L,1);
    int slot,i;
    if(port<1 || port>6)return luaL_argerror(L,1,"port must be 1..6");
    slot=(int)port-1;
    gs_contact_enabled=1;gs_contact_picture(slot,&p);
    if(!p.present){lua_pushnil(L);return 1;}
    lua_newtable(L);gs_contact_push_fighter(L,&p);
    for(i=0;i<5;++i){gs_contact_push_surface(L,&p.surface[i]);lua_setfield(L,-2,gs_contact_kinds[i]);}
    return 1;
}
static int l_contact_events(lua_State* L)
{
    lua_Integer since=luaL_optinteger(L,1,0);
    int i,n=0;
    if(since<0)return luaL_argerror(L,1,"sequence must be nonnegative");
    gs_contact_enabled=1;lua_newtable(L);
    for(i=0;i<gs_contact_count;++i) {
        GsContactEvent* e=&gs_contact_ring[(gs_contact_head+i)%GS_CONTACT_CAP];
        if(e->seq<(uint64_t)since)continue;
        lua_newtable(L);gs_contact_push_fighter(L,&e->fighter);
        gs_setint(L,"seq",(lua_Integer)e->seq);gs_setint(L,"frame",e->frame);gs_setint(L,"port",e->port);
        gs_setstr(L,"event",e->kind);gs_setstr(L,"reason",e->reason);
        gs_contact_push_surface(L,&e->surface);lua_setfield(L,-2,"contact");
        /* Contact identity is also at the event's top level for agent polling. */
        if(e->surface.present) {
            if(e->surface.owner)gs_setint(L,"owner",e->surface.owner);else gs_setstr(L,"owner","stage");
            gs_setstr(L,"label",e->surface.label);gs_setint(L,"part",e->surface.part);gs_setint(L,"line",e->surface.line);
            gs_setnum(L,"offset",e->surface.offset);gs_setnum(L,"t",e->surface.t);
            gs_setstr(L,"kind",gs_contact_kinds[e->surface.kind]);gs_setbool(L,"passthrough",e->surface.passthrough);
            gs_setnum(L,"x0",e->surface.x0);gs_setnum(L,"y0",e->surface.y0);gs_setnum(L,"x1",e->surface.x1);gs_setnum(L,"y1",e->surface.y1);
            gs_push_xy(L,"normal",e->surface.nx,e->surface.ny);
        }
        lua_rawseti(L,-2,++n);
    }
    lua_pushinteger(L,(lua_Integer)gs_contact_next);lua_pushinteger(L,(lua_Integer)gs_contact_dropped);
    return 3;
}
static int l_model_label(lua_State* L)
{
    int h=gs_stage_handle_arg(L,1),i;
    size_t n;
    const char* label=luaL_checklstring(L,2,&n);
    if(n>80 || strlen(label)!=n)return luaL_argerror(L,2,"label must be at most 80 bytes with no NUL");
    for(i=0;i<(int)n;++i)if((unsigned char)label[i]<32 || (unsigned char)label[i]==127)return luaL_argerror(L,2,"label must contain printable text");
    {int slot=gw_ScriptGame_ModelSlot(h),owner=gw_Script_StageResourceOwner();
     if(slot<0 || gw_ScriptGame_ModelField(slot,-5)!=owner)return luaL_error(L,"stale model instance or another resource owner");
     if(!gs_contact_label_put(h,owner,label))return luaL_error(L,"contact label capacity exhausted (1024 per scene)");}
    lua_pushboolean(L,1);return 1;
}
/* Escape user labels and state names, including JSON control bytes. */
static void gs_contact_json_string(FILE* f,const char* s)
{
    fputc('"',f);
    for(;*s;++s) {
        unsigned char c=(unsigned char)*s;
        if(c=='"' || c=='\\'){fputc('\\',f);fputc(c,f);}
        else if(c<32)fprintf(f,"\\u%04x",c);else fputc(c,f);
    }
    fputc('"',f);
}
static void gs_contact_json(FILE* f,const GsContactEvent* e)
{
    const GsContactPicture* p=&e->fighter;
    const GsContactSurface* s=&e->surface;
    fprintf(f,"{\"seq\":%llu,\"frame\":%d,\"port\":%d,\"event\":",(unsigned long long)e->seq,e->frame,e->port);
    gs_contact_json_string(f,e->kind);fputs(",\"reason\":",f);gs_contact_json_string(f,e->reason);
    fprintf(f,",\"x\":%.9g,\"y\":%.9g,\"vx\":%.9g,\"vy\":%.9g,\"facing\":%.9g,\"state\":",p->x,p->y,p->vx,p->vy,p->facing);
    gs_contact_json_string(f,p->state_name);
    fputs(",\"zone_names\":",f);gs_contact_json_string(f,p->zones);
    fputs(",\"zones\":[",f);
    for(int zi=0;zi<p->zone_count;zi++)fprintf(f,"%s{\"handle\":%d,\"frames\":%d}",zi ? ",":"",p->zone_handle[zi],p->zone_frames[zi]);
    fputc(']',f);
    fprintf(f,",\"state_id\":%d,\"state_frame\":%d,\"jumps_left\":%d,\"airborne\":%s,\"contact\":",p->state,p->state_frame,p->jumps_left,p->airborne ? "true":"false");
    if(!s->present)fputs("null",f);
    else {
        fputs("{\"owner\":",f);if(s->owner)fprintf(f,"%d",s->owner);else fputs("\"stage\"",f);
        fputs(",\"label\":",f);gs_contact_json_string(f,s->label);
        fprintf(f,",\"part\":%d,\"line\":%d,\"kind\":",s->part,s->line);gs_contact_json_string(f,gs_contact_kinds[s->kind]);
        fprintf(f,",\"passthrough\":%s,\"x0\":%.9g,\"y0\":%.9g,\"x1\":%.9g,\"y1\":%.9g,\"offset\":%.9g,\"t\":%.9g,\"normal\":{\"x\":%.9g,\"y\":%.9g}}",s->passthrough ? "true":"false",s->x0,s->y0,s->x1,s->y1,s->offset,s->t,s->nx,s->ny);
    }
    fputs("}\n",f);
}
static void gs_contact_trace_close(void)
{ if(gs_contact_file)fclose(gs_contact_file);gs_contact_file=NULL;gs_contact_trace_owner=-1; }
static void gs_contact_trace_event(const GsContactEvent* e)
{
    long at;
    /* A record is bounded below 4096 bytes by the fixed string fields. Rotate BEFORE it. */
    if(gs_contact_bytes+4096>GS_CONTACT_TRACE_MAX) {
        char rotated[MAX_PATH+4];
        fclose(gs_contact_file);gs_contact_file=NULL;
        snprintf(rotated,sizeof rotated,"%s.1",gs_contact_path);
        remove(rotated);
        if(rename(gs_contact_path,rotated)!=0 || !(gs_contact_file=fopen(gs_contact_path,"wb"))) {
            gw_log("contact trace: rotation failed; trace stopped");gs_contact_trace_owner=-1;return;
        }
        gs_contact_bytes=0;
    }
    gs_contact_json(gs_contact_file,e);
    if(fflush(gs_contact_file)!=0 || (at=ftell(gs_contact_file))<0) {
        gw_log("contact trace: write failed; trace stopped");gs_contact_trace_close();return;
    }
    gs_contact_bytes=(size_t)at;
}
static int l_contact_trace(lua_State* L)
{
    int on=lua_toboolean(L,1),changes=0;
    const char* file="contacts.jsonl";
    char path[MAX_PATH];
    FILE* f;long bytes;
    if(gs_contact_trace_owner>=0 && gs_contact_trace_owner!=gs.cur)return luaL_error(L,"contact trace belongs to another script");
    if(lua_istable(L,1)) {
        lua_getfield(L,1,"file");if(!lua_isnil(L,-1)){file=luaL_checkstring(L,-1);if(strlen(file)!=lua_rawlen(L,-1))return luaL_error(L,"trace filename contains NUL");}
        lua_getfield(L,1,"state_changes");if(!lua_isnil(L,-1)){luaL_checktype(L,-1,LUA_TBOOLEAN);changes=lua_toboolean(L,-1);}
    } else luaL_checktype(L,1,LUA_TBOOLEAN);
    if(!on){gs_contact_trace_close();gs_contact_state_changes=0;lua_pushboolean(L,1);return 1;}
    gs_data_path(L,file,path,sizeof path);
    f=fopen(path,"ab");if(!f)return luaL_error(L,"could not open contact trace");
    if(fseek(f,0,SEEK_END)!=0){fclose(f);return luaL_error(L,"could not seek contact trace");}
    bytes=ftell(f);if(bytes<0){fclose(f);return luaL_error(L,"could not measure contact trace");}
    gs_contact_trace_close();gs_contact_file=f;gs_contact_trace_owner=gs.cur;gs_contact_bytes=(size_t)bytes;
    snprintf(gs_contact_path,sizeof gs_contact_path,"%s",path);gs_contact_enabled=1;gs_contact_state_changes=changes;
    lua_pushboolean(L,1);return 1;
}
static int l_contact_overlay(lua_State* L)
{ luaL_checktype(L,1,LUA_TBOOLEAN);gs_contact_overlay=lua_toboolean(L,1);if(gs_contact_overlay)gs_contact_enabled=1;lua_pushboolean(L,1);return 1; }
static int gs_contact_option(lua_State* L,const char* field) {lua_getfield(L,1,field);return !lua_isnil(L,-1);}
static int l_wait_until(lua_State* L)
{
    GsContactWait w={0};int i,at=-1;
    luaL_checktype(L,1,LUA_TTABLE);w.owner=gs.cur;w.timeout=600;w.radius=1;
    lua_pushnil(L);
    while(lua_next(L,1)) {
        const char* k=lua_type(L,-2)==LUA_TSTRING ? lua_tostring(L,-2):NULL;
        if(!k || (strcmp(k,"port") && strcmp(k,"x") && strcmp(k,"y") && strcmp(k,"radius") && strcmp(k,"state") && strcmp(k,"on") && strcmp(k,"airborne") && strcmp(k,"timeout") && strcmp(k,"zone")))return luaL_error(L,"unknown wait condition");
        lua_pop(L,1);
    }
    if(gs_contact_option(L,"port")){lua_Integer v=luaL_checkinteger(L,-1);if(v<1 || v>6)return luaL_error(L,"wait port must be 1..6");w.port=(int)v-1;}lua_pop(L,1);
    if(w.port<0 || w.port>5)return luaL_error(L,"wait port must be 1..6");
#define WAIT_NUM(k,member) if(gs_contact_option(L,k)){w.member=(float)luaL_checknumber(L,-1);if(!isfinite(w.member) || fabsf(w.member)>49000)return luaL_error(L,"wait " k " must be finite within +/-49000");}lua_pop(L,1)
    if(gs_contact_option(L,"x")){w.has_x=1;w.x=(float)luaL_checknumber(L,-1);}lua_pop(L,1);
    if(gs_contact_option(L,"y")){w.has_y=1;w.y=(float)luaL_checknumber(L,-1);}lua_pop(L,1);
    if(!isfinite(w.x)||!isfinite(w.y)||fabsf(w.x)>49000||fabsf(w.y)>49000)return luaL_error(L,"wait coordinates must be finite within +/-49000");
    WAIT_NUM("radius",radius);if(w.radius<0)return luaL_error(L,"wait radius must be nonnegative");
#undef WAIT_NUM
    if(gs_contact_option(L,"timeout")){lua_Integer v=luaL_checkinteger(L,-1);if(v<1 || v>360000)return luaL_error(L,"wait timeout must be 1..360000 frames");w.timeout=(int)v;}lua_pop(L,1);
    if(gs_contact_option(L,"airborne")){luaL_checktype(L,-1,LUA_TBOOLEAN);w.has_air=1;w.airborne=lua_toboolean(L,-1);}lua_pop(L,1);
    if(gs_contact_option(L,"state")) {
        w.has_state=1;
        if(lua_type(L,-1)==LUA_TSTRING) {const char* s=luaL_checkstring(L,-1);if(!*s || strlen(s)>127 || strlen(s)!=lua_rawlen(L,-1))return luaL_error(L,"wait state must be 1..127 bytes with no NUL");snprintf(w.state,128,"%s",s);}
        else {lua_Integer v=luaL_checkinteger(L,-1);if(v<0 || v>INT_MAX)return luaL_error(L,"wait state id must be nonnegative");w.state_id=(int)v;}
    }lua_pop(L,1);
    if(gs_contact_option(L,"on")) {
        w.has_on=1;
        if(lua_type(L,-1)==LUA_TSTRING){const char* s=luaL_checkstring(L,-1);if(!*s || strlen(s)>80 || strlen(s)!=lua_rawlen(L,-1))return luaL_error(L,"wait on label must be 1..80 bytes with no NUL");snprintf(w.label,sizeof w.label,"%s",s);}
        else {lua_Integer v=luaL_checkinteger(L,-1);if(v<1 || v>INT_MAX)return luaL_error(L,"wait on handle must be positive");w.on=(int)v;}
    }lua_pop(L,1);
    gs_zone_string(L,1,"zone",w.zone,sizeof w.zone);
    if(!(w.has_x||w.has_y||w.has_state||w.has_on||w.has_air||w.zone[0]))return luaL_error(L,"wait needs at least one condition");
    /* Keep completed results until their owner reuses capacity; active waits are never evicted. */
    for(i=0;i<GS_CONTACT_WAITS;++i)if(!gs_contact_waits[i].used){at=i;break;}
    if(at<0)for(i=0;i<GS_CONTACT_WAITS;++i)if(gs_contact_waits[i].done && gs_contact_waits[i].owner==gs.cur){at=i;break;}
    if(at<0 || gs_contact_wait_serial==INT_MAX)return luaL_error(L,"wait capacity exhausted");
    w.used=1;w.handle=++gs_contact_wait_serial;gs_contact_waits[at]=w;gs_contact_enabled=1;
    {GsContactPicture p;gs_contact_picture(w.port,&p);gs_contact_wait_seed(&gs_contact_waits[at],&p);if(gs_contact_matches(&w,&p))gs_contact_wait_resolve(&gs_contact_waits[at],1,"conditions reached");}
    lua_pushinteger(L,w.handle);return 1;
}
static int l_wait_status(lua_State* L)
{
    int i;lua_Integer h=luaL_checkinteger(L,1);
    for(i=0;i<GS_CONTACT_WAITS;++i)if(gs_contact_waits[i].used && gs_contact_waits[i].handle==h && gs_contact_waits[i].owner==gs.cur) {
        GsContactWait* w=&gs_contact_waits[i];lua_newtable(L);gs_setbool(L,"done",w->done);gs_setbool(L,"ok",w->ok);
        gs_setint(L,"frames",w->frames);gs_setstr(L,"reason",w->reason);return 1;
    }
    lua_pushnil(L);lua_pushliteral(L,"stale wait handle or another owner");return 2;
}
static void gs_contact_tick(void)
{
    int port;
    if(!gs_contact_enabled)return;
    for(port=0;port<6;++port) {
        GsContactPicture p;
        gs_contact_picture(port,&p);gs_contact_observe(port,&p,gs.match_frame);
        if(p.present && p.airborne) {
            int line=-1;
            while((line=gw_ScriptGame_ContactCross(port,line))>=0) {
                GsContactSurface s;gs_contact_surface(port,SC_FLOOR,line,&s);
                gs_contact_emit("pass_up_through","",port,&p,&s,gs.match_frame);
            }
        }
        gs_contact_wait_tick(port,&p);
    }
}
static void gs_contact_release(int owner)
{
    int i;
    if(gs_contact_trace_owner==owner){gs_contact_trace_close();gs_contact_state_changes=0;}
    for(i=0;i<GS_CONTACT_WAITS;++i)if(gs_contact_waits[i].used && gs_contact_waits[i].owner==owner)memset(&gs_contact_waits[i],0,sizeof gs_contact_waits[i]);
}
/* Console arguments are passed as data; no label or state is interpolated into Lua. */
static int gs_contact_console(const char* command,const char* arg)
{
    lua_State* L=gs.L;int base=lua_gettop(L),i;
    if(!strcmp(command,"contacts")) {
        if(!strcmp(arg,"overlay on") || !strcmp(arg,"overlay off")){gs_contact_overlay=!strcmp(arg,"overlay on");if(gs_contact_overlay)gs_contact_enabled=1;return 0;}
        {char* end;long p=strtol(arg,&end,10);GsContactPicture pic;char text[256];
         if(end==arg || *end || p<1 || p>6){gw_Console_Print(GS_RED,"usage: contacts <1..6> | contacts overlay on|off");return -1;}
         gs_contact_enabled=1;gs_contact_picture((int)p-1,&pic);gs_contact_overlay_text((int)p-1,&pic,text,sizeof text);if(pic.present)gw_Console_Print(GS_WHITE,"%s  x=%.1f y=%.1f vx=%.1f vy=%.1f state=%d frame=%d jumps=%d air=%s",text,pic.x,pic.y,pic.vx,pic.vy,pic.state,pic.state_frame,pic.jumps_left,pic.airborne ? "true":"false");else gw_Console_Print(GS_WHITE,"fighter absent");return 0;}
    }
    if(!strcmp(command,"trace")) {
        if(!strcmp(arg,"on") || !strcmp(arg,"off")) {
            lua_pushcfunction(L,l_contact_trace);lua_pushboolean(L,!strcmp(arg,"on"));
            if(lua_pcall(L,1,1,0)!=LUA_OK){gw_Console_Print(GS_RED,"%s",lua_tostring(L,-1));lua_settop(L,base);return -1;}lua_settop(L,base);return 0;
        }
        if(!strncmp(arg,"last",4) && (!arg[4] || arg[4]==' ')) {
            int n=10;char* end;
            if(arg[4]){long v=strtol(arg+5,&end,10);if(end==arg+5 || *end || v<1 || v>GS_CONTACT_CAP){gw_Console_Print(GS_RED,"trace last count must be 1..512");return -1;}n=(int)v;}
            if(n>gs_contact_count)n=gs_contact_count;
            for(i=gs_contact_count-n;i<gs_contact_count;++i) {
                GsContactEvent* e=&gs_contact_ring[(gs_contact_head+i)%GS_CONTACT_CAP];
                char text[512];gs_contact_event_text(e,text,sizeof text);gw_Console_Print(GS_WHITE,"%s",text);
            }
            gs_contact_enabled=1;return 0;
        }
        gw_Console_Print(GS_RED,"usage: trace on|off|last [n]");return -1;
    }
    if(!strcmp(command,"wait")) {
        char args[1024],*token;int valid=1;
        snprintf(args,sizeof args,"%s",arg);lua_pushcfunction(L,l_wait_until);lua_newtable(L);
        /* wait port=1 x=41 y=8 radius=1 state=Wait on=Floor_4m.003 airborne=false timeout=600 */
        token=strtok(args," \t");
        while(token) {
            char* value=strchr(token,'=');char* end;double number;
            if(!value){valid=0;break;}*value++=0;
            if(!strcmp(token,"zone"))lua_pushstring(L,value);
            else if(!strcmp(token,"state") || !strcmp(token,"on")) {
                number=strtod(value,&end);if(end!=value && !*end)lua_pushnumber(L,number);else lua_pushstring(L,value);
            } else if(!strcmp(token,"airborne")) {
                if(strcmp(value,"true") && strcmp(value,"false")){valid=0;break;}lua_pushboolean(L,!strcmp(value,"true"));
            } else if(!strcmp(token,"port") || !strcmp(token,"x") || !strcmp(token,"y") || !strcmp(token,"radius") || !strcmp(token,"timeout")) {
                number=strtod(value,&end);if(end==value || *end){valid=0;break;}lua_pushnumber(L,number);
            } else {valid=0;break;}
            lua_setfield(L,-2,token);token=strtok(NULL," \t");
        }
        if(!valid){lua_settop(L,base);gw_Console_Print(GS_RED,"usage: wait port=1 x=41 y=8 radius=1 state=Wait on=label airborne=false timeout=600");return -1;}
        if(lua_pcall(L,1,1,0)!=LUA_OK){gw_Console_Print(GS_RED,"%s",lua_tostring(L,-1));lua_settop(L,base);return -1;}
        {int h=(int)lua_tointeger(L,-1);for(i=0;i<GS_CONTACT_WAITS;++i)if(gs_contact_waits[i].handle==h){gs_contact_waits[i].console=1;if(gs_contact_waits[i].done)gw_Console_Print(GS_GREEN,"wait %d: %s (0 frames)",h,gs_contact_waits[i].reason);break;}}
        lua_settop(L,base);return 0;
    }
    return -1;
}
