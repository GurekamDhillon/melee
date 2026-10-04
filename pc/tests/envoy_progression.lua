local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();local C=D.companion
T.test('early growth and diminishing caps',function()
 local c=C.new();assert(C.feed(c,'red',20)==20 and c.stats.power.level==1)
 local f,current,next=C.progress(c.stats.power);assert(f==0 and current==0 and next>0)
 C.feed(c,'red',100000);assert(c.stats.power.level==C.tuning.level_cap and C.effects(c).damage_dealt<=1.1)
end)
T.test('white is temporary but evolution is inherited',function()
 local c=C.new();C.feed(c,'white',1);assert(c.stats.power.grade=='B' and c.dna.power[1]=='C')
 C.feed(c,'red',20);assert(C.evolve(c).type=='power' and c.stats.power.base_grade=='B')
 for _=1,C.tuning.lifespan do C.start_life_run(c) end
 C.reincarnate(c);assert(c.type=='egg' and c.stats.power.grade=='B' and c.stats.power.level==0)
end)
T.test('this life counters survive and carry is not growth',function()
 local S=T.module('save',D);local p=S.new_profile();local c=p.companions[1]
 C.feed(c,'green',200);p=S.decode(S.encode(p));c=p.companions[1];assert(c.stats.speed.life_gain==200)
 c.age=C.tuning.lifespan;C.reincarnate(c);assert(c.stats.speed.points==20 and c.stats.speed.carry==20)
 C.feed(c,'red',20);assert(C.evolve(c).type=='power')
end)
T.test('evolution preserves a mixed hidden allele',function()
 local dna=D.genetics.new();dna.power={'C','E'};local c=C.new(dna,function() return 0 end)
 C.feed(c,'red',20);C.evolve(c);assert(c.dna.power[1]=='B' and c.dna.power[2]=='E')
 C.validate(c)
end)
T.test('balanced ties and evolution only once',function()
 local c=C.new();C.feed(c,'red',20);C.feed(c,'green',20)
 assert(C.evolve(c).type=='balanced' and C.evolve(c)==nil)
 local e=C.effects(c);assert(e.drop_chance==C.tuning.drop_base and e.speed>1 and e.air_speed>e.speed)
end)
T.test('all named passives use existing effects',function()
 local names={};for _,kind in ipairs({'power','speed','guard','jump','balanced'}) do
  local c=C.new();c.type=kind;local p=C.passive(c);assert(p.name and p.description and not names[p.name]);names[p.name]=true
  local e=C.effects(c);assert(e.damage_dealt<=1.1 and e.speed<=1.2 and e.damage_taken>=.85 and e.drop_chance<=1)
 end
end)
T.test('white ties injected and S cap',function()
 local c=C.new();local _,k=C.feed(c,'white',1,function() return .99 end);assert(k=='jump' and c.stats.jump.grade=='B')
 for _,name in ipairs(C.stats) do c.stats[name].grade='S' end
 C.feed(c,'white',1);C.validate(c)
 local before=c.white_drives;C.feed(c,'white',1,function() error('no draw at cap') end)
 -- There is no random draw when all grades are capped.
 assert(c.white_drives==before+1)
 local fresh=C.new();T.refuses(function() C.feed(fresh,'white',1,function() return 1 end) end);assert(fresh.white_drives==0)
end)
T.test('three runs and full lives roundtrip without accumulating transient growth',function()
 local S=T.module('save',D);local p=S.new_profile()
 for life=1,3 do
  for run=1,C.tuning.lifespan do
   local c=p.companions[1];C.start_run(c);S.record_start(p,life*100+run);p=S.decode(S.encode(p));c=p.companions[1]
   C.feed(c,run%2==0 and 'green' or 'red',20);if run==2 then C.feed(c,'white',1) end
   local change=C.settle(c,run==3 and 'win' or 'fail');S.record_result(p,run==3 and 'win' or 'fail',60)
   p.last_settled=p.next_run;p.next_run=p.next_run+1;p.last_result=run==3 and 'win' or 'fail'
   p=S.decode(S.encode(p));c=p.companions[1]
   if run==3 then assert(change.evolution and c.type~='young') end
   if run==C.tuning.lifespan then
    assert(change.reincarnation and c.type=='egg' and c.lives==life and c.age==0)
    for _,k in ipairs(C.stats) do assert(c.stats[k].level==0 and c.stats[k].life_gain==0 and c.stats[k].points==c.stats[k].carry) end
   end
  end
 end
 assert(p.records.runs==18 and p.records.wins==3 and p.records.companions_raised==3 and p.last_seed==306)
end)
T.test('aging refuses overrun and sixth settlement ends life',function()
 local c=C.new();for _=1,C.tuning.lifespan do C.start_run(c) end
 T.refuses(function() C.start_run(c) end);assert(C.settle(c,'quit').reincarnation and c.type=='egg')
 C.start_run(c);assert(c.type=='young' and c.age==1)
end)
T.test('white cannot become permanently inherited through evolution',function()
 local c=C.new();c.stats.power.grade='A';C.feed(c,'red',20);local e=C.evolve(c)
 assert(c.stats.power.base_grade=='B' and c.dna.power[1]=='B' and c.stats.power.grade=='A')
 c.age=C.tuning.lifespan;C.reincarnate(c);assert(c.stats.power.grade=='B')
end)
T.done()
