/* Observer-only native state. Never read by game simulation or saved in snapshots. */
#define GS_CONTACT_CAP 512
#define GS_CONTACT_WAITS 64
#define GS_CONTACT_TRACE_MAX (1024*1024)
typedef struct {
    int present, owner, part, line, kind, passthrough;
    char label[81];
    float x0,y0,x1,y1,offset,t,nx,ny;
} GsContactSurface;
typedef struct {
    int present,state,state_frame,jumps_left,airborne,jump,pass,knocked,dead,rebirth;
    float x,y,vx,vy,facing;
    char state_name[128],zones[512];
    int zone_handle[64],zone_frames[64],zone_count;
    GsContactSurface surface[5];
} GsContactPicture;
typedef struct {
    uint64_t seq; int frame,port;
    char kind[24],reason[32];
    GsContactSurface surface;
    GsContactPicture fighter;
} GsContactEvent;
typedef struct {
    int used,handle,owner,port,frames,timeout,done,ok,console;
    int has_x,has_y,has_state,has_on,has_air,airborne,on,state_id,saw_air;
    float x,y,radius;
    GsContactSurface last_on;
    char state[128],label[81],drop_label[81],reason[256],zone[81];
} GsContactWait;
#define GS_CONTACT_LABELS 1024
/* Handles and resource-owner tokens never repeat within a scene. Retain retired
 * labels so loading an earlier same-scene snapshot can identify that instance. */
