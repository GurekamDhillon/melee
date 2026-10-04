local count=0; local last="Waiting for on_taunt"
function on_taunt(p,m,s) count=count+1; last="on_taunt #"..count..": "..tostring(p).." action="..tostring(m).." sub="..tostring(s); gd.log(last) end
function on_draw() gd.text(20,20,last) end

gd.command("demo_state",function(token) gd.log("tour state "..token.." contract=on_taunt hook="..tostring(type(on_taunt)=="function").." captured="..count.." acceptance=operator-required") end,"demo_state <token>: hook admission; operator must generate and verify real event")
