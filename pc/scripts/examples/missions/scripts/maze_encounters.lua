-- Spatial encounter adapter; the pure mission machine remains unchanged.
return function(D)
  local E={}
  function E.plan(m,work)
    local groups,owner={},{}
    for i,c in ipairs(m.cells)do groups[i]={cells={c},count=c.enemy_budget};owner[c.id]=i end
    local count=#m.cells
    while count>8 do
      local bestA,bestB,best
      for _,c in ipairs(m.cells)do for _,e in ipairs(c.exits)do if e.to then
        local a,b=owner[c.id],owner[e.to]
        if a~=b then local size=groups[a].count+groups[b].count
          local score=#groups[a].cells+#groups[b].cells+size*10
          if size<=32 and (not best or score<best)then bestA,bestB,best=a,b,score end
        end
      end end end
      assert(bestA,'cannot partition connected spatial waves')
      local a,b=groups[bestA],groups[bestB]
      for _,c in ipairs(b.cells)do a.cells[#a.cells+1]=c;owner[c.id]=bestA end
      a.count=a.count+b.count;groups[bestB]=nil;count=count-1
      if work then work('partition')end
    end
    local map,n={},0
    for i=1,#m.cells do local group=groups[i]
      if group and group.count>0 then n=n+1;map[i]=n end
    end
    for id,w in pairs(owner)do owner[id]=map[w]end
    return owner,n
  end
  function E.observe(c)
    if not c.doc.maze then return end
    local z=c.membership;local room=z and not z.transition and z.room==z.committed and z.committed
    if room and not c.stream.loaded[room.id]then room=nil end
    if not c.encounter_cells then c.encounter_cells={}
      for _,cell in ipairs(c.doc.maze.cells)do c.encounter_cells[cell.id]=cell end
    end
    local cell=room and c.encounter_cells[room.id]
    local wave=cell and cell.enemy_budget>0 and cell.enemy_wave
    for _,t in ipairs(c.run.state.m.triggers)do if t.action=='wave'then
      t.x,t.y=99999,99999
      if t.wave==wave then local r=room.rect;t.x=(r.left+r.right)/2;t.y=(r.bottom+r.top)/2;t.w=r.right-r.left;t.h=r.top-r.bottom end
    end end
    c.run.encounter_room=room
  end
  function E.actions(c,actions)
    if not c.doc.maze then return actions end
    local run=c.run;run.encounter_pending=run.encounter_pending or {};local out={}
    for _,a in ipairs(actions)do if a.type=='spawn'then run.encounter_pending[a.index]=a else out[#out+1]=a end end
    if run.state.result then run.encounter_pending={};return out end
    local room=run.encounter_room
    local live=0;for _ in pairs(run.state.tracked)do live=live+1 end
    if room then local r=room.rect
      for i=1,#run.state.m.enemies do local a=run.encounter_pending[i]
        if live<D.mission.LIMITS.per_wave and a and a.x>=r.left and a.x<r.right and a.y>=r.bottom and a.y<r.top then out[#out+1]=a;run.encounter_pending[i]=nil;live=live+1 end
      end
    end
    return out
  end
  return E
end
