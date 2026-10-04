-- The existing mission result owns an offline finish panel and one pause.
return function(D)
  local F={}
  function F.tick(r)
    local c=r.current;if not c or not c.run.state.result then return end
    if r.staging or r.pending or r.world_generation or r.stopping then return end
    if c.run.finish then
      if c.run.finish_restore then
        if r.g.pause and not(r.g.paused and r.g.paused())then r.g.pause();c.run.finish_pause=true end
        c.run.finish_restore=nil
      end
      return
    end
    if c.run.state.result.status~='complete'then return end
    D.commands.cancel(r);c.tour=nil
    c.run.finish=true
    if r.g.pause and not(r.g.paused and r.g.paused())then r.g.pause();c.run.finish_pause=true end
    r.g.log('mission: finish - restart, reroll or stop')
  end
  function F.release(r,restore)
    local c=r.current
    if c and c.run.finish_pause then c.run.finish_restore=restore or nil;if r.g.resume then r.g.resume()end;c.run.finish_pause=nil end
  end
  function F.draw(g,c)
    if not c.run.finish then return end
    local area=g.safe_area and g.safe_area()or{w=640,h=480};local x=math.max(16,(area.w-480)/2)
    if g.fill then g.fill(x,170,480,130,0x10151FEE)end
    g.text(x+24,192,'MISSION COMPLETE',0xEBD175FF)
    g.text(x+24,224,D.mission.result_text(c.run.state))
    g.text(x+24,258,c.doc.maze and c.doc.maze.world and 'mission restart / world reroll / stop'or 'mission restart / maze reroll / stop')
  end
  return F
end
