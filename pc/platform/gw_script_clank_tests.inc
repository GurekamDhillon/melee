/* Real Lua dispatcher regression: specifically guards the event-name/id match. */
static int test_script_clank_event(void)
{
    char out[1024];int wanted,clanks;
    gs_init();
    gs_update_want_events();
    wanted=gs.want_events;clanks=gs_want_clanks;
    /* Empty observation ids must be safe even without a match/object table. */
    if(gw_ScriptGame_ClankValue(0,0,0) || gw_ScriptGame_ClankValue(0,1,1)) {
        gw_test_fail("empty clank observation returned a value");return 1;
    }
    t_exec("saved_clank_hook=on_clank; clank_seen=nil; function on_clank(e) clank_seen=e end",out,sizeof out);
    gs_update_want_events();
    gs.nev=0;
    gw_Script_Clank(0,1,0,-1,gs_camera_bits(3),gs_camera_bits(6),gs_camera_bits(8),
        gs_camera_bits(12),gs_camera_bits(13),3,0,0);
    gs_dispatch_events();
    t_exec("= clank_seen and clank_seen.port_a==1 and clank_seen.port_b==2 and "
        "clank_seen.x==3 and clank_seen.y==6 and clank_seen.z==8 and "
        "clank_seen.damage_a==12 and clank_seen.damage_b==13 and "
        "clank_seen.cancel_a and clank_seen.cancel_b and clank_seen.hitlag_a==0",out,sizeof out);
    t_exec("on_clank=saved_clank_hook; saved_clank_hook=nil; clank_seen=nil",NULL,0);
    /* Recompute from restored hooks; never leave the fixture subscriber armed. */
    gs_update_want_events();
    if(gs.want_events!=wanted || gs_want_clanks!=clanks) {
        gw_test_fail("clank hook cleanup changed event subscription");return 1;
    }
    t_shader_trim(out);
    if(strcmp(out,"true")){gw_test_fail("on_clank producer/dispatch id mismatch or payload loss: %s",out);return 1;}
    return 0;
}
static int test_script_presentation_timer(void)
{
    GwPresentationTimer t={0};char out[512];
    t_exec("= type(gd.hitstop), type(gd.hitstop_cancel), gd.clank_event",out,sizeof out);
    t_shader_trim(out);
    if(strcmp(out,"function\nfunction\ntrue")){gw_test_fail("presentation APIs missing: %s",out);return 1;}
    gw_presentation_start(&t,7,100,71);
    if(!gw_presentation_live(&t,100) || gw_presentation_live(&t,1284) ||
       gw_presentation_cancel(&t,8) || !gw_presentation_cancel(&t,7)) {
        gw_test_fail("presentation deadline/ownership failed");return 1;
    }
    return 0;
}
