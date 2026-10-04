/* Registered suite fixture. No live fighter/collision needed; actual LAB
 * zero-byte rewind acceptance is a separate integrator test. */
static int test_script_hit_rules(void) {
 GsHitRules rules,back;int slot,handle,rc,i;char out[512];
 for(slot=11;slot>=0;--slot)if(!gw_ScriptGame_HitRulesOwner(slot))break;
 if(slot<0){gw_test_fail("hit rules fixture requires an unowned fighter entity");return 1;}
 memset(&rules,0,sizeof rules);rules.count=GS_HR_MAX;rules.status=8;
 rules.rules[0][0]=77;rules.rules[0][1]=4;rules.rules[0][2]=rules.rules[0][3]=-1;
 rules.rules[0][5]=1;rules.rules[0][6]=rules.rules[0][7]=rules.rules[0][8]=rules.rules[0][11]=rules.rules[0][14]=rules.rules[0][15]=gs_fbits(1);
 rules.rules[0][14]=gs_fbits(-.9f);rules.rules[0][15]=gs_fbits(-.9f);
 for(i=1;i<rules.count;++i){memcpy(rules.rules[i],rules.rules[0],sizeof rules.rules[0]);rules.rules[i][0]=77+i;}
 if(!gs_hr_apply(slot,INT_MAX-3,&rules)) {gw_test_fail("hit rules fixture install");return 1;}
 handle=gw_ScriptGame_HitRuleRead(slot,0,12);
 gs_hr_apply(slot,INT_MAX-3,&rules);gs_hr_read(slot,&back);
 if(back.count!=32 || back.rules[31][0]!=108 || !back.rules[31][12] || !handle || handle!=back.rules[0][12] || back.status!=8 || back.rules[0][5]!=1 || back.rules[0][14]!=gs_fbits(-.9f) || back.rules[0][15]!=gs_fbits(-.9f) ||
    gs_hr_apply(slot,INT_MAX-4,&rules)) {
  gw_ScriptGame_HitRulesRelease(INT_MAX-3);gw_test_fail("hit rule stable handles/status/exclusive ownership");return 1;
 }
 gw_ScriptGame_HitRulesRelease(INT_MAX-3);
 if(gw_ScriptGame_HitRulesCount(slot)||gw_ScriptGame_FighterStatus(slot)) {gw_test_fail("hit rule unload release");return 1;}
 rc=t_exec("assert(type(gd.hit_rule_add)=='function' and type(gd.hit_rule_remove)=='function'); "
  "assert(type(gd.hit_rules_clear)=='function' and type(gd.fighter_status)=='function'); "
  "assert(type(gd.hit_rules(1))=='table' and gd.hit_rules(1).percent_only==true and gd.hit_rules(1).progression==true)",out,sizeof out);
 if(rc){gw_test_fail("hit rules API registration: %s",out);return 1;}return 0;
}
