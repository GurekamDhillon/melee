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
  local b=d.bag;local key=table.concat({d.rev or 0,b.context.depth,b.context.loop,#b.items,tostring(b.equipped)},'|')
  if self.m and self.m.key==key then return self.m end
  local pips={};for slot=1,b:slots() do local r=b.equipped[slot];pips[slot]=r and {colour=T().base_colour[r.colour],border=T().rarity_colour[r.rarity]} or false end
  local keys={};for _,m in ipairs(d.lab.engine.list) do if m.kind=='keystone' and h:keystone_set()[m.id] then keys[#keys+1]=m.label end end
  local t=h:totals()
  self.m={key=key,pips=pips,strength=t.strength,keystone=#keys>0 and table.concat(keys,' + ') or nil,depth=b.context.depth,loop=b.context.loop,
   bag=#b.items,label=nil}
  local k=self.g.kit
  local text=('Strength %.2f'):format(t.strength)
  self.m.text=text;self.m.text_w=k and k.measure and k.measure(text,'caption') or #text*7
  if self.m.keystone then self.m.key_text='Keystone: '..self.m.keystone;self.m.key_w=k and k.measure and k.measure(self.m.key_text,'caption') or #self.m.key_text*7 end
  local dl=('Depth %d'):format(b.context.depth)..(b.context.loop>0 and ('  NG+%d'):format(b.context.loop) or '')
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
  if m.key_text then w=w+m.key_w+10 end
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
   if m.key_text then k.text(px,y+19,m.key_text,'caption','gold','left') end
   if flash and self.flash_text then k.text(x,y+44,self.flash_text,'caption','gold','left') end
  else
   g.text(px,y+6,m.text,0xF3F0E8FF,10)
  end
 end
 function Hd:draw_toast()
  local t=self.toasts[1];if not t then return end
  local g=self.g;local k=g.kit;if not k then return end
  local a=g.safe_area();local w=math.min(520,a.w-40);local x=a.x+(a.w-w)/2;local y=a.y+70
  local h=22+#t.lines*20
  k.panel(x,y,w,h)
  for i,l in ipairs(t.lines) do k.text(x+16,y+14+i*20-4,l.text or l,'body',l.colour or (i==1 and 'gold' or 'bone'),'left',{max_w=w-32}) end
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
  out[1]='hud: '..table.concat(pips,' ')..'  '..m.text..'  '..m.depth_text..(m.key_text and ('  '..m.key_text) or '')..(self.flash_left>0 and ('  FLASH '..tostring(self.flash_text)) or '')
  for _,t in ipairs(self.toasts) do for _,l in ipairs(t.lines) do out[#out+1]='hud toast: '..(l.text or l) end end
  if self.card then out[#out+1]='hud card: '..self.card.title..' / '..table.concat(self.card.lines,' | ') end
  return out
 end
 return Hd
end
