-- Observe the entry's private pure machine. Shared mission source stays verbatim.
-- No log parsing or second enemy-status classifier: reward exactly its defeat events.
return function()
  local E={}
  function E.engine(g)
    local adapter=setmetatable({},{__index=g})
    if type(g.set_damage)=='function' then
      adapter.set_damage=function(...)
        local accepted,why=g.set_damage(...)
        -- Current native setter is void; runtime fix5 expects a true result.
        if accepted==nil and why==nil then return true end
        return accepted,why
      end
    end
    return adapter
  end
  function E.observe(machine,sink,log)
    local original=assert(machine.step)
    local function step(...)
      local actions,events=original(...)
      for _,event in ipairs(events) do
        local ok,why=pcall(sink,event)
        if not ok and log then log('envoy: mission listener refused '..tostring(why)) end
      end
      return actions,events
    end
    machine.step=step
    return function() if machine.step==step then machine.step=original end end
  end
  return E
end
