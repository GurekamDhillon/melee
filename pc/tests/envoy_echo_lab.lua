local T=dofile('melee/pc/tests/envoy_testlib.lua');local D=T.rules();D.mod_codec=T.module('mod_codec',D);D.mod_schema=T.module('mod_schema',D);D.mod_pool=T.module('mod_pool',D);D.mod_engine=T.module('mod_engine',D);D.mod_echo_lab=T.module('mod_echo_lab',D)
T.test('echo commands and journal ownership survive checkpoints and cleanup',function()
 local host={g={echoes=function()return {journal=true}end,player=function()return {}end,log=function()end},engine=D.mod_engine.new(1,D.mod_pool),allowed=function()return true end,replaying=function()return false end,options={}}
 local a=D.mod_echo_lab.new(host);assert(a:command('add 24 nair .4'));local ops={};a:ops(ops,{[1]={}},{});assert(#ops==1 and ops[1].op=='echoes' and ops[1].rules[1].delay==24)
 local saved=a:snapshot();local b=D.mod_echo_lab.new(host);b:restore(saved);assert(D.mod_codec.encode(saved)==D.mod_codec.encode(b:snapshot()))
 ops={};b:ops(ops,{},{});assert(#ops==1 and #ops[1].rules==0);assert(b:command('clear'))
 T.refuses(function()b:restore{manual={[1]={{delay=0}}},owned={}}end)
 T.refuses(function()b:restore{manual={[1]={[2]={delay=24,match={move='any'},damage=.4,knockback=1,once_per_move=true}}},owned={}}end)
 host.g.echoes=function()return {}end;assert(not b:command('add 24'))
end)
T.test('presentation creation failure retries without accepted keys or gameplay mutation',function()
 local attempts,logs=0,0;local host={g={log=function()logs=logs+1 end,echo_afterimage=function()attempts=attempts+1;if attempts<=2 then return nil,'emitter owned by another script' end;return 17 end},engine=D.mod_engine.new(1,D.mod_pool)}
 host.engine:set_build(1,{echoes=1},{});local a=D.mod_echo_lab.new(host);a.manual[1]={{delay=8,match={move='any'},damage=.4,knockback=1,once_per_move=true}}
 a:present({[1]={}});assert(not a.visual[1] and not a.visual_keys[1]);a:present({[1]={}});assert(logs==1 and not a.visual_keys[1])
 a:present({[1]={}});assert(a.visual[1]==17 and a.visual_keys[1] and #a.manual[1]==1);assert(not a.visual_ready[1])
end)
T.test('new echo emitters warm after creation and tickets release on reset',function()
 local calls,release={},{};local done=false;local host={g={log=function()end,echo_afterimage=function()calls[#calls+1]='emitter';return 17 end,afterimage_remove=function()end,
 warm=function(d)assert(d.fighters[1]==1);calls[#calls+1]='warm';return 91 end,warm_done=function(h)assert(h==91);return done end,warm_release=function(h)release[#release+1]=h end},engine=D.mod_engine.new(1,D.mod_pool)}
 host.engine:set_build(1,{echoes=1},{});local a=D.mod_echo_lab.new(host);a:present({[1]={}});assert(calls[1]=='emitter' and calls[2]=='warm' and a.warm_jobs[1]==91 and not a.visual_ready[1])
 a:reset();assert(release[1]==91 and not next(a.warm_jobs))
 a:present({[1]={}});done=true;a:present({[1]={}});assert(a.visual_ready[1] and not a.warm_jobs[1] and #release==2)
end)
T.test('failed motion warm retries without abandoning collision ownership',function()
 local warms,releases=0,0;local host={g={log=function()end,echo_afterimage=function()return 17 end,warm=function()warms=warms+1;return warms end,warm_done=function(h)if h==1 then return false,'pipeline preparation failed' end;return true end,warm_release=function()releases=releases+1 end},engine=D.mod_engine.new(1,D.mod_pool)}
 host.engine:set_build(1,{echoes=1},{});local a=D.mod_echo_lab.new(host);a.owned[1]=true
 assert(not a:present({[1]={}}));assert(a.visual[1]==17 and not a.visual_ready[1] and not a.warm_jobs[1] and a.owned[1])
 assert(a:present({[1]={}}));assert(warms==2 and releases==2 and a.visual_ready[1] and a.owned[1])
end)
T.done()
