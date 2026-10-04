-- Offline LAB: choose Mario P1 and any P2 on Final Destination.
-- Rules are installed ahead of combat. on_hit only reads the native result.
local active,started=false,false
local pending={}
local hits,last=0,'Use Mario up-smash, then an aerial on P2'
local function install(value)
 if not gd.match().active or gd.match().netplay then return end
 gd.hit_rules_clear(1)
 gd.fighter_status(1,0)
 if value then
  gd.hit_rule_add(1,{id=101,match={move='smash'},change={element='fire'}})
  gd.hit_rule_add(1,{id=102,match={move='aerial'},change={element='electric'}})
 end
 active=value
end
gd.command('demo_key',function(key) pending[key]=true end,'demo_key H: toggle native conversions')
gd.command('demo_state',function(token)
 gd.log('tour state '..token..' active='..tostring(active)..' rules='..#gd.hit_rules(1)..' hits='..hits..' acceptance=operator-required')
end,'demo_state <token>: query real native table; collisions require actual attacks')
function on_match_start() if not started then install(true);started=true end end
function on_frame() if gd.match().active then on_match_start() end end
function on_tick()
 if pending.H or gd.key_pressed('H') then pending.H=nil;install(not active) end
end
function on_hit(attacker,victim,e)
 if attacker~=1 then return end
 local own=gd.hit_rules(1).owner
 local ids={}
 for i,id in ipairs(e.hit_rule_ids or {}) do
  if (e.hit_rule_owners or {})[i]==own then ids[#ids+1]=tostring(id) end
 end
 hits=hits+1
 last=(e.move_tag or 'unknown')..' / '..(e.element_tag or 'unknown')..' / applied '..table.concat(ids,',')
 gd.log('native hit rules: '..last)
end
function on_draw()
 gd.text(20,20,'Native hit rules: H toggles; Mario P1 up-smash = Fire, aerial = Electric')
 gd.text(20,42,last)
 gd.text(20,64,'Retail effects and electric hitlag; original move sound remains')
end
function on_match_end() started=false;active=false end
-- Unload and scene cleanup are native; existing capsules keep their creation values.
