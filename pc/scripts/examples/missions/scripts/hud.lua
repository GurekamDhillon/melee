return function(D)
  local H={}
  function H.draw(g,current)
    if not current then return end
    local run=current.run
    local text=D.mission.result_text(run.state) or D.mission.hud(run.state)
    local m=current.membership
    if m and m.region then text=text..' | region '..m.region end
    if current.doc.maze and current.doc.maze.world then
      local links={};for _,e in ipairs(current.doc.maze.world.links)do links[#links+1]=e.a..'-'..e.b end
      g.text(16,64,'World: '..table.concat(links,' / '))
    end
    if m then text=text..' | zones room='..(m.room and m.room.id or 'none')..' transition='..(m.transition and m.transition.name or 'none')..' committed='..(m.committed and m.committed.id or 'none')end
    g.text(16,24,current.doc.name..' | '..text)
    if D.mission_finish then D.mission_finish.draw(g,current)end
    if run.message then g.text(16,44,run.message.text) end
  end
  return H
end
