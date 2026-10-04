-- Conservative collision certificates, including neighbouring chunks/seams.
return function()
  local C={HEADROOM=50,CRAWL=20,WALL_GAP=6,STEP=20,LANDING=32}
  local function horizontal(l)return l.y1==l.y2 end
  local function span(l)return math.min(l.x1,l.x2),math.max(l.x1,l.x2)end
  function C.check(m,work)
    local errors,grid={},{}
    local function name(c,l,i)return c.id..'/'..(l.label or ('line'..i))end
    for _,c in ipairs(m.cells or {})do grid[c.x..','..c.y]=c end
    for _,c in ipairs(m.cells or {})do
      for i,p in ipairs(c.level and c.level.parts or {})do if p.collision then errors[#errors+1]=c.id..'/'..(p.name or 'part'..i)..': opaque model collision needs explicit render-only parts and certified lines'end end
      for i,l in ipairs(c.level and c.level.lines or {})do
        if (l.kind=='floor'or l.kind=='ceiling')and not horizontal(l)then errors[#errors+1]=name(c,l,i)..': sloped surface lacks a clearance certificate'end
        if (l.kind=='left_wall'or l.kind=='right_wall')and l.x1~=l.x2 then errors[#errors+1]=name(c,l,i)..': diagonal wall lacks a clearance certificate'end
      end
      for _,e in ipairs(c.exits or{})do if e.side=='up'and e.to and e.traversal=='climb'then
        local previous=0
        for i,p in ipairs(c.platforms or{})do
          if p.y-previous>C.STEP then errors[#errors+1]=c.id..'/climb'..i..': step '..(p.y-previous)..' > '..C.STEP end
          previous=p.y
        end
      end end
      for i,l in ipairs(c.level and c.level.lines or {})do if l.kind=='floor'and horizontal(l)then
        local crawl=l.label and l.label:match('^crawl:')
        for _,tag in ipairs(c.tags or{})do if tag=='crawl'then crawl=true end end
        local a,z=span(l);local required=crawl and C.CRAWL or C.HEADROOM
        for dx=-1,1 do for dy=-1,1 do local other=grid[(c.x+dx)..','..(c.y+dy)]
          for j,o in ipairs(other and other.level and other.level.lines or {})do
            if o.kind=='ceiling'and horizontal(o)and o.y1>l.y1 and o.y1-l.y1<required then
              local oa,oz=span(o)
              if oa<z+5.55 and oz>a-5.55 then errors[#errors+1]=name(c,l,i)..': headroom '..(o.y1-l.y1)..' < '..required..' under '..name(other,o,j)end
            end
            if l.y1>c.y*104 and l.y1<c.y*104+104 and (o.kind=='left_wall'or o.kind=='right_wall')and o.x1==o.x2 and
              math.max(o.y1,o.y2)>l.y1 and math.min(o.y1,o.y2)<l.y1+required then
              local gap=math.max(a-o.x1,o.x1-z,0)
              if gap<C.WALL_GAP then errors[#errors+1]=name(c,l,i)..': wall gap '..gap..' < '..C.WALL_GAP..' at '..name(other,o,j)end
            end
          end
        end end
      end end
      -- A drop must encounter a broad supporting span, or a contiguous sealed floor.
      for _,e in ipairs(c.exits or {})do if e.side=='down'and e.to then
        local below=grid[c.x..','..(c.y-1)];local found=false
        for _,l in ipairs(below and below.level.lines or {})do if l.kind=='floor'and horizontal(l)then
          local a,z=span(l);if a<=c.x*130+65 and z>=c.x*130+65 and z-a>=C.LANDING then found=true end
        end end
        if not found and below then
          local spans={};for _,l in ipairs(below.level.lines)do if l.kind=='floor'and l.y1==below.y*104 and horizontal(l)then local a,z=span(l);spans[#spans+1]={a,z}end end
          table.sort(spans,function(a,z)return a[1]<z[1]end);local lo,hi
          for _,p in ipairs(spans)do if not hi or p[1]>hi then lo,hi=p[1],p[2]else hi=math.max(hi,p[2])end
            if lo<=c.x*130+65 and hi>=c.x*130+65 and hi-lo>=C.LANDING then found=true end
          end
        end
        if not found then errors[#errors+1]=c.id..'/down: no '..C.LANDING..'-unit landing in '..tostring(e.to)end
      end end
      if work then work('clearance')end
    end
    return {ok=#errors==0,errors=errors}
  end
  return C
end
