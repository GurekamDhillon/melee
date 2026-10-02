-- Combat feedback policy + original kit HUD. Bundled lexically; no require/Core calls.
-- Callers supply observed Core abilities and committed events, never menu previews.
local F={capacity=6,pulse_frames=36}
local order={'assault','traversal','guard'}
local priorities={tutorial=15,info=10,release=20,tell=55,clear=60,upgrade=65,blocked=70,
 inheritance=80,error=90,failure=100}
local icons={tutorial='focus',info='focus',release='magic',tell='guard',upgrade='gene',clear='check',
 blocked='guard',inheritance='gene',error='guard',failure='guard'}
local celebratory={upgrade=true,clear=true,inheritance=true}
local stats={{'potency','Power'},{'gain','Gain'},{'capacity','Charge cost'},
 {'reach','Reach'},{'cooldown','Cooldown'}}
local function finite(n) return type(n)=='number' and n==n and math.abs(n)<math.huge end
local function copy(t)
 if type(t)~='table' then return t end
 local out={} for k,v in pairs(t) do out[k]=copy(v) end return out
end
local function clean(s,n)
 if type(s)~='string' then return nil end
 s=s:gsub('[%z\1-\31]',' ')
 if #s==0 then return nil end
 return s:sub(1,n)
end
local function number(n) return string.format('%.3g',n) end
local function purge(s)
 for i=#s.queue,1,-1 do if s.queue[i].expires<=s.clock then table.remove(s.queue,i) end end
end
local function sort(s)
 table.sort(s.queue,function(a,b)
  if a.priority~=b.priority then return a.priority>b.priority end
  return a.serial<b.serial
 end)
end
function F.new(opts)
 local s={reduced=opts and opts.reduced==true or false}
 F.reset(s);return s
end
function F.reset(s)
 s.clock=0;s.serial=0;s.queue={};s.slots={};s.paused=false
