-- Envoy's match HUD on gd.ui.hud (Atlas step 3; spec 13.3). Presentation only: it reads what the legacy HUD (run_hud.lua), the rule host
-- (run_host.lua) and the opponent plates (foe_lab.lua) already keep, and describes it; the engine places it (zones in the title-safe box, clear of
-- the retail percent and timer), draws it under any screen and never lets it take focus.
--
--   top_left    the build strip (slot pips, keystone stones, "n waiting"); seat 1's in co-op
--   top_right   the synergy toast (a SMALL corner toast, never a banner; gd.ui.toast), then the opponent cards (at most three, two in co-op, where
--               seat 2's strip shares the zone); seat 2's strip in co-op
--   top_center  one banner at most: "Collect the drives" with the A glyph, or, during the payout, "Hold Z + Down to leave" with its progress
--   bottom_left the pickup note, one line, four seconds (also the out-of-bounds notice)
--
-- GONE: the starter announcement's centred panel (the starter drive is shown once as a "RUN START" toast in the seat's corner) and the crit
-- pop-up (there is no HUD part for a crit: nothing in this file reads a crit). STAYS: the world-space link flashes and emblem pops (synergy_fx.lua)
-- and the floor arrow (run_host.lua draw_hold). The strength figure and the depth line are developer figures and stay in the legacy draw
-- (`envoy devui on` draws the legacy HUD in full).
--
-- THE DESCRIPTION IS REBUILT ONLY WHEN SOMETHING CHANGED (a key of the strip, the cards, the banner and the note), never per frame. One HUD per
-- script (id "envoy.hud"): in co-op both seats' parts are merged into it. Envoy never calls gd.ui.retail_hide: its first HUD hides nothing retail.
return function(D)
 local K=D.atlas_kit
 local U={seats={},key=nil,shown={}}
 local ID='envoy.hud'
 local EMPTY_FILL,EMPTY_RING=0x161D23FF,0x59656FFF

 local function port_of(host) return (host.seat and host.seat.port) or 1 end
 local function coop(host) return host.seat~=nil end
 -- the corner a seat's strip and toast go to: solo strip top_left, solo toast top_right; co-op seat 1 left, seat 2 right
 local function strip_zone(host) if coop(host) and host.seat.index%2==0 then return 'top_right' end;return 'top_left' end
 local function toast_zone(host) if not coop(host) then return 'top_right' end;return strip_zone(host) end

 -- Atlas draws this HUD: gd.ui.hud exists, the switch is on, and the developer overlay is off (it keeps the legacy draw whole)
 function U.on(host)
  local g=host.g
  if not K.enabled(g) or type(g.ui.hud)~='function' then return false end
  if D.run_hud and D.run_hud.dev_ui and D.run_hud.dev_ui() then return false end
  return true
 end

 local function text_of(l) if type(l)=='table' then return tostring(l.text or '') end;return tostring(l or '') end
 local function rgba_of(arch) return arch and arch.colour and ((arch.colour<<8)|255) or nil end

 local function strip_part(m)
  local pips,keys={},{}
  for i,p in ipairs(m.pips) do pips[i]=p and {fill=p.colour,ring=p.border} or {fill=EMPTY_FILL,ring=EMPTY_RING} end
  for _,kc in ipairs(m.keys) do keys[#keys+1]={letter=kc.letter,rgba=kc.colour} end
  return {kind='strip',pips=pips,keys=keys,wait=(m.waiting or 0)>0 and (('%d waiting'):format(m.waiting)) or nil}
 end

 -- the opponent cards: each opponent's plate (name, then its keystones and a rule count), at most three
 local function cards_of(host,limit)
  local foes=host.mods and host.mods.foes;local out={}
  if not foes or type(foes.labels)~='table' then return out end
  local ports={};for p in pairs(foes.labels) do ports[#ports+1]=p end;table.sort(ports)
  for _,p in ipairs(ports) do
   if #out>=limit then break end
   local l=foes.labels[p];local lines={}
   for i=1,math.min(#l.lines,2) do lines[i]=text_of(l.lines[i]) end
   out[#out+1]={kind='card',title=text_of(l.title),lines=lines}
  end
  return out
 end

 local function banner_of(host)
  if not (host.holding_end and host.hold_banner) then return nil end
  if host.paying then
   local pr=host.leave_w and host.leave_w.progress and host.leave_w:progress() or 0
   return {kind='banner',text='Hold Z + Down to leave',button='Z',progress=math.floor(pr*20)/20}   -- 5 percent steps: the description is not rebuilt per frame
  end
  return {kind='banner',text='Collect the drives',button='A'}
 end

 -- one line, four seconds: the pickup, or the out-of-bounds notice
 local function note_of(host)
  local hd=host.hud
  if hd.flash_left>0 and hd.flash_always and hd.flash_text then return {kind='note',text=tostring(hd.flash_text),seconds=4} end
  local c=hd.card
  if c then return {kind='note',text=c.title..(c.lines[1] and (': '..text_of(c.lines[1])) or ''),seconds=4} end
  return nil
 end

 -- toasts: the synergy notice and the run start. Each one is sent once (marked on the legacy item).
 local function send_toasts(host)
  local g=host.g;local hd=host.hud;local zone=toast_zone(host)
  local c=hd.corners[1]
  if c and not c.atlas_sent then
   c.atlas_sent=true
   local title=text_of(c.lines[1]):upper();local blurb=text_of(c.lines[2])
   pcall(g.ui.toast,{zone=zone,title=title,text=blurb,rgba=rgba_of(c.arch),seconds=4})
  end
  local t=hd.toasts[1]
  if t and not t.atlas_sent then
   t.atlas_sent=true
   if text_of(t.lines[1])=='Your starter drive' then   -- the starter announcement: a toast in the seat's corner, not a centred panel
    pcall(g.ui.toast,{zone=zone,title='RUN START',text=text_of(t.lines[3]~=nil and t.lines[3] or t.lines[2]),seconds=4})
   end
  end
 end

 -- this host's contribution to the HUD, and its key (what changed)
 local function contribution(host)
  local hd=host.hud;local m=hd:model()
  if not m then return nil end
  local p={port=port_of(host),strip_zone=strip_zone(host),coop=coop(host)}
  p.strip=strip_part(m)
  p.cards=cards_of(host,coop(host) and 2 or 3)
  p.banner=banner_of(host);p.note=note_of(host)
  local k={m.key,#m.pips,tostring(m.waiting)}
  for _,c in ipairs(p.cards) do k[#k+1]=c.title..'/'..table.concat(c.lines,'/') end
  if p.banner then k[#k+1]='b:'..p.banner.text..':'..tostring(p.banner.progress) end
  if p.note then k[#k+1]='n:'..p.note.text end
  p.key=table.concat(k,'|')
  return p
 end

 local function merged(seats)
  local ports={};for port in pairs(seats) do ports[#ports+1]=port end;table.sort(ports)
  local zones={top_left={},top_center={},top_right={},bottom_left={}}
  local banner,note
  for _,port in ipairs(ports) do
   local s=seats[port]
   local z=zones[s.strip_zone];z[#z+1]=s.strip
   banner=banner or s.banner;note=note or s.note
  end
  for _,port in ipairs(ports) do   -- the cards after the strips, in the right zone
   local s=seats[port];local z=zones.top_right
   for _,c in ipairs(s.cards) do if #z<4 and #zones.top_right<4 then z[#z+1]=c end end
   break   -- one set of opponent cards: the first seat's (both seats fight the same opponents)
  end
  if banner then zones.top_center[1]=banner end
  if note then zones.bottom_left[1]=note end
  return zones
 end

 -- (re)describe the HUD when something changed
 function U.sync(host)
  local g=host.g
  if not U.on(host) then
   if U.shown[g] then U.clear_all(g) end
   return false
  end
  local c=contribution(host)
  if not c then return false end
  send_toasts(host)
  U.seats[c.port]=c
  local parts={};for port,s in pairs(U.seats) do parts[#parts+1]=port..'='..s.key end;table.sort(parts)
  local key=table.concat(parts,';')
  if key==U.key then return true end
  local ok,err=pcall(g.ui.hud,{id=ID,zones=merged(U.seats)})
  if not ok then K.log_once({host=host},'the HUD description was refused: '..tostring(err),'envoy atlas hud');return false end
  U.key=key;U.shown[g]=true
  return true
 end

 -- a host is gone (the run ended): its seat's parts leave; the last one takes the HUD down
 function U.clear(host)
  U.seats[port_of(host)]=nil
  if next(U.seats)==nil then U.clear_all(host.g) else U.key=nil end
 end
 function U.clear_all(g)
  U.seats={};U.key=nil;U.shown[g]=nil
  if type(g.ui)=='table' and type(g.ui.hud_clear)=='function' then pcall(g.ui.hud_clear) end
 end
 -- the test and scene hook: forget every cache (a new scene's HUD is described afresh)
 function U.reset() U.seats={};U.key=nil;U.shown={} end

 return U
end
