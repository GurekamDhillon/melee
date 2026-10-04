/* Actual native queue/dispatch fixture: snapshots must not be read at deferred delivery. */
extern void gw_Script_HitOriginalElement(int);
static int test_script_gameplay_events(void) {
    char out[2048];
    int rc=0;
    if(t_exec("em1={}; function on_hit(a,v,e) assert(a==1 and v==2 and e.context_valid and e.move_tag=='aerial' and e.element_tag=='fire' and e.attacker_grounded==false and e.victim_grounded==true and e.attacker_damage==20 and e.victim_damage==40 and e.element_tag=='fire' and e.original_element_tag=='ice' and e.element_converted==true); em1[#em1+1]='hit' end; "
              "function on_ko(a,v) assert(a==1 and v==2); em1[#em1+1]='ko' end; "
              "function on_stock_lost(p,n) assert(p==2 and n==1); em1[#em1+1]='stock' end; "
              "function on_jump(p,m,s) assert(p==1 and m==25 and s==false); em1[#em1+1]='jump' end; "
              "function on_perfect_shield(p,m,s) assert(p==2 and s==true); em1[#em1+1]='perfect' end",out,sizeof out)) return 1;
    gs_update_want_events();
    gw_Script_GameEvent(LAB_EV_HIT,0,1,0xff,gs_fbits(12));
    gw_Script_HitContext(0,1,1,65,5,0,1,gs_fbits(20),gs_fbits(40));
    gw_Script_HitOriginalElement(5); /* a hit rule converted Ice into Fire at creation */
    gw_Script_GameEvent(14,0,1,0,0); gw_Script_GameEvent(15,1,1,0,0);
    gw_Script_GameEvent(16,0,25,1,0); gw_Script_GameEvent(16,1,182,8,1);
    if(t_exec("assert(#em1==0)",out,sizeof out)) rc=1;
    gs_dispatch_events();
    if(t_exec("assert(table.concat(em1,',')=='hit,ko,stock,jump,perfect')",out,sizeof out)) rc=1;
    t_exec("on_hit,on_ko,on_stock_lost,on_jump,on_perfect_shield,em1=nil,nil,nil,nil,nil,nil",out,sizeof out);
    gs_update_want_events();
    if(rc) gw_test_fail("gameplay event capture/order fixture: %s",out);
    return rc;
}
