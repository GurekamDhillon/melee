-- Regions stay independently seeded; connectors form one continuous streamed graph.
return function(D)
  local W={MAX_CHUNKS=512,MAX_COORD=40000,MAX_REGIONS=6}
  local dirs={{'left',-1,0,2},{'right',1,0,1},{'up',0,1,4},{'down',0,-1,3}}
  local function copy(t)if type(t)~='table'then return t end;local q={};for k,v in pairs(t)do q[k]=copy(v)end;return q end
  function W.generate(seed,options)
    options=options or{};local n,size=options.regions or 4,options.size or 12;local work=options.work
    assert(type(seed)=='number'and seed==seed and seed%1==0 and math.abs(seed)<=2147483646,'world seed must be signed31-bit integer')
    assert(n%1==0 and n>=2 and n<=W.MAX_REGIONS,'world regions must be2..6');assert(size%1==0 and size>=8 and size<=20,'region size must be8..20')
    local m={version=2,seed=seed,cells={},main_path={},world={seed=seed,size=size,start='r1',goal='r'..n,regions={},links={}}}
    local occupied,authored,recipes={},{},{};local function key(x,y)return x..','..y end
    local function append(c)
      assert(not occupied[key(c.x,c.y)],'world overlap '..c.id);m.cells[#m.cells+1]=c;occupied[key(c.x,c.y)]=c
      assert(#m.cells<=W.MAX_CHUNKS,'world exceeds512 chunks; use a real level change')
      assert(math.abs(c.x*130)+130<W.MAX_COORD and math.abs(c.y*104)+104<W.MAX_COORD,'world exceeds40000-unit bounds; use a real level change')
    end
    for i=1,n do
      local id='r'..i;local derived=((seed%2147483647+i*104729)*48271)%2147483647
      local rseed=options.seeds and options.seeds[id]or derived
      local themes={'gallery','vault','well','switchback'};local theme=themes[(derived%4)+1]
      local templates={}
      for _,t in ipairs(options.templates or D.maze_set.templates)do
        local role=t.id=='start'or t.id=='goal'or t.id=='boss_room'or t.id=='reward_end'
        if role or t.id==theme or not(t.id=='gallery'or t.id=='vault'or t.id=='well'or t.id=='switchback')then templates[#templates+1]=t end
      end
      local byname={};for _,t in ipairs(templates)do byname[t.id]=t end
      local localmaze=D.maze.generate(rseed,{size=size,templates=templates,boss=i==n,loops=true,work=work})
      local region={id=id,seed=rseed,theme=i==n and 'boss'or theme,chunks={},map=D.maze.ascii(localmaze)}
      local rename={};for j,c in ipairs(localmaze.cells)do rename[c.id]='c'..(#m.cells+j)end
      local quota=64//n+(i<=64%n and 1 or 0)
      local left,right,bottom,top=math.huge,-math.huge,math.huge,-math.huge
      for _,source in ipairs(localmaze.cells)do local c=copy(source);c.level=nil;c.id=rename[c.id];c.region=id
        recipes[c.id]=byname[c.template];authored[c.id]=recipes[c.id]and recipes[c.id].level
        c.x=c.x+(i-1)*(2*size+6)+size;c.y=c.y+size
        if i~=n and c.template=='goal'then c.template='gallery'
          if authored[c.id]then authored[c.id]=copy(authored[c.id]);local keep={};for _,p in ipairs(authored[c.id].parts or{})do if not(p.name and p.name:find('goal_marker',1,true))then keep[#keep+1]=p end end;authored[c.id].parts=keep end
        end
        for _,e in ipairs(c.exits)do if e.to then e.to=rename[e.to]end end
        c.enemy_budget=math.min(c.enemy_budget,quota);quota=quota-c.enemy_budget
        append(c);region.chunks[#region.chunks+1]=c.id
        left=math.min(left,c.x);right=math.max(right,c.x+1);bottom=math.min(bottom,c.y);top=math.max(top,c.y+1)
      end
      region.start=rename[localmaze.start];region.goal=rename[localmaze.goal];region.grid={left=left,right=right,bottom=bottom,top=top}
      region.rect={left=left*130,right=right*130,bottom=bottom*104,top=top*104};m.world.regions[i]=region
      if i==1 then m.start=region.start end;if i==n then m.goal=region.goal end
      if work then work('region '..id)end
    end
    local edges={};for i=1,n-1 do edges[#edges+1]={i,i+1,2,1}end
    if n>=3 then table.insert(edges,n,{1,n==3 and n or n-1,3,3})end
    local used={}
    local function port(region,preferred)
      local order={preferred};for side=1,4 do if side~=preferred then order[#order+1]=side end end
      for _,side in ipairs(order)do local d=dirs[side];local best,candidates=nil,{}
        for _,id in ipairs(region.chunks)do local c=m.cells[tonumber(id:sub(2))];local value=c.x*d[2]+c.y*d[3]
          if not best or value>best then best=value;candidates={c}elseif value==best then candidates[#candidates+1]=c end
        end
        for _,c in ipairs(candidates)do
          local recipe=recipes[c.id];local capable=not recipe
          local exits=recipe and(recipe.exits or(recipe.level and recipe.level.exits))
          if exits then for _,e in ipairs(exits)do if e.side==d[1]and not e.sealed then capable=true end end
          elseif recipe and recipe.sides then capable=recipe.sides:find(d[1]:sub(1,1):upper(),1,true)~=nil end
          if capable and not used[c.id..':'..side]and not occupied[key(c.x+d[2],c.y+d[3])]then used[c.id..':'..side]=true;return c,side end
        end
      end
      error('no connector-capable exterior port for '..region.id)
    end
    local function stitch(a,z)
      local side;for i,d in ipairs(dirs)do if z.x==a.x+d[2]and z.y==a.y+d[3]then side=i end end
      assert(side,'connector cells are not adjacent');local opposite=dirs[side][4]
      assert(not a.exits[side].to and not z.exits[opposite].to,'connector reuses an occupied exit')
      a.exits[side].to=z.id;a.exits[side].sealed=false;z.exits[opposite].to=a.id;z.exits[opposite].sealed=false
      if side==3 then a.exits[side].traversal='climb'elseif side==4 then z.exits[opposite].traversal='climb'end
    end
    for index,edge in ipairs(edges)do
      local a,z=m.world.regions[edge[1]],m.world.regions[edge[2]];local from,aside=port(a,edge[3]);local to,zside=port(z,edge[4]);local da,dz=dirs[aside],dirs[zside]
      local start={x=from.x+da[2],y=from.y+da[3]};local finish={x=to.x+dz[2],y=to.y+dz[3]}
      local function blocked(x,y)
        if occupied[key(x,y)]then return true end
        for _,r in ipairs(m.world.regions)do local b=r.grid;if x>=b.left and x<b.right and y>=b.bottom and y<b.top then return true end end
        return false
      end
      local path=D.maze_route.find(start,finish,blocked,{left=-size-8,right=n*(2*size+6),bottom=-size-8,top=3*size+24},work)
      local link={name='connector'..index,a=a.id,b=z.id,from=from.id,to=to.id,chunks={}}
      local previous=from
      for _,p in ipairs(path)do local c={id='c'..(#m.cells+1),x=p.x,y=p.y,template='gallery',connector=link.name,
        tags={'connector'},enemy_budget=0,enemy_slots=copy(D.maze_set.enemy_slots),enemy_kinds=copy(D.maze_set.enemy_kinds),
        platforms=copy(D.maze_set.platforms),spawn=copy(D.maze_set.spawn),exits=copy(D.maze_set.exits),difficulty=1,distance=0}
        for _,e in ipairs(c.exits)do e.sealed=true end
        append(c);link.chunks[#link.chunks+1]=c.id;stitch(previous,c);previous=c
        if work then work('connector chunk')end
      end
      stitch(previous,to);m.world.links[#m.world.links+1]=link
    end
    for _,c in ipairs(m.cells)do c.level=D.maze.geometry(c,authored[c.id]);if work then work('geometry')end end
    m.size=#m.cells
    local parents={[m.start]=false};local queue={m.start};local i=1
    while queue[i]do local c=m.cells[tonumber(queue[i]:sub(2))];i=i+1
      for _,e in ipairs(c.exits)do if e.to and parents[e.to]==nil then parents[e.to]=c.id;queue[#queue+1]=e.to end end
    end
    local at=m.goal;while at do table.insert(m.main_path,1,at);at=parents[at]end
    m.metrics=D.maze_metrics.measure(m);local proof=D.maze_world_check.check(m,work);assert(proof.ok,table.concat(proof.errors,'; '))
    return m
  end
  function W.ascii(m)
    local rows={'world seed='..m.seed..' start='..m.world.start..' goal='..m.world.goal..' chunks='..#m.cells,'connectors:'}
    local ids={};for _,c in ipairs(m.cells)do ids[c.id]=c end
    for _,e in ipairs(m.world.links)do rows[#rows+1]=e.a..(e.oneway and ' -> 'or ' <-> ')..e.b..' ['..e.name..', '..#e.chunks..' chunks; '..e.from..' / '..e.to..']'end
    for _,r in ipairs(m.world.regions)do
      rows[#rows+1]=r.id..' seed='..r.seed..' theme='..r.theme
      local grid={};for _,id in ipairs(r.chunks)do local c=ids[id];grid[c.x..','..c.y]=c end
      for y=r.grid.top-1,r.grid.bottom,-1 do local row={}
        for x=r.grid.left,r.grid.right-1 do local c=grid[x..','..y]
          if not c then row[#row+1]='[     ]'else
            local text=c.id==m.start and 'S'or c.id==m.goal and 'G'or c.reward and '$'or '.'
            for i,e in ipairs(c.exits)do text=text..(e.to and dirs[i][1]:sub(1,1):upper()or '*')end
            row[#row+1]='['..text..']'
          end
        end;rows[#rows+1]=table.concat(row,'')
      end
    end
    return table.concat(rows,'\n')
  end
  return W
end
