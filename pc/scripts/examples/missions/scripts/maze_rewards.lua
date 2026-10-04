-- Maze-only reward policy; the shared mission state machine remains unchanged.
return function(D)
  local R={}
  function R.tick(g,c)
    local maze=c.doc.maze;if not maze then return end
    local p=c.membership and c.membership.point or g.player(1);if not D.fighters.ready(p) then return end
    c.maze_rewards=c.maze_rewards or {}
    for _,cell in ipairs(maze.cells) do
      if cell.reward and not c.maze_rewards[cell.id] and
        math.abs(p.x-(cell.x*130+20))<=12 and math.abs(p.y-(cell.y*104+12))<=16 then
        local percent=math.max(0,math.floor(p.percent or 0)-(cell.reward_amount or 25))
        -- The native setter returns zero values. Verify synchronous readback;
        -- treating nil as refusal repeatedly reapplied an unconsumed cache.
        if math.abs((p.percent or 0)-percent)>0.01 then g.set_damage(1,percent) end
        local live=g.player(1)
        if live and math.abs((live.percent or 0)-percent)<=0.01 then
          c.maze_rewards[cell.id]=true
          g.log('mission: maze reward '..cell.id..' heal '..(cell.reward_amount or 25))
        end
      end
    end
  end
  return R
end
