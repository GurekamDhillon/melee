-- Engine-held launch uses render ticks; ordinary missions use logic frames.
return function(D)
  local L={}
  function L.begin(r,q)
    if r.launch then return end
    r.last_install_error=nil;r.pending_error=nil
    r.launch={frames=0,engine=q.pending==true}
    local accepted,why=pcall(function()
      local command
      if type(q.mission)=='string'and q.mission~=''then D.validator.name(q.mission);command='play '..q.mission
      elseif q.size and q.size>0 then command=('maze %d %d'):format(q.seed,q.size)
      else error('launch needs a mission or maze')end
      r.launch.command=command
      local ok,reason=r:command(command);assert(ok,reason)
    end)
    if not accepted then L.cancel(r,why)end
  end
  function L.cancel(r,why)
    local launch=r.launch;r.launch=nil
    if launch and launch.engine and r.g.launch_cancel then r.g.launch_cancel(tostring(why))end
    r.g.log('mission: launch refused '..tostring(why))
  end
  function L.tick(r)
    local q=r.launch;if not q then return end
    q.frames=q.frames+1
    if r.last_install_error or r.recovery then L.cancel(r,r.last_install_error or 'staging recovery required');return end
    if q.engine then
      if r.staging then D.install.tick(r)
      elseif r.pending then r:retry()end
      if D.maze_commands then D.maze_commands.settle(r)end
    end
    if r.pending_error or r.last_install_error or r.recovery then L.cancel(r,r.pending_error or r.last_install_error or 'staging recovery required');return end
    if r.current and not r.staging and not r.pending then
      if not q.engine or not r.g.launch_ready or r.g.launch_ready()then r.launch=nil;return end
    end
    if q.frames>=7200 then L.cancel(r,'mission launch timed out after 7200 ticks')end
  end
  function L.autostart(r)
    if r.launch then return end
    local q=r.g.launch_request and r.g.launch_request()
    if q and q.pending then return end -- engine delivers the selected on_launch owner
    local data=D.loader.data(r.g,'missions/autostart.lua',true)
    if data then L.begin(r,data);return end
    -- A level may opt in. Refuse ambiguity, never choose an arbitrary folder.
    local selected;local entries=r.g.mod_list('missions/')or{}
    if #entries>64 then r.g.log('mission: autostart scan exceeds 64 folders; use missions/autostart.lua');return end
    for _,e in ipairs(entries)do if e.dir and type(e.name)=='string'and e.name:match('^[%w_-]+$')then
      local level=D.loader.data(r.g,'missions/'..e.name..'/level.lua',true)
      if level and level.autostart==true then
        if selected then r.g.log('mission: ambiguous level autostart; use missions/autostart.lua');return end
        selected=e.name
      end
    end end
    if selected then L.begin(r,{mission=selected})end
  end
  return L
end
