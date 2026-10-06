-- Co-op run: two local players on one team, a build each, through a sequence of Versus-style team stages against opponents the run
-- generates, looping into New Game+. OFFLINE and one machine; it is built so that the rules are the ones an online version would run
-- (see COOP.md and _research/envoy-netplay-scoping-2026-10-05.md): every roll is a pure function of (run seed, stage, loop, player index),
-- no wall clock enters a rule, each player's choice is an explicit recorded event, and the run record has a digest.
--
-- It reuses the run host: one `run_host` per SEAT (own bag, slots, keystones, strip, reward screen, input port) over the one shared rule
-- host (mod_lab: one engine whose statuses and builds are per port, one ground). Nothing of the run host, economy, merge, keystone or
-- progression modules is forked. A one-player run never touches this file.
--
-- The stage flow uses only what the engine offers offline today: gd.scene_launch (mode=vs, teams=1, per-slot team / stocks / CPU level),
-- gd.match_end_hold (the stage never ends by itself; the run decides), gd.pause (the reward moment). What is missing is listed in COOP.md.
return function(D)
 local C={};C.__index=C
 local function seed_for(seed,stage,loop,k) return (seed+stage*104729+loop*15485863+k*32452843)%2147483646+1 end
 C.seed_for=seed_for
 local function rng(seed) local state=seed%2147483646+1;local function nxt(n) state=state*16807%2147483647;return (state-1)/2147483646*(n or 1) end;for _=1,6 do nxt() end;return nxt end -- warmed up: adjacent seeds must not share a first draw
 -- ---- the named rules (each one a value the owner can overrule; `envoy coop tuning <name> <value>`) ---------------------------------
 -- name -> {default, text}
 C.table={
  loop_length={8,'Stages in one pass before New Game+ (the last one is the final: a pick of three rare or unique drives).'},
  stocks_player={3,'Stocks each player starts a stage with (a player who loses them all spectates until the stage ends, back next stage).'},
  stocks_foe={1,'Stocks each opponent has.'},
  foes_base={2,'Opponents on the first stage.'},
  foes_step={3,'One more opponent every this many stages (the count is capped by the free ports: 4).'},
  foes_loop={1,'One more opponent per New Game+ loop.'},
  team_share={0.5,'Opponent strength follows the stronger build plus this share of the other build\'s excess (formula max_share).'},
  team_formula={'max_share','max_share: 1+max excess+share*(other excess). sum_dim: 1+(sum of excess)/(1+0.25*(n-1)). max: the stronger build only.'},
  drop_owner={'first','Who a floor drive belongs to: first (whoever touches it first), causer (the player who last hit that opponent), both (it is duplicated: one each).'},
  down_rule={'spectate','What a player with no stocks left does: spectate (waits, returns next stage) or end (the run ends).'},
  reward_every={2,'A stage owes each player a pick of three every this many stages (a solo run: 3; two players share the floor drops, so a little more often).'},
  cpu_level_base={3,'Opponent CPU level on the first stage; +1 per 2 stages, capped at 9.'},
  retry_on_loss={0,'0: the run ends when both players are out. 1: the stage repeats once with the builds as they were.'},
  max_loops={0,'Stop (as a completed run) after this many New Game+ loops. 0 = no end.'},
  cross_synergy={1,'Informational: 1. A status one player applies to an opponent is read by the other player\'s rules (the engine keeps statuses per victim). Not switchable without an engine field.'},
 }
 C.names={'loop_length','stocks_player','stocks_foe','foes_base','foes_step','foes_loop','team_share','team_formula','drop_owner','down_rule','reward_every','cpu_level_base','retry_on_loss','max_loops','cross_synergy'}
 C.values={}
 function C.reset_tuning() for _,n in ipairs(C.names) do C.values[n]=C.table[n][1] end end
 C.reset_tuning()
 function C.set(name,value)
  assert(C.table[name],'unknown co-op value '..tostring(name))
  local d=C.table[name][1]
  if type(d)=='string' then
   local ok={team_formula={max_share=1,sum_dim=1,max=1},drop_owner={first=1,causer=1,both=1},down_rule={spectate=1,['end']=1}}
   assert(ok[name] and ok[name][value],name..' takes one of '..table.concat((function() local t={} for k in pairs(ok[name]) do t[#t+1]=k end table.sort(t) return t end)(),', '))
  else value=tonumber(value);assert(value and value==value and value>=0 and value<=1000,'a number 0..1000 is required') end
  C.values[name]=value
 end
 function C.lines() local out={};for _,n in ipairs(C.names) do out[#out+1]=('%-14s now %-10s default %-10s %s'):format(n,tostring(C.values[n]),tostring(C.table[n][1]),C.table[n][2]) end;return out end
 local function V(n) return C.values[n] end
 -- ---- pure rules -------------------------------------------------------------------------------------------------------------
 C.stages={'battlefield','fd','fountain','stadium','ys','dreamland','pstadium'}
 -- Strength of the opponents a team of builds meets (each strength is >= 1; 1 is an empty build).
 function C.team_strength(strengths)
  local n=#strengths;if n==0 then return 1 end
  local ex,sum,emax={},0,0
  for i,s in ipairs(strengths) do ex[i]=math.max(0,s-1);sum=sum+ex[i];if ex[i]>emax then emax=ex[i] end end
  local f=V('team_formula')
  if f=='max' then return 1+emax end
  if f=='sum_dim' then return 1+sum/(1+0.25*(n-1)) end
  return 1+emax+V('team_share')*(sum-emax)
 end
 -- The stage a run plays at (stage, loop): a pure function of those, the seed and the tuning. The opponents' BUILDS are rolled later by
 -- the run host (foe_roll) from the team strength at that moment, seeded by the same four numbers.
 function C.plan(seed,stage,loop,fighters)
  fighters=fighters or (D.fighters and D.fighters.retail) or {'fox'}
  local r=rng(seed_for(seed,stage,loop,31))
  local n=math.min(4,V('foes_base')+math.floor(stage/math.max(1,V('foes_step')))+loop*V('foes_loop'))
  local avoid=C.last_stage
  local stage_name;repeat stage_name=C.stages[math.floor(r(#C.stages))+1] until stage_name~=avoid or #C.stages==1
  local foes={};for i=1,n do foes[i]=fighters[math.floor(r(#fighters))+1] end
  local effective=stage+13*loop
  return {stage=stage,loop=loop,stage_name=stage_name,foes=foes,cpu_level=math.min(9,V('cpu_level_base')+math.floor(effective/2)),
   stocks_player=V('stocks_player'),stocks_foe=V('stocks_foe'),final=(stage==V('loop_length')-1)}
 end
 function C.scene(plan,seats)
  local parts={'mode=vs','teams=1','items=off','time=0','enemy_team_colors=1','stage='..plan.stage_name}
  for i,s in ipairs(seats) do
   parts[#parts+1]=('p%d=%s/%s/team0/stocks%d'):format(i,s.fighter,s.policy=='human' and 'hu' or ('cpu'..(s.cpu_level or 9)),plan.stocks_player)
  end
  for i,f in ipairs(plan.foes) do parts[#parts+1]=('p%d=%s/cpu%d/team1/stocks%d'):format(#seats+i,f,plan.cpu_level,plan.stocks_foe) end
  return table.concat(parts,';')
 end
 -- The compact run record of both players: seed, the order of events, a 64-bit digest (two FNV-1a words). Builds are written through the
 -- canonical codec (sorted keys), so the record is the same on every machine that has the same pool.
 local function fnv(text,h)
  h=h or 2166136261
  for i=1,#text do h=((h~text:byte(i))*16777619)&0xFFFFFFFF end
  return h
 end
 function C.digest(text) return ('%08x%08x'):format(fnv(text),fnv(text,fnv('co-op:'))) end
 -- ---- the run -----------------------------------------------------------------------------------------------------------------
 function C.new(g,mods,app)
  local self=setmetatable({g=g,mods=mods,app=app,active=false,state='idle',hosts={},seats={},mode='coop',loop=0,stage=0,state_info={player_port=1},events={},stats={},log_lines={}},C)
  return self
 end
 function C:available()
  for _,api in ipairs({'scene_launch','match_end_hold','pause','resume','player','match'}) do if type(self.g[api])~='function' then return false,'co-op unavailable: engine API '..api end end
  if not (self.mods and self.mods.drives and self.mods.add_seat) then return false,'co-op unavailable: rule host' end
  return true
 end
 function C:log(t) self.g.log('envoy coop: '..t) end
 -- Run-level events: every player decision and every rule outcome that must be the same on both peers, in order.
 function C:event(kind,port,a,b)
  local e={n=#self.events+1,loop=self.loop,stage=self.stage,kind=kind,port=port,a=a,b=b};self.events[#self.events+1]=e
  self:log(('event %d: %s P%s %s %s'):format(e.n,kind,tostring(port),tostring(a),tostring(b)))
 end
 function C:build_text(host)
  local b=host:bag();local c=D.mod_codec
  return c.encode({bag=b:snapshot(),slots=b:slots()})
 end
 function C:record()
  local lines={('seed=%d players=%d loop=%d stage=%d'):format(self.seed or 0,#self.seats,self.loop,self.stage)}
  for _,e in ipairs(self.events) do lines[#lines+1]=('%d|%d|%d|%s|%s|%s|%s'):format(e.n,e.loop,e.stage,e.kind,tostring(e.port),tostring(e.a),tostring(e.b)) end
  for i,h in ipairs(self.hosts) do lines[#lines+1]='build'..i..'='..self:build_text(h) end
  local text=table.concat(lines,'\n');return text,C.digest(text)
 end
 -- The seat table the hosts are built with. The `rules` in it are the co-op rules; the host calls them at its own boundaries.
 function C:make_seat(i,port,fighter,policy)
  local me=self
  local seat={index=i,port=port,fighter=fighter,policy=policy,allies={[1]=true,[2]=true},colour=port}
  function seat.strength(host)
   local s={};for _,h in ipairs(me.hosts) do local _,st=me.mods.engine:family_budget(h:port0());s[#s+1]=st end
   local t=C.team_strength(s);me.last_team_strength=t;me.last_strengths=s;return t
  end
  function seat.team_alive(host) for _,h in ipairs(me.hosts) do local v=me.g.player(h:port0());if v and (v.stocks or 0)>0 then return v end end end
  function seat.anchor_port(host) for _,h in ipairs(me.hosts) do local v=me.g.player(h:port0());if v and (v.stocks or 0)>0 then return h:port0() end end end
  function seat.reward_due(host,stage,final) if final then return true end;local k=V('reward_every');return k<=1 or (stage+1)%k==0 end
  function seat.starting_keystone(host,seed)
   local first=D.keystones.starting(seed);if i==1 then return first end
   for k=1,40 do local id=D.keystones.starting(seed_for(seed,0,0,60+k));if id~=first then return id end end
   return first
  end
  function seat.emit(host,kind,a,b) me:event(kind,host:port0(),a,b) end
  function seat.drop_records(host,record,victim)
   local mode=V('drop_owner');local out={record}
   if mode=='causer' then me.owner_of[record.seed]=me.last_hitter[victim] or 1
   elseif mode=='both' then
    me.owner_of[record.seed]=1
    local twin=host.mods.drives.loot:roll(seed_for(host.seed,host.stage,host.loop,victim+777),host.mods.engine.context)
    me.owner_of[twin.seed]=2;out[2]=twin
   end
   return out
  end
  function seat.route(host,r)
   local o=me.owner_of[r.seed];if not o then return nil end
   local to=me.hosts[o];if to and to~=host then return to end
  end
  function seat.gather(host,list)
   local mine={}
   for k,r in ipairs(list) do
    local o=me.owner_of[r.seed];if not o then o=((me.stage+k)%#me.hosts)+1 end
    local to=me.hosts[o];if to==host then mine[#mine+1]=r else me:log(('uncollected drop goes to P%d'):format(to:port0()));to:gain(r,'uncollected (given by the team)') end
   end
   return mine
  end
  seat.barrier={
   open=function(host) me.pending_screen=host;return true end,
   release=function(host,reason) me:screen_done(host,reason) end,
  }
  return seat
 end
 -- Called once: the hosts exist for the life of the mod (the seat's drive host is added to the rule host once).
 function C:ensure_hosts(app_host_new)
  if #self.hosts>0 then return end
  local mods=self.mods
  local fighters={'fox','marth'}
  local s1=self:make_seat(1,1,fighters[1],'human');s1.drives=mods.drives
  local s2=self:make_seat(2,2,fighters[2],'human');s2.drives=mods:add_seat(2)
  self.seats={s1,s2}
  self.hosts[1]=app_host_new(self.g,mods,self,s1);self.hosts[2]=app_host_new(self.g,mods,self,s2)
  if mods.foes then local old=mods.foes.defer;mods.foes.defer=function() return self.screen_owner~=nil or (old~=nil and old()) end end -- the opponent plate waits while a reward screen is up (it bled through the grid)
  self.host=self.hosts[1]
 end
 function C:start(opts)
  opts=opts or {}
  local ok,why=self:available();if not ok then return false,why end
  if self.active then return false,'a co-op run is active' end
  local m=self.g.match();if m and m.netplay then return false,'offline only' end
  self:ensure_hosts(opts.new_host or D.run_host.new)
  self.seats[1].fighter=opts.f1 or 'fox';self.seats[2].fighter=opts.f2 or 'marth'
  self.seats[1].policy=opts.p1 or 'human';self.seats[2].policy=opts.p2 or 'human'
  self.seed=opts.seed or math.random(0,2147483646)  -- the only per-peer draw: online the host chooses the seed and sends it
  self.max_loops=opts.max_loops
  self.stage,self.loop=0,0;self.events={};self.stats={};self.owner_of={};self.last_hitter={};self.results={}
  self.attempt=0;self.active=true;self.state='launching';self.state_info={player_port=1}
  self.mods.tap=function(kind,a,b) self:tap(kind,a,b) end
  self.mods.drives.drops.item_name='drive_coop' -- the floor drive both players can touch (the solo item only answers port 1: found when seat 2 never collected a drop)
  for i,h in ipairs(self.hosts) do h:run_begin(self.seed) end
  -- run_begin of host 1 resets the shared rule host and every seat's bag; host 2 then gives its own starter from the same run seed (salted by seat).
  self:event('run_begin',0,self.seed,#self.seats)
  C.last_stage=nil
  self:launch()
  return true
 end
 function C:launch()
  local plan=C.plan(self.seed,self.stage,self.loop,D.fighters and D.fighters.retail)
  C.last_stage=plan.stage_name
  self.plan=plan;self.scene_text=C.scene(plan,self.seats)
  self.state='launching';self.launch_ticks=0;self.expected={1,2};for i=1,#plan.foes do self.expected[#self.expected+1]=2+i end
  self.pending_screen=nil;self.cleared=false;self.begin_frames=0
  self.last_pct={};self.dealt={};self.taken={};self.last_hitter={}
  self:log(('stage %d NG+%d: %s'):format(self.stage,self.loop,self.scene_text))
  self.g.scene_launch(self.scene_text)
 end
 -- ---- engine events ---------------------------------------------------------------------------------------------------------------
 function C:scene_started()
  if not self.active then return end
  if self.state=='stage' then self:log('the engine started the stage again by itself (a rematch of the seeded scene): the stage is replayed');self:event('engine_restart',0,self.stage) end
  if self.state=='launching' or self.state=='stage' then self.state='staging';self.begin_frames=0 end
 end
 -- The engine ended the match under the run. With a human in slot 0 the stage-long hold prevents it; with CPU-assisted players (no human
 -- slot 0) or when slot 0 has no stocks the hold does not apply, so the run reads the outcome from the last frames it saw and carries on.
 function C:scene_ended()
  if not self.active then return end
  if self.state=='stage' and not self.cleared then
   local a,f=self.last_allies or 0,self.last_foes or 1
   self:log(('the engine ended the match before the run decided it (allies alive %d, opponents alive %d)'):format(a,f))
   self:event('engine_ended',0,a,f)
   if f==0 and a>0 then self:clear() else self:fail('the engine ended the match') end
  end
 end
 function C:tap(kind,a,b)
  if kind=='hit' and a and b then self.last_hitter[b]=a end
 end
 -- ---- per frame ----------------------------------------------------------------------------------------------------------------------
 function C:present(p) local v=self.g.player(p);return v end
 function C:begin_stage()
  local plan=self.plan;self.state='stage';self.stage_frames=0;self.kinds_seen={};self.stage_id=(self.stage_id or 0)+1
  local opponents={};for i=1,#plan.foes do opponents[i]={port=2+i} end
  self.g.match_end_hold('envoy-coop',true)
  for _,s in ipairs(self.seats) do if s.policy=='cpu' and self.g.cpu_assist then pcall(self.g.cpu_assist,s.port,{lcancel=.9,perfect_shield=.6,tech=.9,wavedash=.3,fast_fall=.6,tech_dir='away',seed=seed_for(self.seed,self.stage,self.loop,s.port)}) end end
  for _,h in ipairs(self.hosts) do h:stage_start({stage_index=self.stage,loop=self.loop,stage_kind='team',opponents=opponents,player_port=h:port0()}) end
  for _,h in ipairs(self.hosts) do h.since=0 end
  self:log(('stage %d begins: %d opponents, strength team %s'):format(self.stage,#plan.foes,tostring(self.last_team_strength)))
 end
 function C:frame()
  if not self.active then return end
  if self.state=='staging' then
   self.begin_frames=self.begin_frames+1
   local all=true;for _,p in ipairs(self.expected) do if not self:present(p) then all=false end end
   if all and self.begin_frames>=2 then self:begin_stage() end
   if self.begin_frames>1800 then self:log('stage never filled its slots: abandoning the run');self:finish('error:slots') end
   return
  end
  if self.state~='stage' then return end
  self.stage_frames=self.stage_frames+1
  for _,h in ipairs(self.hosts) do h:frame() end
  if self.stage_frames==1 then self.foe_strength={};for i=1,#self.plan.foes do self.foe_strength[i]=1 end end
  if self.stage_frames>=60 and self.stage_frames%30==0 then local fs=self.foe_strength -- the strongest each opponent's build was seen during the stage (a roll lands a few seconds in)
   for i=1,#self.plan.foes do local ok,_,v=pcall(function() return self.mods.engine:family_budget(2+i) end);if ok and v and v>(fs[i] or 0) then fs[i]=v end end end
  -- who dealt and took what: percent changes, credited to the last player who hit the victim
  for p,v in pairs(self:snapshot_players()) do
   local before=self.last_pct[p]
   if before and v.percent>before then local d=v.percent-before;self.taken[p]=(self.taken[p] or 0)+d;local a=self.last_hitter[p];if a then self.dealt[a]=(self.dealt[a] or 0)+d end end
   self.last_pct[p]=v.percent
  end
  self:watch()
 end
 function C:snapshot_players() local out={};for p=1,6 do local v=self.g.player(p);if v then out[p]={percent=v.percent or 0,stocks=v.stocks or 0} end end;return out end
 function C:watch()
  if self.cleared then return end
  local allies_alive,foes_alive=0,0
  for p=1,2 do local v=self.g.player(p);if v and (v.stocks or 0)>0 then allies_alive=allies_alive+1 end end
  for i=1,#self.plan.foes do local v=self.g.player(2+i);if v and (v.stocks or 0)>0 then foes_alive=foes_alive+1 end end
  self.last_allies,self.last_foes=allies_alive,foes_alive
  if self.stage_frames<60 then return end
  self.downs=self.downs or {};for p=1,2 do local v=self.g.player(p);local out=(not v) or (v.stocks or 0)<=0;if out and not self.downs[p] then self.downs[p]=true;self:event('player_out',p,self.stage,self.stage_frames) end end
  if V('down_rule')=='end' and allies_alive<2 then return self:fail('a player is out (down_rule end)') end
  if allies_alive==0 then return self:fail('both players are out') end
  if foes_alive==0 and not self.host.holding_end then return self:clear() end
 end
 -- ---- stage clear: results, the reward moment, the next stage ---------------------------------------------------------------------
 function C:stage_row(result)
  local row={loop=self.loop,stage=self.stage,result=result,frames=self.stage_frames,foes=#self.plan.foes,team_strength=self.last_team_strength,strengths=self.last_strengths,
   dealt={},taken={},stage_name=self.plan.stage_name}
  for p=1,6 do row.dealt[p]=self.dealt[p] or 0;row.taken[p]=self.taken[p] or 0 end
  -- the per-stage detail the synthetic campaigns aggregate: both builds' strength, the opponents' strength, drops each seat gained, script cost
  local st={};for _,h in ipairs(self.hosts) do local _,v=self.mods.engine:family_budget(h:port0());st[#st+1]=v end
  local fs=self.foe_strength or {}
  local gains={0,0};for _,e in ipairs(self.events) do if e.kind=='gain' and e.loop==self.loop and e.stage==self.stage and gains[e.port] then gains[e.port]=gains[e.port]+1 end end
  local cost=self.mods.cost and self.mods.cost.frame;local cm,cp,cx=0,0,0
  if cost then local n=math.min(cost.n,240);local sum,sorted=0,{};for i=1,n do sum=sum+cost[i];sorted[i]=cost[i] end;table.sort(sorted);if n>0 then cm=sum/n;cp=sorted[math.max(1,math.ceil(n*.95))] end;cx=cost.max end
  row.detail={builds=st,foe=fs,gains=gains,cost={cm,cp,cx}}
  self:log(('stage detail: loop %d stage %d name=%s builds=%.2f/%.2f foes=%s gains=%d/%d cost=%.3f/%.3f/%.3f'):format(self.loop,self.stage,self.plan.stage_name,st[1] or 0,st[2] or 0,
   table.concat((function() local t={};for i,v in ipairs(fs) do t[i]=('%.2f'):format(v) end;return t end)(),','),gains[1],gains[2],cm,cp,cx))
  self.results[#self.results+1]=row;self.stats[#self.stats+1]=row
  self:log(('stage row: loop %d stage %d %s frames=%d foes=%d team=%.2f dealt P1=%.0f P2=%.0f taken P1=%.0f P2=%.0f'):format(row.loop,row.stage,result,row.frames,row.foes,row.team_strength or 0,row.dealt[1],row.dealt[2],row.taken[1],row.taken[2]))
 end
 function C:clear()
  self.cleared=true;self:stage_row('win');self:event('stage_clear',0,self.stage,self.stage_frames)
  self.g.pause();self.owns_pause=true;self.state='reward'
  self.queue={};for _,h in ipairs(self.hosts) do self.queue[#self.queue+1]=h end
  self.reward_final=self.plan.final
  self:next_screen()
 end
 function C:next_screen()
  while #self.queue>0 do
   local h=table.remove(self.queue,1);self.pending_screen=nil
   local shown=h:stage_reward(self.stage,self.loop,self.plan.final)
   if shown and self.pending_screen==h then self.screen_owner=h;return end
  end
  self.screen_owner=nil;self:advance()
 end
 function C:screen_done(host,reason)
  self:event('screen_done',host:port0(),reason)
  if self.screen_owner==host then self.screen_owner=nil;self.pending_screen=nil;self:next_screen() end
 end
 function C:advance()
  if self.owns_pause then self.g.resume();self.owns_pause=nil end
  local was_final=self.plan.final
  if was_final then
   self.loop=self.loop+1;self.stage=0;self:event('new_game_plus',0,self.loop)
   local ml=self.max_loops or V('max_loops');if ml>0 and self.loop>ml then return self:finish('complete') end
  else self.stage=self.stage+1 end
  self.attempt=0;self.g.match_end_hold('envoy-coop',false)
  self:launch()
 end
 function C:fail(why)
  if self.cleared then return end
  self.cleared=true;self:stage_row('loss');self:event('stage_fail',0,self.stage,why)
  if V('retry_on_loss')>0 and self.attempt<1 then
   self.attempt=self.attempt+1;self:log('stage lost: one retry with the builds as they were');self.g.match_end_hold('envoy-coop',false);return self:launch()
  end
  self:finish('loss')
 end
 function C:finish(reason)
  if not self.active then return end
  local text,digest=self:record()
  self.final={reason=reason,digest=digest,stages=#self.results,loops=self.loop,seed=self.seed}
  self:log(('run end: %s after %d stages (NG+%d) digest=%s'):format(reason,#self.results,self.loop,digest))
  if self.owns_pause then self.g.resume();self.owns_pause=nil end
  pcall(self.g.match_end_hold,'envoy-coop',false)
  for _,h in ipairs(self.hosts) do h:run_end() end
  self.mods.tap=nil;self.mods.drives.drops.item_name=nil;self.active=false;self.state='idle'
  if reason~='stopped' and self.g.scene_launch then pcall(self.g.scene_launch,'mode=menu') end
 end
 function C:stop() if self.active then self:finish('stopped') end;return true end
 function C:tick()
  if not self.active then return end
  if self.state=='launching' then self.launch_ticks=self.launch_ticks+1;if self.launch_ticks>3600 then self:log('the stage never started (launch timeout)');self:finish('error:launch') end end
  for _,h in ipairs(self.hosts) do h:tick() end
 end
 function C:draw()
  if not self.active then return end
  if self.screen_owner then self.screen_owner:draw();self.draws_owner_only=(self.draws_owner_only or 0)+1;return end -- one player's screen is up: the other strip stays out of its way
  if self.state=='reward' then self.draws_other_during_screen=(self.draws_other_during_screen or 0)+1 end -- (counted: it must stay 0 in a run)
  for _,h in ipairs(self.hosts) do h:draw() end
 end
 -- Every place where one machine lets a rule see more than one peer could: listed here so the online version has the checklist.
 C.cheats={
  'both players\' reward choices are available at the same instant on one machine (online: each peer sees only its own offer and sends the pick as a message; the host validates it against the pure offer function)',
  'one clock: the reward screens are shown in turn here; online they are simultaneous with a countdown counted in lobby ticks',
  'both builds are read from one Lua engine (online each peer holds both builds from the run record and evaluates the same way)',
  'a floor drive is touched by whoever walks to it on this machine; online the touch is a game event both peers must see on the same frame',
  'the run seed is drawn here with math.random; online the lobby host chooses it',
  'hosts of both seats are run in one process: a script error stops both',
 }
 return C
end
