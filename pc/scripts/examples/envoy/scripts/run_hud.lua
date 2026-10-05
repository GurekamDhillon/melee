-- Always-on build strip, announcements and the pickup card for a rule-host run. Everything it shows is a small
-- model built when the bag, the context or an announcement changes; the per-frame draw only replays it.
return function(D)
 local Hd={};Hd.__index=Hd
 Hd.tuning={flash_frames=90,announce_frames=360,card_frames=150,anchor='top-left'}
 local function T() return D.drive_text end
 function Hd.new(g,host) return setmetatable({g=g,host=host,flash_left=0,flash_text=nil,toasts={},card=nil,card_left=0,last_trace=nil},Hd) end
 -- Rebuilt when the bag (rev), the progression or the engine changes.
 function Hd:model()
  local h=self.host;local d=h.mods.drives;if not d then return nil end
  local b=d.bag;local key=table.concat({d.rev or 0,b.context.depth,b.context.loop,#b.items,tostring(b.equipped),#h.decide,h.loop or 0},'|')
  if self.m and self.m.key==key then return self.m end
  local pips={};for slot=1,b:slots() do local r=b.equipped[slot];pips[slot]=r and {colour=T().base_colour[r.colour],border=T().rarity_colour[r.rarity]} or false end
  -- Keystones are small cells (an initial letter on the family colour), never a name list.
  local keys={};for _,id in ipairs(h:keystone_ids()) do local r=h:keystone_rule(id);local fam=D.keystones.family(id)
   keys[#keys+1]={letter=(r and r.label or id):sub(1,1):upper(),colour=T().base_colour[fam] or 0xEBD175FF} end
  local t=h:totals()
  self.m={key=key,pips=pips,strength=t.strength,keys=keys,depth=b.context.depth,loop=h.loop or b.context.loop,bag=#b.items,label=nil}
  local k=self.g.kit
  -- Strength is a percentage over an empty build, the same number the screens use; no multiplier, no decimals.
  local text=('%+d%%'):format(math.floor((t.strength-1)*100+.5))
  self.m.text=text;self.m.text_w=k and k.measure and k.measure(text,'caption') or #text*7
  self.m.keys_w=#keys>0 and (math.min(#keys,6)*20+(#keys>6 and 22 or 0)) or 0
  local dl=('Depth %d'):format(b.context.depth)..(self.m.loop>0 and ('  NG+%d'):format(self.m.loop) or '')..(#h.decide>0 and ('  |  %d drive(s) waiting'):format(#h.decide) or '')
  self.m.depth_text=dl;self.m.depth_w=k and k.measure and k.measure(dl,'caption') or #dl*7
  return self.m
 end
 function Hd:flash(text) self.flash_left=Hd.tuning.flash_frames;self.flash_text=text end
 function Hd:announce(lines) self.toasts[#self.toasts+1]={lines=lines,left=Hd.tuning.announce_frames};self.dirty=true end
 function Hd:show_card(title,lines,colour) self.card={title=title,lines=lines,colour=colour};self.card_left=Hd.tuning.card_frames end
 function Hd:clear() self.toasts={};self.card=nil;self.card_left=0;self.flash_left=0;self.m=nil;self.last_trace=nil end
 -- Called from the logic frame: timers count logic frames, so a pause stops them.
 function Hd:frame()
  if self.flash_left>0 then self.flash_left=self.flash_left-1 end
  local t=self.toasts[1];if t then t.left=t.left-1;if t.left<=0 then table.remove(self.toasts,1) end end
  if self.card_left>0 then self.card_left=self.card_left-1;if self.card_left<=0 then self.card=nil end end
 end
 -- A modifier fired: the engine's last trace line changed.
 function Hd:watch(engine)
  local tr=engine and engine.trace;if not tr then return end
  local n=#tr;local last=tr[n]
  if last~=self.last_trace then
   self.last_trace=last
   if last then local who=last:match('by ([%w ]+)$') or last:match('from ([%w ]+)$');self:flash(who and (who..'!') or 'Modifier!') end
  end
 end
 function Hd:draw_strip()
  local g=self.g;local m=self:model();if not m then return end
  local k=g.kit;local a=g.safe_area()
  local x,y=a.x+10,a.y+10;local pip,gap=14,4
  local w=8+#m.pips*(pip+gap)+6+m.text_w+10+m.depth_w+10
  if m.keys_w>0 then w=w+m.keys_w+10 end
  local flash=self.flash_left>0
  g.fill(x,y,w,26,flash and 0x3A3320E8 or 0x10181EC8)
  local px=x+8
  for _,p in ipairs(m.pips) do
   if p then g.fill(px,y+6,pip,pip,p.border);g.fill(px+2,y+8,pip-4,pip-4,p.colour)
   else g.fill(px,y+6,pip,pip,0x59656FFF);g.fill(px+2,y+8,pip-4,pip-4,0x161D23FF) end
   px=px+pip+gap
  end
  px=px+6
  if k then
   k.text(px,y+19,m.text,'caption',flash and 'gold' or 'bone','left');px=px+m.text_w+10
   k.text(px,y+19,m.depth_text,'caption','muted','left');px=px+m.depth_w+10
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
  local a=g.safe_area();local w=math.min(560,a.w-40);local x=a.x+(a.w-w)/2;local y=a.y+70
  -- Long lines wrap inside the panel (once per toast and width) instead of shrinking their text until it cannot be read.
  if not t.rows or t.rows_w~=w then
   t.rows={};t.rows_w=w
   for i,l in ipairs(t.lines) do
    local text=l.text or l
    local parts=(D.drive_menu and D.drive_menu.wrap) and D.drive_menu.wrap(k,text,w-32) or {text}
    for _,part in ipairs(parts) do t.rows[#t.rows+1]={text=part,colour=l.colour or (i==1 and 'gold' or 'bone')} end
   end
  end
  local h=22+#t.rows*20
  k.panel(x,y,w,h)
  for i,r in ipairs(t.rows) do k.text(x+16,y+14+i*20-4,r.text,'body',r.colour,'left',{max_w=w-32}) end
 end
 function Hd:draw_card()
  local c=self.card;if not c then return end
  local g=self.g;local k=g.kit;if not k then return end
  local a=g.safe_area();local w=math.min(420,a.w-40);local x=a.x+(a.w-w)/2;local y=a.y+a.h-150;local h=30+#c.lines*20
  k.panel(x,y,w,h);k.text(x+14,y+22,c.title,'body',c.colour or 'gold','left',{max_w=w-28})
  for i,l in ipairs(c.lines) do k.text(x+14,y+22+i*20,l,'body','bone','left',{max_w=w-28}) end
 end
 function Hd:draw()
  self:draw_strip();self:draw_toast();self:draw_card()
 end
 -- Model dump for tests and the console: what the strip would say.
 function Hd:dump()
  local m=self:model();if not m then return {'hud: no host'} end
  local out={};local pips={}
  for i,p in ipairs(m.pips) do pips[i]=p and ('[%06X/%06X]'):format(p.colour>>8,p.border>>8) or '[empty]' end
  out[1]='hud: '..table.concat(pips,' ')..'  '..m.text..'  '..m.depth_text..(#m.keys>0 and ('  keys['..(function() local l={};for _,kc in ipairs(m.keys) do l[#l+1]=kc.letter end;return table.concat(l,'') end)()..']') or '')..(self.flash_left>0 and ('  FLASH '..tostring(self.flash_text)) or '')
  for _,t in ipairs(self.toasts) do for _,l in ipairs(t.lines) do out[#out+1]='hud toast: '..(l.text or l) end end
  if self.card then out[#out+1]='hud card: '..self.card.title..' / '..table.concat(self.card.lines,' | ') end
  return out
 end
 return Hd
end
