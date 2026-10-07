-- Native kit draws all framing, typography and list rows. No replacement widgets.
return function(D)
 local C=D and D.companion
 local V={};local stats={'power','speed','guard','jump'}
 local traits={'power','speed','guard','jump','colour','two_tone','shiny'}
 local function label(name) return (C and C.tuning.stat_names or {})[name] or name end
 local function val(v) if v==nil then return '?' end;return tostring(v) end
 function V.draw(g,s,c)
  if s.atlas then return end   -- an Atlas screen stands for this one (atlas_kit.menu_show)
  c=c or {};local a=g.safe_area();local k=g.kit
  local w=math.min(a.w-40,740);local x=a.x+(a.w-w)/2;local y=a.y+20
  local function text(t,dy,color) k.text(x+20,y+dy,t,'body',color or 'bone','left',{max_w=w-40}) end
  if s.screen=='playing' then
   k.panel(x,y,w,80);text('ENVOY | '..s.fighter,28);text(c.hud or 'START: pause',55);return
  end
  k.panel(x,y,w,440);text('ENVOY / '..s.screen:upper(),30)
  local list_y=y+60;local list_h=330
  if s.screen=='profile' and c.error then
   text('Profile refused: '..tostring(c.error),65);text('Close Envoy and repair the profile before continuing.',91);list_y=y+120;list_h=240
  elseif s.screen=='companion' then
   local companion=c.companion or {}
   local remaining=math.max(0,((C and C.tuning.lifespan) or 6)-(companion.age or 0))
   text('Age '..val(companion.age)..' | '..remaining..' runs remaining | Type '..val(label(companion.type)),57)
   local passive=C and C.passive and C.passive(companion) or c.passive
   text('Passive: '..val(type(passive)=='table' and passive.name or passive or 'None yet'),78)
   for i,name in ipairs(stats) do
    local stat=(companion.stats or {})[name] or {};local points=stat.points or 0
    local fraction,current,needed=0,0,0
    if C and C.progress and stat.points and stat.level then fraction,current,needed=C.progress(stat) end
    local fill=math.floor(math.max(0,math.min(1,fraction or 0))*10);local bar=string.rep('|',fill)..string.rep('.',10-fill)
    local progress=needed==0 and 'MAX' or tostring(current)..'/'..tostring(needed)
    text(label(name)..'  '..val(stat.grade)..'  Lv '..val(stat.level)..'  ['..bar..'] '..progress..' ('..points..' points)',80+i*22)
   end
   local palette=c.palette or {};local sw=math.min(130,(w-40)/math.max(1,#palette))
   for i,p in ipairs(palette) do
    k.panel(x+20+(i-1)*sw,y+178,sw-6,26,{fill=p.rgba,piece=3})
    k.text(x+22+(i-1)*sw,y+196,p.label or 'colour','body','bone','left',{max_w=sw-10})
   end
   local lines={};for _,name in ipairs(traits) do
    local pair=(companion.dna or {})[name]
    if pair then lines[#lines+1]=label(name)..': '..val(pair[1])..' / '..val(pair[2]) end
   end
   local first=math.min(s.scroll+1,math.max(1,#lines-5))
   for i=first,math.min(#lines,first+5) do text(lines[i],226+(i-first)*21) end
   if #lines>6 then text('DNA: Up / Down scroll',363) end
   list_y=y+374;list_h=30
  elseif s.screen=='reward' then
   local r=c.reward or {};text(r.final and 'FINAL CLEAR / larger drive reward' or 'STAGE CLEAR / choose a drive',65)
   if r.preview then
    for i,name in ipairs(stats) do
     local v,fraction,current,needed,moment=D.hud.stat_view({stats=r.after or {}},name,r)
     local dy=76+i*34;local bar=w-40
     local progress=needed==0 and 'MAX' or tostring(current)..'/'..tostring(needed)
     text(label(name)..' '..val(v.grade)..' L'..val(v.level)..' '..progress..(moment and ' LEVEL UP!' or ''),dy,moment and 0xEBD175FF or 'bone')
     g.fill(x+20,y+dy+7,bar,5,0x44525CFF)
     g.fill(x+20,y+dy+7,bar*fraction,5,D.hud.colours[({'red','green','blue','yellow'})[i]])
     if moment then g.fill(x+20,y+dy+6,bar,7,0xEBD17560) end
    end
    list_y=y+250;list_h=80
   else
    text('Choose one. Growth is kept through continues and game over.',93)
    if r.error then text('SAVE PENDING: '..tostring(r.error)..' | A retries',123,'danger') end
    list_y=y+160;list_h=120
   end
  elseif s.screen=='results' or s.screen=='interlude' then
   local r=s.screen=='results' and (s.result or {}) or (s.interlude_data or {})
   text(s.screen=='results' and ('Boss: '..val(r.boss)) or ('Next: '..val(r.theme or r.next_theme)),65)
   local before=(r.before or {}).stats or r.before or {};local after=(r.after or {}).stats or r.after or {}
   for i,name in ipairs(stats) do
    local b=before[name];local a=after[name]
    local line=label(name)..': drives +'..val((r.drives or {})[name] or 0)..' | levels +'..val((r.levels or {})[name] or 0)
    if b and a then line=label(name)..': '..val(b.grade)..' -> '..val(a.grade)..' | L'..val(b.level)..' -> L'..val(a.level)..' | points '..val(b.points)..' -> '..val(a.points) end
    text(line,86+i*26)
   end
   if r.evolution then
    local e=r.evolution;local kind=type(e)=='table' and (e.type or e.to) or e
    text('EVOLVED: '..val(label(kind)),230)
    local passive=type(e)=='table' and e.passive or r.passive
    if not passive and C and C.passive then passive=C.passive({type=kind}) end
    if type(passive)=='table' then passive=passive.name end
    text('Passive: '..val(passive),255)
   end
   list_y=y+290;list_h=90
  elseif s.screen=='records' then
   local lines=c.records or {};if #lines==0 then lines={'No completed runs yet.'} end
   local first=math.min(s.scroll+1,math.max(1,#lines-9))
   for i=first,math.min(#lines,first+9) do text(lines[i],65+(i-first+1)*24) end
   if #lines>10 then text('Records: Up / Down scroll',332) end
   list_y=y+340;list_h=60
  elseif s.screen=='hub' and c.notice then text(tostring(c.notice),65);list_y=y+90;list_h=290
  elseif s.screen=='setup' then
   text('Fighter: '..s.fighter..' | Classic / Adventure + NG+',61);list_y=y+85;list_h=290
   if c.notice then text(tostring(c.notice),85);list_y=y+112;list_h=260 end
  end
  local entries=s:entries(c);local focus=s.focus[s.screen] or 1
  local visible=math.max(1,math.floor(list_h/30));local first=math.max(1,focus-visible+1)
  k.list(x+20,list_y,w-40,entries,#entries>0 and focus or 0,{pitch=30,h=30,first=first,visible=math.min(visible,#entries)})
  if s.screen=='reward' and c.reward and not c.reward.preview then
   for i,v in ipairs(c.reward.options or {}) do
    local colour=D.hud and D.hud.colours[v.colour] or 0xFFFFFFFF
    k.text(x+w-64,list_y+(i-1)*30+21,'<>','body',colour,'left',{max_w=32})
   end
  end
  text('A: select    B: back    D-pad / stick: navigate',424)
 end
 return V
end


