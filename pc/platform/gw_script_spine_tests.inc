/* Registered real queue/dispatch compatibility check. Requires an EXE suite;
 * syntax/standalone fixtures are not evidence this test has run in the game. */
extern int gw_ScriptGame_ActionCategory(int);
static int test_script_spine_events(void)
{
    char out[2048];int fail=0,motion;unsigned seq=gs_spine_event_sequence,epoch=gs_spine_event_epoch;
    int frame=gs_spine_event_frame,before=gs.nev;
    if(gw_Snap_Resimulating())return 0;
    if(t_exec("spine_events={};spine_old=0;function on_event(e) assert(e.version==1,'ver');assert(type(e.id)=='string','id');assert(e.delivery=='post_frame','delivery');spine_events[#spine_events+1]=e end;function on_action_change(p,a,b,s) assert(type(p)=='number','p '..type(p));assert(type(s)=='boolean','s '..type(s));spine_old=spine_old+1 end",out,sizeof out))return 1;
    gs_update_want_events();
    for(motion=0;motion<400;++motion)if(gw_ScriptGame_ActionCategory(motion))break;
    if(motion==400){fail=1;snprintf(out,sizeof out,"no action category motion");}
    else {
        gw_Script_GameEvent(LAB_EV_ACTION,0,-1,motion,0);
        if(gs.nev!=before+2){fail=1;snprintf(out,sizeof out,"queued %d rows, wanted 2 (want_events=%d)",gs.nev-before,gs.want_events);}
        else if(gs.ev[before+1].spine.cause!=gs.ev[before].spine.sequence){fail=1;snprintf(out,sizeof out,"cause link wrong");}
        gs_dispatch_events();
        if(t_exec("assert(spine_old==1,'old '..spine_old);assert(#spine_events==2,'n '..#spine_events);local a,b=spine_events[1],spine_events[2];assert(b.cause==a.id and b.sequence==a.sequence+1,tostring(a.id)..' '..tostring(b.id)..' '..tostring(b.cause)..' '..a.sequence..' '..b.sequence..' '..a.kind..' '..b.kind)",out,sizeof out))fail=1;
        {   /* zone and 1P signals join the same envelope, in order, with the shared clock */
            GsEvent env;int i,heard=0;
            gs_spine_signal_begin(&env,5,-1,3,9);
            for(i=0;i<gs.n;++i)heard+=gs_spine_signal_deliver(i,&env,"on_zone_enter");
            gs_spine_signal_begin(&env,6,-1,-1,0);
            for(i=0;i<gs.n;++i)heard+=gs_spine_signal_deliver(i,&env,"on_1p_stage_clear");
            if(heard<2){fail=1;snprintf(out,sizeof out,"signal envelope reached %d listeners",heard);}
            else if(t_exec("assert(#spine_events==4,'n '..#spine_events);local z,o=spine_events[3],spine_events[4];assert(z.kind=='on_zone_enter' and z.phase=='world' and z.legacy.b==9,'zone');assert(o.kind=='on_1p_stage_clear' and o.phase=='run' and o.sequence==z.sequence+1 and o.frame==z.frame and o.delivery=='post_frame','1p')",out,sizeof out))fail=1;
        }
        before=gs.nev;gw_Snap_LabResim(1);
        /* The snapshot engine is off in a headless suite, so the resimulation gate cannot be raised
           here; only assert when it can, and never leave rows behind for the next test. */
        if(gw_Snap_Resimulating()){gw_Script_GameEvent(LAB_EV_ACTION,0,-1,motion,0);
            if(gs.nev!=before){fail=1;snprintf(out,sizeof out,"resimulated event was queued");}}
        gw_Snap_LabResim(0);gs.nev=before;
    }
    t_exec("on_event,on_action_change,spine_events,spine_old=nil,nil,nil,nil",out,sizeof out);gs_update_want_events();
    gs_spine_event_sequence=seq;gs_spine_event_epoch=epoch;gs_spine_event_frame=frame;
    if(fail)gw_test_fail("spine queue/envelope/legacy/resimulation fixture: %s",out);
    return fail;
}