static struct {int handle,owner;char label[81];} gs_contact_labels[GS_CONTACT_LABELS];
static const char* gs_contact_label(int handle)
{
    int i;
    for(i=0;i<GS_CONTACT_LABELS;++i)if(handle>0 && gs_contact_labels[i].handle==handle)return gs_contact_labels[i].label;
    return "";
}
static int gs_contact_label_put(int handle,int owner,const char* label)
{
    int i,free_slot=-1;
    for(i=0;i<GS_CONTACT_LABELS;++i) {
        if(!gs_contact_labels[i].handle && free_slot<0)free_slot=i;
        if(gs_contact_labels[i].handle==handle) {
            if(gs_contact_labels[i].owner!=owner)return 0;
            snprintf(gs_contact_labels[i].label,81,"%s",label);return 1;
        }
    }
    if(free_slot<0)return 0;
    gs_contact_labels[free_slot].handle=handle;gs_contact_labels[free_slot].owner=owner;
    snprintf(gs_contact_labels[free_slot].label,81,"%s",label);return 1;
}
static GsContactEvent gs_contact_ring[GS_CONTACT_CAP];
static GsContactPicture gs_contact_previous[6];
static GsContactWait gs_contact_waits[GS_CONTACT_WAITS];
static uint64_t gs_contact_next=1,gs_contact_dropped;
static int gs_contact_count,gs_contact_head,gs_contact_enabled,gs_contact_overlay,gs_contact_state_changes;
static int gs_contact_wait_serial;
static FILE* gs_contact_file;
static char gs_contact_path[MAX_PATH];
static size_t gs_contact_bytes;
static int gs_contact_trace_owner=-1;
static void gs_contact_trace_event(const GsContactEvent* e);
static void gs_contact_project(GsContactSurface* s,float x,float y)
{
    float dx,dy,len2,tmp;
    if(s->x0>s->x1 || (s->x0==s->x1 && s->y0>s->y1)) {
        tmp=s->x0;s->x0=s->x1;s->x1=tmp; tmp=s->y0;s->y0=s->y1;s->y1=tmp;
    }
    dx=s->x1-s->x0;dy=s->y1-s->y0;len2=dx*dx+dy*dy;
    s->t=len2>0 ? ((x-s->x0)*dx+(y-s->y0)*dy)/len2 : 0;
    if(s->t<0)s->t=0;if(s->t>1)s->t=1;
    s->offset=s->t*sqrtf(len2);
}
static void gs_contact_emit(const char* kind,const char* reason,int port,
    const GsContactPicture* p,const GsContactSurface* s,int frame)
{
    GsContactEvent* e;
    int at=(gs_contact_head+gs_contact_count)%GS_CONTACT_CAP;
    if(gs_contact_count==GS_CONTACT_CAP) {
        at=gs_contact_head;gs_contact_head=(gs_contact_head+1)%GS_CONTACT_CAP;++gs_contact_dropped;
    } else ++gs_contact_count;
    e=&gs_contact_ring[at];memset(e,0,sizeof *e);
    e->seq=gs_contact_next++;e->port=port+1;e->frame=frame;e->fighter=*p;
    if(s)e->surface=*s;
    snprintf(e->kind,sizeof e->kind,"%s",kind);snprintf(e->reason,sizeof e->reason,"%s",reason);
    if(gs_contact_file)gs_contact_trace_event(e);
    if(!strcmp(kind,"land")) {
        int i;
        for(i=0;i<GS_CONTACT_WAITS;++i)if(gs_contact_waits[i].used && !gs_contact_waits[i].done && gs_contact_waits[i].port==port)
            gs_contact_waits[i].drop_label[0]=0; /* a later landing supersedes an earlier drop */
    }
    if(!strcmp(kind,"drop_through")) {
        int i;
        for(i=0;i<GS_CONTACT_WAITS;++i) if(gs_contact_waits[i].used && !gs_contact_waits[i].done && gs_contact_waits[i].port==port)
            snprintf(gs_contact_waits[i].drop_label,sizeof gs_contact_waits[i].drop_label,"%s",s && s->label[0] ? s->label : "stage");
    }
}
static const char* gs_contact_leave(const GsContactPicture* a,const GsContactPicture* b)
{
    if(b->pass)return "dropped_through";
    if(b->knocked || b->dead)return "knocked";
    if(b->jump)return "jumped";
    if(b->x<a->surface[0].x0)return "ran_off_left";
    if(b->x>a->surface[0].x1)return "ran_off_right";
    return "lost_contact";
}
static int gs_contact_same(const GsContactSurface* a,const GsContactSurface* b)
{ return a->present==b->present && (!a->present || (a->line==b->line && a->owner==b->owner)); }
static void gs_contact_observe(int port,const GsContactPicture* b,int frame)
{
    GsContactPicture* a=&gs_contact_previous[port];
    int i;
    if(a->present && !b->present && !a->dead)gs_contact_emit("ko","fighter_removed",port,a,NULL,frame);
    if(b->present) {
        for(i=0;i<5;++i) if(!gs_contact_same(&a->surface[i],&b->surface[i])) {
            if(a->surface[i].present) {
                const char* release=i==0 ? "leave" : i==4 ? "ledge_release" : "wall_release";
                const char* reason=i==0 ? gs_contact_leave(a,b) : "";
                if(i!=3)gs_contact_emit(release,reason,port,b,&a->surface[i],frame);
                if(i==0 && b->pass && a->surface[i].passthrough)
                    gs_contact_emit("drop_through","dropped_through",port,b,&a->surface[i],frame);
            }
            if(b->surface[i].present)gs_contact_emit(i==0 ? "land" : i==4 ? "ledge_grab" :
                i==3 ? "ceiling_hit" : "wall_touch","",port,b,&b->surface[i],frame);
        }
        if(b->dead && !a->dead)gs_contact_emit("ko","",port,b,NULL,frame);
        if(b->rebirth && !a->rebirth)gs_contact_emit("respawn","",port,b,NULL,frame);
        if(gs_contact_state_changes && a->present && a->state!=b->state)
            gs_contact_emit("state_change","",port,b,NULL,frame);
    }
    *a=*b;
}
static int gs_contact_matches(const GsContactWait* w,const GsContactPicture* p)
{
    float dx=w->has_x ? p->x-w->x : 0,dy=w->has_y ? p->y-w->y : 0;
    if(!p->present || dx*dx+dy*dy>w->radius*w->radius)return 0;
    if(w->zone[0] && !gs_zone_has_name(w->port,w->zone))return 0;
    if(w->has_state && (w->state[0] ? strcmp(w->state,p->state_name)!=0 : w->state_id!=p->state))return 0;
    if(w->has_air && w->airborne!=p->airborne)return 0;
    if(w->has_on && (!p->surface[0].present || (w->label[0] ? strcmp(w->label,p->surface[0].label)!=0 : w->on!=p->surface[0].owner)))return 0;
    return 1;
}
static const char* gs_contact_name(const GsContactSurface* s)
{ return s->label[0] ? s->label : s->owner ? "unlabelled object" : "stage"; }
static void gs_contact_failure(const GsContactWait* w,const GsContactPicture* p,char* out,size_t cap)
{
    if(w->zone[0])snprintf(out,cap,"timeout %d frames waiting for zone \"%s\"; in %s",w->frames,w->zone,p->zones[0] ? p->zones:"outside every zone");
    else if(!p->present)snprintf(out,cap,"fighter was absent after %d frames",w->frames);
    else if(w->drop_label[0])snprintf(out,cap,"fell through \"%s\"",w->drop_label);
    else if(w->has_air && w->airborne && !w->saw_air)snprintf(out,cap,"never left the ground");
    else if(p->surface[1].present || p->surface[2].present) {
        int side=p->surface[1].present ? 1:2;
        snprintf(out,cap,"stopped at wall \"%s\" (%s side) at x=%.1f after %d frames",gs_contact_name(&p->surface[side]),side==1 ? "left":"right",p->x,w->frames);
    } else if(p->surface[0].present)snprintf(out,cap,"timeout %d frames; last on \"%s\" offset %.1f",w->frames,gs_contact_name(&p->surface[0]),p->surface[0].offset);
    else if(w->last_on.present)snprintf(out,cap,"timeout %d frames; last on \"%s\" offset %.1f",w->frames,gs_contact_name(&w->last_on),w->last_on.offset);
    else snprintf(out,cap,"timeout %d frames; last at x=%.1f y=%.1f in %s",w->frames,p->x,p->y,p->state_name);
}
static void gs_contact_wait_resolve(GsContactWait* w,int ok,const char* reason)
{
    w->done=1;w->ok=ok;snprintf(w->reason,sizeof w->reason,"%s",reason);
    if(w->console){
        gw_Console_Print(ok ? GS_GREEN:GS_RED,"wait %d: %s (%d frames)",w->handle,w->reason,w->frames);
        /* the console socket has replied by now: the log is where a tool reads the result */
        gw_log("script wait %d: %s: %s (%d frames)",w->handle,ok ? "ok":"failed",w->reason,w->frames);
    }
}
static void gs_contact_wait_seed(GsContactWait* w,const GsContactPicture* p)
{
    if(p->present && p->airborne)w->saw_air=1;
    if(p->surface[0].present)w->last_on=p->surface[0];
}
static void gs_contact_wait_tick(int port,const GsContactPicture* p)
{
    int i;
    for(i=0;i<GS_CONTACT_WAITS;++i) {
        GsContactWait* w=&gs_contact_waits[i];char reason[256];
        if(!w->used || w->done || w->port!=port)continue;
        ++w->frames;gs_contact_wait_seed(w,p);
        if(gs_contact_matches(w,p))gs_contact_wait_resolve(w,1,"conditions reached");
        else if(w->frames>=w->timeout){gs_contact_failure(w,p,reason,sizeof reason);gs_contact_wait_resolve(w,0,reason);}
    }
}
static void gs_contacts_reset(const char* reason)
{
    int i;
    gs_contact_count=gs_contact_head=0;gs_contact_dropped=0;++gs_contact_next;
    memset(gs_contact_previous,0,sizeof gs_contact_previous);
    for(i=0;i<GS_CONTACT_WAITS;++i)if(gs_contact_waits[i].used && !gs_contact_waits[i].done)
        gs_contact_wait_resolve(&gs_contact_waits[i],0,reason);
}
static void gs_contact_event_text(const GsContactEvent* e,char* out,size_t cap)
{
    const char* verb=e->kind;char reason[32];int i;
    if(!strcmp(e->kind,"land"))verb="landed on";
    else if(!strcmp(e->kind,"leave"))verb="left";
    else if(!strcmp(e->kind,"wall_touch"))verb=e->surface.kind==1 ? "touched the left wall":"touched the right wall";
    else if(!strcmp(e->kind,"wall_release"))verb=e->surface.kind==1 ? "released the left wall":"released the right wall";
    else if(!strcmp(e->kind,"ceiling_hit"))verb="hit the ceiling";
    else if(!strcmp(e->kind,"pass_up_through"))verb="passed upward through";
    else if(!strcmp(e->kind,"drop_through"))verb="dropped through";
    else if(!strcmp(e->kind,"ledge_grab"))verb="grabbed the ledge";
    else if(!strcmp(e->kind,"ledge_release"))verb="released the ledge";
    else if(!strcmp(e->kind,"state_change"))verb="changed action";
    else if(!strcmp(e->kind,"ko"))verb="was KO'd";
    else if(!strcmp(e->kind,"respawn"))verb="respawned";
    snprintf(reason,sizeof reason,"%s",e->reason);for(i=0;reason[i];++i)if(reason[i]=='_')reason[i]=' ';
    if(e->surface.present)snprintf(out,cap,"frame %d: P%d %s \"%s\" (line %d, offset %.1f), at x=%.1f y=%.1f in %s%s%s.",e->frame,e->port,verb,gs_contact_name(&e->surface),e->surface.line,e->surface.offset,e->fighter.x,e->fighter.y,e->fighter.state_name,reason[0] ? "; ":"",reason);
    else snprintf(out,cap,"frame %d: P%d %s at x=%.1f y=%.1f in %s.",e->frame,e->port,verb,e->fighter.x,e->fighter.y,e->fighter.state_name);
}
static void gs_contact_overlay_text(int port,const GsContactPicture* p,char* text,size_t cap)
{
    const GsContactSurface* s=&p->surface[0];
    if(s->present)snprintf(text,cap,"P%d %s  on %s  %.1f from %s (%.2f)  wall: %s  ceil: %s",port+1,p->state_name,gs_contact_name(s),s->offset,s->x0==s->x1 ? "bottom":"left",s->t,p->surface[1].present ? gs_contact_name(&p->surface[1]):p->surface[2].present ? gs_contact_name(&p->surface[2]):"-",p->surface[3].present ? gs_contact_name(&p->surface[3]):"-");
    else snprintf(text,cap,"P%d %s  on -  wall: %s  ceil: %s",port+1,p->state_name,p->surface[1].present ? gs_contact_name(&p->surface[1]):p->surface[2].present ? gs_contact_name(&p->surface[2]):"-",p->surface[3].present ? gs_contact_name(&p->surface[3]):"-");
    {size_t n=strlen(text);if(n<cap)snprintf(text+n,cap-n,"  zones: %s",p->zones);}
}
