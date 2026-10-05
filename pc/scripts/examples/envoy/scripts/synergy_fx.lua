-- The look of synergies: an ASSEMBLED BUILD (an archetype of the derived graph, mod_graph) has an identity you can see.
-- The presentation language (docs/prompts/codex-earned-states-crits-held.md) gives every output ONE meaning:
--   surface = what is equipped / a status    earned afterimage = a technique-earned state    tracer = a crit's hit    particle burst / post pass = a moment
-- This module adds ONE meaning: **assembled build = an archetype's colour, motif and emblem**. Wherever it shows, it says "these pieces work together":
--   * the grid: link marks between connected pieces, the focused piece's partners and lines, an archetype banner (n/m), a mark on an offer that
--     would advance or complete a chain, one line in the detail panel that says what the connection does;
--   * a fight: a link flash between the fighter and its target when a chain fires (scaled by what it did, softened on rapid hits), an emblem pill
--     beside the HUD strip with a counter that climbs while the chain keeps firing and fades when it stops;
--   * the surface channel: a persistent, subtle motif treatment while the archetype is complete, stronger while its chain fires (lane 14 of the
--     surface params, modifiers_surface.wgsl); opponents with an assembled archetype show it too, and their nameplate names it;
--   * an announcement card the first time an archetype is completed in a run.
-- Everything is presentation: no gameplay write, no rule state. Rolls and effects stay data; this reads the engine's checkpointed state and its trace.
-- Per peer; layers are optional one by one (mod_tuning.fx) and every gd call is guarded.
return function(D)
 local F={};F.__index=F
 local function clamp(x,a,b) return x<a and a or (x>b and b or x) end
 local function G() return D.mod_graph end
 local function FX() return D.mod_tuning and D.mod_tuning.fx or {intensity=1} end
 -- ---- emblems: 9x9 silhouettes, each distinct in greyscale ----------------------------------------------------------------
 local BITS={
  flame={'....#....','...##....','...###...','..####.#.','.#######.','.#######.','.#######.','..#####..','...###...'},
  frost={'....#....','.#..#..#.','..#.#.#..','...###...','#########','...###...','..#.#.#..','.#..#..#.','....#....'},
  arc={'......##.','.....##..','....##...','...#####.','.....##..','....##...','...##....','..##.....','..#......'},
  chevron={'....#....','...###...','..##.##..','.##...##.','....#....','...###...','..##.##..','.##...##.','.........'},
  web={'...###...','...###...','..#...#..','..#...#..','.#.....#.','.#.....#.','###...###','###...###','#########'},
  plate={'#########','#########','####.####','###...###','####.####','.#######.','.#######.','..#####..','...###...'},
  sight={'....#....','..#####..','.#..#..#.','.#..#..#.','#########','.#..#..#.','.#..#..#.','..#####..','....#....'},
  burst={'....#....','....#....','...###...','..#####..','#########','..#####..','...###...','....#....','....#....'},
  spike={'....#....','....#....','...###...','...###...','..#####..','..#####..','.#######.','#########','#########'},
 }
 F.emblems=BITS
 local rect_cache={}
 local function rects(motif,size)
  local key=motif..':'..size;local c=rect_cache[key];if c then return c end
  c={};local rows=BITS[motif] or BITS.burst
  for r=1,9 do
   local y0=math.floor((r-1)*size/9+.5);local y1=math.floor(r*size/9+.5);local line=rows[r]
   local c0
   for col=1,10 do
    local on=col<=9 and line:sub(col,col)=='#'
    if on and not c0 then c0=col elseif not on and c0 then
     local x0=math.floor((c0-1)*size/9+.5);local x1=math.floor((col-1)*size/9+.5)
     c[#c+1]={x0,y0,math.max(1,x1-x0),math.max(1,y1-y0)};c0=nil
    end
   end
  end
  rect_cache[key]=c;return c
 end
 local function rgba(rgb,a) return ((rgb<<8)|clamp(math.floor(a+.5),0,255)) end
 -- Draw an emblem in its archetype colour with a dark backing so it reads on any ground. `size` in canvas pixels (at least 9 reads).
 function F.emblem(g,arch,x,y,size,alpha)
  alpha=alpha or 255;size=math.max(9,math.floor(size));local list=rects(arch.motif,size)
  local back=math.floor(alpha*.8)
  for _,r in ipairs(list) do g.fill(x+r[1]-1,y+r[2]-1,r[3]+2,r[4]+2,rgba(0x0A0D12,back)) end
  for _,r in ipairs(list) do g.fill(x+r[1],y+r[2],r[3],r[4],rgba(arch.colour,alpha)) end
 end
 -- ---- state ------------------------------------------------------------------------------------------------------------------
 function F.new(g,host)
  local self=setmetatable({g=g,host=host,gen=0,chains={},flashes={},active={},announced={},cache={},count=0,counter={}},F)
  F.current=self
  if g.command then
   g.command('synergy',function(arg) return self:command(arg or '') end,'fx | fx preview <archetype> [0..1] | fx <layer> on|off | fx intensity <0..1> | fx announce <archetype> | fx dump | archetypes')
  end
  return self
 end
 function F:reset() self.gen=0;self.chains={};self.flashes={};self.active={};self.announced={};self.cache={};self.counter={};self.pv=nil;self.grid=nil end
 -- fighter positions this frame (read lazily: only when a flash is made or drawn)
 function F:players()
  local g=self.g;local f=g.frame and g.frame() or 0
  if self.pl_f~=f then local t={};for p=1,6 do local v=g.player(p);if v then t[p]=v end end;self.pl=t;self.pl_f=f end
  return self.pl
 end
 function F:engine() return self.host and self.host.mods and self.host.mods.engine end
 function F:pool() return D.mod_pool end
 -- The set of record ids a port holds right now, and the archetypes it has completed (cached on the identity of the engine's build table).
 function F:assembled(e,p)
  local eq=e.equipped[p];local c=self.cache[p]
  if c and c.eq==eq then return c.list,c.held end
  local held={};for id in pairs(eq or {}) do held[id]=true end
  local list=eq and next(eq) and G().completed(held) or {}
  self.cache[p]={eq=eq,list=list,held=held};return list,held
 end
 -- ---- the surface lane (lane 14 of the surface params): archetype index + level, 0 when nothing is assembled -------------------
 function F:lane(e,p)
  local fx=FX();if fx.surface==false or (fx.intensity or 1)<=0 then return 0 end
  local pv=self.pv
  if pv and pv.port==p and e.frame<pv.until_f then return pv.arch.index+clamp(pv.level,0,1)*.99 end
  local list=self:assembled(e,p);if #list==0 then return 0 end
  local pick=list[1]
  local act=self.active[p]
  if act then for _,a in ipairs(list) do if a==act.arch then pick=a end end end
  local level=.625
  if act and act.arch==pick then local age=e.frame-act.frame;if age>=0 and age<90 then level=.625+.375*act.size*(1-age/90) end end
  level=math.floor(level*8+.5)/8   -- eighths: the parameter array is rebuilt only when it changes
  return pick.index+clamp(level*(fx.intensity or 1),0,1)*.99
 end
 -- ---- chains (the engine's trace already knows who fed whom) ----------------------------------------------------------------
 local function labels_of(e)
  local map=e.synfx_labels
  if not map then map={};for _,m in ipairs(e.list) do map[m.label]=m.id end;e.synfx_labels=map end
  return map
 end
 -- The records named by the engine's last trace, in order of appearance.
 function F.trace_ids(e)
  local map=labels_of(e);local ids,seen={},{}
  for _,line in ipairs(e.trace or {}) do
   local name=line:match(' by (.+)$') or line:match(' from (.+)$')
   local id=name and map[name]
   if id and not seen[id] then seen[id]=true;ids[#ids+1]=id end
  end
  return ids
 end
 -- Called every logic frame (cheap: it returns at once unless the trace changed).
 function F:scan(e)
  local gen=e.display.trace_generation or 0
  if gen==self.gen then return end
  self.gen=gen
  local ids=F.trace_ids(e);if #ids<2 then return end
  local arch=G().chain_archetype(self:pool(),ids[1],ids[#ids]);if not arch then return end
  local port=e.trace_port or 1
  local size=clamp(.22*(#ids-1)+.12*#ids/4,.2,1)
  self:chain_fired(e,port,arch,size)
 end
 -- One chain firing: count it, activate the surface, maybe flash a link (never on every hit).
 function F:chain_fired(e,port,arch,size)
  local fx=FX();local f=e.frame
  self.active[port]={arch=arch,frame=f,size=size}
  local c=self.counter[port]
  if c and c.arch==arch and f-c.last<150 then if f-c.stamp>=18 then c.n=c.n+1;c.stamp=f end;c.last=f;c.size=math.max(c.size*.8,size)
  else c={arch=arch,n=1,last=f,stamp=f,size=size,born=f};self.counter[port]=c end
  if fx.chain==false or (fx.intensity or 1)<=0 then return end
  local prev=self.flashes[#self.flashes]
  local soft=prev and f-prev.start<45
  if prev and f-prev.start<18 then return end                 -- never a strobe: a flash at most every 18 frames, softened inside 45
  local target;local players=self:players()
  if players then local best,bd;local me=players[port]
   for p,v in pairs(players) do if p~=port and type(v.x)=='number' and me and type(me.x)=='number' then local d=(v.x-me.x)^2+((v.y or 0)-(me.y or 0))^2;if not bd or d<bd then best,bd=p,d end end end
   target=best end
  self.flashes[#self.flashes+1]={from=port,to=target,arch=arch,start=f,size=size,soft=soft}
  while #self.flashes>4 do table.remove(self.flashes,1) end
 end
 -- The look of the chain pulse for the engine's last trace: the post pass takes a tint and a shape from the archetype.
 local SHAPE={flame=1,chevron=1,web=2,arc=2,frost=4,spike=4,sight=3,burst=3,plate=0}
 function F.pulse_look(e)
  if FX().chain==false then return nil end
  local ids=F.trace_ids(e);if #ids<2 then return nil end
  local arch=G().chain_archetype(D.mod_pool,ids[1],ids[#ids]);if not arch then return nil end
  local c=arch.colour
  return {tint={((c>>16)&255)/255,((c>>8)&255)/255,(c&255)/255,1},shape=SHAPE[arch.motif] or 0,arch=arch}
 end
 -- ---- frame ----------------------------------------------------------------------------------------------------------------------
 function F:frame(e,port0)
  if not e then return end
  self:scan(e)
  local pv=self.pv
  if pv then
   if e.frame>=pv.until_f then self.pv=nil
   else
    if e.frame-pv.last_count>=30 then pv.last_count=e.frame;pv.n=pv.n+1
     self.counter[pv.port]={arch=pv.arch,n=pv.n,last=e.frame,stamp=e.frame,size=pv.level,born=pv.start}
     self.active[pv.port]={arch=pv.arch,frame=e.frame,size=pv.level}
     if pv.n%2==1 or pv.n==1 then local prevf=self.flashes[#self.flashes];if not prevf or e.frame-prevf.start>=24 then
      local target;for p in pairs(self:players() or {}) do if p~=pv.port then target=target or p end end
      self.flashes[#self.flashes+1]={from=pv.port,to=target,arch=pv.arch,start=e.frame,size=pv.level}
      while #self.flashes>4 do table.remove(self.flashes,1) end end end
    end
   end
  end
  -- announce the first completion of each archetype in the run (the run's own fighter)
  if self.host and self.host.hud and port0 and e.frame%20==0 and FX().announce~=false then
   local list=self:assembled(e,port0)
   for _,a in ipairs(list) do if not self.announced[a.id] then self.announced[a.id]=true;self:announce(a) end end
  end
 end
 function F:announce(a)
  local h=self.host;if not (h and h.hud) then return end
  h.hud:announce({{text=a.name..' assembled',colour='gold'},a.blurb},a)
  if h.log then h:log('synergy: '..a.name..' assembled') end
 end
 -- ---- drawing: a fight ---------------------------------------------------------------------------------------------------------
 local function text(g,x,y,s,role,colour,align)
  local k=g.kit
  if k and k.text then k.text(x,y,s,role or 'caption',colour or 'bone',align or 'left') else g.text(x,y-10,s,type(colour)=='number' and colour or 0xF3F0E8FF,1) end
 end
 function F:project(p,players,dy)
  local g=self.g;local v=players and players[p];if not v or type(v.x)~='number' or not g.project then return nil end
  local ok,x,y,vis=pcall(g.project,v.x,(v.y or 0)+(dy or 12),0);if ok and x and vis~=false then return x,y end
 end
 local function curve(x1,y1,x2,y2,t,bend)
  local mx,my=(x1+x2)/2,(y1+y2)/2-bend
  local a,b,c=(1-t)*(1-t),2*(1-t)*t,t*t
  return a*x1+b*mx+c*x2,a*y1+b*my+c*y2
 end
 -- A link flash between the fighter and its target in the archetype's colour; it widens with what the chain did and is soft on rapid hits.
 function F:draw_flashes(e)
  local g=self.g;local fx=FX();if fx.chain==false or (fx.intensity or 1)<=0 or #self.flashes==0 then return end
  local players=self:players()
  for i=#self.flashes,1,-1 do local f=self.flashes[i];local age=e.frame-f.start;local dur=f.soft and 14 or 22
   if age>dur then table.remove(self.flashes,i)
   elseif age>=0 then
    local x1,y1=self:project(f.from,players);local x2,y2=self:project(f.to or f.from,players)
    if x1 then
     if not x2 then x2,y2=x1+60,y1 end
     if f.to==nil or f.to==f.from then x2,y2=x1+70,y1-30 end
     local env=math.sin(clamp(age/dur,0,1)*math.pi);local k=(f.soft and .5 or 1)*(fx.intensity or 1)
     local width=3+math.floor(f.size*4+.5);local alpha=clamp(env*k*(.45+.55*f.size)*255,30,245)
     local col=f.arch.colour;local head=clamp(age/(dur*.7),0,1)
     local bend=14+f.size*16;local px,py
     local steps=16
     for s=0,steps do local t=s/steps
      if t<=head then
       local x,y=curve(x1,y1,x2,y2,t,bend);local fade=.35+.65*(t/math.max(.01,head))
       g.fill(x-width,y-width,width*2,width*2,rgba(col,alpha*fade*.22))   -- soft outer glow
       g.fill(x-width/2,y-width/2,width,width,rgba(col,alpha*fade))
       if px and g.line then for d=-1,1 do g.line(px,py+d,x,y+d,rgba(col,alpha*fade*(d==0 and 1 or .6))) end end
       px,py=x,y
      end
     end
     if px then local sz=4+math.floor(f.size*5);g.fill(px-sz/2,py-sz/2,sz,sz,rgba(0xFFFFFF,alpha*.9));g.fill(px-sz/2+1,py-sz/2+1,sz-2,sz-2,rgba(col,alpha)) end
     -- the emblem pops over the target at the head of the link
     if head>=1 and age<dur then F.emblem(g,f.arch,x2-9,y2-26-math.floor(age*.4),14+math.floor(f.size*6),alpha) end
    end
   end
  end
 end
 -- The pill beside the HUD strip: emblems of the assembled archetypes, and a counter that climbs while a chain keeps firing.
 function F:draw_hud(e,port0)
  local g=self.g;local fx=FX();if fx.hud==false or (fx.intensity or 1)<=0 or not self.host.hud then return end
  local list=self:assembled(e,port0);local c=self.counter[port0];local live=c and e.frame-c.last<150
  if #list==0 and not live then return end
  local hud=self.host.hud;local sw=hud.strip_w or 190;local a=g.safe_area();local x,y=a.x+10+sw+6,a.y+10
  local w=8+#list*(18+3)+(live and 62 or 0)+4
  g.fill(x,y,w,26,0x10181EC8)
  local px=x+8
  for _,arch in ipairs(list) do F.emblem(g,arch,px,y+5,16,230);px=px+21 end
  if live then
   local fade=clamp(1-(e.frame-c.last-90)/60,0,1)   -- holds 1.5 s after the last firing, then fades
   local alpha=math.floor(255*fade)
   local pop=clamp(1-(e.frame-c.stamp)/10,0,1)       -- a small pop on each increment
   F.emblem(g,c.arch,px+2,y+3-math.floor(pop*2),20+math.floor(pop*3),alpha)
   if g.kit then g.kit.text(px+30,y+19,'x'..c.n,'body',rgba(c.arch.colour,alpha),'left') else g.text(px+30,y+6,'x'..c.n,rgba(c.arch.colour,alpha),1) end
  end
 end
 function F:draw(e,port0)
  if not e then return end
  self:draw_flashes(e)
  if port0 then self:draw_hud(e,port0) end
 end
 -- The opponent's nameplate: the archetype it has assembled, right-aligned on the plate's title row.
 function F:plate_tag(a,y,port,e)
  local fx=FX();if fx.nameplate==false then return end
  local list=self:assembled(e,port);if #list==0 then return end
  local g=self.g;local arch=list[1];local k=g.kit
  local name=arch.name;local w=(k and k.measure) and k.measure(name,'caption') or #name*7
  local x=a.x+a.w-32-w-24
  F.emblem(g,arch,x,y+6,16,235);if k then k.text(x+22,y+19,name,'caption',rgba(arch.colour,255),'left') end
 end
 -- ---- drawing: the grid -----------------------------------------------------------------------------------------------------------
 local function ids_of_cell(cell)
  local ref=cell.ref or {}
  if ref.record then local l={};for _,a in ipairs(ref.record.affixes or {}) do l[#l+1]=a.id end;return l end
  if ref.id then return {ref.id} end
  return {}
 end
 local function held_sets(host)
  local b=host:bag();local eq,all={},{}
  for slot=1,b:slots() do local r=b.equipped[slot];if r then for _,a in ipairs(r.affixes) do eq[a.id]=true;all[a.id]=true end end end
  for _,r in ipairs(b.items) do for _,a in ipairs(r.affixes) do all[a.id]=true end end
  for _,id in ipairs(host:keystone_ids()) do eq[id]=true;all[id]=true end
  return eq,all
 end
 F.held_sets=held_sets
 -- Rebuilt when the screen's blocks change: which cells connect, and what each offer would do.
 function F:grid_model(screen)
  local key=tostring(screen.key)..'|'..tostring(screen.layout)
  if self.grid and self.grid.key==key and self.grid.blocks==screen.blocks then return self.grid end
  local host=self.host;local pool=self:pool();local g=G()
  local eq,all=held_sets(host);local cells={}
  for _,b in ipairs(screen.blocks or {}) do
   if b.id=='eq' or b.id=='bag' or b.id=='key' or b.id=='offer' or b.id=='koffer' then
    for i=1,b.cols*b.rows do local c=b.cells[i];if c and not c.empty and c.ref then local ids=ids_of_cell(c);if #ids>0 then cells[#cells+1]={block=b.id,index=i,ids=ids,cell=c,k=b.id..':'..i} end end end
   end
  end
  local m={key=key,blocks=screen.blocks,cells=cells,links={},marks={},offers={},byk={}}
  for _,c in ipairs(cells) do m.byk[c.k]=c end
  for i=1,#cells do for j=i+1,#cells do
   local a,b=cells[i],cells[j]
   -- two offered pieces do not link to each other, only to what you hold
   if not ((a.block=='offer' or a.block=='koffer') and (b.block=='offer' or b.block=='koffer')) then
    local best
    for _,x in ipairs(a.ids) do for _,y in ipairs(b.ids) do local e=g.connected(pool,x,y);if e and not best then best={e=e,a=x,b=y} end end end
    if best then
     local arch=g.chain_archetype(pool,best.e.from,best.e.to)
     m.links[#m.links+1]={a=a.k,b=b.k,arch=arch,e=best.e}
     for _,k in ipairs({a.k,b.k}) do m.marks[k]=m.marks[k] or {};local l=m.marks[k];local dup;for _,x in ipairs(l) do if x==arch then dup=true end end;if not dup and arch and #l<2 then l[#l+1]=arch end end
    end
   end
  end end
  for _,c in ipairs(cells) do if c.block=='offer' or c.block=='koffer' then local kind,arch=g.effect_of(c.ids,all);if kind then m.offers[c.k]={kind=kind,arch=arch} end end end
  m.banner=g.analyze(eq)
  self.grid=m;return m
 end
 -- The detail panel's synergy lines for the focused cell: what the connection does, in words (at most two), and the archetype effect of an offer.
 function F:detail_lines(screen,cell)
  local fx=FX();if fx.grid_links==false then return {} end
  local host=self.host;local pool=self:pool();local g=G();local out={}
  local ids=ids_of_cell(cell);if #ids==0 then return out end
  local eq,all=held_sets(host);local ref=cell.ref or {}
  local own={};for _,id in ipairs(ids) do own[id]=true end
  local tier=D.mod_progression.tier(host:bag().context) or 1
  local n=0
  for _,x in ipairs(ids) do for h in pairs(all) do
   if n<2 and not own[h] and g.connected(pool,x,h) then
    -- the line reads from the side that feeds: "Sets Burn, which your Cinder drive ..."
    local text=g.link_text(pool,x,h,math.min(tier,3));if text then n=n+1;out[#out+1]=text end
   end
  end end
  if ref.kind=='offer' or ref.kind=='koffer' then
   local kind,arch=g.effect_of(ids,all)
   if kind=='complete' then out[#out+1]='This completes '..arch.name..'.' elseif kind=='advance' then out[#out+1]='This builds '..arch.name..'.' end
  end
  return out
 end
 function F:draw_grid(screen)
  local fx=FX();local g=self.g;if (fx.intensity or 1)<=0 then return end
  local view=screen.view;if not view or screen.layout~='main' then return end
  local m=self:grid_model(screen);local L=view.lay;local frame=g.frame and g.frame() or 0
  -- the banner: assembled archetypes and partial progress, in the header between the title and the countdown
  if fx.grid_banner~=false and L and L.head and #m.banner>0 then
   local h=L.head;local k=g.kit;local x=h.x+(h.w>520 and 210 or 130);local y=h.y+4
   local budget=h.w-(x-h.x)-120
   for i,p in ipairs(m.banner) do
    if i>3 then break end
    local a=p.archetype;local label=p.complete and (a.name..' '..p.filled..'/'..p.total) or (a.name..' '..p.filled..'/'..p.total..': needs '..tostring(p.missing))
    local w=18+((k and k.measure) and k.measure(label,'caption') or #label*7)+10
    if x+w-(h.x)>h.w-90 then break end
    if p.complete then g.fill(x-2,y,w+2,22,rgba(a.colour,70));g.box(x-2,y,w+2,22,rgba(a.colour,255)) else g.fill(x-2,y,w+2,22,0x10181ECC);g.box(x-2,y,w+2,22,rgba(a.colour,120)) end
    F.emblem(g,a,x+2,y+3,16,p.complete and 255 or 190)
    if k then k.text(x+22,y+16,label,'caption','bone','left') end
    x=x+w+8
   end
  end
  if fx.grid_links==false and fx.offer_marks==false then return end
  local fc,fb,fi=view:focused();local fk=fb and (fb..':'..fi)
  local pulse=.65+.35*math.sin(frame*.07)
  -- corner marks on every connected piece; a completed archetype's piece wears its emblem
  if fx.grid_links~=false then
   for k,list in pairs(m.marks) do
    local c=m.byk[k];local x,y,w,h=view:cell_rect(c.block,c.index)
    if x then for n,arch in ipairs(list) do
     local done=false;for _,p in ipairs(m.banner) do if p.complete and p.archetype==arch then done=true end end
     local sx=x+3+(n-1)*13;local sy=y+3
     F.emblem(g,arch,sx,sy,11,done and 255 or 225)
    end end
   end
   -- the focused piece's partners: a box in the archetype's colour and a line from it
   if fk then
    local fcell=m.byk[fk]
    for _,l in ipairs(m.links) do
     if l.a==fk or l.b==fk then
      local other=m.byk[l.a==fk and l.b or l.a];local ax,ay,aw,ah=view:cell_rect(fcell.block,fcell.index);local bx,by,bw,bh=view:cell_rect(other.block,other.index)
      if ax and bx and l.arch then
       local col=rgba(l.arch.colour,255)
       g.box(bx-2,by-2,bw+4,bh+4,col);g.box(bx-3,by-3,bw+6,bh+6,rgba(l.arch.colour,150))
       local x1,y1,x2,y2=ax+aw/2,ay+ah/2,bx+bw/2,by+bh/2
       if g.line then for d=-1,1 do g.line(x1+d,y1,x2+d,y2,rgba(l.arch.colour,d==0 and 255 or 120)) end end
       F.emblem(g,l.arch,bx+bw-19,by+3,16,255)
      end
     end
    end
   end
  end
  -- an offer that would advance or complete a chain: its archetype's border, emblem and a bar, readable before any text
  if fx.offer_marks~=false then
   for k,o in pairs(m.offers) do
    local c=m.byk[k];local x,y,w,h=view:cell_rect(c.block,c.index)
    if x then
     local a=o.arch;local col=o.kind=='complete' and 255 or math.floor(150+90*pulse)
     g.box(x-1,y-1,w+2,h+2,rgba(a.colour,col));g.box(x,y,w,h,rgba(a.colour,col));g.box(x+1,y+1,w-2,h-2,rgba(a.colour,col))
     if o.kind=='complete' then g.fill(x+2,y+2,w-4,5,rgba(a.colour,255));g.fill(x+2,y+h-7,w-4,5,rgba(a.colour,255)) end
     F.emblem(g,a,x+w-20,y+4,o.kind=='complete' and 18 or 14,255)
    end
   end
  end
 end
 -- ---- the console ---------------------------------------------------------------------------------------------------------------
 function F.find(word)
  word=(word or ''):lower()
  for _,a in ipairs(G().archetypes) do if a.id==word then return a end end
  for _,a in ipairs(G().archetypes) do if a.name:lower():find(word,1,true) then return a end end
 end
 function F:command(arg)
  local g=self.g;local w={};for x in arg:gmatch('%S+') do w[#w+1]=x end
  local fx=FX()
  if w[1]=='archetypes' then for _,a in ipairs(G().archetypes) do g.log(('synergy: %-8s %-20s #%06X %s  %s'):format(a.id,a.name,a.colour,a.motif,a.blurb)) end;return true end
  if w[1]~='fx' then g.log('synergy: usage: '..'fx | fx preview <archetype> [0..1] | fx <layer> on|off | fx intensity <0..1> | fx announce <archetype> | fx dump | archetypes');return false end
  if w[2]=='preview' then
   local a=F.find(w[3]);if not a then g.log('synergy fx: unknown archetype '..tostring(w[3]));return false end
   local e=self:engine();if not e then g.log('synergy fx: no engine (start a run or the LAB)');return false end
   local level=clamp(tonumber(w[4]) or .8,0,1)
   self.pv={arch=a,level=level,port=1,start=e.frame,until_f=e.frame+300,last_count=e.frame-30,n=0}
   self.flashes={}
   if self.host and self.host.hud then self:announce(a) end
   g.log(('synergy fx: preview %s at %.2f for 5 seconds'):format(a.name,level));return true
  elseif w[2]=='announce' then local a=F.find(w[3]);if not a then return false end;self:announce(a);return true
  elseif w[2]=='intensity' then fx.intensity=clamp(tonumber(w[3]) or 1,0,1);g.log('synergy fx: intensity '..fx.intensity);return true
  elseif w[2]=='dump' then
   local e=self:engine();if not e then return false end
   for p=1,6 do if e.equipped[p] then local list=self:assembled(e,p);local n={};for _,a in ipairs(list) do n[#n+1]=a.name end
    g.log(('synergy fx: P%d assembled=[%s] lane=%.2f'):format(p,table.concat(n,', '),self:lane(e,p))) end end
   return true
  elseif w[2] and D.mod_tuning.fx[w[2]]~=nil then fx[w[2]]=(w[3]=='on' or w[3]==nil);g.log('synergy fx: '..w[2]..' '..(fx[w[2]] and 'on' or 'off'));return true end
  local names={};for _,n in ipairs(D.mod_tuning.fx_names) do names[#names+1]=n..'='..(fx[n]==false and 'off' or 'on') end
  g.log('synergy fx: intensity '..tostring(fx.intensity)..'  layers: '..table.concat(names,' '));return true
 end
 return F
end
