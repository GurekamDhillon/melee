-- Envoy VS builds, part 2: the rule host's offline VS path. `envoy vs on` (the explicit switch) lets `foe` and `envoy vs max` give any CPU port a
-- rolled build in an ordinary offline VS match; the LAB path, a run's host, netplay and the unused-switch default stay as they were.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={};for _,name in ipairs({'mod_schema','mod_codec','mod_engine','keystones','mod_pool','mod_echo_lab'}) do D[name]=T.module(name,D) end
D.mod_display={new=function(g,engine)
 local v={engine=engine}
 function v:warm(e) self.engine=e;return true,'warming' end
 function v:update(e) self.engine=e;e.display.trace_key=table.concat(e.trace,',') end
 function v:clear() end
 function v:intensity(n) self.engine.display.intensity=n end
 function v:on_loadstate(e) self.engine=e;self:update(e) end
 function v:tick(e) end
 function v:draw() end
 return v
end}
for _,n in ipairs({'drive_loot','drive_bag','foe_roll'}) do D[n]=T.module(n,D) end
D.foe_lab=T.module('foe_lab',D)
D.pickup_juice={pitch={},new=function()return {clear=function()end,tick=function()end}end}
for _,n in ipairs({'menu_input','drive_menu','drive_drop','drive_lab'}) do D[n]=T.module(n,D) end
D.mod_lab=T.module('mod_lab',D)

