/* Queue-time envelope foundation. Stream completion/tag declarations are the
 * later observation step; no Lua invocation is introduced during collision. */
extern int gw_ScriptGame_EntityEpoch(void);
extern int gw_ScriptGame_EntityAddressField(int,int,int);
static unsigned gs_spine_event_epoch,gs_spine_event_sequence;
static int gs_spine_event_frame=-1;
/* FramePost retires the simulation gate before its early callbacks, but the
 * observation frame is still the one just simulated until gs.frame advances. */
static int gs_spine_post_pending;
static void gs_spine_hit_context_actor(GsEvent* e)
{
    /* Legacy context replaces the physical actor with throw credit, without
     * carrying the owner's sub identity. Do not guess a live sibling. */
    e->spine.actor[0]=0;
}
static void gs_spine_actor(char out[80],int port,int sub)
{
    GsEntityRef r;out[0]=0;
    if(port>=0&&port<6&&gs_ref_current(1,(unsigned)(port*2+(sub!=0)),&r))gs_ref_text(&r,out);
}
static void gs_spine_address(char out[80],int kind,int address)
{
    GsEntityRef r;out[0]=0;
    if(!gw_ScriptGame_EntityAddressField(kind,address,0))return;
    r.kind=(unsigned)kind;r.id=(unsigned)gw_ScriptGame_EntityAddressField(kind,address,6);
    r.epoch=(unsigned)gw_ScriptGame_EntityAddressField(kind,address,1);
    r.generation=(unsigned)gw_ScriptGame_EntityAddressField(kind,address,2);
    r.definition=(unsigned)gw_ScriptGame_EntityAddressField(kind,address,3);gs_ref_text(&r,out);
}
static void gs_spine_event_capture(GsEvent* e)
{
    unsigned epoch=(unsigned)gw_ScriptGame_EntityEpoch();int frame=gs.frame+(gs_in_frame!=0||gs_spine_post_pending);
    memset(&e->spine,0,sizeof e->spine);
    if(gs_spine_event_epoch!=epoch||gs_spine_event_frame!=frame){gs_spine_event_epoch=epoch;gs_spine_event_frame=frame;gs_spine_event_sequence=0;}
    e->spine.frame=frame;e->spine.epoch=epoch;e->spine.sequence=++gs_spine_event_sequence;
    e->spine.phase=e->what==17?3:e->what==LAB_EV_HIT||e->what==11?2:
        (e->what>=5&&e->what<=9)||e->what==12||e->what==13||e->what==14||e->what==15?4:1;
    /* The legacy hit actor may be an item owner or a thrown body; its sub bit
     * cannot establish credited identity, including a full-queue last slot.
     * Explicit source attribution belongs to the complete observation step. */
    if(e->what==LAB_EV_HIT){gs_spine_actor(e->spine.target,e->b,e->c&LAB_HIT_VICTIM_SUB);}
    else if(e->what==LAB_EV_ACTION||e->what==LAB_EV_LAND||e->what==16)gs_spine_actor(e->spine.actor,e->a,e->d);
    else if(e->what==LAB_EV_HITLAG||e->what==15)gs_spine_actor(e->spine.actor,e->a,e->c);
    else if(e->what==14){gs_spine_actor(e->spine.actor,e->a,0);gs_spine_actor(e->spine.target,e->b,0);}
    else if(e->what==17){int entity=e->a-1;gs_spine_actor(e->spine.actor,entity%6,entity/6);}
}
static void gs_spine_event_push(lua_State* L,const GsEvent* e,const char* hook)
{
    char id[96];static const char* const phases[]={"unknown","action","collision","reaction","lifecycle","world","run"};
    snprintf(id,sizeof id,"v1:%u:%d:%u",e->spine.epoch,e->spine.frame,e->spine.sequence);
    lua_createtable(L,0,12);gs_setint(L,"version",1);gs_setstr(L,"kind",hook);gs_setstr(L,"id",id);
    gs_setint(L,"frame",e->spine.frame);gs_setint(L,"sequence",e->spine.sequence);
    gs_setstr(L,"phase",phases[e->spine.phase>=1&&e->spine.phase<=6?e->spine.phase:0]);gs_setstr(L,"delivery","post_frame");
    if(e->spine.cause){snprintf(id,sizeof id,"v1:%u:%d:%u",e->spine.epoch,e->spine.frame,e->spine.cause);gs_setstr(L,"cause",id);}
    if(e->spine.actor[0])gs_setstr(L,"actor_ref",e->spine.actor);
    if(e->spine.target[0])gs_setstr(L,"target_ref",e->spine.target);
    /* Exact legacy scalars, not guessed typed hit/source tags. */
    lua_createtable(L,0,5);gs_setint(L,"code",e->what);gs_setint(L,"a",e->a);gs_setint(L,"b",e->b);gs_setint(L,"c",e->c);gs_setint(L,"d",e->d);lua_setfield(L,-2,"legacy");
}

/* Signals that never had a queued GsEvent row (zone enter/exit/none/some, the 1P lifecycle) join the
 * same envelope: one epoch/frame/sequence clock, one on_event table, delivered right where their legacy
 * callback runs and before it. The row is built once per signal, so every listening script sees the
 * same id. phase 5 = world (zones), 6 = run (1P stage and game flow). */
static void gs_spine_signal_begin(GsEvent* e,int phase,int port,int a,int b)
{
    memset(e,0,sizeof *e);e->a=a;e->b=b;
    gs_spine_event_capture(e);e->spine.phase=phase;
    if(port>=0)gs_spine_actor(e->spine.actor,port,0);
}
static int gs_spine_signal_deliver(int script,const GsEvent* e,const char* kind)
{
    if(!gs_get_hook(script,"on_event"))return 0;
    gs_spine_event_push(gs.L,e,kind);gs_pcall(script,1,0,"on_event");return 1;
}
