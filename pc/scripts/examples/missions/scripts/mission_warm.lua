-- Material readiness is part of installation, never an in-play surprise.
return function(D)
  local W={DRAW_FRAMES=24}
  function W.cleanup(g,t)
    local w=t.warming;if not w then return end
    if w.handle and g.warm_release then g.warm_release(w.handle);w.handle=nil end
    if w.enemy then g.enemy_remove(w.enemy);w.enemy=nil end
    if w.model then g.model_despawn(w.model);w.model=nil end
    if w.item then g.item_remove(w.item);w.item=nil end
  end
  function W.tick(g,t)
    if not t.warming then
      local w={started=g.time(),enemies={},items={},models={},index=1,draws=0};t.warming=w
      local seen={};local function enemy(k)if not seen[k]then seen[k]=true;w.enemies[#w.enemies+1]=k end end
      for _,e in ipairs(t.doc.mission.enemies)do enemy(e.kind)end
      for _,c in ipairs(t.doc.maze and t.doc.maze.cells or{})do for _,k in ipairs(c.enemy_kinds or{})do enemy(k)end end
      local items={};local function item(k)if not items[k]then items[k]=true;w.items[#w.items+1]=k end end
      for _,k in ipairs(t.doc.level.warm_items or{})do item(k)end
      for _,c in ipairs(t.doc.chunks or{})do for _,k in ipairs(c.level.warm_items or{})do item(k)end end
      if g.item_kinds then for _,k in ipairs(g.item_kinds())do if k.name=='drive'then item('drive')end end end
      for _,path in ipairs(t.asset_paths or{})do w.models[#w.models+1]=t.assets[path]end
      w.native=type(g.warm)=='function'and type(g.warm_done)=='function'and type(g.warm_release)=='function'
      w.jobs={};local batch={enemies=w.enemies,items={}};local count=#w.enemies
      for _,k in ipairs(w.items)do
        if count==120 then w.jobs[#w.jobs+1]=batch;batch={items={}};count=0 end
        batch.items[#batch.items+1]=k;count=count+1
      end
      if count>0 then w.jobs[#w.jobs+1]=batch end;local job={models={}}
      for _,h in ipairs(w.models)do job.models[#job.models+1]=h;if #job.models==120 then w.jobs[#w.jobs+1]=job;job={models={}}end end
      if #job.models>0 then w.jobs[#w.jobs+1]=job end
      w.queue={};for _,k in ipairs(w.enemies)do w.queue[#w.queue+1]={kind='enemy',value=k}end
      for _,k in ipairs(w.items)do w.queue[#w.queue+1]={kind='item',value=k}end
      for _,h in ipairs(w.models)do w.queue[#w.queue+1]={kind='model',value=h}end
    end
    local w=t.warming
    if w.native then
      if not w.handle and w.jobs[w.index]then w.handle=assert(g.warm(w.jobs[w.index]));return false end
      if w.handle then
        local done,why=g.warm_done(w.handle);assert(not why,why)
        if not done then return false end
        g.warm_release(w.handle);w.handle=nil;w.index=w.index+1;return false
      end
    else
      assert(t.covered,'fallback warming requires a drawn opaque loading cover')
      local q=w.queue[w.index]
      if q then
        if not w.active then
          local p=t.player or t.target
          if q.kind=='enemy'then w.enemy=assert(g.spawn_enemy(q.value,p.x+12,p.y+4,{facing=-1}))
          elseif q.kind=='item'then assert(g.item_spawn and g.item_remove,'item fallback warm APIs unavailable');w.item=assert(g.item_spawn(q.value,p.x+12,p.y+4,{}))
          else w.model=assert(g.model_spawn(q.value,{x=p.x,y=p.y,z=0,rot=0,scale=1,collision=false,visible=true}))end
          w.active=true;w.draws=0;return false
        end
        if w.draws<W.DRAW_FRAMES then return false end
        W.cleanup(g,t);w.active=nil;w.index=w.index+1;return false
      end
    end
    if not w.done then w.done=true;g.log(('mission: warm complete mode=%s seconds=%.6f'):format(w.native and 'native'or 'covered-draw',g.time()-w.started))end
    return true
  end
  function W.draw(g,t)
    local a=g.safe_area and g.safe_area()or{w=640,h=480}
    assert(g.fill,'staging requires an opaque loading cover')
    g.fill(0,0,a.w,a.h,0x10151FFF);t.covered=true
    if g.text then g.text(24,32,'Preparing mission...')end
    if t.warming and t.warming.active then t.warming.draws=t.warming.draws+1 end
  end
  return W
end
