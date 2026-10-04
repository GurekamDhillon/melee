local count=0; local last="Waiting for on_ledge_grab"
function on_ledge_grab(p,m,s) count=count+1; last="on_ledge_grab #"..count..": "..tostring(p).." action="..tostring(m).." sub="..tostring(s); gd.log(last) end
function on_draw() gd.text(20,20,last) end

gd.command("demo_state",function(token) gd.log("tour state "..token.." contract=on_ledge_grab hook="..tostring(type(on_ledge_grab)=="function").." captured="..count.." acceptance=operator-required") end,"demo_state <token>: hook admission; operator must generate and verify real event")
