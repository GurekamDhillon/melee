-- One capability: native per-spawn modifiers/tint for enemy CPU ports.
local ports={}
function on_1p_stage_start(e)
 ports={}
 for _,p in ipairs(e.opponents) do
  if gd.spawn_1p(p.port,{run_speed=1.15,damage_dealt=1.05,tint=0xff8800ff}) then ports[#ports+1]=p.port end
 end
end
function on_1p_spawn(e) gd.log('1p spawn port '..e.port..' entity '..e.entity) end
local function clear() for _,p in ipairs(ports) do gd.spawn_1p(p,nil) end;ports={} end
function on_1p_stage_clear() clear() end
function on_1p_game_over() clear() end
function on_unload() clear() end
function on_draw() gd.kit.text('Orange enemies: native spawn speed +15%, damage +5%',24,28,14,'bone') end
