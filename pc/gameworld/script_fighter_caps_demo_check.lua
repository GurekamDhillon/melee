-- Standalone demo contract smoke; run from workspace root with lua.
-- Uses explicit Lua stubs only. Does not run native code, physics or rewind proof.
local base='melee/pc/scripts/examples/demos/fighter-'
local flags={'shield','air_dodge','run','grab','specials'}
local kinds={'intangible','invincible','metal','size'}
local function copy(t) if not t then return nil end; local r={}; for k,v in pairs(t) do r[k]=v end; return r end
local function fixture()
  local g,S=dofile('tools/port/demo_gd_stub.lua')
  S.caps={}; S.effects={}; S.armour={}; S.statuses={}; S.held={}; S.writes=0
  S.players[2].x=20; for id=3,6 do S.players[id].x=200+id end
  local function valid(e) assert(type(e)=='number' and e>=1 and e<=12 and e%1==0); return S.players[e]~=nil end
  local function write(e) S.writes=S.writes+1; return valid(e) and g.match().active and not S.refuse end
  function g.fighter_caps(e,...)
    if select('#',...)==0 then
      if not valid(e) then return nil end
      return copy(S.caps[e])
    end
    local values=(...); if not write(e) then return false end
    local c={air_jumps=-1}; for _,key in ipairs(flags) do c[key]=false end
    for k,v in pairs(values or {}) do
      if k=='air_jumps' then assert(type(v)=='number' and v>=0 and v<=8 and v%1==0)
      else local found=false; for _,key in ipairs(flags) do if k==key then found=true end end; assert(found and type(v)=='boolean') end
      c[k]=v
    end
    local restricted=false; for _,key in ipairs(flags) do if c[key] then restricted=true end end
    S.caps[e]=(c.air_jumps>=0 or restricted) and c or nil; return true
  end
  function g.fighter_effect(e,kind,...)
    local found=false; for _,k in ipairs(kinds) do if k==kind then found=true end end; assert(found)
    if select('#',...)==0 then return valid(e) and copy(S.effects[e] and S.effects[e][kind]) or nil end
    local value,frames=...; assert(type(value)=='number' and value==value and value>=(kind=='size' and .25 or 0) and value<=(kind=='size' and 4 or 1))
    assert(type(frames)=='number' and frames>=0 and frames<=3600 and frames%1==0)
    if not write(e) then return false end
    S.effects[e]=S.effects[e] or {}; S.effects[e][kind]=frames>0 and {value=value,frames=frames} or nil; return true
  end
  function g.fighter_armour(e,...)
    if select('#',...)==0 then return valid(e) and copy(S.armour[e]) or nil end
    local a=(...); if not write(e) then return false end
    local values={damage=0,knockback=0}; for k,v in pairs(a or {}) do assert((k=='damage' or k=='knockback') and type(v)=='number' and v>=0 and v<=1000); values[k]=v end
    S.armour[e]=(values.damage>0 or values.knockback>0) and values or nil; return true
  end
  function g.give_item(e,kind)
    assert(kind=='random' or (type(kind)=='number' and kind>=0 and kind<=255 and kind%1==0))
    if not write(e) or S.held[e] then return false end
    S.held[e]=kind; return true
  end
  function g.opponents_in_radius(e,radius,exclude)
    assert(type(radius)=='number' and radius>=0 and radius<=100000); if exclude then valid(exclude) end; local ids={}
    if not valid(e) or not g.match().active then return ids end
    local p=S.players[e]
    for id,q in pairs(S.players) do
      if id~=e and id~=exclude and (p.team==nil or p.team~=q.team) and (q.x-p.x)^2+(q.y-p.y)^2<=radius^2 then ids[#ids+1]=id end
    end
    table.sort(ids); return ids
  end
  function g.nearest_opponent(e,exclude)
    local best,distance; for _,id in ipairs(g.opponents_in_radius(e,100000,exclude)) do
      local p,q=S.players[e],S.players[id]; local d=(q.x-p.x)^2+(q.y-p.y)^2
      if not distance or d<distance then best,distance=id,d end
    end; return best
  end
  function g.fighter_timed_status(e,channel,...)
    assert(channel>=1 and channel<=4 and channel%1==0)
    if select('#',...)==0 then return valid(e) and copy(S.statuses[e] and S.statuses[e][channel]) or nil end
    local value,frames=...; assert(type(value)=='number' and value==value and type(frames)=='number' and frames>=0 and frames<=3600 and frames%1==0)
    if not write(e) then return false end
    S.statuses[e]=S.statuses[e] or {}; S.statuses[e][channel]=frames>0 and {value=value,frames=frames} or nil; return true
  end
  local advance=g.advance
  function g.advance()
    advance()
    for _,collection in ipairs({S.effects,S.statuses}) do
      for _,values in pairs(collection) do
        for key,v in pairs(values) do v.frames=v.frames-1; if v.frames<=0 then values[key]=nil end end
      end
    end
  end
  return g,S
end
local count=0
for _,slug in ipairs({'air-jumps','restrictions','armour','give-item','targeting','effects'}) do
  for _,scenario in ipairs({'basic','inactive','missing','refusal'}) do
    local g,S=fixture()
    if scenario=='inactive' then S.active=false end
    if scenario=='missing' then S.players[1]=nil end
    if scenario=='refusal' then S.refuse=true end
    local env=setmetatable({gd=g},{__index=_G})
    assert(loadfile(base..slug..'/scripts/main.lua','t',env))()
    local function hook(name,...) if env[name] then env[name](...) end end
    local function frame() g.advance(); hook('on_frame'); hook('on_draw') end
    local function key(k) S.keys[k]=true; hook('on_tick'); hook('on_draw') end
    frame()
    if scenario=='inactive' or scenario=='missing' then
      assert(S.writes==0,'unavailable fighter attempted setup'); hook('on_tick'); hook('on_unload')
    elseif scenario=='refusal' then
      assert(S.logs[1]=='Initial setup refused','refusal misreported'); hook('on_unload')
    else
      assert(S.logs[1]=='Initial setup applied','automatic setup missing')
      local writes=S.writes; frame(); assert(S.writes==writes,'first-frame setup repeated')
      if slug=='air-jumps' then
        assert(g.fighter_caps(1).air_jumps==3); key('J'); assert(g.fighter_caps(1).air_jumps==8)
        key('R'); assert(g.fighter_caps(1)==nil)
      elseif slug=='restrictions' then
        assert(g.fighter_caps(1).shield==true)
        for _,name in ipairs({'air_dodge','run','grab','specials','shield'}) do
          key('F'); local c=g.fighter_caps(1)
          for _,flag in ipairs(flags) do assert(c[flag]==(flag==name),'restriction polarity/cycle wrong') end
        end
        key('R'); assert(g.fighter_caps(1)==nil)
      elseif slug=='armour' then
        assert(g.fighter_armour(1).damage==12); key('A'); assert(g.fighter_armour(1).knockback==60 and g.fighter_armour(1).damage==0)
        key('R'); assert(g.fighter_armour(1)==nil)
      elseif slug=='give-item' then
        assert(S.held[1]=='random'); key('I'); S.commands.demo_state('occupied')
        assert(S.logs[#S.logs]:find('Refused',1,true),'occupied hands misreported')
        S.held[1]=nil; key('I'); assert(S.held[1]=='random')
      elseif slug=='targeting' then
        assert(g.nearest_opponent(1)==2 and #g.opponents_in_radius(1,80)==1)
        assert(g.nearest_opponent(1,2)==3 and #g.opponents_in_radius(1,80,2)==0,'optional exclusion failed')
        local v=assert(g.fighter_timed_status(2,1)); assert(v.value==7 and v.frames==119)
        for _=1,121 do frame() end; assert(g.fighter_timed_status(2,1)==nil,'status did not expire')
        key('T'); assert(g.fighter_timed_status(2,1).frames==120)
        local opponent=S.players[2]
        for id=2,6 do S.players[id]=nil end; key('T'); hook('on_draw')
        S.players[2]=opponent
      elseif slug=='effects' then
        assert(g.fighter_effect(1,'intangible').frames==179)
        for _,kind in ipairs({'invincible','metal','size'}) do key('E'); local e=assert(g.fighter_effect(1,kind)); assert(e.frames==180 and e.value==(kind=='size' and 1.5 or 1)) end
        key('R'); assert(g.fighter_effect(1,'size')==nil)
        for _=1,181 do frame() end
        for _,kind in ipairs(kinds) do assert(g.fighter_effect(1,kind)==nil,'effect did not expire') end
        key('E')
      end
      hook('on_unload')
      assert(g.fighter_caps(1)==nil,'movement owner record survived unload')
      assert(g.fighter_armour(1)==nil,'armour owner record survived unload')
      for _,kind in ipairs(kinds) do assert(g.fighter_effect(1,kind)==nil,'effect survived unload') end
      for _,values in pairs(S.statuses) do assert(next(values)==nil,'status survived unload') end
    end
    count=count+1
  end
end
print('PASS: '..count..' scoped fighter capability demo stub scenarios; native gameplay and rewind untested')
