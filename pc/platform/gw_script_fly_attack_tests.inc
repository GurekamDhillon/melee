/* Boundary checks use real Lua calls and the same debug command parser. */
extern int gw_GenoFly_CursorTest(void);
static int test_script_fly_attack(void) {
    char out[2048];
    if (gw_GenoFly_CursorTest()) {
        gw_test_fail("fly cursor moving center, repeat victim history or offline fallback failed"); return 1;
    }
    if (t_exec("assert(type(gd.fly_target)=='function' and type(gd.fly_attack)=='function'); "
               "assert(type(gd.fly_clear)=='function' and type(gd.fly_state)=='function'); "
               "assert(not pcall(gd.fly_target,0,0,0)); "
               "assert(not pcall(gd.fly_target,1,0/0,0)); "
               "assert(not pcall(gd.fly_target,1,0,math.huge)); "
               "assert(not pcall(gd.fly_target,1,49001,0)); "
               "assert(not pcall(gd.fly_attack,7,true)); "
               "assert(not pcall(gd.fly_attack,1,'on')); "
               "assert(not pcall(gd.fly_attack,1,true,31,6)); "
               "assert(not pcall(gd.fly_attack,1,true,3,0/0)); "
               "assert(not pcall(gd.fly_attack,1,true,3,0)); "
               "assert(not pcall(gd.fly_attack,1,true,3,31)); "
               "assert(not pcall(gd.fly_attack,1,true,{damage=31})); "
               "assert(not pcall(gd.fly_attack,1,true,{radius=0})); "
               "assert(not pcall(gd.fly_attack,1,true,{active=0})); "
               "assert(not pcall(gd.fly_attack,1,true,{active=601})); "
               "assert(not pcall(gd.fly_attack,1,true,{gap=-1})); "
               "assert(not pcall(gd.fly_attack,1,true,{kb=0})); "
               "assert(not pcall(gd.fly_attack,1,true,{kbg=2001})); "
               "assert(not pcall(gd.fly_attack,1,true,{angle=362})); "
               "assert(not pcall(gd.fly_attack,1,true,{active=2.5})); "
               "assert(not pcall(gd.fly_attack,1,true,{every_frame=1})); "
               "assert(not pcall(gd.fly_attack,7,true,{damage=3})); "
               "local s=gd.fly_state(1); assert(not s.attacking and not s.targeting and s.pulses==0)",
               out, sizeof out) != 0) {
        gw_test_fail("fly cursor boundary validation: %s", out); return 1;
    }
    if (gs_fly_console("fly", "target 1 nan 0") != -1 ||
        gs_fly_console("fly", "target 1 0 inf") != -1 ||
        gs_fly_console("fly", "attack 1 on 31 6") != -1 ||
        gs_fly_console("fly", "attack 1 on 3 0") != -1 ||
        gs_fly_console("fly", "attack 1 on 3 6 extra") != -1) {
        gw_test_fail("fly cursor console accepted invalid coordinates or hitbox parameters"); return 1;
    }
    return 0;
}
