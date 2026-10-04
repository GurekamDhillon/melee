extern int gw_ScriptGame_FighterModTest(void);
static int test_script_fighter_mod(void) {
    char out[2048];
    int saved_active=gs.match_active, rc;
    if (gw_ScriptGame_FighterModTest()) {
        gw_test_fail("fighter modifier attrs, ownership, replacement or respawn fixture failed"); return 1;
    }
    gs.match_active=1;
    rc=t_exec("assert(type(gd.fighter_mod)=='function'); "
               "assert(not pcall(gd.fighter_mod,0,{})); "
               "assert(not pcall(gd.fighter_mod,7,{})); "
               "local function reject(v,s) local ok,e=pcall(gd.fighter_mod,1,v); "
               "assert(not ok and tostring(e):find(s,1,true),tostring(e)) end; "
               "reject({run_speed=0/0},'must be finite'); "
               "reject({damage_taken=math.huge},'must be finite'); "
               "reject({typo=1},'unknown fighter modifier'); "
               "assert(gd.fighter_mod(1,{run_speed=99,damage_taken=-1})); "
               "local q=gd.fighter_mod(1); assert(q and q.run_speed==4 and math.abs(q.damage_taken-.1)<.00001); "
               "assert(gd.fighter_mod(1,nil)); assert(gd.fighter_mod(1)==nil)",out,sizeof out);
    gs.match_active=saved_active;
    if (rc) {
        gw_test_fail("fighter modifier Lua validation: %s",out); return 1;
    }
    return 0;
}
