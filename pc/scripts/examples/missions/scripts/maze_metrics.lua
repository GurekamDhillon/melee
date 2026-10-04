-- Shape measurements independent of geometry/generation.
return function()
  local T={}
  function T.measure(m)
    local ids,degree,edges={},{},0
    for _,c in ipairs(m.cells)do ids[c.id]=c;degree[c.id]=0
      for _,e in ipairs(c.exits)do if e.to then degree[c.id]=degree[c.id]+1;edges=edges+1 end end
    end
    local q={branches=0,longest_dead_end=0,turns=0,cycles=edges/2-#m.cells+1,directions={left=0,right=0,up=0,down=0},adjacent_repeats=0}
    for _,c in ipairs(m.cells)do
      if degree[c.id]>=3 then q.branches=q.branches+1 end
      for _,e in ipairs(c.exits)do if e.to and ids[e.to]and c.id<e.to and c.template==ids[e.to].template then q.adjacent_repeats=q.adjacent_repeats+1 end end
      if degree[c.id]==1 and c.id~=m.start and c.id~=m.goal then
        local at,previous,length=c,nil,0;local seen={}
        while at and not seen[at.id]do
          seen[at.id]=true
          if degree[at.id]>2 then break end
          local nextid;for _,e in ipairs(at.exits)do if e.to and e.to~=previous then nextid=e.to;break end end
          if not nextid then break end
          length=length+1;previous=at.id;at=ids[nextid]
        end
        q.longest_dead_end=math.max(q.longest_dead_end,length)
      end
    end
    local previous
    for i=2,#(m.main_path or {})do local a,b=ids[m.main_path[i-1]],ids[m.main_path[i]]
      local direction=b.x>a.x and 'right'or b.x<a.x and 'left'or b.y>a.y and 'up'or 'down'
      q.directions[direction]=q.directions[direction]+1
      if previous and previous~=direction then q.turns=q.turns+1 end;previous=direction
    end
    return q
  end
  return T
end
