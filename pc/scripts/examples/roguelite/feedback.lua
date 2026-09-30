-- Combat feedback policy + original kit HUD. Bundled lexically; no require/Core calls.
-- Callers supply observed Core abilities and committed events, never menu previews.
local F={capacity=6,pulse_frames=36}
local order={'assault','traversal','guard'}
local priorities={info=10,release=20,clear=60,upgrade=65,blocked=70,
 inheritance=80,error=90,failure=100}
local icons={info='focus',release='magic',upgrade='gene',clear='check',
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
local function surface(x,y,w,h,accent)
 gd.fill(x+3,y+3,w,h,0x030712b0);gd.fill(x,y,w,h,0x101a2cee)
 gd.fill(x,y,2,h,accent)
end
function F.draw(s,opts)
 opts=opts or {};local view=F.view(s);local reduced=opts.reduced==true or s.reduced
 if opts.hud~=false then
  -- Bottom rail replaces the large native stock cluster and the old three-row
  -- readiness panel. Percent and run lives come from runtime, never native 99 stocks.
  local x,y,w=12,407,254
  surface(x,y,w,61,0xf0b429ff)
  local player=opts.player or {}
  gd.kit.text(x+10,y+24,string.format('%.0f%%',player.percent or 0),'label','bone','left',{max_w=78,shear=0})
  text(x+95,y+17,'LIVES '..tostring(player.lives or 0),'gold',68)
  text(x+173,y+17,'ITEM '..tostring(player.supplies or 0),'muted',72)
  for i,slot in ipairs(order) do
   local a=view.slots[slot];local sx=x+9+(i-1)*81
   local accent=a and a.family=='frost' and 0x8fdef6ff or 0xff8050ff
   gd.kit.icon('rogue_'..slot,sx,y+35,.25,a and 'bone' or 'muted')
   local status=not a and '--' or a.remaining>0 and string.format('%.1fs',a.remaining/60)
    or a.ready and 'READY' or (number(a.charge)..'/'..number(a.cost))
   text(sx+19,y+47,status,a and a.ready and 'gold' or 'muted',54)
   gd.fill(sx,y+53,73,2,0x36445eff)
   if a and a.ratio>0 then gd.fill(sx,y+53,73*a.ratio,2,accent) end
   if a and a.ready then
    gd.fill(sx,y+56,73,1,0xf0b429ff)
    if not reduced and a.pulse>0 then gd.fill(sx,y+31,73*a.pulse,1,accent) end
   end
  end
  if opts.opponent then
   local enemy=opts.opponent
   surface(280,429,132,39,0xff8050ff)
   text(288,443,enemy.label or 'OPPONENT','muted',116)
   text(288,459,string.format('%.0f%%',enemy.percent or 0)..' / '..tostring(enemy.lives or 1)..' LEFT','bone',116)
  end
 end
 local e=view.notification
 if opts.notifications~=false and e then
  if opts.compact_menu then
   -- Paused workbenches reserve their subtitle strip, leaving actions visible.
   surface(26,63,588,23,0xf0b429ff)
   text(34,78,e.title..(e.detail~='' and (' / '..e.detail) or ''),e.kind=='error' and 'bone' or 'gold',570)
   return
  end
  local x,y,w,h=210,12,418,37
  local warning=e.kind=='error' or e.kind=='failure' or e.kind=='blocked'
  local accent=warning and 0xffab89ff or 0xf0b429ff
  surface(x,y,w,h,accent)
  gd.kit.icon('rogue_'..e.icon,x+9,y+8,.34,warning and 'bone' or 'gold')
  local suffix=e.count>1 and (' x'..e.count) or ''
  text(x+38,y+14,e.title..suffix,warning and 'bone' or 'gold',w-48)
  text(x+38,y+30,e.detail,'bone',w-48)
  if e.celebrate then
   -- Accents remain inside the notification strip, clear of the bottom HUD.
   local age=s.clock-e.created
   local progress=reduced and 1 or math.min(1,.2+age/24)
   gd.fill(x+38,y+h-2,(w-48)*progress,2,accent)
   if not reduced and age<36 then
    local t=age/36
    for i=1,3 do gd.fill(x+w-18-i*9,y+8+i*3-5*t,2,2,accent) end
   end
  end
 end
end
return F
