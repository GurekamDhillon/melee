extern int gw_ScriptGame_CapsTargetTest(void);
extern int gw_ScriptGame_CapsMovementTest(void);
extern int gw_ScriptGame_CapsEffectsTest(void);
static int test_script_fighter_caps(void)
{
    char out[2048]; int saved=gs.match_active,rc;
    if(gw_ScriptGame_CapsTargetTest() || gw_ScriptGame_CapsMovementTest() || gw_ScriptGame_CapsEffectsTest()) {
        gw_test_fail("fighter capabilities game-side fixtures failed"); return 1;
    }
    gs.match_active=1;
    rc=t_exec("local function no(f,...) assert(not pcall(f,...)) end; "
        "assert(type(gd.fighter_caps)=='function' and type(gd.fighter_timed_status)=='function'); "
        "no(gd.fighter_caps,0,{});no(gd.fighter_caps,13,{}); "
        "no(gd.fighter_caps,1,{air_jumps=9});no(gd.fighter_caps,1,{air_jumps=-1}); "
        "no(gd.fighter_caps,1,{shield=1});no(gd.fighter_caps,1,{typo=true}); "
        "no(gd.fighter_effect,1,'size',0,1);no(gd.fighter_effect,1,'size',0/0,1); "
        "no(gd.fighter_effect,1,'metal',1,3601);no(gd.fighter_effect,1,'typo',1,1); "
        "no(gd.fighter_armour,1,{damage=math.huge});no(gd.fighter_armour,1,{typo=1}); "
        "no(gd.give_item,1,'typo');no(gd.give_item,1,-2); "
        "no(gd.opponents_in_radius,1,-1);no(gd.opponents_in_radius,1,0/0); "
        "no(gd.nearest_opponent,1,13);no(gd.opponents_in_radius,1,1,0); "
        "assert(type(gd.fighter_interrupt)=='function'); "
        "no(gd.fighter_interrupt,0,{});no(gd.fighter_interrupt,13,{});no(gd.fighter_interrupt,1,{frames=3601}); "
        "no(gd.fighter_interrupt,1,{frames=-1});no(gd.fighter_interrupt,1,{exits={'typo'}});no(gd.fighter_interrupt,1,{typo=1}); "
        "no(gd.fighter_interrupt,1,{guard=1});no(gd.fighter_interrupt,1,'x'); "
        "no(gd.fighter_timed_status,1,0,1,1);no(gd.fighter_timed_status,1,5,1,1); "
        "no(gd.fighter_timed_status,1,1,0/0,1);no(gd.fighter_timed_status,1,1,1,-1)",out,sizeof out);
    gs.match_active=saved;
    if(rc) {gw_test_fail("fighter capabilities Lua validation: %s",out);return 1;}
    gs.match_active=0;
    rc=t_exec("local function inactive(f,...) local ok,e=pcall(f,...); "
        "assert(not ok and tostring(e):find('active match',1,true),tostring(e)) end; "
        "inactive(gd.fighter_caps,1,{});inactive(gd.fighter_armour,1,nil);inactive(gd.fighter_interrupt,1,{frames=5}); "
        "inactive(gd.fighter_effect,1,'metal',1,1);inactive(gd.give_item,1,11); "
        "inactive(gd.fighter_timed_status,1,1,1,1)",out,sizeof out);
    gs.match_active=saved;
    if(rc) {gw_test_fail("fighter capabilities inactive match policy: %s",out);return 1;}
    return 0;
}
