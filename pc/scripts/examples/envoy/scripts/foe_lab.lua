-- LAB-only opponents; commands stage pure data, frame commits builds.
return function(D)
 local F={};F.__index=F
 local function clone(v) return D.mod_codec.decode(D.mod_codec.encode(v)) end
 function F.new(g,lab)
  local self=setmetatable({g=g,lab=lab,roller=D.foe_roll.new(D.mod_pool),seed=104729,stage=0,builds={},pending={},labels={}},F)
  g.command('foe',function(arg)return self:command(arg or '')end,'roll [strength or -] [seed] [CPU port] [normal|boss|finalboss] | clear | list | stand|fight [port]')
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
    local r=self.roller:roll(strength,seed,stage,p,D.mod_progression.context(self.lab.engine.context),w[5] or 'normal')
    self.lab.display:warm(self.lab.engine);assert(not self.lab.display.error,'shader warmup unavailable')
    self.pending[#self.pending+1]={op='roll',record=r};self.seed=seed;self.stage=stage
   else error('usage: foe roll|clear|list|stand|fight') end
   self.lab.enabled=true;if self.lab.options.activate then self.lab.options.activate() end
  end)
  if not ok then self.g.log('foe: refused '..tostring(why));return false,why end
  self.g.log('foe: '..arg..' accepted');return true
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
 end
 function F:apply()
  for _,e in ipairs(self.pending) do
   if e.op=='clear' then self:retire()
   else local r=e.record;self:cpu(r.port);local mods,implicit=self.roller:validate(r)
    self.lab.engine:set_build(r.port,mods,implicit);self.builds[r.port]=clone(r)
    local names,looks={},{};for slot=1,D.mod_progression.slots(r.build.context or r.context) do local rec=r.build.equipped[slot];if rec then names[#names+1]=self.roller.loot:name(rec);looks[#looks+1]={colour=rec.colour,rarity=rec.rarity} end end
    local keys={};for _,id in ipairs(r.build.keystones or {}) do keys[id]=true end;if r.build.keystone then keys[r.build.keystone]=true end
    for _,m in ipairs(self.lab.engine.list) do if keys[m.id] then names[#names+1]=m.label end end
    self.lab.engine.display.drive_build=self.lab.engine.display.drive_build or {};self.lab.engine.display.drive_build[r.port]=looks
    local role=r.role or 'normal';local factor=D.mod_progression.factor(r.context or r.build.context,role)
    local title=('P%d %s / %s %.2f / target %.2f x%.2f'):format(r.port,tostring((self.g.player(r.port) or {}).char_name or (self.g.player(r.port) or {}).name or 'CPU'),role,r.strength,r.target,factor)
    self.labels[r.port]={title=title,lines=names,left=240};self.g.log('foe: '..title..' / '..table.concat(names,', '))
   end
  end;self.pending={}
 end
 function F:frame() for p,label in pairs(self.labels) do label.left=label.left-1;if label.left<=0 then self.labels[p]=nil end end end
 function F:draw()
  if not self.g.kit then return end;local a=self.g.safe_area();local y=a.y+42;local ports={}
  for p=2,6 do if self.labels[p] then ports[#ports+1]=p end end;if #ports==0 then return end
  local shown=ports[math.floor(self.lab.engine.frame/45)%#ports+1];local l=self.labels[shown]
  local lines={};for _,text in ipairs(l.lines) do
   if D.drive_menu then for _,line in ipairs(D.drive_menu.wrap(self.g.kit,text,a.w-48)) do lines[#lines+1]=line end else lines[#lines+1]=text end
  end
  self.g.kit.panel(a.x+16,y,a.w-32,78);self.g.kit.text(a.x+24,y+19,l.title,'body','bone','left',{max_w=a.w-48})
  local pages=math.max(1,math.ceil(#lines/2));local page=math.floor((240-l.left)/45)%pages
  for n=1,2 do self.g.kit.text(a.x+24,y+23+n*19,lines[page*2+n] or (n==1 and 'Vanilla build' or ''),'body','bone','left',{max_w=a.w-48}) end
 end
 function F:snapshot() return clone{seed=self.seed,stage=self.stage,builds=self.builds,pending=self.pending,labels=self.labels} end
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
 function F:reset() self.builds={};self.pending={};self.labels={};self.seed=104729;self.stage=0 end
 return F
end
