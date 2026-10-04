/* Registered native suite fixture: not executed by syntax-only validation. */
extern int gw_ScriptGame_OnePTemplateTest(void);
static int test_script_1p(void) {
    unsigned char saved[sizeof gs_1p];int i,fail=0;char out[1024];
    if(gw_ScriptGame_OnePTemplateTest())fail=1;
    memcpy(saved,&gs_1p,sizeof saved);memset(&gs_1p,0,sizeof gs_1p);
    if(!gs_1p_can_start(0x2f)||gs_1p_can_start(3)||gs_1p_can_start(4))fail=1;
    gs_1p.scripted=1;if(gs_1p_can_start(0x2f))fail=1;gs_1p.scripted=0;
    gs_1p.mode=3;if(gs_1p_can_start(0x2f))fail=1;gs_1p.mode=0;
    if(gs_1p_hold_accept(1800,1))fail=1; /* no safe callback */
    gs_1p.barrier=gs_1p.dispatch=1;
    if(gs_1p_hold_accept(0,1)||!gs_1p_hold_accept(9999,1)||gs_1p.remaining!=1800)fail=1;
    if(gs_1p_hold_accept(1800,1)||gs_1p_hold_release(2))fail=1;
    for(i=0;i<1799;++i)if(gs_1p_hold_step(1))fail=1;
    if(!gs_1p.held||!gs_1p_hold_step(1)||gs_1p.held)fail=1;
    if(gs_1p_hold_accept(20,1))fail=1; /* deadline never renews */
    gs_1p.used_hold=0;
    if(!gs_1p_hold_accept(20,3)||!gs_1p_hold_step(0)||gs_1p.held)fail=1;
    gs_1p.used_hold=0;
    if(!gs_1p_hold_accept(20,3)||!gs_1p_hold_release(3)||gs_1p_hold_release(3))fail=1;
    memset(&gs_1p,0,sizeof gs_1p);
    if(t_exec("assert(type(gd.mode_1p)=='function'); assert(gd.mode_1p()==nil); "
       "assert(type(gd.end_1p)=='function'); assert(not gd.start_1p{}); "
       "assert(not gd.hold_1p(10)); assert(not gd.release_1p()); assert(not gd.loop_1p(true))",
       out,sizeof out))fail=1;
    memcpy(&gs_1p,saved,sizeof gs_1p);
    if(fail){gw_test_fail("retail 1P bounds, ownership, release or API fixture");return 1;}
    return 0;
}
