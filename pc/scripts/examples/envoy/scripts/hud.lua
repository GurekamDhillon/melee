-- Overlay geometry follows the live canvas, including widescreen and offsets.
return function(D)
  local C=D.companion
  local H={colours={red=0xF07474FF,green=0x77CD9CFF,blue=0x79AAF0FF,yellow=0xEBD175FF,white=0xFFFFFFFF}}
  local names=C.tuning.stat_names or {power='Power',speed='Speed',guard='Guard',jump='Jump'}
  function H.draw(g,c,status,flashes,pickup_flash)
    if not c then return end
    local a=g.safe_area();local w=math.min(280,a.w-24);local h=155
    local x=a.right-w-12;local y=a.y+12
    local kit=g.kit and g.kit.available()
    local function label(xx,yy,text,colour)
      if kit then g.kit.text(xx,yy+10,text,'body',colour,'left',{max_w=w-20})
      else
        local limit=math.max(1,math.floor((w-20)/8))
        if #text>limit then text=text:sub(1,math.max(1,limit-3))..'...' end
        g.text(xx,yy,text,colour,10)
      end
    end
    g.fill(x,y,w,h,0x18232BE8)
    label(x+10,y+8,'SuperTime Envoy | '..(status or 'running'),0xEBD175FF)
    local colour={'red','green','blue','yellow'}
    for i,k in ipairs(C.stats) do
      local s=(c.stats or {})[k] or {};local yy=y+30+(i-1)*27;local bar=w-20
      local fraction,current,needed=0,0,0
      if C.progress and s.points and s.level then fraction,current,needed=C.progress(c,k) end
      fraction=math.max(0,math.min(1,fraction or 0))
      local progress=needed==0 and 'MAX' or ('%d/%d'):format(current,needed)
      local moment=flashes and flashes[k] and ' LEVEL UP!' or ''
      label(x+10,yy,('%s L%d %s %s%s'):format(names[k] or k,s.level or 0,s.grade or '?',progress,moment),0xF3F0E8FF)
      g.fill(x+10,yy+17,bar,5,0x44525CFF)
      g.fill(x+10,yy+17,bar*fraction,5,H.colours[colour[i]])
      if pickup_flash and pickup_flash[colour[i]] then g.fill(x+10,yy+16,bar,7,0xFFFFFF80) end
    end
    if (c.white_drives or 0)>0 then label(x+10,y+h-12,'White drives found: '..c.white_drives,pickup_flash and pickup_flash.white and 0xEBD175FF or 0xFFFFFFFF) end
  end
  function H.pickups(g,pickups)
    for _,p in ipairs(pickups) do
      local x,y,visible=g.project(p.x,p.y,0)
      if x and visible then g.text(x-5,y-8,'<>',H.colours[p.colour],14) end
    end
  end
  return H
end
