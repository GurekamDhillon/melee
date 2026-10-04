static int test_script_deadline(void) {
    char out[512];int old=gs.cur,rc=0;
    t_exec("deadline_hits=0; function on_deadline(name,frames) deadline_hits=deadline_hits+1; deadline_name=name; deadline_frames=frames end; gd.deadline('staging maze',2)",out,sizeof out);
    gs_deadline_frame();
    t_exec("= deadline_hits",out,sizeof out);rc|=strstr(out,"0")==NULL;
    gs_deadline_frame();gs_deadline_frame();
    t_exec("= deadline_hits==1 and deadline_name=='staging maze' and deadline_frames==2",out,sizeof out);
    rc|=strstr(out,"true")==NULL;
    gs.cur=gs.console;
    /* Owner isolation: identical names in a different slot are distinct. */
    gs_deadline_release(gs.console);
    t_exec("= gd.deadline('cancel',1) and gd.deadline_done('cancel')",out,sizeof out);rc|=strstr(out,"true")==NULL;
    gs_deadline_frame();rc|=gs_deadline_active!=0;
    t_exec("gd.deadline('repeat',3)",out,sizeof out);gs_deadline_frame();
    t_exec("= gd.deadline('repeat',99)",out,sizeof out);rc|=strstr(out,"false")==NULL;
    gs_deadline_frame();gs_deadline_frame();
    rc|=gs_deadline_expired_count!=1;
    gs_deadline_release(gs.console);gs.cur=old;
    /* A script catching denied gameplay calls still leaves refusal evidence,
     * even with profiling off. Clone only the test environment, never ownership. */
    if(gs.n<GS_MAX_SCRIPTS) {
        int owner=gs.n++,i,seen=0;GsScript saved=gs.s[owner];
        gs.s[owner]=gs.s[gs.console];gs.s[owner].gameplay=0;
        snprintf(gs.s[owner].id,sizeof gs.s[owner].id,"deadline-refusal-test");
        t_exec("function deadline_owner_open() gd.deadline('shared-name',10) end; gd.deadline('shared-name',10)",out,sizeof out);
        if(gs_get_hook(gs.console,"deadline_owner_open"))gs_pcall(owner,0,0,"deadline_owner_open");else rc=1;
        rc|=gs_deadline_active!=2;
        t_exec("gd.deadline_done('shared-name')",out,sizeof out);rc|=gs_deadline_active!=1;
        gs_deadline_release(owner);rc|=gs_deadline_active!=0;
        t_exec("function deadline_caught() pcall(gd.set_percent,1,10) end",out,sizeof out);
        if(gs_get_hook(gs.console,"deadline_caught"))gs_pcall(owner,0,0,"caught_refusal");else rc=1;
        for(i=0;i<128;++i)if(gs_callback_fail[i].used && gs_callback_fail[i].owner==owner && gs_callback_fail[i].count==1)seen=1;
        rc|=!seen;
        gs_callback_release(owner);gs.s[owner]=saved;gs.n--;
        t_exec("deadline_caught=nil; deadline_owner_open=nil",out,sizeof out);
    } else rc=1;
    t_exec("on_deadline=nil; deadline_hits=nil; deadline_name=nil; deadline_frames=nil",out,sizeof out);
    if(rc)gw_test_fail("deadline one-shot/cancel/repeated-open contract failed: %s",out);
    return rc;
}
