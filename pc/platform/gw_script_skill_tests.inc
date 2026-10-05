/* Skill events, the crit event and the crit generator, earned-afterimage rules. The queue/dispatch half is a real
 * fixture of the native path (gw_Script_SkillEvent / gw_Script_CritEvent -> gs_dispatch_events -> Lua); the
 * observers themselves are proven in the game (they read live fighters). */
extern int gw_ScriptGame_CritDraws(int seed,int n,int chance_bits);
static int test_script_skill(void) {
    char out[2048];
    int rc=0,a,b;
    if(t_exec("sk={}; function on_skill(i) sk[#sk+1]='any:'..i.kind end; "
              "function on_lcancel(i) assert(i.entity==2 and i.port==2 and i.aerial=='fair' and i.hit==true and i.trigger_age==3 and i.lag==22 and i.lag_cancelled==11 and i.frame==0); sk[#sk+1]='lc' end; "
              "function on_wavedash(i) assert(math.abs(i.speed+2.5)<1e-6 and math.abs(i.speed_abs-2.5)<1e-6 and i.frames_after_jump==0 and i.dodge_frames==2); sk[#sk+1]='wd' end; "
              "function on_tech(i) assert(i.direction=='toward' and i.roll==true and i.wall_jump==false); sk[#sk+1]='tech' end; "
              "function on_combo(i) assert(i.entity==1 and i.count==3 and i.attacker==2 and math.abs(i.damage-14)<1e-6); sk[#sk+1]='combo' end; "
              "function on_crit(i) assert(i.attacker==1 and i.victim==2 and i.move_tag=='aerial' and i.forced==true and i.projectile==false and "
              "math.abs(i.base_damage-10)<1e-6 and math.abs(i.final_damage-25)<1e-6 and math.abs(i.added_damage-15)<1e-6 and math.abs(i.strength-0.75)<1e-6 and "
              "math.abs(i.x-1)<1e-6 and math.abs(i.y-2)<1e-6 and math.abs(i.z-3)<1e-6 and math.abs(i.hitlag_frames-8)<1e-6 and math.abs(i.multiplier-2.5)<1e-6 and math.abs(i.launch-1.25)<1e-6); sk[#sk+1]='crit' end",out,sizeof out)) return 1;
    gs_update_want_events();
    gw_Script_SkillEvent(1,2,1,1,3,gs_fbits(22),gs_fbits(11));          /* lcancel: fair, hit, trigger age 3 */
    gw_Script_SkillEvent(5,2,0,2,0,gs_fbits(-2.5f),gs_fbits(2.5f));     /* wavedash */
    gw_Script_SkillEvent(10,2,1,1,0,0,0);                               /* tech: toward, roll */
    gw_Script_SkillEvent(20,1,3,2,0,gs_fbits(14),gs_fbits(4));          /* combo: third hit by entity 2 */
    gw_Script_CritEvent(1,2,5|0x100,gs_fbits(1.25f),gs_fbits(10),gs_fbits(25),gs_fbits(0.75f),gs_fbits(1),gs_fbits(2),gs_fbits(3),gs_fbits(8),gs_fbits(2.5f));
    if(t_exec("assert(#sk==0)",out,sizeof out)) rc=1;
    gs_dispatch_events();
    if(t_exec("assert(table.concat(sk,',')=='lc,any:lcancel,wd,any:wavedash,tech,any:tech,combo,any:combo,crit')",out,sizeof out)) rc=1;
    /* a bad kind or entity is refused at the queue, not dispatched */
    gw_Script_SkillEvent(0,2,0,0,0,0,0);gw_Script_SkillEvent(99,2,0,0,0,0,0);gw_Script_SkillEvent(1,0,0,0,0,0,0);gw_Script_SkillEvent(1,13,0,0,0,0,0);
    if(gs.nev!=0){gw_test_fail("skill event queue accepted an invalid row");rc=1;}
    /* API surface */
    if(t_exec("local k=gd.skill_kinds(); assert(#k==21 and k[1]=='lcancel' and k[5]=='wavedash' and k[21]=='combo_end'); "
              "local t=gd.skill_thresholds(); assert(t.lcancel_window_frames and t.wave_min_speed and t.wavedash_max_airborne_frames and t.dash_dance_frames and t.ledge_dash_frames); "
              "assert(type(gd.skill_history(1))=='table' and #gd.skill_history(1)==0); "
              "assert(not pcall(gd.skill_history,13)); assert(not pcall(gd.skill_history,1,5,'nope')); "
              "assert(gd.crit(1)==nil); assert(type(gd.skill_state(1))=='table')",out,sizeof out)) {rc=1;gw_test_fail("skill API: %s",out);}
    /* the owner's rule: no always-on afterimages */
    if(t_exec("local ok,err=pcall(gd.afterimage_add,1,{trigger='always'}); assert(not ok and tostring(err):find('earned')); "
              "ok,err=pcall(gd.afterimage_add,1,{trigger='moving'}); assert(not ok and tostring(err):find('earned'))",out,sizeof out)) {rc=1;gw_test_fail("afterimage always-on refusal: %s",out);}
    /* crit generator: deterministic, and 5% over 10000 draws lands within four standard deviations (500 +- 87) */
    a=gw_ScriptGame_CritDraws(12345,10000,gs_fbits(0.05f));b=gw_ScriptGame_CritDraws(12345,10000,gs_fbits(0.05f));
    if(a!=b||a<413||a>587){gw_test_fail("crit generator: %d then %d crits in 10000 draws at 5%%",a,b);rc=1;}
    if(gw_ScriptGame_CritDraws(777,1000,gs_fbits(0.0f))!=0||gw_ScriptGame_CritDraws(777,1000,gs_fbits(1.0f))!=1000){gw_test_fail("crit generator bounds");rc=1;}
    t_exec("on_skill,on_lcancel,on_wavedash,on_tech,on_combo,on_crit,sk=nil,nil,nil,nil,nil,nil,nil",out,sizeof out);
    gs_update_want_events();
    return rc;
}
