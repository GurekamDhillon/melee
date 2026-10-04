-- Independent proof over emitted exits and actual collision lines. No generator calls.
return function(D)
  local C={}
  local vectors={left={-1,0,'right'},right={1,0,'left'},up={0,1,'down'},down={0,-1,'up'}}
  local function hasline(c,kind,x1,y1,x2,y2,label,pass)
    for _,l in ipairs(c.level and c.level.lines or {}) do
      if l.kind==kind and l.x1==x1 and l.y1==y1 and l.x2==x2 and l.y2==y2 and
        (not label or l.label==label) and (pass==nil or l.passthrough==pass) then return true end
    end
    return false
  end
  local function climb(c)
    local previous={x1=0,x2=53,y=0}
    if #(c.platforms or {})==0 then return false end
    for _,p in ipairs(c.platforms) do
      if not (type(p.y)=='number' and p.y>previous.y and p.y-previous.y<=20 and
        p.x1<p.x2 and p.x2-p.x1>=12 and p.x1>=0 and p.x2<=130) then return false end
      if math.max(p.x1-previous.x2,previous.x1-p.x2,0)>12 then return false end
      if not hasline(c,'floor',c.x*130+p.x1,c.y*104+p.y,c.x*130+p.x2,c.y*104+p.y) then return false end
      previous=p
    end
    return 104-previous.y>=0 and 104-previous.y<=20 and previous.x1<=77 and previous.x2>=53
  end
  function C.check(m,work)
    local errors,ids,grid,graph,costs,actions={},{},{},{},{},{}
    local function error(s) errors[#errors+1]=s end
    for _,c in ipairs(m.cells or {}) do
      if ids[c.id] then error('duplicate chunk '..c.id) end;ids[c.id]=c;graph[c.id]={};costs[c.id]={};actions[c.id]={}
      if type(c.x)~='number' or type(c.y)~='number' or c.x%1~=0 or c.y%1~=0 then error('invalid grid') end
      local key=tostring(c.x)..','..tostring(c.y)
      if grid[key] then error('overlap '..c.id) end;grid[key]=c.id
    end
    if not ids[m.start] then error('missing start') end
    if not ids[m.goal] then error('missing goal') end
    for _,c in ipairs(m.cells or {}) do
      local slots={}
      for _,e in ipairs(c.exits or {}) do
        local v=vectors[e.side];local slot=e.side..':'..tostring(e.slot)
        if not v or e.slot~=1 or slots[slot] then error('invalid exit slot '..slot)
        else
          slots[slot]=true
          if e.sealed then
            if e.to then error('sealed joined exit '..slot) end
            local x,y=c.x*130,c.y*104;local label='seal:'..slot
            local ok
            if e.side=='left' or e.side=='right' then
              if e.side=='right' then x=x+130 end
              ok=hasline(c,'left_wall',x,y,x,y+32,label) and hasline(c,'right_wall',x,y+32,x,y,label)
            else
              if e.side=='up' then y=y+104 end
              ok=hasline(c,'floor',x+53,y,x+77,y,label) and hasline(c,'ceiling',x+77,y,x+53,y,label)
            end
            if not ok then error('missing seal collision '..c.id..':'..slot) end
          elseif not e.to then error('unsealed exit '..c.id..':'..slot)
          else
            local other=ids[e.to];local mate
            if other then for _,f in ipairs(other.exits or {}) do
              if f.side==v[3] and f.slot==e.slot then mate=f end
            end end
            if not other or other.x~=c.x+v[1] or other.y~=c.y+v[2] or
              not mate or mate.to~=c.id or mate.sealed then error('mismatched exit slot '..c.id..':'..slot)
            else
              local pass=false
              if e.side=='left' or e.side=='right' then
                pass=e.traversal=='walk' and mate.traversal=='walk'
              elseif e.side=='down' then pass=e.traversal=='drop'
              elseif e.traversal=='climb' then
                pass=climb(c);if not pass then error('invalid climb platforms '..c.id) end
              elseif e.traversal~='drop' then error('invalid climb traversal '..c.id) end
              if pass then graph[c.id][#graph[c.id]+1]=e.to
                local hole=c.exits[4]and c.exits[4].to
                local hop=(e.side=='left'or e.side=='right')and hole
                actions[c.id][e.to]=hop and 'hop'or e.traversal
                costs[c.id][e.to]=e.traversal=='climb'and #(c.platforms or {})+1 or hop and 2 or 1
              end
            end
          end
        end
      end
      for side in pairs(vectors) do if not slots[side..':1'] then error('missing exit slot '..c.id..':'..side) end end
      if work then work('graph proof')end
    end
    local distance={};local queue={}
    if ids[m.start] then queue[1]=m.start;distance[m.start]=0 end
    local index=1
    while queue[index] do local id=queue[index];index=index+1
      for _,to in ipairs(graph[id]) do if distance[to]==nil then
        distance[to]=distance[id]+1;queue[#queue+1]=to end end
    end
    if distance[m.goal]==nil then error('goal unreachable') end
    local unreachable={}
    for _,c in ipairs(m.cells or {}) do if distance[c.id]==nil then unreachable[#unreachable+1]=c.id;error('unreachable chunk '..c.id) end end
    if D and D.maze_clearance then for _,s in ipairs(D.maze_clearance.check(m,work).errors)do error(s)end end
    return {ok=#errors==0,errors=errors,distance=distance,unreachable=unreachable,costs=costs,actions=actions}
  end
  function C.folder(g,name,work)
    assert(type(name)=='string' and name:match('^[%w_-]+$'),'unsafe mission name')
    local base='missions/'..name..'/'
    local function data(path)
      local text=assert(g.mod_read(path),'missing '..path)
      return assert(load(text,'@'..path,'t',{}))()
    end
    local root=data(base..'level.lua');local m=assert(root.maze,'folder lacks maze metadata')
    local ids={};for _,c in ipairs(m.cells) do ids[c.id]=c end
    local errors,seen={},{}
    for _,entry in ipairs(root.chunks) do
      if seen[entry.id] then errors[#errors+1]='duplicate chunk folder reference' end
      seen[entry.id]=true
      local c=assert(ids[entry.id],'chunk absent from maze metadata')
      local rect=entry.rect
      if rect.right-rect.left~=130 or rect.top-rect.bottom~=104 then errors[#errors+1]='invalid chunk size' end
      c.x=rect.left/130;c.y=rect.bottom/104
      c.level=data(base..'chunks/'..entry.id..'/level.lua');c.exits=c.level.exits;if work then work('check child')end
    end
    for id in pairs(ids) do if not seen[id] then errors[#errors+1]='missing chunk folder reference '..id end end
    if #root.chunks~=#m.cells then errors[#errors+1]='missing chunk folder reference' end
    local mission=data(base..'mission.lua')
    for _,which in ipairs({'start','goal'}) do
      local c=ids[m[which]];local p=mission[which]
      if not c or not p or p.x<c.x*130 or p.x>=c.x*130+130 or p.y<c.y*104 or p.y>=c.y*104+104 then
        errors[#errors+1]='mission '..which..' outside declared graph chunk'
      end
    end
    local result=m.world and D.maze_world_check.check(m,work)or C.check(m,work)
    for _,e in ipairs(errors) do result.errors[#result.errors+1]=e end
    result.ok=#result.errors==0;return result
  end
  return C
end
