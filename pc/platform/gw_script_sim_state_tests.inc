/* Safe fixture tests; no fighter or scene required. The actual snapshot/logic
 * ordering is additionally checked by gd.rewind_test in the owner's LAB. */
static int test_script_sim_state(void) {
    static const unsigned char payload[]={0,1,255,'x'};
    char out[512]; int rc;
    gs_sim_input=payload; gs_sim_input_size=sizeof payload;
    if(!gw_ScriptGame_SimCommit(INT_MAX-1,sizeof payload) ||
       gw_ScriptGame_SimSize(INT_MAX-1)!=(int)sizeof payload ||
       gw_ScriptGame_SimByte(INT_MAX-1,2)!=255 ||
       gw_ScriptGame_SimSize(INT_MAX-2)!=-1 ||
       gw_ScriptGame_SimCommit(INT_MAX-1,GS_SIM_BYTES+1)) {
        gs_sim_input=NULL; gs_sim_input_size=0; gw_ScriptGame_SimRelease(INT_MAX-1);
        gw_test_fail("script simulation checkpoint bounds or generation isolation"); return 1;
    }
    gs_sim_input_size=1;
    gw_ScriptGame_SimCommit(INT_MAX-1,1);
    if(gw_ScriptGame_SimSize(INT_MAX-1)!=1 || gw_ScriptGame_SimByte(INT_MAX-1,2)!=0) {
        gs_sim_input=NULL; gs_sim_input_size=0; gw_ScriptGame_SimRelease(INT_MAX-1);
        gw_test_fail("script simulation checkpoint replacement"); return 1;
    }
    gs_sim_input=NULL; gs_sim_input_size=0; gw_ScriptGame_SimRelease(INT_MAX-1);
    rc=t_exec("assert(gd.sim_supported and type(gd.sim_read)=='function'); "
              "assert(type(gd.sim_replaying())=='boolean' and type(gd.sim_clear)=='function'); "
              "local ok,e=pcall(gd.sim_commit,'test',{}); "
              "assert(not ok and tostring(e):find('on_frame',1,true))",out,sizeof out);
    if(rc) { gw_test_fail("script simulation API frame guard: %s",out); return 1; }
    return 0;
}
