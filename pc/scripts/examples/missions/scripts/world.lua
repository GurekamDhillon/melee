-- Area builders own geometry; references are released only after their instances disappear.
return function(D)
  local W={}
  function W.options(p)
    local o={}
    for _,k in ipairs({'x','y','z','rot','scale','scale_x','scale_y','scale_z','collision','floor_flags'}) do o[k]=p[k] end
    return o
  end
  function W.load(g,name,level,staging)
    local area={name=name,assets={},parts={},level=level,replacements={},serial=0,lines={},borrowed=staging and staging.assets,offset=staging and staging.offset}
    local ok,why=pcall(function()
      for _,p in ipairs(level.parts) do
        if not area.assets[p.path] then area.assets[p.path]=staging and assert(staging.assets[p.path]) or assert(g.model_load(p.path)) end
      end
      local prepared=g.area_prepare and g.area_activate
      local build=prepared and g.area_prepare or g.area_load
      local token=build(name,function()
        for i,p in ipairs(level.parts) do
          local options=W.options(p)
          if staging then options.y=options.y+staging.offset;options.visible=false end
          area.parts[i]={handle=assert(g.model_spawn(area.assets[p.path],options)),part=p,position_offset=staging and staging.offset or 0}
          if g.model_label then g.model_label(area.parts[i].handle,p.label or p.part) end
        end
        for _,l in ipairs(level.lines) do
          local offset=staging and staging.offset or 0
          area.lines[#area.lines+1]={handle=assert(g.stage_add_line(l.x1,l.y1+offset,l.x2,l.y2+offset,l.kind,l.opts)),line=l}
        end
      end)
      assert(token,'area construction refused');area.constructed=true
      if prepared then area.prepared=token;assert(g.area_activate(token),'area activation refused')end
    end)
    if not ok then
      if area.constructed then pcall(g.area_unload,name)end
      if not area.borrowed then for _,a in pairs(area.assets) do pcall(g.model_release,a) end end
      error(why,0)
    end
    return area
  end
  function W.unload(g,a)
    for _,replacement in pairs(a.replacements) do W.unload(g,replacement) end
    -- Match-end hooks may run after native scene cleanup has reclaimed these handles.
    local ok,why=pcall(g.area_unload,a.name)
    if not ok and g.match().active then g.log('mission: cleanup refused '..tostring(why)) end
    if not a.borrowed then for _,asset in pairs(a.assets) do pcall(g.model_release,asset) end end
  end
  function W.position(g,area,offset)
    if (area.offset or 0)==offset then return end
    for _,replacement in pairs(area.replacements) do W.position(g,replacement,offset) end
    for _,entry in ipairs(area.parts) do
      local options=W.options(entry.part);options.y=options.y+offset;options.visible=offset==0
      local live=g.model_get and g.model_get(entry.handle);local desired={}
      for k,v in pairs(options)do if k~='collision'and k~='floor_flags'then desired[k]=v end end
      local changed=entry.position_offset~=offset
      if live and D.install_restore then changed=not D.install_restore.same(live,desired)end
      if changed then assert(g.model_set(entry.handle,options),'model activation refused')end
      entry.position_offset=offset
    end
    for _,entry in ipairs(area.lines) do
      local l=entry.line
      if (entry.position_offset or area.offset or 0)~=offset then
        assert(g.stage_move(entry.handle,(l.x1+l.x2)/2,(l.y1+l.y2)/2+offset),'line activation refused');entry.position_offset=offset
      end
    end
    area.offset=offset
  end
  function W.bounds(g,level)
    for _,k in ipairs({'camera','blast'}) do
      local b=level[k]
      if b then
        if D.install_restore then D.install_restore.bounds(g,k,b)else assert(g['stage_set_'..k..'_bounds'](b.left,b.right,b.top,b.bottom))end
      end
    end
    for slot,p in pairs(level.spawn)do
      if D.install_restore then D.install_restore.spawn(g,slot,p)else assert(g.stage_set_spawn(slot,p.x,p.y))end
    end
  end
  -- Trigger edits can touch hundreds of instances. Visit at most one match per tick.
  function W.collision_tick(g,current,job)
    job.area=job.area or 1;job.part=job.part or 1
    while job.areas[job.area]do
      local area=job.areas[job.area];local live=area==current.root
      for _,v in pairs(current.stream.loaded)do if v==area then live=true;break end end
      if live then
        while area.parts[job.part]do
          local entry=area.parts[job.part];job.part=job.part+1;local p=entry.part;local a=job.action
          if (p.x-a.x)^2+(p.y-a.y)^2<=a.r^2 then
            -- A one-entry view retains the area's ownership and replacement table.
            local view={name=area.name,parts={entry},serial=area.serial,replacements=area.replacements}
            W.collision(g,{view},a);area.serial=view.serial;return false,true
          end
        end
      end
      job.area=job.area+1;job.part=1
    end
    return true,false
  end
  function W.collision(g,areas,a)
    -- Collision is spawn-only. Replacement areas belong to the source area and
    -- are unloaded with it, including on streaming, reload and script unload.
    local changed={}
    local ok,why=pcall(function()
      for _,area in ipairs(areas) do
      for i,entry in ipairs(area.parts) do
        local p=entry.part
        if (p.x-a.x)^2+(p.y-a.y)^2<=a.r^2 then
          local q=W.options(p);q.part=p.part;q.path=p.path;q.label=p.label;q.collision=not a.open
          area.serial=area.serial+1
          local replacement=W.load(g,area.name..'_r'..area.serial..'_'..i,{parts={q},lines={}})
          changed[#changed+1]={area=area,replacement=replacement,entry=entry,new=replacement.parts[1].handle,old=entry.handle}
        end
      end
      end
    end)
    if not ok then for _,c in ipairs(changed) do W.unload(g,c.replacement) end error(why,0) end
    for _,c in ipairs(changed) do
      g.model_despawn(c.old)
      local previous=c.area.replacements[c.entry]
      if previous then W.unload(g,previous) end
      c.entry.handle=c.new;c.area.replacements[c.entry]=c.replacement
    end
  end
  return W
end