-- An offline VS match (not LAB mode) with P1 and P2 both level 9 CPUs, as the scene `p1=peach/cpu9/fight;p2=fox/cpu9/fight` makes them.
local function fixture(opts)
 opts=opts or {}
 local s={commands={},logs={},modes={},commits=0,lab=opts.lab or false,net=false,blocked=false,players={
  [1]={percent=0,stocks=4,falls=0,char=1,action=14,airborne=false,cpu=not opts.human},
  [2]={percent=0,stocks=4,falls=0,char=2,action=14,airborne=false,cpu=true}}}
 local g={pad=function()return {} end,input_mask=function()end,items=function()return {} end,paused=function()return true end,sim_supported=true,
  hit_rule_add=function()error('direct hit-rule write is not replayable')end,fighter_status=function()error('direct status write is not replayable')end,
  command=function(name,fn) s.commands[name]=fn end,log=function(line) s.logs[#s.logs+1]=line end,
  cpu_mode=function(p,m) s.modes[#s.modes+1]=p..':'..m;return true end,
  match=function() return {active=true,netplay=s.net,stage=8} end,lab_mode=function() return s.lab end,
  player=function(p) return s.players[p] end,sim_replaying=function() return false end,
  hit_rules=function() return {percent_only=true,progression=true} end,echoes=function() return {journal=true} end,
  sim_read=function() return s.blob end,sim_clear=function() s.blob=nil end,
  sim_commit=function(blob,ops) s.commits=s.commits+1;s.blob=blob;s.ops=ops;return true end}
 s.g=g
 local o={};if opts.hosted then o.run_host=function() return true end;o.run_ready=function() return true end end
 o.blocked=function() return s.blocked end
 s.a=D.mod_lab.new(g,o);return s,s.a
end
local function frames(s,a,n) for _=1,n or 20 do a:frame() end end
local function logged(s,text) for _,l in ipairs(s.logs) do if l:find(text,1,true) then return true end end return false end

T.test('the unused switch changes nothing: a plain VS match refuses foe and vs, a LAB match still works',function()
 local s,a=fixture()
 local ok,why=a.foes:command('roll 1.8 1 2');assert(not ok and tostring(why):find('LAB',1,true),tostring(why))
 ok,why=a:vs_command('max');assert(not ok and tostring(why):find('envoy vs on',1,true),'the refusal names the switch: '..tostring(why))
 ok,why=a:allowed();assert(not ok)
 frames(s,a);assert(s.commits==0 and next(a.foes.builds)==nil)
 local l,la=fixture({lab=true,human=true}) -- the LAB: sync roll, pending at once, the same as before
 assert(la.foes:command('roll 1.8 1 2'));assert(#la.foes.pending==1)
 frames(l,la);assert(la.foes.builds[2] and l.commits>0)
end)

T.test('the switch is explicit, offline, per match and refused online or under a run',function()
 local s,a=fixture()
 assert(a:vs_command('on'));assert(a:allowed())
 s.net=true;assert(not a:allowed(),'netplay must refuse even with the switch on');local ok=a:vs_command('on');assert(not ok,'the switch must refuse online')
 s.net=false;assert(a.vs_on,'the switch survives a refused online call only while no scene ends')
 a:scene();assert(not a:allowed(),'a new scene clears the switch');assert(not a.vs_on)
 assert(a:vs_command('on'));s.net=true;frames(s,a);assert(next(a.foes.builds)==nil and s.commits==0,'nothing published online')
 s.net=false;s.blocked=true;a:scene();ok=a:vs_command('on');assert(not ok,'an Envoy run or director blocks the VS switch')
 local h,ha=fixture({hosted=true});ok=ha:vs_command('on');assert(not ok,'a run host owns its own builds');assert(ha:allowed(),'the host path stays allowed without any switch')
 assert(a:vs_command('off') or true)
end)

T.test('envoy vs max: every present CPU gets a rolled max build and is set to fight, P1 included, over slices',function()
 local s,a=fixture();assert(a:vs_command('on'))
 assert(a:vs_command('max 5'));assert(#a.foes.pending==0 and next(a.foes.jobs),'the roll is sliced across frames, not done in one call')
 frames(s,a,40)
 for p=1,2 do local r=a.foes.builds[p];assert(r,'P'..p..' has no build');assert(r.port==p and r.strength>1);a.foes.roller:validate(r) end
 local fought={};for _,m in ipairs(s.modes) do fought[m]=true end;assert(fought['1:fight'] and fought['2:fight'],table.concat(s.modes,','))
 assert(D.mod_progression.effective(a.engine.context)>=10,'an unset depth means the top band (depth 12 loop 3)')
 assert(a.engine.equipped[1] and a.engine.equipped[2],'both builds are installed in the engine')
 assert(s.commits>0)
 assert(not a.foes.builds[1].build.seed,'records carry no hidden fields');assert(next(a.foes.jobs)==nil,'jobs are finished')
 -- distinct per port
 assert(D.mod_codec.encode(a.foes.builds[1].build)~=D.mod_codec.encode(a.foes.builds[2].build))
 -- a numeric strength and a depth/loop argument
 assert(a:vs_command('max 9 3 2'));frames(s,a,40);assert(a.engine.context.depth==3 and a.engine.context.loop==2,'depth and loop arguments set the context')
 assert(a.foes.builds[1] and a.foes.builds[2])
end)

T.test('a human P1 keeps its own build: vs rolls only the CPUs, and foe refuses a human port',function()
 local s,a=fixture({human=true});assert(a:vs_command('on'));assert(a:vs_command('max 3'));frames(s,a,40)
 assert(a.foes.builds[2] and not a.foes.builds[1])
 local ok,why=a.foes:command('roll max 3 1');assert(not ok and tostring(why):find('CPU',1,true),tostring(why))
 local none,na=fixture();none.players[1].cpu=false;none.players[2].cpu=false;assert(na:vs_command('on'));ok,why=na:vs_command('max');assert(not ok and tostring(why):find('CPU',1,true),tostring(why))
end)

T.test('foe takes any CPU port 1..6 in a VS match with the switch, not a missing one',function()
 local s,a=fixture();s.players[3]={percent=0,stocks=4,falls=0,char=3,action=14,airborne=false,cpu=true};assert(a:vs_command('on'))
 assert(a.foes:command('roll max 11 3'));frames(s,a,40);assert(a.foes.builds[3] and a.foes.builds[3].port==3)
 assert(a.foes:command('roll 1.8 11 1'));frames(s,a,10);assert(a.foes.builds[1])
 local ok,why=a.foes:command('roll 1.8 1 7');assert(not ok and tostring(why):find('1..6',1,true),tostring(why))
 ok=a.foes:command('stand 1');assert(ok);ok=a.foes:command('fight 3');assert(ok)
end)

T.test('seeds above 2^31 are folded and said so; a bad seed says what is wrong',function()
 local s,a=fixture({lab=true,human=true})
 assert(a.foes:command('roll 1.8 4294967297 2'));assert(logged(s,'folded'),'the fold is logged');assert(a.foes.pending[1].record.seed<=2147483646)
 local ok,why=a.foes:command('roll 1.8 -4 2');assert(not ok and tostring(why):find('seed',1,true),tostring(why))
 ok,why=a.foes:command('roll 1.8 1.5 2');assert(not ok and tostring(why):find('whole',1,true),tostring(why))
end)

T.test('an omitted strength targets the player, or max when P1 is a CPU or absent; a high number is sliced and always lands',function()
 local s,a=fixture();assert(a:vs_command('on'));assert(a.foes:command('roll - 4 2'));assert(next(a.foes.jobs) and a.foes.jobs[2].job.max,'no strength with a CPU P1 means max')
 frames(s,a,30);assert(a.foes.builds[2])
 local l,la=fixture({lab=true,human=true});assert(la.foes:command('roll - 4 2'));assert(#la.foes.pending==1 and la.foes.pending[1].record.requested<2,'a human player keeps the old rule (its own strength)')
 local h,ha=fixture();assert(ha:vs_command('on'));assert(ha.foes:command('roll 400 5 2'));assert(#ha.foes.pending==1,'a number is the single call, bounded')
 frames(h,ha,5);assert(ha.foes.builds[2] and not logged(h,'refused'),'a high strength lands')
 assert(ha:vs_command('400 6'));assert(next(ha.foes.jobs),'envoy vs slices a number too (the whole search)')
 frames(h,ha,260);assert(ha.foes.builds[1] and ha.foes.builds[2] and not logged(h,'refused'),'the sliced high strengths land')
end)

T.test('a roll that cannot finish says why instead of vanishing, and a vanished CPU drops its job',function()
 local s,a=fixture();assert(a:vs_command('on'));assert(a:vs_command('max 2'));s.players[2]=nil;frames(s,a,40)
 assert(a.foes.builds[1] and not a.foes.builds[2] and logged(s,'dropped'),'the missing CPU is reported')
end)
T.done()
