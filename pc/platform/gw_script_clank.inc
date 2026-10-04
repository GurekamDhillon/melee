/* Single event id shared by producer and dispatcher, with typed payload.
 * Lua runs at FramePost only; no pointer escapes into Lua or game-state write. */
extern int gw_ScriptGame_ClankValue(int address,int is_item,int rebound);
int gw_Script_ClankWanted(void)
{ return gs.L!=NULL && gs.want_events && gs_want_clanks && !gw_Snap_Resimulating(); }
void gw_Script_Clank(int a,int b,int item,int kind,int x,int y,int z,
                     int da,int db,int flags,int ea,int eb)
{
    GsEvent* e;GsClank* c;int i;
    if(!gw_Script_ClankWanted())return;
    /* Many overlapping hitboxes can resolve the same clash. First contact wins
     * per live entity pair per frame; don't replay the whole effect per capsule. */
    for(i=0;i<gs.nev;++i)if(gs.ev[i].what==GS_EV_CLANK) {
        GsClank* old=&gs.ev[i].clank;
        if((old->entity_a==ea && old->entity_b==eb) ||
           (old->entity_a==eb && old->entity_b==ea))return;
    }
    if(gs.nev>=GS_MAX_EVENTS){++gs.ev_dropped;return;}
    e=&gs.ev[gs.nev++];memset(e,0,sizeof *e);e->what=GS_EV_CLANK;c=&e->clank;
    c->port_a=a+1;c->port_b=b<0?0:b+1;c->item=item;c->item_kind=kind;
    c->entity_a=ea;c->entity_b=eb;c->cancel_a=flags&1;c->cancel_b=(flags>>1)&1;
    c->x=gs_camera_float(x);c->y=gs_camera_float(y);c->z=gs_camera_float(z);
    c->damage_a=gs_camera_float(da);c->damage_b=gs_camera_float(db);
#ifdef GS_SPINE_EVENTS
    gs_spine_event_capture(e);
    gs_spine_address(e->spine.actor,1,ea);
    gs_spine_address(e->spine.target,item?2:1,eb);
#endif
}
/* Snapshot ALL queued clashes before ANY Lua handler (even a different event)
 * can issue an immediate gd.hit or another gameplay write. Every listener sees
 * the same final collision result, independent of listener/event ordering. */
static void gs_clank_snapshot(GsClank* c)
{
    if(c->finalized)return;
    c->finalized=1;
    c->rebound_a=gw_ScriptGame_ClankValue(c->entity_a,0,1);
    c->rebound_b=gw_ScriptGame_ClankValue(c->entity_b,c->item!=0,1);
    c->hitlag_a=gs_camera_float(gw_ScriptGame_ClankValue(c->entity_a,0,0));
    c->hitlag_b=gs_camera_float(gw_ScriptGame_ClankValue(c->entity_b,c->item!=0,0));
}
static void gs_clanks_snapshot_pending(void)
{
    int i;for(i=0;i<gs.nev;++i)if(gs.ev[i].what==GS_EV_CLANK)gs_clank_snapshot(&gs.ev[i].clank);
}
static void gs_clank_push(lua_State* L,const GsClank* c)
{
    int ra=c->rebound_a,rb=c->rebound_b;
    lua_createtable(L,0,16);
    gs_setint(L,"port_a",c->port_a);
    if(c->port_b)gs_setint(L,"port_b",c->port_b);
    if(c->item){gs_setnum(L,"item",(double)(uint32_t)c->item);gs_setint(L,"item_kind",c->item_kind);}
    gs_setnum(L,"x",c->x);gs_setnum(L,"y",c->y);gs_setnum(L,"z",c->z);
    gs_setnum(L,"damage_a",c->damage_a);gs_setnum(L,"damage_b",c->damage_b);
    gs_setbool(L,"cancel_a",c->cancel_a);gs_setbool(L,"cancel_b",c->cancel_b);
    gs_setbool(L,"rebound_a",ra);gs_setbool(L,"rebound_b",rb);gs_setbool(L,"rebound",ra||rb);
    gs_setnum(L,"hitlag_a",c->hitlag_a);
    gs_setnum(L,"hitlag_b",c->hitlag_b);
}
