-- Non-physical authored membership; P1 cur_pos (x,y), once per logic frame.
return function(D)
  local Z={defaults={commit_distance=8,vertical_commit_distance=12,ease_frames=24,curve='smoothstep',doorway_margin=12}}
  local function inside(b,p,closed)
    return p and p.x>=b.left and (closed and p.x<=b.right or p.x<b.right)
      and p.y>=b.bottom and (closed and p.y<=b.top or p.y<b.top)
  end
  local function overlap(a,b)return a.left<b.right and a.right>b.left and a.bottom<b.top and a.top>b.bottom end
  function Z.tuning(doc,door,room)
    local q={};for k,v in pairs(Z.defaults)do q[k]=v end
    for k,v in pairs(doc.level.camera_settings or {})do q[k]=v end
    for k,v in pairs(room and room.level and room.level.camera_settings or {})do q[k]=v end
    for k,v in pairs(q.doors and door and q.doors[door.name] or {})do q[k]=v end
    for k,v in pairs(door and door.camera or {})do q[k]=v end
    return q
  end
  function Z.prepare(doc,work)
    if doc.zone_data then return doc.zone_data end
    local out={zones={},rooms={},regions={},chunks={}};local names={};local authored_rooms=false;local authored_doors=false
    for _,c in ipairs(doc.chunks)do out.chunks[c.id]=c end
    local function add(z)
      assert(not names[z.name],'duplicate zone '..z.name);names[z.name]=true
      out.zones[#out.zones+1]=z
      if z.kind=='room' then
        assert(out.chunks[z.room],'unknown room '..tostring(z.room)..' in '..z.name)
        for _,old in ipairs(out.rooms)do assert(not overlap(z.rect,old.rect),'overlapping room zones '..old.name..' and '..z.name)end
        out.rooms[#out.rooms+1]=z
      elseif z.kind=='region'then out.regions[#out.regions+1]=z
      else
        for _,id in ipairs(z.rooms)do assert(out.chunks[id],'unknown room '..id..' in '..z.name)end
      end
      if work then work('zone membership')end
    end
    local function authored(level)
      for _,z in ipairs(level.zones or {})do
        if z.kind=='room' then authored_rooms=true elseif z.kind=='transition'then authored_doors=true end;add(z)
      end
    end
    authored(doc.level);for _,c in ipairs(doc.chunks)do authored(c.level or {})end
    if not authored_rooms then for _,c in ipairs(doc.chunks)do add({name='room_'..c.id,kind='room',room=c.id,rect=c.rect})end end
    local cells={};for _,c in ipairs(doc.maze and doc.maze.cells or {})do cells[c.id]=c end
    if not authored_doors then
      for i,a in ipairs(doc.chunks)do for j=i+1,#doc.chunks do
        local b=doc.chunks[j];local ar,br=a.rect,b.rect;local axis,border,lo,hi,side
        if ar.right==br.left or br.right==ar.left then
          axis='x';border=ar.right==br.left and ar.right or ar.left;side=ar.right==br.left and 'right' or 'left'
          lo=math.max(ar.bottom,br.bottom);hi=math.min(ar.top,br.top)
        elseif ar.top==br.bottom or br.top==ar.bottom then
          axis='y';border=ar.top==br.bottom and ar.top or ar.bottom;side=ar.top==br.bottom and 'up' or 'down'
          lo=math.max(ar.left,br.left);hi=math.min(ar.right,br.right)
        end
        local cell=cells[a.id];local open=not cell
        if cell then for _,e in ipairs(cell.exits or {})do if e.side==side and e.to==b.id and not e.sealed then open=true end end end
        if axis and lo<hi and open then
          local z={name='door_'..a.id..'_'..b.id,kind='transition',rooms={a.id,b.id},axis=axis}
          local q=Z.tuning(doc,z,a);local distance=axis=='x' and q.commit_distance or q.vertical_commit_distance
          if cell then
            if axis=='x' then lo,hi=ar.bottom,ar.bottom+32 else lo,hi=ar.left+53,ar.left+77 end
          end
          z.rect=axis=='x' and {left=border-distance,right=border+distance,bottom=lo,top=hi}
            or {bottom=border-distance,top=border+distance,left=lo,right=hi}
          add(z)
        end
      end end
    end
    doc.zone_data=out;return out
  end
  -- Only this adapter knows about native membership. Until an owner supplies a
  -- name mapping, Lua volumes are authoritative; raw native handle lists are not guessed.
  function Z.source(g,data,p)
    if g.zones_at and Z.native_source then
      local result=Z.native_source(g.zones_at(1),data,p)
      if result then return result end
    end
    local hits={};for _,z in ipairs(data.zones)do if inside(z.rect,p,z.kind=='transition')then hits[#hits+1]=z end end
    return hits
  end
  function Z.initial(doc,p,previous)
    local data=Z.prepare(doc)
    if previous and data.chunks[previous.id]then return data.chunks[previous.id]end
    local door,room
    for _,z in ipairs(data.zones)do if inside(z.rect,p,z.kind=='transition')then
      if z.kind=='transition' then door=door or z elseif z.kind=='room'then room=data.chunks[z.room]end
    end end
    if not door then return room end
    -- A fresh marker inside a doorway has no crossing history: prefer the
    -- authored start room if joined, otherwise the door's first declared room.
    for _,id in ipairs(door.rooms)do
      for _,z in ipairs(data.rooms)do if z.room==id and inside(z.rect,doc.mission.start)then return data.chunks[id]end end
    end
    return data.chunks[door.rooms[1]]
  end
  function Z.sample(g,c,p,frame)
    if c.membership and c.membership.frame==frame then return c.membership end
    local initializing=not c.zone_state
    local data=Z.prepare(c.doc);local st=c.zone_state or {committed=c.stream.current,transitions=0,lock=0};c.zone_state=st
    local prior={committed=st.committed,lock=st.lock,transitions=st.transitions,door=st.door}
    local hits=Z.source(g,data,p)
    if not p or (p.action and p.action<=13)then hits={}end
    local rooms,doors={},{}
    for _,z in ipairs(hits)do if z.kind=='room' then rooms[#rooms+1]=z elseif z.kind=='transition'then doors[#doors+1]=z end end
    table.sort(doors,function(a,b)return a.name<b.name end)
    local door=doors[1];local room=#rooms==1 and data.chunks[rooms[1].room] or nil
    local outside=#rooms==0 and #doors==0 and #data.rooms>0
    if outside and not st.outside then g.log('mission: zones outside left='..(st.last_names or 'none')..'; committed='..(st.committed and st.committed.id or 'none'))end
    if not outside and st.outside then
      local names={};for _,z in ipairs(hits)do names[#names+1]=z.name end
      g.log('mission: zones re-enter '..table.concat(names,','))
    end
    if not outside then local names={};for _,z in ipairs(hits)do names[#names+1]=z.name end;st.last_names=table.concat(names,',')end
    st.outside=outside
    if door then st.door=door end
    local tuning=Z.tuning(c.doc,door or st.door,room or st.committed);local changed=false
    if room and not door and room~=st.committed and frame>=st.lock and (not initializing or not st.committed) then
      local old=st.committed
      -- Upward airborne excursions do not own an upper room until landing.
      local upward=old and room.rect.bottom>=old.rect.top
      if not (upward and p and p.airborne==true)then
        st.committed=room;changed=true
        if old then st.transitions=st.transitions+1;st.lock=frame+(tuning.mode=='chunk' and tuning.ease_frames or 0) end
        st.door=nil
        g.log('mission: zones commit '..room.id)
      end
    end
    local result={frame=frame,point=p,hits=hits,room=room,transition=door,outside=outside,
      prior=prior,committed=st.committed,changed=changed,transitions=st.transitions,tuning=tuning,lock=st.lock}
    result.region=Z.region(c.doc,result.committed)
    c.membership=result;c.stream.membership=result;c.stream.zone_state=st;return result
  end
  function Z.region(doc,room)
    if doc.maze and doc.maze.world and room then
      for _,cell in ipairs(doc.maze.cells)do if cell.id==room.id then return cell.region or cell.connector end end
    end
  end
  function Z.reject(stream,result)
    if not result or not result.changed then return end
    local st=stream.zone_state
    for k,v in pairs(result.prior)do st[k]=v end
    st.committed=result.prior.committed;st.door=result.prior.door
    result.committed=st.committed;result.changed=false;result.transitions=st.transitions;result.region=Z.region(stream.doc,result.committed)
  end
  function Z.describe(g,c)
    local data=Z.prepare(c.doc)
    for _,z in ipairs(data.zones)do local b=z.rect
      g.log(('mission: zone %s %s rooms=%s rect=%.1f,%.1f,%.1f,%.1f'):format(z.name,z.kind,
        z.room or z.region or table.concat(z.rooms,','),b.left,b.right,b.bottom,b.top))
    end
    local m=c.membership
    g.log('mission: zones membership room='..(m and m.room and m.room.id or 'none')..
      ' transition='..(m and m.transition and m.transition.name or 'none')..
      ' committed='..(m and m.committed and m.committed.id or 'none')..' outside='..tostring(m and m.outside or false))
  end
  return Z
end
