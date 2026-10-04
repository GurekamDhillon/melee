-- Whole-world reachability and every regional connector pair, using directed edges.
return function(D)
  local W={}
  function W.check(m,work)
    local proof=D.maze_check.check(m,work);local world=m.world
    if not world then proof.errors[#proof.errors+1]='missing world metadata';proof.ok=false;return proof end
    local ids,ports,regions={},{},{}
    for _,c in ipairs(m.cells)do ids[c.id]=c end
    for _,r in ipairs(world.regions)do regions[r.id]=r;ports[r.id]={}end
    local function reach(start,region)
      local seen={[start]=true};local q={start};local i=1
      while q[i]do local c=ids[q[i]];i=i+1
        for _,e in ipairs(c and c.exits or {})do
          local pass=e.side=='left'or e.side=='right'or e.side=='down'or e.traversal=='climb'
          if pass and e.to and ids[e.to]and(not region or ids[e.to].region==region)and not seen[e.to]then seen[e.to]=true;q[#q+1]=e.to end
        end
      end;return seen
    end
    local reverse={};for _,c in ipairs(m.cells)do reverse[c.id]={}end
    for _,c in ipairs(m.cells)do for _,e in ipairs(c.exits)do
      local pass=e.side=='left'or e.side=='right'or e.side=='down'or e.traversal=='climb'
      if pass and e.to and reverse[e.to]then reverse[e.to][#reverse[e.to]+1]=c.id end
    end end
    local escape={[m.goal]=true};local queue={m.goal};local index=1
    while queue[index]do for _,id in ipairs(reverse[queue[index]]or{})do if not escape[id]then escape[id]=true;queue[#queue+1]=id end end;index=index+1 end
    for _,c in ipairs(m.cells)do if not escape[c.id]then proof.errors[#proof.errors+1]='soft lock: '..c.id..' cannot reach goal'end end
    for _,link in ipairs(world.links)do
      if not regions[link.a]or not regions[link.b]or not ids[link.from]or not ids[link.to]then proof.errors[#proof.errors+1]='invalid world connector '..tostring(link.name)
      else ports[link.a][#ports[link.a]+1]=link.from;ports[link.b][#ports[link.b]+1]=link.to
        local chain={link.from};for _,id in ipairs(link.chunks or{})do chain[#chain+1]=id end;chain[#chain+1]=link.to
        if ids[link.from].region~=link.a or ids[link.to].region~=link.b then proof.errors[#proof.errors+1]='connector region mismatch '..link.name end
        local function pass(from,to)
          for _,e in ipairs(ids[from]and ids[from].exits or{})do
            if e.to==to and not e.sealed and(e.side=='left'or e.side=='right'or e.side=='down'or e.traversal=='climb')then return true end
          end
        end
        for i=2,#chain do
          if not pass(chain[i-1],chain[i])then proof.errors[#proof.errors+1]='broken forward connector '..link.name..' '..chain[i-1]..' -> '..chain[i]end
          if not link.oneway and not pass(chain[i],chain[i-1])then proof.errors[#proof.errors+1]='broken return connector '..link.name..' '..chain[i]..' -> '..chain[i-1]end
        end
        local seen=reach(link.from);if not seen[link.to]then proof.errors[#proof.errors+1]='broken world connector '..link.name end
      end
    end
    proof.region_pairs=0
    for _,r in ipairs(world.regions)do
      local list=ports[r.id];list[#list+1]=r.start;list[#list+1]=r.goal
      for _,a in ipairs(list)do local seen=reach(a,r.id)
        for _,z in ipairs(list)do if not seen[z]then proof.errors[#proof.errors+1]=r.id..': connector pair '..a..' -> '..z..' unreachable'else proof.region_pairs=proof.region_pairs+1 end end
        if work then work('connector proof')end
      end
    end
    proof.ok=#proof.errors==0;return proof
  end
  return W
end
