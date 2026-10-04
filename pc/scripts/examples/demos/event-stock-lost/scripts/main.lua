local count=0; local last="Waiting for on_stock_lost"
function on_stock_lost(p,n) count=count+1; last="on_stock_lost #"..count..": "..tostring(p).." stocks="..tostring(n); gd.log(last) end
function on_draw() gd.text(20,20,last) end

gd.command("demo_state",function(token) gd.log("tour state "..token.." contract=on_stock_lost hook="..tostring(type(on_stock_lost)=="function").." captured="..count.." acceptance=operator-required") end,"demo_state <token>: hook admission; operator must generate and verify real event")
