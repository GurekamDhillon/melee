local count=0; local last="Waiting for on_ko"
function on_ko(a,v) count=count+1; last="on_ko #"..count..": "..tostring(a).." > "..tostring(v); gd.log(last) end
function on_draw() gd.text(20,20,last) end

gd.command("demo_state",function(token) gd.log("tour state "..token.." contract=on_ko hook="..tostring(type(on_ko)=="function").." captured="..count.." acceptance=operator-required") end,"demo_state <token>: hook admission; operator must generate and verify real event")
