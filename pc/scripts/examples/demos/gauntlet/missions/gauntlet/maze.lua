-- Pure generation/serialization. Python invokes this exact implementation.
return function(D)
  local M={};local set=D.maze_set
  local dirs={{'left',-1,0,'right','L'},{'right',1,0,'left','R'},
    {'up',0,1,'down','U'},{'down',0,-1,'up','D'}}
  local function copy(t) if type(t)~='table' then return t end
    local q={};for k,v in pairs(t) do q[k]=copy(v) end;return q end
  function M.encode(t)
    if type(t)=='string' then return string.format('%q',t) end
    if type(t)~='table' then return tostring(t) end
    local keys={};for k in pairs(t) do keys[#keys+1]=k end
    table.sort(keys,function(a,b) if type(a)==type(b) then return a<b end return type(a)<type(b) end)
    local q={};for _,k in ipairs(keys) do q[#q+1]='['..M.encode(k)..']='..M.encode(t[k]) end
    return '{'..table.concat(q,',')..'}'
  end
  function M.random(seed)
    local state=seed%2147483647;if state==0 then state=1 end
    return function(n) state=(state*48271)%2147483647;return 1+state%n end
  end
  local function line(out,x1,y1,x2,y2,kind,pass,label)
    out[#out+1]={x1=x1,y1=y1,x2=x2,y2=y2,kind=kind,passthrough=pass or false,draw=true,label=label}
  end
  function M.geometry(c,authored)
    local x,y=c.x*130,c.y*104
    local q={version=2,units=6.5,parts={},lines={},camera={mode='chunk',margin=4,frames=45}}
    local function edge(a,b,d,e,kind,label) line(q.lines,x+a,y+b,x+d,y+e,kind,false,label) end
    -- Double-faced boundary collision; a wall stops entry from either side.
    local function horizontal(a,b,h,label)
      edge(a,h,b,h,'floor',label);edge(b,h,a,h,'ceiling',label)
    end
    local function vertical(a,b,h,label)
      edge(h,a,h,b,'left_wall',label);edge(h,b,h,a,'right_wall',label)
    end
    if authored then
      q=copy(authored)
      q.lines=q.lines or {};q.parts=q.parts or {}
      local keep={};for _,l in ipairs(q.lines) do if not (l.label and l.label:match('^seal:')) then keep[#keep+1]=l end end
      q.lines=keep
      for _,p in ipairs(q.parts) do p.x=p.x+x;p.y=p.y+y;p.name=c.id..'_'..(p.name or p.part) end
      for _,l in ipairs(q.lines) do l.x1=l.x1+x;l.x2=l.x2+x;l.y1=l.y1+y;l.y2=l.y2+y end
      for _,p in pairs(q.spawn or {}) do p.x=p.x+x;p.y=p.y+y end
      for _,p in ipairs(q.markers or {}) do p.x=p.x+x;p.y=p.y+y end
      local function shift(rect)
        if rect then rect.left=rect.left+x;rect.right=rect.right+x;rect.bottom=rect.bottom+y;rect.top=rect.top+y end
      end
      shift(q.blast)
      if q.camera then
        if q.camera.left then shift(q.camera) end
        shift(q.camera.ends)
        if q.camera.x then q.camera.x=q.camera.x+x end
      end
    else
      for _,l in ipairs(set.shell) do edge(l[2],l[3],l[4],l[5],l[1]) end
    end
    for _,e in ipairs(c.exits) do if e.sealed then
      local label='seal:'..e.side..':'..e.slot
      if e.side=='left' then vertical(0,32,0,label)
      elseif e.side=='right' then vertical(0,32,130,label)
      elseif e.side=='up' then horizontal(53,77,104,label)
      else horizontal(53,77,0,label) end
    end end
    for _,p in ipairs(authored and {} or c.platforms) do
      line(q.lines,x+p.x1,y+p.y,x+p.x2,y+p.y,'floor',true,'climb')
    end
    -- Existing kit slabs decorate the supported floor strips and platforms.
    local slabs=authored and {} or copy(set.slabs)
    if not authored then for _,p in ipairs(c.platforms) do slabs[#slabs+1]=p end end
    for i,p in ipairs(slabs) do q.parts[#q.parts+1]={part=set.part,name=c.id..'_slab'..i,
      x=x+(p.x1+p.x2)/2,y=y+p.y,z=0,rot=0,scale_x=(p.x2-p.x1)/26,
      collision=false,floor_flags=0} end
    return q
  end
  function M.generate(seed,options,random)
    options=options or {};local size=options.size or 12
    assert(type(seed)=='number' and seed==seed and seed%1==0 and math.abs(seed)<=2147483646,'seed must be signed 31-bit integer')
    assert(type(size)=='number' and size%1==0 and size>=8 and size<=20,'size must be 8..20')
    local length=options.length or math.ceil(size*.7)
    assert(length%1==0 and length>=4 and length<=size,'length must be 4..size')
    local rng=random or M.random(seed)
    local function choose(n) local i=rng(n);assert(type(i)=='number' and i%1==0 and i>=1 and i<=n,'RNG out of bounds');return i end
    local m={version=1,seed=seed,size=size,cells={},main_path={},start='c1'}
    local occupied={}
    local function key(x,y) return x..','..y end
    local function add(x,y,parent,d)
      local c={id='c'..(#m.cells+1),x=x,y=y,exits=copy(set.exits),platforms=copy(set.platforms),links={},forward={}}
      m.cells[#m.cells+1]=c;occupied[key(x,y)]=c
      if parent then c.links[d[4]]=parent.id;parent.links[d[1]]=c.id;parent.forward[d[1]]=true end
      return c
    end
    local c=add(0,0);m.main_path[1]=c.id
    for i=2,length do
      -- Each new column gets a random vertical direction; collision falls back
      -- to east. No rejection/reseed loop, including with adversarial injected RNG.
      local d=dirs[choose(3)+1] -- right, up or down
      if occupied[key(c.x+d[2],c.y+d[3])] then d=dirs[2] end
      c=add(c.x+d[2],c.y+d[3],c,d);m.main_path[i]=c.id
    end
    m.goal=c.id
    while #m.cells<size do
      local choices={}
      for _,p in ipairs(m.cells) do for _,d in ipairs(dirs) do
        if not occupied[key(p.x+d[2],p.y+d[3])] then choices[#choices+1]={p=p,d=d} end
      end end
      local q=choices[choose(#choices)];add(q.p.x+q.d[2],q.p.y+q.d[3],q.p,q.d)
    end
    local function distances()
      local dist={[m.start]=0};local queue={m.cells[1]};local index=1
      while queue[index] do local p=queue[index];index=index+1
        for _,d in ipairs(dirs) do local id=p.links[d[1]]
          if id and not dist[id] then dist[id]=dist[p.id]+1;queue[#queue+1]=m.cells[tonumber(id:sub(2))] end
        end
      end;return dist
    end
    local dist=distances()
    -- Loops connect neighbours whose depths differ by at most one: they cannot shorten the main
    -- path or change the distance-based difficulty curve.
    if options.loops~=false then for _,p in ipairs(m.cells) do for _,d in ipairs(dirs) do
      local other=occupied[key(p.x+d[2],p.y+d[3])]
      if other and not p.links[d[1]] and math.abs(dist[p.id]-dist[other.id])<=1 and
        p.id~=m.goal and other.id~=m.goal and choose(3)==1 then
        -- Adjacent grid cells always have opposite parity; equal depths cannot
        -- occur, so allow a one-depth difference (still no shorter route).
        p.links[d[1]]=other.id;other.links[d[4]]=p.id
      end
    end end end
    for _,p in ipairs(m.cells) do
      local needed='';local degree=0
      for _,d in ipairs(dirs) do if p.links[d[1]] then needed=needed..d[5];degree=degree+1 end end
      local candidates={}
      local templates=options.templates or set.templates
      local function declared(t,side,letter)
        local exits=t.exits or (t.level and t.level.exits)
        if exits then
          for _,e in ipairs(exits) do
            if e.side==side and e.slot==1 and not e.sealed then return e end
          end
          return nil
        end
        if t.sides:find(letter,1,true) then
          for _,e in ipairs(set.exits) do if e.side==side then return e end end
        end
      end
      local function fits(t)
        for _,d in ipairs(dirs) do if p.links[d[1]] then
          local e=declared(t,d[1],d[5]);if not e then return false end
          if (d[1]=='left' or d[1]=='right') and e.traversal~='walk' then return false end
          if d[1]=='down' and e.traversal~='drop' then return false end
          if d[1]=='up' and (e.traversal~='climb' and e.traversal~='drop') then return false end
          if p.forward.up and d[1]=='up' and e.traversal~='climb' then return false end
        end end;return true
      end
      for _,t in ipairs(templates) do
        if fits(t) and t.id~='start' and t.id~='goal' and t.id~='boss_room' and t.id~='reward_end' then candidates[#candidates+1]=t end
      end
      assert(#candidates>0,'no traversable recipe for '..p.id..' '..needed)
      local template=candidates[choose(#candidates)]
      local roles={}
      for _,t in ipairs(templates) do roles[t.id]=t end
      if p.id==m.start then template=assert(roles.start,'missing start recipe')
      elseif p.id==m.goal then template=assert(roles[options.boss and 'boss_room' or 'goal'],'missing goal recipe')
      elseif degree==1 then template=assert(roles.reward_end,'missing reward recipe');p.reward=true end
      assert(fits(template),'role recipe lacks required traversable exit')
      if p.reward then p.reward_amount=25 end
      p.template=template.id;p.tags=copy(template.tags)
      if template.platforms then p.platforms=copy(template.platforms) end
      p.spawn=copy(template.spawn or (template.level and template.level.spawn and template.level.spawn[0]) or set.spawn)
      p.enemy_slots=copy(template.enemy_slots or set.enemy_slots)
      p.distance=dist[p.id];p.difficulty=1+dist[p.id]/math.max(1,dist[m.goal])
      p.enemy_budget=math.min(template.enemy_budget or 3,3,math.floor(dist[p.id]/3))
      assert(p.enemy_budget>=0 and p.enemy_budget%1==0 and #p.enemy_slots>=p.enemy_budget,'invalid enemy budget/slots')
      p.enemy_kinds=copy(template.enemy_kinds or set.enemy_kinds)
      assert(#p.enemy_kinds>0,'enemy kinds must not be empty')
      for i,e in ipairs(p.exits) do
        local authored=declared(template,e.side,dirs[i][5])
        if authored then e.traversal=authored.traversal end
        e.to=p.links[e.side];e.sealed=e.to==nil
      end
      p.links=nil;p.forward=nil;p.level=M.geometry(p,template.level)
    end
    return m
  end
  function M.ascii(m)
    local cells={};local minx,maxx,miny,maxy=0,0,0,0
    for _,c in ipairs(m.cells) do cells[c.x..','..c.y]=c
      minx=math.min(minx,c.x);maxx=math.max(maxx,c.x);miny=math.min(miny,c.y);maxy=math.max(maxy,c.y) end
    local rows={'maze seed='..m.seed..' size='..#m.cells..' (L R U D; *=sealed)'}
    for y=maxy,miny,-1 do local row={}
      for x=minx,maxx do local c=cells[x..','..y]
        if not c then row[#row+1]='[     ]' else
          local text=c.id==m.start and 'S' or c.id==m.goal and 'G' or c.reward and '$' or '.'
          for i,e in ipairs(c.exits) do text=text..(e.to and dirs[i][5] or '*') end
          row[#row+1]='['..text..']'
        end
      end;rows[#rows+1]=table.concat(row,'')
    end;return table.concat(rows,'\n')
  end
  function M.starters()
    local files,templates={},copy(set.templates)
    for _,t in ipairs(templates) do
      local c={id=t.id,x=0,y=0,platforms=copy(set.platforms),exits=copy(set.exits)}
      for i,e in ipairs(c.exits) do e.sealed=not t.sides:find(dirs[i][5],1,true) end
      local level=M.geometry(c);level.size=copy(set.size);level.exits=copy(c.exits)
      level.tags=copy(t.tags);level.spawn={[0]=copy(set.spawn)}
      t.level=level;t.platforms=copy(set.platforms)
      files['maze-chunks/'..t.id..'/level.lua']='return '..M.encode(level)..'\n'
    end
    files['maze-chunks/library.lua']='return '..M.encode(templates)..'\n'
    return files
  end
  function M.files(m,name)
    assert(name:match('^[%w_-]+$'),'unsafe folder name')
    local report=D.maze_check.check(m);assert(report.ok,table.concat(report.errors,'; '))
    local root={version=2,units=6.5,parts={},chunks={},camera={mode='chunk'},markers={}}
    local mission={enemies={},triggers={},objective={type='reach_goal'}}
    local files={};local folder='missions/'..name..'/'
    local bounds={left=0,right=130,bottom=0,top=104}
    for _,c in ipairs(m.cells) do local x,y=c.x*130,c.y*104
      local spawn={x=x+c.spawn.x,y=y+c.spawn.y}
      root.chunks[#root.chunks+1]={id=c.id,rect={left=x,right=x+130,bottom=y,top=y+104},spawn=spawn}
      root.markers[#root.markers+1]={name=c.id,x=spawn.x,y=spawn.y}
      bounds.left=math.min(bounds.left,x);bounds.right=math.max(bounds.right,x+130)
      bounds.bottom=math.min(bounds.bottom,y);bounds.top=math.max(bounds.top,y+104)
      if c.id==m.start then mission.start=spawn end
      if c.id==m.goal then mission.goal={x=x+104,y=y+12,w=24,h=32} end
      if c.reward then mission.triggers[#mission.triggers+1]={x=x+20,y=y+12,w=24,h=32,
        action='message',text='Reward cache: '..c.id,once=true} end
      for i=1,c.enemy_budget do local p=c.enemy_slots[i]
        assert(p.x>=0 and p.x<130 and p.y>=0 and p.y<104,'enemy slot outside chunk')
        mission.enemies[#mission.enemies+1]={x=x+p.x,y=y+p.y,kind=c.enemy_kinds[(i-1)%#c.enemy_kinds+1],
          wave=math.floor((#mission.enemies)/32)+1}
      end
      local child=copy(c.level);child.size=copy(set.size);child.exits=copy(c.exits)
      child.maze={template=c.template,tags=c.tags,difficulty=c.difficulty,enemy_budget=c.enemy_budget}
      files[folder..'chunks/'..c.id..'/level.lua']='return '..M.encode(child)..'\n'
    end
    root.blast={left=bounds.left-200,right=bounds.right+200,bottom=bounds.bottom-200,top=bounds.top+200}
    root.maze=copy(m)
    files[folder..'level.lua']='return '..M.encode(root)..'\n'
    files[folder..'mission.lua']='return '..M.encode(mission)..'\n'
    return files
  end
  return M
end
