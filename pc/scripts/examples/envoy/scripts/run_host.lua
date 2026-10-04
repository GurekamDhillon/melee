-- Run adapter for the rule host (mod_lab): Classic and Adventure install the same pool, bag, slots,
-- opponent rolls and looks the LAB does, at the director's own lifecycle boundaries (run begin, stage
-- start, fighter spawn, stage clear, run end). Everything here is plain data handed to the host; the host
-- commits it through the checkpoint like any other rule. It exists only while the retail run's `rules`
-- switch is on, so the old companion-stat route stays playable beside it.
return function(D)
 local H={};H.__index=H
 local function seed_for(seed,stage,loop,port)
  return (seed+stage*104729+loop*15485863+port*32452843)%2147483646+1
 end
 -- settle_frames: the look warm-up draws the fighters' models, and a retail stage's fighters are not safe to draw
 -- until the entrance has played (a crash in the model draw 30 frames in was seen on the first try).
 H.tuning={drop_chance=.25,max_owed=6,settle_frames=30,roll_attempts=2}
 function H.new(g,mods,retail)
  return setmetatable({g=g,mods=mods,retail=retail,running=false,owed={},fell={},rolls={},stage=0,loop=0,drops=0,since=0},H)
 end
 function H:log(text) self.g.log('envoy rules: '..text) end
 function H:run_begin(seed)
  self.running=true;self.seed=seed;self.owed={};self.fell={};self.rolls={};self.stage=0;self.loop=0;self.drops=0
  self.mods:run_end() -- a new run starts from an empty bag, whatever the last one left
  self.mods:set_context(D.mod_progression.context(0,0))
  self:stage_reward(0,0,false,5) -- a starter drive, so the first opponents already roll against a build
  self:log('run begin seed='..seed)
 end
 -- Progression follows the director's stage and NG+ loop; the bag's slots and keystones grow with it.
 function H:stage_start(e)
  if not self.running then return end
  self.stage=e.stage_index or 0;self.loop=e.loop or self.retail.loop or 0;self.fell={};self.since=0 -- rolls survive: a fighter's spawn signal can arrive before this stage's start signal
  self.mods:set_context(D.mod_progression.context(math.min(self.stage,12),self.loop))
  self:log(('stage %d NG+%d context depth=%d slots=%d'):format(self.stage,self.loop,self.mods.engine.context.depth,D.mod_progression.slots(self.mods.engine.context)))
 end
 -- Opponents roll a build from the player's current strength, exactly as `foe roll` does.
 function H:spawn(e)
  if not self.running or not e or not e.port or e.port==(self.retail.state and self.retail.state.player_port or 1) then return end
  -- The spawn signal fires while the scene is still being built: only note the port here. The roll (which
  -- also warms the look shaders) is queued from the first logic frame, when the fighter is fully present.
  self.rolls[e.port]=true
 end
 function H:roll_wanted()
  local foes=self.mods.foes;if not foes then return end
  -- Rolls scale to the player's published build: wait until owed drives are in the bag and committed.
  if self.owed[1] or #self.mods.drives.pending>0 then return end
  -- A new stage starts with a fresh engine: the bag's build reaches it at the host's next logic frame.
  if self.mods.drives:has_build() and not self.mods.drives.applied then return end
  for p in pairs(self.rolls) do
   local v=self.g.player(p)
   if v and v.cpu then
    local ok,why=pcall(function()
     if not (foes.jobs and foes.jobs[p]) then
      local _,strength=self.mods.engine:family_budget(1)
      foes:roll_begin(p,strength,seed_for(self.seed,self.stage,self.loop,p),self.stage,'normal')
     end
     return foes:roll_advance(p,H.tuning.roll_attempts)
    end)
    if not ok then self.rolls[p]=nil;self:log('opponent roll refused P'..p..': '..tostring(why))
    elseif why then self.rolls[p]=nil;self:log('queued opponent roll P'..p) end
   end
  end
 end
 -- A cleared stage offers one drive, equipped into the first free slot (the bag menu can swap it).
 function H:stage_reward(stage,loop,final,salt)
  if not self.running then return false end
  local record=self.mods.drives.loot:roll(seed_for(self.seed,stage,loop,salt or (final and 11 or 7)),self.mods.engine.context)
  if #self.owed<H.tuning.max_owed then self.owed[#self.owed+1]=record end
  self:log('stage clear offers '..self.mods.drives.loot:name(record));return true
 end
 -- A defeated opponent can drop a drive where it fell; picking it up fills the bag like any LAB drop.
 function H:ready() return self.running and self.since>=H.tuning.settle_frames end
 function H:frame()
  if not self.running then return end
  self.since=self.since+1;if not self:ready() then return end
  local drives=self.mods.drives;if not drives then return end
  local port0=self.retail.state and self.retail.state.player_port or 1
  for p,build in pairs(self.mods.foes and self.mods.foes.builds or {}) do
   local v=self.g.player(p);local falls=v and v.falls or 0
   if self.fell[p] and falls>self.fell[p] and p~=port0 then
    self.drops=self.drops+1
    local roll=((seed_for(self.seed,self.stage,self.loop,p)+self.drops*48271)%2147483646)/2147483646
    if roll<H.tuning.drop_chance and drives.drops:count()<4 and #drives.bag.items+drives.drops:count()<12 then
     local ok,why=pcall(function() drives.drops:spawn(drives.loot:roll(seed_for(self.seed,self.stage,self.loop,p+self.drops),self.mods.engine.context)) end)
     self:log(ok and ('opponent P'..p..' dropped a drive') or ('drop refused: '..tostring(why)))
    end
   end
   self.fell[p]=falls
  end
 end
 -- Rolling a build and validating a bag edit are each close to a script call's whole budget (2M instructions,
 -- 50 ms), and on_frame already carries the host's own work: staging happens in on_tick, a call of its own.
 -- Both only queue plain data; the host commits it at its next logic frame.
 function H:tick()
  if not self:ready() then return end
  local drives=self.mods.drives;if not drives then return end
  if self.owed[1] and self.mods:allowed() and not self.mods:replaying() then
   local ok,detail=drives:grant(self.owed[1])
   if ok then self:log('bag: '..detail..' '..drives.loot:name(self.owed[1]));table.remove(self.owed,1)
   elseif detail=='bag full' then table.remove(self.owed,1);self:log('bag full: offered drive discarded') end
  end
  self:roll_wanted()
 end
 function H:run_end()
  if not self.running then return end
  self.running=false;self.owed={};self.fell={};self.rolls={};self.mods:run_end();self:log('run end: bag and build cleared')
 end
 return H
end
