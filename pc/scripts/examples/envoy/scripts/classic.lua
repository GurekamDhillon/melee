-- Retail director only: no generated stages, scene-table changes or CPU AI writes.
return function(D)
 local C,S=D.companion,D.save;local R={};R.__index=R
 local colours={'red','green','blue','yellow'}
 local tint={power=0xF07474FF,speed=0x77CD9CFF,guard=0x79AAF0FF,jump=0xEBD175FF}
 local function clone(p) return S.decode(S.encode(p)) end
 local function rng(seed,stage,loop,port)
  local state=(seed+stage*104729+loop*15485863+port*32452843)%2147483646+1
  local function next() state=(state*48271)%2147483647;return state/2147483647 end
  for _=1,6 do next() end;return next
 end
 function R.roll(c,seed,stage,loop,port,count)
  local t=C.tuning.retail;local random=rng(seed,stage,loop,port);local total=0
  for _,k in ipairs(C.stats) do total=total+c.stats[k].level end
  total=math.max(t.starter_budget,total)*(1+t.loop_scale*loop)/math.max(1,count or 1)^t.team_exponent
  total=math.max(4*t.stat_floor,math.min(4*t.stat_ceiling,total))
  local width=math.min(total*t.budget_width,total-4*t.stat_floor,4*t.stat_ceiling-total)
  total=math.floor(total+(random()*2-1)*width+.5)
  local weights={};local sum=0;local out={}
  for _,k in ipairs(C.stats) do local w=1+random()*t.spread_width;weights[k]=w;sum=sum+w;out[k]=t.stat_floor end
  local left=total-4*t.stat_floor
  for _,k in ipairs(C.stats) do local n=math.min(t.stat_ceiling-t.stat_floor,math.floor(left*weights[k]/sum));out[k]=out[k]+n end
  local used=0;for _,k in ipairs(C.stats) do used=used+out[k] end
  local i=math.floor(random()*4)+1
  while used<total do local k=C.stats[i];if out[k]<t.stat_ceiling then out[k]=out[k]+1;used=used+1 end;i=i%4+1 end
  return out
 end
 function R.new(g,profile,commit)
  -- rules: the new route. When true the run installs the rule host (pool, bag, slots, opponent rolls,
  -- looks) instead of the companion-stat templates; false keeps the old stat route exactly as it was.
  return setmetatable({g=g,profile=profile,commit=commit,active=false,owned={},items={},observed={},cleared={},loop=0,elapsed=0,rules=false},R)
 end
 function R:available()
  for _,api in ipairs({'start_1p','mode_1p','hold_1p','release_1p','loop_1p','spawn_1p','end_1p'}) do
   if type(self.g[api])~='function' then return false,'Classic / Adventure unavailable: engine API '..api end
  end
  return true
 end
 function R:save()
  S.refresh_records(self.working);local ok,why=self.commit(self.working)
  if ok then self.profile=clone(self.working) end;return ok,why
 end
 function R:start(mode,fighter,difficulty,stocks,seed)
  if self.active or self.pending then return false,'retail run active or save pending' end
  local ok,why=self:available();if not ok then return false,why end
  if mode~='classic' and mode~='adventure' then return false,'choose Classic or Adventure' end
  local match=self.g.match();if match and match.netplay then return false,'offline only' end
  if not self.profile or self.profile.next_run>=1000000 then return false,'profile unavailable or ledger full' end
  local working=clone(self.profile);local c=working.companions[working.active]
  seed=working.pending_seed or seed or math.random(0,2147483646)
  if not working.pending_seed then C.start_run(c);S.record_start(working,seed) end
  ok,why=self.g.start_1p{mode=mode,fighter=fighter,difficulty=difficulty or C.tuning.retail.difficulty,
   stocks=stocks or C.tuning.retail.stocks,loop=C.tuning.retail.ngplus}
  if not ok then return false,why end
  self.working=working;self.companion=c;ok,why=self:save()
  if not ok then self.g.end_1p();self.working=nil;self.companion=nil;return false,why end
  self.active=true;self.mode=mode;self.seed=seed;self.loop=0;self.elapsed=0;self.cleared={};self.reward=nil;self.results=nil;self.completed={};self.final_rewards={};self.deferred_complete=nil
  self.initial=clone(self.profile).companions[working.active].stats
  if self.rules and self.host then self.host:run_begin(seed) end
  self.g.log('envoy: retail '..mode..' started seed='..seed..(self.rules and ' (rule host)' or ' (companion stats)'));return true
 end
 local function modifiers(c)
  local e=C.retail_effects(c);return {damage_dealt=e.damage_dealt,damage_taken=e.damage_taken,
   run_speed=e.speed,air_speed=e.air_speed,shield_max=e.shield_max,
   jump_height=e.jump_height,air_jump_height=e.air_jump_height,knockback_taken=e.knockback_taken}
 end
 function R:clear_mods()
  for port in pairs(self.owned) do
   local ok,accepted=pcall(self.g.spawn_1p,port,nil)
   if ok and accepted~=false then self.owned[port]=nil end
  end
 end
 function R:clear_items()
  for h,v in pairs(self.items) do
   v.retired=true
   if self.g.item_despawn then local ok,accepted=pcall(self.g.item_despawn,h);if ok and accepted then self.items[h]=nil end end
  end
  self.observed={}
 end
 function R:stage_start(e)
  if not self.active then return end
  if self.awaiting_loop then
   self.awaiting_loop=nil;S.record_start(self.working,self.seed)
   local ok=self:save();if not ok then self:finish('fail');return end
  end
  self:clear_mods();self:clear_items();self.state=e;self.loop=e.loop or self.loop;self.tags={};self.tag_ticks=C.tuning.retail.tag_ticks
  self.boss=false;self.last_damage=nil;self.guard_flash=0
  -- The rule host replaces the stat templates: it rolls opponents when each fighter spawns and applies the
  -- bag at the checkpoint, so nothing below (companion modifiers, stat nameplates) runs on this route.
  if self.rules and self.host then self.enemy_templates={};self.host:stage_start(e);return end
  local port=e.player_port or 1
  assert(self.g.spawn_1p(port,modifiers(self.companion)),'player retail modifiers refused');self.owned[port]=true
  local count=#(e.opponents or {});self.enemy_templates={}
  for _,enemy in ipairs(e.opponents or {}) do
   local levels=R.roll(self.companion,self.seed,e.stage_index or 0,self.loop,enemy.port,count)
   local c=C.new();local leader=C.stats[1]
   for _,k in ipairs(C.stats) do
    c.stats[k].level=levels[k];c.stats[k].points=C.threshold(levels[k]);c.stats[k].life_gain=c.stats[k].points
    if levels[k]>levels[leader] then leader=k end
   end
   local v=modifiers(c);v.tint=tint[leader];self.enemy_templates[enemy.port]=v
   assert(self.g.spawn_1p(enemy.port,v),'opponent retail modifiers refused');self.owned[enemy.port]=true
   self.tags[#self.tags+1]={port=enemy.port,stat=leader,colour=tint[leader],label='P'..enemy.port..': '..C.tuning.stat_names[leader]}
  end
  local labels={};for _,tag in ipairs(self.tags) do labels[#labels+1]=tag.label end
  self.g.log('envoy: '..self.mode..' stage='..tostring(e.stage_index)..' NG+'..self.loop..' '..table.concat(labels,', '))
 end
 function R:spawn(e)
  if self.rules and self.host then self.host:spawn(e);return end
  if self.active and self.enemy_templates and self.enemy_templates[e.port] then
   -- Native templates already applied before this notification/first logic.
   self.owned[e.port]=true
  end
 end
 function R:boss_defeated() if self.active then self.boss=true end end
 function R:stage_clear(e)
  if not self.active then return end
  self:clear_mods();self:clear_items();local t=C.tuning.retail;local stage=e.stage_index or 0
  local key=(e.loop or self.loop)..':'..stage;if self.cleared[key] then return end
  self.cleared[key]=true
  local final=e.final==true
  if self.rules and self.host then self:offer_reward(stage,e.loop or self.loop,final);return end
  if not final and (t.reward_every<=0 or (stage+1)%t.reward_every~=0) then return end
  self:offer_reward(stage,e.loop or self.loop,final)
 end
 function R:offer_reward(stage,loop,final)
  local t=C.tuning.retail
  -- Rule host route: the reward is a choice of drives; the host claims the barrier hold and shows the reward screen.
  if self.rules and self.host then if final then self.final_rewards[loop]=true end;return self.host:stage_reward(stage,loop,final) end
  if not self.g.hold_1p(t.hold_ticks) then self.g.log('envoy: reward hold unavailable');return false end
  local random=rng(self.seed,stage,loop,final and 11 or 7);local options={};local first=math.floor(random()*4)
  for i=1,3 do options[i]={colour=colours[(first+i-1)%4+1],points=t.reward_points[i]*(final and t.final_multiplier or 1)} end
  if random()<t.white_chance then options[3]={colour='white',points=1} end
  if final then self.final_rewards[loop]=true end
  for _,v in ipairs(options) do v.effect=C.reward_effect(self.companion,v.colour,v.points) end
  self.reward={options=options,focus=1,stage=stage,final=final,ticks=0};self.g.log('envoy: reward choice stage='..stage);return true
 end
 function R:pick(index)
  local r=self.reward;if not r or r.preview then return false,'no unclaimed reward' end
  local choice=r.options[index];if not choice then return false,'invalid reward' end
  if not r.prepared then
   r.before=clone(self.working).companions[self.working.active].stats
   r.prepared=clone(self.working);local c=r.prepared.companions[r.prepared.active]
   C.feed(c,choice.colour,choice.points);r.after=c.stats;S.refresh_records(r.prepared);r.choice=index
  elseif index~=r.choice then return false,'retry the prepared reward' end
  local ok,why=self.commit(r.prepared);if not ok then r.error=why;return false,why end
  self.working=r.prepared;self.companion=self.working.companions[self.working.active];self.profile=clone(self.working)
  r.error=nil;r.preview=true;r.animation=0;r.animation_frames=C.tuning.retail.reward_animation_frames;r.levelups={}
  for _,k in ipairs(C.stats) do if r.after[k].level>r.before[k].level then r.levelups[k]=r.after[k].level-r.before[k].level end end
  return true
 end
 function R:acknowledge()
  if not self.reward or not self.reward.preview then return false,'choose a reward first' end
  self.reward=nil
  if self.deferred_complete then local e=self.deferred_complete;self.deferred_complete=nil;self:complete(e) end
  self.g.release_1p();return true
 end
 function R:complete(e)
  if not self.active or self.completing then return end
  local loop=e.loop or self.loop;if self.completed[loop] then return end
  -- Adventure may decide Giga Bowser only after the preceding clear barrier.
  -- One reward moment per clear: a final that the stage's own clear already rewarded (Bowser, Giga Bowser) offers nothing a second time.
  local already=self.cleared[loop..':'..(e.stage_index or 11)]
  if self.mode=='adventure' and e.final and not e.no_loop and not self.final_rewards[loop] and not self.reward and not already then
   self:offer_reward(e.stage_index or 11,loop,true)
  end
  if self.reward then self.deferred_complete=e;return end
  self.completed[loop]=true
  self:clear_mods();self.reward=nil;C.evolve(self.companion)
  local p=self.working;S.record_result(p,'win',math.max(1,self.elapsed))
  p.last_settled=p.next_run;p.next_run=p.next_run+1;p.last_result='win'
  self.loop=(e.loop or self.loop)+1
  self.results={outcome='win',boss='Defeated / NG+'..self.loop,frames=self.elapsed,before=self.initial,after=clone(p).companions[p.active].stats}
  if C.tuning.retail.ngplus and not e.no_loop and p.next_run<1000000 then
   self.completing=true
   local ok,why=self:save()
   if ok then self.completing=nil;self.awaiting_loop=true;self.elapsed=0;self.g.log('envoy: NG+'..self.loop..' stats and age carried')
   else self.results.reincarnation=C.reincarnate(self.companion);self.g.loop_1p(false);self.g.end_1p();self.pending='complete';self.active=false;self.g.log('envoy: completion save pending '..tostring(why)) end
  else
   self.results.reincarnation=C.reincarnate(self.companion);self.active=false;self.g.loop_1p(false);self.pending='complete';self:retry_save()
  end
 end
 function R:retry_save()
  if not self.pending then return true end
  local ok,why=self:save();if ok then self.pending=nil;self.completing=nil end;return ok,why
 end
 function R:finish(reason)
  if not self.active then self:clear_mods();return self:retry_save() end
  self:clear_mods();self:clear_items();self.g.loop_1p(false);self.g.release_1p();self.reward=nil;self.deferred_complete=nil
  self.g.end_1p()
  if self.awaiting_loop then
   self.awaiting_loop=nil;self.active=false;self.results.reincarnation=C.reincarnate(self.companion);self.pending='complete';return self:retry_save()
  end
  local before=clone(self.working).companions[self.working.active].stats
  S.refresh_records(self.working);local moments=C.settle(self.companion,reason)
  S.record_result(self.working,reason,self.elapsed)
  local p=self.working;p.last_settled=p.next_run;p.next_run=p.next_run+1;p.last_result=reason
  self.active=false;self.pending=reason
  self.results={outcome=reason,boss='NG+'..self.loop,frames=self.elapsed,before=self.initial,
   after=before,evolution=moments.evolution,reincarnation=moments.reincarnation}
  self.g.log('envoy: retail settled '..reason);return self:retry_save()
 end
 function R:game_over() return self:finish('fail') end
 function R:tick()
  if self.active then
   if self.tag_ticks and self.tag_ticks>0 then self.tag_ticks=self.tag_ticks-1 end
   if self.reward then
    if self.reward.preview then self.reward.animation=math.min(self.reward.animation_frames,self.reward.animation+1) end
    self.reward.ticks=self.reward.ticks+1;local mode=self.g.mode_1p()
    if not mode or mode.held==false or self.reward.ticks>=C.tuning.retail.hold_ticks then
     self.reward=nil
     if self.deferred_complete then local e=self.deferred_complete;self.deferred_complete=nil;e.no_loop=true;self:complete(e) end
     self.g.release_1p();self.g.log('envoy: reward hold expired; retail continues')
    end
   end
  else
   self:clear_mods();self:clear_items()
   -- Every way a run ends (clear, game over, stop) passes through here once the run is inactive.
   if self.host and self.host.running then self.host:run_end() end
  end
 end
 function R:item_collect(e)
  local v=self.items[e.item]
  if not self.active or not v or v.retired or e.port~=(self.state and self.state.player_port or 1) or e.name~='drive' or
   type(e.payload)~='table' or e.payload.colour~=v.colour or e.payload.amount~=v.amount then return false end
  self.items[e.item]=nil;C.feed(self.companion,v.colour,v.amount);S.refresh_records(self.working);return true
 end
 function R:physical_tick()
  if not self.g.player then return end
  for port in pairs(self.enemy_templates or {}) do
   local p=self.g.player(port);local last=self.observed[port]
   -- Missing players may be a transform or teardown, never infer a stock KO.
   local defeated=last and p and (p.falls or 0)>last.falls
   if C.tuning.retail.physical_drops and defeated and self.g.item_spawn then
    local random=rng(self.seed,self.state.stage_index or 0,self.loop,port+(last.falls+1)*31)
    if random()<C.tuning.retail.physical_drop_chance then
     local colour=colours[math.floor(random()*4)+1];local amount=C.tuning.drive_points
     local ok,h=pcall(self.g.item_spawn,'drive',last.x,last.y,{payload={colour=colour,amount=amount}})
     if ok and type(h)=='number' and h>0 then self.items[h]={colour=colour,amount=amount} end
    end
   end
   self.observed[port]=p and {falls=p.falls or 0,x=p.x or 0,y=p.y or 0} or nil
  end
 end
 function R:frame()
  if self.active then
   self.elapsed=math.min(999999999,self.elapsed+1)
   if self.rules and self.host then self.host:frame() else self:physical_tick() end
   self.guard_flash=math.max(0,(self.guard_flash or 0)-1)
   local p=self.g.player and self.g.player(self.state and self.state.player_port or 1)
   local damage=p and p.percent
   if type(damage)=='number' then
    if self.last_damage and damage>self.last_damage and self.companion.stats.guard.level>0 then self.guard_flash=C.tuning.retail.guard_flash_frames end
    self.last_damage=damage
   else self.last_damage=nil end
  end
 end
 return R
end
