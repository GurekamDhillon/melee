-- LAB-only opponents; commands stage pure data, frame commits builds.
return function(D)
 local F={};F.__index=F
 local function clone(v) return D.mod_codec.decode(D.mod_codec.encode(v)) end
 function F.new(g,lab)
  local self=setmetatable({g=g,lab=lab,roller=D.foe_roll.new(D.mod_pool),seed=104729,stage=0,builds={},pending={},labels={},drive=true,driver=D.foe_driver and D.foe_driver.new(g)},F)
  g.command('foe',function(arg)return self:command(arg or '')end,'roll [strength or -] [seed] [CPU port] [normal|boss|finalboss] | clear | list | stand|fight [port] | drive on|off|report')
  return self
 end
 function F:cpu(p)
  p=tonumber(p or 2);assert(p and p%1==0 and p>=2 and p<=6,'CPU port 2..6 required')
  local v=self.g.player(p);assert(v and v.cpu,'present CPU fighter required');return p
 end
 function F:command(arg)
  local ok,why=pcall(function()
   local allowed,reason=self.lab:allowed();assert(allowed,reason);assert(not self.lab:replaying(),'foe edit refused during rewind')
   local w={};for word in arg:gmatch('%S+') do w[#w+1]=word end
   if w[1]=='list' then
    assert(#w==1,'usage: foe list');for p=2,6 do local r=self.builds[p];if r then local mods=self.roller:validate(r);local names={};for _,m in ipairs(self.lab.engine.list) do if mods[m.id] then names[#names+1]=m.label..' T'..D.mod_schema.level(mods[m.id])..' x'..D.mod_schema.copies(mods[m.id]) end end;self.g.log(('foe: P%d strength %.2f / target %.2f / %s'):format(p,r.strength,r.target,table.concat(names,', '))) end end;return
   elseif w[1]=='sliced' then self.sliced=(w[2]=='on');return
   elseif w[1]=='held' then self.held=(w[2]=='on');self.g.log('foe: held cap '..tostring(self.held));return
   elseif w[1]=='drive' then
    assert(self.driver,'no opponent driver');if w[2]=='off' then self.drive=false;self.driver:reset() elseif w[2]=='on' then self.drive=true
    elseif w[2]=='player' then -- debug (balance harness): drive the player's own build too, at a chosen skill, so a CPU stands in for a skilled player
     local sk=tonumber(w[3] or .5);assert(sk and sk>=0 and sk<=1,'skill 0..1');self.drive=true
     local eq=self.lab.engine.equipped[1] or {};local ctx=self.lab.engine.context
     self.driver.foes[1]={want=D.foe_driver.wanted(eq),skill=sk,seed=tonumber(w[4] or 7),last_action=-1,cool=0,gap=0,context=ctx}
     self.driver.stats[1]={skill=sk,attempts={},events={},switches=0,opportunities={}};self.lab.enabled=true
    elseif w[2]=='force' then -- debug: drive every technique for a CPU at a chosen skill (rates are measured this way)
     local p=self:cpu(w[3]);local sk=tonumber(w[4] or .5);assert(sk and sk>=0 and sk<=1,'skill 0..1');self.drive=true
     self.driver.foes[p]={want={lcancel=true,wavedash=true,ps=true,tech=true},skill=sk,seed=tonumber(w[5] or 1),last_action=-1,cool=0,gap=0}
     self.driver.stats[p]={skill=sk,attempts={},events={},switches=0,opportunities={}};self.lab.enabled=true
    else for _,l in ipairs(self.driver:report()) do self.g.log(l) end end;return
   elseif w[1]=='stand' or w[1]=='fight' then
    assert(#w<=2,'usage: foe stand|fight [port]');local p=self:cpu(w[2]);assert(self.g.cpu_mode and self.g.cpu_mode(p,w[1]),'native CPU mode refused');return
   end
   if w[1]=='clear' then assert(#w==1,'usage: foe clear');self.pending={{op='clear'}}
   elseif w[1]=='roll' then
    assert(#self.pending<12,'foe pending queue full')
    for _,e in ipairs(self.lab.pending) do assert(e.port==1,'wait for pending CPU modifier edits to commit') end
    assert(#w<=5,'usage: foe roll [strength or -] [seed] [port] [role]');local p=self:cpu(w[4])
    local _,player=self.lab.engine:family_budget(1);local strength=(not w[2] or w[2]=='-') and player or tonumber(w[2]);local seed=tonumber(w[3] or self.seed)
    local m=self.g.match() or {};local stage=m.stage or m.stage_id or 0;assert(type(stage)=='number' and stage%1==0 and stage>=0,'numeric stage identity required')
    local held;if self.held and self.lab.drives then local b=self.lab.drives.bag;local n=0;for i=1,b:slots() do if b.equipped[i] then n=n+1 end end;held={drives=n,keystones=#(b.keystones or {})} end
    if self.sliced then self:roll_begin(p,strength,seed,stage,w[5] or 'normal',held);self.lab.enabled=true;return end -- debug: the search runs a few attempts per frame (the host's way), not in one call
    local r
    if held then local job=self.roller:roll_job(strength,seed,stage,p,D.mod_progression.context(self.lab.engine.context),w[5] or 'normal',held);repeat r=self.roller:roll_step(job,math.huge) until r
    else r=self.roller:roll(strength,seed,stage,p,D.mod_progression.context(self.lab.engine.context),w[5] or 'normal') end
    self.lab.display:warm(self.lab.engine);assert(not self.lab.display.error,'shader warmup unavailable')
    self.pending[#self.pending+1]={op='roll',record=r};self.seed=seed;self.stage=stage
   else error('usage: foe roll|clear|list|stand|fight') end
   self.lab.enabled=true;if self.lab.options.activate then self.lab.options.activate() end
  end)
  if not ok then self.g.log('foe: refused '..tostring(why));return false,why end
  self.g.log('foe: '..arg..' accepted');return true
 end
 -- Run adapter: the roll the console `foe roll` stages, minus the command parsing, in slices. A script call
 -- is limited to 2M instructions or 50 ms and a whole roll is more, so the run spends a few candidate builds
 -- per call. The CPU must already be present. The host's own frame warms the look shaders before publishing.
 function F:roll_begin(p,strength,seed,stage,role,held)
  assert(#self.pending<12,'foe pending queue full');self:cpu(p)
  self.jobs=self.jobs or {}
  self.jobs[p]={job=self.roller:roll_job(strength,seed,stage,p,D.mod_progression.context(self.lab.engine.context),role or 'normal',held),seed=seed,stage=stage}
 end
 function F:roll_advance(p,attempts)
  local j=self.jobs and self.jobs[p];if not j then return false end
  local ok,r=pcall(self.roller.roll_step,self.roller,j.job,attempts)
  if not ok then self.jobs[p]=nil;error(r,0) end
  if not r then return false end
  self.jobs[p]=nil;self.pending[#self.pending+1]={op='roll',record=r};self.seed=j.seed;self.stage=j.stage;self.lab.enabled=true
  return true
 end
 function F:retire()
  local engine=self.lab.engine
  for p in pairs(self.builds) do engine:clear(p);self.lab.display:clear(p);self.lab.debug_equipped[p]=nil;if engine.display.drive_build then engine.display.drive_build[p]=nil end end
  -- Provenance survives native chains; retire only statuses descended from these CPUs.
  for _,at in pairs(engine.statuses) do for name,v in pairs(at) do
   for _,line in ipairs(v.origin or {}) do if line:match('^Envoy foe P[2-6]$') then at[name]=nil;break end end
  end end
  local queue={};for _,e in ipairs(engine.queue) do if not self.builds[e.port] and not self.builds[e.target] then queue[#queue+1]=e end end;engine.queue=queue
  self.builds={};self.labels={}
  if self.driver then self.driver:reset() end
 end
 function F:apply()
  for _,e in ipairs(self.pending) do
   if e.op=='clear' then self:retire()
   elseif not (self.g.player(e.record.port) and self.g.player(e.record.port).cpu) then self.g.log('foe: roll for P'..tostring(e.record.port)..' dropped, the opponent is gone')
   else local r=e.record;self:cpu(r.port);local mods,implicit=self.roller:validate(r)
    self.lab.engine:set_build(r.port,mods,implicit);self.builds[r.port]=clone(r)
    if self.driver and self.drive then self.driver:set(r.port,mods,r.context or r.build.context,r.seed) end
    local names,looks={},{};for slot=1,D.mod_progression.slots(r.build.context or r.context) do local rec=r.build.equipped[slot];if rec then names[#names+1]=self.roller.loot:name(rec);looks[#looks+1]={colour=rec.colour,rarity=rec.rarity} end end
    local keys={};for _,id in ipairs(r.build.keystones or {}) do keys[id]=true end;if r.build.keystone then keys[r.build.keystone]=true end
    for _,m in ipairs(self.lab.engine.list) do if keys[m.id] then names[#names+1]=m.label end end
    local plain=D.drive_text and D.drive_text.build_lines(self.roller.loot,r.build,self.lab.engine.list) or names
    self.lab.engine.display.drive_build=self.lab.engine.display.drive_build or {};self.lab.engine.display.drive_build[r.port]=looks
    local role=r.role or 'normal';local factor=D.mod_progression.factor(r.context or r.build.context,role)
    local title=('P%d %s / %s %.2f / target %.2f x%.2f'):format(r.port,tostring(D.fighters and D.fighters.plate(self.g,(self.g.player(r.port) or {}).char_name or (self.g.player(r.port) or {}).name) or 'CPU'),role,r.strength,r.target,factor)
    local foe=(self.g.player(r.port) or {});local plate=('%s  (opponent strength %.1f)'):format(tostring(D.fighters and D.fighters.plate(self.g,foe.char_name or foe.name) or 'Opponent'),r.strength)
    if #plain==0 then plain={'No modifiers: a vanilla fighter'} end
    self.labels[r.port]={title=self.lab:hosted() and plate or title,lines=self.lab:hosted() and plain or names,left=240};self.g.log('foe: '..title..' / '..table.concat(names,', '))
   end
  end;self.pending={}
 end
 function F:frame()
  if self.sliced and self.jobs then for p in pairs(self.jobs) do local ok,why=pcall(self.roll_advance,self,p,2);if not ok then self.jobs[p]=nil;self.g.log('foe: roll refused '..tostring(why)) end end end
  if self.driver and self.drive and next(self.driver.foes) then local m=self.g.match();if m and m.active then self.driver:frame(m.frame) end end
  for p,label in pairs(self.labels) do label.left=label.left-1;if label.left<=0 then self.labels[p]=nil end end end
 function F:draw()
  if not self.g.kit then return end;local a=self.g.safe_area();local y=a.y+42;local ports={}
  for p=2,6 do if self.labels[p] then ports[#ports+1]=p end end;if #ports==0 then return end
  local shown=ports[math.floor(self.lab.engine.frame/45)%#ports+1];local l=self.labels[shown]
  -- Wrapping measures text through the kit: done once per label and width, not on every drawn frame.
  self.wrapped=self.wrapped or setmetatable({},{__mode='k'})
  local cached=self.wrapped[l];local lines
  if cached and cached.w==a.w then lines=cached.lines
  else
   lines={};for _,text in ipairs(l.lines) do
    if D.drive_menu then for _,line in ipairs(D.drive_menu.wrap(self.g.kit,text,a.w-48)) do lines[#lines+1]=line end else lines[#lines+1]=text end
   end
   self.wrapped[l]={w=a.w,lines=lines}
  end
  local per=self.lab:hosted() and 3 or 2
  self.g.kit.panel(a.x+16,y,a.w-32,40+per*19);self.g.kit.text(a.x+24,y+19,l.title,'body','bone','left',{max_w=a.w-48})
  local pages=math.max(1,math.ceil(#lines/per));local page=math.floor((240-l.left)/(per==3 and 70 or 45))%pages
  for n=1,per do self.g.kit.text(a.x+24,y+23+n*19,lines[page*per+n] or (n==1 and 'Vanilla build' or ''),'body','bone','left',{max_w=a.w-48}) end
 end
 -- `raw`: references instead of a detached copy, for a caller that only encodes the result (the per-frame checkpoint).
 function F:snapshot(raw) local s={seed=self.seed,stage=self.stage,builds=self.builds,pending=self.pending,labels=self.labels};if raw then return s end;return clone(s) end
 function F:validate(s)
  assert(type(s)=='table','invalid foe checkpoint');for k in pairs(s) do assert(({seed=true,stage=true,builds=true,pending=true,labels=true})[k],'unknown foe field') end
  assert(type(s.seed)=='number' and s.seed%1==0 and s.seed>=0 and s.seed<=2147483646,'invalid foe seed');assert(type(s.stage)=='number' and s.stage%1==0 and s.stage>=0 and s.stage<=2147483646,'invalid foe stage')
  assert(type(s.builds)=='table' and type(s.pending)=='table' and type(s.labels)=='table','invalid foe roots')
  for p,r in pairs(s.builds) do assert(p==r.port,'foe port mismatch');self.roller:validate(r) end
  local count=0;for i,e in pairs(s.pending) do assert(type(i)=='number' and i%1==0 and i>=1 and i<=12,'invalid foe pending index');count=count+1;assert(e.op=='clear' or e.op=='roll','invalid foe action');if e.op=='roll' then self.roller:validate(e.record) end end;assert(count==#s.pending,'sparse foe pending')
  for p,l in pairs(s.labels) do assert(s.builds[p] and type(l.title)=='string' and type(l.lines)=='table' and type(l.left)=='number' and l.left%1==0 and l.left>=1 and l.left<=240,'invalid foe label');for _,line in ipairs(l.lines) do assert(type(line)=='string','invalid foe label text') end end
  return clone(s)
 end
 function F:restore(s) s=self:validate(s);self.seed=s.seed;self.stage=s.stage;self.builds=s.builds;self.pending=s.pending;self.labels=s.labels end
 function F:reset() if self.driver then self.driver:reset() end;self.builds={};self.pending={};self.labels={};self.jobs={};self.seed=104729;self.stage=0 end
 return F
end
