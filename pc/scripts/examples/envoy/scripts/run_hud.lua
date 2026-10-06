-- Always-on build strip, announcements and the pickup note for a rule-host run. Everything it shows is a small
-- model built when the bag, the context or an announcement changes; the per-frame draw only replays it.
--
-- WHAT A MATCH SHOWS (gameplay UI audit, 2026-10-05): the strip is the slot squares and the keystone letter cells, nothing else (the strength
-- percentage and the depth text are developer figures: `envoy devui on`; the between-stage screens carry them for a player). No flash text under
-- the strip except the out-of-bounds notice. The synergy message is a SMALL top-corner note, not a banner. A pickup is a one-line note. Teaching
-- and error toasts are logged always and shown only with the developer overlay on (mod_tuning.dev_ui).
return function(D)
 local Hd={};Hd.__index=Hd
 Hd.tuning={flash_frames=90,announce_frames=360,card_frames=150,corner_frames=240,anchor='top-left'}
 local function T() return D.drive_text end
 -- The one predicate for developer-only overlays (off by default): `envoy devui on|off`, or the ENVOY_DEVUI environment flag where the mod can read it.
 function Hd.dev_ui() return D.mod_tuning~=nil and D.mod_tuning.dev_ui~=nil and D.mod_tuning.dev_ui()==true end
 function Hd.new(g,host) return setmetatable({g=g,host=host,flash_left=0,flash_text=nil,toasts={},corners={},card=nil,card_left=0,last_trace=nil},Hd) end
 -- Rebuilt when the bag (rev), the progression or the engine changes.
 function Hd:model()
  local h=self.host;local d=h.mods.drives;if not d then return nil end
  local dev=Hd.dev_ui()
  local b=d.bag;local key=table.concat({d.rev or 0,b.context.depth,b.context.loop,#b.items,tostring(b.equipped),#h.decide,h.loop or 0,dev and 'dev' or 'play'},'|')
  if self.m and self.m.key==key then return self.m end
  local pips={};for slot=1,b:slots() do local r=b.equipped[slot];pips[slot]=r and {colour=T().base_colour[r.colour],border=T().rarity_colour[r.rarity]} or false end
  -- Keystones are small cells (an initial letter on the family colour), never a name list.
  local keys={};for _,id in ipairs(h:keystone_ids()) do local r=h:keystone_rule(id);local fam=D.keystones.family(id)
   keys[#keys+1]={letter=(r and r.label or id):sub(1,1):upper(),colour=T().base_colour[fam] or 0xEBD175FF} end
  local t=h:totals()
  self.m={key=key,pips=pips,strength=t.strength,keys=keys,depth=b.context.depth,loop=h.loop or b.context.loop,bag=#b.items,label=nil,dev=dev,waiting=#h.decide}
  local k=self.g.kit
  -- The strength percentage and the depth line are developer figures now (a player reads them on the between-stage and bag screens).
  local text=('%+d%%'):format(math.floor((t.strength-1)*100+.5))
  self.m.text=dev and text or '';self.m.text_w=dev and (k and k.measure and k.measure(text,'caption') or #text*7) or 0
  self.m.keys_w=#keys>0 and (math.min(#keys,6)*20+(#keys>6 and 22 or 0)) or 0
  local dl=dev and (('Depth %d'):format(b.context.depth)..(self.m.loop>0 and ('  NG+%d'):format(self.m.loop) or '')) or ''
  self.m.depth_text=dl;self.m.depth_w=dev and (k and k.measure and k.measure(dl,'caption') or #dl*7) or 0
  -- Drives that wait for a decision are a fact the player needs, so they keep a small marker (not a depth figure).
  local wt=#h.decide>0 and ('%d waiting'):format(#h.decide) or ''
  self.m.wait_text=wt;self.m.wait_w=#h.decide>0 and (k and k.measure and k.measure(wt,'caption') or #wt*7) or 0
  return self.m
 end
 -- A short flash under the strip. Only the out-of-bounds notice (`always`) shows in play; every other flash is a developer figure.
 function Hd:flash(text,always)
  if not (always or Hd.dev_ui()) then return end
  self.flash_left=Hd.tuning.flash_frames;self.flash_text=text
 end
 function Hd:announce(lines,arch) self.toasts[#self.toasts+1]={lines=lines,left=Hd.tuning.announce_frames,arch=arch};self.dirty=true end
 -- The SMALL top-corner notification (the synergy message): one at a time, short, never the wide banner.
 function Hd:corner(lines,arch) self.corners[#self.corners+1]={lines=lines,left=Hd.tuning.corner_frames,arch=arch} end
 -- A pickup is a one-line note in the corner of the screen: title and its first line on one row.
 function Hd:show_card(title,lines,colour) self.card={title=title,lines=lines,colour=colour};self.card_left=Hd.tuning.card_frames end
 function Hd:clear() self.toasts={};self.corners={};self.card=nil;self.card_left=0;self.flash_left=0;self.m=nil;self.last_trace=nil end
 -- Called from the logic frame: timers count logic frames, so a pause stops them.
 function Hd:frame()
  if self.flash_left>0 then self.flash_left=self.flash_left-1 end
  local t=self.toasts[1];if t then t.left=t.left-1;if t.left<=0 then table.remove(self.toasts,1) end end
  local c=self.corners[1];if c then c.left=c.left-1;if c.left<=0 then table.remove(self.corners,1) end end
  if self.card_left>0 then self.card_left=self.card_left-1;if self.card_left<=0 then self.card=nil end end
 end
 -- A modifier fired: the engine's last trace line changed (a developer flash: gated in Hd:flash).
 function Hd:watch(engine)
  local tr=engine and engine.trace;if not tr then return end
  local n=#tr;local last=tr[n]
  if last~=self.last_trace then
   self.last_trace=last
   if last then local who=last:match('by ([%w ]+)$') or last:match('from ([%w ]+)$');self:flash(who and (who..'!') or 'Modifier!') end
  end
 end
 -- Co-op: each seat's strip, toasts and cards sit on its own side of the screen and carry its port colour (P1 red, P2 blue). A one-player
 -- host has no seat and draws exactly as before.
 Hd.port_colour={0xE0574FFF,0x4F86E0FF,0xE5C447FF,0x56B66AFF}
 function Hd:seat() return self.host.seat end
 function Hd:place(a,w,x0) local seat=self.host.seat;if not seat then return x0 end;if seat.index%2==0 then return a.x+a.w-w-10 end;return a.x+10 end
 function Hd:draw_strip()
  local g=self.g;local m=self:model();if not m then return end
  local k=g.kit;local a=g.safe_area()
  local seat=self.host.seat;local tag=seat and 26 or 0
  local x,y=a.x+10,a.y+10;local pip,gap=14,4
  local w=tag+8+#m.pips*(pip+gap)+6
  if m.text_w>0 then w=w+m.text_w+10 end
  if m.depth_w>0 then w=w+m.depth_w+10 end
  if m.wait_w>0 then w=w+m.wait_w+10 end
  if m.keys_w>0 then w=w+m.keys_w+10 end
  self.strip_w=w;if seat then x=self:place(a,w,x) end
  local flash=self.flash_left>0
  g.fill(x,y,w,26,flash and 0x3A3320E8 or 0x10181EC8)
  if seat then local pc=Hd.port_colour[seat.port] or 0xFFFFFFFF;g.fill(x,y,tag-2,26,pc);if k then k.text(x+(tag-2)//2,y+18,'P'..seat.port,'caption','ink','center') end end
  local px=x+tag+8
  for _,p in ipairs(m.pips) do
   if p then g.fill(px,y+6,pip,pip,p.border);g.fill(px+2,y+8,pip-4,pip-4,p.colour)
   else g.fill(px,y+6,pip,pip,0x59656FFF);g.fill(px+2,y+8,pip-4,pip-4,0x161D23FF) end
   px=px+pip+gap
  end
  px=px+6
  if k then
   if m.text_w>0 then k.text(px,y+19,m.text,'caption',flash and 'gold' or 'bone','left');px=px+m.text_w+10 end
   if m.depth_w>0 then k.text(px,y+19,m.depth_text,'caption','muted','left');px=px+m.depth_w+10 end
   if m.wait_w>0 then k.text(px,y+19,m.wait_text,'caption','gold','left');px=px+m.wait_w+10 end
   for i,kc in ipairs(m.keys) do
    if i>6 then k.text(px,y+19,'+'..(#m.keys-6),'caption','gold','left');break end
    g.fill(px,y+4,16,18,0xEBD175FF);g.fill(px+1,y+5,14,16,kc.colour);k.text(px+8,y+18,kc.letter,'caption','ink','center');px=px+20
   end
   if flash and self.flash_text then k.text(x,y+44,self.flash_text,'caption','gold','left') end
  else
   g.text(px,y+6,m.text,0xF3F0E8FF,10)
  end
 end
 function Hd:draw_toast()
  local t=self.toasts[1];if not t then return end
  local g=self.g;local k=g.kit;if not k then return end
  local a=g.safe_area();local seat=self.host.seat;local w=math.min(560,a.w-40);if seat then w=math.min(420,a.w//2-30) end;local x=a.x+(a.w-w)/2;local y=a.y+70
  if seat then x=self:place(a,w,x) end
  -- Long lines wrap inside the panel (once per toast and width) instead of shrinking their text until it cannot be read.
  if not t.rows or t.rows_w~=w then
   t.rows={};t.rows_w=w
   for i,l in ipairs(t.lines) do
    local text=l.text or l
    local parts=(D.drive_menu and D.drive_menu.wrap) and D.drive_menu.wrap(k,text,w-32-(t.arch and 56 or 0)) or {text}
    for _,part in ipairs(parts) do t.rows[#t.rows+1]={text=part,colour=l.colour or (i==1 and 'gold' or 'bone')} end
   end
  end
  local h=22+#t.rows*20
  if t.arch then h=math.max(h,64) end
  k.panel(x,y,w,h)
  if seat then g.fill(x,y,5,h,Hd.port_colour[seat.port] or 0xFFFFFFFF) end
  -- An archetype card: its emblem in its own colour on a tinted tile (the first time a build is assembled), the lines beside it.
  local ox=0
  if t.arch and D.synergy_fx then
   ox=56;g.fill(x+14,y+14,40,40,0x0A0D12F0);g.box(x+14,y+14,40,40,(t.arch.colour<<8)|255)
   D.synergy_fx.emblem(g,t.arch,x+20,y+20,28,255)
  end
  for i,r in ipairs(t.rows) do k.text(x+16+ox,y+14+i*20-4,r.text,'body',r.colour,'left',{max_w=w-32-ox}) end
 end
 -- The small corner note: a 22 px emblem, the title, the one-line blurb in caption type. Top-right (the opponent cards sit below it).
 function Hd:draw_corner()
  local c=self.corners[1];if not c then return end
  local g=self.g;local k=g.kit;if not k then return end
  local a=g.safe_area();local w=math.min(300,a.w//3);local x=a.x+a.w-w-10;local y=a.y+10;local h=40
  local seat=self.host.seat;if seat and seat.index%2==1 then x=a.x+a.w-w-10 end
  k.panel(x,y,w,h)
  local ox=0
  if c.arch and D.synergy_fx then ox=30;D.synergy_fx.emblem(g,c.arch,x+8,y+9,22,255) end
  local title=c.lines[1] and (c.lines[1].text or c.lines[1]) or '';local blurb=c.lines[2] and (c.lines[2].text or c.lines[2]) or ''
  k.text(x+10+ox,y+17,title,'caption',(c.lines[1] and c.lines[1].colour) or 'gold','left',{max_w=w-20-ox})
  if blurb~='' then k.text(x+10+ox,y+33,blurb,'caption','muted','left',{max_w=w-20-ox}) end
 end
 function Hd:draw_card()
  local c=self.card;if not c then return end
  local g=self.g;local k=g.kit;if not k then return end
  local a=g.safe_area();local seat=self.host.seat;local w=math.min(360,a.w//2-30);local x=a.x+10;local y=a.y+a.h-56;local h=26
  if seat then x=self:place(a,w,x) end
  local line=c.title..(c.lines[1] and (': '..(c.lines[1].text or c.lines[1])) or '')
  k.panel(x,y,w,h);if seat then g.fill(x,y,5,h,Hd.port_colour[seat.port] or 0xFFFFFFFF) end
  k.text(x+12,y+18,line,'caption',c.colour or 'gold','left',{max_w=w-24})
 end
 function Hd:draw()
  self:draw_strip();self:draw_toast();self:draw_corner();self:draw_card()
 end
 -- Model dump for tests and the console: what the strip would say.
 function Hd:dump()
  local m=self:model();if not m then return {'hud: no host'} end
  local out={};local pips={}
  for i,p in ipairs(m.pips) do pips[i]=p and ('[%06X/%06X]'):format(p.colour>>8,p.border>>8) or '[empty]' end
  out[1]='hud: '..table.concat(pips,' ')..(m.text~='' and ('  '..m.text) or '')..(m.depth_text~='' and ('  '..m.depth_text) or '')..(m.wait_text~='' and ('  '..m.wait_text) or '')..(#m.keys>0 and ('  keys['..(function() local l={};for _,kc in ipairs(m.keys) do l[#l+1]=kc.letter end;return table.concat(l,'') end)()..']') or '')..(self.flash_left>0 and ('  FLASH '..tostring(self.flash_text)) or '')
  for _,t in ipairs(self.toasts) do for _,l in ipairs(t.lines) do out[#out+1]='hud toast: '..(l.text or l) end end
  for _,c in ipairs(self.corners) do for _,l in ipairs(c.lines) do out[#out+1]='hud corner: '..(l.text or l) end end
  if self.card then out[#out+1]='hud card: '..self.card.title..' / '..table.concat(self.card.lines,' | ') end
  return out
 end
 return Hd
end
