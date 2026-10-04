-- Pure generation/serialization. Python invokes this exact implementation.
return function(D)
  local M={};local set=D.maze_set
  local dirs={{'left',-1,0,'right','L'},{'right',1,0,'left','R'},
    {'up',0,1,'down','U'},{'down',0,-1,'up','D'}}
  local function copy(t) if type(t)~='table' then return t end
    local q={};for k,v in pairs(t) do q[k]=copy(v) end;return q end
  function M.encode(t,work)
    if type(t)=='string' then return string.format('%q',t) end
    if type(t)~='table' then return tostring(t) end
    if work then work('encode')end
    local keys={};for k in pairs(t) do keys[#keys+1]=k end
    table.sort(keys,function(a,b) if type(a)==type(b) then return a<b end return type(a)<type(b) end)
    local q={};for _,k in ipairs(keys) do q[#q+1]='['..M.encode(k,work)..']='..M.encode(t[k],work) end
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
      for _,z in ipairs(q.zones or{})do z.name=c.id..'_'..z.name;shift(z.rect)end
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
    if not authored then
      for i,side in ipairs({0,130})do q.parts[#q.parts+1]={part=i==1 and 'bf_wall_solid_4m' or 'bf_wall_doorway_4m',
        name=c.id..'_wall'..i,x=x+side,y=y+32,z=0,rot=0,scale_x=0.12,scale_y=72/26,collision=false,floor_flags=0}end
      if c.template=='goal'or c.template=='boss_room'then
        q.parts[#q.parts+1]={part='bf_door_leaf',name=c.id..'_goal_marker',x=x+104,y=y,z=0,rot=0,collision=false,floor_flags=0}
      end
      -- Visually distinguish vertical/turn recipes using the existing beam kit.
      if c.template=='shaft'or c.template=='corner_up'or c.template=='corner_down'or c.template=='gallery'or c.template=='well'then q.parts[3].part='bf_beam_4m' end
      if c.template=='vault'or c.template=='switchback'then q.parts[4].part='bf_beam_4m' end
    end
    local keep={}
    for _,l in ipairs(q.lines)do
      local remove=l.label=='walk_bridge'
      if l.label=='climb'and c.exits[3].sealed and l.y1-y>54 then remove=true end
      if l.kind=='ceiling'and l.y1==l.y2 and ((l.y1==y+104 and c.exits[3].sealed)or(l.y1==y and c.exits[4].sealed))then
        if l.x1==x+43 and l.x2==x then l.x1=x+53 end
        if l.x1==x+130 and l.x2==x+87 then l.x2=x+77 end
      end
      if l.kind=='ceiling'and l.y1==l.y2 and ((l.y1==y+104 and not c.exits[3].sealed)or(l.y1==y and not c.exits[4].sealed))then
        if l.x1==x+53 and l.x2==x then l.x1=x+43 end
        if l.x1==x+130 and l.x2==x+77 then l.x2=x+87 end
      end
      if not remove then keep[#keep+1]=l end
    end
    q.lines=keep;keep={}
    for _,p in ipairs(q.parts)do
      local remove=p.name and p.name:find('drop_bridge',1,true)
      if c.exits[3].sealed and p.part==set.part and p.y-y>54 and p.y-y<104 then remove=true end
      if not remove then keep[#keep+1]=p end
    end
    q.parts=keep
    return q
  end
  function M.generate(seed,options,random)
    options=options or {};local size=options.size or 12
    assert(type(seed)=='number' and seed==seed and seed%1==0 and math.abs(seed)<=2147483646,'seed must be signed 31-bit integer')
    assert(type(size)=='number' and size%1==0 and size>=8 and size<=20,'size must be 8..20')
    local length=options.length or math.max(4,math.ceil(size*.55))
    assert(length%1==0 and length>=4 and length<=size,'length must be 4..size')
    local rng=random or M.random(seed)
    local function choose(n) local i=rng(n);assert(type(i)=='number' and i%1==0 and i>=1 and i<=n,'RNG out of bounds');return i end
    options=copy(options);options.size=size;options.length=length
    local m,dist=D.maze_topology.generate(seed,options,choose,copy)
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
      local different={}
      for _,t in ipairs(candidates)do local repeat_room=false
        for _,id in pairs(p.links)do local old=m.cells[tonumber(id:sub(2))];if old.template==t.id then repeat_room=true end end
        if not repeat_room then different[#different+1]=t end
      end
      if #different>0 then candidates=different end
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
      if options.work then options.work('region geometry')end
    end
    if D.maze_clearance then local proof=D.maze_clearance.check(m,options.work);assert(proof.ok,table.concat(proof.errors,'; '))end
    m.metrics=D.maze_metrics.measure(m)
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
    end
    local q=m.metrics or D.maze_metrics.measure(m)
    rows[#rows+1]=('shape branches=%d dead_end=%d turns=%d cycles=%d L/R/U/D=%d/%d/%d/%d'):format(q.branches,q.longest_dead_end,q.turns,q.cycles,q.directions.left,q.directions.right,q.directions.up,q.directions.down)
    return table.concat(rows,'\n')
  end
  function M.starters()
    local files,templates={},copy(set.templates)
    for _,t in ipairs(templates) do
      local c={id=t.id,template=t.id,x=0,y=0,platforms=copy(set.platforms),exits=copy(set.exits)}
      for i,e in ipairs(c.exits) do e.sealed=not t.sides:find(dirs[i][5],1,true) end
      local level=M.geometry(c);level.size=copy(set.size);level.exits=copy(c.exits)
      level.tags=copy(t.tags);level.spawn={[0]=copy(set.spawn)}
      t.level=level;t.platforms=copy(set.platforms)
      files['missions/maze-chunks/'..t.id..'/level.lua']='return '..M.encode(level)..'\n'
    end
    files['missions/maze-chunks/library.lua']='return '..M.encode(templates)..'\n'
    return files
  end
  function M.files(m,name,work)
    assert(name:match('^[%w_-]+$'),'unsafe folder name')
    local report=m.world and D.maze_world_check.check(m,work)or D.maze_check.check(m,work);assert(report.ok,table.concat(report.errors,'; '))
    local root={version=2,units=6.5,parts={},chunks={},camera={mode='chunk'},markers={},zones={}}
    local mission={enemies={},triggers={},objective={type='reach_goal'}}
    local files={};local folder='missions/'..name..'/'
    local owner,waves=D.maze_encounters.plan(m,work)
    for w=1,waves do mission.triggers[#mission.triggers+1]={x=99999,y=99999,w=1,h=1,action='wave',wave=w}end
    local bounds={left=0,right=130,bottom=0,top=104}
    for _,r in ipairs(m.world and m.world.regions or {})do root.zones[#root.zones+1]={name='region_'..r.id,kind='region',region=r.id,rect=copy(r.rect)}end
    for _,c in ipairs(m.cells) do local x,y=c.x*130,c.y*104
      local spawn={x=x+c.spawn.x,y=y+c.spawn.y}
      root.chunks[#root.chunks+1]={id=c.id,rect={left=x,right=x+130,bottom=y,top=y+104},spawn=spawn}
      root.zones[#root.zones+1]={name='room_'..c.id,kind='room',room=c.id,rect={left=x,right=x+130,bottom=y,top=y+104}}
      for _,e in ipairs(c.exits)do if e.to and tonumber(c.id:sub(2))<tonumber(e.to:sub(2))then
        local rect
        if e.side=='left'or e.side=='right'then local border=x+(e.side=='right'and 130 or 0)
          rect={left=border-8,right=border+8,bottom=y,top=y+32}
        else local border=y+(e.side=='up'and 104 or 0)
          rect={left=x+53,right=x+77,bottom=border-12,top=border+12}
        end
        root.zones[#root.zones+1]={name='door_'..c.id..'_'..e.to,kind='transition',rooms={c.id,e.to},rect=rect,
          camera={ease_frames=24,curve='smoothstep',doorway_margin=12}}
      end end
      root.markers[#root.markers+1]={name=c.id,x=spawn.x,y=spawn.y}
      bounds.left=math.min(bounds.left,x);bounds.right=math.max(bounds.right,x+130)
      bounds.bottom=math.min(bounds.bottom,y);bounds.top=math.max(bounds.top,y+104)
      if c.id==m.start then mission.start=spawn end
      if c.id==m.goal then mission.goal={x=x+104,y=y+12,w=24,h=32} end
      for i=1,c.enemy_budget do local p=c.enemy_slots[i]
        assert(p.x>=0 and p.x<130 and p.y>=0 and p.y<104,'enemy slot outside chunk')
        mission.enemies[#mission.enemies+1]={x=x+p.x,y=y+p.y,kind=c.enemy_kinds[(i-1)%#c.enemy_kinds+1],
          wave=owner[c.id]}
      end
      local child=copy(c.level);child.size=copy(set.size);child.exits=copy(c.exits)
      child.maze={template=c.template,tags=c.tags,difficulty=c.difficulty,enemy_budget=c.enemy_budget}
      files[folder..'chunks/'..c.id..'/level.lua']='return '..M.encode(child,work)..'\n'
    end
    root.blast={left=bounds.left-200,right=bounds.right+200,bottom=bounds.bottom-200,top=bounds.top+200}
    root.maze={};for k,v in pairs(m)do if k~='cells'then root.maze[k]=copy(v)end end
    root.maze.cells={}
    for _,c in ipairs(m.cells)do local q={};for k,v in pairs(c)do if k~='level'then q[k]=copy(v)end end
      q.enemy_wave=owner[c.id];root.maze.cells[#root.maze.cells+1]=q;if work then work('metadata')end
    end
    files[folder..'level.lua']='return '..M.encode(root,work)..'\n'
    files[folder..'mission.lua']='return '..M.encode(mission,work)..'\n'
    return files
  end
  return M
end
