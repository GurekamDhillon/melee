local enabled=false
local request=false
gd.command("demo_key",function(key) if key=="M" then request=true end end,"demo_key M: toggle the demonstrated multiplier")
function on_tick()
 if gd.match().active and (request or gd.key_pressed("M")) then
  request=false; enabled=not enabled; gd.fighter_mod(1,enabled and {shield_regen=2} or nil)
 end
end
function on_draw() gd.text(20,20,"M: shield_regen x2 / baseline; "..tostring(enabled)) end
function on_unload() if gd.match().active then gd.fighter_mod(1,nil) end end

gd.command("demo_state",function(token) local q=gd.fighter_mod(1); gd.log("tour state "..token.." "..tostring(enabled).." shield_regen="..tostring(q and q.shield_regen or 1)) end,"demo_state <token>: read the demonstrated live multiplier")