end
-- Enqueue lifetime includes time spent waiting behind a higher priority notice.
-- Identical keys coalesce without extending expiry: refusal spam cannot starve a queue.
function F.notify(s,event)
 if type(event)~='table' or not priorities[event.kind or 'info'] then return nil,'invalid kind' end
 local kind=event.kind or 'info';local title=clean(event.title,128)
 if not title then return nil,'missing title' end
 local key=clean(event.key,96) or title
 local ttl=event.ttl or 240
 if not finite(ttl) or ttl%1~=0 or ttl<1 or ttl>3600 then return nil,'invalid lifetime' end
 purge(s)
 for _,e in ipairs(s.queue) do
  if e.key==key and e.kind==kind then
   e.title=title;e.detail=clean(event.detail,256) or '';e.count=math.min(99,e.count+1)
   -- Retain arrival time, priority and lifetime; new payload is copied, never aliased.
   if event.deltas then e.deltas=copy(event.deltas) end
   return true,'coalesced'
  end
 end
 sort(s)
 if #s.queue>=F.capacity then
  if priorities[kind]<=s.queue[#s.queue].priority then return false,'queue full' end
  table.remove(s.queue)
 end
 s.serial=s.serial+1
 s.queue[#s.queue+1]={key=key,kind=kind,title=title,detail=clean(event.detail,256) or '',
  priority=priorities[kind],created=s.clock,expires=s.clock+ttl,serial=s.serial,
  count=1,icon=icons[kind],celebrate=celebratory[kind]==true,deltas=copy(event.deltas)}
 sort(s);return true
end
function F.dismiss(s,key)
 for i=#s.queue,1,-1 do if key==nil or s.queue[i].key==key then table.remove(s.queue,i) end end
end
-- Tutorial and enemy-tell delivery. Both are ordinary queued notices: they
-- coalesce by key, honour lifetimes, and can never preempt a warning because
-- their priorities sit below clear/blocked/error/failure. A tutorial hint is
-- never allowed to pause combat; the caller only forwards this to F.notify.
function F.tutorial(s,step)
 if step==nil then return nil,'no tutorial step' end
 if type(step)=='string' then step={title=step,key='tutorial',detail=''} end
 return F.notify(s,{key=step.key or 'tutorial',kind='tutorial',
  title=step.title,detail=step.detail or '',ttl=step.ttl or 480})
end
function F.tell(s,event)
 if type(event)~='table' or not clean(event.key,96) then return nil,'missing tell identity' end
 return F.notify(s,{key=event.key,kind='tell',title=clean(event.title,96) or 'INCOMING',
  detail=clean(event.detail,256) or 'Move or shield before it lands',ttl=event.ttl or 120})
end
local function observation(a,id)
 if type(a)~='table' or not clean(a.name,96) or not finite(a.charge) or a.charge<0
  or not finite(a.cost) or a.cost<=0 or not finite(a.remaining) or a.remaining<0 then return nil end
 return {id=id or a.name..':'..tostring(a.family),name=clean(a.name,96),family=a.family,
  charge=a.charge,cost=a.cost,remaining=a.remaining,ready=a.ready==true,
  ratio=math.min(1,a.charge/a.cost)}
end
-- frames comes from one deduplicated simulation hook; draw never advances time.
-- Paused calls may refresh loadout observations but cannot trigger cosmetic readiness.
function F.update(s,abilities,frames,paused,ids)
 frames=frames or 0
 assert(finite(frames) and frames%1==0 and frames>=0 and frames<=36000,'invalid feedback tick')
 assert(type(abilities)=='table','missing abilities')
 s.paused=paused==true
 if not s.paused then s.clock=s.clock+frames end
 purge(s)
 for _,slot in ipairs(order) do
  local a=observation(abilities[slot],ids and ids[slot]);local old=s.slots[slot]
  if a then
   if old and old.id==a.id then
    if a.ready and not old.ready and not s.paused then a.pulse_at=s.clock
    elseif a.ready and old.ready then a.pulse_at=old.pulse_at end
   end
   s.slots[slot]=a
  else s.slots[slot]=nil end
 end
end
-- The caller has already committed Core.reward and supplies two Core.resolve copies.
function F.reward(s,event)
 if type(event)~='table' or type(event.before)~='table' or type(event.after)~='table' then
  return nil,'missing committed comparison'
 end
 local deltas,parts={},{}
 for _,stat in ipairs(stats) do
  local key,label=stat[1],stat[2];local a,b=event.before[key],event.after[key]
  if finite(a) and finite(b) and a~=b then
   deltas[#deltas+1]={stat=key,before=a,after=b,delta=b-a}
   local unit=key=='cooldown' and 'f' or ''
   parts[#parts+1]=label..' '..number(a)..unit..' > '..number(b)..unit
  end
 end
 if #deltas==0 then
  return F.notify(s,{key=event.key,kind='info',title='No effective stat change',
   detail=clean(event.name,96) or 'Resolved values unchanged'})
 end
 return F.notify(s,{key=event.key,kind='upgrade',title=clean(event.name,96) or 'Run upgrade applied',
  detail=table.concat(parts,' | '),deltas=deltas,ttl=300})
end
function F.clear(s,event)
 if type(event)~='table' or not clean(event.key,96) then return nil,'missing encounter identity' end
 return F.notify(s,{key=event.key,kind='clear',title='ENCOUNTER CLEARED',
  detail=(clean(event.title,96) or 'Passage')..(event.reward==true and ' / Choose a reward' or ' / Path open'),ttl=300})
end
function F.finish(s,result)
 if type(result)~='table' then return nil,'missing finish result' end
 if result.outcome=='failure' then
  return F.notify(s,{key='finish',kind='failure',title='RUN ENDED',
   detail='Collection retained / Temporary run upgrades lost',ttl=360})
 elseif result.outcome=='success' then
  local exported=clean(result.export,96)
  return F.notify(s,{key='finish',kind='inheritance',title='RUN COMPLETE',
   detail=exported and ('Inherited '..exported..' added / Temporary upgrades excluded')
    or 'Collection retained / No gene exported',ttl=360})
 end
 return nil,'invalid finish outcome'
end
-- Read-only diagnostics; copy so a menu or test cannot modify retained observations.
function F.view(s)
 local out={clock=s.clock,paused=s.paused,slots={},queued=#s.queue,notification=copy(s.queue[1])}
 for _,slot in ipairs(order) do
  local a=copy(s.slots[slot])
  if a then
   a.pulse=not s.reduced and a.pulse_at and math.max(0,1-(s.clock-a.pulse_at)/F.pulse_frames) or 0
   a.status=a.ready and 'READY' or a.remaining>0 and 'COOLDOWN' or 'CHARGING'
   out.slots[slot]=a
  end
 end
 return out
end
local function text(x,y,label,color,w)
 gd.kit.text(x,y,label,'caption',color or 'bone','left',{max_w=w,shear=0})
end
-- The drop shadow is inset so every fill stays strictly inside the declared
-- box [x,x+w] x [y,y+h]; overlapped HUD boxes can then be checked for exact
-- ownership without a shadow crossing a neighbour.
local function surface(x,y,w,h,accent)
 gd.fill(x+3,y+3,math.max(0,w-3),math.max(0,h-3),0x030712b0);gd.fill(x,y,w,h,0x101a2cee)
 gd.fill(x,y,math.max(1,math.min(2,w)),h,accent)
end
function F.draw(s,opts)
 opts=opts or {};local view=F.view(s);local reduced=opts.reduced==true or s.reduced
 local lay=opts.layout
  -- One notification routine shared by the fallback and compact/new-HUD paths.
  -- Compact mode draws a single line whose baseline plus caption height stays
  -- inside the declared strip; normal mode draws title and detail and relies on
  -- the layout's note height being large enough for both.
  local function draw_notification()
   local e=view.notification
   if opts.notifications==false or not e then return end
   local nx,ny,nw,nh,scale
   if opts.compact_menu then
    if lay then local n=lay.compact_notification or lay.notification;nx,ny,nw,nh=n.x,n.y,n.w,n.h;scale=lay.compact_scale or lay.scale
    else nx,ny,nw,nh,scale=26,63,588,23,1 end
    local sc=function(n) return math.floor(n*scale+0.5) end
    local warning=e.kind=='error' or e.kind=='failure' or e.kind=='blocked'
    surface(nx,ny,nw,nh,warning and 0xffab89ff or 0xf0b429ff)
    local ty=ny+math.max(0,nh-sc(14))
    text(nx+sc(8),ty,e.title..(e.detail~='' and (' / '..e.detail) or ''),warning and 'bone' or 'gold',nw-sc(16))
    return
   end
   if lay then local n=lay.notification;nx,ny,nw,nh=n.x,n.y,n.w,n.h;scale=lay.note_scale or lay.scale
   else nx,ny,nw,nh,scale=210,12,418,37,1 end
   local sc=function(n) return math.floor(n*scale+0.5) end
   local warning=e.kind=='error' or e.kind=='failure' or e.kind=='blocked'
   local accent=warning and 0xffab89ff or 0xf0b429ff
   surface(nx,ny,nw,nh,accent)
   gd.kit.icon('rogue_'..e.icon,nx+sc(9),ny+sc(8),.34*scale,warning and 'bone' or 'gold')
   local suffix=e.count>1 and (' x'..e.count) or ''
   text(nx+sc(38),ny+sc(14),e.title..suffix,warning and 'bone' or 'gold',nw-sc(48))
   text(nx+sc(38),ny+sc(30),e.detail,'bone',nw-sc(48))
   if e.celebrate then
    local age=s.clock-e.created
    local progress=reduced and 1 or math.min(1,.2+age/24)
    gd.fill(nx+sc(38),ny+nh-sc(2),math.max(0,nw-sc(48))*progress,math.max(1,sc(2)),accent)
    if not reduced and age<36 then
     local t=age/36
     for i=1,3 do gd.fill(nx+nw-sc(18)-i*sc(9),ny+sc(8)+i*sc(3)-5*t,math.max(1,sc(2)),math.max(1,sc(2)),accent) end
    end
   end
  end
  if opts.hud~=false then
   local player=opts.player or {}
   if lay and lay.replace_vanilla==false then
    -- The native stock/percent cluster stays: draw only minimal readable
    -- life/damage anchors and the layout's opponent block, never a large panel.
    local scale=lay.content_scale or lay.scale
    local sc=function(n) return math.floor(n*scale+0.5) end
    local function anchor(r,label,color)
     gd.fill(r.x+sc(2),r.y+sc(2),math.max(0,r.w-sc(2)),math.max(0,r.h-sc(2)),0x030712b0)
     gd.fill(r.x,r.y,r.w,r.h,0x101a2cee);gd.fill(r.x,r.y,math.max(1,math.min(2,r.w)),r.h,0xf0b429ff)
     gd.kit.text(r.x+sc(6),r.y+sc(4),label,'caption',color,'left',{max_w=math.max(1,r.w-sc(10)),shear=0})
    end
    anchor(lay.fallback.lives,'LIVES '..tostring(player.lives or 0),'gold')
    anchor(lay.fallback.damage,string.format('%.0f%%',player.percent or 0),'bone')
    if opts.opponent then
     local enemy=opts.opponent;local o=lay.opponent;local os=lay.opponent_scale or lay.scale
     local osc=function(n) return math.floor(n*os+0.5) end
     gd.kit.text(o.x+osc(6),o.y+osc(6),enemy.label or 'OPPONENT','caption','muted','left',{max_w=math.max(1,o.w-osc(12)),shear=0})
     gd.kit.text(o.x+osc(6),o.y+osc(24),string.format('%.0f%%',enemy.percent or 0)..' / '..tostring(enemy.lives or 1)..' LEFT','caption','bone','left',{max_w=math.max(1,o.w-osc(12)),shear=0})
    end
   else
    -- Compact rail. Content scale and box both come from the responsive layout,
    -- so the bars always fit inside the rail at every dpi/aspect combination.
    local scale=lay and (lay.content_scale or lay.scale) or 1
    local sc=function(n) return math.floor(n*scale+0.5) end
    local rx,ry,rw,rh
    if lay then local rail=lay.rail;rx,ry,rw,rh=rail.x,rail.y,rail.w,rail.h
    else rx,ry,rw,rh=12,407,254,61 end
    surface(rx,ry,rw,rh,0xf0b429ff)
    gd.kit.text(rx+sc(10),ry+sc(24),string.format('%.0f%%',player.percent or 0),'label','bone','left',{max_w=sc(78),shear=0})
    text(rx+sc(95),ry+sc(17),'LIVES '..tostring(player.lives or 0),'gold',sc(68))
    text(rx+sc(173),ry+sc(17),'ITEM '..tostring(player.supplies or 0),'muted',sc(72))
    for i,slot in ipairs(order) do
     local a=view.slots[slot];local sx=rx+sc(9)+(i-1)*sc(81)
     local accent=a and a.family=='frost' and 0x8fdef6ff or 0xff8050ff
     gd.kit.icon('rogue_'..slot,sx,ry+sc(35),.25*scale,a and 'bone' or 'muted')
     local status=not a and '--' or a.remaining>0 and string.format('%.1fs',a.remaining/60)
      or a.ready and 'READY' or (number(a.charge)..'/'..number(a.cost))
     text(sx+sc(19),ry+sc(47),status,a and a.ready and 'gold' or 'muted',sc(54))
     gd.fill(sx,ry+sc(53),sc(73),sc(2),0x36445eff)
     if a and a.ratio>0 then gd.fill(sx,ry+sc(53),sc(73)*a.ratio,sc(2),accent) end
     if a and a.ready then
      gd.fill(sx,ry+sc(56),sc(73),sc(1),0xf0b429ff)
      if not reduced and a.pulse>0 then gd.fill(sx,ry+sc(31),sc(73)*a.pulse,sc(1),accent) end
     end
    end
    if opts.opponent then
     local enemy=opts.opponent;local o=lay and lay.opponent
     local os=lay and (lay.opponent_scale or lay.scale) or 1
     local osc=function(n) return math.floor(n*os+0.5) end
     local ox,oy,ow,oh
     if o then ox,oy,ow,oh=o.x,o.y,o.w,o.h
     else ow,oh=osc(132),osc(39);ox,oy=rx+rw+osc(12),ry+rh-oh end
     surface(ox,oy,ow,oh,0xff8050ff)
     text(ox+osc(8),oy+osc(14),enemy.label or 'OPPONENT','muted',ow-osc(16))
     text(ox+osc(8),oy+osc(30),string.format('%.0f%%',enemy.percent or 0)..' / '..tostring(enemy.lives or 1)..' LEFT','bone',ow-osc(16))
    end
   end
  end
  draw_notification()
end
return F
