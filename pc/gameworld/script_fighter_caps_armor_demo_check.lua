-- Scoped typed armour demo contract smoke, workspace-root invocation.
-- Explicit stub; never executes native collisions or rewind.
local types={'knockback','damage_threshold','knockback_threshold','super','hit_count','damage_pool'}
local count=0
for _,scenario in ipairs({'basic','inactive','missing','refusal'}) do
  local g,S=dofile('tools/port/demo_gd_stub.lua')
  S.armor={}; S.hits={}; S.writes=0
  if scenario=='inactive' then S.active=false end
  if scenario=='missing' then S.players[2]=nil end
  local function available(e) return S.players[e] and S.active~=false and scenario~='refusal' end
  function g.fighter_armor(e,...)
    assert(type(e)=='number' and e>=1 and e<=12 and e%1==0)
    if select('#',...)==0 then
      local rows={}; if not S.players[e] then return nil end
      for _,kind in ipairs(types) do
        local a=S.armor[kind]
        if a then local copy={}; for k,v in pairs(a) do copy[k]=v end; rows[#rows+1]=copy end
      end
      return rows
    end
    S.writes=S.writes+1; if not available(e) then return false end
    local a=(...)
    if a==nil then S.armor={}; return true end
    local found=false; for _,kind in ipairs(types) do if a.type==kind then found=true end end; assert(found)
    if a.clear then S.armor[a.type]=nil; return true end
    assert(type(a.value)=='number' and a.value>=0)
    assert(a.frames==nil or (a.frames>=0 and a.frames<=36000 and a.frames%1==0))
    assert(a.direction==nil or a.direction=='any' or a.direction=='front' or a.direction=='back')
    S.armor[a.type]={type=a.type,value=a.value,remaining=a.value,frames=a.frames or 0,
      state=a.state or -1,from=a.from or -1,to=a.to or -1,direction=a.direction or 'any',enabled=true}
    return true
  end
  function g.hit(target,h)
    assert(target==1 and h.from==2 and h.angle==45 and type(h.kbg)=='number' and type(h.bkb)=='number')
    if not available(target) or not S.players[h.from] then return false end
    local hit={}; for k,v in pairs(h) do hit[k]=v end; S.hits[#S.hits+1]=hit
    return true
  end
  local advance=g.advance
  function g.advance()
    advance()
    for kind,a in pairs(S.armor) do
      if a.frames>0 then a.frames=a.frames-1; if a.frames==0 then S.armor[kind]=nil end end
    end
  end
  local env=setmetatable({gd=g},{__index=_G})
  assert(loadfile('melee/pc/scripts/examples/demos/armor-types/scripts/main.lua','t',env))()
  local function hook(name,...) if env[name] then env[name](...) end end
  local function frame() g.advance(); hook('on_frame'); hook('on_draw') end
  local function key(k) S.keys[k]=true; hook('on_tick'); hook('on_draw') end
  frame()
  if scenario=='inactive' or scenario=='missing' then
    assert(S.writes==0,'unavailable match attempted setup'); hook('on_unload')
  elseif scenario=='refusal' then
    assert(next(S.armor)==nil,'refusal installed armour'); hook('on_unload')
  else
    assert(S.players[2].cpu_mode=='stand','P2 not standing')
    assert(g.fighter_armor(1)[1].type=='knockback')
    local writes=S.writes; frame(); assert(S.writes==writes,'setup repeated')
    for _,kind in ipairs({'damage_threshold','knockback_threshold','super','hit_count','damage_pool','knockback'}) do
      key('A'); local rows=g.fighter_armor(1)
      assert(#rows==1 and rows[1].type==kind and rows[1].frames==600,'cycle failed')
      key('W'); key('M'); key('S')
      local n=#S.hits; assert(S.hits[n-2].damage==3 and S.hits[n-1].damage==10 and S.hits[n].damage==25,'hit strengths wrong')
      -- Inject an observational payload to test HUD/event wiring, not absorption physics.
      hook('on_armor',{port=1,entity=1,sub=false,type=kind,absorbed=true,broke=false,damage=3,knockback=20})
      assert(S.logs[#S.logs]:find(kind..' absorbed=true broke=false damage=3 KB=20',1,true),'event payload lost')
    end
    local n=#S.hits; key('SPACE'); for _=1,270 do frame() end
    assert(#S.hits==n+3,'auto barrage cadence wrong'); key('SPACE')
    key('R'); assert(g.fighter_armor(1)[1].frames==600)
    for _=1,600 do frame() end
    assert(#g.fighter_armor(1)==0,'timed armour did not expire')
    key('R'); hook('on_unload'); assert(#g.fighter_armor(1)==0,'armour survived unload')
  end
  count=count+1
end
print('PASS: '..count..' typed armour demo stub scenarios; native hits, absorption and rewind untested')
