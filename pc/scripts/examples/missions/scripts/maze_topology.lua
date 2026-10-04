-- Seeded graph layout: a loop core, self-avoiding route and sustained wrong turns.
return function(D)
  local T={};local dirs={{'left',-1,0,'right','L'},{'right',1,0,'left','R'},{'up',0,1,'down','U'},{'down',0,-1,'up','D'}}
  function T.generate(seed,options,choose,copy)
    local size,length=options.size,options.length
    local m={version=2,seed=seed,size=size,cells={},main_path={},start='c1'};local occupied={}
    local canclimb=not options.templates
    for _,t in ipairs(options.templates or {})do
      if not t.exits and not t.level and t.sides and t.sides:find('U',1,true)then canclimb=true end
      for _,e in ipairs(t.exits or(t.level and t.level.exits)or{})do if e.side=='up'and e.traversal=='climb'then canclimb=true end end end
    local function key(x,y)return x..','..y end
    local function add(x,y,parent,d,branch)
      local c={id='c'..(#m.cells+1),x=x,y=y,exits=copy(D.maze_set.exits),platforms=copy(D.maze_set.platforms),links={},forward={},branch=branch}
      m.cells[#m.cells+1]=c;occupied[key(x,y)]=c
      if parent then c.links[d[4]]=parent.id;parent.links[d[1]]=c.id;parent.forward[d[1]]=true end
      return c
    end
    local c=add(0,0);m.main_path[1]=c.id;local depth=2
    if options.loops~=false and canclimb then
      local a=dirs[choose(4)];local perpendicular={}
      for _,d in ipairs(dirs)do if d[2]*a[2]+d[3]*a[3]==0 then perpendicular[#perpendicular+1]=d end end
      local b=perpendicular[choose(2)];local opposite
      for _,d in ipairs(dirs)do if d[1]==a[4]then opposite=d end end
      for _,d in ipairs({a,b,opposite})do c=add(c.x+d[2],c.y+d[3],c,d);m.main_path[depth]=c.id;depth=depth+1 end
      c.links[b[4]]=m.start;m.cells[1].links[b[1]]=c.id
    end
    local search=0
    local function choices(p)
      local out={};for _,d in ipairs(dirs)do if (canclimb or d[1]~='up')and not occupied[key(p.x+d[2],p.y+d[3])]then out[#out+1]=d end end;return out
    end
    local function grow(parent,n)
      if n>length then c=parent;return true end
      search=search+1;if search>512 then return false end
      local out=choices(parent)
      while #out>0 do local d=table.remove(out,choose(#out));local child=add(parent.x+d[2],parent.y+d[3],parent,d);m.main_path[n]=child.id
        if grow(child,n+1)then return true end
        occupied[key(child.x,child.y)]=nil;table.remove(m.cells);m.main_path[n]=nil;parent.links[d[1]]=nil;parent.forward[d[1]]=nil
      end;return false
    end
    if not grow(c,depth)then
      local out=choices(c);local d=out[choose(#out)]
      for n=depth,length do c=add(c.x+d[2],c.y+d[3],c,d);m.main_path[n]=c.id end
    end
    m.goal=c.id
    local last,age,limit=nil,0,0
    while #m.cells<size do
      local out=last and age<limit and choices(last)or{}
      local parent=last
      if #out==0 then
        local roots={};for _,p in ipairs(m.cells)do if p.id~=m.goal then for _,d in ipairs(choices(p))do roots[#roots+1]={p=p,d=d}end end end
        local q=roots[choose(#roots)];parent=q.p;out={q.d};age=0;limit=math.min(size-#m.cells,3+choose(4))
      end
      local d=out[choose(#out)];last=add(parent.x+d[2],parent.y+d[3],parent,d,true);age=age+1
    end
    if options.loops~=false then for _,p in ipairs(m.cells)do if not p.branch then for _,d in ipairs(dirs)do
      local other=occupied[key(p.x+d[2],p.y+d[3])]
      if other and not other.branch and not p.links[d[1]]and choose(4)~=1 then p.links[d[1]]=other.id;other.links[d[4]]=p.id end
    end end end end
    local dist={[m.start]=0};local queue={m.cells[1]};local i=1
    while queue[i]do local p=queue[i];i=i+1;for _,d in ipairs(dirs)do local id=p.links[d[1]]
      if id and dist[id]==nil then dist[id]=dist[p.id]+1;queue[#queue+1]=m.cells[tonumber(id:sub(2))]end
    end end
    return m,dist
  end
  return T
end
