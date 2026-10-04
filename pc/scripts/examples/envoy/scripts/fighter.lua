-- Own native modifiers and the match-start boss reserve. No actor is spawned.
return function(D)
  local F={};F.__index=F
  function F.new(g)
    return setmetatable({g=g,owned=false,modified=false,ready=false,boss=false,ticks=0,modes={}},F)
  end
  function F:modes_tick()
    for port=2,6 do
      local p=self.g.player(port)
      if p and p.cpu then
        local mode=port==2 and self.boss and 'fight' or 'stand'
        local key=mode..':'..tostring(p.falls)..':'..tostring(p.kind)..':'..tostring(p.char)
        if self.modes[port]~=key then
          assert(self.g.cpu_mode(port,mode),'CPU mode refused');self.modes[port]=key
        end
      end
    end
  end
  function F:apply(c)
    local e=D.companion.effects(c)
    local key=table.concat({e.damage_dealt,e.damage_taken,e.speed,e.air_speed,e.shield_max},':')
    if key==self.key then return end
    assert(self.g.fighter_mod(1,{damage_dealt=e.damage_dealt,damage_taken=e.damage_taken,
      run_speed=e.speed,air_speed=e.air_speed,shield_max=e.shield_max}),'fighter modifier refused')
    self.modified=true;self.key=key
    self.g.log(('envoy: modifiers applied power=%.3f speed=%.3f guard=%.3f shield=%.3f'):format(e.damage_dealt,e.speed,e.damage_taken,e.shield_max))
  end
  function F:reserve()
    local benched,entities=self.g.fighter_benched(2)
    if benched and not self.owned then return false,'boss already reserved by another owner' end
    if benched then self.ready=true;return true end
    for _,e in ipairs(entities or {}) do
      if e.present and (e.refusal_code or 0)~=0 then return true,'waiting' end
    end
    local ok,why=self.g.fighter_bench(2)
    if not ok then return false,why end
    self.owned=true;self.ready=true;self.g.log('envoy: boss benched');return true
  end
  function F:begin(c)
    if self.started then self:clear();if self.started then return false,'native cleanup pending' end end
    for _,name in ipairs({'fighter_mod','fighter_bench','fighter_benched','fighter_call','cpu_mode'}) do
      if type(self.g[name])~='function' then return false,'engine API required: '..name end
    end
    if self.g.fighter_benched(2) then return false,'boss already reserved' end
    self.started=true;self.boss=false;self.ready=false;self.ticks=0
    local ok,why=pcall(function() self:modes_tick();self:apply(c) end)
    if not ok then self:clear();return false,why end
    local ready,reason=self:reserve()
    if not ready then self:clear();return false,reason end
    local reset,why=self:reset_damage()
    if not reset then self:clear();return false,why end
    return true,reason
  end
  function F:reset_damage()
    local set=self.g.set_damage or self.g.set_percent
    if type(set)~='function' then return false,'engine damage setter required' end
    for port=1,2 do
      local ok,accepted,why=pcall(set,port,0)
      -- Native setters return no value on success.
      if not ok or accepted==false or (accepted==nil and why~=nil) then return false,'damage reset refused P'..port..': '..tostring(why or accepted) end
    end
    self.g.log('envoy: damage reset P1=0 P2=0');return true
  end
  function F:tick(c)
    if not self.started then return true end
    local ok,why=pcall(function() self:modes_tick();self:apply(c) end)
    if not ok then return false,why end
    if self.ready or self.boss then return true end
    self.ticks=self.ticks+1
    if self.ticks>=600 then return false,'boss bench readiness timeout' end
    if self.ticks%6==0 then return self:reserve() end
    return true,'waiting'
  end
  function F:activate(x,y)
    if not self.ready or not self.owned then return false,'boss reserve not ready' end
    local reset,reason=self:reset_damage();if not reset then return false,reason end
    local ok,why=self.g.fighter_call(2,x,y,{facing=-1,intangible_frames=30})
    if not ok then return false,why end
    self.owned=false;self.boss=true;self.modes[2]=nil;self:modes_tick()
    self.g.log('envoy: boss called x='..x..' y='..y);return true
  end
  function F:clear()
    if not self.started then return end
    if self.modified then
      local ok,why=pcall(self.g.fighter_mod,1,nil)
      if ok and why then self.modified=false;self.key=nil;self.g.log('envoy: modifiers cleared') end
    end
    if self.owned then
      local match=self.g.match()
      if not match or not match.active or not self.g.fighter_benched(2) then self.owned=false
      else
        local x,y=self.g.stage_spawn(1)
        local ok,called=pcall(self.g.fighter_call,2,x or 0,y or 30,{facing=-1})
        if ok and called then self.owned=false end
      end
    end
    self.boss=false;self.modes={};pcall(self.modes_tick,self)
    self.ready=false
    -- Retain cleanup ownership when a native refusal needs a later retry.
    if not self.modified and not self.owned then self.started=false end
  end
  return F
end
