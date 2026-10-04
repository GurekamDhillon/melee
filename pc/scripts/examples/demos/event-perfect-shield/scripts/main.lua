local count=0; local last="Waiting for on_perfect_shield"
function on_perfect_shield(p,m,s) count=count+1; last="on_perfect_shield #"..count..": "..tostring(p).." action="..tostring(m).." sub="..tostring(s); gd.log(last) end
function on_draw() gd.text(20,20,last) end

gd.command("demo_state",function(token) gd.log("tour state "..token.." contract=on_perfect_shield hook="..tostring(type(on_perfect_shield)=="function").." captured="..count.." acceptance=operator-required") end,"demo_state <token>: hook admission; operator must generate and verify real event")
