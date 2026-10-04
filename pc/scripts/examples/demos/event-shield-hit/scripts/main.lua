local count=0; local last="Waiting for on_shield_hit"
function on_shield_hit(p,m,s) count=count+1; last="on_shield_hit #"..count..": "..tostring(p).." action="..tostring(m).." sub="..tostring(s); gd.log(last) end
function on_draw() gd.text(20,20,last) end

gd.command("demo_state",function(token) gd.log("tour state "..token.." contract=on_shield_hit hook="..tostring(type(on_shield_hit)=="function").." captured="..count.." acceptance=operator-required") end,"demo_state <token>: hook admission; operator must generate and verify real event")
